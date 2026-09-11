// High-finding campaign C: FE-01.V3 (packaged select/rescan/inspect/save/start
// with snapshot + command capture), FE-02.V2 (runtime B adoption after
// measured A), IPC-01.V2 (early child exit, unreadable GGUF, window close
// during startup). Usage: node scripts/g05_vitems_c.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";
import { existsSync, copyFileSync, mkdirSync, writeFileSync } from "node:fs";
import { execSync } from "node:child_process";

const [portArg] = process.argv.slice(2);
const PORT = Number(portArg);
let client = await attach(PORT);
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const HOME = process.env.USERPROFILE ?? "C:\\Users\\Mubarak";
const HEALTH_MODEL = `${HOME}\\AppData\\Local\\Localmotive\\health-models\\e3131339bf4e8065265593d4fd8f7bb7ff2d3abff1edb5618aa1197b89cad9f5\\SmolLM2-135M-Q4_K_M.gguf`;
const BACKUP_DIR = `${process.cwd()}\\.hermes-0.6\\vitems-backup`;

const clickText = (text) =>
  evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === ${JSON.stringify(text)} && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
// A successful start navigates the app to the Control view, where the same
// action is labelled "Start profile"; accept both labels.
const clickStart = () =>
  evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => ["Start", "Start profile"].includes((x.textContent ?? "").trim()) && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
const nav = (label) =>
  evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === ${JSON.stringify(label)} && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
const bodyText = () => evaluate(`(document.body.innerText || "").replace(/\\s+/g, " ").slice(0, 6000)`);
const notices = () => evaluate(`(() => [...document.querySelectorAll(".notice-line,.notice")].map(x => (x.textContent ?? "").trim()).filter(Boolean).slice(-3))()`);
const clickTextIn = (rowText, buttonText) =>
  evaluate(`(() => { const rows = [...document.querySelectorAll("article,section,li,div")].filter(x => (x.textContent ?? "").includes(${JSON.stringify(rowText)}) && x.querySelector("button")); const row = rows[rows.length - 1]; if (!row) return "no-row"; const b = [...row.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === ${JSON.stringify(buttonText)}); if (!b) return "no-button"; if (b.disabled) return "disabled"; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return "clicked"; })()`);
const waitLive = async (want = true, tries = 90) => {
  for (let i = 0; i < tries; i += 1) {
    await settle(1000);
    const text = await bodyText();
    const live = /LIVE/.test(text) && !/Starting/.test(text);
    if (live === want) return true;
  }
  return false;
};
const stopServer = async () => { await clickText("Stop server"); await waitLive(false, 25); };
const llamaCount = () => {
  try {
    const out = execSync('tasklist /FI "IMAGENAME eq llama-server.exe" /FO CSV /NH', { encoding: "utf8" });
    return out.split(/\r?\n/).filter((l) => l.toLowerCase().includes("llama-server.exe")).length;
  } catch { return 0; }
};

console.log("D_PRE LLAMA_PROCS", llamaCount());
await stopServer();

// ---------- P1: FE-01.V3 ----------
await nav("Inventory");
await settle(800);
console.log("P1_RESCAN", await clickText("Rescan"));
await settle(3500);
const inv = await bodyText();
console.log("P1_INVENTORY_ROWS", (inv.match(/smollm2/gi) ?? []).length);
await nav("Profile");
await settle(700);
console.log("P1_SAVE", await clickText("Save"));
await settle(600);
const previewBlocks = await evaluate(`(() => [...document.querySelectorAll(".command-preview pre")].map(x => (x.textContent ?? "").trim()).slice(0, 3))()`);
console.log("P1_COMMAND_PREVIEW", JSON.stringify(previewBlocks).slice(0, 600));
await nav("Runtime");
await settle(700);
console.log("P1_INSPECT", await clickText("Inspect"));
for (let i = 0; i < 30; i += 1) { await settle(1000); const n = await notices(); if (/nspec/i.test(n.slice(-1)[0] ?? "")) break; }
await nav("Profile");
await settle(600);
console.log("P1_START", await clickText("Start"));
const live1 = await waitLive(true);
const st = await bodyText();
console.log("P1_LIVE", live1, "| identity:", JSON.stringify((st.match(/LIVE[^|]{0,120}/) ?? [''])[0]).slice(0, 160));
const preview2 = await evaluate(`(() => [...document.querySelectorAll(".command-preview pre")].map(x => (x.textContent ?? "").trim()).slice(0, 3))()`);
console.log("P1_COMMAND_PREVIEW2", JSON.stringify(preview2).slice(0, 700));
await stopServer();
console.log("P1_DONE");

