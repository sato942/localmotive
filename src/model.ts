export type Companion = {
  path: string;
  name: string;
  role: "mmproj" | "dspark" | "mtp" | "dflash" | "eagle3" | string;
  sizeBytes: number;
};

export type LogicalModel = {
  id: string;
  name: string;
  directory: string;
  firstShard: string;
  sizeBytes: number;
  shardCount: number;
  expectedShards: number;
  complete: boolean;
  quant: string;
  companions: Companion[];
};

export type LaunchProfile = {
  name: string;
  runtime: string;
  model: string;
  draftModel: string | null;
  mmproj: string | null;
  host: string;
  port: number;
  alias: string;
  context: number;
  parallel: number;
  gpuLayers: string;
  cpuMoe: number;
  cpuFfn: number;
  threads: number;
  threadsBatch: number;
  batch: number;
  ubatch: number;
  flashAttention: "auto" | "on" | "off";
  fit: boolean;
  fitTarget: string;
  fitCtx: number;
  kvOffload: boolean;
  cacheTypeK: string;
  cacheTypeV: string;
  loadMode: string;
  lazyMode: string;
  splitMode: string;
  tensorSplit: string;
  mainGpu: number;
  device: string;
  continuousBatching: boolean;
  cachePrompt: boolean;
  cacheReuse: number;
  cacheRam: number;
  contextCheckpoints: number;
  contextShift: boolean;
  warmup: boolean;
  sleepIdleSeconds: number;
  timeout: number;
  threadsHttp: number;
  ssePingInterval: number;
  metrics: boolean;
  slots: boolean;
  webUi: boolean;
  corsOrigins: string;
  apiKeyFile: string;
  sslKeyFile: string;
  sslCertFile: string;
  jinja: boolean;
  reasoning: string;
  reasoningEffort: string;
  reasoningBudget: number;
  reasoningPreserve: boolean;
  chatTemplateFile: string;
  temperature: number;
  topK: number;
  topP: number;
  minP: number;
  repeatPenalty: number;
  repeatLastN: number;
  seed: number;
  dryMultiplier: number;
  dryBase: number;
  specType: string;
  draftMax: number;
  draftMin: number;
  draftPMin: number;
  draftPSplit: number;
  draftGpuLayers: string;
  draftCacheTypeK: string;
  draftCacheTypeV: string;
  ngramMatch: number;
  ngramMin: number;
  ngramMax: number;
  ngramSizeN: number;
  ngramSizeM: number;
  ngramMinHits: number;
  mmprojOffload: boolean;
  mmprojDevice: string;
  imageMinTokens: number;
  imageMaxTokens: number;
  lora: string;
  loraScaled: string;
  overrideTensor: string;
  overrideKv: string;
  verbosity: number;
  logTimestamps: boolean;
  extraArgs: string[];
};

export type HardwareInfo = {
  architecture: string;
  gpuNames: string[];
  vendor: string;
  cudaMajor: number | null;
  driverVersion: string;
  detectionStatus: string;
  recommendation: string;
};

export type GithubAsset = {
  name: string;
  browserDownloadUrl: string;
  size: number;
  digest: string | null;
};

export type RuntimeOption = {
  id: string;
  label: string;
  backend: string;
  installKey: string;
  description: string;
  compatibility: string;
  asset: GithubAsset;
  companionAsset: GithubAsset | null;
  recommended: boolean;
};

export type RuntimeCatalog = {
  tag: string;
  publishedAt: string;
  options: RuntimeOption[];
};

export type InstalledRuntime = {
  tag: string;
  backend: string;
  runtimePath: string;
  installRoot: string;
  reused: boolean;
};

export type RuntimeIdentity = {
  path: string;
  backend: string;
  cudaMajor: number | null;
  tag: string | null;
  installKey: string | null;
  source: "manifest" | "dlls" | "none" | string;
};

export type ManagedRuntimeRecord = {
  tag: string;
  backend: string;
  installKey: string;
  runtimePath: string;
  installRoot: string;
};

export type RuntimeOptionState =
  | { kind: "install" }
  | { kind: "active" }
  | { kind: "update"; from: string }
  | { kind: "use"; runtimePath: string };

export function tagBuild(tag: string | null | undefined): number | null {
  if (!tag) return null;
  const digits = tag.trim().replace(/^b/i, "");
  return /^\d+$/.test(digits) ? Number(digits) : null;
}

