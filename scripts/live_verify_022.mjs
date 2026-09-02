// Live verification of 0.2.2 over CDP against the packaged app:
//   A. Runtime Manager option states (active / update / use / install)
//   B. Cloud credential round-trip through Windows Credential Manager
//   C. A real AI tuning run (cloud advisor + local llama-server measurements)
//   D. Credential removal afterwards
// The API key is read from the environment inside THIS process and handed to the
// app's own `cloud_save_credential` command. It is never printed.
const PORT = process.argv[2] || '10012';
const RUNTIME = process.argv[3] || 'C:\\llama\\llama-server.exe';
const MODEL_ROOT = process.argv[4] || 'C:\\models';
const MODEL_HINT = process.argv[5] || 'LFM2.5-2.6B';
const CLOUD_MODEL = process.argv[6] || 'anthropic/claude-sonnet-4.6';
const TRIALS = Number(process.argv[7] || 4);
const CONTEXT = Number(process.argv[8] || 8192);

async function target() {
  for (let i = 0; i < 60; i += 1) {
    try {
      const pages = await (await fetch(`http://127.0.0.1:${PORT}/json/list`)).json();
      const page = pages.find((p) => p.type === 'page' && p.webSocketDebuggerUrl);
      if (page) return page;
    } catch {}
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error('no CDP page');
}

function connect(url) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url);
    let id = 0;
    const pending = new Map();
    const events = [];
    ws.onmessage = (ev) => {
      const msg = JSON.parse(ev.data);
      if (msg.id && pending.has(msg.id)) {
        const { resolve: res, reject: rej } = pending.get(msg.id);
        pending.delete(msg.id);
        if (msg.error) rej(new Error(JSON.stringify(msg.error)));
        else res(msg.result);
      } else if (msg.method === 'Runtime.consoleAPICalled') {
        events.push(msg.params.args.map((a) => a.value ?? a.description).join(' '));
      }
    };
    ws.onerror = reject;
    ws.onopen = () =>
      resolve({
        events,
        send(method, params) {
          id += 1;
          const mid = id;
          return new Promise((res, rej) => {
            pending.set(mid, { resolve: res, reject: rej });
            ws.send(JSON.stringify({ id: mid, method, params }));
          });
        },
        close: () => ws.close(),
      });
  });
}

async function evaluate(cdp, expression, timeoutMs = 60000) {
  const r = await cdp.send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true, timeout: timeoutMs });
  if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description || JSON.stringify(r.exceptionDetails));
  return r.result.value;
}

const results = {};
const invoke = (cmd, args) => `window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(args || {})})`;

const OUT = process.env.LIVE_OUT || `${process.env.LOCALAPPDATA}\\Temp\\gguf-pilot-live-022.json`;
async function emit(obj, code) {
  const fs = await import('node:fs');
  fs.writeFileSync(OUT, JSON.stringify(obj, null, 2));
  process.stdout.write(JSON.stringify(obj, null, 2) + '\n');
  process.exit(code);
}

