// G-05 packaged accessibility probe (keyboard traversal + reduced motion).
//
// Runs against the packaged binary over CDP: Tab traversal must reach real
// controls with a visible focus indicator, and the app must honor
// prefers-reduced-motion. Screen-reader passes are out of scope here and are
// recorded separately as NOT RUN.
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
