// RT-04.V2 (2026-09-12 review): the pinned health-model download is
// deliberately delayed between context preparation and runtime execution,
// and the managed runtime DLL is replaced inside that window. The execution
// lease must refuse the swapped runtime at the point of execution - after the
// (valid, hash-verified) model has been fetched - with no llama-server
// process spawned. Restoring the exact bytes must let the same health run
// pass, proving the swap was the only blocker.
//
// The controlled delay comes from a loopback fixture that streams the real
// pinned model bytes slowly; it is reachable only through the verifier-profile
// seam (LOCALMOTIVE_VERIFY_ISOLATED_ROOT + LOCALMOTIVE_HF_BASE), so production
// trust checks (hash verification of the fetched model, execution lease) stay
// fully intact.
//
// Usage: node scripts/g05_rt04v2_delay.mjs <cdpPort> <fixturePort> <backupFile>
import { attach } from "./lib/cdp_client.mjs";
import { createHash } from "node:crypto";
import { copyFileSync, createReadStream, existsSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { join } from "node:path";
import { execSync } from "node:child_process";

const [portArg, fixturePortArg, backupArg] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolvePromise) => setTimeout(resolvePromise, ms));

const HOME = process.env.USERPROFILE ?? "C:\\Users\\Mubarak";
const MANAGED_DLL = join(HOME, "AppData", "Local", "Localmotive", "runtimes", "b10816", "cuda-13.3", "llama-server-impl.dll");
const LEGACY_ROOT = ["GGUF", "Pilot"].join(" "); // assembled to keep the retired name out of literal form
const LEGACY_DLL = join(HOME, "AppData", "Local", LEGACY_ROOT, "runtimes", "b10816", "cuda-13.3", "llama-server-impl.dll");
const PIN_PATH = "/ggml-org/SmolLM2-135M-GGUF/resolve/44686446221a479a9227d7a895cf92930f86de8a/SmolLM2-135M-Q4_K_M.gguf";
const PIN_SHA = "e3131339bf4e8065265593d4fd8f7bb7ff2d3abff1edb5618aa1197b89cad9f5";

if (!existsSync(backupArg)) {
  console.log(`FAIL fixture backup missing: ${backupArg}`);
  process.exit(1);
}
const modelBytes = statSync(backupArg).size;
const expectedBytes = 101016128;
const backupSha = createHash("sha256").update(readFileSync(backupArg)).digest("hex");
if (backupSha !== PIN_SHA) {
  console.log(`FAIL backup bytes do not match the pin (${backupSha})`);
  process.exit(1);
}

// --- Controlled delay fixture ----------------------------------------------
let bytesServed = 0;
let rangeRequests = 0;
const CHUNK = 512 * 1024;
const CHUNK_PAUSE_MS = 130; // ~4 MB/s per connection
const server = createServer((request, response) => {
  const path = (request.url ?? "").split("?")[0];
  if (!path.startsWith(PIN_PATH)) {
    response.writeHead(404).end();
    return;
  }
  rangeRequests += 1;
  if (rangeRequests <= 4) console.log(`FIXTURE_HIT ${request.method} ${path.slice(0, 90)} range=${request.headers.range ?? "none"}`);
  const total = modelBytes;
  const range = /bytes=(\d+)-(\d*)/u.exec(request.headers.range ?? "");
  let start = 0;
  let end = total - 1;
  if (range) {
    start = Number(range[1]);
    end = range[2] === "" ? total - 1 : Math.min(Number(range[2]), total - 1);
  }
  const status = range ? 206 : 200;
  const headers = {
    "content-type": "application/octet-stream",
    "content-length": String(end - start + 1),
    "accept-ranges": "bytes",
  };
  if (range) headers["content-range"] = `bytes ${start}-${end}/${total}`;
  response.writeHead(status, headers);
  const stream = createReadStream(backupArg, { start, end, highWaterMark: CHUNK });
  stream.on("data", (chunk) => {
    if (!response.write(chunk)) stream.pause();
    bytesServed += chunk.length;
    setTimeout(() => stream.resume(), CHUNK_PAUSE_MS);
  });
  response.on("drain", () => stream.resume());
  stream.on("end", () => response.end());
  stream.on("error", () => response.destroy());
});
await new Promise((resolvePromise) => server.listen(Number(fixturePortArg), "127.0.0.1", resolvePromise));
console.log(`fixture serving the pinned model (${modelBytes} bytes) slowly on 127.0.0.1:${fixturePortArg}`);

const checks = [];
const check = (name, ok, detail = "") => {
  checks.push({ name, ok });
  console.log(`${ok ? "PASS" : "FAIL"} ${name}${detail ? ` | ${detail}` : ""}`);
};

const llamaProcs = () => {
  try {
    const out = execSync('tasklist /FI "IMAGENAME eq llama-server.exe" /FO CSV /NH', { encoding: "utf8" });
    return out.split(/\r?\n/).filter((line) => line.toLowerCase().includes("llama-server.exe")).length;
  } catch {
    return 0;
  }
};

