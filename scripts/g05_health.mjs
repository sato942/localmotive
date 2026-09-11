// G-05 health-run driver: clicks "Run 7-stage health" on the active managed
// runtime and reports the terminal state. Used for the baseline run, the
// tampered-bytes run and the restored run (audit RT-04 packaged evidence).
// Usage: node scripts/g05_health.mjs <debugPort> [label]
import { attach } from "./lib/cdp_client.mjs";

const [portArg, label = "run"] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "Runtime",
  );
  if (button) button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
await settle(1200);

const clicked = await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").includes("Run 7-stage health"),
  );
  if (!button || button.disabled) return false;
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
console.log(`[${label}] CLICKED:`, clicked);

let last = "";
let terminal = null;
for (let attempt = 0; attempt < 200; attempt += 1) {
  await settle(3000);
  const state = await evaluate(`(() => {
    const nodes = [...document.querySelectorAll(
      ".health-result, [role=alert], .warning-band, .health-stage, .runtime-option",
    )].map((n) => n.textContent.replace(/\\s+/g, " ").slice(0, 400));
    return {
      results: nodes.filter((t) => /PASS|FAIL|UNKNOWN|healthy|refus|mismatch|digest|changed|error/i.test(t)).slice(0, 6),
      buttons: [...document.querySelectorAll("button")].map((b) => (b.textContent ?? "").trim()).filter((t) => /health/i.test(t)).slice(0, 3),
    };
  })()`);
  const line = JSON.stringify(state);
  if (line !== last) {
    console.log(`[${label} ${attempt}]`, line.slice(0, 420));
    last = line;
  }
  const text = state.results.join(" | ");
  if (/PASS|FAIL|refus|mismatch|digest|changed/i.test(text)) {
    terminal = text;
    break;
  }
  if (state.buttons.length === 0) {
    terminal = "button gone — session ended";
    break;
  }
}
console.log(`[${label}] TERMINAL:`, terminal ?? "timeout");
console.log(`G05_HEALTH_${label.toUpperCase()} ${terminal && /PASS/i.test(terminal) ? "PASS" : "OTHER"}`);
await client.close?.();
process.exit(0);
