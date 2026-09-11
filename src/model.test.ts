import { describe, expect, it } from "vitest";
import * as modelModule from "./model";
import {
  formatExtraArgs,
  parseExtraArgs,
  artifactReadyForLaunch,
  calibrationState,
  catalogRevision,
  conflictingCapacityMetrics,
  defaultWorkload,
  derivedEvidence,
  downloadKey,
  downloadPercent,
  downloadReadiness,
  errorText,
  hardwareFitBudget,
  keepLatestRequest,
  evidenceRunLabel,
  selectionAfterRescan,
  displayedSelection,
  tuningRunSummary,
  applySuggestedPort,
  profileIdentity,
  responseIsCurrent,
  managedHealthRequest,
  manualGpuOverride,
  modelHiddenByFitRule,
  retainOrDisposeListener,
  etaLabel,
  normalizeProfile,
  originLabel,
  qualityPassRate,
  rateLabel,
  runtimeIdentityMismatch,
  runtimeInstallRequest,
  runtimeCatalogViewState,
  runtimeOptionState,
  suggestedProfile,
  validateWorkload,
  type ArtifactInspection,
  type CalibrationModel,
  type DeviceAllocationPlan,
  type Evidence,
  type GpuAdapterInfo,
  type GithubAsset,
  type LogicalModel,
  type ManagedRuntimeRecord,
  type RuntimeIdentity,
  type RuntimeOption,
  type StorageVolumeEvidence,
  type Workload,
} from "./model";

describe("runtime catalog presentation", () => {
  it("preserves typed rate-limit retry metadata from IPC", () => {
    const error = {
      kind: "rate_limited" as const,
      message: "Retry later",
      retryAfterSeconds: 120,
    };

    const parser = (
      modelModule as typeof modelModule & {
        runtimeCatalogErrorFromUnknown?: (value: unknown) => unknown;
      }
    ).runtimeCatalogErrorFromUnknown ?? (() => null);
    expect(parser(error)).toEqual(error);
  });

  it("shows a catalog error after loading stops", () => {
    const error = {
      kind: "rate_limited" as const,
      message: "Catalog request was rate limited",
      retryAfterSeconds: 120,
    };
    expect(
      runtimeCatalogViewState({
        loading: false,
        catalog: null,
        error,
      }),
    ).toEqual({ kind: "error", error });
  });

  it("uses an explicit empty state when an approved catalog has no installable options", () => {
    const catalog = {
      tag: "b10816",
      publishedAt: "2026-09-04T00:00:00Z",
      options: [],
      availability: [],
      origin: "network" as const,
      warning: null,
      recommendationReason: "CPU fallback fixture",
    };

    expect(
      runtimeCatalogViewState({
        loading: false,
        catalog,
        error: null,
      }),
    ).toEqual({ kind: "empty", catalog });
  });

  it("sends only the immutable install key and exact selected adapter", () => {
    const cpu = { backend: "cpu", installKey: "cpu" } as RuntimeOption;
    const cuda = { backend: "cuda", installKey: "cuda-13.3" } as RuntimeOption;

    expect(runtimeInstallRequest(cpu, "ignored-adapter")).toEqual({
      installKey: "cpu",
      adapterId: null,
    });
    expect(runtimeInstallRequest(cuda, "gpu-7")).toEqual({
      installKey: "cuda-13.3",
      adapterId: "gpu-7",
    });
    expect(() => runtimeInstallRequest(cuda, "")).toThrow(/adapter/i);
  });

  it("sends only immutable managed-health authority while the backend owns the model pin", () => {
    const cpu = { backend: "cpu", installKey: "cpu" } as RuntimeOption;
    const cuda = { backend: "cuda", installKey: "cuda-13.3" } as RuntimeOption;

    expect(managedHealthRequest(cpu, "ignored")).toEqual({
      installKey: "cpu",
      adapterId: null,
    });
    expect(managedHealthRequest(cuda, "gpu-7")).toEqual({
      installKey: "cuda-13.3",
      adapterId: "gpu-7",
    });
  });
});

describe("managed health presentation", () => {
  it("presents an interrupted health contract as cancelled rather than failed", () => {
    const classify = (
      modelModule as typeof modelModule & {
        managedHealthOutcome?: (value: modelModule.ManagedHealthResult) => string;
      }
    ).managedHealthOutcome ?? (() => "failed");
    const result = {
      passed: false,
      stages: [
        {
          stage: "device_enumeration" as const,
          status: "FAIL" as const,
          durationMs: 1,
          failureReason: "cancelled" as const,
          detail: "Cancelled by the user",
          completion: null,
        },
      ],
    } as modelModule.ManagedHealthResult;

    expect(classify(result)).toBe("cancelled");
  });
});