(async () => {
  const page = await target();
  const cdp = await connect(page.webSocketDebuggerUrl);
  await cdp.send('Runtime.enable');

  // ---- A. Runtime option states -------------------------------------------
  results.identity = await evaluate(cdp, `${invoke('describe_runtime', { path: RUNTIME })}`);
  results.managedBefore = await evaluate(cdp, `${invoke('list_managed_runtimes')}`);
  const catalog = await evaluate(cdp, `${invoke('fetch_runtime_catalog')}.then(c => ({tag: c.tag, options: c.options.map(o => ({id: o.id, backend: o.backend, installKey: o.installKey, recommended: o.recommended}))}))`);
  results.catalog = catalog;
  // Drive the UI: select the runtime and open the Runtime tab, then read the button labels.
  await evaluate(cdp, `localStorage.setItem('gguf-pilot:runtime', ${JSON.stringify(RUNTIME)}); localStorage.setItem('gguf-pilot:model-root', ${JSON.stringify(MODEL_ROOT)}); location.reload(); true`);
  await new Promise((r) => setTimeout(r, 6500));
  await evaluate(cdp, `(() => { const b=[...document.querySelectorAll('.nav-item')].find(x=>/runtime/i.test(x.textContent)); b && b.click(); return true; })()`);
  await new Promise((r) => setTimeout(r, 4000));
  results.runtimeUi = await evaluate(cdp, `[...document.querySelectorAll('.runtime-option')].map(o => ({title: o.querySelector('.runtime-option-title strong')?.textContent, tags: [...o.querySelectorAll('.state-tag')].map(t=>t.textContent), role: o.querySelector('.runtime-role')?.textContent, button: o.querySelector('.runtime-option-action .button')?.textContent?.trim(), disabled: o.querySelector('.runtime-option-action .button')?.disabled, active: o.classList.contains('is-active')}))`);
  results.sidebarTrust = await evaluate(cdp, `[...document.querySelectorAll('.runtime-trust')].map(t => t.textContent)`);
  results.navCount = await evaluate(cdp, `document.querySelectorAll('.nav-item').length`);

  // ---- B. Credential round-trip -----------------------------------------------
  // Read the key inside this process only: env var first, then Hermes' own .env.
  // It is passed to the app's `cloud_save_credential` command and never printed.
  let key = process.env.OPENROUTER_API_KEY;
  if (!key) {
    const fs = await import('node:fs');
    const envPath = `${process.env.LOCALAPPDATA}\\hermes\\.env`;
    const line = fs.readFileSync(envPath, 'utf8').split(/\r?\n/).find((l) => /^\s*(export\s+)?OPENROUTER_API_KEY\s*=/.test(l));
    if (line) key = line.split('=').slice(1).join('=').trim().replace(/^["']|["']$/g, '');
  }
  if (!key) throw new Error('OPENROUTER_API_KEY not available to this process');
  results.keySource = process.env.OPENROUTER_API_KEY ? 'env' : 'hermes .env (read in-process)';
  results.keyShape = { length: key.length, prefix: key.slice(0, 6) + '…' };
  results.credentialBefore = await evaluate(cdp, `${invoke('cloud_credential_status', { provider: 'openrouter' })}`);
  results.credentialSaved = await evaluate(cdp, `${invoke('cloud_save_credential', { provider: 'openrouter', secret: key })}`);
  results.modelsListed = await evaluate(cdp, `${invoke('cloud_list_models', { provider: 'openrouter' })}.then(m => ({count: m.length, hasTarget: m.some(x => x.id === ${JSON.stringify(CLOUD_MODEL)}), sample: m.slice(0,3).map(x=>x.id)}))`, 90000);
  results.probe = await evaluate(cdp, `${invoke('cloud_probe', { provider: 'openrouter', model: CLOUD_MODEL })}`, 90000);

  // ---- C. Real tuning run -----------------------------------------------------
  const chosen = await evaluate(cdp, `${invoke('scan_models', { root: MODEL_ROOT })}.then(m => { const s = m.find(x => x.complete && x.name.includes(${JSON.stringify(MODEL_HINT)})); return s && {id: s.id, name: s.name, firstShard: s.firstShard, companions: s.companions.map(c => c.role + ': ' + c.path)}; })`);
  results.chosen = chosen && { name: chosen.name, companions: chosen.companions };
  if (!chosen) throw new Error(`no complete model matching ${MODEL_HINT}`);
  results.gguf = await evaluate(cdp, `${invoke('read_gguf_summary', { path: chosen.firstShard })}.then(g => ({arch: g.architecture, ctx: g.contextLength, layers: g.blockCount, size: g.sizeLabel, quant: g.fileType}))`);

  const profile = {
    name: 'live-verify',
    runtime: RUNTIME,
    model: chosen.firstShard,
    alias: 'live-verify',
    host: '127.0.0.1',
    port: 8141,
    context: CONTEXT,
    gpuLayers: '99',
    specType: 'none',
  };
  const progress = [];
  const t0 = Date.now();
  results.tuning = await evaluate(
    cdp,
    `${invoke('start_tuning', { request: { profile, provider: 'openrouter', model: CLOUD_MODEL, targetContext: CONTEXT, maxTrials: TRIALS, tokens: 192, repeats: 2, companions: chosen.companions } })}.then(r => ({
      baselineTps: r.baselineTps, bestTps: r.bestTps, bestIndex: r.bestIndex, stoppedReason: r.stoppedReason, targetContext: r.targetContext, provider: r.provider, model: r.model,
      trials: r.trials.map(t => ({index: t.index, changes: t.changes, mean: t.meanTps, error: t.error ? String(t.error).slice(0, 200) : null, rationale: t.rationale.slice(0, 220)})),
      bestProfile: { context: r.bestProfile.context, gpuLayers: r.bestProfile.gpuLayers, batch: r.bestProfile.batch, ubatch: r.bestProfile.ubatch, flashAttention: r.bestProfile.flashAttention, cacheTypeK: r.bestProfile.cacheTypeK, cacheTypeV: r.bestProfile.cacheTypeV, specType: r.bestProfile.specType, host: r.bestProfile.host, port: r.bestProfile.port, alias: r.bestProfile.alias }
    }), e => ({ error: String(e) }))`,
    20 * 60 * 1000,
  );
  results.tuningSeconds = Math.round((Date.now() - t0) / 1000);
  results.serverAfter = await evaluate(cdp, `${invoke('server_status')}.then(s => ({running: s.running}))`);

  // ---- D. Remove the credential -------------------------------------------------
  results.credentialCleared = await evaluate(cdp, `${invoke('cloud_clear_credential', { provider: 'openrouter' })}`);
  results.credentialAfter = await evaluate(cdp, `${invoke('cloud_credential_status', { provider: 'openrouter' })}`);

  cdp.close();
  await emit(results, 0);
})().catch(async (e) => {
  await emit({ error: String(e), stack: e?.stack?.split('\n').slice(0, 4), partial: results }, 1);
});
