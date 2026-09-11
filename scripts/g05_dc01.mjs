// G-05 / DC-01.V3 packaged sequence driver: with the app freshly restarted
// (within the cooldown window), open HF Catalog, read the cooldown/request
// presentation, and browse + filter a cached entry.
// Usage: node scripts/g05_dc01.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";

const [portArg] = process.argv.slice(2);
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
// The catalog may already be loaded from the earlier session; if not, refresh.
let text = await evaluate("document.body.textContent || \"\"");
if (text.includes("catalog is not loaded") || text.includes("not loaded")) {
  await evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").includes("Refresh catalog"),
    );
    if (button) button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);
  for (let attempt = 0; attempt < 60; attempt += 1) {
    await settle(1500);
    text = await evaluate("document.body.textContent || \"\"");
    if (/b\d{4}|models/.test(text)) break;
  }
}

const snapshot = await evaluate(`(() => {
  const rows = [...document.querySelectorAll(".catalog-file-row")];
  const heading = document.querySelector(".catalog-screen .section-heading")?.textContent ?? "";
  const notice = [...document.querySelectorAll("[role=status], .notice, .cooldown, .catalog-notice")]
    .map((n) => n.textContent.replace(/\\s+/g, " ").slice(0, 200))
    .filter((t) => /cooldown|refresh|next|cached|hours|minutes/i.test(t))
    .slice(0, 4);
  return {
    rowCount: rows.length,
    heading: heading.replace(/\\s+/g, " ").slice(0, 200),
    cooldownNotice: notice,
    firstRow: rows[0] ? rows[0].textContent.replace(/\\s+/g, " ").slice(0, 200) : null,
  };
})()`);
console.log("CATALOG SNAPSHOT:", JSON.stringify(snapshot, null, 1));

// Filter by a term taken from the first visible row (cached browse + filter).
if (snapshot.firstRow) {
  const term = (snapshot.firstRow.match(/[A-Za-z0-9][A-Za-z0-9._-]{3,}/) ?? ["qwen"])[0];
  const filtered = await evaluate(`(() => {
    const input = document.querySelector(".catalog-screen input[type=text]");
    if (!input) return { applied: false };
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
    setter.call(input, ${JSON.stringify(term)});
    input.dispatchEvent(new Event("input", { bubbles: true }));
    return { applied: true, term: ${JSON.stringify(term)} };
  })()`);
  await settle(900);
  const after = await evaluate(`(() => {
    const rows = [...document.querySelectorAll(".catalog-file-row")];
    return { rowCount: rows.length, first: rows[0] ? rows[0].textContent.replace(/\\s+/g, " ").slice(0, 160) : null };
  })()`);
  console.log("FILTER:", JSON.stringify({ ...filtered, after }, null, 1));
}
console.log("DC01_SEQUENCE DONE");
await client.close?.();
process.exit(0);