// ---------- P2: FE-02.V2 CPU adoption ----------
await nav("Runtime");
await settle(800);
console.log("P2_ADOPT_CPU", await clickTextIn("CPU-only Windows x64 package", "Use this build"));
await settle(4000);
const rt = await bodyText();
const cpuActive = /CPU[^|]{0,120}ACTIVE/i.test(rt.replace(/\s+/g, " "));
console.log("P2_CPU_ROW_ACTIVE", cpuActive);
// If the CPU row is not active, fall back: adopt the CUDA 12.4 or report honestly.
await nav("Profile");
await settle(500);
console.log("P2_START", await clickText("Start"));
const liveB = await waitLive(true, 90);
console.log("P2_LIVE_B", liveB);
if (liveB) {
  await settle(2000);
  const log = execSync(`powershell -NoProfile -Command "Get-ChildItem $env:TEMP\\localmotive\\server-8080-*.log | Sort-Object LastWriteTime | Select-Object -Last 1 | ForEach-Object FullName"`, { encoding: "utf8" }).trim();
  console.log("P2_LOG", log);
  const tail = execSync(`powershell -NoProfile -Command "Get-Content -Path '${log}' -TotalCount 12 | Out-String"`, { encoding: "utf8" });
  console.log("P2_LOG_HEAD", JSON.stringify(tail.replace(/\s+/g, " ").slice(0, 400)));
  await stopServer();
}
console.log("P2_DONE");

// ---------- P3: IPC-01.V2 early child exit ----------
await nav("Profile");
await settle(500);
console.log("P3_START", await clickStart());
const live3 = await waitLive(true, 90);
console.log("P3_LIVE", live3);
if (live3) {
  execSync('powershell -NoProfile -Command "Get-Process llama-server -ErrorAction SilentlyContinue | Stop-Process -Force"');
  console.log("P3_KILLED_CHILD");
  let terminal = "";
  for (let i = 0; i < 40; i += 1) {
    await settle(1000);
    const t = await bodyText();
    if (/exited|stopped unexpectedly|not running|lost/i.test(t) && !/LIVE/.test(t.slice(0, 400))) { terminal = (await notices()).slice(-1)[0] ?? "state changed"; break; }
    if (!/LIVE/.test(t)) { terminal = "status no longer LIVE"; break; }
  }
  console.log("P3_TERMINAL", JSON.stringify(terminal).slice(0, 160), "| LLAMA_PROCS", llamaCount());
  // The exit detection races the next click: wait for the non-LIVE state and
  // an enabled Start before restarting (a click during the transition lands
  // on the still-visible Stop).
  await waitLive(false, 30);
  await settle(800);
  console.log("P3_RESTART", await clickStart());
  const live3b = await waitLive(true, 90);
  console.log("P3_LIVE_AGAIN", live3b);
  await stopServer();
}
console.log("P3_DONE");

// ---------- P4: IPC-01.V2 unreadable GGUF ----------
mkdirSync(BACKUP_DIR, { recursive: true });
const modelBackup = `${BACKUP_DIR}\\SmolLM2-135M-Q4_K_M.gguf.orig`;
if (!existsSync(modelBackup)) copyFileSync(HEALTH_MODEL, modelBackup);
writeFileSync(HEALTH_MODEL, "NOT-A-GGUF-FILE");
console.log("P4_MODEL_CORRUPTED");
await nav("Profile");
await settle(500);
console.log("P4_START", await clickText("Start"));
let failText = "";
for (let i = 0; i < 45; i += 1) {
  await settle(1000);
  const t = await bodyText();
  const live = /LIVE/.test(t) && !/Starting/.test(t);
  if (live) { failText = "LIVE (unexpected)"; break; }
  const m = t.match(/failed to load|unable to load|exited with code|could not start|invalid[^.]{0,60}/i);
  if (m) { failText = m[0]; break; }
  const n = (await notices()).slice(-1)[0] ?? "";
  if (n && !/Started/.test(n) && /fail|invalid|bad|error|exit/i.test(n)) { failText = n; break; }
}
console.log("P4_TERMINAL", JSON.stringify(failText).slice(0, 200), "| LLAMA_PROCS", llamaCount());
copyFileSync(modelBackup, HEALTH_MODEL);
console.log("P4_MODEL_RESTORED");
await settle(1500);
console.log("P4_RESTART", await clickText("Start"));
const live4 = await waitLive(true, 90);
console.log("P4_LIVE_AGAIN", live4);
await stopServer();
console.log("P4_DONE");

// ---------- P5: IPC-01.V2 window close during startup ----------
await nav("Profile");
await settle(400);
console.log("P5_START", await clickText("Start"));
await settle(2500);
execSync('powershell -NoProfile -Command "Get-Process localmotive-portable,localmotive -ErrorAction SilentlyContinue | Stop-Process -Force; exit 0"');
await settle(3000);
console.log("P5_APP_KILLED | LLAMA_PROCS_AFTER", llamaCount());
execSync(`powershell -NoProfile -Command "Start-Process -FilePath '.hermes-0.6\\final-candidates\\localmotive-portable.exe'; exit 0"`);
let ready = false;
for (let i = 0; i < 60 && !ready; i += 1) {
  await settle(1000);
  try { client = await attach(PORT); ready = true; } catch { /* not yet */ }
}
console.log("P5_RELAUNCH_CDP", ready);
if (ready) {
  console.log("P5_START_AGAIN", await clickText("Start"));
  const live5 = await waitLive(true, 90);
  console.log("P5_LIVE", live5);
  await stopServer();
}
console.log("G05_VITEMS_C DONE");
try { await client.close(); } catch { /* closed */ }
process.exit(0);
