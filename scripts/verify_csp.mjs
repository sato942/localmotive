// IPC-02 packaged Content Security Policy verification.
//
// Runs against the packaged binary: launches it with a debugger port,
// navigates the real UI, and then proves that (a) ordinary operation raises
// no CSP violations, (b) an injected inline script and a remote fetch are
// blocked by the policy, and (c) hostile text stays inert as text. This is
// packaged evidence, not a source inspection.
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

let client = null;
try {
  client = await attach(port, { deadlineMs: 60_000 });
  await sleep(4_000);

  const failures = [];

  // Phase 1: the real UI renders under the production policy.
  const boot = await client.evaluate(`(() => ({
    rootChildren: document.getElementById("root")?.childElementCount ?? 0,
    text: document.body.innerText.length,
  }))()`);
  if (!boot || boot.rootChildren === 0 || boot.text < 50) {
    failures.push(`UI did not render under CSP: ${JSON.stringify(boot)}`);
  }

  const navs = await client.evaluate(`(() => {
    const buttons = Array.from(document.querySelectorAll("nav button, .nav button, aside button"));
    return buttons.map((button) => button.textContent.trim()).filter((text) => text.length > 0);
  })()`);
  for (const label of navs ?? []) {
    await client.evaluate(`(() => {
      const buttons = Array.from(document.querySelectorAll("nav button, .nav button, aside button"));
      const button = buttons.find((item) => item.textContent.trim() === ${JSON.stringify(label)});
      if (button) button.click();
      return true;
    })()`);
    await sleep(600);
  }

  // The scoped opener must still open an allowed origin: exercise the About
  // screen's repository link (github.com is inside the granted scope).
  await client.evaluate(`(() => {
    const buttons = Array.from(document.querySelectorAll("nav button, .nav button, aside button"));
    const about = buttons.find((item) => /about/i.test(item.textContent.trim()));
    if (about) about.click();
    return true;
  })()`);
  await sleep(800);
  const openerResult = await client.evaluate(`(() => {
    const button = Array.from(document.querySelectorAll("button")).find((item) =>
      item.textContent.includes("Project on GitHub"),
    );
    if (!button) return "missing";
    button.click();
    return "clicked";
  })()`);
  await sleep(1_200);
  if (openerResult !== "clicked") {
    failures.push(`the repository opener action was ${openerResult}`);
  }

  const navigationViolations = client.exceptions.filter((entry) =>
    entry.text.includes("Content Security Policy"),
  );
  if (navigationViolations.length > 0) {
    failures.push(
      `the application itself raised CSP violations during navigation: ${JSON.stringify(navigationViolations.map((entry) => entry.text))}`,
    );
  }
  const navigationExceptions = client.exceptions.filter(
    (entry) => !entry.text.includes("Content Security Policy"),
  );
  if (navigationExceptions.length > 0) {
    failures.push(`unexpected console errors during navigation: ${JSON.stringify(navigationExceptions.map((entry) => entry.text))}`);
  }

  // Phase 2a: an injected inline script must not execute.
  const before = client.exceptions.length;
  const inlineResult = await client.evaluate(`(() => {
    window.__cspPwned = 0;
    const script = document.createElement("script");
    script.textContent = "window.__cspPwned = 1";
    document.body.appendChild(script);
    script.remove();
    return window.__cspPwned;
  })()`);
  if (inlineResult !== 0) {
    failures.push(`an injected inline script executed (__cspPwned=${inlineResult})`);
  }
  await sleep(800);
  const blockedMessages = client.exceptions
    .slice(before)
    .filter((entry) => entry.text.includes("Content Security Policy"));
  if (blockedMessages.length === 0) {
    failures.push("no CSP violation was reported for the injected inline script");
  }

  // Phase 2b: a remote fetch must be blocked by connect-src.
  const fetchResult = await client.evaluate(`(async () => {
    try {
      await fetch("https://example.com/csp-probe", { mode: "no-cors" });
      return "allowed";
    } catch (error) {
      return "blocked";
    }
  })()`);
  if (fetchResult !== "blocked") {
    failures.push(`a remote fetch was not blocked by CSP (${fetchResult})`);
  }

  // Phase 2c: hostile text stays inert (React/DOM text nodes, never HTML).
  const hostile = await client.evaluate(`(() => {
    window.__cspImg = 0;
    const container = document.createElement("div");
    container.id = "csp-hostile-fixture";
    container.textContent = '<img src=x onerror="window.__cspImg=1"><script>window.__cspImg=2<\\/script>';
    document.body.appendChild(container);
    const imgs = container.querySelectorAll("img").length;
    const scripts = container.querySelectorAll("script").length;
    container.remove();
    return { imgs, scripts, fired: window.__cspImg };
  })()`);
  if (!hostile || hostile.imgs !== 0 || hostile.scripts !== 0 || hostile.fired !== 0) {
    failures.push(`hostile text was not inert: ${JSON.stringify(hostile)}`);
  }

  if (failures.length > 0) {
    console.log(JSON.stringify({ pass: false, failures }, null, 2));
    await shutdown();
    process.exit(1);
  }
  console.log(
    JSON.stringify(
      {
        pass: true,
        port,
        navigationTargets: (navs ?? []).length,
        openerAction: "clicked",
        cspViolationsDuringNavigation: navigationViolations.length,
        inlineScriptBlocked: true,
        remoteFetchBlocked: true,
        hostileTextInert: true,
      },
      null,
      2,
    ),
  );
  client.close();
  await shutdown();
  process.exit(0);
} catch (error) {
  console.log(JSON.stringify({ pass: false, error: String(error) }, null, 2));
  try {
    client?.close();
  } catch {
    // ignore
  }
  await shutdown();
  process.exit(1);
}
