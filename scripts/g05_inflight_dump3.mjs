// Click Run v2 and dump the panel's message/error surfaces right after.
import { attach } from "./lib/cdp_client.mjs";
const client = await attach(Number(process.argv[2] ?? 10070));
const ev = (e) => client.evaluate(e);
const settle = (ms) => new Promise((r) => setTimeout(r, ms));
const probe = () => ev(`(() => {
  const msgs = [...document.querySelectorAll(".evidence-message,.field-error,[role=alert],.error")].map(x => (x.textContent||"").replace(/\\s+/g," ").trim().slice(0,200)).filter(Boolean);
  const inputs = [...document.querySelectorAll("input")].map(x => ({ l: (x.closest("label")?.textContent||"").trim().slice(0,28), v: x.value.slice(0,40), invalid: x.getAttribute("aria-invalid") }));
  const t = (document.body.innerText||"").replace(/\\s+/g," ");
  const running = /Benchmark running|Cancelling/.test(t);
  return { msgs, inputs: inputs.filter(x => /prompt|generated|warmup|trial|limit|tok/i.test(x.l)), running };
})()`);
console.log("PRE", JSON.stringify(await probe(), null, 1).slice(0, 900));
console.log("CLICK", await ev('(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === "Run v2 benchmark" && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()'));
await settle(3500);
console.log("POST3", JSON.stringify(await probe(), null, 1).slice(0, 1400));
await client.close();
process.exit(0);
