// MT-01.V3 packaged chain on the fixed binary: ensure live server, inspect
// artifact, run preflight, run the DEFAULT v2 workload (1 warmup, 5 trials),
// capture the distribution. Usage: node scripts/g05_mt01d.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";

const [portArg] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const clickExact = (label) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === ${JSON.stringify(label)} && !candidate.disabled,
    );
    if (!button) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);

const lastNotice = () =>
  evaluate(`(() => {
    const lines = [...document.querySelectorAll(".notice-line")].map((node) =>
      (node.textContent ?? "").replace(/\\s+/g, " ").trim(),
    ).filter(Boolean);
    return lines.length ? lines[lines.length - 1].slice(0, 240) : null;
  })()`);

await clickExact("Profile");
await settle(1500);
let live = await evaluate(`/Stop server/.test(document.body.textContent || "")`);
if (!live) {
  await clickExact("Start");
  for (let attempt = 0; attempt < 60; attempt += 1) {
    await settle(1500);
    live = await evaluate(`/Stop server/.test(document.body.textContent || "")`);
    if (live) break;
  }
}
console.log("SERVER LIVE:", live);
if (!live) {
  console.log("NOTICE:", await lastNotice());
  console.log("G05_MT01D NOT_RUN");
  await client.close?.();
  process.exit(1);
}

await clickExact("Benchmark");
await settle(1500);

console.log("inspect-artifact:", await clickExact("Inspect artifact"));
for (let attempt = 0; attempt < 40; attempt += 1) {
  await settle(1500);
  const line = await lastNotice();
  if (line && /artifact|inspect|GGUF|shape|context|layer|ready/i.test(line)) {
    if (!/failed|error|missing/i.test(line)) break;
  }
}
console.log("artifact notice:", await lastNotice());

console.log("preflight:", await clickExact("Run preflight"));
let preflightLine = null;
for (let attempt = 0; attempt < 60; attempt += 1) {
  await settle(1500);
  const line = await lastNotice();
  preflightLine = line;
  if (line && /preflight/i.test(line)) break;
}
console.log("preflight notice:", preflightLine);
const preflightPanel = await evaluate(`(() => {
  const text = document.body.textContent || "";
  const i = text.indexOf("Preflight class");
  return i < 0 ? null : text.slice(i, i + 260).replace(/\\s+/g, " ");
})()`);
console.log("preflight panel:", preflightPanel);

console.log("v2:", await clickExact("Run v2 benchmark"));
const t0 = Date.now();
let done = null;
for (let attempt = 0; attempt < 150; attempt += 1) {
  await settle(2000);
  const state = await evaluate(`(() => {
    const text = document.body.textContent || "";
    const last = [...document.querySelectorAll(".notice-line")].map((node) =>
      (node.textContent ?? "").replace(/\\s+/g, " ").trim(),
    ).filter(Boolean).slice(-1)[0] ?? null;
    const idle = [...document.querySelectorAll("button")].some(
      (candidate) => (candidate.textContent ?? "").trim() === "Run v2 benchmark" && !candidate.disabled,
    );
    const measured = /sampled|tok\\/s/.test(text) && !/No v2 result/.test(text);
    return { last, idle, measured };
  })()`);
  if (attempt % 10 === 9) console.log(`[${attempt}] ${((Date.now() - t0) / 1000).toFixed(0)}s`, state.last);
  if (state.idle && state.measured && attempt > 2) {
    done = state;
    break;
  }
  if (state.idle && attempt > 8 && !state.measured) {
    done = { ...state, measured: false };
    break;
  }
}
console.log("ELAPSED_S:", ((Date.now() - t0) / 1000).toFixed(1));
const region = await evaluate(`(() => {
  const text = document.body.textContent || "";
  const i = text.indexOf("Decode throughput");
  return i < 0 ? null : text.slice(i, i + 520).replace(/\\s+/g, " ");
})()`);
console.log("RESULT REGION:", region);
const samples = await evaluate(`(() => {
  const text = document.body.textContent || "";
  const m = text.match(/\\d+\\s*\\/\\s*\\d+\\s*sampled|sampled[^.]{0,80}/i);
  return m ? m[0].replace(/\\s+/g, " ") : null;
})()`);
console.log("SAMPLES:", samples);
console.log(`G05_MT01D ${done && done.measured ? "PASS" : "FAIL"}`);

await clickExact("Profile");
await settle(1200);
await clickExact("Stop server");
await settle(2500);
await client.close?.();
process.exit(done && done.measured ? 0 : 1);
