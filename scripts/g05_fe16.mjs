// FE-16.V1/V3 packaged verification.
//
// V1: overlapping scan/cloud/start orders must leave the active server and
// the active evidence run untouched, and the legacy/v2/Start guards must
// follow the real lifecycle through actual UI interactions.
// V3: editing the profile draft while the server runs must not change the
// displayed running identity, and an unexpected exit must leave the final
// bounded log visible and retrievable.
//
// Usage: node scripts/g05_fe16.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";
import { execSync } from "node:child_process";

const [portArg] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

let failed = 0;
const check = (name, ok, detail = "") => {
  if (!ok) failed += 1;
  console.log(`${ok ? "PASS" : "FAIL"} ${name}${detail ? ` | ${detail}` : ""}`);
};

const clickText = (text) =>
  evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === ${JSON.stringify(text)} && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
const buttonState = (text) =>
  evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === ${JSON.stringify(text)}); return b ? { present: true, disabled: b.disabled } : { present: false, disabled: null }; })()`);
const nav = (label) => clickText(label);
const status = () => evaluate(`window.__TAURI_INTERNALS__.invoke("server_status")`);
const readLog = () => evaluate(`window.__TAURI_INTERNALS__.invoke("read_server_log")`);
const instruments = () =>
  evaluate(`(() => [...document.querySelectorAll(".instrument-strip .instrument")].map(x => ({ label: (x.querySelector(".instrument-label")?.textContent ?? "").trim(), value: (x.querySelector("strong")?.textContent ?? "").trim(), detail: (x.querySelector("small")?.textContent ?? "").trim() })))()`);
const setNamedInput = (labelText, value) =>
  evaluate(`(() => { const label = [...document.querySelectorAll("label")].find(x => (x.textContent ?? "").includes(${JSON.stringify(labelText)})); const input = label?.querySelector("input,textarea"); if (!input) return false; const proto = input.tagName === "TEXTAREA" ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype; const setter = Object.getOwnPropertyDescriptor(proto, "value").set; setter.call(input, ${JSON.stringify(value)}); input.dispatchEvent(new Event("input", { bubbles: true })); return true; })()`);
const namedInputValue = (labelText) =>
  evaluate(`(() => { const label = [...document.querySelectorAll("label")].find(x => (x.textContent ?? "").includes(${JSON.stringify(labelText)})); return label?.querySelector("input,textarea")?.value ?? null; })()`);
const bodyText = () => evaluate(`(document.body.innerText || "").replace(/\\s+/g, " ")`);
const logPanelText = () => evaluate(`(() => { const panel = [...document.querySelectorAll(".terminal-panel")][0]; return panel ? (panel.innerText || "").replace(/\\s+/g, " ").slice(0, 600) : null; })()`);
const llamaPids = () => {
  try {
    const out = execSync('tasklist /FI "IMAGENAME eq llama-server.exe" /FO CSV /NH', { encoding: "utf8" });
    return out.split(/\r?\n/).filter((l) => l.toLowerCase().includes("llama-server.exe")).map((l) => (l.split(",")[1] ?? "").replaceAll('"', ""));
  } catch { return []; }
};
const waitFor = async (predicate, tries = 60, stepMs = 1000) => {
  for (let i = 0; i < tries; i += 1) {
    await settle(stepMs);
    if (await predicate()) return true;
  }
  return false;
};

// ---------- Preconditions ----------
let st = await status();
if (st.running) {
  await nav("Control");
  await settle(600);
  await clickText("Stop server");
  await waitFor(async () => (await status()).running === false, 30);
}
await nav("Control");
await settle(600);
const started = await clickText("Start profile");
const live = await waitFor(async () => (await status()).running === true, 90);
st = await status();
check("fe16.baseline.reaches-live", started && live && st.running === true, `pid=${st.pid}`);
const pid0 = st.pid;
const specType0 = st.specType;

// ---------- V1 order 1: scan + cloud reload overlap the live server ----------
await nav("Inventory");
await settle(500);
const rescanClicked = await clickText("Rescan");
await nav("AI Tune");
await settle(300);
await nav("Control");
await settle(5000);
st = await status();
check("fe16.v1.scan-cloud-overlap-keeps-live", rescanClicked === true && st.running === true && st.pid === pid0, `rescan=${rescanClicked} running=${st.running} pid=${st.pid}`);

// ---------- V1 order 2: a long evidence run survives navigation and a rescan ----------
await nav("Benchmark");
await settle(800);
await setNamedInput("Benchmark trials", "10");
await settle(300);
const v2Clicked = await clickText("Run v2 benchmark");
const cancelLive = await waitFor(async () => {
  const state = await buttonState("Cancel");
  return state.present && state.disabled === false;
}, 30, 250);
check("fe16.v1.v2-run-starts", v2Clicked === true && cancelLive, `clicked=${v2Clicked} cancel=${cancelLive}`);
const legacyDuring = await buttonState("Run benchmark");
check("fe16.v1.legacy-guarded-during-v2", legacyDuring.present && legacyDuring.disabled === true, JSON.stringify(legacyDuring));
await nav("Inventory");
await settle(400);
await clickText("Rescan");
await nav("Benchmark");
await settle(800);
const cancelStill = await buttonState("Cancel");
st = await status();
check("fe16.v1.v2-survives-overlap", cancelStill.present && cancelStill.disabled === false && st.running === true && st.pid === pid0, `cancel=${JSON.stringify(cancelStill)} running=${st.running}`);
const cancelled = await clickText("Cancel");
const settled = await waitFor(async () => {
  const state = await buttonState("Cancel");
  return state.present && state.disabled === true;
}, 90, 500);
const legacyAfter = await buttonState("Run benchmark");
check("fe16.v1.cancel-releases-guards", cancelled === true && settled && legacyAfter.present && legacyAfter.disabled === false, `cancel=${cancelled} settled=${settled} legacy=${JSON.stringify(legacyAfter)}`);
st = await status();
check("fe16.v1.server-persists-after-cancel", st.running === true && st.pid === pid0, `running=${st.running} pid=${st.pid}`);

// ---------- V3: draft edits must not change the running identity ----------
await nav("Control");
await settle(600);
const before = await instruments();
st = await status();
await nav("Profile");
await settle(700);
const draftNameBefore = await namedInputValue("Profile name");
const edited = await setNamedInput("Profile name", "fe16-draft-edited");
await settle(400);
const draftNameAfter = await namedInputValue("Profile name");
await nav("Control");
await settle(600);
const after = await instruments();
const stAfterEdit = await status();
const strategyBefore = before.find((i) => i.label === "STRATEGY");
const strategyAfter = after.find((i) => i.label === "STRATEGY");
const processBefore = before.find((i) => i.label === "PROCESS");
const processAfter = after.find((i) => i.label === "PROCESS");
const endpointBefore = before.find((i) => i.label === "ENDPOINT");
const endpointAfter = after.find((i) => i.label === "ENDPOINT");
check("fe16.v3.draft-edit-applied", edited === true && draftNameAfter === "fe16-draft-edited", `before=${JSON.stringify(draftNameBefore)} after=${JSON.stringify(draftNameAfter)}`);
check(
  "fe16.v3.running-identity-unchanged-by-edit",
  JSON.stringify(strategyBefore) === JSON.stringify(strategyAfter) &&
    JSON.stringify(processBefore) === JSON.stringify(processAfter) &&
    JSON.stringify(endpointBefore) === JSON.stringify(endpointAfter) &&
    stAfterEdit.specType === specType0 &&
    stAfterEdit.pid === pid0,
  `strategy=${JSON.stringify(strategyAfter)} process=${JSON.stringify(processAfter)}`,
);
// Restore the draft label so later probes see the stored profile name again.
await nav("Profile");
await settle(600);
await setNamedInput("Profile name", draftNameBefore ?? "smollm2-135m");
await settle(300);

// ---------- V3: unexpected exit keeps the final bounded log visible ----------
const killPid = st.pid ?? pid0;
try {
  execSync(`taskkill /F /PID ${killPid}`, { stdio: "pipe" });
} catch (error) {
  check("fe16.v3.unexpected-exit-kill", false, `taskkill failed: ${error.message}`);
}
const exited = await waitFor(async () => (await status()).running === false, 30);
const stExited = await status();
const pids = llamaPids();
check("fe16.v3.state-goes-non-live", exited && stExited.running === false && !pids.includes(String(killPid)), `exited=${exited} running=${stExited.running} pids=${JSON.stringify(pids)}`);
const finalLog = await readLog();
const panelText = await logPanelText();
const retained = await evaluate(`(() => { const el = document.querySelector(".log-retained pre"); return el ? (el.textContent ?? "").length : 0; })()`);
check(
  "fe16.v3.final-log-visible",
  typeof finalLog === "string" && finalLog.length > 0 && panelText !== null && panelText.includes("Server is stopped") && retained > 0,
  `read_server_log=${typeof finalLog === "string" ? finalLog.length : "?"} chars retained=${retained}`,
);
check("fe16.v3.log-path-retained", typeof stExited.logPath === "string" && stExited.logPath.length > 0, `logPath=${stExited.logPath}`);
const text = await bodyText();
check("fe16.v3.no-leaked-placeholder-text", !text.includes("props.profile"), "");
// A clean start from the exited state must reach LIVE again (recovery).
await nav("Control");
await settle(600);
await clickText("Start profile");
const liveAgain = await waitFor(async () => (await status()).running === true, 90);
check("fe16.v3.restart-after-exit", liveAgain, "");
await clickText("Stop server");
await waitFor(async () => (await status()).running === false, 30);

console.log(`FE16_SUMMARY ${failed === 0 ? "ALL-PASS" : `${failed}-FAILED`}`);
process.exit(failed === 0 ? 0 : 1);