/**
 * Whether the active executable is the same *variant* as a catalog option:
 * same backend, and for CUDA the same CUDA major version. The managed manifest
 * is authoritative when present; otherwise the DLL-derived identity is used.
 */
function activeMatchesVariant(option: RuntimeOption, active: RuntimeIdentity): boolean {
  if (active.source === "none") return false;
  if (active.installKey) return active.installKey === option.installKey;
  if (active.backend !== option.backend) return false;
  if (option.backend !== "cuda") return true;
  const optionMajor = tagBuild(option.installKey.replace(/^cuda-/, "").split(".")[0]);
  return active.cudaMajor !== null && optionMajor === active.cudaMajor;
}

/**
 * Decide what the action button beside a catalog option should do, given what is
 * already on disk and which executable is active.
 */
export function runtimeOptionState(
  option: RuntimeOption,
  catalogTag: string,
  managed: ManagedRuntimeRecord[],
  active: RuntimeIdentity | null,
  activeBuild: string | null,
): RuntimeOptionState {
  const releaseBuild = tagBuild(catalogTag);
  const currentBuild = tagBuild(activeBuild);

  if (active && activeMatchesVariant(option, active)) {
    if (releaseBuild !== null && currentBuild !== null) {
      if (currentBuild >= releaseBuild) return { kind: "active" };
      return { kind: "update", from: `b${currentBuild}` };
    }
    return { kind: "active" };
  }

  const installed = managed.find((record) => record.installKey === option.installKey && record.tag === catalogTag);
  if (installed) return { kind: "use", runtimePath: installed.runtimePath };
  return { kind: "install" };
}

export type RuntimeCapabilities = {
  path: string;
  version: string;
  build: string;
  commit: string;
  specTypes: string[];
  supportedFlags: string[];
  metrics: boolean;
  multimodal: boolean;
  fit: boolean;
};

export type ServerStatus = {
  running: boolean;
  pid: number | null;
  profileName: string | null;
  alias: string | null;
  port: number | null;
  command: string | null;
  logPath: string | null;
  startedAt: number | null;
  exitCode: number | null;
};

export type BenchmarkSummary = {
  samples: number[];
  meanTps: number;
  medianTps: number;
  minTps: number;
  maxTps: number;
  tokens: number;
  repeats: number;
};

export type GgufSummary = {
  architecture: string;
  name: string;
  sizeLabel: string;
  fileType: number | null;
  blockCount: number | null;
  contextLength: number | null;
  embeddingLength: number | null;
  headCount: number | null;
  headCountKv: number | null;
  expertCount: number | null;
  expertUsedCount: number | null;
  vocabSize: number | null;
  ropeFreqBase: number | null;
  tensorCount: number;
  kvCount: number;
};

export type CloudProvider = {
  id: string;
  label: string;
  baseUrl: string;
  keyPrefixHint: string;
  consoleUrl: string;
  supportsOauth: boolean;
  defaultModel: string;
  listsModels: boolean;
};

export type CredentialStatus = {
  provider: string;
  configured: boolean;
  masked: string;
};

export type CloudModel = { id: string; label: string };

export type TuningTrial = {
  index: number;
  changes: Record<string, unknown>;
  rationale: string;
  meanTps: number | null;
  medianTps: number | null;
  error: string | null;
  command: string;
};

export type TuningReport = {
  baselineTps: number | null;
  bestIndex: number | null;
  bestTps: number | null;
  bestProfile: LaunchProfile;
  trials: TuningTrial[];
  stoppedReason: string;
};

export type TuningProgress = {
  phase: "prepare" | "launch" | "measure" | "trial" | string;
  message: string;
  trial: TuningTrial | null;
};

/** Context lengths offered by the tuner; the model's native limit caps the list. */
export function contextChoices(nativeLimit: number | null): number[] {
  const ladder = [2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144];
  const capped = nativeLimit ? ladder.filter((value) => value <= nativeLimit) : ladder;
  if (nativeLimit && !capped.includes(nativeLimit) && nativeLimit >= 512) capped.push(nativeLimit);
  return capped.length ? capped : [2048];
}

export function describeChanges(changes: Record<string, unknown>): string {
  const entries = Object.entries(changes);
  if (!entries.length) return "baseline";
  return entries.map(([key, value]) => `${key}=${String(value)}`).join("  ");
}

export function slugAlias(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "")
    .slice(0, 64);
}

