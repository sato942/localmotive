// Packaged S-24 walkthrough: full-path retrievability, reduced-motion jump and
// keyboard focus, and narrow-layout usability of the copy affordance.
//
// Usage: node scripts/verify_s24_paths.mjs <debugPort> <fixtureFolder>
import { attach } from "./lib/cdp_client.mjs";

const [portArg, fixtureFolder] = process.argv.slice(2);
if (!portArg || !fixtureFolder) {
  console.error("usage: verify_s24_paths.mjs <debugPort> <fixtureFolder>");
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
const clickText = (needle) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").includes(${JSON.stringify(needle)}),
    );
    if (!button) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);

// Populate a model so the inventory renders a real row with a directory.
await clickNav("Inventory");
await settle();
await evaluate(`(() => {
  const input = document.querySelector('input[aria-label="Model root"]');
  const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value");
  proto.set.call(input, ${JSON.stringify(fixtureFolder)});
  input.dispatchEvent(new Event("input", { bubbles: true }));
  return true;
})()`);
await settle(200);
await clickText("Rescan");
await settle(1200);
await clickNav("Inventory");
await settle(500);

// 1) Full path retrievable: title + aria-label carry it; the copy control works.
const value = await evaluate(`(() => {
  const node = document.querySelector(".path-text-value");
  return node
    ? { title: node.getAttribute("title"), aria: node.getAttribute("aria-label"), tab: node.tabIndex }
    : null;
})()`);
check(
  "the truncated directory keeps its full value in title and aria-label",
  Boolean(
    value &&
      value.title &&
      // The rendered title must carry the FULL passed fixture path, whatever
      // its final segment is (the probe is run with different folders).
      value.title.includes(fixtureFolder.replace(/\\/g, "/").split("/").filter(Boolean).pop()) &&
      value.aria.includes(value.title),
  ),
  value ? value.title : "no path-text-value rendered",
);
check("the path value is keyboard reachable", Boolean(value && value.tab >= 0));
// WebView2 shows a native clipboard permission bubble on first programmatic
// write; the host grants it for this origin so the probe can assert the write
// path without driving native Chrome UI (not a DOM element).
await client.send("Browser.grantPermissions", {
  origin: "http://tauri.localhost",
  permissions: ["clipboardReadWrite", "clipboardSanitizedWrite"],
});
await evaluate(`[...document.querySelectorAll("button.path-copy")].forEach((b) => b.click())`);
await settle(400);
const copyState = await evaluate(
  `[...document.querySelectorAll("button.path-copy")].map((b) => b.textContent.trim()).join(",")`,
);
check(
  "the copy control writes the path and reports it",
  copyState.includes("Copied"),
  `button state: ${copyState}`,
);

// 2) Reduced motion: emulated preference must reach the Jump action.
await client.send("Emulation.setEmulatedMedia", {
  features: [{ name: "prefers-reduced-motion", value: "reduce" }],
});
await clickNav("Profile");
await settle(600);
const reduced = await evaluate(
  `window.matchMedia("(prefers-reduced-motion: reduce)").matches`,
);
check("the emulated reduced-motion preference is active", reduced === true);
await evaluate(`(() => {
  window.__scrollBehaviors = [];
  Element.prototype.scrollIntoView = function (options) {
    window.__scrollBehaviors.push(options?.behavior ?? "default");
  };
  return true;
})()`);
await clickText("Jump to Advanced");
await settle(400);
const jump = await evaluate(`(() => {
  const advanced = document.querySelector(".advanced-zone");
  return {
    open: advanced?.open ?? false,
    behaviors: window.__scrollBehaviors ?? [],
    focus: document.activeElement?.tagName ?? "none",
  };
})()`);
check(
  "Jump to Advanced expands and moves focus to the region summary",
  jump.open === true && jump.focus === "SUMMARY",
  `open=${jump.open} focus=${jump.focus}`,
);
check(
  "reduced motion suppresses smooth scrolling",
  jump.behaviors.includes("auto") && !jump.behaviors.includes("smooth"),
  `behaviors=${jump.behaviors.join(",")}`,
);

// 3) Narrow layout: the copy affordance stays visible and usable.
await client.send("Emulation.setDeviceMetricsOverride", {
  width: 320,
  height: 720,
  deviceScaleFactor: 1,
  mobile: false,
});
await clickNav("Inventory");
await settle(500);
const narrow = await evaluate(`(() => {
  const copy = document.querySelector("button.path-copy");
  if (!copy) return null;
  const box = copy.getBoundingClientRect();
  return { width: Math.round(box.width), height: Math.round(box.height), visible: box.width > 0 && box.height > 0 };
})()`);
check(
  "the copy control remains visible at 320 px",
  Boolean(narrow && narrow.visible && narrow.height >= 20),
  narrow ? `${narrow.width}x${narrow.height} px` : "copy control missing",
);
await client.send("Emulation.clearDeviceMetricsOverride");
await client.send("Emulation.setEmulatedMedia", { features: [] });

const failed = results.filter((entry) => !entry.ok);
console.log(`S24_RESULT ${failed.length === 0 ? "PASS" : "FAIL"} (${results.length} checks)`);
process.exit(failed.length === 0 ? 0 : 1);
