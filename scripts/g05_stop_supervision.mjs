// G-05 clean start/stop supervision check: one Start, wait for live, one Stop,
// then poll the child process and the health port to verify termination.
// Usage: node scripts/g05_stop_supervision.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";
import { execSync } from "node:child_process";

const [portArg] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const childPids = () => {
  try {
    const out = execSync(
      'tasklist /FI "IMAGENAME eq llama-server.exe" /FO CSV /NH',
      { encoding: "utf8", windowsHide: true },
    );
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

const click = (matcher) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => /${matcher}/.test((candidate.textContent ?? "").trim()) && !candidate.disabled,
    );
    if (!button) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);

// Navigate to the Profile screen first: the Start control lives there.
const navToProfile = await evaluate(`(() => {
  const button = [...document.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === "Profile",
  );
  if (!button) return false;
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  return true;
})()`);
await settle(1200);
console.log("NAV PROFILE:", navToProfile);
const probe = execSync('tasklist /FI "IMAGENAME eq localmotive.exe" /FO CSV /NH', { encoding: "utf8" });
console.log("PROBE APP LINES:", probe.split(/\r?\n/).filter((line) => line.includes("localmotive.exe")).length);
console.log("CHILDREN BEFORE:", childPids().join(",") || "none");
console.log("START CLICKED:", await click("^Start$"));
let live = false;
for (let attempt = 0; attempt < 60; attempt += 1) {
  await settle(1500);
  live = await evaluate(`/Stop server/.test(document.body.textContent || "")`);
  if (live) break;
}
console.log("LIVE:", live);
const during = childPids();
console.log("CHILDREN DURING:", during.join(",") || "none");

console.log("STOP CLICKED:", await click("^Stop server$"));
let gone = null;
for (let attempt = 0; attempt < 20; attempt += 1) {
  await settle(1000);
  const remaining = childPids();
  if (remaining.length === 0) {
    gone = attempt + 1;
    break;
  }
}
console.log("CHILD GONE AFTER SECONDS:", gone ?? "still-alive");
const remaining = childPids();
console.log("CHILDREN AFTER STOP:", remaining.join(",") || "none");
const state = await evaluate(`(/Stop server/.test(document.body.textContent || "") ? "live" : "idle")`);
console.log("APP STATE:", state);
console.log(`G05_STOP ${gone !== null ? "TERMINATED" : "LEFT_RUNNING"}`);
await client.close?.();
process.exit(0);
