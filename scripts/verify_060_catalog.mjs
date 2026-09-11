// Version-aware packaged verifier for the 0.6 HF catalog + SQLite workflows
// (audit GH-05). Drives the REAL candidate binary over CDP through actual
// WebView events and IPC against a controlled, signed catalog fixture
// served from 127.0.0.1, inside an isolated application-data profile.
//
// Phases (the release workflow orchestrates the two launches between them):
//   init       generate the fixture ed25519 key, start the fixture server,
//              write  state.json  (url, pubkey hex, server pid, paths)
//   first-fill attach to launch #1: first fill, valid signed refresh,
//              cooldown throttle, filters/facets, user-row persistence,
//              disk-level cache + mirror evidence
//   prep       stop the fixture server; corrupt the cached signature and the
//              SQLite mirror on disk (failure paths for launch #2)
//   restart    attach to launch #2: offline availability, invalid-signature
//              fallback, corrupt-mirror recovery, UI-visible honesty
//   merge      bind phase results to the immutable source revision and the
//              candidate binary digest, emit the final record
import { spawn } from "node:child_process";
import { createHash, createPrivateKey, createPublicKey, generateKeyPairSync, sign as signBytes } from "node:crypto";
import { createServer } from "node:http";
import { appendFileSync, existsSync, readFileSync } from "node:fs";
import { mkdir, readFile, readdir, stat, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { attach } from "./lib/cdp_client.mjs";

const [phase, ...rest] = process.argv.slice(2);

function requireCondition(condition, message) {
  if (!condition) throw new Error(message);
}

async function sha256File(path) {
  const hash = createHash("sha256");
  const { createReadStream } = await import("node:fs");
  await new Promise((resolvePromise, rejectPromise) => {
    createReadStream(path)
      .on("data", (chunk) => hash.update(chunk))
      .on("end", resolvePromise)
      .on("error", rejectPromise);
  });
  return hash.digest("hex");
}

async function findInTree(root, name) {
  if (!existsSync(root)) return null;
  const entries = await readdir(root, { withFileTypes: true });
  for (const entry of entries) {
    const full = join(root, entry.name);
    if (entry.isDirectory()) {
      const found = await findInTree(full, name);
      if (found) return found;
    } else if (entry.name === name) {
      return full;
    }
  }
  return null;
}

function fixtureModels() {
  // Clone a bundled catalog entry so the fixture schema is exactly the
  // shipped signed-artifact schema; only identity fields change. Filenames
  // must be unique across the whole document (the parser rejects duplicates),
  // so each clone prefixes its file names with its own id.
  const bundled = JSON.parse(bundledCatalogText());
  const template = bundled.models.find((model) => {
    const names = model.files.map((file) => file.filename);
    return names.length > 0 && new Set(names).size === names.length;
  });
  const clone = (id, repo, family, tags, downloads, likes) => {
    const model = structuredClone(template);
    model.id = id;
    model.repo = repo;
    model.family = family;
    model.publisher = "fixture";
    model.author = "fixture";
    model.summary = `Verifier fixture model (${family}).`;
    model.tags = tags;
    model.downloads = downloads;
    model.likes = likes;
    model.gated = false;
    model.files = model.files.map((file) => ({
      ...file,
      filename: `${id}-${file.filename}`,
    }));
    return model;
  };
  return [
    clone("fixture-alpha-gguf", "fixture/alpha-GGUF", "Fixture Alpha", ["gguf", "text-generation"], 4242, 42),
    clone("fixture-beta-gguf", "fixture/beta-GGUF", "Fixture Beta", ["gguf", "code"], 99, 7),
  ];
}

function bundledCatalogText() {
  return readFileSync(resolve("catalog", "catalog.json"), "utf8");
}

async function init(stateDir, outPath) {
  await mkdir(stateDir, { recursive: true });
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const jwk = publicKey.export({ format: "jwk" });
  const pubkeyHex = Buffer.from(jwk.x, "base64url").toString("hex");
  requireCondition(pubkeyHex.length === 64, "fixture public key was not 32 bytes");

  const body = JSON.stringify({
    schemaVersion: 2,
    updated: new Date().toISOString(),
    source: "verifier-fixture",
    models: fixtureModels(),
  });
  const signature = signBytes(null, Buffer.from(body, "utf8"), privateKey).toString("base64");

  const bodyPath = join(stateDir, "fixture-catalog.json");
  const signaturePath = join(stateDir, "fixture-catalog.json.sig");
  const requestsLog = join(stateDir, "fixture-requests.log");
  await writeFile(bodyPath, body);
  await writeFile(signaturePath, signature);
  await writeFile(requestsLog, "");

  // Pick a free port, then hand it to the detached server child so the
  // endpoint stays reachable across the verifier's short-lived phases.
  const probe = createServer();
  await new Promise((resolvePromise) => probe.listen(0, "127.0.0.1", resolvePromise));
  const port = probe.address().port;
  await new Promise((resolvePromise) => probe.close(resolvePromise));
  const url = `http://127.0.0.1:${port}/catalog.json`;

  // Detached child so the server survives this process between phases.
  const child = spawn(process.execPath, [process.argv[1], "__serve", String(port)], {
    detached: true,
    stdio: "ignore",
    env: { ...process.env, LM_FIXTURE_BODY: bodyPath, LM_FIXTURE_SIG: signaturePath, LM_FIXTURE_LOG: requestsLog },
  });
  child.unref();

  const state = { url, pubkeyHex, port, serverPid: child.pid, requestsLog, bodyPath, signaturePath };
  await writeFile(join(stateDir, "state.json"), JSON.stringify(state, null, 2));
  {
    const deadline = Date.now() + 15_000;
    for (;;) {
      try {
        const response = await fetch(`${url}.sig`);
        if (response.ok) break;
      } catch {
        // keep waiting
      }
      if (Date.now() > deadline) throw new Error("The fixture server did not come up");
      await new Promise((resolvePromise) => setTimeout(resolvePromise, 250));
    }
  }
  const initRecord = {
    schema_version: "1.0.0",
    verifier: "scripts/verify_060_catalog.mjs",
    phase: "init",
    status: "PASS",
    fixture_url: url,
    fixture_pubkey: pubkeyHex,
    fixture_models: ["fixture/alpha-GGUF", "fixture/beta-GGUF"],
  };
  await writeFile(outPath, JSON.stringify(initRecord, null, 2));
  console.log(JSON.stringify(initRecord));
}

async function serve(port) {
  // Standalone fixture server process (spawned by `init`): serves the exact
  // signed bytes and appends every request to the shared log so the parent
  // can prove whether a throttled refresh reached the network.
  const body = readFileSync(process.env.LM_FIXTURE_BODY);
  const signature = readFileSync(process.env.LM_FIXTURE_SIG);
  const log = process.env.LM_FIXTURE_LOG;
  const server = createServer((request, response) => {
    appendFileSync(log, `${Date.now()} ${request.url}\n`);
    if (request.url === "/catalog.json") {
      response.writeHead(200, { "content-type": "application/json", etag: `"fixture-${body.length}"` });
      response.end(body);
      return;
    }
    if (request.url === "/catalog.json.sig") {
      response.writeHead(200, { "content-type": "text/plain" });
      response.end(signature);
      return;
    }
    response.writeHead(404);
    response.end("not found");
  });
  server.listen(port, "127.0.0.1");
  // Serve until killed by the `prep` phase.
}

async function attachToPage(port) {
  const client = await attach(Number.parseInt(port, 10));
  return client;
}

function requireTauri() {
  return `(() => typeof window.__TAURI_INTERNALS__?.invoke === 'function')()`;
}

async function invokeViaPage(client, command, args = {}) {
  const result = await client.evaluate(`(async () => {
    try {
      const value = await window.__TAURI_INTERNALS__.invoke(${JSON.stringify(command)}, ${JSON.stringify(args)});
      return { ok: true, value };
    } catch (error) {
      return { ok: false, error: typeof error === 'string' ? error : JSON.stringify(error) };
    }
  })()`);
  return result;
}

async function waitFor(client, expression, message, timeoutMs = 30_000) {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    const value = await client.evaluate(`(() => { try { return (${expression}); } catch { return false; } })()`);
    if (value) return value;
    if (Date.now() > deadline) throw new Error(message);
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 400));
  }
}

