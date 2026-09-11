// High-finding campaign D: FE-02.V2 (adoption with process-path proof),
// FE-05.V1/V2/V3 (deferred run survival, cross-screen completion, provenance
// records), MT-05.V3 (cancel/restart records + reservation release).
// Usage: node scripts/g05_vitems_d.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";
import { execSync } from "node:child_process";

const [portArg] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const clickText = (text) =>
  evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === ${JSON.stringify(text)} && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
const clickButtonIn = (rowFragment, buttonText) =>
  evaluate(`(() => { const b = [...document.querySelectorAll("button")].filter(x => (x.textContent ?? "").trim() === ${JSON.stringify(buttonText)}); const target = b.find(btn => { let n = btn; for (let k = 0; k < 6 && n; k += 1) { if ((n.textContent ?? "").includes(${JSON.stringify(rowFragment)})) return true; n = n.parentElement; } return false; }); if (!target) return "no-button"; if (target.disabled) return "disabled"; target.dispatchEvent(new MouseEvent("click", { bubbles: true })); return "clicked"; })()`);
const nav = (label) => clickText(label);
const bodyText = () => evaluate(`(document.body.innerText || "").replace(/\\s+/g, " ").slice(0, 8000)`);
const notices = () => evaluate(`(() => [...document.querySelectorAll(".notice-line,.notice")].map(x => (x.textContent ?? "").trim()).filter(Boolean).slice(-2))()`);
const waitLive = async (want = true, tries = 90) => {
  for (let i = 0; i < tries; i += 1) {
    await settle(1000);
    const text = await bodyText();
    const live = /LIVE/.test(text) && !/Starting/.test(text);
    if (live === want) return true;
  }
  return false;
};
const childPath = () => {
  try {
    const out = execSync('powershell -NoProfile -Command "(Get-Process llama-server -ErrorAction SilentlyContinue | Select-Object -First 1).Path"', { encoding: "utf8" }).trim();
    return out || "(none)";
  } catch { return "(none)"; }
};
const llamaCount = () => {
  try {
    const out = execSync('tasklist /FI "IMAGENAME eq llama-server.exe" /FO CSV /NH', { encoding: "utf8" });
    return out.split(/\r?\n/).filter((l) => l.toLowerCase().includes("llama-server.exe")).length;
  } catch { return 0; }
};
const stopServer = async () => { await clickText("Stop server"); await waitLive(false, 25); };

console.log("D0 LLAMA", llamaCount());

// ---------- FE-02.V2 ----------
// 1) A: adopt the managed CUDA 13.3 build, start, prove the child path.
await nav("Runtime");
await settle(1000);
console.log("D1_ADOPT_CUDA", await clickButtonIn("NVIDIA CUDA 13.3", "Use this build"));
await settle(4000);
await nav("Profile");
await settle(600);
console.log("D1_SAVE", await clickText("Save"));
await settle(500);
console.log("D1_START", await clickText("Start"));
const liveA = await waitLive(true);
await settle(1500);
console.log("D1_LIVE", liveA, "| CHILD_PATH", childPath());
await stopServer();

// 2) B: adopt the managed CPU build, start, prove the child path follows B.
await nav("Runtime");
await settle(1000);
console.log("D2_ADOPT_CPU", await clickButtonIn("CPU-only Windows x64 package", "Use this build"));
await settle(4000);
const rtText = await bodyText();
console.log("D2_ACTIVE_HINT", JSON.stringify((rtText.match(/[A-Za-z0-9 \\-·]{0,60}ACTIVE[^|]{0,60}/) ?? [""])[0]).slice(0, 140));
await nav("Profile");
await settle(600);
console.log("D2_SAVE", await clickText("Save"));
await settle(500);
console.log("D2_START", await clickText("Start"));
const liveB = await waitLive(true);
await settle(1500);
console.log("D2_LIVE", liveB, "| CHILD_PATH", childPath());
await stopServer();

