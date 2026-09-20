// FE-05.V3 packaged verification: benchmark profile A, leave to edit the
// profile and start B, benchmark B, compare both records with distinct
// provenance, and recover them through the supported saved-manifest replay
// route.
//
// Usage: node scripts/g05_fe05v3.mjs <debugPort>
import { attach } from "./lib/cdp_client.mjs";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

const [portArg] = process.argv.slice(2);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

let failed = 0;
const check = (name, ok, detail = "") => {
  if (!ok) failed += 1;
  console.log(`${ok ? "PASS" : "FAIL"} ${name}${detail ? ` | ${detail}` : ""}`);
};

const HOME = process.env.USERPROFILE ?? "C:\\Users\\Mubarak";
const BENCH_DIR = join(HOME, "AppData", "Roaming", "io.github.localmotive.app", "benchmarks");

const clickText = (text) =>
  evaluate(`(() => { const b = [...document.querySelectorAll("button")].find(x => (x.textContent ?? "").trim() === ${JSON.stringify(text)} && !x.disabled); if (!b) return false; b.dispatchEvent(new MouseEvent("click", { bubbles: true })); return true; })()`);
const nav = (label) => clickText(label);
const status = () => evaluate(`window.__TAURI_INTERNALS__.invoke("server_status")`);
const setAriaInput = (ariaLabel, value) =>
  evaluate(`(() => { const input = [...document.querySelectorAll("input,textarea")].find(x => (x.getAttribute("aria-label") ?? "") === ${JSON.stringify(ariaLabel)}); if (!input) return false; const proto = input.tagName === "TEXTAREA" ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype; const setter = Object.getOwnPropertyDescriptor(proto, "value").set; setter.call(input, ${JSON.stringify(value)}); input.dispatchEvent(new Event("input", { bubbles: true })); return true; })()`);
const setNamedInput = (labelText, value) =>
  evaluate(`(() => { const label = [...document.querySelectorAll("label")].find(x => (x.textContent ?? "").includes(${JSON.stringify(labelText)})); const input = label?.querySelector("input,textarea"); if (!input) return false; const proto = input.tagName === "TEXTAREA" ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype; const setter = Object.getOwnPropertyDescriptor(proto, "value").set; setter.call(input, ${JSON.stringify(value)}); input.dispatchEvent(new Event("input", { bubbles: true })); return true; })()`);
const readAriaValue = (ariaLabel) =>
  evaluate(`(() => { const input = [...document.querySelectorAll("input,textarea")].find(x => (x.getAttribute("aria-label") ?? "") === ${JSON.stringify(ariaLabel)}); return input ? input.value : null; })()`);
const panelResultText = () =>
  evaluate(`(() => { const section = document.querySelector("section[aria-labelledby=benchmark-v2-title]"); return section ? (section.innerText || "").replace(/\\s+/g, " ").slice(0, 1400) : null; })()`);
const waitFor = async (predicate, tries = 60, stepMs = 1000) => {
  for (let i = 0; i < tries; i += 1) {
    await settle(stepMs);
    if (await predicate()) return true;
  }
  return false;
};

const manifestsWithWorkload = (workloadId) => {
  if (!existsSync(BENCH_DIR)) return [];
  const results = [];
  for (const name of readdirSync(BENCH_DIR)) {
    if (!name.endsWith(".json")) continue;
    const path = join(BENCH_DIR, name);
    try {
      const parsed = JSON.parse(readFileSync(path, "utf8"));
      if (parsed?.workload?.id === workloadId) results.push({ path, parsed });
    } catch {
      // A malformed manifest is not this walk's subject; the panel quarantines those.
    }
  }
  // Newest first: a previous walk can leave older manifests with the same
  // workload id, and the current run is the subject.
  return results.sort((a, b) => statSync(b.path).mtimeMs - statSync(a.path).mtimeMs);
};

// ---------- Preconditions: server LIVE ----------
let st = await status();
if (!st.running) {
  await nav("Control");
  await settle(600);
  await clickText("Start profile");
  await waitFor(async () => (await status()).running === true, 90);
  st = await status();
}
check("fe05v3.baseline.reaches-live", st.running === true, `pid=${st.pid}`);

// ---------- Profile A: benchmark ----------
await nav("Benchmark");
await settle(1200);
const profileNameA = await evaluate(`(() => { const label = [...document.querySelectorAll("label")].find(x => (x.textContent ?? "").includes("Profile name")); return label?.querySelector("input")?.value ?? ""; })()`);
await setAriaInput("Benchmark workload ID", "fe05v3-a");
await setNamedInput("Benchmark warmups", "0");
await setNamedInput("Benchmark trials", "1");
await settle(400);
const runAClicked = await clickText("Run v2 benchmark");
const terminalA = await waitFor(async () => {
  const text = await panelResultText();
  return typeof text === "string" && /successful trials|No v2 result/i.test(text) && !/Benchmarking/.test(text);
}, 240, 1000);
const panelA = await panelResultText();
const measuredA = /[0-9]+\.[0-9]+ tok\/s/.test(panelA ?? "");
check("fe05v3.run-a-measured", runAClicked === true && terminalA && measuredA, `clicked=${runAClicked} measured=${measuredA}`);
const manifestsA = manifestsWithWorkload("fe05v3-a");
check("fe05v3.run-a-manifest-persisted", manifestsA.length >= 1, `count=${manifestsA.length} path=${manifestsA[0]?.path ?? "none"}`);

