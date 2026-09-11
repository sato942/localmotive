// G-05 packaged managed-install driver: with the legacy GGUF Pilot runtime
// tree parked, the CUDA row must offer a real Install; drive it to completion
// and record the install states, the primary-root outcome and the digest
// verification behavior. Usage: node scripts/verify_g05_managed_install.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";

const [portArg] = process.argv.slice(2);
if (!portArg) {
  console.error("usage: verify_g05_managed_install.mjs <debugPort>");
  process.exit(2);
}
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const readRow = (label) =>
  evaluate(`(() => {
    const node = [...document.querySelectorAll("article.runtime-option")].find(
      (entry) => (entry.querySelector("strong")?.textContent ?? "").trim() === ${JSON.stringify(label)},
    );
    if (!node) return null;
    const button = node.querySelector("button");
    return {
      cls: node.className,
      button: (button?.textContent ?? "").trim(),
      disabled: Boolean(button?.disabled),
      text: node.textContent.replace(/\\s+/g, " ").slice(0, 240),
      alert: [...document.querySelectorAll("[role=alert], .warning-band")].map((n) => n.textContent.replace(/\\s+/g, " ").slice(0, 200)).slice(0, 3),
    };
  })()`);

const clickNav = (label) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === ${JSON.stringify(label)},
    );
    if (!button) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);

await clickNav("Runtime");
await settle(800);
const initial = await readRow("NVIDIA CUDA 13.3");
console.log("INITIAL:", JSON.stringify(initial));

if (!initial || !initial.button.includes("Install")) {
  console.log("G05_MANAGED_INSTALL BLOCKED — CUDA row is not installable:", initial ? initial.button : "row missing");
  await client.close?.();
  process.exit(3);
}

const clicked = await evaluate(`(() => {
  const node = [...document.querySelectorAll("article.runtime-option")].find(
    (entry) => (entry.querySelector("strong")?.textContent ?? "").trim() === "NVIDIA CUDA 13.3",
  );
  const button = node?.querySelector("button");
  if (!button) return false;
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
console.log("CLICKED:", clicked);

let outcome = null;
let lastText = "";
for (let attempt = 0; attempt < 600; attempt += 1) {
  await settle(3000);
  const row = await readRow("NVIDIA CUDA 13.3");
  if (!row) {
    console.log(`[${attempt}] row disappeared`);
    continue;
  }
  const line = `[${attempt}] cls=${row.cls} btn=${row.button} text=${row.text.slice(0, 120)}`;
  if (line !== lastText) {
    console.log(line);
    lastText = line;
  }
  if (row.cls.includes("is-active") || row.text.includes("ACTIVE")) {
    outcome = "ACTIVE";
    break;
  }
  if (/failed|Failed/.test(row.text) || /failed|Failed/.test((row.alert ?? []).join(" "))) {
    outcome = "FAILED";
    console.log("ALERTS:", JSON.stringify(row.alert));
    break;
  }
}

const finalRow = await readRow("NVIDIA CUDA 13.3");
console.log("FINAL:", JSON.stringify(finalRow));
console.log(`\nG05_MANAGED_INSTALL ${outcome === "ACTIVE" ? "PASS" : outcome ?? "TIMEOUT"}`);
await client.close?.();
process.exit(outcome === "ACTIVE" ? 0 : 1);
