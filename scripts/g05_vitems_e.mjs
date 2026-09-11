// Campaign E: FE-05.V1 clean cancel-under-navigation, MT-05.V3 reservation
// release, FE-05.V3 calibration records walk. Usage: node scripts/g05_vitems_e.mjs <port>
import { attach } from "./lib/cdp_client.mjs";
import { execSync } from "node:child_process";

const [portArg] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const clickText = (text) =>
  evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === ${JSON.stringify(text)} && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
const nav = (label) => clickText(label);
const bodyText = () => evaluate(`(document.body.innerText || "").replace(/\\s+/g, " ").slice(0, 9000)`);
const enabledCancel = () =>
  evaluate(`(() => { const b = [...document.querySelectorAll("button")].filter(x => (x.textContent ?? "").trim() === "Cancel" && !x.disabled && !/notice/i.test(x.className)); return b.length ? "yes" : [...document.querySelectorAll("button")].filter(x => (x.textContent ?? "").trim() === "Cancel").map(x => "disabled").join(",") || "none"; })()`);
const clickEnabledCancel = () =>
  evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === "Cancel" && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
const waitLive = async (want = true, tries = 60) => {
  for (let i = 0; i < tries; i += 1) { await settle(1000); const t = await bodyText(); const live = /LIVE/.test(t) && !/Starting/.test(t); if (live === want) return true; }
  return false;
};
const llamaCount = () => {
  try { const out = execSync('tasklist /FI "IMAGENAME eq llama-server.exe" /FO CSV /NH', { encoding: "utf8" }); return out.split(/\r?\n/).filter((l) => l.toLowerCase().includes("llama-server.exe")).length; } catch { return 0; }
};

await nav("Benchmark");
await settle(1200);
console.log("E1_RUN", await clickText("Run v2 benchmark"));
await settle(4500);
console.log("E1_CANCEL_STATE_IN_FLIGHT", await enabledCancel());
await nav("Dashboard");
await settle(2000);
await nav("Benchmark");
await settle(1200);
console.log("E1_CANCEL_STATE_AFTER_RETURN", await enabledCancel());
console.log("E1_CLICK_CANCEL", await clickEnabledCancel());
let terminal = "";
for (let i = 0; i < 75; i += 1) {
  await settle(1000);
  const t = await bodyText();
  const running = /Cancelling|Benchmark running/.test(t);
  if (!running) { terminal = /No v2 result/.test(t) ? "idle-no-result (cancel discarded the run)" : "idle-with-earlier-result"; break; }
}
console.log("E1_TERMINAL", JSON.stringify(terminal).slice(0, 140), "| LLAMA", llamaCount());
const st = await bodyText();
console.log("E1_STATUS_SLICE", JSON.stringify((st.match(/(LIVE|IDLE)[^|]{0,90}/) ?? [""])[0]).slice(0, 140));

// MT-05.V3: reservation release - stop and start must both work after the cancel.
console.log("E2_STOP", await clickText("Stop server"));
const stopped = await waitLive(false, 30);
console.log("E2_STOPPED", stopped, "| LLAMA", llamaCount());
console.log("E2_START", await clickText("Start"));
const started = await waitLive(true, 90);
console.log("E2_STARTED", started, "| CHILD_EXE", execSync('powershell -NoProfile -Command "(Get-Process llama-server -ErrorAction SilentlyContinue | Select-Object -First 1).Path"', { encoding: "utf8" }).trim());

// FE-05.V3: calibration records walk.
await nav("Benchmark");
await settle(1200);
const calSlice = await evaluate(`(() => { const el = [...document.querySelectorAll("*")].filter(x => (x.textContent ?? "").includes("CALIBRATION") && (x.textContent ?? "").length < 5000); const root = el[el.length - 1]; return root ? (root.textContent ?? "").replace(/\\s+/g, " ").slice(0, 1200) : "no-section"; })()`);
console.log("E3_CALIBRATION_SECTION", JSON.stringify(calSlice).slice(0, 1300));
console.log("E3_BUTTONS", JSON.stringify(await evaluate(`(() => [...document.querySelectorAll("button")].filter(b => /anchor|calibrat|record|manifest/i.test(b.textContent || "")).map(b => ({ t: (b.textContent || "").trim(), dis: b.disabled })).slice(0, 8))()`)));
// Open whichever control reveals the saved records (avoid Replay manifest: native dialog).
const opened = await evaluate(`(() => { const b = [...document.querySelectorAll("button")].filter(x => /saved|records|load|show/i.test(x.textContent || "") && !x.disabled); if (!b.length) return "none"; b[0].dispatchEvent(new MouseEvent("click", { bubbles: true })); return (b[0].textContent || "").trim(); })()`);
console.log("E3_OPENED", JSON.stringify(opened));
await settle(2500);
const recText = await evaluate(`(() => { const t = (document.body.innerText || "").replace(/\\s+/g, " "); const m = t.match(/SAVED[^£]{0,600}|RECORDS[^£]{0,600}|compatibility[^ ]{0,120}/i); return m ? m[0].slice(0, 700) : "no-records-text"; })()`);
console.log("E3_RECORDS_TEXT", JSON.stringify(recText).slice(0, 900));
console.log("E3_DONE");
await client.close();
process.exit(0);
