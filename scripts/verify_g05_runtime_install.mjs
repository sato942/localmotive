// G-05 packaged runtime install driver: fetch the official runtime catalog
// through the real UI, install the recommended option, and record the states
// the audit's target-verification items need (install -> inspect -> launch
// preservation). Runs against the packaged binary over CDP.
//
// Usage: node scripts/verify_g05_runtime_install.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";

const [portArg] = process.argv.slice(2);
if (!portArg) {
  console.error("usage: verify_g05_runtime_install.mjs <debugPort>");
  process.exit(2);
}
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms = 500) => new Promise((resolve) => setTimeout(resolve, ms));

const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok });
  console.log(`${ok ? "PASS" : "FAIL"} ${name}${detail ? ` — ${detail}` : ""}`);
};

const clickNav = (label) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === ${JSON.stringify(label)},
    );
    if (!button) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);

const clickText = (needle) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").includes(${JSON.stringify(needle)}),
    );
    if (!button) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);

// 1. Runtime screen.
check("Runtime nav opens", (await clickNav("Runtime")) === true);
await settle(700);

// 2. Fetch the official runtime catalog through the UI.
check("Check latest release pressed", (await clickText("Check latest release")) === true);
let catalogText = "";
for (let attempt = 0; attempt < 120; attempt += 1) {
  catalogText = await evaluate("document.body.textContent || \"\"");
  if (/b\d{4}/.test(catalogText) || catalogText.includes("No approved runtime")) break;
  await settle(1000);
}
const hasTag = /b\d{4}/.test(catalogText);
check("Runtime catalog fetched", hasTag, hasTag ? catalogText.match(/b\d{4}/)[0] : catalogText.slice(0, 80));

// 3. Report the offered options (no fabrication: read the DOM). The tag can
// appear in the status line before the option rows paint, so poll for rows.
let options = [];
for (let attempt = 0; attempt < 60; attempt += 1) {
  options = await evaluate(`(() => {
  return [...document.querySelectorAll("article.runtime-option")].map((node) => ({
    label: (node.querySelector("strong")?.textContent ?? "").trim(),
    recommended: node.className.includes("recommended"),
    active: node.className.includes("is-active"),
    button: (node.querySelector("button")?.textContent ?? "").trim(),
  }));
})()`);
  if (options && options.length > 0) break;
  await settle(1000);
}
console.log("OPTIONS:", JSON.stringify(options, null, 1));

// 4. Install the recommended option when the catalog offers one.
if (options && options.length > 0) {
  const target = options.find((o) => o.recommended && !o.active) ?? options.find((o) => !o.active);
  if (target) {
    const clicked = await evaluate(`(() => {
      const node = [...document.querySelectorAll("article.runtime-option")].find((entry) =>
        (entry.querySelector("strong")?.textContent ?? "").trim() === ${JSON.stringify(target.label)},
      );
      const button = node?.querySelector("button");
      if (!button) return false;
      button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      return true;
    })()`);
    check("Install clicked for " + target.label, clicked === true);
    let lastText = "";
    let installed = false;
    for (let attempt = 0; attempt < 900; attempt += 1) {
      lastText = await evaluate("document.body.textContent || \"\"");
      if (lastText.includes("ACTIVE")) {
        installed = true;
        break;
      }
      if (lastText.includes("failed") || lastText.includes("Failed") || lastText.includes("error")) {
        break;
      }
      await settle(2000);
    }
    check("Runtime installed and active", installed, lastText.slice(-160).replace(/\s+/g, " "));
    if (!installed) {
      console.log("TAIL:", lastText.slice(-600).replace(/\s+/g, " "));
    }
  } else {
    check("Install clicked", false, "no installable option offered");
  }
} else {
  check("Install clicked", false, "catalog offered no options");
}

const failed = results.filter((r) => !r.ok);
console.log(`\nG05_INSTALL ${failed.length === 0 ? "PASS" : "FAIL"} (${results.length} checks)` +
  (failed.length ? ` — failing: ${failed.map((f) => f.name).join("; ")}` : ""));
await client.close?.();
process.exit(failed.length === 0 ? 0 : 1);
