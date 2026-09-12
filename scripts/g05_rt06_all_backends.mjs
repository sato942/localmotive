// RT-06.V3 live measurement on a representative supported Windows host:
// install every Windows x64 backend from the pinned llama.cpp release, then
// measure bytes hashed / verification jobs / elapsed time for listing,
// selecting, and starting one managed runtime, using the packaged binary's
// own IPC and the process-wide instrumentation counters.
//
// Usage: node scripts/g05_rt06_all_backends.mjs <cdpPort> <adapterId> [installBatch]
//   installBatch (default "all") - comma list of installKeys to install; use
//   e.g. "cpu,vulkan" for a smoke pass before the full run.
//
// Writes .hermes-0.6/rt06-all-backends-result.json and prints a summary.

import { writeFileSync } from "node:fs";
import { attach } from "./lib/cdp_client.mjs";

const port = Number(process.argv[2] ?? 0);
const adapterId = process.argv[3] ?? "";
const batch = (process.argv[4] ?? "all").trim();
if (!port) {
  console.error("usage: node g05_rt06_all_backends.mjs <cdpPort> <adapterId> [installBatch]");
  process.exit(2);
}

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const client = await attach(port, { deadlineMs: 120_000 });

const invoke = (command, args = {}) =>
  client.evaluate(
    `window.__TAURI_INTERNALS__.invoke(${JSON.stringify(command)}, ${JSON.stringify(args)})`,
    20 * 60 * 1000,
  );

// Some commands reject with a bare string (the runtime-compatibility guard);
// capture the rejection inside the page so the reason survives.
const invokeCatching = (command, args = {}) =>
  client.evaluate(
    `(async () => { try { const value = await window.__TAURI_INTERNALS__.invoke(${JSON.stringify(command)}, ${JSON.stringify(args)}); return { ok: true, value }; } catch (error) { return { ok: false, error: String(error) }; } })()`,
    20 * 60 * 1000,
  );

const checks = [];
const record = (name, ok, detail) => {
  checks.push({ name, ok, detail });
  console.log(`${ok ? "PASS" : "FAIL"} ${name} | ${detail}`);
};

const stats = async () => {
  const s = await invoke("runtime_verification_stats");
  return {
    jobs_started: Number(s.jobsStarted ?? s.jobs_started ?? 0),
    jobs_coalesced: Number(s.jobsCoalesced ?? s.jobs_coalesced ?? 0),
    bytes_hashed: Number(s.bytesHashed ?? s.bytes_hashed ?? 0),
    jobs_cancelled: Number(s.jobsCancelled ?? s.jobs_cancelled ?? 0),
  };
};
const delta = (before, after) => ({
  jobs_started: after.jobs_started - before.jobs_started,
  jobs_coalesced: after.jobs_coalesced - before.jobs_coalesced,
  bytes_hashed: after.bytes_hashed - before.bytes_hashed,
  jobs_cancelled: after.jobs_cancelled - before.jobs_cancelled,
});

await sleep(3000);
const setup = await invoke("load_runtime_setup", { adapterId: adapterId || null });
const options = (setup?.catalog?.options ?? []).filter((o) => o.backend !== "cuda-companion");
record("catalog.exposes-every-windows-backend", options.length >= 7, `options=${options.map((o) => o.installKey).join(",")}`);

const wanted =
  batch === "all" ? options.map((o) => o.installKey) : batch.split(",").map((s) => s.trim());
const installs = [];
for (const installKey of wanted) {
  const option = options.find((o) => o.installKey === installKey);
  if (!option) {
    record(`install.${installKey}`, false, "no catalog option for this install key");
    continue;
  }
  const before = await stats();
  const startedAt = Date.now();
  const attempt = await invokeCatching("install_managed_runtime", {
    request: {
      installKey: option.installKey,
      adapterId: option.backend === "cpu" ? null : adapterId || null,
    },
  });
  try {
    if (!attempt.ok) throw new Error(attempt.error);
    const installed = attempt.value;
    const elapsedMs = Date.now() - startedAt;
    const after = await stats();
    installs.push({
      installKey,
      backend: option.backend,
      outcome: installed?.reused ? "reused" : "installed",
      asset: option.asset?.name ?? null,
      assetBytes: option.asset?.size ?? null,
      companionAsset: option.companionAsset?.name ?? null,
      companionBytes: option.companionAsset?.size ?? null,
      elapsedMs,
      verification: delta(before, after),
    });
    record(`install.${installKey}`, true, `asset=${option.asset?.name} bytes=${option.asset?.size} elapsed=${elapsedMs}ms outcome=${installs.at(-1).outcome}`);
  } catch (error) {
    const message = String(error);
    // A backend whose vendor does not match any installed adapter is refused
    // by the runtime-compatibility guard; the two observed guard messages are
    // "GPU adapter <id> (<vendor>) is incompatible with the <backend> runtime"
    // and, without an adapter, "The selected runtime requires one bounded GPU
    // adapter ID". That refusal is correct behavior, not an install failure,
    // and proves a hostile adapter choice cannot install a wrong-vendor
    // runtime.
    const guarded = /incompatible with the .+ runtime|requires one bounded GPU adapter ID/.test(message);
    installs.push({
      installKey,
      backend: option.backend,
      outcome: guarded ? "refused-by-design" : "failed",
      asset: option.asset?.name ?? null,
      assetBytes: option.asset?.size ?? null,
      elapsedMs: Date.now() - startedAt,
      error: message.slice(0, 300),
    });
    record(
      `refusal.${installKey}`,
      guarded,
      guarded ? `guarded refusal: ${message.slice(0, 120)}` : `UNEXPECTED error=${message.slice(0, 160)}`,
    );
  }
}