describe("launch failure presentation", () => {
  it("shows the message and bounded log tail from structured launch evidence", () => {
    const failure = JSON.stringify({
      schema: 1,
      phase: "health_exit",
      exitCode: 7,
      timedOut: false,
      cancelled: false,
      message: "llama-server exited before becoming healthy",
      logTail: "useful exit detail",
    });

    expect(errorText(failure)).toBe(
      "llama-server exited before becoming healthy\nuseful exit detail",
    );
  });

  it("shows structured server-status failure evidence", () => {
    expect(
      errorText({
        schema: 1,
        phase: "runtime_exit",
        exitCode: 7,
        timedOut: false,
        cancelled: false,
        message: "llama-server exited after health validation",
        logTail: "useful runtime exit detail",
      }),
    ).toBe("llama-server exited after health validation\nuseful runtime exit detail");
  });
});

describe("v0.3 evidence contracts", () => {
  it("preserves device capacity beside the unvalidated allocation estimate", () => {
    const evidence = (value: number): Evidence<number> => ({
      value,
      level: "observed",
      source: { kind: "windowsApi", detail: "fixture" },
      observedAtMs: 42,
      notes: [],
    });
    const plan: DeviceAllocationPlan = {
      adapterId: "gpu-0",
      weightBytes: evidence(100),
      kvCacheBytes: evidence(50),
      totalBytes: evidence(150),
      availableBytes: evidence(200),
      note: "Launch validation remains required",
    };

    expect(plan.availableBytes.value).toBe(200);
    expect(plan.note).toMatch(/validation/i);
  });

  it("keeps resident and available bytes separate for each volume", () => {
    const volume: StorageVolumeEvidence = {
      volumePath: {
        value: "C:\\",
        level: "observed",
        source: { kind: "windowsApi", detail: "GetVolumePathNameW" },
        observedAtMs: 42,
        notes: [],
      },
      residentBytes: {
        value: 100,
        level: "exact",
        source: { kind: "fileSystem", detail: "selected artifact bytes" },
        observedAtMs: 42,
        notes: [],
      },
      availableBytes: {
        value: 2_000,
        level: "observed",
        source: { kind: "windowsApi", detail: "GetDiskFreeSpaceExW" },
        observedAtMs: 42,
        notes: [],
      },
    };

    expect(volume.residentBytes.value).toBe(100);
    expect(volume.availableBytes.value).toBe(2_000);
  });

  it("propagates an unknown required input into derived evidence", () => {
    const result = derivedEvidence(
      4096,
      ["exact", "unknown"],
      { kind: "calculation", detail: "KV bytes" },
      42,
      [],
    );

    expect(result.level).toBe("unknown");
    expect(result.value).toBeNull();
    expect(result.notes.join(" ")).toMatch(/required input/i);
  });

  it("keeps known calculations derived instead of measured", () => {
    const result: Evidence<number> = derivedEvidence(
      4096,
      ["exact", "observed"],
      { kind: "calculation", detail: "KV bytes" },
      42,
      [],
    );

    expect(result.level).toBe("derived");
    expect(result.value).toBe(4096);
  });

  it("validates the default benchmark workload", () => {
    const workload = defaultWorkload();

    expect(validateWorkload(workload)).toEqual([]);
    expect(workload).toMatchObject({ warmups: 1, trials: 5, seed: 42 });
  });

  it("rejects invalid workload limits before IPC", () => {
    const errors = validateWorkload({
      ...defaultWorkload(),
      trials: 0,
      concurrency: 65,
    });

    expect(errors.map((error) => error.code)).toEqual(["invalidRange", "limitExceeded"]);
    expect(errors.map((error) => error.field)).toEqual([
      "workload.trials",
      "workload.concurrency",
    ]);
  });

  it("mirrors the Rust workload defaults exactly (contract fixture)", () => {
    // Literal mirror of evidence.rs Workload::default() with its serde
    // camelCase names (audit FE-17 I4): if either side changes, this fails.
    const rustDefaultWorkload: Workload = {
      id: "technical-explanation-v1",
      promptTokens: 512,
      generationTokens: 256,
      warmups: 1,
      trials: 5,
      seed: 42,
      concurrency: 1,
      stream: false,
      cacheMode: "warm",
      timeoutMs: 600_000,
    };
    expect(defaultWorkload()).toEqual(rustDefaultWorkload);
    expect(validateWorkload(rustDefaultWorkload)).toEqual([]);
  });

  it("requires whole numbers where Rust deserializes integers", () => {
    const fractional = validateWorkload({
      ...defaultWorkload(),
      trials: 2.5,
      promptTokens: 511.5,
    });
    expect(fractional.map((error) => error.field)).toEqual([
      "workload.promptTokens",
      "workload.trials",
    ]);
    expect(fractional.every((error) => error.message === "Value must be a whole number")).toBe(true);

    // Complete maxima and a blank identity surface as field errors.
    const over = validateWorkload({
      ...defaultWorkload(),
      promptTokens: 1_048_577,
      generationTokens: 65_537,
      warmups: 11,
      timeoutMs: 3_600_001,
      id: "   ",
    });
    expect(over.map((error) => error.field)).toEqual([
      "workload.id",
      "workload.promptTokens",
      "workload.generationTokens",
      "workload.warmups",
      "workload.timeoutMs",
    ]);
    expect(over.every((error) => error.code === "limitExceeded" || error.code === "emptyIdentity")).toBe(true);
  });
});

