// Verify the 0.2.4 items on the packaged app over CDP:
//   3. runtime persistence across a restart and after switching runtimes
//   4. About tab present and populated
//   5. DeepSeek DSpark companions ranked + counted chip + picker
//   6. free-port selection when 8080 is occupied
const PORT = process.argv[2] || '10013';
const RUNTIME = process.argv[3] || 'C:\\llama\\llama-server.exe';
const MODEL_ROOT = process.argv[4] || 'C:\\models';
const OUT = `${process.env.LOCALAPPDATA}\\Temp\\gguf-pilot-verify-024.json`;
const fs = await import('node:fs');
const net = await import('node:net');

async function target() {
  for (let i = 0; i < 90; i += 1) {
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
    ws.onmessage = (ev) => {
      const m = JSON.parse(ev.data);
      if (m.id && pending.has(m.id)) {
        const { res, rej } = pending.get(m.id);
        pending.delete(m.id);
        m.error ? rej(new Error(JSON.stringify(m.error))) : res(m.result);
      }
    };
    ws.onerror = reject;
    ws.onopen = () => resolve({
      send(method, params) {
        id += 1;
        const mid = id;
        return new Promise((res, rej) => { pending.set(mid, { res, rej }); ws.send(JSON.stringify({ id: mid, method, params })); });
      },
      close: () => ws.close(),
    });
  });
}
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const results = {};

const page = await target();
const cdp = await connect(page.webSocketDebuggerUrl);
await cdp.send('Runtime.enable');
const evaluate = async (expression, timeout = 60000) => {
  const r = await cdp.send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true, timeout });
  if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description || 'eval failed');
  return r.result.value;
};
const invoke = (cmd, args) => `window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(args || {})})`;
const clickNav = async (label) => {
  const ok = await evaluate(`(() => { const b=[...document.querySelectorAll('.nav-item')].find(x=>x.textContent.trim().toLowerCase().includes(${JSON.stringify(label)})); b && b.click(); return !!b; })()`);
  await sleep(1100);
  return ok;
};

// ---- 6. Free port when 8080 is busy -----------------------------------------
const blocker = net.createServer();
await new Promise((r) => blocker.listen(8080, '127.0.0.1', r));
results.portWith8080Busy = await evaluate(`${invoke('suggest_port', { host: '127.0.0.1', preferred: 8080 })}`);
await new Promise((r) => blocker.close(r));
results.portWith8080Free = await evaluate(`${invoke('suggest_port', { host: '127.0.0.1', preferred: 8080 })}`);

// ---- 5. DeepSeek companions --------------------------------------------------
results.deepseek = await evaluate(
  `${invoke('scan_models', { root: MODEL_ROOT })}.then(ms => { const m = ms.find(x => x.name.includes('UD-IQ4_XS')); return m && { name: m.name, quant: m.quant, count: m.companions.length, order: m.companions.map(c => c.name) }; })`,
);

// ---- 3. Runtime persistence --------------------------------------------------
await evaluate(`localStorage.setItem('gguf-pilot:runtime', ${JSON.stringify(RUNTIME)}); localStorage.setItem('gguf-pilot:model-root', ${JSON.stringify(MODEL_ROOT)}); true`);
// Save a profile pinned to a deliberately stale runtime, as 0.2.2 would have.
const modelId = await evaluate(`${invoke('scan_models', { root: MODEL_ROOT })}.then(ms => ms.find(x => x.name.includes('LFM2.5-2.6B'))?.id)`);
await evaluate(`localStorage.setItem('gguf-pilot:profile:' + ${JSON.stringify(modelId)}, JSON.stringify({name:'stale', runtime:'C:\\\\old\\\\llama-server.exe', context: 16384, port: 8080})); location.reload(); true`);
await sleep(7000);
await clickNav('inventory');
await sleep(2500);
await evaluate(`(() => { const r=[...document.querySelectorAll('.table-row')].find(x=>x.textContent.includes('LFM2.5-2.6B')); r && r.click(); return !!r; })()`);
await sleep(1500);
results.staleProfileAdoptsCurrentRuntime = await evaluate(`(() => { const inputs=[...document.querySelectorAll('.profile-screen input')]; const rt=inputs.find(i=>i.value && i.value.toLowerCase().includes('llama-server.exe')); const ctx=[...document.querySelectorAll('.profile-screen input')].map(i=>i.value); return { runtimeField: rt?.value, keptContext: ctx.includes('16384') }; })()`);
results.runtimeRemembered = await evaluate(`localStorage.getItem('gguf-pilot:runtime')`);

