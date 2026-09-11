// G-05 v2 cancellation outcome capture: run, cancel, then read the panel and
// status surfaces for the cancellation wording. Usage: node scripts/g05_v2_cancel2.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";

const [portArg] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").includes("Run v2 benchmark") && !candidate.disabled,
  );
  if (button) button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
await settle(800);
const clicked = await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "Cancel" && !candidate.disabled,
  );
  if (!button) return false;
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
console.log("CANCEL CLICKED:", clicked);

for (const wait of [1500, 3000, 6000]) {
  await settle(wait);
  const surfaces = await evaluate(`(() => {
    const panel = document.querySelector(".evidence-panel");
    const strip = document.body.textContent || "";
    const cancelWords = [];
    const re = /[^.\\n]*cancel(?:led|ed|ling)?[^.\\n]*/gi;
    let match;
    while ((match = re.exec(strip)) !== null && cancelWords.length < 6) {
      if (!/Custom|candidate|cancellable/.test(match[0])) cancelWords.push(match[0].replace(/\\s+/g, " ").slice(0, 150));
    }
    return {
      panelHead: panel ? panel.textContent.replace(/\\s+/g, " ").slice(0, 260) : null,
      cancelWords,
      runButton: (() => {
        const button = [...document.querySelectorAll("button")].find(
          (candidate) => (candidate.textContent ?? "").includes("Run v2 benchmark"),
        );
        return button ? (button.disabled ? "disabled" : "enabled") : "absent";
      })(),
    };
  })()`);
  console.log(`AFTER +${wait}ms:`, JSON.stringify(surfaces, null, 1));
}
await client.close?.();
process.exit(0);