describe("v0.3 artifact contracts", () => {
  it("allows launch only for complete artifacts with consistent headers", () => {
    const artifact: ArtifactInspection = {
      logicalId: "a".repeat(64),
      contentId: "b".repeat(64),
      logicalName: "Fixture-Q4",
      firstShard: "C:/models/Fixture-Q4-00001-of-00002.gguf",
      expectedShards: 2,
      complete: true,
      headerConsistent: true,
      identityLevel: "exact",
      shardBytes: 20,
      companionBytes: 0,
      shards: [],
      companions: [],
      summary: null,
      problems: [],
    };

    expect(artifactReadyForLaunch(artifact)).toBe(true);
    expect(artifactReadyForLaunch({ ...artifact, complete: false })).toBe(false);
    expect(artifactReadyForLaunch({ ...artifact, headerConsistent: false })).toBe(false);
  });
});

describe("v0.3 measurement decisions", () => {

  it("leaves quality unknown when the suite did not produce scored cases", () => {
    expect(qualityPassRate(null)).toBeNull();
    const emptySuite = {
      suiteId: "v1",
      seed: 42,
      observedAtMs: null,
      modelLogicalId: null,
      runtimeSha256: null,
      cases: [],
    };
    expect(qualityPassRate({ ...emptySuite, status: "notRun" })).toBeNull();
    expect(qualityPassRate({ ...emptySuite, status: "error" })).toBeNull();
  });
});

describe("v0.3 calibration decisions", () => {
  const model: CalibrationModel = {
    compatibilityKey: "a".repeat(64),
    factor: 1,
    residualStandardDeviation: 0.1,
    anchorCount: 3,
    createdAtMs: 100,
    expiresAtMs: 200,
  };

  it("distinguishes compatible, expired, incompatible, and unavailable calibration", () => {
    expect(calibrationState(model, model.compatibilityKey, 150)).toBe("compatible");
    expect(calibrationState(model, model.compatibilityKey, 201)).toBe("expired");
    expect(calibrationState(model, "b".repeat(64), 150)).toBe("incompatible");
    // Audit MT-14: a model created after the clock is not yet applicable,
    // and freshness follows the source evidence, not the rebuild time.
    expect(calibrationState(model, model.compatibilityKey, 50)).toBe("scheduled");
    const dayMs = 24 * 60 * 60 * 1_000;
    const staleModel: CalibrationModel = {
      ...model,
      sourceEvidenceAtMs: 1_000,
      expiresAtMs: 1_000 + 200 * dayMs,
    };
    expect(
      calibrationState(staleModel, staleModel.compatibilityKey, 1_000 + 91 * dayMs),
    ).toBe("staleEvidence");
    expect(
      calibrationState(staleModel, staleModel.compatibilityKey, 1_000 + 89 * dayMs),
    ).toBe("compatible");
    expect(calibrationState(null, model.compatibilityKey, 150)).toBe("unavailable");
  });
});

