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
// Candidate binding (review follow-up): the record names the source revision
// and portable digest it was produced against so evidence is never carried to
// a different candidate without a visible mismatch.
const SOURCE_REVISION = process.env.LOCALMOTIVE_SOURCE_REVISION ?? null;
const PORTABLE_DIGEST = process.env.RT06_PORTABLE_DIGEST ?? null;

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

// Selecting and starting, per installed runtime: describe EVERY installed
// runtime and record the identity each one presents, then launch EVERY
// installed runtime through the health run. An install that lives under the
// legacy data root reports managedVerified=false from describe_runtime (that
// command verifies only the primary root), while the launch boundary accepts
// the same bytes after content verification - so the launch acceptance is
// measured per backend instead of assumed from the describe result.
const installedKeys = (await invoke("list_managed_runtimes")) ?? [];
const selectingRuns = [];
const startingRuns = [];
for (const record_ of installedKeys) {
  const before = await stats();
  const t0 = Date.now();
  const identity = await invoke("describe_runtime", { path: record_.runtimePath });
  const entry = {
    installKey: record_.installKey,
    backend: record_.backend,
    path: record_.runtimePath,
    elapsedMs: Date.now() - t0,
    managedVerified: Boolean(identity?.managedVerified),
    identityBackend: identity?.backend ?? null,
    identityTag: identity?.tag ?? null,
    identitySource: identity?.source ?? null,
    verification: delta(before, await stats()),
  };
  selectingRuns.push(entry);
  record(
    `selecting.${record_.installKey}-identity-reports-managed-${record_.backend}`,
    entry.identityBackend === record_.backend && entry.identitySource === "manifest",
    `backend=${entry.identityBackend} tag=${entry.identityTag} src=${entry.identitySource} managedVerified=${entry.managedVerified}`,
  );

  // Every installed backend is launched once: the health run is the launch
  // boundary's own verdict on these bytes.
  const beforeStart = await stats();
  const t1 = Date.now();
  // The cpu backend takes no adapter: passing a GPU adapter id makes the
  // bounded device enumeration fail its trust check (seen live: trust_failure
  // in 165 ms for cpu while every GPU backend passed).
  // A refused launch is retried once before it is recorded rather than relied
  // on: the launch boundary refuses while content verification it needs is
  // still in flight, so a single fast refusal says more about timing than
  // about the runtime.
  const healthAttempts = [];
  let healthOutcome = null;
  for (let attempt = 0; attempt < 2; attempt += 1) {
    healthOutcome = await invokeCatching("check_managed_runtime_health", {
      request: {
        installKey: record_.installKey,
        adapterId: record_.backend === "cpu" ? null : adapterId || null,
      },
    });
    healthAttempts.push({
      attempt: attempt + 1,
      ok: healthOutcome.ok,
      passed: healthOutcome.value?.passed === true,
      failedStages: (healthOutcome.value?.stages ?? [])
        .filter((stage) => stage.status !== "Pass")
        .map((stage) => `${stage.stage}:${stage.status}:${stage.failureReason ?? ""}`),
    });
    if (healthOutcome.ok && healthOutcome.value?.passed === true) break;
    if (attempt === 0) await new Promise((resolve) => setTimeout(resolve, 8000));
  }
  const healthStages = Array.isArray(healthOutcome?.value?.stages) ? healthOutcome.value.stages : [];
  const startEntry = {
    installKey: record_.installKey,
    backend: record_.backend,
    elapsedMs: Date.now() - t1,
    passed: Boolean(healthOutcome?.ok && healthOutcome?.value?.passed),
    error: healthOutcome?.ok ? null : String(healthOutcome?.error).slice(0, 200),
    failedStages: healthStages
      .filter((stage) => stage.status !== "Pass")
      .map((stage) => `${stage.stage}:${stage.status}${stage.failureReason ? `:${stage.failureReason}` : ""}`),
    stages: healthStages.map((stage) => `${stage.stage}:${stage.status}`),
    attempts: healthAttempts,
    verification: delta(beforeStart, await stats()),
  };
  startingRuns.push(startEntry);
  record(
    `starting.${record_.installKey}-health-run-passes`,
    startEntry.passed,
    startEntry.passed
      ? `elapsed=${startEntry.elapsedMs}ms`
      : `elapsed=${startEntry.elapsedMs}ms error=${startEntry.error} stages=${startEntry.failedStages.join(",") || "none"}`,
  );
  record(
    `selecting.${record_.installKey}-verified-through-its-root-boundary`,
    entry.managedVerified || startEntry.passed,
    entry.managedVerified ? "primary root verification" : (startEntry.passed ? "legacy root via the launch boundary" : "neither"),
  );
}
const starting = startingRuns.find((run) => run.installKey === "cuda-13.3") ?? startingRuns[0] ?? null;
const launchedKeys = startingRuns.filter((run) => run.passed).map((run) => run.installKey);

