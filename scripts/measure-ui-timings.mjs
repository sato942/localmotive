// Stage 2 UI timings (backlog UI-speed): launch-to-window, time-to-
// interactive, and click-to-painted for all 8 views on the PACKAGED binary.
// Owns its process, isolated profile/data, and cleanup. Writes JSON.
//
// Methodology: CDP evaluate round-trips add a few ms of overhead per poll
// (15 ms interval), so short timings skew high — conservative for caps.
// Medians of 3 runs per view. Anything needing a model (profile save,
// benchmark start) is attempted only when its control is enabled and
// otherwise recorded as skipped with its reason.
import { spawn, execSync } from "node:child_process";
import { mkdtempSync, mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { attach } from "./lib/cdp_client.mjs";

const [cdpPort, portable, outJson] = [Number(process.argv[2]), process.argv[3], process.argv[4]];
if (!cdpPort || !portable || !outJson) {
  console.error("usage: node scripts/measure-ui-timings.mjs <CdpPort> <PortableExe> <OutJson>");
  process.exit(2);
}

const root = mkdtempSync(join(tmpdir(), "ui-timings-"));
const profile = join(root, "wv2-profile");
mkdirSync(profile, { recursive: true });
const data = join(root, "appdata");
mkdirSync(data, { recursive: true });

const tSpawn = Date.now();
const child = spawn(portable, [], {
  env: {
    ...process.env,
    LOCALAPPDATA: data,
    WEBVIEW2_USER_DATA_FOLDER: profile,
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${cdpPort} --user-data-dir=${profile}`,
  },
  stdio: "ignore",
  detached: true,
});
child.unref();

const report = { launchToWindowMs: null, timeToInteractiveMs: null, views: {}, skipped: {} };
let client = null;
try {
  // Launch-to-window: first page target with a debugger URL.
  const windowDeadline = Date.now() + 120_000;
  for (;;) {
    try {
      const list = await (await fetch(`http://127.0.0.1:${cdpPort}/json/list`)).json();
      if (list.some((t) => t.type === "page" && t.webSocketDebuggerUrl)) break;
    } catch {}
    if (Date.now() > windowDeadline) throw new Error("no page target appeared");
    await new Promise((r) => setTimeout(r, 250));
  }
  report.launchToWindowMs = Date.now() - tSpawn;

  client = await attach(cdpPort, { deadlineMs: 60_000 });
  const pollTrue = async (expression, timeoutMs = 30_000) => {
    const deadline = Date.now() + timeoutMs;
    for (;;) {
      if (await client.evaluate(`Boolean(${expression})`, 10_000)) return;
      if (Date.now() > deadline) throw new Error(`timed out waiting for ${expression}`);
      await new Promise((r) => setTimeout(r, 15));
    }
  };
  await pollTrue("document.querySelector('nav button')?.getBoundingClientRect().width > 0");
  report.timeToInteractiveMs = Date.now() - tSpawn;

  const views = [
    ["Control", ".dashboard-screen"],
    ["Inventory", ".inventory-screen"],
    ["HF Catalog", ".catalog-screen"],
    ["Runtime", ".runtime-screen"],
    ["Profile", ".profile-screen"],
    ["AI Tune", ".tune-screen"],
    ["Benchmark", ".benchmark-screen"],
    ["About", ".about-screen"],
  ];
  for (const [label, marker] of views) {
    const samples = [];
    let skipped = null;
    for (let i = 0; i < 3; i += 1) {
      try {
        // Park on a different view first so every sample is a real switch,
        // then time click-to-PAINTED inside the page: the click dispatches,
        // React re-renders, and two animation frames mean the pixels are out.
        const measured = await client.evaluate(`(async () => {
          const find = (name) => [...document.querySelectorAll('nav button')]
            .find((item) => item.textContent.trim() === name);
          const other = ${JSON.stringify(label)} === "About" ? "Control" : "About";
          const park = find(other);
          if (!park) return { error: "park button missing" };
          park.click();
          const markerOf = (name) => ({
            Control: ".dashboard-screen", Inventory: ".inventory-screen",
            "HF Catalog": ".catalog-screen", Runtime: ".runtime-screen",
            Profile: ".profile-screen", "AI Tune": ".tune-screen",
            Benchmark: ".benchmark-screen", About: ".about-screen",
          })[name];
          const visible = (sel) => {
            const el = document.querySelector(sel);
            return !!el && el.getBoundingClientRect().width > 0;
          };
          const deadline = performance.now() + 10000;
          while (!visible(markerOf(other)) && performance.now() < deadline) {
            await new Promise((r) => setTimeout(r, 10));
          }
          if (!visible(markerOf(other))) return { error: "park view never visible" };
          const target = find(${JSON.stringify(label)});
          if (!target) return { error: "nav button missing" };
          const tClick = performance.now();
          target.click();
          while (!visible(${JSON.stringify(marker)}) && performance.now() - tClick < 10000) {
            await new Promise((r) => setTimeout(r, 5));
          }
          if (!visible(${JSON.stringify(marker)})) return { error: "target never visible" };
          await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
          return { ms: performance.now() - tClick };
        })()`, 30_000);
        if (measured?.error) {
          skipped = measured.error;
          break;
        }
        samples.push(Math.round(measured.ms * 10) / 10);
      } catch (error) {
        skipped = String(error?.message ?? error);
        break;
      }
    }
    if (skipped && samples.length === 0) report.skipped[label] = skipped;
    else {
      samples.sort((a, b) => a - b);
      report.views[label] = { samples, medianMs: samples[Math.floor(samples.length / 2)] };
      if (skipped) report.views[label].note = skipped;
    }
  }

  // Profile save→feedback: only when a real Profile screen with an enabled
  // Save exists (needs a selected model; isolated data has none).
  try {
    const saveState = await client.evaluate(`(() => {
      if (!document.querySelector('.profile-screen')) return "no-screen";
      const save = [...document.querySelectorAll('.profile-screen button')]
        .find((b) => b.textContent.trim() === 'Save');
      if (!save) return "no-button";
      if (save.disabled) return "disabled";
      save.click();
      return "clicked";
    })()`);
    if (saveState === "clicked") {
      const tSave = Date.now();
      await pollTrue(`document.querySelector('.profile-screen [role="status"]')?.textContent.length > 0`);
      report.views["Profile save→feedback"] = { samples: [Date.now() - tSave], medianMs: Date.now() - tSave };
    } else {
      report.skipped["Profile save→feedback"] = `save ${saveState} without a model fixture`;
    }
  } catch (error) {
    report.skipped["Profile save→feedback"] = String(error?.message ?? error);
  }
  report.skipped["Benchmark start"] = "needs a downloaded model + running server fixture; not wired in Stage 2";
} finally {
  try {
    execSync(`taskkill /pid ${child.pid} /T /F`, { stdio: "ignore" });
  } catch {}
  try {
    client?.socket?.close();
  } catch {}
}
writeFileSync(outJson, JSON.stringify(report, null, 2));
console.log(JSON.stringify(report, null, 2));