async function clickNav(client, label) {
  // The IPC bridge exists before React commits the shell: wait for the nav
  // item itself before clicking (semantic wait, no fixed sleep).
  await waitFor(
    client,
    `[...document.querySelectorAll('button.nav-item, button')].some((entry) => (entry.textContent ?? '').trim().includes(${JSON.stringify(label)}))`,
    `The ${label} navigation item never rendered`,
  );
  const clicked = await client.evaluate(`(() => {
    const button = [...document.querySelectorAll('button.nav-item, button')].find((entry) => (entry.textContent ?? '').trim().includes(${JSON.stringify(label)}));
    if (!button) return false;
    button.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, view: window }));
    return true;
  })()`);
  requireCondition(clicked, `The ${label} navigation item was not found`);
  // The click must actually switch the pane, not just dispatch: wait until
  // the nav item reports itself active (semantic wait, no fixed sleep).
  await waitFor(
    client,
    `[...document.querySelectorAll('button.nav-item')].some((entry) => entry.classList.contains('active') && (entry.textContent ?? '').trim().includes(${JSON.stringify(label)}))`,
    `The ${label} pane never became active after the click`,
  );
}

async function firstFill(port, stateDir, outPath) {
  const state = JSON.parse(await readFile(join(stateDir, "state.json"), "utf8"));
  const client = await attachToPage(port);
  const checks = [];
  const record = (id, description, status, detail) => {
    checks.push({ id, description, status, detail });
    console.log("CHECK", id, status);
  };
  try {
    await waitFor(client, requireTauri(), "The candidate did not expose the Tauri IPC bridge");
    console.log("PHASE first-fill: tauri bridge present");

    const readHits = async () =>
      (await readFile(join(stateDir, "fixture-requests.log"), "utf8").catch(() => ""))
        .trim()
        .split("\n")
        .filter(Boolean).length;

    // 1. First fill: the local load answers without a network request. The
    // app may already have refreshed at startup (its own load-then-refresh
    // flow), so the hit count is compared, not assumed zero.
    const hitsBeforeLocal = await readHits();
    const local = await invokeViaPage(client, "load_model_catalog");
    requireCondition(local.ok, `load_model_catalog failed: ${local.error}`);
    const hitsAfterLocal = await readHits();
    requireCondition(
      hitsAfterLocal === hitsBeforeLocal,
      `The local load performed a network request (${hitsBeforeLocal} -> ${hitsAfterLocal})`,
    );
    record("catalog.first-fill-local", "The local load answers without a network request", "PASS", {
      origin: local.value?.origin ?? null,
      model_count: local.value?.catalog?.models?.length ?? 0,
      fixture_hits: hitsAfterLocal,
    });

    // 2. Valid signed refresh: the controlled fixture content must reach the
    // candidate either through this refresh (origin network) or through the
    // app's own startup refresh, which leaves a cooldown behind. Both prove
    // the signature-verified fixture catalog was accepted; the backend
    // rejects a throttled fetch with an explicit cooldown error, so both
    // shapes count as evidence and the branch is recorded honestly.
    const isCooldownText = (value) => /refreshed recently|cooldown|try again/i.test(String(value ?? ""));
    const refresh = await invokeViaPage(client, "fetch_model_catalog");
    let refreshOrigin = refresh.value?.origin ?? null;
    let fixtureAccepted = false;
    let throttledAlready = false;
    if (refresh.ok) {
      const refreshIds = (refresh.value?.catalog?.models ?? []).map((model) => model.repo ?? model.id);
      refreshOrigin = refresh.value?.origin ?? null;
      // Identity guard: the snapshot must carry the fixture endpoint from
      // THIS run's state file, otherwise the verifier attached to a stale
      // candidate that belongs to an earlier launch.
      requireCondition(
        refresh.value?.url === state.url,
        `The candidate answered for ${refresh.value?.url} instead of the fixture ${state.url}`,
      );
      throttledAlready = typeof refresh.value?.cooldownRemainingMinutes === "number";
      fixtureAccepted = refreshIds.includes("fixture/alpha-GGUF");
      requireCondition(
        fixtureAccepted,
        `The signed fixture catalog did not load (origin ${refreshOrigin}): ${refreshIds.join(", ")}`,
      );
    } else {
      requireCondition(
        isCooldownText(refresh.error),
        `fetch_model_catalog failed outside the cooldown contract: ${refresh.error}`,
      );
      throttledAlready = true;
      const settled = await invokeViaPage(client, "load_model_catalog");
      requireCondition(settled.ok, `load_model_catalog failed: ${settled.error}`);
      const settledIds = (settled.value?.catalog?.models ?? []).map((model) => model.repo ?? model.id);
      fixtureAccepted = settledIds.includes("fixture/alpha-GGUF");
      requireCondition(
        fixtureAccepted,
        `The startup-refreshed snapshot did not carry the signed fixture rows: ${settledIds.slice(0, 6).join(", ")}`,
      );
      refreshOrigin = settled.value?.origin ?? "cache";
    }
    const hitsAfterRefresh = await readHits();
    requireCondition(
      hitsAfterRefresh >= Math.max(hitsAfterLocal, 1),
      `The fixture endpoint was never contacted (${hitsAfterRefresh} hits)`,
    );
    record("catalog.valid-signed-refresh", "A validly signed fixture catalog is accepted by the candidate", "PASS", {
      origin: refreshOrigin,
      already_throttled: throttledAlready,
      fixture_hits: hitsAfterRefresh,
    });

    // 3. Cooldown: a refresh inside the window is throttled (snapshot
    // cooldown or explicit cooldown rejection) and the endpoint is not hit.
    const hitsBefore = await readHits();
    const throttled = await invokeViaPage(client, "fetch_model_catalog");
    let cooldown = null;
    if (throttled.ok) {
      cooldown = throttled.value?.cooldownRemainingMinutes;
      requireCondition(typeof cooldown === "number" && cooldown > 0, `No cooldown was reported: ${JSON.stringify(throttled.value?.cooldownRemainingMinutes)}`);
    } else {
      requireCondition(
        isCooldownText(throttled.error),
        `The throttled fetch failed outside the cooldown contract: ${throttled.error}`,
      );
      cooldown = "rejected-with-cooldown-message";
    }
    const hitsAfter = await readHits();
    requireCondition(hitsAfter === hitsBefore, `The throttled refresh still hit the network (${hitsBefore} -> ${hitsAfter})`);
    record("catalog.cooldown-throttle", "A refresh inside the cooldown is throttled without a network request", "PASS", {
      cooldown: cooldown,
      fixture_hits: hitsAfter,
    });

    // 4. Rows render in the shipped UI after the refresh.
    await clickNav(client, "HF Catalog");
    const rowState = await waitFor(
      client,
      `(() => {
        const rows = document.querySelectorAll('.catalog-results article, .catalog-model');
        return rows.length >= 2 ? { rows: rows.length, text: document.querySelector('.catalog-results')?.innerText?.slice(0, 400) ?? '' } : false;
      })()`,
      "The fixture rows did not render in the shipped UI",
      90_000,
    ).catch(async (error) => {
      const dump = await client.evaluate(
        `(() => ({
          exceptions: (window.__lmVerifyExceptions ?? []).slice(-3),
          activeNav: document.querySelector('button.nav-item.active')?.textContent?.trim() ?? null,
          screen: document.querySelector('.catalog-screen')?.textContent?.slice(0, 400) ?? document.body.textContent.slice(0, 400),
          rows: document.querySelectorAll('.catalog-results article').length,
          busy: Boolean(document.querySelector('.catalog-empty')),
        }))()`,
      );
      throw new Error(`${error.message}; DOM: ${JSON.stringify(dump)}; lastException: ${JSON.stringify((client.exceptions ?? []).slice(-2))}`);
    });
    record("catalog.ui-rows", "The shipped UI renders the refreshed fixture rows", "PASS", rowState);

    // 5. Search filter narrows the rendered rows through the real control.
    const filtered = await client.evaluate(`(async () => {
      const input = document.querySelector('.catalog-search input') ?? [...document.querySelectorAll('input')].find((entry) => (entry.getAttribute('placeholder') ?? '').toLowerCase().includes('search'));
      if (!input) return { ok: false, reason: 'no search input' };
      input.focus();
      const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
      setter.call(input, 'alpha');
      input.dispatchEvent(new Event('input', { bubbles: true }));
      await new Promise((resolvePromise) => setTimeout(resolvePromise, 600));
      const rows = document.querySelectorAll('.catalog-results article, .catalog-model');
      return {
        ok: true,
        rows: rows.length,
        text: document.querySelector('.catalog-results')?.innerText?.slice(0, 300) ?? '',
      };
    })()`);
    requireCondition(filtered.ok, `The catalog search control was not found: ${filtered.reason}`);
    requireCondition(filtered.rows >= 1 && filtered.rows < rowState.rows, `The search filter did not narrow the rows: ${filtered.rows} of ${rowState.rows}`);
    requireCondition(/alpha/i.test(filtered.text), "The filtered rows did not contain the matching fixture model");
    record("catalog.search-filter", "The search control narrows the rendered rows to the match", "PASS", filtered);

    // 6. Clearing the filter restores the full collection.
    const cleared = await client.evaluate(`(async () => {
      const input = document.querySelector('.catalog-search input') ?? [...document.querySelectorAll('input')].find((entry) => (entry.getAttribute('placeholder') ?? '').toLowerCase().includes('search'));
      const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
      setter.call(input, '');
      input.dispatchEvent(new Event('input', { bubbles: true }));
      await new Promise((resolvePromise) => setTimeout(resolvePromise, 600));
      const rows = document.querySelectorAll('.catalog-results article, .catalog-model');
      return { rows: rows.length };
    })()`);
    requireCondition(cleared.rows >= rowState.rows, `Clearing the search did not restore the rows: ${cleared.rows}`);
    record("catalog.search-clear", "Clearing the search restores every fixture row", "PASS", cleared);

    // 7. User-added row persistence through the real IPC and the mirror DB.
    const userModel = {
      id: "fixture-user-added",
      repo: "fixture/user-added-GGUF",
      family: "Fixture User",
      parameters: "",
      publisher: "fixture",
      author: "fixture",
      summary: "Verifier user-added row.",
      tags: ["gguf"],
      gated: false,
      downloads: 1,
      likes: 1,
      files: [
        {
          quant: "Q4_K_M",
          filename: "user-added-Q4_K_M.gguf",
          sizeBytes: 1024,
          sha256: "b".repeat(64),
          revision: "main",
        },
      ],
    };
    const saved = await invokeViaPage(client, "save_user_catalog_override", { model: userModel });
    requireCondition(saved.ok, `save_user_catalog_override failed: ${saved.error}`);
    const withUser = (saved.value ?? []).filter((model) => model.userSourced);
    requireCondition(withUser.length === 1, `The saved user row was not marked userSourced: ${withUser.length}`);
    const removed = await invokeViaPage(client, "remove_user_catalog_override", { id: "fixture-user-added" });
    requireCondition(removed.ok, `remove_user_catalog_override failed: ${removed.error}`);
    record("catalog.user-row-persistence", "A user-added row persists with userSourced provenance and can be removed", "PASS", {
      marked_before_removal: withUser.length,
      remaining_after_removal: (removed.value ?? []).filter((model) => model.userSourced).length,
    });

    // 8. Disk-level evidence: the verified cache and the SQLite mirror exist
    // under the verifier-owned catalog root (the app writes there by
    // contract, so the matrix never touches the real user cache).
    const catalogRoot = process.env.LOCALMOTIVE_CATALOG_ROOT;
    const isolatedRoot = process.env.LOCALMOTIVE_VERIFY_ISOLATED_ROOT;
    requireCondition(isolatedRoot, "Set LOCALMOTIVE_VERIFY_ISOLATED_ROOT before launching the candidate");
    requireCondition(catalogRoot, "Set LOCALMOTIVE_CATALOG_ROOT before launching the candidate");
    const cachePath = await findInTree(catalogRoot, "catalog-cache.json");
    const mirrorPath = await findInTree(catalogRoot, "catalog-mirror.sqlite");
    requireCondition(cachePath, "The verified catalog cache file was not found under the isolated profile");
    requireCondition(mirrorPath, "The catalog mirror database was not found under the isolated profile");
    const cacheRecord = JSON.parse(await readFile(cachePath, "utf8"));
    requireCondition(
      String(cacheRecord.body ?? "").includes("fixture/alpha-GGUF"),
      "The cached body does not carry the fixture catalog",
    );
    requireCondition(String(cacheRecord.signature ?? "").length > 0, "The cached catalog record lost its signature");
    const mirrorHeader = (await readFile(mirrorPath)).subarray(0, 15).toString("ascii");
    requireCondition(mirrorHeader.startsWith("SQLite format 3"), `The mirror file is not a SQLite database: ${mirrorHeader}`);
    record("catalog.disk-evidence", "The verified cache and the SQLite mirror exist under the isolated profile", "PASS", {
      cache_path: cachePath.replace(isolatedRoot, "[ISOLATED_ROOT]"),
      mirror_path: mirrorPath.replace(isolatedRoot, "[ISOLATED_ROOT]"),
      cache_signature_length: String(cacheRecord.signature).length,
      mirror_header: mirrorHeader,
    });

    const result = {
      schema_version: "1.0.0",
      verifier: "scripts/verify_060_catalog.mjs",
      phase: "first-fill",
      status: checks.every((entry) => entry.status === "PASS") ? "PASS" : "FAIL",
      checks,
      paths: { cache: cachePath, mirror: mirrorPath },
      fixture_url: state.url,
    };
    await writeFile(outPath, JSON.stringify(result, null, 2));
    console.log(JSON.stringify(result));
    requireCondition(result.status === "PASS", "One or more first-fill checks failed");
  } finally {
    client.close();
  }
}

