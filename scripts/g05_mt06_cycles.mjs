// MT-06 cancellation driver, R11 (follow-up review db548c8): the packaged
// cancellation campaign measures APP-OWNED process identities instead of
// process-name counts, coordinates each cancel with real server-side request
// acceptance, asserts the persisted terminal record, bounds the cleanup
// latency, and exercises an immediate restart to prove no overlapping
// inference is possible while the previous run drains.
//
// Usage: node scripts/g05_mt06_cycles.mjs <debugPort> [cycles] <appPid> [sourceRevision] [portableDigest]
// appPid is REQUIRED: the owned-process checks are parent-scoped to it.
import { attach } from "./lib/cdp_client.mjs";
import { execSync } from "node:child_process";
import { readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { connect } from "node:net";
import https from "node:https";

const [portArg, cyclesArg, appPidArg, sourceRevisionArg, portableDigestArg] = process.argv.slice(2);
const CYCLES = Number(cyclesArg ?? 6);
const APP_PID = Number(appPidArg);
if (!Number.isInteger(APP_PID) || APP_PID <= 0) {
  console.error("usage: g05_mt06_cycles.mjs <debugPort> [cycles] <appPid> [sourceRevision] [portableDigest]");
  process.exit(2);
}
const EVIDENCE_PATH = process.env.MT06_EVIDENCE_PATH ?? join(process.cwd(), ".hermes-0.6", "mt06-cycles-result.json");
const BENCH_DIR = join(process.env.APPDATA ?? "", "io.github.localmotive.app", "benchmarks");
const RUN_SETTLE_BOUND_MS = 60_000;
const DRAIN_BOUND_MS = 300_000;
const API_KEY = readFileSync(join(process.cwd(), ".hermes-0.6", "g05-tls", "api-key.txt"), "utf8").trim();

const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolvePromise) => setTimeout(resolvePromise, ms));

const clickExact = (label) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === ${JSON.stringify(label)} && !candidate.disabled,
    );
    if (!button) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);