// ---------- Edit the profile and start B ----------
await nav("Profile");
await settle(800);
await setNamedInput("Profile name", "fe05v3-b");
await setNamedInput("Context", "4096");
await settle(300);
const savedB = await clickText("Save");
check("fe05v3.profile-b-saved", savedB === true, "");
await nav("Control");
await settle(600);
await clickText("Stop server");
await waitFor(async () => (await status()).running === false, 30);
await clickText("Start profile");
const liveB = await waitFor(async () => (await status()).running === true, 90);
const stB = await status();
check("fe05v3.profile-b-started", liveB && stB.pid !== st.pid, `pidA=${st.pid} pidB=${stB.pid}`);

// ---------- Profile B: benchmark ----------
await nav("Benchmark");
await settle(1200);
await setAriaInput("Benchmark workload ID", "fe05v3-b");
await setNamedInput("Benchmark warmups", "0");
await setNamedInput("Benchmark trials", "1");
await settle(400);
const runBClicked = await clickText("Run v2 benchmark");
const terminalB = await waitFor(async () => {
  const text = await panelResultText();
  return typeof text === "string" && /successful trials|No v2 result/i.test(text) && !/Benchmarking/.test(text);
}, 240, 1000);
const panelB = await panelResultText();
const measuredB = /[0-9]+\.[0-9]+ tok\/s/.test(panelB ?? "");
check("fe05v3.run-b-measured", runBClicked === true && terminalB && measuredB, `clicked=${runBClicked} measured=${measuredB}`);
const manifestsB = manifestsWithWorkload("fe05v3-b");
check("fe05v3.run-b-manifest-persisted", manifestsB.length >= 1, `count=${manifestsB.length} path=${manifestsB[0]?.path ?? "none"}`);

// ---------- Compare provenance ----------
const resolveContext = (manifest) => {
  const args = Array.isArray(manifest?.launch?.commandArgs) ? manifest.launch.commandArgs : [];
  const index = args.indexOf("-c");
  return index >= 0 ? args[index + 1] : null;
};
const ctxA = manifestsA[0] ? resolveContext(manifestsA[0].parsed) : null;
const ctxB = manifestsB[0] ? resolveContext(manifestsB[0].parsed) : null;
const keyA = manifestsA[0]?.parsed?.compatibilityKey ?? null;
const keyB = manifestsB[0]?.parsed?.compatibilityKey ?? null;
check("fe05v3.records-distinct-paths", Boolean(manifestsA[0] && manifestsB[0] && manifestsA[0].path !== manifestsB[0].path), `A=${manifestsA[0]?.path ?? "none"} B=${manifestsB[0]?.path ?? "none"}`);
check("fe05v3.records-distinct-provenance", ctxA === "8192" && ctxB === "4096", `contextA=${ctxA} contextB=${ctxB}`);
check(
  "fe05v3.records-carry-identities",
  Boolean(keyA && keyB && keyA !== keyB) &&
    (manifestsA[0]?.parsed?.executionSnapshotSchema === "localmotive.execution-snapshot.v2") &&
    (manifestsB[0]?.parsed?.executionSnapshotSchema === "localmotive.execution-snapshot.v2"),
  `keyA=${String(keyA).slice(0, 20)} keyB=${String(keyB).slice(0, 20)} schema=${manifestsB[0]?.parsed?.executionSnapshotSchema ?? "none"}`,
);

// ---------- Recovery through the supported saved-manifest route ----------
const replayed = await clickText("Replay manifest");
const replayRoundtrip = await waitFor(async () => (await readAriaValue("Benchmark workload ID")) === "fe05v3-b", 10, 500);
check("fe05v3.replay-manifest-route", replayed === true && replayRoundtrip, `workload=${await readAriaValue("Benchmark workload ID")}`);

// ---------- Restore the shared machine state ----------
await nav("Profile");
await settle(700);
await setNamedInput("Profile name", profileNameA || "smollm2-135m");
await setNamedInput("Context", "8192");
await settle(300);
await clickText("Save");
await nav("Control");
await settle(600);
await clickText("Stop server");
await waitFor(async () => (await status()).running === false, 30);

console.log(`FE05V3_SUMMARY ${failed === 0 ? "ALL-PASS" : `${failed}-FAILED`}`);
process.exit(failed === 0 ? 0 : 1);