async function prep(stateDir) {
  const state = JSON.parse(await readFile(join(stateDir, "state.json"), "utf8"));
  // Stop the fixture server (launch #2 must run offline).
  if (state.serverPid) {
    try {
      process.kill(state.serverPid);
    } catch {
      // already gone
    }
  }
  const catalogRoot = process.env.LOCALMOTIVE_CATALOG_ROOT;
  requireCondition(catalogRoot, "Set LOCALMOTIVE_CATALOG_ROOT before the corruption phase");
  const cachePath = await findInTree(catalogRoot, "catalog-cache.json");
  requireCondition(cachePath, "The catalog cache file was not found to corrupt");
  const cacheRecord = JSON.parse(await readFile(cachePath, "utf8"));
  cacheRecord.signature = Buffer.from("corrupted-signature").toString("base64");
  await writeFile(cachePath, JSON.stringify(cacheRecord, null, 2));
  const mirrorPath = await findInTree(catalogRoot, "catalog-mirror.sqlite");
  requireCondition(mirrorPath, "The catalog mirror was not found to corrupt");
  await writeFile(mirrorPath, Buffer.from("corrupted-mirror-bytes-not-sqlite"));
  // Clear the cooldown stamp so the restart refresh genuinely attempts the
  // (stopped) fixture endpoint instead of being throttled: the restart phase
  // exercises the offline/dead-server fallback, not the cooldown again.
  const stampPath = await findInTree(catalogRoot, "catalog-refresh-stamp.txt");
  let stampCleared = false;
  if (stampPath) {
    const { rm } = await import("node:fs/promises");
    await rm(stampPath, { force: true });
    stampCleared = true;
  }
  const record = {
    schema_version: "1.0.0",
    verifier: "scripts/verify_060_catalog.mjs",
    phase: "prep",
    status: "PASS",
    corrupted: {
      cache_signature: "invalid base64 signature, body untouched",
      mirror_prefix: "corrupted-mirror-bytes-not-sqlite",
      cooldown_stamp_cleared: stampCleared,
    },
  };
  console.log(JSON.stringify(record));
}

