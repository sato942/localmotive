// MT-06 extension (2026-09-12 review): repeated cancel/restart cycles on the
// packaged binary with resource measurements, not just caller latency.
// Per cycle: run v2 benchmark -> cancel mid-flight -> the run ends -> the
// server reports zero in-flight requests (/metrics requests_processing) and a
// single owned child process. Every other cycle stops the server (children
// reach zero) and starts it again (reservation released, listener re-owned).
// After the final stop the listener must be gone.
//
// Usage: node scripts/g05_mt06_cycles.mjs <debugPort> [cycles]
import { attach } from "./lib/cdp_client.mjs";
import { execSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { connect } from "node:net";
import https from "node:https";

const [portArg, cyclesArg] = process.argv.slice(2);
const CYCLES = Number(cyclesArg ?? 6);
const client = await attach(Number(portArg));
const evaluate = (expr) => client.evaluate(expr);
const settle = (ms) => new Promise((resolvePromise) => setTimeout(resolvePromise, ms));
const API_KEY = readFileSync(join(process.cwd(), ".hermes-0.6", "g05-tls", "api-key.txt"), "utf8").trim();

const clickExact = (label) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === ${JSON.stringify(label)} && !candidate.disabled,
    );
    if (!button) return false;
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    return true;
  })()`);

const buttonState = (label) =>
  evaluate(`(() => {
    const button = [...document.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === ${JSON.stringify(label)},
    );
    if (!button) return "absent";
    return button.disabled ? "disabled" : "enabled";
  })()`);

const lastNotice = () =>
  evaluate(`(() => {
    const lines = [...document.querySelectorAll(".notice-line")].map((node) =>
      (node.textContent ?? "").replace(/\\s+/g, " ").trim(),
    ).filter(Boolean);
    return lines.length ? lines[lines.length - 1].slice(0, 240) : null;
  })()`);

// Only one screen is mounted at a time; the Start/Stop controls live on the
// Profile screen, so navigate there before using them (with refuted clicks
// retried while the child is still alive).
const stopServer = async () => {
  for (let attempt = 0; attempt < 3; attempt += 1) {
    // The stop control lives on the Control screen ("Stop server" /
    // "Cancel start"); the Profile screen only carries Start.
    await clickExact("Control");
    await settle(1200);
    const stopped = await clickExact("Stop server");
    if (!stopped) await clickExact("Cancel start");
    let zero = false;
    for (let wait = 0; wait < 12 && !zero; wait += 1) {
      await settle(1000);
      zero = llamaProcs() === 0;
    }
    if (zero) return true;
  }
  return llamaProcs() === 0;
};

const llamaProcs = () => {
  try {
    const out = execSync('tasklist /FI "IMAGENAME eq llama-server.exe" /FO CSV /NH', { encoding: "utf8" });
    return out.split(/\r?\n/).filter((line) => line.toLowerCase().includes("llama-server.exe")).length;
  } catch {
    return 0;
  }
};

const metricsField = (name) =>
  new Promise((resolvePromise) => {
    const request = https.request(
      {
        host: "127.0.0.1",
        port: 8080,
        path: "/metrics",
        method: "GET",
        headers: { Authorization: `Bearer ${API_KEY}` },
        rejectUnauthorized: false,
        timeout: 4000,
      },
      (response) => {
        let data = "";
        response.on("data", (chunk) => {
          data += chunk;
        });
        response.on("end", () => {
          const match = data.match(new RegExp(`^${name}\\s+(\\S+)$`, "m"));
          resolvePromise(match ? match[1] : null);
        });
      },
    );
    request.on("error", (error) => resolvePromise(`ERR:${error.code ?? error.message}`));
    request.on("timeout", () => {
      request.destroy();
      resolvePromise("ERR:timeout");
    });
    request.end();
  });

const listenerAlive = () =>
  new Promise((resolvePromise) => {
    const socket = connect({ host: "127.0.0.1", port: 8080 });
    const done = (value) => {
      socket.destroy();
      resolvePromise(value);
    };
    socket.on("connect", () => done(true));
    socket.on("error", () => done(false));
    setTimeout(() => done(false), 2000);
  });

const checks = [];
const check = (name, ok, detail = "") => {
  checks.push({ name, ok });
  console.log(`${ok ? "PASS" : "FAIL"} ${name}${detail ? ` | ${detail}` : ""}`);
};

// --- Preconditions: live server, artifact inspected, preflight run once ----
await clickExact("Profile");
await settle(1200);
let live = await evaluate(`/Stop server/.test(document.body.textContent || "")`);
if (!live) {
  await clickExact("Start");
  for (let attempt = 0; attempt < 60 && !live; attempt += 1) {
    await settle(1500);
    live = await evaluate(`/Stop server/.test(document.body.textContent || "")`);
  }
}
check("mt06.precondition-server-live", live, live ? "" : `notice=${await lastNotice()}`);
if (!live) {
  console.log("MT06 SUMMARY: server did not start");
  process.exit(1);
}
await clickExact("Benchmark");
await settle(1200);
await clickExact("Inspect artifact");
await settle(2500);
await clickExact("Run preflight");
await settle(4000);

// --- Cycles ----------------------------------------------------------------
const results = [];
for (let cycle = 1; cycle <= CYCLES; cycle += 1) {
  await clickExact("Benchmark");
  await settle(1200);
  const started = await clickExact("Run v2 benchmark");
  let inFlight = false;
  for (let attempt = 0; attempt < 40; attempt += 1) {
    await settle(1000);
    if ((await buttonState("Cancel")) === "enabled") {
      inFlight = true;
      break;
    }
  }
  const cancelled = inFlight ? await clickExact("Cancel") : false;
  const cancelAt = Date.now();
  let settled = false;
  let notice = null;
  for (let attempt = 0; attempt < 120 && !settled; attempt += 1) {
    await settle(1000);
    notice = await lastNotice();
    const cancelState = await buttonState("Cancel");
    const panelFinished = await evaluate(`/Benchmark finished|No benchmark is running/i.test(document.body.innerText || "")`);
    if (cancelState !== "enabled" || panelFinished) {
      settled = true;
    }
    if (Date.now() - cancelAt > 150000) break;
  }
  // Resource measurements after the run ends.
  let processing = await metricsField("llamacpp:requests_processing");
  let drained = processing === "0";
  for (let attempt = 0; attempt < 20 && !drained; attempt += 1) {
    await settle(1000);
    processing = await metricsField("llamacpp:requests_processing");
    drained = processing === "0";
  }
  const procs = llamaProcs();
  results.push({ cycle, started, inFlight, cancelled, settled, processing, procs });
  console.log(
    `CYCLE ${cycle}: started=${started} inFlight=${inFlight} cancelled=${cancelled} settled=${settled} requests_processing=${processing} children=${procs} notice="${String(notice ?? "").slice(0, 90)}"`,
  );
  check(`mt06.cycle-${cycle}-cancel-in-flight`, started && inFlight && cancelled);
  check(`mt06.cycle-${cycle}-run-ended`, settled);
  check(`mt06.cycle-${cycle}-requests-drained`, drained, `requests_processing=${processing}`);
  check(`mt06.cycle-${cycle}-children-bounded`, procs === 1, `children=${procs}`);

  if (cycle % 2 === 0) {
    const zero = await stopServer();
    check(`mt06.cycle-${cycle}-stop-clears-children`, zero, `children=${llamaProcs()}`);
    check(`mt06.cycle-${cycle}-listener-released`, !(await listenerAlive()));
    await settle(1500);
    await clickExact("Control");
    await settle(800);
    await clickExact("Start profile");
    let back = false;
    for (let attempt = 0; attempt < 60 && !back; attempt += 1) {
      await settle(1500);
      back = await evaluate(`/Stop server/.test(document.body.textContent || "")`);
    }
    check(`mt06.cycle-${cycle}-restart-reaches-live`, back);
  }
}

// --- Final teardown ---------------------------------------------------------
const finalZero = await stopServer();
check("mt06.final-stop-clears-children", finalZero, `children=${llamaProcs()}`);
check("mt06.final-listener-released", !(await listenerAlive()));
check("mt06.worker-requests-never-orphaned", results.every((entry) => entry.processing === "0"));

const failed = checks.filter((entry) => !entry.ok);
console.log(`MT06 SUMMARY: ${checks.length - failed.length}/${checks.length} checks PASS`);
process.exit(failed.length === 0 ? 0 : 1);