// ---- 4. About tab -------------------------------------------------------------
results.aboutNavPresent = await clickNav('about');
await sleep(1200);
results.about = await evaluate(`({
  heading: document.querySelector('.about-screen h1')?.textContent,
  version: [...document.querySelectorAll('.about-identity span')].map(s=>s.textContent)[0],
  rows: [...document.querySelectorAll('.about-screen .spec-list div, .about-screen .runtime-facts div')].map(d=>d.querySelector('dt')?.textContent + '=' + d.querySelector('dd')?.textContent).slice(0, 14),
  credits: [...document.querySelectorAll('.credit strong')].map(c=>c.textContent),
  overflow: [...document.querySelectorAll('.about-screen *')].filter(e=>e.getBoundingClientRect().right > innerWidth + 1).length
})`);
results.navLabels = await evaluate(`[...document.querySelectorAll('.nav-item')].map(n=>n.textContent.trim())`);

// ---- 5b. Picker + counted chip in the UI --------------------------------------
await clickNav('inventory');
await sleep(2000);
results.inventoryChip = await evaluate(`(() => { const r=[...document.querySelectorAll('.table-row')].find(x=>x.textContent.includes('UD-IQ4_XS')); return r && [...r.querySelectorAll('.companion-stack i')].map(i=>i.textContent); })()`);
await evaluate(`(() => { const r=[...document.querySelectorAll('.table-row')].find(x=>x.textContent.includes('UD-IQ4_XS')); r && r.click(); return !!r; })()`);
await sleep(1800);
results.draftPicker = await evaluate(`(() => { const sel=[...document.querySelectorAll('.profile-screen select')].find(s=>[...s.options].some(o=>o.textContent.includes('DSPARK'))); return sel && { options: sel.options.length, first: sel.options[1]?.textContent, selected: sel.selectedIndex }; })()`);

// Mobile nav check at 390x844
await cdp.send('Emulation.setDeviceMetricsOverride', { width: 390, height: 844, deviceScaleFactor: 2, mobile: true });
await sleep(900);
results.mobileNav = await evaluate(`({ items: [...document.querySelectorAll('.nav-item')].map(n=>{const r=n.getBoundingClientRect(); return Math.round(r.width)+'x'+Math.round(r.height);}), scrollW: document.documentElement.scrollWidth, overflow: [...document.querySelectorAll('*')].filter(e=>e.getBoundingClientRect().right>innerWidth+1).length })`);
await clickNav('about');
await sleep(900);
await cdp.send('Page.enable');
const shot = await cdp.send('Page.captureScreenshot', { format: 'png' });
fs.mkdirSync('.impeccable/review', { recursive: true });
fs.writeFileSync('.impeccable/review/v024-about-mobile.png', Buffer.from(shot.data, 'base64'));
await cdp.send('Emulation.clearDeviceMetricsOverride');
await sleep(700);
const shot2 = await cdp.send('Page.captureScreenshot', { format: 'png' });
fs.writeFileSync('.impeccable/review/v024-about-desktop.png', Buffer.from(shot2.data, 'base64'));

fs.writeFileSync(OUT, JSON.stringify(results, null, 2));
console.log(JSON.stringify(results, null, 2));
cdp.close();
process.exit(0);
