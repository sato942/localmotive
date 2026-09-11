// MT-01.V3 full chain: Inspect artifact -> include adapter -> Run preflight ->
// Run v2 benchmark (default workload: 1 warmup, 5 trials) and capture the
// distribution. Usage: node scripts/g05_mt01c.mjs <debugPort>
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

const noticeLine = () =>
  evaluate(`(() => {
    const lines = [...document.querySelectorAll(".notice-line")].map((node) =>
      (node.textContent ?? "").replace(/\\s+/g, " ").trim(),
    ).filter(Boolean);
    return lines.length ? lines[lines.length - 1].slice(0, 220) : null;
  })()`);

await clickExact("Benchmark");
await settle(1500);

console.log("inspect-artifact:", await clickExact("Inspect artifact"));
for (let attempt = 0; attempt < 30; attempt += 1) {
  await settle(1500);
  const line = await noticeLine();
  if (line && /artifact|inspect|GGUF|shape|tensor/i.test(line)) {
    console.log("artifact notice:", line);
    if (/inspected|ready|layer|context/i.test(line) && !/failed|error|missing/i.test(line)) break;
  }
}
console.log("notice after artifact:", await noticeLine());

// Include the first NVIDIA adapter explicitly if a checkbox is offered.
const adapter = await evaluate(`(() => {
  const rows = [...document.querySelectorAll("input[type=checkbox]")];
  const target = rows.find((node) => {
    const label = node.closest("label")?.textContent ?? node.parentElement?.textContent ?? "";
    return /5090|NVIDIA/i.test(label);
  });
  if (!target) return "no-adapter-checkbox";
  if (!target.checked) {
    target.click();
    return "checked";
  }
  return "already-checked";
})()`);
console.log("adapter:", adapter);
await settle(800);

console.log("preflight:", await clickExact("Run preflight"));
for (let attempt = 0; attempt < 40; attempt += 1) {
  await settle(1500);
  const line = await noticeLine();
  if (line) console.log(`[${attempt}]`, line);
  if (line && /preflight (passed|failed|is)|trust|policy/i.test(line)) break;
  if (attempt > 6 && line && !/preflight/i.test(line)) break;
}

console.log("v2:", await clickExact("Run v2 benchmark"));
const t0 = Date.now();
let done = null;
for (let attempt = 0; attempt < 120; attempt += 1) {
  await settle(2000);
  const state = await evaluate(`(() => {
    const text = document.body.textContent || "";
    const last = [...document.querySelectorAll(".notice-line")].map((node) =>
      (node.textContent ?? "").replace(/\\s+/g, " ").trim(),
    ).filter(Boolean).slice(-1)[0] ?? null;
    const idle = [...document.querySelectorAll("button")].some(
      (candidate) => (candidate.textContent ?? "").trim() === "Run v2 benchmark" && !candidate.disabled,
    );
    const stats = text.match(/[^.\\n]*(tok\\/s|Median|sampled|samples|trials)[^.\\n]*/i);
    return { last, idle, stats: stats ? stats[0].replace(/\\s+/g, " ").slice(0, 240) : null };
  })()`);
  if (attempt % 5 === 4) console.log(`v2 [${attempt}] ${(Date.now() - t0) / 1000}s`, state.last);
  if (state.idle && attempt > 4) {
    done = state;
    break;
  }
}
console.log("ELAPSED_S:", ((Date.now() - t0) / 1000).toFixed(1));
console.log("FINAL NOTICE:", done?.last);
console.log("STATS:", done?.stats);
const full = await evaluate(`(() => {
  const text = document.body.textContent || "";
  const i = text.indexOf("Decode throughput");
  return i < 0 ? null : text.slice(i, i + 420).replace(/\\s+/g, " ");
})()`);
console.log("RESULT REGION:", full);
console.log(`G05_MT01C ${done ? "DONE" : "TIMEOUT"}`);
await client.close?.();
process.exit(0);