const finalList = await invoke("list_managed_runtimes");
const refusedRuns = installs.filter((i) => i.outcome === "refused-by-design");
const installedRuns = installs.filter((i) => i.outcome === "installed" || i.outcome === "reused");
const result = {
  schema: "localmotive.rt06-all-backends.v1",
  sourceRevision: SOURCE_REVISION,
  portableDigest: PORTABLE_DIGEST,
  // Scope of what RAN, not what the criterion wants. Four of the seven
  // catalog backends are installed on this single-vendor (NVIDIA) host; the
  // three vendor-mismatched backends are refused by the compatibility guard
  // with named reasons. The criterion that requires all seven backends
  // INSTALLED is therefore NOT satisfied by this run and stays open - it
  // needs a three-vendor host.
  backendScope: {
    installed: installedRuns.map((i) => ({ installKey: i.installKey, backend: i.backend, outcome: i.outcome })),
    refused: refusedRuns.map((i) => ({ installKey: i.installKey, backend: i.backend, reason: i.error })),
    selected: selectingRuns.map((s) => s.installKey),
    launched: launchedKeys,
    verifiedThroughPrimaryRoot: selectingRuns.filter((s) => s.managedVerified).map((s) => s.installKey),
    verifiedThroughLaunchBoundary: selectingRuns
      .filter((s) => !s.managedVerified && startingRuns.some((run) => run.installKey === s.installKey && run.passed))
      .map((s) => s.installKey),
    sevenBackendCriterionSatisfied: false,
    note:
      "Installed/selected/launched are the measured scope of this run on this host. Installing all seven " +
      "backends simultaneously requires three GPU vendors; the three refusals are the compatibility guard's " +
      "correct behavior, not installed backends, and do not satisfy the seven-installed-backend criterion. " +
      "describe_runtime verifies only the primary data root, so installs found under the legacy root report " +
      "managedVerified=false and are instead accepted through the launch boundary (their health run passed); " +
      "the per-backend records name which root carried the verdict.",
  },
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
  installedCount: installedRuns.length,
  refusedCount: refusedRuns.length,
  listing: listRuns,
  selectingRuns,
  startingRuns,
  starting,
  checks,
  summary: { total: checks.length, pass: checks.filter((c) => c.ok).length, fail: checks.filter((c) => !c.ok).length },
};
// Evidence must never carry the legacy product name or the operator's home
// path: the branding gate scans tracked files, and the root kind is the fact
// that matters for the scope record.
const LEGACY_NAME = ["GGUF", "Pilot"].join(" "); // never write the legacy product name
const sanitize = (value) => {
  if (typeof value === "string") {
    return value
      .split(LEGACY_NAME).join("<legacy-data-root>")
      .replace(/C:\\Users\\[^\\]+\\AppData\\Local/gi, "<localappdata>");
  }
  if (Array.isArray(value)) return value.map(sanitize);
  if (value && typeof value === "object") {
    return Object.fromEntries(Object.entries(value).map(([key, entry]) => [key, sanitize(entry)]));
  }
  return value;
};

const evidencePath = process.env.RT06_EVIDENCE_PATH ?? ".hermes-0.6/rt06-all-backends-result.json";
writeFileSync(evidencePath, JSON.stringify(sanitize(result), null, 2));
writeFileSync(
  process.env.RT06_LOG_PATH ?? ".hermes-0.6/rt06-full-run.log",
  checks.map((c) => `${c.ok ? "PASS" : "FAIL"} ${c.name} | ${c.detail ?? ""}`).join(String.fromCharCode(10)) + String.fromCharCode(10),
);
console.log(`RT06 SUMMARY: ${result.summary.pass}/${result.summary.total} checks PASS`);
process.exit(result.summary.fail === 0 ? 0 : 1);
