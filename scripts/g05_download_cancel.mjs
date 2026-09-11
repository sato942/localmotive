// G-05 download-cancellation driver: start a real catalog download into a
// scratch folder, cancel it in flight through the UI control, and record the
// cancellation semantics (partial retained, job stopped, no further writes).
// Usage: node scripts/g05_download_cancel.mjs <debugPort> <scratchFolder>
import { attach } from "./lib/cdp_client.mjs";
import { readdir, stat } from "node:fs/promises";
import { join } from "node:path";

const [portArg, scratch] = process.argv.slice(2);
if (!portArg || !scratch) {
  console.error("usage: g05_download_cancel.mjs <debugPort> <scratchFolder>");
  process.exit(2);
}
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

await clickNav("HF Catalog");
await settle(1500);

// Point downloads at the scratch folder.
const setRoot = await evaluate(`(() => {
  const input = [...document.querySelectorAll(".catalog-screen input")].find(
    (candidate) => (candidate.placeholder ?? "").startsWith("Choose where"),
  );
  if (!input) return false;
  const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
  setter.call(input, ${JSON.stringify(scratch)});
  input.dispatchEvent(new Event("input", { bubbles: true }));
  return true;
})()`);
console.log("SCRATCH ROOT SET:", setRoot);
await settle(600);

// Find the smallest offered file by matching MiB-scale rows (small quants and
// projector files), else fall back to the first row with a Download button.
const target = await evaluate(`(() => {
  const rows = [...document.querySelectorAll(".catalog-file-row")];
  const withButton = rows.filter((row) =>
    [...row.querySelectorAll("button")].some((button) => (button.textContent ?? "").includes("Download")),
  );
  const miB = withButton.find((row) => /MiB/.test(row.textContent ?? ""));
  const row = miB ?? withButton[0];
  if (!row) return null;
  return { text: row.textContent.replace(/\\s+/g, " ").slice(0, 200) };
})()`);
console.log("TARGET ROW:", JSON.stringify(target));

const started = await evaluate(`(() => {
  const rows = [...document.querySelectorAll(".catalog-file-row")];
  const withButton = rows.filter((row) =>
    [...row.querySelectorAll("button")].some((button) => (button.textContent ?? "").includes("Download")),
  );
  const miB = withButton.find((row) => /MiB/.test(row.textContent ?? ""));
  const row = miB ?? withButton[0];
  if (!row) return false;
  const button = [...row.querySelectorAll("button")].find((candidate) =>
    (candidate.textContent ?? "").includes("Download"),
  );
  if (!button || button.disabled) return false;
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
console.log("DOWNLOAD CLICKED:", started);

// Wait until a cancel control appears (the download must be in flight).
let cancelLabel = null;
for (let attempt = 0; attempt < 40; attempt += 1) {
  await settle(attempt < 8 ? 1500 : 500);
  cancelLabel = await evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => /Keep & stop|Keep and stop|Cancel/.test((candidate.textContent ?? "").trim()) && !candidate.disabled,
    );
    return button ? button.textContent.trim() : null;
  })()`);
  if (cancelLabel) break;
}
console.log("CANCEL CONTROL:", cancelLabel);

const sizeBefore = [];
try {
  for (const name of await readdir(scratch)) {
    sizeBefore.push([name, (await stat(join(scratch, name))).size]);
  }
} catch {
  // folder may not exist yet
}

if (cancelLabel) {
  const clicked = await evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => /Keep & stop|Keep and stop|Cancel/.test((candidate.textContent ?? "").trim()) && !candidate.disabled,
    );
    if (!button) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);
  console.log("CANCEL CLICKED:", clicked);
  await settle(4000);
  const state = await evaluate(`(() => {
    const text = document.body.textContent || "";
    const match = text.match(/[^.\\n]*(stopped|cancelled|canceled|partial|kept)[^.\\n]*/i);
    return match ? match[0].replace(/\\s+/g, " ").slice(0, 220) : null;
  })()`);
  console.log("POST-CANCEL NOTICE:", state);
}

const sizes = [];
try {
  for (const name of await readdir(scratch)) {
    sizes.push([name, (await stat(join(scratch, name))).size]);
  }
} catch {
  // ignore
}
console.log("SCRATCH BEFORE:", JSON.stringify(sizeBefore));
console.log("SCRATCH AFTER:", JSON.stringify(sizes));
const frozen = JSON.parse(JSON.stringify(sizes));
await settle(4000);
const sizes2 = [];
try {
  for (const name of await readdir(scratch)) {
    sizes2.push([name, (await stat(join(scratch, name))).size]);
  }
} catch {
  // ignore
}
console.log("SCRATCH LATER:", JSON.stringify(sizes2));
const still = JSON.stringify(frozen) === JSON.stringify(sizes2);
console.log(`G05_DOWNLOAD_CANCEL ${cancelLabel ? (still ? "STOPPED_AND_STABLE" : "STILL_WRITING") : "NO_CANCEL_WINDOW"}`);
await client.close?.();
process.exit(0);