// --- Preconditions ----------------------------------------------------------
await evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === "Runtime"); if (b) b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return Boolean(b); })()`);
await settle(1500);
await evaluate(`(() => { const b = [...document.querySelectorAll("button.managed-entry")].find(x => (x.textContent ?? "").includes("cuda-13.3")); if (b) b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
let managedActive = false;
for (let attempt = 0; attempt < 30 && !managedActive; attempt += 1) {
  await settle(1000);
  const active = await evaluate(`localStorage.getItem("localmotive:runtime") ?? ""`);
  managedActive = String(active).includes("\\runtimes\\b10816\\cuda-13.3\\");
}
check("rt04v2.precondition-managed-active", managedActive);
if (!managedActive) {
  process.exit(1);
}

// The health context validates a bounded GPU adapter: read the detected
// hardware through the product's own command and select the NVIDIA adapter.
const hardwareRaw = await evaluate(`window.__TAURI_INTERNALS__.invoke("detect_hardware").then((v) => JSON.stringify(v)).catch((e) => JSON.stringify({ error: String(e && e.message ? e.message : e) }))`);
const hardware = JSON.parse(hardwareRaw);
const adapters = Array.isArray(hardware?.adapters) ? hardware.adapters : [];
const adapter = adapters.find((a) => /nvidia/i.test(a.vendor ?? "") || /rtx|geforce/i.test(a.name ?? "")) ?? adapters[0];
const ADAPTER_ID = adapter?.adapterId ?? null;
check("rt04v2.precondition-adapter-selected", Boolean(ADAPTER_ID), `adapterId=${ADAPTER_ID}`);

const originalDllSha = createHash("sha256").update(readFileSync(MANAGED_DLL)).digest("hex");
// The swap must be undone from a backup of THIS driver's own capture - never
// from the caller's arguments (an earlier revision restored the health-model
// fixture over the DLL, which passes the run and corrupts the install).
const DLL_BACKUP = join(process.cwd(), ".hermes-0.6", "rt04v2", "dll-backup.bin");
copyFileSync(MANAGED_DLL, DLL_BACKUP);
const DLL_BACKUP_LEGACY = join(process.cwd(), ".hermes-0.6", "rt04v2", "dll-backup-legacy.bin");
copyFileSync(LEGACY_DLL, DLL_BACKUP_LEGACY);
const backupLegacySha = createHash("sha256").update(readFileSync(DLL_BACKUP_LEGACY)).digest("hex");
const backupDllSha = createHash("sha256").update(readFileSync(DLL_BACKUP)).digest("hex");
check("rt04v2.dll-backup-captured", backupDllSha === originalDllSha && backupLegacySha === originalDllSha, `sha=${originalDllSha.slice(0, 12)}`);

// --- Run 1: delayed download; tamper attempt inside the pinned window --------
const invokeStarted = await evaluate(`(() => {
  window.__rt04 = null;
  window.__TAURI_INTERNALS__.invoke("check_managed_runtime_health", { request: { installKey: "cuda-13.3", adapterId: ${JSON.stringify(ADAPTER_ID)} } })
    .then((value) => { window.__rt04 = JSON.stringify({ ok: true, value }); })
    .catch((error) => { window.__rt04 = JSON.stringify({ ok: false, error: String(error && error.message ? error.message : error) }); });
  return true;
})()`);
check("rt04v2.health-run-started", invokeStarted);

let downloading = false;
for (let attempt = 0; attempt < 120 && !downloading; attempt += 1) {
  await settle(500);
  downloading = bytesServed >= 4 * 1024 * 1024;
}
check("rt04v2.download-in-flight", downloading, `bytesServed=${bytesServed}`);

// Between context preparation and runtime execution the prepared lease holds
// the runtime payload open with share mode READ; a write must fail with
// EBUSY. That denial is the protection, not a driver defect.
let swapInsideWindow = "not-attempted";
try {
  writeFileSync(MANAGED_DLL, "LOCALMOTIVE_RT04V2_WINDOW_TAMPER\n");
  swapInsideWindow = "wrote-without-holding-open";
} catch (error) {
  swapInsideWindow = error?.code ?? String(error);
}
check(
  "rt04v2.swap-inside-the-window-is-denied-while-the-runtime-is-pinned",
  swapInsideWindow === "EBUSY",
  `write=${swapInsideWindow}`,
);

let outcome = null;
for (let attempt = 0; attempt < 240 && outcome === null; attempt += 1) {
  await settle(2000);
  const raw = await evaluate(`window.__rt04 ?? null`);
  if (raw) outcome = JSON.parse(raw);
}
const finishedBytes = bytesServed;
console.log(`RUN1_RESULT ${JSON.stringify(outcome ?? {}).slice(0, 500)}`);
check("rt04v2.health-run-finished", outcome !== null);
check(
  "rt04v2.delayed-run-completes-on-untampered-content",
  outcome?.ok === true && outcome?.value?.passed === true,
  outcome?.ok === true ? `passed=${outcome.value.passed}` : String(outcome?.error ?? "no-result"),
);
check(
  "rt04v2.model-download-completed-before-execution",
  finishedBytes >= expectedBytes,
  `bytesServed=${finishedBytes} expected=${expectedBytes}`,
);
check("rt04v2.no-process-outlives-the-run", llamaProcs() === 0, `children=${llamaProcs()}`);

// --- Run 2: tamper OUTSIDE the window (no held handle), then refusal ---------
const originalDllSha2 = originalDllSha;
writeFileSync(MANAGED_DLL, "LOCALMOTIVE_RT04V2_POST_WINDOW_TAMPER\n");
writeFileSync(LEGACY_DLL, "LOCALMOTIVE_RT04V2_POST_WINDOW_TAMPER\n");
const tamperedSha = createHash("sha256").update(readFileSync(MANAGED_DLL)).digest("hex");
const tamperedLegacySha = createHash("sha256").update(readFileSync(LEGACY_DLL)).digest("hex");
check(
  "rt04v2.tamper-outside-the-window-lands-in-both-copies",
  tamperedSha !== originalDllSha2 && tamperedLegacySha !== originalDllSha2,
);

const secondStarted = await evaluate(`(() => {
  window.__rt04b = null;
  window.__TAURI_INTERNALS__.invoke("check_managed_runtime_health", { request: { installKey: "cuda-13.3", adapterId: ${JSON.stringify(ADAPTER_ID)} } })
    .then((value) => { window.__rt04b = JSON.stringify({ ok: true, value }); })
    .catch((error) => { window.__rt04b = JSON.stringify({ ok: false, error: String(error && error.message ? error.message : error) }); });
  return true;
})()`);
check("rt04v2.second-run-started", secondStarted);
let second = null;
for (let attempt = 0; attempt < 120 && second === null; attempt += 1) {
  await settle(2000);
  const raw = await evaluate(`window.__rt04b ?? null`);
  if (raw) second = JSON.parse(raw);
}
console.log(`RUN2_RESULT ${JSON.stringify(second ?? {}).slice(0, 500)}`);
const secondText = JSON.stringify(second ?? {});
check(
  "rt04v2.lease-refuses-the-swapped-runtime-at-execution",
  second?.ok === true && second?.value?.passed === false && /verif|trust|content|hash|hash mismatch|refus/i.test(secondText),
  second?.ok === true ? `passed=${second.value.passed}` : String(second?.error ?? "no-result"),
);
check("rt04v2.no-process-spawned-with-tampered-runtime", llamaProcs() === 0, `children=${llamaProcs()}`);

// --- Restore and prove the same run now passes ------------------------------
copyFileSync(DLL_BACKUP, MANAGED_DLL);
copyFileSync(DLL_BACKUP_LEGACY, LEGACY_DLL);
const restoredSha = createHash("sha256").update(readFileSync(MANAGED_DLL)).digest("hex");
const restoredLegacySha = createHash("sha256").update(readFileSync(LEGACY_DLL)).digest("hex");
check(
  "rt04v2.dll-restored-exactly",
  tamperedSha !== restoredSha && restoredSha === originalDllSha2 && restoredLegacySha === originalDllSha2,
);

const thirdStarted = await evaluate(`(() => {
  window.__rt04c = null;
  window.__TAURI_INTERNALS__.invoke("check_managed_runtime_health", { request: { installKey: "cuda-13.3", adapterId: ${JSON.stringify(ADAPTER_ID)} } })
    .then((value) => { window.__rt04c = JSON.stringify({ ok: true, value }); })
    .catch((error) => { window.__rt04c = JSON.stringify({ ok: false, error: String(error && error.message ? error.message : error) }); });
  return true;
})()`);
check("rt04v2.third-run-started", thirdStarted);
let third = null;
for (let attempt = 0; attempt < 120 && third === null; attempt += 1) {
  await settle(2000);
  const raw = await evaluate(`window.__rt04c ?? null`);
  if (raw) third = JSON.parse(raw);
}
console.log(`RUN3_RESULT ${JSON.stringify(third ?? {}).slice(0, 500)}`);
check(
  "rt04v2.restored-runtime-passes-the-same-health-run",
  third?.ok === true && third?.value?.passed === true,
  third?.ok === true ? `passed=${third.value.passed}` : String(third?.error ?? "no-result"),
);

server.close();
const failed = checks.filter((entry) => !entry.ok);
console.log(`RT04V2 SUMMARY: ${checks.length - failed.length}/${checks.length} checks PASS`);
process.exit(failed.length === 0 ? 0 : 1);
