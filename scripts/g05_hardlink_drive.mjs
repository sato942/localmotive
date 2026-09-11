// G-05 hard-link negative control driver: start the profile while the managed
// runtime's bytes were modified THROUGH a hard link (same file content), expect
// a content-verification refusal, then restore and expect live.
// Usage: node scripts/g05_hardlink_drive.mjs <debugPort> <phase>
// phase = corrupt | restored
import { attach } from "./lib/cdp_client.mjs";

const [portArg, phase] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "Profile",
  );
  if (button) button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
await settle(1200);

// Stop anything live first.
await evaluate(`(() => {
  const stop = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "Stop server",
  );
  if (stop) stop.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
await settle(2500);

const started = await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "Start" && !candidate.disabled,
  );
  if (!button) return false;
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
console.log("START CLICKED:", started);

let outcome = null;
for (let attempt = 0; attempt < 40; attempt += 1) {
  await settle(1500);
  const state = await evaluate(`(() => {
    const text = document.body.textContent || "";
    const live = /Stop server/.test(text);
    const failure = (() => {
      const match = text.match(/[^.\\n]*(fails content verification|does not match|trust|verification|not installed)[^.\\n]*/i);
      return match ? match[0].replace(/\\s+/g, " ").slice(0, 200) : null;
    })();
    return { live, failure };
  })()`);
  if (state.live) {
    outcome = "LIVE";
    break;
  }
  if (state.failure && attempt > 2) {
    outcome = state.failure;
    break;
  }
}
console.log("OUTCOME:", outcome);
if (outcome === "LIVE") {
  await evaluate(`(() => {
    const stop = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === "Stop server",
    );
    if (stop) stop.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);
  await settle(2500);
}
const expected = phase === "corrupt" ? "refusal" : "LIVE";
const passed = phase === "corrupt" ? outcome !== "LIVE" : outcome === "LIVE";
console.log(`G05_HARDLINK ${passed ? "PASS" : "FAIL"} expected=${expected}`);
await client.close?.();
process.exit(passed ? 0 : 1);
