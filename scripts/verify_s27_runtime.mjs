// Packaged S-27 runtime-screen probe: the extracted RuntimeScreen must render
// the same user-visible surface as the pre-extraction App JSX, on the real
// WebView2 binary. Assertions use public DOM text and accessible names only.
//
// Usage: node scripts/verify_s27_runtime.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";

const [portArg] = process.argv.slice(2);
if (!portArg) {
  console.error("usage: verify_s27_runtime.mjs <debugPort>");
  process.exit(2);
}
const client = await attach(Number(portArg));

const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok });
  console.log(`${ok ? "PASS" : "FAIL"} ${name}${detail ? ` — ${detail}` : ""}`);
};
const settle = (ms = 400) => new Promise((resolve) => setTimeout(resolve, ms));
const evaluate = (expr) => client.evaluate(expr);
const text = () => evaluate("document.body.innerText");

const clickNav = (label) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === ${JSON.stringify(label)},
    );
    if (!button) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);

// 1. Open the Runtime screen through its public navigation entry.
const opened = await clickNav("Runtime");
await settle(600);
check("Runtime nav opens the screen", opened === true);

// 2. The heading and subtitle render (pre-extraction copy).
const bodyText = await text();
// innerText reflects CSS text-transform, and the design system uppercases
// headings, so compare case-insensitively.
const bodyLower = bodyText.toLowerCase();
check("Runtime manager heading present", bodyLower.includes("runtime manager"));
check(
  "Runtime subtitle present",
  bodyLower.includes("detect this windows pc") && bodyLower.includes("keep llama.cpp updateable"),
);

// 3. The refresh action exposes an accessible name.
const refresh = await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").includes("Check latest release"),
  );
  return button ? { text: button.textContent.trim(), disabled: button.disabled } : null;
})()`);
check("Check latest release action present", refresh !== null, refresh ? refresh.text : "missing");

// 4. The managed-runtime section renders its heading.
check("Managed runtime panel present", bodyLower.includes("managed runtime"));

// 5. The install/choose actions are reachable.
const actions = await evaluate(`(() => {
  const labels = [...document.querySelectorAll("button")].map((b) => (b.textContent ?? "").trim());
  return {
    hasInspect: labels.some((l) => l.includes("Inspect") || l.includes("Use this")),
    hasChoose: labels.some((l) => l.includes("Choose") || l.includes("Browse")),
  };
})()`);
check(
  "Path inspection/selection actions reachable",
  actions && (actions.hasInspect || actions.hasChoose),
  JSON.stringify(actions),
);

// 6. No uncaught exception was logged while the screen rendered.
const exceptions = client.exceptions ?? [];
check("No runtime-screen exceptions", exceptions.length === 0, JSON.stringify(exceptions.slice(0, 2)));

const failed = results.filter((r) => !r.ok);
console.log(
  `\nS27_RESULT ${failed.length === 0 ? "PASS" : "FAIL"} (${results.length} checks)` +
    (failed.length ? ` — failing: ${failed.map((f) => f.name).join("; ")}` : ""),
);
await client.close?.();
process.exit(failed.length === 0 ? 0 : 1);
