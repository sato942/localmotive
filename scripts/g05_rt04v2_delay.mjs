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

const originalDllSha = createHash("sha256").update(readFileSync(MANAGED_DLL)).digest("hex");

// --- Run 1: delayed download + swap inside the window -----------------------
const invokeStarted = await evaluate(`(() => {
  window.__rt04 = null;
  window.__TAURI_INTERNALS__.invoke("check_managed_runtime_health", { request: { installKey: "b10816" } })
    .then((value) => { window.__rt04 = JSON.stringify({ ok: true, value }); })
    .catch((error) => { window.__rt04 = JSON.stringify({ ok: false, error: String(error && error.message ? error.message : error) }); });
  return true;
})()`);
check("rt04v2.health-run-started", invokeStarted);

let downloading = false;
for (let attempt = 0; attempt < 60 && !downloading; attempt += 1) {
  await settle(500);
  downloading = bytesServed >= 4 * 1024 * 1024;
}
check("rt04v2.download-in-flight", downloading, `bytesServed=${bytesServed}`);

const bytesAtTamper = bytesServed;
writeFileSync(MANAGED_DLL, "LOCALMOTIVE_RT04V2_DELAY_TAMPER\n");
const tamperedSha = createHash("sha256").update(readFileSync(MANAGED_DLL)).digest("hex");
check("rt04v2.dll-swapped-mid-download", tamperedSha !== originalDllSha, `bytesAtSwap=${bytesAtTamper}`);

let outcome = null;
for (let attempt = 0; attempt < 180 && outcome === null; attempt += 1) {
  await settle(2000);
  const raw = await evaluate(`window.__rt04 ?? null`);
  if (raw) outcome = JSON.parse(raw);
}
const finishedBytes = bytesServed;
check("rt04v2.health-run-finished", outcome !== null);
const resultText = JSON.stringify(outcome ?? {});
check(
  "rt04v2.lease-refused-the-swap-between-prep-and-execution",
  outcome?.ok === true && outcome?.value?.passed === false && /verif|trust|content/i.test(resultText),
  `${outcome?.value?.passed === false ? "passed=false" : "unexpected"} bytesServed=${finishedBytes}`,
);
check(
  "rt04v2.model-download-completed-before-refusal",
  finishedBytes >= expectedBytes,
  `bytesServed=${finishedBytes} expected=${expectedBytes}`,
);
check("rt04v2.no-process-spawned-with-tampered-runtime", llamaProcs() === 0, `children=${llamaProcs()}`);

// --- Restore and prove the same run now passes ------------------------------
copyFileSync(backupArg, MANAGED_DLL);
const restoredSha = createHash("sha256").update(readFileSync(MANAGED_DLL)).digest("hex");
check("rt04v2.dll-restored-exactly", tamperedSha !== restoredSha && originalDllSha === restoredSha);

const secondStarted = await evaluate(`(() => {
  window.__rt04b = null;
  window.__TAURI_INTERNALS__.invoke("check_managed_runtime_health", { request: { installKey: "b10816" } })
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
check(
  "rt04v2.restored-runtime-passes-the-same-health-run",
  second?.ok === true && second?.value?.passed === true,
  second?.ok === true ? `passed=${second.value.passed}` : String(second?.error ?? "no-result"),
);

server.close();
const failed = checks.filter((entry) => !entry.ok);
console.log(`RT04V2 SUMMARY: ${checks.length - failed.length}/${checks.length} checks PASS`);
process.exit(failed.length === 0 ? 0 : 1);
