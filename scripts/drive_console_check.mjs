// Drive the packaged Localmotive over CDP to exercise every child-process path
// (hardware detection, runtime capability inspection, model scan, server start/stop)
// while watch_console_windows.py polls for console windows.
//
// P1-13 (LAB-03): attach through the shared scripts/lib/cdp_client.mjs instead
// of a third inline WebSocket copy.
import { attach } from "./lib/cdp_client.mjs";

const PORT = process.argv[2] || '10011';
const RUNTIME = process.argv[3] || 'C:\llama\llama-server.exe';
const MODEL_ROOT = process.argv[4] || 'C:\models';

async function evaluate(cdp, expression) {
  return cdp.evaluate(expression);
};

const results = {};

(async () => {
  const cdp = await attach(PORT, { deadlineMs: 30_000 });

  const invoke = (cmd, args) =>
    `window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(args || {})})`;

  // 1. Hardware detection: spawns nvidia-smi.exe twice and possibly powershell.exe.
  results.hardware = await evaluate(
    cdp,
    `${invoke('detect_hardware')}.then(h => ({vendor: h.vendor, arch: h.architecture, cudaMajor: h.cudaMajor, driver: h.driverVersion, status: h.detectionStatus}))`,
  );

  // 2. Runtime capability inspection: spawns llama-server.exe --version and --help.
  results.runtime = await evaluate(
    cdp,
    `${invoke('inspect_runtime', { path: RUNTIME })}.then(c => ({build: c.build, commit: c.commit, specTypes: c.specTypes.length, flags: c.supportedFlags.length}))`,
  );

  // 3. Model scan (no child process, but confirms the app is live).
  results.scan = await evaluate(
    cdp,
    `${invoke('scan_models', { root: MODEL_ROOT })}.then(m => ({models: m.length, complete: m.filter(x => x.complete).length}))`,
  );

  // 4. Start the model server: spawns llama-server.exe for real.
  const target0 = await evaluate(
    cdp,
    `${invoke('scan_models', { root: MODEL_ROOT })}.then(m => { const s=[...m].filter(x=>x.complete).sort((a,b)=>a.sizeBytes-b.sizeBytes)[0]; return s && {name: s.name, target: s.firstShard, size: s.sizeBytes}; })`,
  );
  results.chosen = target0;

  if (target0) {
    const profile = {
      name: 'console-window-check',
      runtime: RUNTIME,
      model: target0.target,
      alias: 'console-check',
      host: '127.0.0.1',
      port: 8137,
      context: 2048,
      gpuLayers: '99',
      specType: 'none',
    };
    results.start = await evaluate(
      cdp,
      `${invoke('start_server', { profile })}.then(s => ({running: s.running, pid: s.pid, port: s.port}), e => ({error: String(e)}))`,
    );
    await new Promise((r) => setTimeout(r, 9000));
    results.health = await evaluate(
      cdp,
      `fetch('http://127.0.0.1:8137/health').then(r => r.text(), e => 'ERR ' + e)`,
    );
    results.status = await evaluate(
      cdp,
      `${invoke('server_status')}.then(s => ({running: s.running, pid: s.pid}))`,
    );
    results.stop = await evaluate(
      cdp,
      `${invoke('stop_server')}.then(s => ({running: s.running}), e => ({error: String(e)}))`,
    );
  }

  console.log(JSON.stringify(results, null, 2));
  cdp.close();
  process.exit(0);
})().catch((e) => {
  console.log(JSON.stringify({ error: String(e), partial: results }, null, 2));
  process.exit(1);
});
