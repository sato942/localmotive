// G-05 active-cancellation driver (audit G-05.I1): cancel a server start in
// flight, then cancel a running benchmark, and report the observable states.
// Usage: node scripts/g05_cancellation.mjs <debugPort>
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

const bodyText = () => evaluate("document.body.textContent || \"\"");
const hasButton = (needle) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").includes(${JSON.stringify(needle)}) && !candidate.disabled,
    );
    return Boolean(button);
  })()`);
const clickButton = (needle) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").includes(${JSON.stringify(needle)}) && !candidate.disabled,
    );
    if (!button) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);

// 0. Make sure a model is selected (scan the health-model folder if needed).
await clickNav("Profile");
await settle(1200);
let text = await bodyText();
if (text.includes("No models to profile yet") || text.includes("No model selected")) {
  await clickNav("Inventory");
  await settle(1200);
  await evaluate(`(() => {
    const input = document.querySelector('input[aria-label="Model root"]');
    if (!input) return false;
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
    setter.call(input, "C:/Users/Mubarak/AppData/Local/Localmotive/health-models/e3131339bf4e8065265593d4fd8f7bb7ff2d3abff1edb5618aa1197b89cad9f5");
    input.dispatchEvent(new Event("input", { bubbles: true }));
    return true;
  })()`);
  await clickButton("Rescan");
  for (let attempt = 0; attempt < 20; attempt += 1) {
    await settle(1500);
    const picked = await evaluate(`(() => {
      const rows = [...document.querySelectorAll("tbody tr")];
      const row = rows.find((entry) => (entry.textContent ?? "").includes("SmolLM2")) ?? rows[0];
      if (!row) return false;
      row.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      return true;
    })()`);
    if (picked) break;
  }
  await settle(1200);
  await clickNav("Profile");
  await settle(1500);
}

// 1. Cancel a server start in flight.
const startClicked = await clickButton("Start");
console.log("START CLICKED:", startClicked);
let cancelSeen = false;
for (let attempt = 0; attempt < 40; attempt += 1) {
  await settle(300);
  if (await hasButton("Cancel start")) {
    cancelSeen = true;
    break;
  }
  // If the server became live before we could cancel, stop here.
  text = await bodyText();
  if (/Stop server/.test(text)) break;
}
if (cancelSeen) {
  await clickButton("Cancel start");
  await settle(2500);
  text = await bodyText();
  const cancelled = /cancel/i.test(text.match(/[^.\n]*(cancel)[^.\n]*/i)?.[0] ?? "");
  console.log("START CANCELLED:", cancelled, (text.match(/[^.\n]*cancel[^.\n]*/i)?.[0] ?? "").slice(0, 140));
} else {
  console.log("START CANCEL WINDOW NOT CAUGHT (server may have started too quickly)");
}
// Ensure no server is left running before the benchmark leg.
text = await bodyText();
if (/Stop server/.test(text)) {
  await clickButton("Stop server");
  for (let attempt = 0; attempt < 20; attempt += 1) {
    await settle(1000);
    text = await bodyText();
    if (!/Stop server/.test(text)) break;
  }
}

// 2. Start normally, then cancel a running benchmark.
await clickButton("Start");
let live = false;
for (let attempt = 0; attempt < 60; attempt += 1) {
  await settle(2000);
  text = await bodyText();
  if (/Stop server/.test(text)) {
    live = true;
    break;
  }
}
console.log("SERVER LIVE:", live);
if (live) {
  await clickNav("Benchmark");
  await settle(1200);
  const benchClicked = await clickButton("Run benchmark");
  console.log("BENCH CLICKED:", benchClicked);
  let benchCancelSeen = false;
  for (let attempt = 0; attempt < 60; attempt += 1) {
    await settle(250);
    const cancel = await evaluate(`(() => {
      const button = [...document.querySelectorAll("button")].find(
        (candidate) => /cancel/i.test(candidate.textContent ?? "") && !candidate.disabled,
      );
      if (!button) return null;
      return (button.textContent ?? "").trim();
    })()`);
    if (cancel) {
      benchCancelSeen = true;
      await evaluate(`(() => {
        const button = [...document.querySelectorAll("button")].find(
          (candidate) => /cancel/i.test(candidate.textContent ?? "") && !candidate.disabled,
        );
        if (button) button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
        return true;
      })()`);
      console.log("BENCH CANCEL CLICKED:", cancel);
      break;
    }
    if (await hasButton("Run benchmark")) break;
  }
  await settle(4000);
  text = await bodyText();
  const outcome = text.match(/[^.\n]*(cancelled|canceled)[^.\n]*/i);
  console.log("BENCH CANCEL SEEN:", benchCancelSeen, "OUTCOME:", outcome ? outcome[0].slice(0, 160) : "none");
}

await client.close?.();
process.exit(0);
