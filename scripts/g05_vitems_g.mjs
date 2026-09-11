// Post-stop start retry + CALIBRATION section dump.
import { attach } from "./lib/cdp_client.mjs";
import { execSync } from "node:child_process";
const client = await attach(Number(process.argv[2] ?? 10070));
const ev = (e) => client.evaluate(e);
const settle = (ms) => new Promise((r) => setTimeout(r, ms));
const bodyText = () => ev(`(document.body.innerText || "").replace(/\\s+/g, " ")`);
const ll = () => { try { return execSync('tasklist /FI "IMAGENAME eq llama-server.exe" /FO CSV /NH', { encoding: "utf8" }).split(/\r?\n/).filter((l) => l.toLowerCase().includes("llama-server")).length; } catch { return 0; } };

console.log("L1_BUTTONS", JSON.stringify(await ev(`(() => [...document.querySelectorAll("button")].map(b => ({ t: (b.textContent||"").trim().slice(0,18), dis: b.disabled })).filter(b => /start|stop/i.test(b.t)))()`)));
const clicked = await ev('(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === "Start" && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()');
console.log("L1_START_CLICKED", clicked);
let live = false;
for (let i = 0; i < 90 && !live; i += 1) { await settle(1000); const t = await bodyText(); live = /LIVE/.test(t) && !/Starting/.test(t); }
console.log("L1_LIVE", live, "| LLAMA", ll());
if (!live) {
  console.log("L1_NOTICE", JSON.stringify((await ev(`(() => [...document.querySelectorAll(".notice-line,.notice")].map(x => (x.textContent||"").trim()).filter(Boolean).slice(-2))()`))));
}

// CALIBRATION section.
const cal = await ev(`(() => {
  const heads = [...document.querySelectorAll("h2,h3,h4,.panel-title")].filter(h => /CALIBRATION/i.test(h.textContent || ""));
  if (!heads.length) return "no-heading";
  const head = heads[0];
  let root = head; for (let k = 0; k < 4 && root.parentElement; k += 1) root = root.parentElement;
  return (root.textContent || "").replace(/\\s+/g, " ").slice(0, 1400);
})()`);
console.log("L2_CALIBRATION", JSON.stringify(cal).slice(0, 1500));
console.log("L2_BUTTONS", JSON.stringify(await ev(`(() => [...document.querySelectorAll("button")].filter(b => /anchor|calibrat|history|record/i.test(b.textContent || "")).map(b => ({ t: (b.textContent||"").trim(), dis: b.disabled })))()`)));
console.log("L_DONE");
await client.close();
process.exit(0);
