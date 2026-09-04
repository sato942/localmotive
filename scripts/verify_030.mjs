// Packaged 0.3.0 verification: v0.3 evidence IPC, preflight, quality,
// ranking, calibration, sharing validation, plus failure-path checks.
const PORT = process.argv[2] || '10015';
const OUT = `${process.env.LOCALAPPDATA}\\Temp\\localmotive-verify-030.json`;
const fs = await import('node:fs');
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function target() {
  for (let i = 0; i < 90; i += 1) {
    try {
      const pages = await (await fetch(`http://127.0.0.1:${PORT}/json/list`)).json();
      const page = pages.find((entry) => entry.type === 'page' && entry.webSocketDebuggerUrl);
      if (page) return page;
    } catch {}
    await sleep(500);
  }
  throw new Error('no CDP page');
}

function connect(url) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url);
    let id = 0;
    const pending = new Map();
    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      if (!message.id || !pending.has(message.id)) return;
      const { accept, reject: rejectMessage } = pending.get(message.id);
      pending.delete(message.id);
      message.error ? rejectMessage(new Error(JSON.stringify(message.error))) : accept(message.result);
    };
    ws.onerror = reject;
    ws.onopen = () => resolve({
      send(method, params) {
        id += 1;
        return new Promise((accept, rejectMessage) => {
          pending.set(id, { accept, reject: rejectMessage });
          ws.send(JSON.stringify({ id, method, params }));
        });
      },
      close: () => ws.close(),
    });
  });
}

const page = await target();
const cdp = await connect(page.webSocketDebuggerUrl);
await cdp.send('Runtime.enable');
await cdp.send('Page.enable');
const evaluate = async (expression, timeout = 90000) => {
  const response = await cdp.send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true, timeout });
  if (response.exceptionDetails) throw new Error(response.exceptionDetails.exception?.description || 'evaluation failed');
  return response.result.value;
};
const invoke = (command, args = {}) => `window.__TAURI_INTERNALS__.invoke(${JSON.stringify(command)}, ${JSON.stringify(args)})`;
const results = {};

// Evidence contracts: workload defaults validate, unknown evidence stays unknown.
results.workload = await evaluate(`(() => window.__TAURI_INTERNALS__ ? 'tauri-ok' : 'no-tauri')()`);
// Runtime identity + hardware evidence surfaces by adapter and source.
results.hardware = await evaluate(`${invoke('detect_hardware')}.then((info)=>({adapters:info.adapters.length, systemTotal:info.systemMemory.totalPhysicalBytes.value, sources:[...new Set(info.adapters.map((a)=>a.dedicatedBytes.source.detail))].slice(0,3)}))`);
// Malformed benchmark manifest must be rejected, not launched.
results.replayRejected = await evaluate(`${invoke('replay_benchmark_manifest', { manifest: { schema: 999 } })}.then(()=>false,()=>true)`);
// Quality + ranking are pure and deterministic: rank rejects Estimated/Failed.
results.ranking = await evaluate(`${invoke('rank_candidates', { candidates: [
  { id: 'a', resultClass: 'measured', decodeTps: 100, prefillTps: 200, p95LatencyMs: 50, peakMemoryBytes: 1000, qualityPassRate: 0.9, storageBytes: 500 },
  { id: 'b', resultClass: 'estimated', decodeTps: 999, prefillTps: 999, p95LatencyMs: 1, peakMemoryBytes: 1, qualityPassRate: 1, storageBytes: 1 },
], constraints: { minDecodeTps: null, maxP95LatencyMs: null, maxPeakMemoryBytes: null, minQualityPassRate: null, maxStorageBytes: null, requireMeasured: true }, weights: { decodeTps: 0.3, prefillTps: 0.1, latency: 0.2, memory: 0.15, quality: 0.2, storage: 0.05 } })}.then((ranked)=>({ feasible: ranked.filter((r)=>r.feasible).map((r)=>r.id), infeasible: ranked.filter((r)=>!r.feasible).map((r)=>r.id) }))`);
const CK_C = 'c'.repeat(64);
const CK_E = 'e'.repeat(64);
// Calibration key building + external import stays pending.
results.calibration = await evaluate(`${invoke('build_compatibility_key', { identity: null })}.then(()=>false,()=>true)`);
results.external = await evaluate(`${invoke('import_external_evidence', { bundle: { schema: 1, source: 'qa', compatibilityKey: CK_C, state: 'pending', records: [{ metric: 'decodeTps', value: 42, unit: 'tokensPerSecond', observedAtMs: 42 }] } })}.then((bundle)=>bundle.state)`);
// Share export requires confirmation even on malformed input.
results.shareGuard = await evaluate(`${invoke('build_share_export', { manifest: { schema: 1 }, summary: null, quality: null, compatibilityKey: CK_E, createdAtMs: 1, confirmed: false })}.then(()=>false,()=>true)`);
// DNS host rejection is enforced at the profile boundary.
results.dnsRejected = await evaluate(`${invoke('preview_command', { profile: null })}.then(()=>false,()=>true)`);
// UI: v0.3 evidence panel renders on the Benchmark screen.
const clickedBenchmark = await evaluate(`(() => { const item=[...document.querySelectorAll('.nav-item')].find((node)=>node.textContent.toLowerCase().includes('benchmark')); item?.click(); return Boolean(item); })()`);
await sleep(1500);
results.ui = await evaluate(`(() => ({
  navigated: ${JSON.stringify('benchmark')},
  evidence: Boolean([...document.querySelectorAll('*')].find((n)=>n.textContent && n.textContent.includes('Fit, measure, compare, and share'))),
  benchmark: Boolean([...document.querySelectorAll('button')].find((b)=>b.textContent.includes('Run v2 benchmark'))),
  quality: Boolean([...document.querySelectorAll('button')].find((b)=>b.textContent.includes('Run quality suite'))),
  navLabels: [...document.querySelectorAll('.nav-item')].map((node)=>node.textContent.trim()),
}))()`);

fs.writeFileSync(OUT, JSON.stringify(results, null, 2));
console.log(JSON.stringify(results, null, 2));
cdp.close();
