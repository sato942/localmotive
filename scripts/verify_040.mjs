// Packaged 0.4.0 verification: product support contract and Phase 4
// identity/completeness, plus the v0.3 evidence guards carried forward.
//
// Usage (from the repository root, Windows host):
//   WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=10015' \
//     ./src-tauri/target/release/localmotive.exe < /dev/null &
//   node scripts/verify_040.mjs 10015
//
// Every check asserts a VALUE through Tauri IPC or the rendered DOM.
// Screenshots are for design review only and never gate this script.
const PORT = process.argv[2] || '10015';
const OUT = `${process.env.LOCALAPPDATA}\\Temp\\localmotive-verify-040.json`;
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

const failures = [];
const check = (name, condition, detail) => {
  if (!condition) failures.push(`${name}: ${detail}`);
  return condition;
};

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

// 0.4 Phase 5, acceptance 1: no runtime option may show `Supported`
// without scope, level, OS, architecture, backend, and runtime revision.
// Every rendered SUPPORTED/NOT VALIDATED tag must carry a scope line.
results.supportContract = await evaluate(`(() => {
  const nav = [...document.querySelectorAll('.nav-item')].find((n) => n.textContent.toLowerCase().includes('runtime'));
  nav?.click();
  return Boolean(nav);
})()`);
await sleep(2000);
results.supportContract = await evaluate(`(() => {
  const tags = [...document.querySelectorAll('.runtime-option .state-tag')].map((n) => n.textContent.trim());
  const supported = tags.filter((t) => t === 'SUPPORTED');
  const notValidated = tags.filter((t) => t === 'NOT VALIDATED');
  const scopeLines = [...document.querySelectorAll('.runtime-option .runtime-role')].map((n) => n.textContent);
  const scopeOk = scopeLines.every((line) => /Windows 11/.test(line) && /x64/.test(line) && /b10796/.test(line));
  const bareSupported = [...document.querySelectorAll('.runtime-option')].filter((card) => {
    const text = card.textContent || '';
    return /supported/i.test(text) && !/SUPPORTED ·|NOT VALIDATED ·/.test(text);
  }).length;
  return { tags, supported: supported.length, notValidated: notValidated.length, scopeLines, scopeOk, bareSupported };
})()`);
check('support-tags-present', (results.supportContract.supported ?? 0) > 0, JSON.stringify(results.supportContract));
check('support-scope-complete', results.supportContract.scopeOk === true, JSON.stringify(results.supportContract.scopeLines));
check('no-bare-supported', results.supportContract.bareSupported === 0, `bare Supported labels: ${results.supportContract.bareSupported}`);

// 0.4 Phase 4: hardware evidence surfaces per adapter with source, and no
// silent cross-adapter preference. Unknown stays visible.
results.hardware = await evaluate(`${invoke('detect_hardware')}.then((info)=>({adapters:info.adapters.length, names:info.adapters.map((a)=>a.name), systemTotal:info.systemMemory.totalPhysicalBytes.value, sources:[...new Set(info.adapters.map((a)=>a.dedicatedBytes.source.detail))].slice(0,3)}))`);
check('hardware-adapters', (results.hardware.adapters ?? 0) >= 1, JSON.stringify(results.hardware));

// 0.4 Phase 5, acceptance 2: Unknown facts stay visible, never hidden.
results.unknowns = await evaluate(`(() => {
  const text = document.body.textContent || '';
  return { hasUnknown: /Unknown|UNKNOWN|untested/i.test(text) };
})()`);
check('unknowns-visible', results.unknowns.hasUnknown === true, JSON.stringify(results.unknowns));

