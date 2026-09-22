// Working-tree diagnostic. Reuses the packaged matrix's CDP transport.
// This record is not release qualification or installer lifecycle evidence.
import { spawn, execFileSync } from 'node:child_process';
import { mkdir, readFile, writeFile, symlink, unlink } from 'node:fs/promises';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { once } from 'node:events';
import { attach } from '../../scripts/lib/cdp_client.mjs';

const out = resolve(process.argv[2] || 'artifacts/ideas-audit-20260919-2202');
const exe = resolve('src-tauri/target/release/localmotive.exe');
const isolated = resolve(out, `native-${Date.now()}`);
const models = resolve(isolated, 'models');
await mkdir(models, { recursive: true });
await mkdir(resolve(isolated, 'appdata'), { recursive: true });
const gguf = Buffer.alloc(24);
gguf.write('GGUF'); gguf.writeUInt32LE(3, 4);
await writeFile(resolve(models, 'audit-Q4_K_M.gguf'), gguf);
const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');
const report = {
  scope: 'Working-tree UI diagnostic; not release qualification',
  startedAt: new Date().toISOString(), source: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(),
  sourceDirty: true, executable: exe, sha256: sha256(await readFile(exe)), checks: [],
};
const child = spawn(exe, [], {
  env: { ...process.env, LOCALAPPDATA: resolve(isolated, 'appdata'),
    LOCALMOTIVE_VERIFY_ISOLATED_ROOT: resolve(isolated, 'appdata'),
    WEBVIEW2_USER_DATA_FOLDER: resolve(isolated, 'webview'),
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: '--remote-debugging-port=10193' },
  stdio: 'ignore', windowsHide: true,
});
child.on('error', (error) => { report.spawnError = error.message; });
let client;
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const requireValue = (condition, message) => { if (!condition) throw new Error(message); };
async function wait(expression) {
  const deadline = Date.now() + 30000;
  do {
    const result = await client.evaluate(expression, 5000);
    if (result) return result;
    await pause(100);
  } while (Date.now() < deadline);
  throw new Error(`Timed out waiting for ${expression}`);
}
const nav = async (label) => {
  await client.evaluate(`(() => { const b = [...document.querySelectorAll('nav button, .nav button, aside button')].find(b => b.textContent.trim() === ${JSON.stringify(label)}); if (!b) throw new Error('Navigation missing'); b.click(); return true; })()`);
};
const setField = async (label, value, screen = '.profile-screen') => {
  await client.evaluate(`(() => {
    const input = [...document.querySelectorAll(${JSON.stringify(`${screen} label`)})].find(l => l.textContent.startsWith(${JSON.stringify(label)}))?.querySelector('input');
    if (!input) throw new Error('Numeric input missing');
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set.call(input, ${JSON.stringify(value)});
    input.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  })()`);
};
const click = async (selector, label) => {
  await client.evaluate(`(() => { const b = [...document.querySelectorAll(${JSON.stringify(selector)})].find(b => b.textContent.trim() === ${JSON.stringify(label)}); if (!b || b.getBoundingClientRect().width === 0) throw new Error('Visible action missing'); b.click(); return true; })()`);
};
const check = async (id, action) => {
  try { const evidence = await action(); report.checks.push({ id, status: 'PASS', evidence }); }
  catch (error) { report.checks.push({ id, status: 'FAIL', error: error.message }); throw error; }
};
try {
  client = await attach(10193, { deadlineMs: 60000 });
  await wait(`document.querySelector('.app-shell') !== null`);
  await client.evaluate(`(() => {
    localStorage.setItem('localmotive:model-root', ${JSON.stringify(models)});
    localStorage.setItem('localmotive:runtime', ${JSON.stringify(resolve(isolated, 'not-installed', 'llama-server.exe'))});
    location.reload(); return true;
  })()`);
  await wait(`document.querySelector('.profile-screen') !== null`);
  await check('uninspected-speculation', async () => {
    const facts = await client.evaluate(`(() => {
      const s = [...document.querySelectorAll('.profile-screen select')].find(s => s.closest('label')?.textContent.startsWith('Speculative method'));
      return { disabled: s.disabled, values: [...s.options].map(o => o.value), invalidChildren: s.querySelectorAll('small').length };
    })()`);
    requireValue(facts.disabled && facts.values.join(',') === 'none' && facts.invalidChildren === 0, JSON.stringify(facts));
    return facts;
  });
  await check('numeric-domains', async () => {
    const facts = await client.evaluate(`(() => {
      const inputs = [...document.querySelectorAll('.profile-screen input[type=number]')];
      return { count: inputs.length, unbounded: inputs.filter(i => i.min === '' || i.max === '' || !i.required).length };
    })()`);
    requireValue(facts.count === 41 && facts.unbounded === 0, JSON.stringify(facts));
    return facts;
  });
  await check('invalid-tuning-workload-refused-before-runtime', async () => {
    const request = { profile: { runtime: resolve(isolated, 'not-installed', 'llama-server.exe') }, provider: 'invalid-fixture', model: 'invalid-fixture', targetContext: 4096, maxTrials: 1, tokens: 4096, repeats: 99, companions: [] };
    const message = await client.evaluate(`window.__TAURI_INTERNALS__.invoke('start_tuning', { request: ${JSON.stringify(request)} }).then(() => 'unexpected success', String)`);
    requireValue(message === 'Tuning tokens must be between 64 and 2048', message);
    return { message };
  });
  await check('raw-file-flags-require-profile-fields', async () => {
    const results = await client.evaluate(`(async () => {
      const results = [];
      for (const flag of ['--lora', '--lora-scaled', '--mmproj']) {
        try {
          await window.__TAURI_INTERNALS__.invoke('preview_command', { profile: { alias: 'fixture', extraArgs: [flag + '=C:/not-selected/file.gguf'] } });
          results.push({ flag, accepted: true });
        } catch (error) { results.push({ flag, message: String(error) }); }
      }
      return results;
    })()`);
    requireValue(results.length === 3 && results.every(r => !r.accepted && r.message?.includes('managed profile flag')), `Raw file flags escaped their profile controls: ${JSON.stringify(results)}`);
    return { rejectedFlags: results.map(r => r.flag) };
  });
  await check('invalid-benchmark-workload-refused-before-server', async () => {
    const message = await client.evaluate(`window.__TAURI_INTERNALS__.invoke('benchmark_server', { tokens: 4294967295, repeats: 1 }).then(() => 'unexpected success', String)`);
    requireValue(message === 'Tokens must be between 64 and 4096', message);
    return { message };
  });
  const junction = resolve(isolated, 'linked-models');
  await symlink(models, junction, 'junction');
  const linkedModel = resolve(junction, 'audit-Q4_K_M.gguf');
  for (const [command, args] of [
    ['read_gguf_summary', { path: linkedModel }],
    ['inspect_model_artifact', { firstShard: linkedModel, companions: [], hashFiles: false }],
  ]) {
    await check(`${command}-rejects-junction`, async () => {
      const message = await client.evaluate(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(command)}, ${JSON.stringify(args)}).then(() => 'unexpected success', String)`);
      requireValue(message.includes('reparse-point ancestor'), message);
      return { refusedBeforeReading: true };
    });
  }
  await unlink(junction);
  await setField('Port', '65536');
  await check('invalid-save-refused', async () => {
    await click('.profile-screen button', 'Save');
    await wait(`document.body.textContent.includes('Port must be between 1 and 65535.')`);
    const count = await client.evaluate(`Object.keys(localStorage).filter(k => k.startsWith('localmotive:profile:')).length`);
    requireValue(count === 0, 'Invalid profile was saved');
    return { persistedProfiles: count };
  });
  await check('invalid-control-start-refused', async () => {
    await nav('Control');
    await wait(`document.querySelector('.control-screen, .dashboard-screen') !== null || document.body.textContent.includes('Start profile')`);
    await click('main button', 'Start profile');
    await wait(`document.body.textContent.includes('Port must be between 1 and 65535.')`);
    const status = await client.evaluate(`window.__TAURI_INTERNALS__.invoke('server_status')`);
    requireValue(status.running === false && status.pid === null, 'Invalid input started a process');
    return { phase: status.phase, pid: status.pid };
  });
  await nav('Profile');
  await wait(`document.querySelector('.profile-screen') !== null`);
  await setField('Port', '8080');
  await setField('Context tokens', '');
  await check('blank-number-not-zero', async () => {
    const value = await client.evaluate(`[...document.querySelectorAll('.profile-screen label')].find(l => l.textContent.startsWith('Context tokens')).querySelector('input').value`);
    requireValue(value === '', `Cleared context became ${value}`);
    await click('.profile-screen button', 'Save');
    await wait(`document.body.textContent.includes('Context tokens must be a whole number.')`);
    return { value };
  });
  await setField('Context tokens', '4096');
  await check('corrected-profile-save', async () => {
    await click('.profile-screen button', 'Save');
    const profile = await wait(`(() => { const k = Object.keys(localStorage).find(k => k.startsWith('localmotive:profile:')); return k ? JSON.parse(localStorage.getItem(k)) : null; })()`);
    requireValue(profile.context === 4096 && profile.port === 8080, 'Corrected values did not persist');
    return { context: profile.context, port: profile.port };
  });
  await nav('AI Tune');
  await wait(`document.querySelector('.tune-screen') !== null`);
  // Seed metadata-only readiness in this isolated UI. Do not replace Tauri's
  // immutable IPC bridge or dispatch a valid tuning request.
  report.tuneBoundary = await client.evaluate(`(() => {
    const screen = document.querySelector('.tune-screen');
    let fiber = screen[Object.keys(screen).find(k => k.startsWith('__reactFiber$'))];
    while (fiber && typeof fiber.memoizedProps?.chooseCloudModel !== 'function') fiber = fiber.return;
    if (!fiber) throw new Error('Tune props not found');
    const props = fiber.memoizedProps;
    const matches = [];
    for (let parent = fiber.return; parent; parent = parent.return) {
      for (let hook = parent.memoizedState; hook; hook = hook.next) {
        const value = hook.memoizedState;
        if (value !== null && typeof value === 'object' && typeof value.provider === 'string'
            && typeof value.configured === 'boolean' && 'masked' in value && hook.queue?.dispatch) {
          matches.push(hook);
        }
      }
    }
    if (matches.length !== 1) throw new Error('Expected one credential-status hook, found ' + matches.length);
    matches[0].queue.dispatch({ ...matches[0].memoizedState, configured: true, masked: 'fixture' });
    props.chooseCloudModel('fixture-advisor');
    return { credentialMetadataHooks: matches.length, boundary: 'UI readiness metadata only; no stored credential or IPC replacement' };
  })()`);
  await wait(`(() => { const b = document.querySelector('.tune-screen .section-heading .actions button'); return b?.textContent.includes('Auto-tune') && !b.disabled; })()`);
  for (const [label, invalid, corrected, field] of [
      ['AI trials', '13', '12', 'maxTrials'],
      ['Tokens per measurement', '2049', '257', 'tokens'],
      ['Repeats per trial', '6', '5', 'repeats'],
    ]) {
      await check(`tune-input-${field}`, async () => {
        await setField(label, '', '.tune-screen');
        const blank = await client.evaluate(`(() => {
          const input = [...document.querySelectorAll('.tune-screen label')].find(l => l.textContent.startsWith(${JSON.stringify(label)})).querySelector('input');
          const button = document.querySelector('.tune-screen .section-heading .actions button');
          return { value: input.value, valid: input.checkValidity(), disabled: button.disabled, error: document.querySelector('.tune-screen [role="status"]')?.textContent };
        })()`);
        requireValue(blank.value === '' && !blank.valid && blank.disabled && blank.error?.includes(label), JSON.stringify(blank));
        await setField(label, invalid, '.tune-screen');
        const rejected = await client.evaluate(`(() => {
          const button = document.querySelector('.tune-screen .section-heading .actions button');
          return { disabled: button.disabled };
        })()`);
        requireValue(rejected.disabled, JSON.stringify(rejected));
        await setField(label, corrected, '.tune-screen');
        const recovery = await client.evaluate(`(() => {
          const input = [...document.querySelectorAll('.tune-screen label')].find(l => l.textContent.startsWith(${JSON.stringify(label)})).querySelector('input');
          const button = document.querySelector('.tune-screen .section-heading .actions button');
          return { value: input.value, valid: input.checkValidity(), enabled: !button.disabled, error: document.querySelector('.tune-screen [role="status"]')?.textContent ?? null };
        })()`);
        requireValue(recovery.value === corrected && recovery.valid && recovery.enabled && recovery.error === null, JSON.stringify(recovery));
        return { label, blank, rejected, recovery, boundary: 'real packaged controls; metadata-only readiness fixture; valid Tune action not clicked' };
      });
    }
  await nav('Benchmark');
  await wait(`document.querySelector('input[aria-label="Benchmark prompt tokens"]')?.getBoundingClientRect().width > 0`);
  for (const [version, label, field, invalid, corrected] of [
    ['v2', 'Prompt tokens', 'promptTokens', '1048577', '1'],
    ['v2', 'Generated tokens', 'generationTokens', '65537', '257'],
    ['v2', 'Warmups', 'warmups', '0.5', '0'],
    ['v2', 'Trials', 'trials', '101', '5'],
    ['legacy', 'Forced output tokens', 'tokens', '4097', '65'],
    ['legacy', 'Measured repeats', 'repeats', '11', '10'],
  ]) {
    await check(`benchmark-${version}-input-${field}`, async () => {
      const screen = version === 'v2' ? '[aria-labelledby="benchmark-v2-title"]' : '.benchmark-setup';
      const errorSelector = version === 'v2' ? '[aria-label="Workload validation errors"]' : '[role="status"]';
      const errorLabel = version === 'v2' ? `workload.${field}` : label;
      const facts = () => client.evaluate(`(() => {
        const section = document.querySelector(${JSON.stringify(screen)});
        const input = [...section.querySelectorAll('label')].find(l => l.textContent.startsWith(${JSON.stringify(label)})).querySelector('input');
        return { value: input.value, valid: input.checkValidity(), error: section.querySelector(${JSON.stringify(errorSelector)})?.textContent ?? null };
      })()`);
      await setField(label, '', screen);
      const blank = await facts();
      requireValue(blank.value === '' && !blank.valid && blank.error?.includes(errorLabel), JSON.stringify(blank));
      await setField(label, invalid, screen);
      const rejected = await facts();
      requireValue(rejected.value === invalid && !rejected.valid && rejected.error?.includes(errorLabel), JSON.stringify(rejected));
      await setField(label, corrected, screen);
      const recovery = await facts();
      requireValue(recovery.value === corrected && recovery.valid && recovery.error === null, JSON.stringify(recovery));
      return { label, blank, rejected, recovery, boundary: 'real packaged editing and validation; server idle; no benchmark action clicked' };
    });
  }
  await check('legacy-cancel-handle-survives-navigation-and-panel-publish', async () => {
    const setup = await client.evaluate(`(() => {
      const section = document.querySelector('.benchmark-setup');
      let fiber = section[Object.keys(section).find(k => k.startsWith('__reactFiber$'))];
      while (fiber && typeof fiber.memoizedProps?.runBenchmark !== 'function') fiber = fiber.return;
      if (!fiber) throw new Error('Benchmark props not found');
      const props = fiber.memoizedProps;
      const matches = [];
      for (let parent = fiber.return; parent; parent = parent.return) {
        let previous = null;
        for (let hook = parent.memoizedState; hook; hook = hook.next) {
          const value = hook.memoizedState;
          if (value !== null && typeof value === 'object' && value.current === false
              && Object.keys(value).length === 1 && !hook.queue
              && previous?.memoizedState === null && typeof previous.queue?.dispatch === 'function') {
            matches.push(previous);
          }
          previous = hook;
        }
      }
      if (matches.length !== 1) throw new Error('Expected one legacy owner beside its active guard, found ' + matches.length);
      const publish = matches[0].queue.dispatch;
      window.__legacyOwnershipFixture = {
        cancels: 0, clear: () => publish(null), clearPanel: () => props.setEvidenceRun(null),
      };
      publish({ kind: 'benchmark', cancel: () => { window.__legacyOwnershipFixture.cancels += 1; } });
      return { ownerHooks: matches.length, boundary: 'UI handle fixture only; no benchmark request or IPC replacement' };
    })()`);
    await wait(`document.querySelector('.evidence-run-band')?.textContent.includes('Benchmark running')`);
    await nav('Inventory');
    const before = await client.evaluate(`(() => { const band = document.querySelector('.evidence-run-band'); return { visible: band?.getBoundingClientRect().width > 0, text: band?.textContent }; })()`);
    requireValue(before.visible && before.text.includes('Cancel'), JSON.stringify(before));
    await client.evaluate(`window.__legacyOwnershipFixture.clearPanel()`);
    await client.evaluate(`new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))`);
    const retained = await client.evaluate(`document.querySelector('.evidence-run-band')?.textContent.includes('Benchmark running')`);
    requireValue(retained, 'Panel publication erased the legacy cancellation handle');
    await click('.evidence-run-band button', 'Cancel');
    const cancels = await wait(`window.__legacyOwnershipFixture.cancels`);
    requireValue(cancels === 1, 'The visible control did not call its owner exactly once');
    await client.evaluate(`window.__legacyOwnershipFixture.clear()`);
    await wait(`document.querySelector('.evidence-run-band') === null`);
    return { setup, visibleAfterNavigation: before.visible, retainedAfterPanelPublish: retained, cancelCallbacks: cancels };
  });
  await check('idle-cancel-command-refuses-without-work', async () => {
    const message = await client.evaluate(`window.__TAURI_INTERNALS__.invoke('cancel_benchmark').then(() => 'unexpected success', String)`);
    requireValue(message === 'No benchmark is running', message);
    return { message, boundary: 'real IPC; no active Rust benchmark' };
  });
  await check('idle-stop-command-remains-repeatable', async () => {
    const results = await client.evaluate(`(async () => {
      const results = [];
      for (let i = 0; i < 2; i += 1) {
        const status = await window.__TAURI_INTERNALS__.invoke('stop_server');
        results.push({ running: status.running, pid: status.pid, phase: status.phase });
      }
      return results;
    })()`);
    requireValue(results.length === 2 && results.every(status => status.running === false && status.pid === null), JSON.stringify(results));
    return { results, boundary: 'real packaged Stop IPC with an idle server; no model started' };
  });
  await nav('About');
  await wait(`document.querySelector('.about-screen') !== null`);
  await check('manual-update-disclosure', async () => {
    const facts = await client.evaluate(`(() => ({ button: [...document.querySelectorAll('.about-screen button')].some(b => b.textContent.includes('Check releases')), manual: document.querySelector('.about-screen').textContent.includes('Updates are manual') }))()`);
    requireValue(facts.button && facts.manual, 'Manual update controls or disclosure missing');
    return facts;
  });
  await check('about-mobile-layout', async () => {
    await client.send('Emulation.setDeviceMetricsOverride', { width: 360, height: 800, deviceScaleFactor: 1, mobile: false });
    await pause(200);
    const facts = await client.evaluate(`(() => {
      const buttons = [...document.querySelectorAll('.about-screen button')].filter(b => b.getBoundingClientRect().width > 0);
      return { width: innerWidth, scrollWidth: document.documentElement.scrollWidth, undersized: buttons.filter(b => b.getBoundingClientRect().height < 44).length, buttons: buttons.length };
    })()`);
    requireValue(facts.scrollWidth <= facts.width && facts.buttons > 0 && facts.undersized === 0, JSON.stringify(facts));
    await client.send('Emulation.clearDeviceMetricsOverride');
    return facts;
  });
  await check('uncaught-ui-errors', async () => {
    const errors = client.exceptions.filter(e => !e.text.includes('devtools'));
    requireValue(errors.length === 0, JSON.stringify(errors));
    return { count: errors.length };
  });
} catch (error) {
  report.error = error.message;
} finally {
  client?.close();
  if (child.exitCode === null && child.signalCode === null) {
    const exited = once(child, 'exit');
    child.kill();
    await Promise.race([exited, pause(10000)]);
  }
  report.cleanup = child.exitCode !== null || child.signalCode !== null ? 'PASS' : 'FAIL';
  report.finishedAt = new Date().toISOString();
  report.status = !report.error && report.checks.length > 0 && report.cleanup === 'PASS' ? 'PASS' : 'FAIL';
  await writeFile(resolve(out, 'native-ui.json'), JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify(report, null, 2));
  process.exit(report.status === 'PASS' ? 0 : 1);
}