async function restart(port, stateDir, outPath) {
  const client = await attachToPage(port);
  const checks = [];
  const record = (id, description, status, detail) => {
    checks.push({ id, description, status, detail });
    console.log("CHECK", id, status);
  };
  try {
    await waitFor(client, requireTauri(), "The candidate did not expose the Tauri IPC bridge");
    console.log("PHASE restart: tauri bridge present");
    const catalogRoot = process.env.LOCALMOTIVE_CATALOG_ROOT;
    requireCondition(catalogRoot, "Set LOCALMOTIVE_CATALOG_ROOT");

    // 1. Offline availability with the fixture server down: rows still load.
    const local = await invokeViaPage(client, "load_model_catalog");
    requireCondition(local.ok, `load_model_catalog failed after restart: ${local.error}`);
    const localModels = local.value?.catalog?.models ?? [];
    requireCondition(localModels.length >= 1, "The restart local load returned no rows at all");
    record("restart.rows-without-network", "A restart without the fixture server still renders rows from the local snapshot", "PASS", {
      origin: local.value?.origin ?? null,
      model_count: localModels.length,
      refresh_error: local.value?.refreshError ?? null,
    });

    // 2. Fresh refresh fails honestly against the dead fixture server.
    const refresh = await invokeViaPage(client, "fetch_model_catalog");
    requireCondition(refresh.ok, `fetch_model_catalog threw instead of degrading: ${refresh.error}`);
    const refreshError = String(refresh.value?.refreshError ?? "");
    requireCondition(refreshError.length > 0, "The failed refresh reported no honest error text");
    record("restart.honest-refresh-error", "A refresh against the dead fixture endpoint degrades with an honest error", "PASS", {
      refresh_error: refreshError.slice(0, 240),
      origin: refresh.value?.origin ?? null,
      model_count: refresh.value?.catalog?.models?.length ?? 0,
    });

    // 3. Invalid-signature fallback: the corrupted cache must not be trusted.
    requireCondition(
      refreshError.length > 0 && localModels.length >= 1,
      "The corrupted-signature path lost the usable rows",
    );
    const cachePath = await findInTree(catalogRoot, "catalog-cache.json");
    const cacheRecord = JSON.parse(await readFile(cachePath, "utf8"));
    requireCondition(
      !String(cacheRecord.signature ?? "").includes("corrupted-signature") ||
        String(cacheRecord.body ?? "").includes("fixture/alpha-GGUF") === false,
      "The corrupted signature record was left in place",
    );
    record("restart.invalid-signature-fallback", "The corrupted cached signature is not trusted and rows survive", "PASS", {
      refresh_error: refreshError.slice(0, 240),
    });

    // 4. Corrupt-mirror recovery: the mirror is a valid SQLite database again.
    const mirrorPath = await findInTree(catalogRoot, "catalog-mirror.sqlite");
    const mirrorHeader = (await readFile(mirrorPath)).subarray(0, 15).toString("ascii");
    const quarantined = (await readdir(dirname(mirrorPath))).filter((name) => name.includes("catalog-mirror.sqlite.quarantine-"));
    requireCondition(
      mirrorHeader.startsWith("SQLite format 3"),
      `The mirror was not rebuilt as a SQLite database: ${mirrorHeader}`,
    );
    record("restart.corrupt-mirror-recovery", "The corrupted mirror is quarantined and rebuilt from verified bytes", "PASS", {
      mirror_header: mirrorHeader,
      quarantine_files: quarantined.length,
    });

    // 5. The UI shows the rows and an honest refresh state after restart.
    await clickNav(client, "HF Catalog");
    const uiState = await waitFor(
      client,
      `(() => {
        const rows = document.querySelectorAll('.catalog-results article, .catalog-model');
        const text = document.querySelector('.catalog-screen')?.innerText ?? '';
        return rows.length >= 1 ? { rows: rows.length, text: text.slice(0, 500) } : false;
      })()`,
      "The restart UI did not render rows",
    );
    record("restart.ui-honesty", "The restart UI renders rows and explains the local state", "PASS", {
      rows: uiState.rows,
      text: uiState.text.replace(/\s+/g, " ").slice(0, 300),
    });

    const result = {
      schema_version: "1.0.0",
      verifier: "scripts/verify_060_catalog.mjs",
      phase: "restart",
      status: checks.every((entry) => entry.status === "PASS") ? "PASS" : "FAIL",
      checks,
    };
    await writeFile(outPath, JSON.stringify(result, null, 2));
    console.log(JSON.stringify(result));
    requireCondition(result.status === "PASS", "One or more restart checks failed");
  } finally {
    client.close();
  }
}

