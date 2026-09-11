// G-05 high-contrast probe: emulate Windows High Contrast Mode signals
// (forced-colors: active, prefers-contrast: more) in the packaged WebView2
// engine and assert that text stays visible, controls keep size, and no
// horizontal overflow appears. Screenshots are saved for the evidence record.
// Usage: node scripts/verify_high_contrast.mjs <debugPort> <outDir>
import { attach } from "./lib/cdp_client.mjs";
import { writeFile } from "node:fs/promises";

const [portArg, outDir] = process.argv.slice(2);
const port = Number(portArg);
const client = await attach(port);
const { send, evaluate } = client;
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const states = [
  { name: "forced-colors-active", features: [{ name: "forced-colors", value: "active" }] },
  { name: "prefers-contrast-more", features: [{ name: "prefers-contrast", value: "more" }] },
];

const results = [];
for (const state of states) {
  await send("Emulation.setEmulatedMedia", { features: state.features });
  await settle(600);
  const probe = await evaluate(`(() => {
    const visible = (node) => {
      const rect = node.getBoundingClientRect();
      return rect.width > 0 && rect.height > 0;
    };
    const buttons = [...document.querySelectorAll("button")];
    const visibleButtons = buttons.filter(visible);
    const texts = [...document.querySelectorAll("p, span, h1, h2, label, td, th")].filter(
      (node) => (node.textContent ?? "").trim().length > 2 && visible(node),
    );
    const transparentText = texts.filter((node) => {
      const color = getComputedStyle(node).color.replace(/\\s+/g, "");
      return color === "rgba(0,0,0,0)" || color === "transparent";
    }).length;
    const zeroSizeButtons = visibleButtons.filter((node) => {
      const rect = node.getBoundingClientRect();
      return rect.width < 8 || rect.height < 8;
    }).length;
    const interactive = [...document.querySelectorAll("input, select")]
      .filter((node) => node.offsetParent !== null)
      .slice(0, 6)
      .map((node) => {
        const style = getComputedStyle(node);
        return { tag: node.tagName, color: style.color, background: style.backgroundColor };
      });
    return {
      textNodes: texts.length,
      transparentText,
      buttons: buttons.length,
      zeroSizeButtons,
      overflow: document.documentElement.scrollWidth - window.innerWidth,
      bodyHeight: document.body.clientHeight,
      interactive,
    };
  })()`);
  const shot = await send("Page.captureScreenshot", { format: "png" });
  const shotPath = `${outDir}/high-contrast-${state.name}.png`;
  await writeFile(shotPath, Buffer.from(shot.data, "base64"));
  const ok =
    probe.bodyHeight > 100 &&
    probe.transparentText === 0 &&
    probe.zeroSizeButtons === 0 &&
    probe.overflow <= 4;
  results.push({ state: state.name, ok, probe, shotPath });
  console.log(
    `STATE ${state.name}: ${ok ? "PASS" : "FAIL"} texts=${probe.textNodes} transparent=${probe.transparentText} buttons=${probe.buttons} zeroSize=${probe.zeroSizeButtons} overflow=${probe.overflow}px`,
  );
  console.log(`  screenshot: ${shotPath}`);
}

await send("Emulation.setEmulatedMedia", { features: [] });
const allOk = results.every((entry) => entry.ok);
console.log(`HIGH_CONTRAST_RESULT ${allOk ? "PASS" : "FAIL"}`);
console.log("NOTE: engine-level media emulation approximates Windows High Contrast Mode; an OS-level HCM (Win+U) session remains a separate manual check.");
await client.close?.();
process.exit(allOk ? 0 : 1);