describe("manual GPU override decisions", () => {
  it("converts one explicit GiB value without combining shared memory", () => {
    expect(manualGpuOverride("gpu-0", "8", "Firmware reserve")).toEqual({
      adapterId: "gpu-0",
      dedicatedBytes: 8 * 1024 ** 3,
      sharedBytes: null,
      note: "Firmware reserve",
    });
  });

  it("rejects missing adapters and invalid capacities", () => {
    expect(manualGpuOverride("", "8", "")).toBeNull();
    expect(manualGpuOverride("gpu-0", "0", "")).toBeNull();
    expect(manualGpuOverride("gpu-0", "Infinity", "")).toBeNull();
  });
});

describe("hardware evidence", () => {
  it("keeps conflicts visible by metric", () => {
    const observed = (value: number, kind: "windowsApi" | "nvidiaSmi") => ({
      value,
      level: "observed" as const,
      source: { kind, detail: kind },
      observedAtMs: 42,
      notes: [],
    });
    const unknown = {
      value: null,
      level: "unknown" as const,
      source: { kind: "unknown" as const, detail: "fixture" },
      observedAtMs: 42,
      notes: ["unknown"],
    };
    const adapter: GpuAdapterInfo = {
      adapterId: "luid:1",
      compatibilityId: "pci:10de:0001:00000000:00",
      name: "Fixture GPU",
      vendor: "nvidia",
      driver: {
        value: "1",
        level: "observed",
        source: { kind: "nvidiaSmi", detail: "fixture" },
        observedAtMs: 42,
        notes: [],
      },
      backend: {
        value: "cuda",
        level: "heuristic",
        source: { kind: "policy", detail: "fixture" },
        observedAtMs: 42,
        notes: [],
      },
      dedicatedBytes: observed(100, "windowsApi"),
      sharedBytes: observed(50, "windowsApi"),
      budgetBytes: observed(80, "windowsApi"),
      currentUsageBytes: observed(10, "windowsApi"),
      availableBudgetBytes: observed(70, "windowsApi"),
      reservationBytes: unknown,
      availableForReservationBytes: unknown,
      capacityObservations: [
        { metric: "dedicated", evidence: observed(100, "windowsApi") },
        { metric: "dedicated", evidence: observed(99, "nvidiaSmi") },
        { metric: "budget", evidence: observed(80, "windowsApi") },
      ],
    };

    expect(conflictingCapacityMetrics(adapter)).toEqual(["dedicated"]);
  });
});

describe("asynchronous UI race guards", () => {
  it("accepts only the newest request result", () => {
    expect(keepLatestRequest(3, 3)).toBe(true);
    expect(keepLatestRequest(2, 3)).toBe(false);
  });

  it("immediately disposes a listener that resolves after unmount", () => {
    let stopped = 0;
    expect(retainOrDisposeListener(true, () => { stopped += 1; })).toBeNull();
    expect(stopped).toBe(1);
    const stop = () => { stopped += 1; };
    expect(retainOrDisposeListener(false, stop)).toBe(stop);
  });
});

describe("catalog revisions", () => {
  it("preserves a pinned revision and defaults only omitted values", () => {
    const base = { quant: "Q4", filename: "x.gguf", sizeBytes: 1, sha256: "a".repeat(64) };
    expect(catalogRevision({ ...base, revision: "refs/pr/7" })).toBe("refs/pr/7");
    expect(catalogRevision(base)).toBe("main");
  });
});

const model: LogicalModel = {
  id: "target",
  name: "LFM2.5-2.6B-Q8_0",
  directory: "C:\\models\\LFM",
  firstShard: "C:\\models\\LFM\\target.gguf",
  sizeBytes: 2_874_779_648,
  shardCount: 1,
  expectedShards: 1,
  complete: true,
  quant: "Q8_0",
  shards: [],
  companions: [
    {
      path: "C:\\models\\LFM\\draft.gguf",
      name: "LFM2.5-DSpark-F16.gguf",
      role: "dspark",
      sizeBytes: 663_691_776,
    },
  ],
};

