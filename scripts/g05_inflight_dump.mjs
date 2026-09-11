// In-flight v2 dump: buttons, cancels, banners.
import { attach } from "./lib/cdp_client.mjs";
const client = await attach(Number(process.argv[2] ?? 10070));
const dump = await client.evaluate(`(() => {
  const all = [...document.querySelectorAll("button")].map(b => ({ t: (b.textContent||"").trim().slice(0,24), dis: b.disabled, cls: b.className }));
  const bar = [...document.querySelectorAll(".notice-line,.notice,[class*=banner],[class*=progress],[class*=running]")].map(x => ({ c: x.className, t: (x.textContent||"").replace(/\\s+/g," ").trim().slice(0,140) }));
  return { btnCount: all.length, cancels: all.filter(b => /cancel/i.test(b.t)), notices: bar.slice(0,6), runv2: all.filter(b => /Run v2/.test(b.t)) };
})()`);
console.log(JSON.stringify(dump, null, 1));
await client.close();
process.exit(0);
