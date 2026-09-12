// Packaged tamper negative against the FINAL candidate: replace the managed
// runtime DLL with a marker file, try to start, and prove the app refuses
// before any spawn (trust_failure, no llama-server process), then restore the
// exact bytes and prove a clean start (audit RT-04 / G-05 re-bind).
//
// Preconditions are enforced: the managed build under test must be the ACTIVE
// runtime and no server may be running, otherwise the start click merely
// reports "Stop the running server..." and the trust check is never reached.
//
// Usage: node scripts/g05_tamper_dll.mjs <cdpPort>
import { attach } from "./lib/cdp_client.mjs";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, copyFileSync, writeFileSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { execSync } from "node:child_process";

const PORT = Number(process.argv[2] ?? 10070);
const HOME = process.env.USERPROFILE ?? "C:\\Users\\Mubarak";
const DLL = join(HOME, "AppData", "Local", "Localmotive", "runtimes", "b10816", "cuda-13.3", "llama-server-impl.dll");
const BACKUP_DIR = join(process.cwd(), ".hermes-0.6", "g05-tamper-final");
const BACKUP = join(BACKUP_DIR, "llama-server-impl.dll.orig");

const sha = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");
const llamaProcs = () => {
  try {
    const out = execSync('tasklist /FI "IMAGENAME eq llama-server.exe" /FO CSV /NH', { encoding: "utf8" });
    return out.split(/\r?\n/).filter((line) => line.toLowerCase().includes("llama-server.exe")).length;
  } catch {
    return 0;
  }
};
const settle = (ms) => new Promise((resolvePromise) => setTimeout(resolvePromise, ms));

mkdirSync(BACKUP_DIR, { recursive: true });
const originalSha = sha(DLL);
if (!existsSync(BACKUP)) copyFileSync(DLL, BACKUP);
console.log(`ORIGINAL_SHA ${originalSha}`);

const client = await attach(PORT);
const notice = () =>
  client.evaluate(`(() => { const n = [...document.querySelectorAll(".notice-line")].map(x => (x.textContent ?? "").trim()).filter(Boolean); return n.slice(-1)[0] ?? ""; })()`);
const status = () =>
  client.evaluate(`(() => { const t = document.body.innerText || ""; const m = t.match(/LIVE|OFFLINE|Starting…|Starting|Start .{0,40}/); return m ? m[0] : "?"; })()`);
const clickStop = () =>
  client.evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === "Stop server" && !x.disabled); if (b) { b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; } return false; })()`);
const procsZero = async (seconds) => {
  for (let attempt = 0; attempt < seconds; attempt += 1) {
    if (llamaProcs() === 0) return true;
    await settle(1000);
  }
  return llamaProcs() === 0;
};

// Precondition 1: no server may be running (mt01d and friends sometimes leave
// one up; a live server turns the start click into a "stop first" refusal).
if (llamaProcs() > 0) {
  // Only one screen is mounted at a time; the stop control lives on the
  // Control screen ("Stop server" / "Cancel start").
  await client.evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === "Control"); if (b) b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return Boolean(b); })()`);
  await settle(1500);
  await clickStop();
}
let stoppedBeforeTamper = await procsZero(12);
if (!stoppedBeforeTamper) {
  // The stop click can land while the view is mid-update; retry it while the
  // child is still alive before declaring the precondition failed.
  for (let attempt = 0; attempt < 3 && !stoppedBeforeTamper; attempt += 1) {
    await clickStop();
    stoppedBeforeTamper = await procsZero(12);
  }
}
console.log(`PRECONDITION_NO_SERVER ${stoppedBeforeTamper} procs=${llamaProcs()}`);
if (!stoppedBeforeTamper) {
  console.log("G05_TAMPER_FINAL FAIL (a server survived the stop attempt)");
  process.exit(1);
}

// Precondition 2: the managed build whose DLL is under test must be active,
// or a start would launch a different runtime and never read this DLL. The
// managed entries render on the Runtime screen, so navigate there first.
await client.evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === "Runtime"); if (b) b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return Boolean(b); })()`);
await settle(1500);
const activated = await client.evaluate(`(() => { const b = [...document.querySelectorAll("button.managed-entry")].find(x => (x.textContent ?? "").includes("cuda-13.3")); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
// Activation runs a full runtime inspection and persists asynchronously, so
// poll the stored selection instead of a fixed wait.
let managedActive = false;
let activeRuntime = "";
for (let attempt = 0; attempt < 30 && !managedActive; attempt += 1) {
  await settle(1000);
  activeRuntime = await client.evaluate(`localStorage.getItem("localmotive:runtime") ?? ""`);
  managedActive = String(activeRuntime).includes("\\runtimes\\b10816\\cuda-13.3\\");
}
console.log(`PRECONDITION_MANAGED_ACTIVE ${managedActive} clicked=${activated} (${String(activeRuntime).slice(-70)})`);
if (!managedActive) {
  console.log("G05_TAMPER_FINAL FAIL (the managed runtime under test is not active)");
  process.exit(1);
}

// 1) Tamper: overwrite the DLL with marker bytes, then start from the
// Profile screen (that is where Start lives; the Runtime view has none).
writeFileSync(DLL, "LOCALMOTIVE_TAMPER_MARKER\n");
console.log(`TAMPERED_SHA ${sha(DLL)}`);
await client.evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === "Profile"); if (b) b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return Boolean(b); })()`);
await settle(1200);
const tamperClicked = await client.evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => ["Start", "Start profile"].includes((x.textContent ?? "").trim()) && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
console.log(`TAMPER_START_CLICKED ${tamperClicked}`);
if (!tamperClicked) {
  console.log(`TAMPER_START_CANDIDATES ${await client.evaluate(`(() => [...document.querySelectorAll("button")].map(b => (b.textContent ?? "").trim()).filter((t) => /start|stop/i.test(t)).join(","))()`)}`);
}
await settle(8000);
const refuseNotice = (await notice()).replace(/\s+/g, " ");
console.log(`TAMPER_NOTICE ${refuseNotice}`);
console.log(`TAMPER_PROCESSES ${llamaProcs()}`);
const refused = /not installed or fails content verification|trust_failure|verification/i.test(refuseNotice);

// 2) Restore exact bytes.
copyFileSync(BACKUP, DLL);
const restoredSha = sha(DLL);
console.log(`RESTORED_SHA ${restoredSha} MATCHES ${restoredSha === originalSha}`);

// 3) Clean start after restore.
const started = await client.evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => ["Start", "Start profile"].includes((x.textContent ?? "").trim()) && !x.disabled); if (b) { b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; } return false; })()`);
console.log(`RESTART_CLICKED ${started}`);
let live = false;
for (let attempt = 0; attempt < 40 && !live; attempt += 1) {
  await settle(1000);
  live = /LIVE/i.test(await status());
}
console.log(`RESTORED_LIVE ${live} (${await status()})`);

// Stop for a clean end state; retry until the process count really reaches 0.
if (llamaProcs() > 0) await clickStop();
const cleanEnd = await procsZero(30);
console.log(`FINAL_PROCESSES ${llamaProcs()}`);
const pass = refused && restoredSha === originalSha && live && cleanEnd;
console.log(pass ? "G05_TAMPER_FINAL PASS" : "G05_TAMPER_FINAL FAIL");
process.exit(pass ? 0 : 1);