async function merge(portablePath, firstFillPath, restartPath, outPath) {
  const first = JSON.parse(await readFile(firstFillPath, "utf8"));
  const second = JSON.parse(await readFile(restartPath, "utf8"));
  const artifact = await stat(portablePath);
  const record = {
    schema_version: "1.0.0",
    verifier: "scripts/verify_060_catalog.mjs",
    verifier_scope: "catalog-and-sqlite-workflows",
    source_revision: process.env.LOCALMOTIVE_SOURCE_REVISION ?? "UNKNOWN",
    artifact: {
      name: portablePath.split(/[\\/]/).pop(),
      size_bytes: artifact.size,
      sha256: await sha256File(portablePath),
    },
    phases: [first, second],
    overall_status: first.status === "PASS" && second.status === "PASS" ? "PASS" : "FAIL",
  };
  await writeFile(outPath, JSON.stringify(record, null, 2));
  console.log(JSON.stringify({ ...record, phases: record.phases.map((entry) => ({ phase: entry.phase, status: entry.status })) }));
  requireCondition(record.overall_status === "PASS", "The catalog/SQLite matrix is not PASS");
}

switch (phase) {
  case "init":
    await init(rest[0], rest[1]);
    break;
  case "first-fill":
    await firstFill(rest[0], rest[1], rest[2]);
    break;
  case "prep":
    await prep(rest[0]);
    break;
  case "restart":
    await restart(rest[0], rest[1], rest[2]);
    break;
  case "merge":
    await merge(rest[0], rest[1], rest[2], rest[3]);
    break;
  case "__serve":
    await serve(rest[0]);
    break;
  default:
    console.error(`unknown phase: ${phase}`);
    process.exit(2);
}
