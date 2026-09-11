// Clean single-instance stop/start cycle + calibration history walk.
import { attach } from "./lib/cdp_client.mjs";
import { execSync } from "node:child_process";
const client = await attach(Number(process.argv[2] ?? 10070));
const ev = (e) => client.evaluate(e);
const settle = (ms) => new Promise((r) => setTimeout(r, ms));
const clickText = (t) => ev(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === ${JSON.stringify(t)} && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
const nav = (l) => clickText(l);
const bodyText = () => ev(`(document.body.innerText || "").replace(/\\s+/g, " ")`);
const waitLive = async (want, tries = 90) => { for (let i = 0; i < tries; i += 1) { await settle(1000); const t = await bodyText(); const live = /LIVE/.test(t) && !/Starting/.test(t); if (live === want) return true; } return false; };
const ll = () => { try { return execSync('tasklist /FI "IMAGENAME eq llama-server.exe" /FO CSV /NH', { encoding: "utf8" }).split(/\r?\n/).filter((l) => l.toLowerCase().includes("llama-server")).length; } catch { return 0; } };

await nav("Profile");
await settle(900);
console.log("K1_BUTTONS", JSON.stringify(await ev(`(() => [...document.querySelectorAll("button")].map(b => ({ t: (b.textContent||"").trim().slice(0,20), dis: b.disabled })).filter(b => /start|stop/i.test(b.t)))()`)));
console.log("K1_START", await clickText("Start"));
console.log("K1_LIVE", await waitLive(true), "| LLAMA", ll());
console.log("K2_BUTTONS_LIVE", JSON.stringify(await ev(`(() => [...document.querySelectorAll("button")].map(b => ({ t: (b.textContent||"").trim().slice(0,20), dis: b.disabled })).filter(b => /start|stop/i.test(b.t)))()`)));
console.log("K2_STOP", await clickText("Stop server"));
const stopped = await waitLive(false, 30);
console.log("K2_STOPPED", stopped, "| LLAMA", ll());
console.log("K3_START_AGAIN", await clickText("Start"));
const live2 = await waitLive(true, 90);
console.log("K3_LIVE", live2, "| LLAMA", ll());

// Calibration/history walk.
await nav("Benchmark");
await settle(1200);
await ev(`(() => { const el = [...document.querySelectorAll("*")].find(x => (x.textContent || "").trim() === "Clear local history"); if (el) el.scrollIntoView(); return !!el; })()`);
await settle(600);
const cal = await ev(`(() => { const btn = [...document.querySelectorAll("button")].find(b => (b.textContent || "").trim() === "Clear local history"); if (!btn) return "no-button"; let root = btn; for (let k = 0; k < 5 && root.parentElement; k += 1) root = root.parentElement; return (root.textContent || "").replace(/\\s+/g, " ").slice(0, 1100); })()`);
console.log("K4_CALIBRATION", JSON.stringify(cal).slice(0, 1200));
console.log("K5_DONE");
await client.close();
process.exit(0);
