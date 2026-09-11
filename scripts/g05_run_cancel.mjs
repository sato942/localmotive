// G-05 active-cancellation driver, tight poll: click Run benchmark, catch the
// enabled Cancel within ~2 s, click it, and report the cancelled presentation.
// Usage: node scripts/g05_run_cancel.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";

const [portArg] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "Benchmark",
  );
  if (button) button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
await settle(1200);

// Slow the run down so the cancel window is realistic (forced output
// tokens 4096); the audit's cancellation item needs a run still in flight.
await evaluate(`(() => {
  const inputs = [...document.querySelectorAll("input[type=number]")];
  const target = inputs[0];
  if (!target) return false;
  const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
  setter.call(target, "4096");
  target.dispatchEvent(new Event("input", { bubbles: true }));
  return true;
})()`);
await settle(500);

const runClicked = await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").includes("Run benchmark") && !candidate.disabled,
  );
  if (!button) return false;
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
console.log("RUN CLICKED:", runClicked);

let cancelClicked = false;
let finishedEarly = false;
for (let attempt = 0; attempt < 120; attempt += 1) {
  await settle(100);
  const state = await evaluate(`(() => {
    const cancel = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === "Cancel",
    );
    const run = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").includes("Run benchmark"),
    );
    return { cancelEnabled: Boolean(cancel && !cancel.disabled), runEnabled: Boolean(run && !run.disabled) };
  })()`);
  if (state.cancelEnabled) {
    cancelClicked = await evaluate(`(() => {
      const cancel = [...document.querySelectorAll("button")].find(
        (candidate) => (candidate.textContent ?? "").trim() === "Cancel",
      );
      if (!cancel || cancel.disabled) return false;
      cancel.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      return true;
    })()`);
    console.log("CANCEL CLICKED at poll", attempt, ":", cancelClicked);
    break;
  }
  if (state.runEnabled) {
    finishedEarly = true;
    console.log("RUN FINISHED before cancel at poll", attempt);
    break;
  }
}

await settle(3500);
const outcome = await evaluate(`(() => {
  const text = document.body.textContent || "";
  const notice = text.match(/[^.\\n]*cancel[^.\\n]*/i);
  const panel = document.querySelector(".results-panel");
  return {
    notice: notice ? notice[0].replace(/\\s+/g, " ").slice(0, 220) : null,
    panel: panel ? panel.textContent.replace(/\\s+/g, " ").slice(0, 220) : null,
    cancelButtonNow: (() => {
      const cancel = [...document.querySelectorAll("button")].find(
        (candidate) => (candidate.textContent ?? "").trim() === "Cancel",
      );
      return cancel ? (cancel.disabled ? "disabled" : "enabled") : "absent";
    })(),
  };
})()`);
console.log("OUTCOME:", JSON.stringify(outcome, null, 1));
console.log(`G05_CANCEL ${cancelClicked ? "CLICKED" : finishedEarly ? "TOO_FAST" : "UNKNOWN"}`);
await client.close?.();
process.exit(0);
