// G-05 TLS retry: cancel any stuck start, re-set the TLS fields, start again,
// and report the live state. Usage: node scripts/g05_tls_retry.mjs <debugPort> <certDir>
import { attach } from "./lib/cdp_client.mjs";

const [portArg, certDir] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => /Cancel|Stop/.test((candidate.textContent ?? "").trim()) && !candidate.disabled,
  );
  if (button) button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
await settle(2500);

const clickNav = (label) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === ${JSON.stringify(label)},
    );
    if (button) button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);
await clickNav("Profile");
await settle(1500);

const reSet = await evaluate(`(() => {
  const labels = [...document.querySelectorAll("label")];
  const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
  const values = {
    "API key file": ${JSON.stringify(certDir + "/api-key.txt")},
    "SSL private key": ${JSON.stringify(certDir + "/key.pem")},
    "SSL certificate": ${JSON.stringify(certDir + "/cert.pem")},
  };
  const out = {};
  for (const [labelText, value] of Object.entries(values)) {
    const label = labels.find((entry) => (entry.textContent ?? "").trim().startsWith(labelText));
    const input = label ? label.querySelector("input") : null;
    if (input) {
      setter.call(input, value);
      input.dispatchEvent(new Event("input", { bubbles: true }));
      out[labelText] = "set";
    } else {
      out[labelText] = "missing";
    }
  }
  return out;
})()`);
console.log("FIELDS:", JSON.stringify(reSet));
await settle(600);

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
for (let attempt = 0; attempt < 40; attempt += 1) {
  await settle(1500);
  const state = await evaluate(`(() => {
    const text = document.body.textContent || "";
    return {
      live: /Stop server/.test(text),
      starting: /Cancel start/.test(text),
      failure: (() => {
        const m = text.match(/[^.\\n]*(certificate|TLS|no SAN|invalid peer|failed|exited)[^.\\n]*/i);
        return m ? m[0].replace(/\\s+/g, " ").slice(0, 220) : null;
      })(),
    };
  })()`);
  if (state.live) {
    live = true;
    console.log("LIVE at", attempt);
    break;
  }
  if (!state.starting && state.failure) {
    console.log("FAILURE:", state.failure);
    break;
  }
}
const notice = await evaluate(`(() => {
  const m = (document.body.textContent || "").match(/[^.\\n]*(Started|listening|Stop server|health)[^.\\n]*/i);
  return m ? m[0].replace(/\\s+/g, " ").slice(0, 220) : null;
})()`);
console.log("NOTICE:", notice);
console.log(`G05_TLS_RETRY ${live ? "LIVE" : "NOT_LIVE"}`);
await client.close?.();
process.exit(live ? 0 : 1);
