// Packaged S-27 tune-screen probe: the extracted TuneScreen must render the
// same user-visible surface as the pre-extraction App JSX, on the real
// WebView2 binary. Assertions use public DOM text and accessible names only.
//
// Usage: node scripts/verify_s27_tune.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";

const [portArg] = process.argv.slice(2);
if (!portArg) {
  console.error("usage: verify_s27_tune.mjs <debugPort>");
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

// 1. Open the Tune screen through its public navigation entry.
const opened = await clickNav("AI Tune");
await settle(600);
check("AI Tune nav opens the screen", opened === true);

// Poll until the screen content has painted (the nav click returns before
// the WebView2 frame settles).
let bodyLower = "";
for (let attempt = 0; attempt < 20; attempt += 1) {
  const raw = await evaluate("document.body.textContent || \"\"");
  bodyLower = raw.toLowerCase();
  if (bodyLower.includes("ai tuning") && bodyLower.includes("this pc measures them")) break;
  await settle(250);
}

// 2. The screen heading and readiness ladder render.
check(
  "Tune heading present",
  bodyLower.includes("ai tuning") && bodyLower.includes("this pc measures them"),
  bodyLower.slice(0, 80).replace(/\s+/g, " "),
);

// 3. The cloud provider control and its accessible tablist are reachable.
const providerState = await evaluate(`(() => {
  const tablist = document.querySelector('[role="tablist"]');
  const tabs = tablist ? tablist.querySelectorAll('[role="tab"]').length : 0;
  const keyInput = document.querySelector('input[type="password"]');
  return { tabs, hasKeyInput: Boolean(keyInput) };
})()`);
check(
  "Provider tabs and key input reachable",
  providerState && providerState.tabs >= 1 && providerState.hasKeyInput,
  JSON.stringify(providerState),
);

// 4. The disclosure radio pair renders with the minimal option explained.
const disclosure = await evaluate(`(() => {
  const radios = [...document.querySelectorAll('input[name="tune-disclosure"]')];
  return {
    count: radios.length,
    checked: radios.filter((r) => r.checked).length,
    labels: radios.map((r) => (r.closest("label")?.textContent ?? "").slice(0, 24)),
  };
})()`);
check(
  "Disclosure radios render with one selection",
  disclosure && disclosure.count === 2 && disclosure.checked === 1,
  JSON.stringify(disclosure),
);

// 5. The Auto-tune action renders and reflects readiness.
const autotune = await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").includes("Auto-tune"),
  );
  return button ? { text: button.textContent.trim().slice(0, 60), disabled: button.disabled } : null;
})()`);
check("Auto-tune action present", autotune !== null, autotune ? JSON.stringify(autotune) : "missing");

// 6. No uncaught exception was logged while the screen rendered.
const exceptions = client.exceptions ?? [];
check("No tune-screen exceptions", exceptions.length === 0, JSON.stringify(exceptions.slice(0, 2)));

const failed = results.filter((r) => !r.ok);
console.log(
  `\nS27TUNE_RESULT ${failed.length === 0 ? "PASS" : "FAIL"} (${results.length} checks)` +
    (failed.length ? ` — failing: ${failed.map((f) => f.name).join("; ")}` : ""),
);
await client.close?.();
process.exit(failed.length === 0 ? 0 : 1);
