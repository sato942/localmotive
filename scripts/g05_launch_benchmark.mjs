// G-05 launch + warm benchmark driver: select the SmolLM2 health model from
// the inventory, start the managed CUDA runtime through the Profile screen,
// wait for the running state, then run the warm benchmark and report the
// measured numbers. This is target evidence (RT-02.V3 / G-05.I2).
// Usage: node scripts/g05_launch_benchmark.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";

const [portArg] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const clickNav = (label) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === ${JSON.stringify(label)},
    );
    if (!button) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);

const clickContains = (needle) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").includes(${JSON.stringify(needle)}),
    );
    if (!button || button.disabled) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);

const bodyText = () => evaluate("document.body.textContent || \"\"");

// 1. Select the model if the profile is not already loaded.
await clickNav("Profile");
await settle(1200);
let text = await bodyText();
if (text.includes("No models to profile yet") || text.includes("No model selected")) {
  await clickNav("Inventory");
  await settle(1200);
  const picked = await evaluate(`(() => {
    const rows = [...document.querySelectorAll("tbody tr")];
    const row = rows.find((entry) => (entry.textContent ?? "").includes("SmolLM2")) ?? rows[0];
    if (!row) return false;
    row.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return Boolean(row.textContent);
  })()`);
  console.log("MODEL PICKED:", picked);
  await settle(1500);
  await clickNav("Profile");
  await settle(1500);
}

// 2. Start the managed runtime.
const started = await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "Start",
  );
  if (!button || button.disabled) return false;
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
console.log("START CLICKED:", started);
let running = false;
for (let attempt = 0; attempt < 150; attempt += 1) {
  await settle(2000);
  text = await bodyText();
  if (/Stop server|Cancel start/.test(text)) {
    running = true;
    break;
  }
  // Only a terminal failure in the status line stops the wait; the page
  // contains the word "error" in unrelated copy (always-mounted evidence
  // panel), so broad matching would break the wait immediately.
  const statusLine = await evaluate(`(() => {
    const node = document.querySelector(".status-line, .server-status, footer");
    return node ? node.textContent.replace(/\s+/g, " ") : "";
  })()`);
  if (/failed|exited|refused/i.test(statusLine)) {
    console.log("STATUS LINE:", statusLine.slice(0, 200));
    break;
  }
}
const runningMatch = text.match(/[^.\n]*(llama\.cpp b\d+|Running|Stop server)[^.\n]*/i);
console.log("RUNNING:", running, runningMatch ? runningMatch[0].replace(/\s+/g, " ").slice(0, 160) : "");

// 3. Warm benchmark.
if (running) {
  await clickNav("Benchmark");
  await settle(1500);
  const benchClicked = await clickContains("Run benchmark");
  console.log("BENCHMARK CLICKED:", benchClicked);
  let result = null;
  for (let attempt = 0; attempt < 180; attempt += 1) {
    await settle(2000);
    const benchText = await evaluate(`(() => {
      const node = document.querySelector(".results-panel");
      return node ? node.textContent.replace(/\\s+/g, " ") : "";
    })()`);
    const numbers = benchText.match(/mean|Median|Range/i);
    if (numbers && /tok\/s/.test(benchText)) {
      result = benchText;
      break;
    }
  }
  console.log("BENCH RESULT:", result ?? "none");
  console.log(`G05_BENCH ${result ? "PASS" : "FAIL"}`);
} else {
  console.log("G05_BENCH FAIL — server did not reach running");
}

await client.close?.();
process.exit(0);