export function suggestedProfile(model: LogicalModel, runtime: string): LaunchProfile {
  const draft = model.companions.find((c) =>
    ["dspark", "mtp", "dflash", "eagle3"].includes(c.role),
  );
  const projector = model.companions.find((c) => c.role === "mmproj");
  const specMap: Record<string, string> = {
    dspark: "draft-dspark",
    mtp: "draft-mtp",
    dflash: "draft-dflash",
    eagle3: "draft-eagle3",
  };
  return {
    name: `${model.name} / ${draft ? draft.role.toUpperCase() : "Baseline"}`,
    runtime,
    model: model.firstShard,
    draftModel: draft?.path ?? null,
    mmproj: projector?.path ?? null,
    host: "127.0.0.1",
    port: 8080,
    alias: slugAlias(model.name),
    context: 8192,
    parallel: 1,
    gpuLayers: "all",
    cpuMoe: 0,
    cpuFfn: 0,
    threads: -1,
    threadsBatch: -1,
    batch: 2048,
    ubatch: 512,
    flashAttention: "auto",
    fit: true,
    fitTarget: "1024",
    fitCtx: 4096,
    kvOffload: true,
    cacheTypeK: "f16",
    cacheTypeV: "f16",
    loadMode: "auto",
    lazyMode: "auto",
    splitMode: "layer",
    tensorSplit: "",
    mainGpu: 0,
    device: "",
    continuousBatching: true,
    cachePrompt: true,
    cacheReuse: 0,
    cacheRam: 8192,
    contextCheckpoints: 32,
    contextShift: false,
    warmup: true,
    sleepIdleSeconds: -1,
    timeout: 3600,
    threadsHttp: -1,
    ssePingInterval: 30,
    metrics: true,
    slots: true,
    webUi: true,
    corsOrigins: "localhost",
    apiKeyFile: "",
    sslKeyFile: "",
    sslCertFile: "",
    jinja: true,
    reasoning: "auto",
    reasoningEffort: "default",
    reasoningBudget: -1,
    reasoningPreserve: false,
    chatTemplateFile: "",
    temperature: 0.8,
    topK: 40,
    topP: 0.95,
    minP: 0.05,
    repeatPenalty: 1.0,
    repeatLastN: 64,
    seed: -1,
    dryMultiplier: 0,
    dryBase: 1.75,
    specType: draft ? specMap[draft.role] : "none",
    draftMax: draft?.role === "dspark" ? 3 : 5,
    draftMin: 0,
    draftPMin: 0,
    draftPSplit: 0.1,
    draftGpuLayers: "auto",
    draftCacheTypeK: "f16",
    draftCacheTypeV: "f16",
    ngramMatch: 24,
    ngramMin: 48,
    ngramMax: 64,
    ngramSizeN: 12,
    ngramSizeM: 48,
    ngramMinHits: 1,
    mmprojOffload: true,
    mmprojDevice: "auto",
    imageMinTokens: 0,
    imageMaxTokens: 0,
    lora: "",
    loraScaled: "",
    overrideTensor: "",
    overrideKv: "",
    verbosity: 3,
    logTimestamps: true,
    extraArgs: [],
  };
}

export function normalizeProfile(
  stored: Partial<LaunchProfile> & { flashAttention?: LaunchProfile["flashAttention"] | boolean },
  model: LogicalModel,
  runtime: string,
): LaunchProfile {
  const merged = { ...suggestedProfile(model, runtime), ...stored } as LaunchProfile & {
    flashAttention: LaunchProfile["flashAttention"] | boolean;
  };
  return {
    ...merged,
    // The runtime in use wins over whatever was current when this profile was
    // saved, so installing an update does not silently re-pin the old exe.
    runtime: runtime || merged.runtime,
    flashAttention:
      typeof merged.flashAttention === "boolean"
        ? merged.flashAttention
          ? "on"
          : "off"
        : merged.flashAttention,
    extraArgs: (merged.extraArgs ?? []).filter((arg) => arg !== "--jinja"),
  };
}

export type AboutInfo = {
  name: string;
  version: string;
  tauriVersion: string;
  identifier: string;
  os: string;
  runtimeRoot: string;
  logDir: string;
  repository: string;
  license: string;
};