describe("normalizeProfile", () => {
  it("adopts the current runtime instead of the one saved with the profile", () => {
    // Bug: after installing a runtime update, opening a model whose profile was
    // saved against the old executable silently pinned the app back to it.
    const stored = {
      ...suggestedProfile(model, "C:\\llama\\llama-server.exe"),
      name: "My tuned profile",
      context: 16384,
    };
    const next = normalizeProfile(
      stored,
      model,
      "C:\\Users\\x\\AppData\\Local\\Localmotive\\runtimes\\b10752\\cuda-13.3\\llama-server.exe",
    );
    expect(next.runtime).toBe(
      "C:\\Users\\x\\AppData\\Local\\Localmotive\\runtimes\\b10752\\cuda-13.3\\llama-server.exe",
    );
    // Everything the user actually chose survives.
    expect(next.name).toBe("My tuned profile");
    expect(next.context).toBe(16384);
  });

  it("keeps the stored runtime when no runtime is currently selected", () => {
    const stored = { ...suggestedProfile(model, "C:\\llama\\llama-server.exe") };
    expect(normalizeProfile(stored, model, "").runtime).toBe("C:\\llama\\llama-server.exe");
  });
});

describe("suggestedProfile", () => {
  it("selects an explicit compatible DSpark companion", () => {
    const profile = suggestedProfile(model, "C:\\llama\\llama-server.exe");
    expect(profile.specType).toBe("draft-dspark");
    expect(profile.draftModel).toBe("C:\\models\\LFM\\draft.gguf");
    expect(profile.alias).toBe("lfm2-5-2-6b-q8-0");
    expect(profile.host).toBe("127.0.0.1");
    expect(profile.flashAttention).toBe("auto");
    expect(profile.cacheTypeK).toBe("f16");
    expect(profile.cachePrompt).toBe(true);
    expect(profile.continuousBatching).toBe(true);
    expect(profile.reasoning).toBe("auto");
    expect(profile.corsOrigins).toBe("localhost");
  });
});

const asset = (name: string): GithubAsset => ({ name, browserDownloadUrl: "https://example.invalid/" + name, size: 1, digest: null });
const option = (installKey: string, backend: string): RuntimeOption => ({
  id: `b10752:${installKey}`,
  label: installKey,
  backend,
  installKey,
  description: "",
  compatibility: "",
  asset: asset(`llama-b10752-bin-win-${installKey}-x64.zip`),
  companionAsset: null,
  recommended: false,
});
const cuda13 = option("cuda-13.3", "cuda");
const cuda12 = option("cuda-12.4", "cuda");
const cpu = option("cpu", "cpu");
const managedCuda13 = (tag: string): ManagedRuntimeRecord => ({
  tag,
  backend: "cuda",
  installKey: "cuda-13.3",
  runtimePath: `C:\\managed\\${tag}\\cuda-13.3\\llama-server.exe`,
  installRoot: `C:\\managed\\${tag}\\cuda-13.3`,
  contentVerified: false,
});
const identity = (over: Partial<RuntimeIdentity>): RuntimeIdentity => ({
  path: "C:\\llama\\llama-server.exe",
  backend: "unknown",
  cudaMajor: null,
  tag: null,
  installKey: null,
  source: "none",
  managedVerified: false,
  ...over,
});

