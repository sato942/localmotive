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

const readManifest = (path) => {
  const record = JSON.parse(readFileSync(path, "utf8"));
  return { path, terminalOutcome: record.terminalOutcome ?? null, workload: record.workload ?? null };
};

const newestManifest = () => {
  try {
    const candidates = readdirSync(BENCH_DIR)
      .filter((name) => name.endsWith(".json"))
      .map((name) => join(BENCH_DIR, name))
      .map((path) => ({ path, mtime: statSync(path).mtimeMs }))
      .sort((a, b) => b.mtime - a.mtime);
    return candidates.length ? readManifest(candidates[0].path) : null;
  } catch {
    return null;
  }
};

const recordsSince = (sinceMs) => {
  try {
    return readdirSync(BENCH_DIR)
      .filter((name) => name.endsWith(".json"))
      .map((name) => join(BENCH_DIR, name))
      .filter((path) => statSync(path).mtimeMs >= sinceMs);
  } catch {
    return [];
  }
};

/// Set a React-controlled numeric input through the native setter so the
/// component's onChange observes the change (used to widen the final cycle's
/// generation window so the previous request is provably active).
const setNumberInput = (label, value) =>
  evaluate(`(() => {
    const input = [...document.querySelectorAll("input")].find(
      (candidate) => candidate.getAttribute("aria-label") === ${JSON.stringify(label)},
    );
    if (!input) return "absent";
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
    setter.call(input, ${JSON.stringify(String(value))});
    input.dispatchEvent(new Event("input", { bubbles: true }));
    return input.value;
  })()`);

/// Direct IPC attempt at the replacement run. This proves the BACKEND refuses
/// replacement work while the previous run still owns the machine - it does
/// not depend on the UI disabling its buttons.
const invokeBenchmarkAttempt = (workload) =>
  evaluate(`(async () => {
    const internals = window.__TAURI_INTERNALS__;
    if (!internals || typeof internals.invoke !== "function") {
      return { status: "unavailable" };
    }
    const started = Date.now();
    const attempt = internals.invoke("benchmark_v2", ${JSON.stringify({ workload })});
    const outcome = await Promise.race([
      attempt.then(() => ({ status: "accepted" }), (error) => ({ status: "refused", error: String(error) })),
      new Promise((resolvePromise) => setTimeout(() => resolvePromise({ status: "pending" }), 2500)),
    ]);
    return { ...outcome, elapsedMs: Date.now() - started };
  })()`);

