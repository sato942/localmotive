// MT-01.V3: run the DEFAULT v2 workload (warmups 1, trials 5) against the
// packaged approved runtime through the app's own client, and record the
// distribution. Usage: node scripts/g05_mt01_five_trials.mjs <debugPort>
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

// Ensure the server is live.
await clickNav("Profile");
await settle(1500);
let live = await evaluate(`/Stop server/.test(document.body.textContent || "")`);
if (!live) {
  await evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === "Start" && !candidate.disabled,
    );
    if (button) button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);
  for (let attempt = 0; attempt < 60; attempt += 1) {
    await settle(1500);
    live = await evaluate(`/Stop server/.test(document.body.textContent || "")`);
    if (live) break;
  }
}
console.log("SERVER LIVE:", live);
if (!live) {
  console.log("G05_MT01 NOT_RUN");
  await client.close?.();
  process.exit(1);
}

await clickNav("Benchmark");
await settle(1500);
const started = await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "Run v2 benchmark" && !candidate.disabled,
  );
  if (!button) return false;
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
console.log("V2 RUN CLICKED:", started);
const t0 = Date.now();

let result = null;
for (let attempt = 0; attempt < 90; attempt += 1) {
  await settle(2000);
  const panel = await evaluate(`(() => {
    const nodes = [...document.querySelectorAll(".results-panel, .evidence-status-card, .v2-result, section")];
    const text = nodes.map((node) => node.textContent ?? "").join(" ");
    const match = text.match(/[^.\\n]*(tok\\/s|Median|samples|trials|mean)[^.\\n]*/i);
    const hasRun = [...document.querySelectorAll("button")].some(
      (candidate) => (candidate.textContent ?? "").trim() === "Run v2 benchmark" && !candidate.disabled,
    );
    return { snippet: match ? match[0].replace(/\\s+/g, " ").slice(0, 240) : null, idle: hasRun };
  })()`);
  if (panel.snippet && panel.idle && attempt > 2) {
    result = panel.snippet;
    break;
  }
}
const elapsed = ((Date.now() - t0) / 1000).toFixed(1);
console.log("ELAPSED_S:", elapsed);
console.log("RESULT:", result ?? "none");

// Full result region for the record.
const full = await evaluate(`(() => {
  const node = [...document.querySelectorAll("section, div")].find(
    (entry) => /tok\\/s/.test(entry.textContent ?? "") && (entry.textContent ?? "").length < 1600,
  );
  return node ? node.textContent.replace(/\\s+/g, " ").slice(0, 900) : null;
})()`);
console.log("FULL:", full);

await clickNav("Profile");
await settle(1200);
await evaluate(`(() => {
  const stop = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "Stop server",
  );
  if (stop) stop.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
await settle(2500);
console.log(`G05_MT01 ${result ? "PASS" : "FAIL"}`);
await client.close?.();
process.exit(result ? 0 : 1);
