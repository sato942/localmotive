// Start a v2 run and dump the in-flight state at two points.
import { attach } from "./lib/cdp_client.mjs";
const client = await attach(Number(process.argv[2] ?? 10070));
const ev = (e) => client.evaluate(e);
const settle = (ms) => new Promise((r) => setTimeout(r, ms));
const dump = () => ev(`(() => {
  const all = [...document.querySelectorAll("button")].map(b => ({ t: (b.textContent||"").trim().slice(0,26), dis: b.disabled, cls: b.className }));
  const bar = [...document.querySelectorAll(".notice-line,.notice,[class*=banner],[class*=progress],[class*=running]")].map(x => ({ c: x.className, t: (x.textContent||"").replace(/\\s+/g," ").trim().slice(0,120) }));
  const t = (document.body.innerText || "").replace(/\\s+/g, " ");
  const mark = (t.match(/Benchmark running|Cancelling|Run v2 benchmark|SAMPLES|Decode throughput|No v2 result/g) || []).slice(-4);
  return { cancels: all.filter(b => /cancel/i.test(b.t)), notices: bar.slice(0,4), marks: mark };
})()`);
console.log("CLICK", await ev('(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === "Run v2 benchmark" && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()'));
await settle(6000);
console.log("T6", JSON.stringify(await dump(), null, 1));
await settle(9000);
console.log("T15", JSON.stringify(await dump(), null, 1));
await client.close();
process.exit(0);
