// G-05 packaged accessibility probe (keyboard traversal, reduced motion,
// high-DPI/zoom, forced-colors high contrast).
//
// Runs against the packaged binary over CDP: Tab traversal must reach real
// controls with a visible focus indicator, the app must honor
// prefers-reduced-motion, a 2x DPI / 150% zoom layout must stay overflow-free
// with operable controls, and forced-colors (high contrast) must keep every
// control reachable with words rather than colour alone. Screen-reader passes
// The automation-tree leg consumes the same accessibility tree a screen
// reader consumes (CDP Accessibility domain); a manual Narrator/NVDA listening
// pass and live-credential scenarios remain out of scope and are recorded
// separately as NOT RUN.
import { spawn } from "node:child_process";
import { attach } from "./lib/cdp_client.mjs";

const exePath = process.argv[2] ?? "src-tauri/target/release/localmotive.exe";
const port = 57000 + Math.floor(Math.random() * 900);

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const child = spawn(exePath, [], {
  env: { ...process.env, WEBRTC_DISABLE: "1", WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}` },
  stdio: "ignore",
});

async function shutdown() {
  try {
    child.kill();
  } catch {
    /* exited */
  }
  await sleep(500);
}

const failures = [];
let client = null;
try {
  client = await attach(port, { deadlineMs: 60_000 });
  await sleep(4_000);

  // 1) Keyboard traversal with a visible focus indicator.
  const visited = [];
  for (let step = 0; step < 25; step += 1) {
    await client.send("Input.dispatchKeyEvent", { type: "keyDown", key: "Tab", code: "Tab", windowsVirtualKeyCode: 9 });
    await client.send("Input.dispatchKeyEvent", { type: "keyUp", key: "Tab", code: "Tab", windowsVirtualKeyCode: 9 });
    await sleep(120);
    const entry = await client.evaluate(`(() => {
      const active = document.activeElement;
      if (!active || active === document.body) return null;
      const style = getComputedStyle(active);
      return {
        tag: active.tagName,
        label: (active.getAttribute("aria-label") || active.textContent || "").trim().slice(0, 40),
        outlineWidth: style.outlineWidth,
        outlineStyle: style.outlineStyle,
      };
    })()`);
    if (entry) visited.push(entry);
  }
  const distinct = new Map();
  for (const entry of visited) distinct.set(`${entry.tag}:${entry.label}`, entry);
  if (distinct.size < 6) {
    failures.push(`keyboard traversal reached only ${distinct.size} distinct controls`);
  }
  const outlined = [...distinct.values()].filter(
    (entry) => entry.outlineStyle !== "none" && parseFloat(entry.outlineWidth) > 0,
  );
  if (outlined.length === 0) {
    failures.push("no traversed control showed a visible focus outline");
  }
  console.log(
    `keyboard: ${distinct.size} distinct controls, ${outlined.length} with a visible outline`,
  );
  for (const entry of [...distinct.values()].slice(0, 8)) {
    console.log(`  ${entry.tag} "${entry.label}" outline ${entry.outlineWidth} ${entry.outlineStyle}`);
  }

  // 2) Reduced motion must be honored.
  await client.send("Emulation.setEmulatedMedia", {
    features: [{ name: "prefers-reduced-motion", value: "reduce" }],
  });
  await sleep(300);
  const reduced = await client.evaluate(`(() => ({
    matches: matchMedia("(prefers-reduced-motion: reduce)").matches,
    hasRule: [...document.styleSheets].some((sheet) => {
      try {
        return [...sheet.cssRules].some((rule) => rule.cssText && rule.cssText.includes("prefers-reduced-motion"));
      } catch {
        return false;
      }
    }),
  }))()`);
  if (!reduced.matches) failures.push("reduced-motion emulation did not apply");
  if (!reduced.hasRule) failures.push("no prefers-reduced-motion rule found in the stylesheets");
  console.log(`reduced-motion: emulated ${reduced.matches}, stylesheet rule ${reduced.hasRule}`);
  await client.send("Emulation.setEmulatedMedia", { features: [] });

  // 3) High-DPI / zoom: a 2x DPI 150% zoom layout must not overflow
  // horizontally and every rendered button must keep a usable rectangle.
  await client.send("Emulation.setDeviceMetricsOverride", {
    width: 1280,
    height: 800,
    deviceScaleFactor: 2,
    mobile: false,
  });
  await client.send("Emulation.setPageScaleFactor", { pageScaleFactor: 1.5 });
  await sleep(400);
  const zoomed = await client.evaluate(`(() => {
    const doc = document.documentElement;
    const buttons = [...document.querySelectorAll("button")].filter((b) => b.offsetParent !== null);
    const unusable = buttons.filter((b) => {
      const r = b.getBoundingClientRect();
      return r.width < 8 || r.height < 8;
    });
    return {
      dpr: window.devicePixelRatio,
      scrollWidth: doc.scrollWidth,
      innerWidth: window.innerWidth,
      visibleButtons: buttons.length,
      unusable: unusable.length,
    };
  })()`);
  if (zoomed.dpr < 2) failures.push(`deviceScaleFactor 2 did not apply (dpr=${zoomed.dpr})`);
  if (zoomed.scrollWidth > zoomed.innerWidth + 8) {
    failures.push(`zoomed layout overflows horizontally (${zoomed.scrollWidth} > ${zoomed.innerWidth})`);
  }
  if (zoomed.visibleButtons === 0 || zoomed.unusable > 0) {
    failures.push(`zoomed layout left ${zoomed.unusable} unusable button(s) of ${zoomed.visibleButtons}`);
  }
  console.log(
    `zoom: dpr ${zoomed.dpr}, scrollWidth ${zoomed.scrollWidth} <= innerWidth ${zoomed.innerWidth}, ` +
      `${zoomed.visibleButtons} visible buttons, ${zoomed.unusable} unusable`,
  );
  await client.send("Emulation.setPageScaleFactor", { pageScaleFactor: 1 });
  await client.send("Emulation.clearDeviceMetricsOverride");

  // 4) Forced colors (high contrast): emulation must apply and the design
  // rule "colour is never the only channel" must hold - every control keeps
  // words, and state tags are not colour-only.
  // Navigate to a screen that carries runtime state tags so the word-bearing
  // check is not vacuous.
  await client.evaluate(`(() => {
    const control = [...document.querySelectorAll("button")].find(
      (b) => (b.textContent || "").trim() === "Control",
    );
    if (control) control.click();
    return Boolean(control);
  })()`);
  await sleep(800);
  await client.send("Emulation.setEmulatedMedia", {
    features: [{ name: "forced-colors", value: "active" }],
  });
  await sleep(300);
  const forced = await client.evaluate(`(() => {
    const controls = [...document.querySelectorAll("button")];
    const wordless = controls.filter((b) => {
      const text = (b.textContent || "").trim();
      const label = (b.getAttribute("aria-label") || "").trim();
      return text.length === 0 && label.length === 0;
    });
    const tagish = [...document.querySelectorAll("[class*='state'], [class*='tag'], [class*='badge'], [class*='pill']")];
    const wordlessTags = tagish.filter((el) => (el.textContent || "").trim().length === 0);
    return {
      matches: matchMedia("(forced-colors: active)").matches,
      controls: controls.length,
      wordless: wordless.length,
      tags: tagish.length,
      wordlessTags: wordlessTags.length,
    };
  })()`);
  if (!forced.matches) failures.push("forced-colors emulation did not apply");
  if (forced.wordless > 0) {
    failures.push(`${forced.wordless} control(s) carry no words under forced colors`);
  }
  if (forced.wordlessTags > 0) {
    failures.push(`${forced.wordlessTags} state tag(s) rely on colour alone under forced colors`);
  }
  console.log(
    `forced-colors: emulated ${forced.matches}, ${forced.controls} controls (${forced.wordless} wordless), ` +
      `${forced.tags} tags (${forced.wordlessTags} wordless)`,
  );
  await client.send("Emulation.setEmulatedMedia", { features: [] });

  // 5) Screen-reader semantics via the accessibility tree: every interactive
  // node a screen reader would announce must carry a non-empty accessible
  // name, so keyboard focus never announces a bare control.
  await client.send("Accessibility.enable");
  const tree = await client.send("Accessibility.getFullAXTree");
  const interactiveRoles = new Set(["button", "link", "textbox", "combobox", "checkbox", "radio", "tab", "menuitem", "slider"]);
  const nodes = (tree?.nodes ?? []).filter((node) => !node.ignored);
  const interactive = nodes.filter((node) => interactiveRoles.has(node?.role?.value));
  const unnamed = interactive.filter((node) => {
    const name = node?.name?.value;
    return typeof name !== "string" || name.trim().length === 0;
  });
  if (interactive.length < 6) {
    failures.push(`accessibility tree exposed only ${interactive.length} interactive node(s)`);
  }
  if (unnamed.length > 0) {
    failures.push(`${unnamed.length} interactive node(s) have no accessible name`);
    for (const node of unnamed.slice(0, 5)) {
      console.error(`  unnamed: role=${node?.role?.value} backendId=${node?.backendDOMNodeId ?? "?"}`);
    }
  }
  const named = interactive.filter((node) => (node?.name?.value || "").trim().length > 0);
  console.log(
    `accessibility tree: ${nodes.length} nodes, ${interactive.length} interactive, ${named.length} with names`,
  );
  await client.send("Accessibility.disable").catch(() => {});

  if (failures.length > 0) {
    console.error("A11Y_FAIL");
    for (const failure of failures) console.error(`- ${failure}`);
    await shutdown();
    process.exit(1);
  }
  console.log("A11Y_PASS");
  await shutdown();
  process.exit(0);
} catch (error) {
  console.error(`A11Y_ERROR ${String(error)}`);
  await shutdown();
  process.exit(1);
}
