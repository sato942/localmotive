// G-05 partial-byte download retention: start a real catalog download, wait
// until the .part has bytes, click "Keep & stop", verify the partial and its
// sidecar survive with no further writes, then Resume and verify growth.
// Usage: node scripts/g05_partial_retention.mjs <debugPort> <scratchDir>
import { attach } from "./lib/cdp_client.mjs";
import { readdir, stat } from "node:fs/promises";
import { join } from "node:path";

const [portArg, scratch] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const listing = async () => {
  try {
    const out = [];
    for (const name of await readdir(scratch)) {
      out.push([name, (await stat(join(scratch, name))).size]);
    }
    return out;
  } catch {
    return [];
  }
};

const clickText = (matcher) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => /${matcher}/.test((candidate.textContent ?? "").trim()) && !candidate.disabled,
    );
    if (!button) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);

// Navigate + point downloads at the scratch folder.
await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "HF Catalog",
  );
  if (button) button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
await settle(1500);
await evaluate(`(() => {
  const input = [...document.querySelectorAll(".catalog-screen input")].find(
    (candidate) => (candidate.placeholder ?? "").startsWith("Choose where"),
  );
  if (!input) return false;
  const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
  setter.call(input, ${JSON.stringify(scratch)});
  input.dispatchEvent(new Event("input", { bubbles: true }));
  return true;
})()`);
await settle(800);

// Prefer the small model row with an offered download/resume control.
const clicked = await evaluate(`(() => {
  const rows = [...document.querySelectorAll(".catalog-file-row")];
  const fresh = rows.filter((entry) =>
    [...entry.querySelectorAll("button")].some((candidate) => (candidate.textContent ?? "").trim() === "Download" && !candidate.disabled),
  );
  const row = fresh[0];
  if (!row) return "no-fresh-row";
  const button = [...row.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "Download" && !candidate.disabled,
  );
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return button.textContent.trim() + " :: " + row.textContent.replace(/\s+/g, " ").slice(0, 90);
})()`);
console.log("DOWNLOAD CONTROL:", clicked);

// Wait until the .part on disk has real bytes.
let partName = null;
let baseline = 0;
for (let attempt = 0; attempt < 120; attempt += 1) {
  await settle(700);
  const files = await listing();
  const part = files.find(([name]) => name.endsWith(".part"));
  if (part && part[1] > 4 * 1024 * 1024) {
    partName = part[0];
    baseline = part[1];
    break;
  }
}
console.log("PART WITH BYTES:", partName, baseline);
if (!partName) {
  console.log("G05_PARTIAL NO_BYTES");
  await client.close?.();
  process.exit(1);
}

const cancelled = await clickText("^Keep & stop$|^Keep and stop$");
console.log("CANCEL CLICKED:", cancelled);
await settle(3000);
const after1 = (await listing()).find(([name]) => name === partName)?.[1] ?? 0;
await settle(3500);
const after2 = (await listing()).find(([name]) => name === partName)?.[1] ?? 0;
const sidecar = (await listing()).some(([name]) => name.endsWith(".json"));
const lock = (await listing()).some(([name]) => name.endsWith(".lm-lock"));
console.log(`AFTER CANCEL: part=${after1} -> later=${after2} sidecar=${sidecar} lock=${lock}`);
const notice = await evaluate(`(() => {
  const match = (document.body.textContent || "").match(/[^.\\n]*(kept|partial|stopped|cancelled)[^.\\n]*/i);
  return match ? match[0].replace(/\\s+/g, " ").slice(0, 180) : null;
})()`);
console.log("NOTICE:", notice);

// Resume: the partial must grow again when the user continues.
const resumed = await clickText("^Resume$");
console.log("RESUME CLICKED:", resumed);
let grew = false;
if (resumed) {
  for (let attempt = 0; attempt < 60; attempt += 1) {
    await settle(700);
    const size = (await listing()).find(([name]) => name === partName)?.[1] ?? 0;
    if (size > baseline + 1024 * 1024) {
      grew = true;
      break;
    }
  }
  await clickText("^Keep & stop$|^Keep and stop$");
}
console.log("RESUME GREW:", grew);

const stable = after2 === after1 && after1 >= baseline && sidecar;
const pass = stable && resumed ? grew : stable;
console.log(`G05_PARTIAL ${pass ? "PASS" : "FAIL"} retained=${after1} resumeGrew=${grew}`);
await client.close?.();
process.exit(pass ? 0 : 1);
