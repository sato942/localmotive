// High-finding verification campaign A (FE-04.V2, IPC-01.V2, FE-02.V2).
// Usage: node scripts/g05_vitems_a.mjs <debugPort>
// Assumes the packaged app is live with a saved SmolLM2 profile whose runtime
// is the managed CUDA install at MANAGED below.
import { attach } from "./lib/cdp_client.mjs";
import { existsSync } from "node:fs";

const [portArg] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const HOME = process.env.USERPROFILE ?? "C:\\Users\\Mubarak";
const MANAGED = `${HOME}\\AppData\\Local\\Localmotive\\runtimes\\b10816\\cuda-13.3\\llama-server.exe`;
const LEGACY_CPU = `${HOME}\\AppData\\Local\\GGUF Pilot\\runtimes\\b10816\\cpu\\llama-server.exe`;
const SLEEPER = `${process.cwd()}\\.hermes-0.6\\slow-runtime.exe`;

const clickText = (text) =>
  evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === ${JSON.stringify(text)} && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
const notice = () =>
  evaluate(`(() => { const n = [...document.querySelectorAll(".notice-line,.notice")].map(x => (x.textContent ?? "").trim()).filter(Boolean); return n.slice(-1)[0] ?? ""; })()`);
const nav = (label) =>
  evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === ${JSON.stringify(label)} && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
const setPathInput = (value) =>
  evaluate(`(() => { const input = [...document.querySelectorAll("input")].find(x => (x.placeholder ?? "").includes("llama-server.exe")); if (!input) return false; const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value").set; setter.call(input, ${JSON.stringify(value)}); input.dispatchEvent(new Event("input", { bubbles: true })); return true; })()`);
const bodyText = () => evaluate(`(document.body.innerText || "").replace(/\\s+/g, " ").slice(0, 4000)`);
const llamaProcs = () => evaluate(`"checked"`); // host-side check below

console.log("SLEEPER_EXISTS", existsSync(SLEEPER));
console.log("MANAGED_EXISTS", existsSync(MANAGED));
console.log("LEGACY_CPU_EXISTS", existsSync(LEGACY_CPU));

// ---------- Phase A: FE-04.V2 slow-probe responsiveness ----------
await nav("Runtime");
await settle(800);
console.log("A_SET_PATH", await setPathInput(SLEEPER));
await settle(400);
const t0 = Date.now();
console.log("A_INSPECT_CLICKED", await clickText("Inspect"));
// During the 6 s x2 sleeper window, interactive round-trips must stay fast.
await settle(700);
const tNav = Date.now();
await nav("Dashboard");
const navMs = Date.now() - tNav;
await settle(300);
const tBack = Date.now();
await nav("Runtime");
const backMs = Date.now() - tBack;
const tResp = Date.now();
await evaluate("1+1");
const respMs = Date.now() - tResp;
console.log(`A_INTERACTIVE nav=${navMs}ms back=${backMs}ms evalRoundtrip=${respMs}ms windowCoarse=${Date.now() - t0}ms`);
// wait for the inspect to settle
let sawInspect = "";
for (let i = 0; i < 30; i += 1) {
  await settle(1000);
  const n = await notice();
  if (n && !/Started/.test(n)) { sawInspect = n; break; }
}
console.log("A_INSPECT_NOTICE", sawInspect.replace(/\s+/g, " ").slice(0, 160));
console.log("A_ELAPSED_MS", Date.now() - t0);

// ---------- Phase B: IPC-01.V2 early child exit ----------
await nav("Profile");
await settle(600);
console.log("B_SAVE_CLICKED", await clickText("Save"));
await settle(800);
const tStart = Date.now();
console.log("B_START_CLICKED", await clickText("Start"));
let terminal = "";
for (let i = 0; i < 60; i += 1) {
  await settle(1000);
  const text = await bodyText();
  const m = text.match(/exited with code \\d+|could not be started|failed to start|not a valid runtime|exited immediately/);
  if (m) { terminal = m[0]; break; }
  const live = /LIVE/.test(text) && !/Starting/.test(text);
  if (live) { terminal = "LIVE (unexpected)"; break; }
  if (/Cancel/.test(text) && i > 45) break;
}
console.log(`B_TERMINAL "${terminal}" after ${Date.now() - tStart}ms`);
console.log("B_NOTICE", (await notice()).replace(/\s+/g, " ").slice(0, 200));

// ---------- Phase C: restore the managed runtime and start clean ----------
await nav("Runtime");
await settle(600);
console.log("C_SET_PATH", await setPathInput(MANAGED));
await settle(300);
console.log("C_INSPECT", await clickText("Inspect"));
for (let i = 0; i < 40; i += 1) { await settle(1000); const n = await notice(); if (n && /nspec|fail|not/i.test(n) && !/Started/.test(n)) break; }
await nav("Profile");
await settle(400);
console.log("C_SAVE", await clickText("Save"));
await settle(600);
console.log("C_START", await clickText("Start"));
let live = false;
for (let i = 0; i < 60 && !live; i += 1) { await settle(1000); const text = await bodyText(); live = /LIVE/.test(text) && !/Starting/.test(text); }
console.log("C_LIVE", live);
console.log("C_NOTICE", (await notice()).replace(/\s+/g, " ").slice(0, 160));

// ---------- Phase D: FE-02.V2 runtime B adoption (legacy CPU after measured A) ----------
await nav("Runtime");
await settle(600);
console.log("D_SET_PATH", await setPathInput(LEGACY_CPU));
await settle(300);
console.log("D_INSPECT", await clickText("Inspect"));
for (let i = 0; i < 40; i += 1) { await settle(1000); const n = await notice(); if (n && /nspec|fail|not/i.test(n) && !/Started/.test(n)) break; }
console.log("D_NOTICE", (await notice()).replace(/\s+/g, " ").slice(0, 160));
// stop the current (A) server first if live, then adopt B
await nav("Profile");
await settle(400);
console.log("D_STOP", await clickText("Stop server"));
await settle(2500);
console.log("D_SAVE", await clickText("Save"));
await settle(600);
console.log("D_START", await clickText("Start"));
let liveB = false;
for (let i = 0; i < 60 && !liveB; i += 1) { await settle(1000); const text = await bodyText(); liveB = /LIVE/.test(text) && !/Starting/.test(text); }
console.log("D_LIVE", liveB);
const dText = await bodyText();
const usesCpu = /cpu-|\bcpu\b/i.test(dText) && !/cuda-13\.3/i.test(dText.slice(0, 2000));
console.log("D_STATUS_SLICE", dText.slice(0, 600));
console.log("D_ADOPTED_CPU_ONLY", usesCpu);
console.log("D_DONE");
await client.close();
process.exit(0);