export function bytesLabel(bytes: number): string {
  if (bytes >= 1024 ** 4) return `${(bytes / 1024 ** 4).toFixed(2)} TiB`;
  if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(2)} GiB`;
  return `${(bytes / 1024 ** 2).toFixed(0)} MiB`;
}

// ---------------------------------------------------------------------------
// HF catalog
// ---------------------------------------------------------------------------

export type CatalogFile = {
  quant: string;
  filename: string;
  sizeBytes: number;
  sha256: string;
  revision?: string;
};

export function catalogRevision(file: CatalogFile): string {
  return file.revision || "main";
}

export function keepLatestRequest(sequence: number, current: number): boolean {
  return sequence === current;
}

export function retainOrDisposeListener(disposed: boolean, stop: () => void): (() => void) | null {
  if (disposed) {
    stop();
    return null;
  }
  return stop;
}

export type CatalogModel = {
  id: string;
  repo: string;
  family: string;
  parameters: string;
  publisher: string;
  summary: string;
  tags: string[];
  gated: boolean;
  downloads: number;
  likes: number;
  files: CatalogFile[];
};

export type Catalog = {
  schemaVersion: number;
  updated: string;
  source: string;
  note: string;
  models: CatalogModel[];
};

export type CatalogSort = "downloads" | "likes" | "name" | "size";

export type CatalogQuery = {
  text: string;
  tag: string;
  quant: string;
  maxBytes: number;
  hideGated: boolean;
  sort: CatalogSort;
};

export type CatalogSnapshot = {
  catalog: Catalog;
  origin: "network" | "not-modified" | "cache" | "bundled";
  fetchedAt: string;
  url: string;
};

export type TokenStatus = {
  configured: boolean;
  masked: string;
};

export type DownloadEvent = {
  key: string;
  downloaded: number;
  total: number;
  bytesPerSecond: number;
  state: "downloading" | "verifying" | "done" | "error";
  message: string;
  path: string;
};

/** Key identifying one downloadable file across the catalog. */
export function downloadKey(repo: string, filename: string): string {
  return `${repo}/${filename}`;
}

/**
 * What the catalog origin means for the user. A cached list may be stale, and
 * saying so is more honest than showing it as if it were live.
 */
export function originLabel(origin: CatalogSnapshot["origin"]): {
  label: string;
  tone: "ok" | "warn";
} {
  switch (origin) {
    case "network":
      return { label: "LIVE · JUST FETCHED", tone: "ok" };
    case "not-modified":
      return { label: "LIVE · UNCHANGED", tone: "ok" };
    case "cache":
      return { label: "OFFLINE · SHOWING LAST SAVED LIST", tone: "warn" };
    default:
      return { label: "BUILT-IN LIST", tone: "warn" };
  }
}

/**
 * Whether a download may start, and why not when it may not. Kept pure so the
 * rule is tested rather than scattered through JSX.
 */
export function downloadReadiness(input: {
  destination: string;
  running: boolean;
  alreadyOnDisk: boolean;
  gated: boolean;
  hasToken: boolean;
}): { canStart: boolean; reason: string } {
  if (!input.destination.trim()) {
    return { canStart: false, reason: "Choose a model folder first." };
  }
  if (input.running) {
    return { canStart: false, reason: "This file is already downloading." };
  }
  if (input.gated && !input.hasToken) {
    return {
      canStart: false,
      reason: "This repository is gated. Add a Hugging Face token below.",
    };
  }
  return { canStart: true, reason: "" };
}

/** Percentage complete, clamped so a bad total can never break the bar. */
export function downloadPercent(downloaded: number, total: number): number {
  if (!Number.isFinite(downloaded) || !Number.isFinite(total) || total <= 0) return 0;
  return Math.max(0, Math.min(100, (downloaded / total) * 100));
}

/** "4m 20s" / "1h 05m" / "" when the rate is not yet known. */
export function etaLabel(downloaded: number, total: number, bytesPerSecond: number): string {
  if (bytesPerSecond <= 0 || total <= 0 || downloaded >= total) return "";
  const seconds = Math.round((total - downloaded) / bytesPerSecond);
  if (seconds < 60) return `${seconds}s left`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${String(seconds % 60).padStart(2, "0")}s left`;
  return `${Math.floor(seconds / 3600)}h ${String(Math.floor((seconds % 3600) / 60)).padStart(2, "0")}m left`;
}

export function rateLabel(bytesPerSecond: number): string {
  if (bytesPerSecond <= 0) return "";
  return `${bytesLabel(bytesPerSecond)}/s`;
}
