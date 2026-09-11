// G-05 benchmark-over-TLS driver: with a TLS+api-key profile running, run the
// screen benchmark (which uses the app's own local client) and print the
// measured result. Usage: node scripts/g05_bench_tls.mjs <debugPort>
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
await settle(1400);

const clicked = await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").includes("Run benchmark") && !candidate.disabled,
  );
  if (!button) return false;
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
console.log("BENCH CLICKED:", clicked);

let result = null;
for (let attempt = 0; attempt < 40; attempt += 1) {
  await settle(2000);
  const panel = await evaluate(`(() => {
    const node = document.querySelector(".results-panel");
    return node ? node.textContent.replace(/\\s+/g, " ") : "";
  })()`);
  if (/tok\/s/.test(panel) && /Median/.test(panel)) {
    result = panel.slice(0, 260);
    break;
  }
  const failure = await evaluate(`(() => {
    const match = (document.body.textContent || "").match(/[^.\\n]*(TLS|certificate|unauthorized|401|refused|failed)[^.\\n]*/i);
    return match ? match[0].replace(/\\s+/g, " ").slice(0, 200) : null;
  })()`);
  if (failure) console.log(`[${attempt}]`, failure);
}
console.log("TLS BENCH RESULT:", result ?? "none");
console.log(`G05_TLS_BENCH ${result ? "PASS" : "FAIL"}`);
await client.close?.();
process.exit(0);
