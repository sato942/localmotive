// Fast-poll cancel capture + stop/start with the live label.
import { attach } from "./lib/cdp_client.mjs";
import { execSync } from "node:child_process";
const client = await attach(Number(process.argv[2] ?? 10070));
const ev = (e) => client.evaluate(e);
const settle = (ms) => new Promise((r) => setTimeout(r, ms));
const clickText = (t) => ev(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === ${JSON.stringify(t)} && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
const nav = (l) => clickText(l);
const bodyText = () => ev(`(document.body.innerText || "").replace(/\\s+/g, " ")`);
const ll = () => { try { return execSync('tasklist /FI "IMAGENAME eq llama-server.exe" /FO CSV /NH', { encoding: "utf8" }).split(/\r?\n/).filter((l) => l.toLowerCase().includes("llama-server")).length; } catch { return 0; } };

// 1) Profile buttons with the server live.
await nav("Profile");
await settle(900);
console.log("P_BUTTONS", JSON.stringify(await ev(`(() => [...document.querySelectorAll("button")].map(b => ({ t: (b.textContent||"").trim().slice(0,22), dis: b.disabled })).filter(b => /stop|start|cancel|save|inspect/i.test(b.t)))()`)));

// 2) Benchmark: start a run and fast-poll for an enabled Cancel.
await nav("Benchmark");
await settle(900);
console.log("R_RUN", await clickText("Run v2 benchmark"));
let captured = "never-enabled";
const t0 = Date.now();
for (let i = 0; i < 120; i += 1) {
  await settle(200);
  const state = await ev(`(() => [...document.querySelectorAll("button")].filter(b => (b.textContent || "").trim() === "Cancel").map(b => !b.disabled))()`);
  if (state.some((enabled) => enabled)) {
    const clicked = await ev('(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === "Cancel" && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()');
    captured = `enabled@${Date.now() - t0}ms clicked=${clicked}`;
    break;
  }
  const t = await bodyText();
  if (/Decode throughput/.test(t) && !/Benchmark running/.test(t) && Date.now() - t0 > 2000) { captured = `run-finished@${Date.now() - t0}ms (cancel window missed)`; break; }
}
console.log("R_CANCEL", captured);
await settle(3000);
const t1 = await bodyText();
console.log("R_STATE", JSON.stringify((t1.match(/Benchmark running|Cancelling|No v2 result|Decode throughput[^|]{0,50}/g) ?? []).slice(-2)));
for (let i = 0; i < 60; i += 1) { await settle(1000); const t = await bodyText(); if (!/Benchmark running|Cancelling/.test(t)) break; }
console.log("R_FINAL | LLAMA", ll());

// 3) Stop/start with the real label from step 1.
await nav("Profile");
await settle(800);
console.log("P_STOP_CLICK", await clickText("Stop server"), await clickText("Stop"));
await settle(3000);
console.log("P_AFTER_STOP LLAMA", ll());
console.log("P_START_CLICK", await clickText("Start"));
let live = false;
for (let i = 0; i < 60 && !live; i += 1) { await settle(1000); const t = await bodyText(); live = /LIVE/.test(t) && !/Starting/.test(t); }
console.log("P_LIVE_AGAIN", live, "| LLAMA", ll());
console.log("F_DONE");
await client.close();
process.exit(0);
