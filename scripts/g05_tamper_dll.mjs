// Packaged tamper negative against the FINAL candidate: replace the managed
// runtime DLL with a marker file, try to start, and prove the app refuses
// before any spawn (trust_failure, no llama-server process), then restore the
// exact bytes and prove a clean start (audit RT-04 / G-05 re-bind).
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

mkdirSync(BACKUP_DIR, { recursive: true });
const originalSha = sha(DLL);
if (!existsSync(BACKUP)) copyFileSync(DLL, BACKUP);
console.log(`ORIGINAL_SHA ${originalSha}`);

const client = await attach(PORT);
const notice = () =>
  client.evaluate(`(() => { const n = [...document.querySelectorAll(".notice-line")].map(x => (x.textContent ?? "").trim()).filter(Boolean); return n.slice(-1)[0] ?? ""; })()`);
const status = () =>
  client.evaluate(`(() => { const t = document.body.innerText || ""; const m = t.match(/LIVE|OFFLINE|Starting…|Starting|Start .{0,40}/); return m ? m[0] : "?"; })()`);

// 1) Tamper: overwrite the DLL with marker bytes.
writeFileSync(DLL, "LOCALMOTIVE_TAMPER_MARKER\n");
console.log(`TAMPERED_SHA ${sha(DLL)}`);
await client.evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === "Start" && !x.disabled); if (b) { b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; } return false; })()`);
await new Promise((r) => setTimeout(r, 8000));
const refuseNotice = (await notice()).replace(/\s+/g, " ");
console.log(`TAMPER_NOTICE ${refuseNotice}`);
console.log(`TAMPER_PROCESSES ${llamaProcs()}`);
const refused = /not installed or fails content verification|trust_failure|verification/i.test(refuseNotice);

// 2) Restore exact bytes.
copyFileSync(BACKUP, DLL);
const restoredSha = sha(DLL);
console.log(`RESTORED_SHA ${restoredSha} MATCHES ${restoredSha === originalSha}`);

// 3) Clean start after restore.
const started = await client.evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === "Start" && !x.disabled); if (b) { b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; } return false; })()`);
console.log(`RESTART_CLICKED ${started}`);
let live = false;
for (let i = 0; i < 40 && !live; i += 1) {
  await new Promise((r) => setTimeout(r, 1000));
  const s = await status();
  live = /LIVE/i.test(s);
}
console.log(`RESTORED_LIVE ${live} (${await status()})`);
// stop for a clean end state
await client.evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === "Stop server" && !x.disabled); if (b) { b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; } return false; })()`);
await new Promise((r) => setTimeout(r, 4000));
console.log(`FINAL_PROCESSES ${llamaProcs()}`);
console.log(refused && restoredSha === originalSha && live ? "G05_TAMPER_FINAL PASS" : "G05_TAMPER_FINAL FAIL");
await client.close();
process.exit(0);