describe("runtimeOptionState", () => {
  it("offers install when nothing matching is installed or active", () => {
    expect(runtimeOptionState(cuda13, "b10752", [], null, null)).toEqual({ kind: "install" });
  });

  it("marks the active managed build of the same variant as active", () => {
    const state = runtimeOptionState(cuda13, "b10752", [managedCuda13("b10752")], identity({ source: "manifest", backend: "cuda", cudaMajor: 13, tag: "b10752", installKey: "cuda-13.3" }), "10752");
    expect(state).toEqual({ kind: "active" });
  });

  it("offers an update when the active managed build of the same variant is older", () => {
    const state = runtimeOptionState(cuda13, "b10760", [managedCuda13("b10752")], identity({ source: "manifest", backend: "cuda", cudaMajor: 13, tag: "b10752", installKey: "cuda-13.3" }), "10752");
    expect(state).toEqual({ kind: "update", from: "b10752" });
  });

  it("offers to use an installed-but-inactive managed build of the same release", () => {
    const state = runtimeOptionState(cuda13, "b10752", [managedCuda13("b10752")], identity({ source: "manifest", backend: "cpu", tag: "b10752", installKey: "cpu" }), "10752");
    expect(state).toEqual({ kind: "use", runtimePath: "C:\\managed\\b10752\\cuda-13.3\\llama-server.exe" });
  });

  it("recognises a user-supplied CUDA 13 build by its DLLs and offers an update when the release is newer", () => {
    const state = runtimeOptionState(cuda13, "b10752", [], identity({ source: "dlls", backend: "cuda", cudaMajor: 13 }), "10679");
    expect(state).toEqual({ kind: "update", from: "b10679" });
  });

  it("does not treat a user-supplied CUDA 13 build as matching the CUDA 12 package", () => {
    expect(runtimeOptionState(cuda12, "b10752", [], identity({ source: "dlls", backend: "cuda", cudaMajor: 13 }), "10679")).toEqual({ kind: "install" });
  });

  it("marks a user-supplied build active when its build equals the release", () => {
    expect(runtimeOptionState(cpu, "b10752", [], identity({ source: "dlls", backend: "cpu" }), "10752")).toEqual({ kind: "active" });
  });

  it("still offers install when the active build cannot be identified", () => {
    expect(runtimeOptionState(cpu, "b10752", [], identity({ source: "none" }), "10752")).toEqual({ kind: "install" });
  });

  it("offers reinstall with detail when installed files disagree with the install record", () => {
    // Phase 5 RED: a `manifest-dll-mismatch` identity must surface as
    // `mismatch` with actionable detail, never as `active`, `update`,
    // `use`, or `install`. The frontend already renders this state
    // (MISMATCH · REINSTALL in App.tsx); the decision function must now
    // produce it. Finding A-09 (clean-machine dependency failure)
    // records this gap.
    const active = identity({ source: "manifest-dll-mismatch", backend: "mismatch", cudaMajor: 13, tag: "b10752", installKey: "cuda-13.3" });
    expect(runtimeIdentityMismatch(cuda13, active)).toBe("Installed files disagree with the install record");
    expect(runtimeIdentityMismatch(cuda12, active)).toBeNull();
    expect(runtimeIdentityMismatch(cuda13, null)).toBeNull();
    const state = runtimeOptionState(cuda13, "b10752", [managedCuda13("b10752")], active, "10752");
    expect(state).toEqual({ kind: "mismatch", detail: "Installed files disagree with the install record" });
  });
});

describe("HF catalog download rules", () => {
  const base = {
    destination: "C:/models",
    running: false,
    alreadyOnDisk: false,
    gated: false,
    hasToken: false,
  };

  it("refuses to start without a destination folder and says so", () => {
    const result = downloadReadiness({ ...base, destination: "" });
    expect(result.canStart).toBe(false);
    expect(result.reason).toMatch(/model folder/i);
    expect(downloadReadiness({ ...base, destination: "   " }).canStart).toBe(false);
  });

  it("refuses a second download of a file already in flight", () => {
    const result = downloadReadiness({ ...base, running: true });
    expect(result.canStart).toBe(false);
    expect(result.reason).toMatch(/already downloading/i);
  });

  it("lets the backend verify and reuse a file that is already on disk", () => {
    // A same-named local file may be stale or corrupt. The backend checks size
    // and a Hugging Face SHA-256 ETag instead of trusting the filename alone.
    expect(downloadReadiness({ ...base, alreadyOnDisk: true }).canStart).toBe(true);
  });

  it("blocks a gated repo until a token exists, then allows it", () => {
    // Gated repos return 401/403 without a token; catching it here gives a
    // better message than letting the download fail after it starts.
    const blocked = downloadReadiness({ ...base, gated: true, hasToken: false });
    expect(blocked.canStart).toBe(false);
    expect(blocked.reason).toMatch(/token/i);
    expect(downloadReadiness({ ...base, gated: true, hasToken: true }).canStart).toBe(true);
  });

  it("allows a normal download and gives no reason text", () => {
    const result = downloadReadiness(base);
    expect(result.canStart).toBe(true);
    expect(result.reason).toBe("");
  });

  it("builds a stable key per repo and file", () => {
    expect(downloadKey("unsloth/Qwen3.8-27B-GGUF", "a.gguf")).toBe(
      "unsloth/Qwen3.8-27B-GGUF/a.gguf",
    );
    expect(downloadKey("a/b", "x.gguf")).not.toBe(downloadKey("a/b", "y.gguf"));
  });
});

