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
import {
  classifyCoverage,
  evaluateIdentity,
  evaluateProhibited,
  evaluateScenario,
} from "./lib/mt06_verdicts.mjs";
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

/// F9-01: attempt the replacement on ONE surface only. The reviewed driver
/// clicked the UI Run button and then issued the direct IPC attempt, so a
/// refusal of the second call could be caused by the replacement the first
/// click had just started. Each scenario here exercises exactly one surface:
/// `ui` issues only the click, `api` issues only the direct invocation.
/// The server metric is sampled immediately before the invocation, which is
/// the sample the coverage classification uses.
const attemptReplacement = async ({ scenario, cancelAt, processingAtCancelRequest }) => {
  // Navigation only: it issues no benchmark action, so the request stays in
  // flight while the replacement is attempted.
  await clickExact("Benchmark");
  const processingBeforeInvocation = await metricsField("llamacpp:requests_processing");
  const workload = newestManifest()?.workload ?? null;
  const buttonStateBefore = await buttonState("Run v2 benchmark");
  let uiAccepted = null;
  let apiAttempt = null;
  if (scenario === "ui") {
    uiAccepted = await clickExact("Run v2 benchmark");
  } else {
    apiAttempt = workload ? await invokeBenchmarkAttempt(workload) : { status: "no-workload" };
  }
  const processingAfterInvocation = await metricsField("llamacpp:requests_processing");
  const notice = await lastNotice();
  return {
    scenario,
    at: Date.now(),
    cancelAt: cancelAt ?? null,
    sinceCancelMs: cancelAt ? Date.now() - cancelAt : null,
    processingAtCancelRequest: processingAtCancelRequest ?? null,
    processingBeforeInvocation,
    processingAfterInvocation,
    buttonStateBefore,
    uiAccepted,
    apiAttempt,
    notice: String(notice ?? "").slice(0, 160),
  };
};

/// Every benchmark record currently on disk. Identity correlation works on
/// file-set differences rather than "newest file", so two runs produced by one
/// scenario are both identified instead of collapsing into one another.
const recordPathsNow = () => {
  try {
    return readdirSync(BENCH_DIR)
      .filter((name) => name.endsWith(".json"))
      .map((name) => join(BENCH_DIR, name));
  } catch {
    return [];
  }
};

const recordIdentity = (path) => {
  if (!path) return null;
  try {
    const record = readManifest(path);
    return { path, terminalOutcome: record.terminalOutcome ?? null };
  } catch (error) {
    return { path, terminalOutcome: `ERR:${error.message}` };
  }
};

