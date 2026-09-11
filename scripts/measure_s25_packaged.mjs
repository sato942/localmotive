// S-25 packaged performance measurements (audit S-25 I1/I2): catalog render
// and per-keystroke latency at the real catalog size, main-thread stalls
// during scan and typing, scan wall time and cancellation latency. Prints
// JSON lines; run against the packaged binary launched with the isolated
// verifier environment.
//
// Usage: node scripts/measure_s25_packaged.mjs <debugPort> <fixtureFolder>
import { attach } from "./lib/cdp_client.mjs";

const [portArg, fixtureFolder] = process.argv.slice(2);
if (!portArg || !fixtureFolder) {
  console.error("usage: measure_s25_packaged.mjs <debugPort> <fixtureFolder>");
  process.exit(2);
}
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms = 400) => new Promise((resolve) => setTimeout(resolve, ms));

const clickNav = (label) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === ${JSON.stringify(label)},
    );
    if (!button) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);
const waitFor = async (predicate, timeoutMs = 20_000) => {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    if (await predicate()) return true;
    if (Date.now() > deadline) return false;
    await settle(100);
  }
};

const startFrameProbe = () =>
  evaluate(`(() => {
    window.__frameTimes = [];
    window.__frameProbe = true;
    const tick = (timestamp) => {
      if (!window.__frameProbe) return;
      window.__frameTimes.push(timestamp);
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
    return true;
  })()`);
const stopFrameProbe = () =>
  evaluate(`(() => {
    window.__frameProbe = false;
    const times = window.__frameTimes ?? [];
    let maxGap = 0;
    for (let index = 1; index < times.length; index += 1) {
      maxGap = Math.max(maxGap, times[index] - times[index - 1]);
    }
    return { frames: times.length, maxGapMs: Math.round(maxGap * 10) / 10 };
  })()`);

// ---- 1) Catalog render at the real size -----------------------------------
await clickNav("HF Catalog");
const renderStarted = Date.now();
const rowsArrived = await waitFor(async () =>
  (await evaluate(`document.querySelectorAll("article.catalog-model").length`)) > 100,
);
const renderMs = Date.now() - renderStarted;
const rows = await evaluate(`document.querySelectorAll("article.catalog-model").length`);
console.log(
  `S25_CATALOG_RENDER ${JSON.stringify({ rowsArrived, renderMs, rows })}`,
);

// ---- 2) Per-keystroke filter latency --------------------------------------
const typing = await evaluate(`(async () => {
  const input = document.querySelector('input[placeholder*="Search"], input[type="search"], .catalog-search input, input[aria-label*="earch"]');
  if (!input) return { error: "no search input" };
  const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value");
  const timings = [];
  for (const value of ["q", "q4", "q4_", "q4_k"]) {
    const before = document.querySelectorAll("article.catalog-model").length;
    const started = performance.now();
    proto.set.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
    // Wait for the DOM to reflect a change (or 3 frames for same-count cases).
    await new Promise((resolve) => {
      let frames = 0;
      const check = () => {
        frames += 1;
        const after = document.querySelectorAll("article.catalog-model").length;
        if (after !== before || frames >= 90) resolve();
        else requestAnimationFrame(check);
      };
      requestAnimationFrame(check);
    });
    timings.push({ value, ms: Math.round((performance.now() - started) * 10) / 10 });
  }
  return { timings };
})()`);
console.log(`S25_CATALOG_TYPING ${JSON.stringify(typing)}`);

// ---- 3) Scan stalls, wall time, cancellation ------------------------------
await clickNav("Inventory");
await settle(300);
await evaluate(`(() => {
  const input = document.querySelector('input[aria-label="Model root"]');
  const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value");
  proto.set.call(input, ${JSON.stringify(fixtureFolder)});
  input.dispatchEvent(new Event("input", { bubbles: true }));
  return true;
})()`);
await settle(200);
await startFrameProbe();
const scanStarted = Date.now();
await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").includes("Rescan"),
  );
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
const scanDone = await waitFor(async () =>
  (await evaluate("document.body.innerText")).includes("logical targets indexed"),
);
const scanMs = Date.now() - scanStarted;
const scanFrames = await stopFrameProbe();
await clickNav("Inventory");
await settle(400);
const scanRows = await evaluate(
  `document.querySelectorAll("table.inventory-table tbody tr").length`,
);
console.log(
  `S25_SCAN ${JSON.stringify({ scanDone, scanMs, rows: scanRows, ...scanFrames })}`,
);

// Cancellation latency: start a rescan and cancel it immediately.
await startFrameProbe();
await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").includes("Rescan"),
  );
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
await settle(120);
const cancelStarted = Date.now();
await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").includes("Cancel scan"),
  );
  if (!button) return false;
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
const cancelDone = await waitFor(
  async () =>
    !(await evaluate("document.body.innerText")).includes("Cancel scan"),
  10_000,
);
const cancelMs = Date.now() - cancelStarted;
const cancelFrames = await stopFrameProbe();
console.log(
  `S25_CANCEL ${JSON.stringify({ cancelDone, cancelMs, ...cancelFrames })}`,
);

// ---- 4) Typing stalls on the editor screen (input responsiveness) ---------
await clickNav("Profile");
await settle(400);
await startFrameProbe();
const typeStalls = await evaluate(`(async () => {
  const input = document.querySelector('.path-bar input, input[aria-label="Model root"]')
    ?? document.querySelector("input[type=number]")
    ?? document.querySelector("input");
  if (!input) return { error: "no input" };
  const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value");
  const started = performance.now();
  for (let index = 0; index < 30; index += 1) {
    proto.set.call(input, "x".repeat(index + 1));
    input.dispatchEvent(new Event("input", { bubbles: true }));
    await new Promise((resolve) => requestAnimationFrame(resolve));
  }
  return { ms: Math.round(performance.now() - started) };
})()`);
await settle(300);
const typeFrames = await stopFrameProbe();
console.log(
  `S25_TYPING_STALLS ${JSON.stringify({ ...typeStalls, ...typeFrames })}`,
);

client.close();
