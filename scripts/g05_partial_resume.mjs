// G-05 partial resume, row-scoped: click Resume on the Qwen row only, and
// measure progress through the .part.json sidecar's completed-byte sum (the
// .part file is preallocated to full size, so file size cannot show progress).
// Usage: node scripts/g05_partial_resume.mjs <debugPort> <scratchDir>
import { attach } from "./lib/cdp_client.mjs";
import { readFile } from "node:fs/promises";
import { join } from "node:path";

const [portArg, scratch] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const sidecar = join(scratch, "Qwen3.8-27B-Q4_0.gguf.part.json");
const doneSum = async () => {
  const doc = JSON.parse(await readFile(sidecar, "utf8"));
  return doc.chunks.reduce((total, chunk) => total + chunk.done, 0);
};

const before = await doneSum();
console.log("DONE BEFORE:", before);

const clicked = await evaluate(`(() => {
  const row = [...document.querySelectorAll(".catalog-file-row")].find(
    (entry) => /Qwen3\\.8-27B-Q4_0/.test(entry.textContent ?? ""),
  );
  if (!row) return "no-row";
  const button = [...row.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "Resume" && !candidate.disabled,
  );
  if (!button) return "no-resume";
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return "clicked";
})()`);
console.log("QWEN RESUME:", clicked);

let grew = false;
let last = before;
for (let attempt = 0; attempt < 40; attempt += 1) {
  await settle(1500);
  try {
    last = await doneSum();
  } catch {
    // sidecar momentarily replaced
  }
  if (last > before + 2 * 1024 * 1024) {
    grew = true;
    break;
  }
}
console.log("DONE AFTER:", last, "delta:", last - before);

// Stop the row again to leave a bounded state.
await evaluate(`(() => {
  const row = [...document.querySelectorAll(".catalog-file-row")].find(
    (entry) => /Qwen3\\.8-27B-Q4_0/.test(entry.textContent ?? ""),
  );
  if (!row) return false;
  const button = [...row.querySelectorAll("button")].find(
    (candidate) => /Keep & stop|Keep and stop/.test((candidate.textContent ?? "").trim()) && !candidate.disabled,
  );
  if (button) button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
await settle(2000);
console.log(`G05_RESUME ${grew ? "PASS" : "FAIL"} delta=${last - before}`);
await client.close?.();
process.exit(grew ? 0 : 1);
