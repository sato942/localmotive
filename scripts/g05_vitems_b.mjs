// High-finding campaign B legs 1-2: FE-04.V2 slow-probe responsiveness via a
// sleeper swapped into the legacy runtime path, then the restore check.
// Usage: node scripts/g05_vitems_b.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";
import { copyFileSync, existsSync, mkdirSync } from "node:fs";

const [portArg] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const HOME = process.env.USERPROFILE ?? "C:\\Users\\Mubarak";
const LEGACY = `${HOME}\\AppData\\Local\\GGUF Pilot\\runtimes\\b10816\\cpu\\llama-server.exe`;
const BACKUP_DIR = `${process.cwd()}\\.hermes-0.6\\legacy-backup`;
const SLEEPER = `${process.cwd()}\\.hermes-0.6\\slow-runtime.exe`;

const clickText = (text) =>
  evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === ${JSON.stringify(text)} && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
const nav = (label) =>
  evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === ${JSON.stringify(label)} && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
const setPathInput = (value) =>
  evaluate(`(() => { const input = [...document.querySelectorAll("input")].find(x => (x.placeholder ?? "").includes("llama-server.exe")); if (!input) return false; const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value").set; setter.call(input, ${JSON.stringify(value)}); input.dispatchEvent(new Event("input", { bubbles: true })); return true; })()`);
const notices = () =>
  evaluate(`(() => [...document.querySelectorAll(".notice-line,.notice")].map(x => (x.textContent ?? "").trim()).filter(Boolean).slice(-3))()`);

// Swap the sleeper into the legacy path (backup the real exe once).
mkdirSync(BACKUP_DIR, { recursive: true });
const backup = `${BACKUP_DIR}\\llama-server-cpu.exe.orig`;
if (!existsSync(backup)) copyFileSync(LEGACY, backup);
copyFileSync(SLEEPER, LEGACY);
console.log("SWAPPED_SLEEPER_INTO_LEGACY");

await nav("Runtime");
await settle(800);
console.log("B1_SET_PATH", await setPathInput(LEGACY));
await settle(400);
const t0 = Date.now();
console.log("B1_INSPECT_CLICKED", await clickText("Inspect"));
// Interactive round-trips during the slow window.
await settle(600);
const tNav = Date.now(); await nav("Dashboard"); const navMs = Date.now() - tNav;
await settle(200);
const tBack = Date.now(); await nav("Runtime"); const backMs = Date.now() - tBack;
const tEval = Date.now(); await evaluate("(() => document.title)()"); const evalMs = Date.now() - tEval;
console.log(`B1_INTERACTIVE nav=${navMs}ms back=${backMs}ms eval=${evalMs}ms`);
// Wait for a NEW inspect-ish notice or 20 s.
let done = "";
for (let i = 0; i < 25; i += 1) {
  await settle(1000);
  const n = await notices();
  const last = n.slice(-1)[0] ?? "";
  if (/nspec|fail|invalid|not|unrecogn|exit/i.test(last)) { done = last; break; }
}
console.log(`B1_INSPECT_RESULT "${done.replace(/\s+/g, " ").slice(0, 180)}" ELAPSED_MS ${Date.now() - t0}`);
console.log("B1_NOTICES", JSON.stringify(await notices()));

// Restore the legacy exe and re-inspect to prove the normal path returns.
copyFileSync(backup, LEGACY);
console.log("RESTORED_LEGACY");
await settle(600);
console.log("B2_INSPECT_CLICKED", await clickText("Inspect"));
let done2 = "";
const t2 = Date.now();
for (let i = 0; i < 20; i += 1) {
  await settle(1000);
  const n = await notices();
  const last = n.slice(-1)[0] ?? "";
  if (/nspec/i.test(last)) { done2 = last; break; }
}
console.log(`B2_INSPECT_RESULT "${done2.replace(/\s+/g, " ").slice(0, 160)}" ELAPSED_MS ${Date.now() - t2}`);
console.log("G05_VITEMS_B DONE");
await client.close();
process.exit(0);
