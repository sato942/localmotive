// Clean-slate v2 run + cancel probe + MT-05 reservation release.
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
const cancels = () => ev(`(() => [...document.querySelectorAll("button")].filter(b => (b.textContent || "").trim() === "Cancel").map(b => ({ dis: b.disabled, cls: b.className })))()`);

await nav("Profile");
await settle(800);
console.log("S1_START", await clickText("Start"));
console.log("S1_LIVE", await waitLive(true), "| CHILD", execSync('powershell -NoProfile -Command "(Get-Process llama-server -ErrorAction SilentlyContinue | Select-Object -First 1).Path"', { encoding: "utf8" }).trim());
await nav("Benchmark");
await settle(1000);
console.log("S2_RUN", await clickText("Run v2 benchmark"));
await settle(4000);
const t = await bodyText();
console.log("S2_T4 running=", /Benchmark running/.test(t), "| cancels", JSON.stringify(await cancels()));
await settle(6000);
const t2 = await bodyText();
console.log("S2_T10 running=", /Benchmark running/.test(t2), "| cancels", JSON.stringify(await cancels()));
const clickedCancel = await ev('(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === "Cancel" && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()');
console.log("S2_CANCEL_CLICKED", clickedCancel);
let term = "";
for (let i = 0; i < 80; i += 1) { await settle(1000); const tt = await bodyText(); if (!/Benchmark running|Cancelling/.test(tt)) { term = /No v2 result/.test(tt) ? "idle-no-result" : /Decode throughput/.test(tt) ? "idle-with-result" : "idle"; break; } }
console.log("S2_TERMINAL", term, "| LLAMA", ll());
// MT-05.V3: stop/start must both work after the run/cancel sequence.
await nav("Profile");
await settle(800);
console.log("S3_STOP", await clickText("Stop server"));
console.log("S3_STOPPED", await waitLive(false, 30), "| LLAMA", ll());
console.log("S3_START", await clickText("Start"));
console.log("S3_STARTED", await waitLive(true, 90), "| LLAMA", ll());
console.log("E_DONE");
await client.close();
process.exit(0);