/// R16: attempt the replacement run at the earliest point the UI permits any
/// action after a cancel. The server metric is read at entry - before the UI
/// navigation costs any time - and again at the click instant; the direct IPC
/// attempt is the authoritative refusal probe.
const attemptImmediateRestart = async ({ cancelAt, processingAtCancelRequest } = {}) => {
  const processingAtEntry = await metricsField("llamacpp:requests_processing");
  await clickExact("Benchmark");
  await settle(600);
  const workload = newestManifest()?.workload ?? null;
  const processingAtAttempt = await metricsField("llamacpp:requests_processing");
  const buttonStateAtAttempt = await buttonState("Run v2 benchmark");
  const uiAccepted = await clickExact("Run v2 benchmark");
  const apiAttempt = workload ? await invokeBenchmarkAttempt(workload) : { status: "no-workload" };
  const notice = await lastNotice();
  return {
    at: Date.now(),
    cancelAt: cancelAt ?? null,
    sinceCancelMs: cancelAt ? Date.now() - cancelAt : null,
    processingAtCancelRequest: processingAtCancelRequest ?? null,
    processingAtEntry,
    processingAtAttempt,
    buttonStateAtAttempt,
    uiAccepted,
    apiAttempt,
    notice: String(notice ?? "").slice(0, 160),
  };
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
  // A wider generation window for every cycle: the server must still be
  // generating when the harness observes it, or the cancel lands on an
  // already-finished request (seen live: one cycle missed its cancel and the
  // in-flight coordination could not issue one at all).
  const generatedSet = await setNumberInput("Benchmark generated tokens", 1024);
  check(`mt06.cycle-${cycle}-window-workload-set`, String(generatedSet) === "1024", `input=${generatedSet}`);
  if (cycle === CYCLES) {
    // The final cycle proves the restart attempt happens while the previous
    // request is still active, so it keeps the widest window.
    await settle(800);
  }
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
  const processingAtCancelRequest = await metricsField("llamacpp:requests_processing");
  const cancelled = accepted ? await clickExact("Cancel") : false;
  const cancelAt = Date.now();
  // R16: attempt the replacement at the earliest point the UI permits any
  // action after the cancel - before the settle/drain waits. The previous
  // harness waited for the run to settle (and the server to drain) first,
  // which made its "immediate restart" claim vacuous.
  const finalAttempt =
    cycle === CYCLES
      ? await attemptImmediateRestart({ cancelAt, processingAtCancelRequest })
      : null;
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
  // R16: while the final cycle's abandoned request is still active, no
  // replacement work may overlap it: every drain sample counts server
  // requests and app-owned servers.
  let windowOverlap = false;
  let windowMaxProcessing = Number.isFinite(Number(processing)) ? Number(processing) : 0;
  let windowMaxChildren = 0;
  for (let attempt = 0; attempt < 600 && !drained; attempt += 1) {
    await settle(500);
    processing = await metricsField("llamacpp:requests_processing");
    drained = processing === "0";
    if (finalAttempt) {
      const numeric = Number(processing);
      if (Number.isFinite(numeric)) windowMaxProcessing = Math.max(windowMaxProcessing, numeric);
      if (processing === "2" || processing === "3") windowOverlap = true;
      const childrenNow = ownedServerPids();
      windowMaxChildren = Math.max(windowMaxChildren, childrenNow.length);
      if (childrenNow.length > 1) windowOverlap = true;
    }
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
  // Seen live on the final cycle: a cancel that lands BETWEEN attempts ends
  // the run without a terminal outcome in its record. That is a real outcome
  // of the coordination, not a pass, so the harness runs one more measured
  // attempt and cancels it while its first trial is in flight, then asserts
  // the persisted terminal record of THAT attempt. The retry is recorded.
  if (record && !record.terminalOutcome) {
    const retryStart = Date.now();
    const tokensBeforeRetry = await metricsField("llamacpp:tokens_predicted_total");
    const retryStarted = await clickExact("Run v2 benchmark");
    let inFlightRetry = false;
    for (let attempt = 0; attempt < 90 && !inFlightRetry; attempt += 1) {
      await settle(1000);
      const processingRetry = await metricsField("llamacpp:requests_processing");
      const tokensNow = await metricsField("llamacpp:tokens_predicted_total");
      const generatedRetry =
        Number.isFinite(Number(tokensNow)) && Number.isFinite(Number(tokensBeforeRetry))
          ? Number(tokensNow) - Number(tokensBeforeRetry)
          : 0;
      if (processingRetry === "1" && generatedRetry >= 1) inFlightRetry = true;
    }
    const cancelledRetry = inFlightRetry ? await clickExact("Cancel") : false;
    let recordRetry = record;
    for (let attempt = 0; attempt < 60; attempt += 1) {
      await settle(500);
      const candidate = newestBenchmarkRecord(retryStart);
      if (candidate?.terminalOutcome) {
        recordRetry = candidate;
        break;
      }
    }
    run.terminalOutcomeRetry = {
      attempted: Boolean(retryStarted),
      inFlight: inFlightRetry,
      cancelled: cancelledRetry,
      outcome: recordRetry?.terminalOutcome ?? null,
      path: recordRetry?.path ?? null,
    };
    record = recordRetry;
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
    // R16: immediate-restart proof. The replacement attempt already ran (see
    // attemptImmediateRestart above) while the previous request was still
    // active. This block asserts the refusal, the overlap-free window, and
    // that the replacement run is permitted and recorded once cleanup ends.
    const observedActive =
      finalAttempt?.processingAtCancelRequest === "1" || finalAttempt?.processingAtEntry === "1";
    check(
      "mt06.immediate-restart-attempted-while-previous-active",
      observedActive &&
        Number.isFinite(Number(finalAttempt?.sinceCancelMs)) &&
        Number(finalAttempt?.sinceCancelMs) <= 3000,
      `processingAtCancel=${finalAttempt?.processingAtCancelRequest} processingAtEntry=${finalAttempt?.processingAtEntry} processingAtClick=${finalAttempt?.processingAtAttempt} sinceCancelMs=${finalAttempt?.sinceCancelMs}`,
    );
    const refusalMessage = String(finalAttempt?.apiAttempt?.error ?? "");
    // Refused means: the direct IPC attempt was rejected, or the UI refused /
    // disabled the action. The API verdict is the authoritative one; the UI
    // verdict and the notice are recorded as supporting evidence.
    check(
      "mt06.immediate-restart-refused-while-previous-active",
      finalAttempt?.apiAttempt?.status === "refused" ||
        finalAttempt?.uiAccepted === false ||
        finalAttempt?.buttonStateAtAttempt === "disabled",
      `api=${finalAttempt?.apiAttempt?.status} uiAccepted=${finalAttempt?.uiAccepted} uiState=${finalAttempt?.buttonStateAtAttempt} notice=${finalAttempt?.notice?.slice(0, 60)}${refusalMessage ? ` msg=${refusalMessage.slice(0, 100)}` : ""}`,
    );
    check(
      "mt06.final-no-overlap-before-drain",
      !windowOverlap && windowMaxProcessing <= 1 && windowMaxChildren <= 1,
      `maxProcessing=${windowMaxProcessing} maxChildren=${windowMaxChildren}`,
    );
    check(
      "mt06.final-previous-request-drained",
      drained,
      `drainAfterAttemptMs=${finalAttempt ? drainStart - finalAttempt.at + drainMs : drainMs}`,
    );
    run.finalAttempt = finalAttempt;

    // After cleanup completes the UI must permit the replacement run: refused
    // or serialized, never blocked forever.
    let restarted = false;
    for (let attempt = 0; attempt < 60 && !restarted; attempt += 1) {
      if ((await buttonState("Run v2 benchmark")) === "enabled") {
        restarted = await clickExact("Run v2 benchmark");
        break;
      }
      await settle(1000);
    }
    const restartAt = Date.now();
    let restartAccepted = false;
    let overlap = windowOverlap;
    while (Date.now() - restartAt < 120_000 && !restartAccepted) {
      const current = await metricsField("llamacpp:requests_processing");
      if (current === "2" || current === "3") overlap = true;
      if (ownedServerPids().length > 1) overlap = true;
      if (current === "1" && restarted) restartAccepted = true;
      await settle(400);
    }
    check(
      "mt06.final-restart-after-cleanup",
      restarted && restartAccepted,
      `restarted=${restarted} accepted=${restartAccepted}`,
    );
    check("mt06.immediate-restart-no-overlap", !overlap, overlap ? "two concurrent requests or servers observed" : "");

    // The replacement run persists its own terminal record: the cancelled
    // record from the previous run must not be the only evidence of the cycle.
    await clickExact("Cancel");
    let replacementRecord = null;
    for (let attempt = 0; attempt < 240 && !replacementRecord; attempt += 1) {
      await settle(500);
      const candidates = recordsSince(restartAt)
        .map((path) => {
          try {
            return readManifest(path);
          } catch {
            return null;
          }
        })
        .filter(Boolean)
        .filter((record) => record.terminalOutcome === "cancelled");
      if (candidates.length > 0) replacementRecord = candidates[candidates.length - 1];
    }
    if (replacementRecord) run.replacementRecordPath = replacementRecord.path;
    check(
      "mt06.final-replacement-terminal-record",
      Boolean(replacementRecord),
      `record=${replacementRecord?.path ?? "none"}`,
    );
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