// 3) Restore A and leave it live for the FE-05 legs.
await nav("Runtime");
await settle(1000);
console.log("D3_ADOPT_CUDA", await clickButtonIn("NVIDIA CUDA 13.3", "Use this build"));
await settle(4000);
await nav("Profile");
await settle(600);
console.log("D3_SAVE", await clickText("Save"));
await settle(500);
console.log("D3_START", await clickText("Start"));
const liveA2 = await waitLive(true);
await settle(1500);
console.log("D3_LIVE", liveA2, "| CHILD_PATH", childPath());
console.log("FE02V2 A=", childPath());

// ---------- FE-05.V1: navigate away and back, cancel keeps the handle ----------
await nav("Benchmark");
await settle(1200);
console.log("F51_RUN", await clickText("Run v2 benchmark"));
await settle(2500);
await nav("Dashboard");
await settle(2000);
await nav("Benchmark");
await settle(1500);
const midText = await bodyText();
const cancelling = /Cancelling|Cancel/.test(midText);
console.log("F51_BACK cancellingVisible=", cancelling);
console.log("F51_CANCEL", await clickText("Cancel"));
let cancelled = "";
for (let i = 0; i < 60; i += 1) {
  await settle(1000);
  const t = await bodyText();
  if (!/Cancelling/.test(t) && /No v2 result|Decode throughput|Cancelled/i.test(t)) { cancelled = (t.match(/Cancelled[^.]{0,60}|No v2 result/) ?? ["idle"])[0]; break; }
}
console.log("F51_TERMINAL", JSON.stringify(cancelled).slice(0, 140), "| LLAMA", llamaCount());

// ---------- FE-05.V2: complete while another screen is open ----------
await nav("Dashboard");
await settle(800);
await nav("Benchmark");
await settle(800);
console.log("F52_RUN", await clickText("Run v2 benchmark"));
await settle(2500);
await nav("Dashboard");
console.log("F52_AWAY");
let done = "";
for (let i = 0; i < 90; i += 1) {
  await settle(1000);
  const t = await bodyText();
  if (/Decode throughput/.test(t) && /sampled/.test(t)) { done = "completed-while-away"; break; }
}
await nav("Benchmark");
await settle(1500);
const afterText = await bodyText();
const resultVisible = /Decode throughput/.test(afterText) && /p50/.test(afterText);
console.log("F52_AWAY_RESULT", done, "| visibleAfterReturn", resultVisible);
console.log("F52_RESULT_SLICE", JSON.stringify((afterText.match(/Decode throughput[^|]{0,140}/) ?? [""])[0]).slice(0, 180));

// ---------- FE-05.V3 + MT-05.V3: provenance records + generations ----------
const recBtns = await evaluate(`(() => [...document.querySelectorAll("button")].map(x => (x.textContent ?? "").trim()).filter(x => /record|anchor|calibrat|manifest|recover/i.test(x)).slice(0, 10))()`);
console.log("F53_RECORD_BUTTONS", JSON.stringify(recBtns));
for (const label of recBtns) {
  const clicked = await clickText(label);
  await settle(1500);
  const t = await bodyText();
  if (/compatibility|provenance|anchor|record/i.test(t)) {
    console.log("F53_AFTER", JSON.stringify(label), JSON.stringify((t.match(/compatibilityKey[^ ]{0,80}|[a-f0-9]{12,64}/gi) ?? []).slice(0, 6)));
    break;
  }
}
console.log("F53_SCREEN_SLICE", JSON.stringify((await bodyText()).match(/CALIBRATION|RECORDS|EVIDENCE[^|]{0,200}/g) ?? []));
console.log("MT05_STATUS", JSON.stringify((await bodyText()).match(/LIVE[^|]{0,100}/) ?? []));
console.log("MT05_LLAMA", llamaCount());
await stopServer();
console.log("MT05_AFTER_STOP LLAMA", llamaCount());
console.log("G05_VITEMS_D DONE");
await client.close();
process.exit(0);
