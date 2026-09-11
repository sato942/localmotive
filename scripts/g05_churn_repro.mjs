// G-05 churn repro: Start -> Cancel mid-start -> Start again -> Stop, three
// times; after each Stop, report any llama-server child that survives.
// Usage: node scripts/g05_churn_repro.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";
import { execSync } from "node:child_process";

const [portArg] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const childPids = () => {
  try {
    const out = execSync('tasklist /FI "IMAGENAME eq llama-server.exe" /FO CSV /NH', { encoding: "utf8" });
    const pids = [];
    for (const line of out.split(/\r?\n/)) {
      const match = line.match(/^"llama-server\.exe","(\d+)"/);
      if (match) pids.push(match[1]);
    }
    return pids;
  } catch {
    return [];
  }
};

const clickEnabled = (matcher) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => /${matcher}/.test((candidate.textContent ?? "").trim()) && !candidate.disabled,
    );
    if (!button) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);

const isLive = () => evaluate(`/Stop server/.test(document.body.textContent || "")`);

await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "Profile",
  );
  if (button) button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
await settle(1200);

for (let round = 1; round <= 3; round += 1) {
  console.log(`--- round ${round}`);
  const started = await clickEnabled("^Start$");
  console.log("start:", started);
  let cancelled = false;
  for (let attempt = 0; attempt < 30; attempt += 1) {
    await settle(200);
    cancelled = await clickEnabled("^Cancel");
    if (cancelled) break;
    if (await isLive()) break;
  }
  console.log("cancel:", cancelled, "children:", childPids().join(",") || "none");
  await settle(2500);
  const started2 = await clickEnabled("^Start$");
  console.log("start2:", started2);
  let live = false;
  for (let attempt = 0; attempt < 40; attempt += 1) {
    await settle(1000);
    live = await isLive();
    if (live) break;
  }
  console.log("live:", live, "children:", childPids().join(",") || "none");
  const stopped = await clickEnabled("^Stop server$");
  console.log("stop:", stopped);
  let leftover = childPids();
  for (let attempt = 0; attempt < 10 && leftover.length > 0; attempt += 1) {
    await settle(1000);
    leftover = childPids();
  }
  console.log(`round ${round} leftovers:`, leftover.join(",") || "none");
  if (leftover.length > 0) {
    console.log("ORPHAN_FOUND", leftover.join(","));
    break;
  }
}
await client.close?.();
process.exit(0);
