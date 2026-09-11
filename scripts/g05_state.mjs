// G-05 managed-install state reporter (fast, one-shot).
// Usage: node scripts/g05_state.mjs <debugPort> [--click-install]
import { attach } from "./lib/cdp_client.mjs";

const [portArg, flag] = process.argv.slice(2);
if (!portArg) {
  console.error("usage: g05_state.mjs <debugPort> [--click-install]");
  process.exit(2);
}
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);

// Make sure the runtime screen is the active view before reading rows.
await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "Runtime",
  );
  if (button) button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
await new Promise((r) => setTimeout(r, 1200));

if (flag === "--click-install") {
  const clicked = await evaluate(`(() => {
    const node = [...document.querySelectorAll("article.runtime-option")].find(
      (entry) => (entry.querySelector("strong")?.textContent ?? "").trim() === "NVIDIA CUDA 13.3",
    );
    const button = node?.querySelector("button");
    if (!button || button.disabled) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);
  console.log("CLICKED:", clicked);
  await new Promise((r) => setTimeout(r, 2500));
}

const state = await evaluate(`(() => {
  const node = [...document.querySelectorAll("article.runtime-option")].find(
    (entry) => (entry.querySelector("strong")?.textContent ?? "").trim() === "NVIDIA CUDA 13.3",
  );
  const button = node?.querySelector("button");
  const progress = document.querySelector(".runtime-install-progress, .install-progress, progress");
  return {
    cls: node?.className ?? null,
    button: (button?.textContent ?? "").trim(),
    disabled: Boolean(button?.disabled),
    text: node ? node.textContent.replace(/\\s+/g, " ").slice(0, 200) : null,
    progress: progress ? progress.textContent.replace(/\\s+/g, " ").slice(0, 200) : null,
    alerts: [...document.querySelectorAll("[role=alert], .warning-band")]
      .map((n) => n.textContent.replace(/\\s+/g, " ").slice(0, 180))
      .slice(0, 4),
  };
})()`);
console.log(JSON.stringify(state, null, 1));
await client.close?.();
process.exit(0);
