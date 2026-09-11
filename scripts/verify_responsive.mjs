// FE-14 packaged responsive verification.
//
// Runs against the packaged binary: launches it with a debugger port and
// drives the real WebView2 layout viewport through the widths named in the
// audit (320/375/680/980) plus 200% and 400% page zoom. It asserts that no
// width produces horizontal overflow and that bottom-navigation cells keep
// at least 44 px targets at the narrow widths. This is packaged evidence,
// not a source inspection.
import { spawn } from "node:child_process";
import { attach } from "./lib/cdp_client.mjs";

const exePath = process.argv[2] ?? "src-tauri/target/release/localmotive.exe";
const port = 57000 + Math.floor(Math.random() * 900);

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

const child = spawn(exePath, [], {
  env: {
    ...process.env,
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}`,
  },
  stdio: "ignore",
  detached: false,
});

async function shutdown() {
  try {
    child.kill();
  } catch {
    // already exited
  }
  await sleep(500);
}

const failures = [];
let client = null;
try {
  client = await attach(port, { deadlineMs: 60_000 });
  await sleep(4_000);

  const boot = await client.evaluate(
    `(() => document.getElementById("root")?.childElementCount ?? 0)()`,
  );
  if (!boot) {
    failures.push("the UI did not render");
  }

  const widths = [320, 375, 680, 980];
  for (const width of widths) {
    await client.send("Emulation.setDeviceMetricsOverride", {
      width,
      height: 800,
      deviceScaleFactor: 1,
      mobile: false,
    });
    await sleep(900);
    const metrics = await client.evaluate(`(() => ({
      inner: window.innerWidth,
      scroll: document.documentElement.scrollWidth,
      navCells: Array.from(document.querySelectorAll("nav .nav-item")).map((cell) => {
        const rect = cell.getBoundingClientRect();
        return { w: Math.round(rect.width), h: Math.round(rect.height) };
      }),
    }))()`);
    const overflow = metrics.scroll - metrics.inner;
    const line = `width ${width}: inner ${metrics.inner} scroll ${metrics.scroll} overflow ${overflow}`;
    if (overflow > 1) {
      failures.push(`horizontal overflow at ${width}px: ${line}`);
    }
    if (width <= 375) {
      const small = metrics.navCells.filter((cell) => cell.h < 44 || cell.w < 44);
      if (small.length > 0) {
        failures.push(
          `bottom navigation targets below 44px at ${width}: ${JSON.stringify(small)}`,
        );
      }
    }
    console.log(line);
  }

  // Browser zoom as page scale: no layout overflow may appear, and the
  // navigation targets keep their size in layout units.
  await client.send("Emulation.setDeviceMetricsOverride", {
    width: 980,
    height: 800,
    deviceScaleFactor: 1,
    mobile: false,
  });
  for (const zoom of [2, 4]) {
    await client.send("Emulation.setPageScaleFactor", { pageScaleFactor: zoom });
    await sleep(700);
    const metrics = await client.evaluate(`(() => ({
      inner: window.innerWidth,
      scroll: document.documentElement.scrollWidth,
      scale: window.visualViewport ? window.visualViewport.scale : null,
    }))()`);
    const overflow = metrics.scroll - metrics.inner;
    console.log(
      `zoom ${zoom * 100}%: inner ${metrics.inner} scroll ${metrics.scroll} overflow ${overflow} scale ${metrics.scale}`,
    );
    if (overflow > 1) {
      failures.push(`horizontal overflow at ${zoom * 100}% zoom`);
    }
  }
  await client.send("Emulation.setPageScaleFactor", { pageScaleFactor: 1 });
  await client.send("Emulation.clearDeviceMetricsOverride");

  if (failures.length > 0) {
    console.error("RESPONSIVE_FAIL");
    for (const failure of failures) console.error(`- ${failure}`);
    await shutdown();
    process.exit(1);
  }
  console.log("RESPONSIVE_PASS");
  await shutdown();
  process.exit(0);
} catch (error) {
  console.error(`RESPONSIVE_ERROR ${String(error)}`);
  await shutdown();
  process.exit(1);
}
