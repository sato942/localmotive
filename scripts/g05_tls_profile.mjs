// G-05 TLS/auth local profile driver: fill the profile's API-key /
// SSL-certificate / SSL-key fields through the editor, save, start, then
// report the preview argv and the running process command line.
// Usage: node scripts/g05_tls_profile.mjs <debugPort> <certDir>
import { attach } from "./lib/cdp_client.mjs";

const [portArg, certDir] = process.argv.slice(2);
if (!portArg || !certDir) {
  console.error("usage: g05_tls_profile.mjs <debugPort> <certDir>");
  process.exit(2);
}
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

const setFieldByLabel = (labelText, value) =>
  evaluate(`(() => {
    const labels = [...document.querySelectorAll("label")];
    const label = labels.find((entry) => (entry.textContent ?? "").trim().startsWith(${JSON.stringify(labelText)}));
    if (!label) return "label-missing";
    const input = label.querySelector("input");
    if (!input) return "input-missing";
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
    setter.call(input, ${JSON.stringify(value)});
    input.dispatchEvent(new Event("input", { bubbles: true }));
    return input.value;
  })()`);

// 1. Select a model if the profile is empty.
await clickNav("Profile");
await settle(1200);
let text = await evaluate("document.body.textContent || \"\"");
if (text.includes("No models to profile yet") || text.includes("No model selected")) {
  await clickNav("Inventory");
  await settle(1200);
  for (let attempt = 0; attempt < 20; attempt += 1) {
    const picked = await evaluate(`(() => {
      const rows = [...document.querySelectorAll("tbody tr")];
      const row = rows.find((entry) => (entry.textContent ?? "").includes("SmolLM2")) ?? rows[0];
      if (!row) return false;
      row.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      return true;
    })()`);
    if (picked) break;
    await settle(1500);
  }
  await clickNav("Profile");
  await settle(1500);
}

// 2. Fill the TLS/auth fields.
const cert = `${certDir}/cert.pem`;
const key = `${certDir}/key.pem`;
const apiKey = `${certDir}/api-key.txt`;
console.log("SET apiKeyFile:", await setFieldByLabel("API key file", apiKey));
console.log("SET sslCert:", await setFieldByLabel("SSL certificate", cert));
console.log("SET sslKey:", await setFieldByLabel("SSL private key", key));
await settle(600);

// 3. Save so the draft persists (Save is the secondary action next to Start).
await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "Save",
  );
  if (button) button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
await settle(900);

// 4. Read the provisional command preview (argv, capability-filtered).
const preview = await evaluate(`(() => {
  const node = document.querySelector(".command-preview, aside.command-preview, pre");
  return node ? node.textContent.replace(/\\s+/g, " ").slice(0, 600) : null;
})()`);
console.log("PREVIEW:", preview);

// 5. Start and wait for live.
const started = await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "Start" && !candidate.disabled,
  );
  if (!button) return false;
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
console.log("START CLICKED:", started);
let live = false;
for (let attempt = 0; attempt < 60; attempt += 1) {
  await settle(1500);
  text = await evaluate("document.body.textContent || \"\"");
  if (/Stop server/.test(text)) {
    live = true;
    break;
  }
  if (/failed|refused|error band|Exit/i.test(text) && attempt > 6) break;
}
console.log("SERVER LIVE:", live);
const notice = await evaluate(`(() => {
  const match = (document.body.textContent || "").match(/[^.\\n]*(TLS|SSL|api key|api-key|refused|failed|listening|started)[^.\\n]*/i);
  return match ? match[0].replace(/\\s+/g, " ").slice(0, 240) : null;
})()`);
console.log("NOTICE:", notice);
await client.close?.();
process.exit(live ? 0 : 1);