describe("download progress display", () => {
  it("clamps the percentage instead of overflowing the bar", () => {
    expect(downloadPercent(0, 100)).toBe(0);
    expect(downloadPercent(50, 100)).toBe(50);
    expect(downloadPercent(100, 100)).toBe(100);
    expect(downloadPercent(500, 100)).toBe(100);
    expect(downloadPercent(-5, 100)).toBe(0);
  });

  it("returns zero rather than NaN when the total is unknown", () => {
    // A NaN width silently breaks the progress bar, so guard every bad input.
    expect(downloadPercent(10, 0)).toBe(0);
    expect(downloadPercent(10, Number.NaN)).toBe(0);
    expect(downloadPercent(Number.NaN, 100)).toBe(0);
    expect(downloadPercent(10, Number.POSITIVE_INFINITY)).toBe(0);
  });

  it("formats the remaining time in units a person reads", () => {
    expect(etaLabel(0, 1000, 100)).toBe("10s left");
    expect(etaLabel(0, 100_000, 100)).toBe("16m 40s left");
    expect(etaLabel(0, 10_000_000, 1000)).toBe("2h 46m left");
  });

  it("shows no estimate when there is nothing to estimate from", () => {
    expect(etaLabel(0, 1000, 0)).toBe("");
    expect(etaLabel(1000, 1000, 100)).toBe("");
    expect(etaLabel(0, 0, 100)).toBe("");
  });

  it("labels transfer rate only when it is known", () => {
    expect(rateLabel(0)).toBe("");
    expect(rateLabel(1024 ** 3)).toBe("1.00 GiB/s");
  });
});

describe("catalog origin honesty", () => {
  it("marks a cached list as possibly stale rather than live", () => {
    // Showing an offline cache as live would misrepresent the catalog.
    expect(originLabel("cache").tone).toBe("warn");
    expect(originLabel("cache").label).toMatch(/OFFLINE/);
    expect(originLabel("network").tone).toBe("ok");
    expect(originLabel("not-modified").tone).toBe("ok");
  });

  it("always includes words, never colour alone", () => {
    for (const origin of ["network", "not-modified", "cache", "bundled"] as const) {
      expect(originLabel(origin).label.trim().length).toBeGreaterThan(3);
    }
  });
});

describe("hardware auto-fit budget", () => {
  it("prefers dedicated VRAM and never sums budgets", () => {
    // Mirrors catalog::hardware_fit_budget. Rust owns truth; this pins the
    // same answers on the frontend copy so the UI cannot drift.
    expect(hardwareFitBudget([8_000_000_000], [32_000_000_000], 64_000_000_000)).toEqual({
      budgetBytes: 8_000_000_000,
      source: "dedicated",
    });
    expect(hardwareFitBudget([], [32_000_000_000], 64_000_000_000)).toEqual({
      budgetBytes: 32_000_000_000,
      source: "shared",
    });
    expect(hardwareFitBudget([], [], 64_000_000_000)).toEqual({
      budgetBytes: 64_000_000_000,
      source: "system",
    });
    expect(hardwareFitBudget([null, 0], [null], 0)).toEqual({ budgetBytes: 0, source: "system" });
  });

  it("hides files above half the budget by default and disables on zero", () => {
    // A 20 GB Q8 row hides on a 32 GiB budget at 500 per mille.
    expect(modelHiddenByFitRule(20_000_000_000, 500, 34_359_738_368)).toBe(true);
    expect(modelHiddenByFitRule(20_000_000_000, 0, 34_359_738_368)).toBe(false);
    expect(modelHiddenByFitRule(20_000_000_000, 500, 0)).toBe(false);
    expect(modelHiddenByFitRule(20_000_000_000, 1000, 20_000_000_000)).toBe(false);
    expect(modelHiddenByFitRule(20_000_000_000, undefined, 34_359_738_368)).toBe(false);
  });
});