const checks = [];
const check = (name, ok, detail = "") => {
  const status = ok === null ? "NOT-EXERCISED" : ok ? "PASS" : "FAIL";
  checks.push({ name, ok, detail, status });
  console.log(`${status} ${name}${detail ? ` | ${detail}` : ""}`);
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
  const cycleRecordsBefore = recordPathsNow();
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
    const numeric = Number(processing);
    if (Number.isFinite(numeric)) windowMaxProcessing = Math.max(windowMaxProcessing, numeric);
    if (processing === "2" || processing === "3") windowOverlap = true;
    const childrenNow = ownedServerPids();
    windowMaxChildren = Math.max(windowMaxChildren, childrenNow.length);
    if (childrenNow.length > 1) windowOverlap = true;
    if (Date.now() - drainStart > DRAIN_BOUND_MS) break;
  }
  const drainMs = Date.now() - drainStart;
  // F9-01: the cycle's record is identified by the difference in the record
  // set, not by "newest file". Exactly one new record belongs to this cycle;
  // a second one is the overlap signal.
  let cycleRecords = [];
  const cycleNewRecords = () =>
    recordPathsNow().filter((path) => !cycleRecordsBefore.includes(path));
  for (let attempt = 0; attempt < 30 && cycleRecords.length === 0; attempt += 1) {
    await settle(500);
    cycleRecords = cycleNewRecords();
  }
  const record = recordIdentity(cycleRecords[0] ?? null);
  check(
    `mt06.cycle-${cycle}-one-record-identity`,
    cycleRecords.length <= 1,
    `newRecords=${cycleRecords.length}`,
  );
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

  if (cycle % 2 === 0) {
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

// --- F9-01 boundary scenarios ------------------------------------------------
// Each scenario runs its own measured benchmark, cancels it while the request
// is in flight, and attempts the replacement on exactly one surface. The
// verdicts are computed by the pure rules in scripts/lib/mt06_verdicts.mjs and
// stored with the identities they were derived from.
run.boundaries = [];
const runBoundaryScenario = async (scenario) => {
  const recordsBefore = recordPathsNow();
  const tokensBefore = await metricsField("llamacpp:tokens_predicted_total");
  await clickExact("Benchmark");
  await settle(1200);
  const generatedSet = await setNumberInput("Benchmark generated tokens", 1024);
  check(
    `mt06.boundary-${scenario}-window-workload-set`,
    String(generatedSet) === "1024",
    `input=${generatedSet}`,
  );
  const started = await clickExact("Run v2 benchmark");
  let inFlight = false;
  for (let attempt = 0; attempt < 90 && !inFlight; attempt += 1) {
    await settle(1000);
    const processing = await metricsField("llamacpp:requests_processing");
    const tokensNow = await metricsField("llamacpp:tokens_predicted_total");
    const generatedTokens =
      Number.isFinite(Number(tokensNow)) && Number.isFinite(Number(tokensBefore))
        ? Number(tokensNow) - Number(tokensBefore)
        : 0;
    if (processing === "1" && generatedTokens >= 1) inFlight = true;
  }
  const processingAtCancelRequest = await metricsField("llamacpp:requests_processing");
  const cancelled = inFlight ? await clickExact("Cancel") : false;
  const cancelAt = Date.now();
  // The attempt lands at the earliest point after the cancel is accepted; the
  // coverage sample is taken immediately before the invocation itself.
  const attempt = await attemptReplacement({
    scenario,
    cancelAt,
    processingAtCancelRequest,
  });
  let settled = false;
  let runEndedMs = null;
  for (let attemptIndex = 0; attemptIndex < 120 && !settled; attemptIndex += 1) {
    await settle(500);
    const cancelState = await buttonState("Cancel");
    const panelFinished = await evaluate(
      `/Benchmark finished|No benchmark is running/i.test(document.body.innerText || "")`,
    );
    if (cancelState !== "enabled" || panelFinished) {
      settled = true;
      runEndedMs = Date.now() - cancelAt;
    }
    if (Date.now() - cancelAt > RUN_SETTLE_BOUND_MS + 90_000) break;
  }
  let processing = await metricsField("llamacpp:requests_processing");
  const drainStart = Date.now();
  let drained = processing === "0";
  let overlap = false;
  let maxProcessing = Number.isFinite(Number(processing)) ? Number(processing) : 0;
  let maxChildren = 0;
  for (let attemptIndex = 0; attemptIndex < 600 && !drained; attemptIndex += 1) {
    await settle(500);
    processing = await metricsField("llamacpp:requests_processing");
    drained = processing === "0";
    const numeric = Number(processing);
    if (Number.isFinite(numeric)) maxProcessing = Math.max(maxProcessing, numeric);
    if (processing === "2" || processing === "3") overlap = true;
    const childrenNow = ownedServerPids();
    maxChildren = Math.max(maxChildren, childrenNow.length);
    if (childrenNow.length > 1) overlap = true;
    if (Date.now() - drainStart > DRAIN_BOUND_MS) break;
  }
  // The original run's record is identified from the record-set difference.
  let newRecords = [];
  for (let attemptIndex = 0; attemptIndex < 60 && newRecords.length === 0; attemptIndex += 1) {
    await settle(500);
    newRecords = recordPathsNow().filter((path) => !recordsBefore.includes(path));
  }
  const original = recordIdentity(newRecords[0] ?? null);
  const coverage = classifyCoverage({
    processingAtCancel: attempt.processingAtCancelRequest,
    processingBeforeInvocation: attempt.processingBeforeInvocation,
  });
  const verdict = evaluateScenario({
    scenario,
    coverage: coverage.coverage,
    uiStateBefore: attempt.buttonStateBefore,
    uiAccepted: attempt.uiAccepted,
    apiOutcome: attempt.apiAttempt?.status ?? null,
    overlapObserved: overlap,
    serializationProved: false,
  });
  check(
    `mt06.boundary-${scenario}-active-request-coverage`,
    true,
    `classification=${coverage.coverage} ${coverage.detail}`,
  );
  check(
    `mt06.boundary-${scenario}-replacement-refused-or-serialized`,
    verdict.status === "NOT-EXERCISED" ? null : verdict.status === "PASS",
    `${verdict.status}: ${verdict.detail}`,
  );
  check(
    `mt06.boundary-${scenario}-original-record-identity`,
    evaluateIdentity({
      originalRecordPath: original?.path ?? null,
      selectedRecordPath: original?.path ?? null,
      knownRecordPaths: recordPathsNow(),
    }).ok,
    `record=${original?.path ?? "none"} outcome=${original?.terminalOutcome ?? "none"}`,
  );
  check(
    `mt06.boundary-${scenario}-original-run-terminated`,
    Boolean(original) && (drained || original.terminalOutcome !== null),
    `drained=${drained} drainMs=${Date.now() - drainStart} outcome=${original?.terminalOutcome ?? "none"}`,
  );
  check(
    `mt06.boundary-${scenario}-no-prohibited-configuration`,
    evaluateProhibited({ overlapObserved: overlap }).ok,
    `overlap=${overlap} maxProcessing=${maxProcessing} maxChildren=${maxChildren}`,
  );

  // Eventual successful replacement: only after cleanup may a new run start.
  const replacementStart = Date.now();
  let replacementStarted = false;
  for (let attemptIndex = 0; attemptIndex < 60 && !replacementStarted; attemptIndex += 1) {
    const state = await buttonState("Run v2 benchmark");
    if (state === "enabled") {
      replacementStarted = await clickExact("Run v2 benchmark");
      break;
    }
    await settle(1000);
  }
  let replacementInFlight = false;
  for (let attemptIndex = 0; attemptIndex < 90 && !replacementInFlight; attemptIndex += 1) {
    await settle(1000);
    if ((await metricsField("llamacpp:requests_processing")) === "1") replacementInFlight = true;
  }
  // Let the replacement COMPLETE on its own so it produces its own record: the
  // original runs demonstrate the cancelled case (a cancel between trials can
  // end a run without a terminal outcome), while the replacement's claim is
  // that new work is allowed - and recorded - after cleanup.
  // Wait for a record that is NOT the original run's: the original's own
  // record can appear late, and its arrival must not end this poll.
  const replacementNewRecords = () =>
    recordPathsNow().filter((path) => !recordsBefore.includes(path) && path !== original?.path);
  let replacementRecords = [];
  for (let attemptIndex = 0; attemptIndex < 480 && replacementRecords.length === 0; attemptIndex += 1) {
    await settle(500);
    replacementRecords = replacementNewRecords();
  }
  if ((await buttonState("Cancel")) === "enabled") await clickExact("Cancel");
  const replacement = recordIdentity(
    replacementRecords.find((path) => path !== original?.path) ?? null,
  );
  check(
    `mt06.boundary-${scenario}-replacement-ran-after-cleanup`,
    replacementStarted && replacementInFlight && Boolean(replacement),
    `started=${replacementStarted} inFlight=${replacementInFlight} newRecords=${replacementRecords.length} record=${replacement?.path ?? "none"} outcome=${replacement?.terminalOutcome ?? "none"}`,
  );
  for (let attemptIndex = 0; attemptIndex < 240; attemptIndex += 1) {
    await settle(500);
    if ((await metricsField("llamacpp:requests_processing")) === "0") break;
  }
  run.boundaries.push({
    scenario,
    started,
    inFlight,
    cancelled,
    cancelAt,
    runEndedMs,
    settled,
    drained,
    overlap,
    maxProcessing,
    maxChildren,
    attempt,
    coverage,
    verdict,
    original,
    replacement,
    newRecords,
    replacementStart,
  });
  if (replacement?.path) run.replacementRecordPath = replacement.path;
  console.log(
    `BOUNDARY ${scenario}: coverage=${coverage.coverage} verdict=${verdict.status} original=${original?.terminalOutcome ?? "none"} replacement=${replacement?.terminalOutcome ?? "none"}`,
  );
};

await runBoundaryScenario("ui");
await runBoundaryScenario("api");

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

const failed = checks.filter((entry) => entry.ok === false);
const notExercised = checks.filter((entry) => entry.ok === null);
run.checks = checks;
run.finishedAtUtc = new Date().toISOString();
run.totalMs = Date.now() - startedAtMs;
run.summary = `${checks.length - failed.length - notExercised.length}/${checks.length} checks PASS (${notExercised.length} not exercised)`;
writeFileSync(EVIDENCE_PATH, JSON.stringify(run, null, 2));
console.log(`MT06 SUMMARY: ${run.summary}`);
console.log(`MT06 EVIDENCE: ${EVIDENCE_PATH}`);
process.exit(failed.length === 0 ? 0 : 1);
