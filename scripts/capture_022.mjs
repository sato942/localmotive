// Capture the AI Tune and Runtime screens of the packaged app at desktop and
// 390x844 for the batched visual review. Usage: node scripts/capture_022.mjs <cdpPort>
const PORT = process.argv[2] || '10012';
const fs = await import('node:fs');
const OUT = '.impeccable/review';
fs.mkdirSync(OUT, { recursive: true });

const pages = await (await fetch(`http://127.0.0.1:${PORT}/json/list`)).json();
const page = pages.find((p) => p.type === 'page' && p.webSocketDebuggerUrl);
const ws = new WebSocket(page.webSocketDebuggerUrl);
let id = 0;
const pending = new Map();
ws.onmessage = (ev) => {
  const m = JSON.parse(ev.data);
  if (m.id && pending.has(m.id)) {
    const { res, rej } = pending.get(m.id);
    pending.delete(m.id);
    m.error ? rej(new Error(JSON.stringify(m.error))) : res(m.result);
  }
};
await new Promise((r) => (ws.onopen = r));
const send = (method, params) =>
  new Promise((res, rej) => {
    id += 1;
    pending.set(id, { res, rej });
    ws.send(JSON.stringify({ id, method, params }));
  });
const evaluate = async (expression) => {
  const r = await send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
  if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description || 'eval failed');
  return r.result.value;
};
const shot = async (name) => {
  const r = await send('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(`${OUT}/${name}.png`, Buffer.from(r.data, 'base64'));
};
const clickNav = async (label) => {
  await evaluate(`(() => { const b=[...document.querySelectorAll('.nav-item')].find(x=>x.textContent.trim().toLowerCase().includes(${JSON.stringify(label)})); b && b.click(); return !!b; })()`);
  await new Promise((r) => setTimeout(r, 900));
};
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

await send('Page.enable');
await send('Emulation.clearDeviceMetricsOverride').catch(() => {});
await evaluate(`localStorage.setItem('gguf-pilot:runtime','C:\\\\llama\\\\llama-server.exe'); localStorage.setItem('gguf-pilot:model-root','C:\\\\models'); location.reload(); true`);
await sleep(7000);

// Desktop (native window size)
const results = {};
results.viewport = await evaluate(`({w: innerWidth, h: innerHeight})`);
await clickNav('runtime');
await sleep(3500);
await shot('v022-runtime-desktop');
await clickNav('ai tune');
await sleep(1500);
await shot('v022-tune-desktop');
results.tuneDesktop = await evaluate(`({
  tabs: [...document.querySelectorAll('.provider-tab')].map(t=>({label:t.textContent, active:t.classList.contains('active'), h: t.getBoundingClientRect().height})),
  steps: [...document.querySelectorAll('.setup-step')].map(s=>s.className),
  hasTrialLog: !!document.querySelector('.trial-list'),
  hasResult: !!document.querySelector('.tune-results .result-main') || !!document.querySelector('.tune-results .empty-result'),
  autoTuneDisabled: document.querySelector('.tune-screen .actions .button')?.disabled,
  overflow: [...document.querySelectorAll('.tune-screen *')].filter(e=>e.scrollWidth>e.clientWidth+1 && getComputedStyle(e).overflowX==='visible').slice(0,5).map(e=>e.className)
})`);
// Feed a stored report (from the live run) so the ledger and result render with real data.
const live = JSON.parse(fs.readFileSync(`${process.env.LOCALAPPDATA}\\Temp\\gguf-pilot-live-022.json`, 'utf8'));
if (live.tuning && !live.tuning.error) {
  const modelId = await evaluate(`window.__TAURI_INTERNALS__.invoke('scan_models',{root:'C:\\\\models'}).then(m=>m.find(x=>x.name.includes('LFM2.5-2.6B'))?.id)`);
  const report = { ...live.tuning, trials: live.tuning.trials.map((t) => ({ index: t.index, changes: t.changes, rationale: t.rationale, meanTps: t.mean, medianTps: t.mean, error: t.error, command: '' })), bestProfile: { ...live.tuning.bestProfile } };
  // bestProfile from the driver is partial; fetch a full profile shape via a fresh suggested one is not available here, so store the report and let the UI read numbers only.
  await evaluate(`localStorage.setItem('gguf-pilot:tuning:' + ${JSON.stringify(modelId)}, ${JSON.stringify(JSON.stringify(report))}); true`);
  await clickNav('inventory');
  await sleep(2500);
  await evaluate(`(() => { const row=[...document.querySelectorAll('.table-row')].find(r=>r.textContent.includes('LFM2.5-2.6B')); row && row.click(); return !!row; })()`);
  await sleep(800);
  await clickNav('ai tune');
  await sleep(1500);
  await shot('v022-tune-desktop-report');
  results.tuneReport = await evaluate(`({
    result: document.querySelector('.tune-results .result-main strong')?.textContent,
    range: document.querySelector('.result-range')?.textContent,
    trials: document.querySelectorAll('.trial-entry').length,
    bestEntries: document.querySelectorAll('.trial-entry.best').length,
    failed: document.querySelectorAll('.trial-entry.failed').length,
    adopt: document.querySelector('.tune-actions .button')?.textContent
  })`);
}

// Mobile 390x844
await send('Emulation.setDeviceMetricsOverride', { width: 390, height: 844, deviceScaleFactor: 2, mobile: true });
await sleep(1200);
await shot('v022-tune-mobile');
results.tuneMobile = await evaluate(`({
  inner: [innerWidth, innerHeight],
  scrollW: document.documentElement.scrollWidth,
  nav: [...document.querySelectorAll('.nav-item')].map(n=>{const r=n.getBoundingClientRect(); return {label:n.textContent.trim(), w:Math.round(r.width), h:Math.round(r.height)}}),
  tabs: [...document.querySelectorAll('.provider-tab')].map(t=>{const r=t.getBoundingClientRect(); return {w:Math.round(r.width), h:Math.round(r.height)}}),
  overflow: [...document.querySelectorAll('*')].filter(e=>e.getBoundingClientRect().right>innerWidth+1).slice(0,6).map(e=>e.className||e.tagName)
})`);
await evaluate(`window.scrollTo(0, document.documentElement.scrollHeight); true`);
await sleep(500);
await shot('v022-tune-mobile-bottom');
results.tuneMobileBottom = await evaluate(`(() => { const nav=document.querySelector('.rail nav').getBoundingClientRect(); const all=[...document.querySelectorAll('.screen *')].map(e=>e.getBoundingClientRect().bottom).filter(b=>b>0); return {navTop: Math.round(nav.top), lastContent: Math.round(Math.max(...all)), scrollY: Math.round(scrollY)}; })()`);
await clickNav('runtime');
await sleep(2500);
await evaluate(`window.scrollTo(0, 0); true`);
await shot('v022-runtime-mobile');
results.runtimeMobile = await evaluate(`({
  buttons: [...document.querySelectorAll('.runtime-option-action .button')].map(b=>({t:b.textContent.trim(), h:Math.round(b.getBoundingClientRect().height), disabled:b.disabled})),
  overflow: [...document.querySelectorAll('*')].filter(e=>e.getBoundingClientRect().right>innerWidth+1).slice(0,6).map(e=>e.className||e.tagName)
})`);
await send('Emulation.clearDeviceMetricsOverride');

fs.writeFileSync(`${OUT}/v022-metrics.json`, JSON.stringify(results, null, 2));
console.log(JSON.stringify(results, null, 2));
ws.close();
process.exit(0);