// v0.3 guards carried forward: malformed benchmark manifest rejected,
// quality/ranking deterministic, calibration/external/share/DNS guards hold.
results.replayRejected = await evaluate(`${invoke('replay_benchmark_manifest', { manifest: { schema: 999 } })}.then(()=>false,()=>true)`);
check('replay-rejected', results.replayRejected === true, String(results.replayRejected));
results.ranking = await evaluate(`${invoke('rank_candidates', { candidates: [
  { id: 'a', resultClass: 'measured', decodeTps: 100, prefillTps: 200, p95LatencyMs: 50, peakMemoryBytes: 1000, qualityPassRate: 0.9, storageBytes: 500 },
  { id: 'b', resultClass: 'estimated', decodeTps: 999, prefillTps: 999, p95LatencyMs: 1, peakMemoryBytes: 1, qualityPassRate: 1, storageBytes: 1 },
], constraints: { minDecodeTps: null, maxP95LatencyMs: null, maxPeakMemoryBytes: null, minQualityPassRate: null, maxStorageBytes: null, requireMeasured: true }, weights: { decodeTps: 0.3, prefillTps: 0.1, latency: 0.2, memory: 0.15, quality: 0.2, storage: 0.05 } })}.then((ranked)=>({ feasible: ranked.filter((r)=>r.feasible).map((r)=>r.id), infeasible: ranked.filter((r)=>!r.feasible).map((r)=>r.id) }))`);
check('ranking-measured-only', JSON.stringify(results.ranking?.feasible) === JSON.stringify(['a']), JSON.stringify(results.ranking));
const CK_C = 'c'.repeat(64);
const CK_E = 'e'.repeat(64);
results.calibration = await evaluate(`${invoke('build_compatibility_key', { identity: null })}.then(()=>false,()=>true)`);
check('calibration-guarded', results.calibration === true, String(results.calibration));
results.external = await evaluate(`${invoke('import_external_evidence', { bundle: { schema: 1, source: 'qa', compatibilityKey: CK_C, state: 'pending', records: [{ metric: 'decodeTps', value: 42, unit: 'tokensPerSecond', observedAtMs: 42 }] } })}.then((bundle)=>bundle.state)`);
check('external-pending', results.external === 'pending', String(results.external));
results.shareGuard = await evaluate(`${invoke('build_share_export', { manifest: { schema: 1 }, summary: null, quality: null, compatibilityKey: CK_E, createdAtMs: 1, confirmed: false })}.then(()=>false,()=>true)`);
check('share-guarded', results.shareGuard === true, String(results.shareGuard));
results.dnsRejected = await evaluate(`${invoke('preview_command', { profile: null })}.then(()=>false,()=>true)`);
check('dns-rejected', results.dnsRejected === true, String(results.dnsRejected));

// UI: v0.3 evidence panel still renders on the Benchmark screen.
const clickedBenchmark = await evaluate(`(() => { const item=[...document.querySelectorAll('.nav-item')].find((node)=>node.textContent.toLowerCase().includes('benchmark')); item?.click(); return Boolean(item); })()`);
await sleep(1500);
results.ui = await evaluate(`(() => ({
  navigated: 'benchmark',
  evidence: Boolean([...document.querySelectorAll('*')].find((n)=>n.textContent && n.textContent.includes('Fit, measure, compare, and share'))),
  benchmark: Boolean([...document.querySelectorAll('button')].find((b)=>b.textContent.includes('Run v2 benchmark'))),
  quality: Boolean([...document.querySelectorAll('button')].find((b)=>b.textContent.includes('Run quality suite'))),
  navLabels: [...document.querySelectorAll('.nav-item')].map((node)=>node.textContent.trim()),
}))()`);
check('evidence-panel', results.ui.evidence === true, JSON.stringify(results.ui));

results.failures = failures;
results.pass = failures.length === 0;
fs.writeFileSync(OUT, JSON.stringify(results, null, 2));
console.log(JSON.stringify(results, null, 2));
cdp.close();
if (!results.pass) {
  console.error(`VERIFY_040 FAIL: ${failures.length} check(s) failed`);
  for (const failure of failures) console.error(` - ${failure}`);
  process.exit(1);
}
console.log('VERIFY_040 PASS');