describe("FE-03 stale response guards", () => {
  const profile = (
    over: Partial<import("./model").LaunchProfile> = {},
  ): import("./model").LaunchProfile =>
    ({
      name: "profile-a",
      runtime: "C:/runtime/llama-server.exe",
      model: "C:/models/a.gguf",
      draftModel: null,
      mmproj: null,
      host: "127.0.0.1",
      port: 8080,
      alias: "a",
      context: 4096,
      gpuLayers: "all",
      batch: 2048,
      ubatch: 512,
      flashAttention: "auto",
      threads: -1,
      threadsBatch: -1,
      cacheTypeK: "f16",
      cacheTypeV: "f16",
      ...over,
    }) as unknown as import("./model").LaunchProfile;

  it("accepts only the latest response for the current resource identity", () => {
    expect(responseIsCurrent(3, 3, "openrouter", "openrouter")).toBe(true);
    // A newer request exists: the stale response is discarded.
    expect(responseIsCurrent(2, 3, "openrouter", "openrouter")).toBe(false);
    // The resource changed: a late response for the old provider is discarded.
    expect(responseIsCurrent(3, 3, "openrouter", "anthropic")).toBe(false);
  });

  it("applies a suggested port only to the unchanged originating profile", () => {
    const original = profile();
    const identity = profileIdentity(original);
    const applied = applySuggestedPort(original, identity, 8080, 8081);
    expect(applied.port).toBe(8081);
    expect(applied.model).toBe(original.model);

    // A later manual edit (different port) is never overwritten.
    const edited = profile({ port: 9000 });
    expect(applySuggestedPort(edited, identity, 8080, 8081).port).toBe(9000);
    // A newly selected profile is never overwritten.
    const other = profile({ model: "C:/models/b.gguf" });
    expect(applySuggestedPort(other, identity, 8080, 8081)).toBe(other);

    // No-op cases return the same reference, so callers can compare identity.
    expect(applySuggestedPort(original, identity, 8080, 8080)).toBe(original);
  });

  it("labels the app-wide evidence run for every screen", () => {
    expect(evidenceRunLabel("benchmark")).toBe("Benchmark running");
    expect(evidenceRunLabel("quality")).toBe("Quality suite running");
  });

  it("derives profile identity from the fields a stale response must match", () => {
    const base = profile();
    expect(profileIdentity(base)).toBe(profileIdentity(profile()));
    expect(profileIdentity(base)).not.toBe(
      profileIdentity(profile({ name: "renamed" })),
    );
    expect(profileIdentity(base)).not.toBe(
      profileIdentity(profile({ model: "C:/models/b.gguf" })),
    );
    expect(profileIdentity(base)).not.toBe(
      profileIdentity(profile({ runtime: "C:/runtime/other.exe" })),
    );
  });
});

describe("FE-01 and FE-02 selection and run identity", () => {
  const modelOf = (id: string): never => ({ id, name: id }) as never;

  it("keeps a surviving selection, replaces a missing one, clears on failure", () => {
    const inventory = [modelOf("a"), modelOf("b")];
    // Same model still present: the selection is preserved, never reset to
    // the first entry (audit FE-01).
    expect(selectionAfterRescan(inventory, "b")).toBe("b");
    // Missing model: the replacement is the first remaining entry.
    expect(selectionAfterRescan(inventory, "gone")).toBe("a");
    // Empty inventory: explicit null clears selection and profile together.
    expect(selectionAfterRescan([], "a")).toBeNull();
    // Failed scan: null models also clears together.
    expect(selectionAfterRescan(null, "a")).toBeNull();
  });

  it("never silently substitutes another model for a missing selection", () => {
    const inventory = [modelOf("a")];
    expect(displayedSelection(inventory as never, "a")).toBe(inventory[0]);
    expect(displayedSelection(inventory as never, "missing")).toBeUndefined();
  });

  it("labels a tuning run or report from its immutable record", () => {
    expect(
      tuningRunSummary({ modelName: "Llama 3", provider: "openrouter", advisor: "claude", context: 8192 }),
    ).toBe("Llama 3 · openrouter · claude · 8,192 ctx");
  });
});

describe("parseExtraArgs (FE-08)", () => {
  it("keeps separate tokens that were typed one character at a time", () => {
    // The raw field used to trim and re-split on every keystroke, so typing
    // "--flash-attn " dropped the separating space and glued the next token.
    expect(parseExtraArgs("--flash-attn --ctx-size 4096")).toEqual([
      "--flash-attn",
      "--ctx-size",
      "4096",
    ]);
  });

  it("supports quoted values that contain spaces", () => {
    expect(parseExtraArgs('--lora "C:/my models/adapters/x.bin" --seed 7')).toEqual([
      "--lora",
      "C:/my models/adapters/x.bin",
      "--seed",
      "7",
    ]);
    expect(parseExtraArgs('--flag "a \\"quoted\\" value"')).toEqual([
      "--flag",
      'a "quoted" value',
    ]);
  });

  it("treats empty and whitespace-only drafts as no arguments", () => {
    expect(parseExtraArgs("")).toEqual([]);
    expect(parseExtraArgs("   \t ")).toEqual([]);
  });

  it("round-trips through formatExtraArgs", () => {
    const tokens = ["--flash-attn", "--lora", "C:/my models/x.bin", "--flag=a b"];
    const formatted = formatExtraArgs(tokens);
    expect(formatExtraArgs(parseExtraArgs(formatted))).toBe(formatted);
    expect(parseExtraArgs(formatted)).toEqual(tokens);
  });
});
