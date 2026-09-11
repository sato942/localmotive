// G-05 active cancellation against the v2 benchmark run (the run that
// publishes a cancel handle to the app). Usage: node scripts/g05_v2_cancel.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";

const [portArg] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const v2Clicked = await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").includes("Run v2 benchmark") && !candidate.disabled,
  );
  if (!button) return false;
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
console.log("V2 RUN CLICKED:", v2Clicked);

let cancelClicked = false;
for (let attempt = 0; attempt < 60; attempt += 1) {
  await settle(700);
  const state = await evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === "Cancel",
    );
    return button ? { disabled: button.disabled } : null;
  })()`);
  if (state && !state.disabled) {
    cancelClicked = await evaluate(`(() => {
      const button = [...document.querySelectorAll("button")].find(
        (candidate) => (candidate.textContent ?? "").trim() === "Cancel",
      );
      if (!button || button.disabled) return false;
      button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      return true;
    })()`);
    console.log("CANCEL CLICKED at poll", attempt, ":", cancelClicked);
    break;
  }
}

await settle(5000);
const outcome = await evaluate(`(() => {
  const text = document.body.textContent || "";
  const match = text.match(/[^.\\n]*cancel[^.\\n]*/i);
  return {
    notice: match ? match[0].replace(/\\s+/g, " ").slice(0, 240) : null,
    resultPanel: document.querySelector(".evidence-panel")?.textContent?.replace(/\\s+/g, " ").slice(0, 320) ?? null,
  };
})()`);
console.log("OUTCOME:", JSON.stringify(outcome, null, 1));
console.log(`G05_V2_CANCEL ${cancelClicked ? "CLICKED" : "NO_WINDOW"}`);
await client.close?.();
process.exit(0);