// Listing: two consecutive enumerations make repeated-work visible.
const listRuns = [];
for (let pass = 0; pass < 2; pass += 1) {
  const before = await stats();
  const t0 = Date.now();
  const listed = await invoke("list_managed_runtimes");
  const elapsedMs = Date.now() - t0;
  const after = await stats();
  listRuns.push({ pass, elapsedMs, count: listed?.length ?? null, verification: delta(before, after) });
}
const eligible = installs.filter((i) => i.outcome === "installed" || i.outcome === "reused").length;
record(
  "listing.reports-every-installed-runtime",
  (listRuns[0]?.count ?? 0) >= eligible,
  `count=${listRuns[0]?.count} eligible=${eligible} refused-by-design=${installs.length - eligible}`,
);
record(
  "listing.second-pass-avoids-rehashing",
  listRuns[1].verification.bytes_hashed === 0,
  `pass1_bytes=${listRuns[0].verification.bytes_hashed} pass2_bytes=${listRuns[1].verification.bytes_hashed}`,
);

// Selecting + starting one managed runtime (cuda-13.3 when installed).
const record13 = (await invoke("list_managed_runtimes")).find((r) => r.installKey === "cuda-13.3");
let selecting = null;
let starting = null;
if (record13) {
  const before = await stats();
  const t0 = Date.now();
  const identity = await invoke("describe_runtime", { path: record13.runtimePath });
  selecting = {
    elapsedMs: Date.now() - t0,
    managedVerified: Boolean(identity?.managedVerified),
    verification: delta(before, await stats()),
  };
  record("selecting.describes-with-verified-identity", selecting.managedVerified, `elapsed=${selecting.elapsedMs}ms`);

  const beforeStart = await stats();
  const t1 = Date.now();
  const health = await invoke("check_managed_runtime_health", {
    request: { installKey: "cuda-13.3", adapterId: adapterId || null },
  });
  starting = {
    elapsedMs: Date.now() - t1,
    passed: Boolean(health?.passed),
    verification: delta(beforeStart, await stats()),
  };
  record("starting.health-run-passes", starting.passed, `elapsed=${starting.elapsedMs}ms`);
} else {
  record("selecting.describes-with-verified-identity", false, "cuda-13.3 not installed by this run");
  record("starting.health-run-passes", false, "cuda-13.3 not installed by this run");
}

const finalList = await invoke("list_managed_runtimes");
const result = {
  schema: "localmotive.rt06-all-backends.v1",
  availability: (setup?.catalog?.availability ?? []).map((a) => ({
    installKey: a.installKey,
    status: a.status,
    reason: a.reason,
  })),
  installedRecords: (finalList ?? []).map((r) => ({
    installKey: r.installKey,
    backend: r.backend,
    contentVerified: Boolean(r.contentVerified),
  })),
  generatedAt: new Date().toISOString(),
  host: { cdpPort: port, adapterId, batch },
  catalog: {
    tag: setup?.catalog?.tag ?? null,
    optionCount: options.length,
    options: options.map((o) => ({
      installKey: o.installKey,
      backend: o.backend,
      asset: o.asset?.name ?? null,
      assetBytes: o.asset?.size ?? null,
      companion: o.companionAsset?.name ?? null,
      companionBytes: o.companionAsset?.size ?? null,
      recommended: Boolean(o.recommended),
    })),
  },
  installs,
  listing: listRuns,
  selecting,
  starting,
  checks,
  summary: { total: checks.length, pass: checks.filter((c) => c.ok).length, fail: checks.filter((c) => !c.ok).length },
};
writeFileSync(".hermes-0.6/rt06-all-backends-result.json", JSON.stringify(result, null, 2));
console.log(`RT06 SUMMARY: ${result.summary.pass}/${result.summary.total} checks PASS`);
process.exit(result.summary.fail === 0 ? 0 : 1);
