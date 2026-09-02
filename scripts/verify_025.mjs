// Packaged 0.2.5 verification: catalog IPC, filters, secure token UI, download
// input validation, desktop/mobile layout, and real release metadata.
const PORT = process.argv[2] || '10014';
const OUT = `${process.env.LOCALAPPDATA}\\Temp\\gguf-pilot-verify-025.json`;
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
const clickNav = async (label) => {
  const clicked = await evaluate(`(() => { const item=[...document.querySelectorAll('.nav-item')].find((node)=>node.textContent.toLowerCase().includes(${JSON.stringify(label.toLowerCase())})); item?.click(); return Boolean(item); })()`);
  await sleep(1500);
  return clicked;
};
const results = {};

results.catalog = await evaluate(`${invoke('fetch_model_catalog')}.then((snapshot)=>({origin:snapshot.origin, models:snapshot.catalog.models.length, files:snapshot.catalog.models.reduce((sum,model)=>sum+model.files.length,0), schema:snapshot.catalog.schemaVersion}))`);
results.filtered = await evaluate(`${invoke('fetch_model_catalog')}.then((snapshot)=>${invoke('filter_catalog', { models: '__MODELS__', query: { text: 'phi mini', tag: '', quant: 'Q4_K_M', maxBytes: 0, hideGated: false, sort: 'name' } })})`.replace('"__MODELS__"', 'snapshot.catalog.models'));
results.facets = await evaluate(`${invoke('fetch_model_catalog')}.then((snapshot)=>window.__TAURI_INTERNALS__.invoke('catalog_facets',{models:snapshot.catalog.models}))`);
results.token = await evaluate(`${invoke('hf_token_status')}`);
results.traversalRejected = await evaluate(`${invoke('download_catalog_file', { repo: 'owner/repo', filename: '../escape.gguf', revision: null, destination: 'C:\\models', connections: 4 })}.then(()=>false,()=>true)`);
results.relativeDestinationRejected = await evaluate(`${invoke('download_catalog_file', { repo: 'owner/repo', filename: 'model.gguf', revision: null, destination: 'relative', connections: 4 })}.then(()=>false,()=>true)`);

results.catalogNavPresent = await clickNav('hf catalog');
results.ui = await evaluate(`(() => ({
  heading: document.querySelector('.catalog-screen h1')?.textContent,
  rows: document.querySelectorAll('.catalog-model').length,
  resultText: document.querySelector('.catalog-result-count')?.textContent,
  tokenType: document.querySelector('.catalog-token-body input')?.getAttribute('type'),
  tokenLocation: document.querySelector('.catalog-sidebar')?.textContent.includes('Windows Credential Manager'),
  directCopy: document.querySelector('.catalog-screen')?.textContent.includes('directly from Hugging Face'),
  navLabels: [...document.querySelectorAll('.nav-item')].map((node)=>node.textContent.trim()),
  overflow: [...document.querySelectorAll('*')].filter((node)=>node.getBoundingClientRect().right>innerWidth+1).length
}))()`);
results.search = await evaluate(`(async()=>{const input=document.querySelector('.catalog-search input'); const setter=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set; setter.call(input,'phi mini'); input.dispatchEvent(new Event('input',{bubbles:true})); await new Promise(r=>setTimeout(r,1200)); return {rows:document.querySelectorAll('.catalog-model').length,names:[...document.querySelectorAll('.catalog-model-head>div>strong')].map(n=>n.textContent)};})()`);

const desktop = await cdp.send('Page.captureScreenshot', { format: 'png' });
fs.mkdirSync('.impeccable/review', { recursive: true });
fs.writeFileSync('.impeccable/review/v025-catalog-desktop.png', Buffer.from(desktop.data, 'base64'));

await cdp.send('Emulation.setDeviceMetricsOverride', { width: 390, height: 844, deviceScaleFactor: 2, mobile: true });
await sleep(900);
results.mobile = await evaluate(`(() => ({
  navItems:[...document.querySelectorAll('.nav-item')].map((node)=>{const box=node.getBoundingClientRect(); return {label:node.textContent.trim(),width:Math.round(box.width),height:Math.round(box.height)}}),
  scrollWidth:document.documentElement.scrollWidth,
  overflow:[...document.querySelectorAll('*')].filter((node)=>node.getBoundingClientRect().right>innerWidth+1).length,
  filterColumns:getComputedStyle(document.querySelector('.catalog-filters')).gridTemplateColumns
}))()`);
await evaluate(`window.scrollTo(0,document.documentElement.scrollHeight); true`);
await sleep(300);
results.mobileClearance = await evaluate(`(() => {const nav=document.querySelector('.rail nav').getBoundingClientRect(); const visible=[...document.querySelectorAll('.catalog-screen *')].map((node)=>node.getBoundingClientRect().bottom).filter((bottom)=>bottom>0&&bottom<nav.top+1); return {atBottom:Math.abs(document.documentElement.scrollHeight-window.scrollY-window.innerHeight)<3,navTop:Math.round(nav.top),lastContentBottom:Math.round(Math.max(...visible)),clearance:Math.round(nav.top-Math.max(...visible))};})()`);
const mobile = await cdp.send('Page.captureScreenshot', { format: 'png' });
fs.writeFileSync('.impeccable/review/v025-catalog-mobile.png', Buffer.from(mobile.data, 'base64'));

fs.writeFileSync(OUT, JSON.stringify(results, null, 2));
console.log(JSON.stringify(results, null, 2));
cdp.close();