const buttonState = (label) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === ${JSON.stringify(label)},
    );
    if (!button) return "absent";
    return button.disabled ? "disabled" : "enabled";
  })()`);

const lastNotice = () =>
  evaluate(`(() => {
    const lines = [...document.querySelectorAll(".notice-line")].map((node) =>
      (node.textContent ?? "").replace(/\\s+/g, " ").trim(),
    ).filter(Boolean);
    return lines.length ? lines[lines.length - 1].slice(0, 240) : null;
  })()`);

// --- Owned-process measurement (R11) ----------------------------------------
// The app owns its server as a child process. Counts are scoped to the app's
// PID via Win32_Process parentage; an enumeration failure THROWS (it must
// never be reported as zero children, which would fake a clean stop).
const ownedServerPids = () => {
  const command = `powershell -NoProfile -Command "(Get-CimInstance Win32_Process -Filter \\"Name='llama-server.exe'\\" | Where-Object { $_.ParentProcessId -eq ${APP_PID} } | Select-Object -ExpandProperty ProcessId) -join ','"`;
  const output = execSync(command, { encoding: "utf8", windowsHide: true, timeout: 20_000 });
  const trimmed = output.trim();
  if (trimmed === "" || trimmed === "0") return [];
  return trimmed.split(",").map((value) => Number(value.trim())).filter((value) => Number.isInteger(value));
};

const stopServer = async () => {
  for (let attempt = 0; attempt < 3; attempt += 1) {
    await clickExact("Control");
    await settle(1200);
    const stopped = await clickExact("Stop server");
    if (!stopped) await clickExact("Cancel start");
    let zero = false;
    for (let wait = 0; wait < 12 && !zero; wait += 1) {
      await settle(1000);
      try {
        zero = ownedServerPids().length === 0;
      } catch {
        zero = false;
      }
    }
    if (zero) return true;
  }
  return ownedServerPids().length === 0;
};

const metricsField = (name) =>
  new Promise((resolvePromise) => {
    const request = https.request(
      {
        host: "127.0.0.1",
        port: 8080,
        path: "/metrics",
        method: "GET",
        headers: { Authorization: `Bearer ${API_KEY}` },
        rejectUnauthorized: false,
        timeout: 4000,
      },
      (response) => {
        let data = "";
        response.on("data", (chunk) => {
          data += chunk;
        });
        response.on("end", () => {
          const match = data.match(new RegExp(`^${name}\\s+(\\S+)$`, "m"));
          resolvePromise(match ? match[1] : null);
        });
      },
    );
    request.on("error", (error) => resolvePromise(`ERR:${error.code ?? error.message}`));
    request.on("timeout", () => {
      request.destroy();
      resolvePromise("ERR:timeout");
    });
    request.end();
  });

const listenerAlive = () =>
  new Promise((resolvePromise) => {
    const socket = connect({ host: "127.0.0.1", port: 8080 });
    const done = (value) => {
      socket.destroy();
      resolvePromise(value);
    };
    socket.on("connect", () => done(true));
    socket.on("error", () => done(false));
    setTimeout(() => done(false), 2000);
  });

const newestBenchmarkRecord = (sinceMs) => {
  try {
    const candidates = readdirSync(BENCH_DIR)
      .filter((name) => name.endsWith(".json"))
      .map((name) => join(BENCH_DIR, name))
      .map((path) => ({ path, mtime: statSync(path).mtimeMs }))
      .filter((entry) => entry.mtime >= sinceMs)
      .sort((a, b) => b.mtime - a.mtime);
    if (candidates.length === 0) return null;
    const record = JSON.parse(readFileSync(candidates[0].path, "utf8"));
    return { path: candidates[0].path, terminalOutcome: record.terminalOutcome ?? null };
  } catch (error) {
    return { path: null, terminalOutcome: `ERR:${error.message}` };
  }
};

const checks = [];
const check = (name, ok, detail = "") => {
  checks.push({ name, ok, detail });
  console.log(`${ok ? "PASS" : "FAIL"} ${name}${detail ? ` | ${detail}` : ""}`);
};

const startedAtMs = Date.now();
const run = {
  schema: "localmotive.mt06-cancellation.v1",
  startedAtUtc: new Date().toISOString(),
  sourceRevision: sourceRevisionArg ?? null,
  portableDigest: portableDigestArg ?? null,
  appPid: APP_PID,
  cycles: [],
};

// --- Preconditions ----------------------------------------------------------
await clickExact("Profile");
await settle(1200);
let live = await evaluate(`/Stop server/.test(document.body.textContent || "")`);
if (!live) {
  await clickExact("Start");
  for (let attempt = 0; attempt < 60 && !live; attempt += 1) {
    await settle(1500);
    live = await evaluate(`/Stop server/.test(document.body.textContent || "")`);
  }
}
check("mt06.precondition-server-live", live, live ? "" : `notice=${await lastNotice()}`);
if (!live) {
  console.log("MT06 SUMMARY: server did not start");
  writeFileSync(EVIDENCE_PATH, JSON.stringify({ ...run, checks, summary: "server did not start" }, null, 2));
  process.exit(1);
}
let ownedServers = [];
try {
  for (let attempt = 0; attempt < 20 && ownedServers.length === 0; attempt += 1) {
    ownedServers = ownedServerPids();
    if (ownedServers.length === 0) await settle(1000);
  }
} catch (error) {
  check("mt06.owned-pid-enumeration", false, String(error.message).slice(0, 160));
  writeFileSync(EVIDENCE_PATH, JSON.stringify({ ...run, checks, summary: "enumeration failed" }, null, 2));
  process.exit(1);
}
check("mt06.owned-pid-enumeration", ownedServers.length === 1, `pids=${ownedServers.join(",")}`);
if (ownedServers.length !== 1) {
  // Running cycles without known ownership would fake every child check.
  console.log("MT06 SUMMARY: owned server process was not found for this app PID");
  writeFileSync(EVIDENCE_PATH, JSON.stringify({ ...run, checks, summary: "owned pid not found" }, null, 2));
  process.exit(1);
}
let currentOwnedServerPid = ownedServers[0];
await clickExact("Benchmark");
await settle(1200);
await clickExact("Inspect artifact");
await settle(2500);
await clickExact("Run preflight");
await settle(4000);

// --- Cycles ----------------------------------------------------------------
for (let cycle = 1; cycle <= CYCLES; cycle += 1) {
  const cycleStart = Date.now();
  const tokensBefore = await metricsField("llamacpp:tokens_predicted_total");
  await clickExact("Benchmark");
  await settle(1200);
  const started = await clickExact("Run v2 benchmark");
  let inFlight = false;
  let accepted = false;
  for (let attempt = 0; attempt < 90 && !accepted; attempt += 1) {
    await settle(1000);
    const processing = await metricsField("llamacpp:requests_processing");
    const tokensNow = await metricsField("llamacpp:tokens_predicted_total");
    // R11: cancel only AFTER the server reports the request in flight AND
    // has actually generated tokens for it. A pre-trial cancellation
    // (prompt preparation) aborts before the manifest exists by design and
    // would make the persisted-record check meaningless.
    const generatedTokens =
      Number.isFinite(Number(tokensNow)) && Number.isFinite(Number(tokensBefore))
        ? Number(tokensNow) - Number(tokensBefore)
        : 0;
    if (processing === "1" && generatedTokens >= 1) {
      inFlight = (await buttonState("Cancel")) === "enabled";
      accepted = inFlight;
    }
  }
  const cancelled = accepted ? await clickExact("Cancel") : false;
  const cancelAt = Date.now();
  let settled = false;
  let notice = null;
  let runEndedMs = null;
  for (let attempt = 0; attempt < 120 && !settled; attempt += 1) {
    await settle(500);
    notice = await lastNotice();
    const cancelState = await buttonState("Cancel");
    const panelFinished = await evaluate(`/Benchmark finished|No benchmark is running/i.test(document.body.innerText || "")`);
    if (cancelState !== "enabled" || panelFinished) {
      settled = true;
      runEndedMs = Date.now() - cancelAt;
    }
    if (Date.now() - cancelAt > RUN_SETTLE_BOUND_MS + 90_000) break;
  }
  let processing = await metricsField("llamacpp:requests_processing");
  const drainStart = Date.now();
  let drained = processing === "0";
  for (let attempt = 0; attempt < 600 && !drained; attempt += 1) {
    await settle(500);
    processing = await metricsField("llamacpp:requests_processing");
    drained = processing === "0";
    if (Date.now() - drainStart > DRAIN_BOUND_MS) break;
  }
  const drainMs = Date.now() - drainStart;
  // The manifest is persisted at the very end of the run; the odd cycles can
  // reach this check before the write lands, so poll briefly for it.
  let record = newestBenchmarkRecord(cycleStart);
  for (let attempt = 0; attempt < 30 && !record; attempt += 1) {
    await settle(500);
    record = newestBenchmarkRecord(cycleStart);
  }
  let children = null;
  let childError = null;
  try {
    children = ownedServerPids();
  } catch (error) {
    childError = String(error.message).slice(0, 160);
  }
  const entry = {
    cycle,
    started,
    inFlight,
    accepted,
    cancelled,
    settled,
    runEndedMs,
    drained,
    drainMs,
    processing,
    ownedChildren: children,
    childError,
    terminalOutcome: record?.terminalOutcome ?? null,
    recordPath: record?.path ?? null,
    notice: String(notice ?? "").slice(0, 160),
  };
  run.cycles.push(entry);
  console.log(
    `CYCLE ${cycle}: accepted=${accepted} cancelled=${cancelled} settled=${settled}(${runEndedMs ?? "-"}ms) drained=${drained}(${drainMs}ms) record=${entry.terminalOutcome} children=${children?.join(",") ?? childError}`,
  );
  check(`mt06.cycle-${cycle}-cancel-in-flight`, started && inFlight && accepted && cancelled);
  check(`mt06.cycle-${cycle}-run-ended-bounded`, settled && runEndedMs !== null && runEndedMs <= RUN_SETTLE_BOUND_MS, `runEndedMs=${runEndedMs}`);
  check(`mt06.cycle-${cycle}-requests-drained`, drained, `requests_processing=${processing} drainMs=${drainMs}`);
  check(`mt06.cycle-${cycle}-cleanup-bounded`, drained && drainMs <= DRAIN_BOUND_MS, `drainMs=${drainMs}`);
  check(
    `mt06.cycle-${cycle}-persisted-terminal-outcome`,
    entry.terminalOutcome === "cancelled",
    `record=${record?.path ?? "none"} outcome=${entry.terminalOutcome}`,
  );
  check(
    `mt06.cycle-${cycle}-owned-children-exactly-one`,
    childError === null && children?.length === 1 && children[0] === currentOwnedServerPid,
    childError ?? `children=${children?.join(",")}`,
  );

  if (cycle === CYCLES) {
    // R11: immediate-restart overlap proof. Start a new run the moment the
    // previous run reports ended; its requests must only begin after the old
    // request drained, and at no sample may two server requests overlap.
    await clickExact("Benchmark");
    await settle(600);
    const restarted = await clickExact("Run v2 benchmark");
    let overlap = false;
    let restartAccepted = false;
    const restartAt = Date.now();
    while (Date.now() - restartAt < 120_000 && !restartAccepted) {
      const current = await metricsField("llamacpp:requests_processing");
      if (current === "2" || current === "3") overlap = true;
      if (current === "1" && restarted) restartAccepted = true;
      const childrenNow = ownedServerPids();
      if (childrenNow.length > 1) overlap = true;
      await settle(500);
    }
    check("mt06.immediate-restart-starts", restarted && restartAccepted);
    check("mt06.immediate-restart-no-overlap", !overlap, overlap ? "two concurrent requests or servers observed" : "");
    await clickExact("Cancel");
    for (let attempt = 0; attempt < 240; attempt += 1) {
      await settle(500);
      if ((await metricsField("llamacpp:requests_processing")) === "0") break;
    }
  } else if (cycle % 2 === 0) {
    const zero = await stopServer();
    check(`mt06.cycle-${cycle}-stop-clears-owned-children`, zero, `children=${ownedServerPids().join(",")}`);
    check(`mt06.cycle-${cycle}-listener-released`, !(await listenerAlive()));
    await settle(1500);
    await clickExact("Control");
    await settle(800);
    await clickExact("Start profile");
    let back = false;
    for (let attempt = 0; attempt < 60 && !back; attempt += 1) {
      await settle(1500);
      back = await evaluate(`/Stop server/.test(document.body.textContent || "")`);
    }
    check(`mt06.cycle-${cycle}-restart-reaches-live`, back);
    if (back) {
      // The deliberate stop/restart starts a NEW server process; track the
      // current owned identity instead of the one captured at precondition.
      for (let attempt = 0; attempt < 20; attempt += 1) {
        const refreshed = ownedServerPids();
        if (refreshed.length === 1) {
          currentOwnedServerPid = refreshed[0];
          break;
        }
        await settle(500);
      }
    }
  }
}

// --- Final teardown ---------------------------------------------------------
const finalZero = await stopServer();
let finalChildren = [];
try {
  finalChildren = ownedServerPids();
} catch (error) {
  check("mt06.final-owned-pid-enumeration", false, String(error.message).slice(0, 160));
}
check("mt06.final-stop-clears-owned-children", finalZero && finalChildren.length === 0, `children=${finalChildren.join(",")}`);
check("mt06.final-listener-released", !(await listenerAlive()));

const failed = checks.filter((entry) => !entry.ok);
run.checks = checks;
run.finishedAtUtc = new Date().toISOString();
run.totalMs = Date.now() - startedAtMs;
run.summary = `${checks.length - failed.length}/${checks.length} checks PASS`;
writeFileSync(EVIDENCE_PATH, JSON.stringify(run, null, 2));
console.log(`MT06 SUMMARY: ${run.summary}`);
console.log(`MT06 EVIDENCE: ${EVIDENCE_PATH}`);
process.exit(failed.length === 0 ? 0 : 1);
