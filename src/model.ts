export type EvidenceLevel =
  | "exact"
  | "observed"
  | "derived"
  | "userOverride"
  | "catalog"
  | "heuristic"
  | "unknown";

export type EvidenceSourceKind =
  | "fileSystem"
  | "ggufMetadata"
  | "runtime"
  | "windowsApi"
  | "nvidiaSmi"
  | "cim"
  | "user"
  | "catalog"
  | "benchmark"
  | "calculation"
  | "policy"
  | "import"
  | "unknown";

export type EvidenceSource = {
  kind: EvidenceSourceKind;
  detail: string;
};

export type Evidence<T> = {
  value: T | null;
  level: EvidenceLevel;
  source: EvidenceSource;
  observedAtMs: number;
  notes: string[];
};

export type ErrorCode =
  | "unsupportedSchema"
  | "missingValue"
  | "unexpectedValue"
  | "invalidRange"
  | "limitExceeded"
  | "nonFinite"
  | "invalidDigest"
  | "emptyIdentity"
  | "evidenceUpgrade"
  | "inconsistentEvidence";

export type DomainError = {
  code: ErrorCode;
  field: string;
  message: string;
};

export type FitClass =
  | "unknown"
  | "estimated"
  | "preflightLikely"
  | "preflightTight"
  | "requiresOffload"
  | "launchValidated"
  | "measured"
  | "failed";

export type ExecutionPath =
  | "unknown"
  | "fullGpu"
  | "layerOffload"
  | "cpu"
  | "unifiedMemory"
  | "multiGpu"
  | "unsupported";

export type CacheMode = "cold" | "warm";

export type Workload = {
  id: string;
  promptTokens: number;
  generationTokens: number;
  warmups: number;
  trials: number;
  seed: number | null;
  concurrency: number;
  stream: boolean;
  cacheMode: CacheMode;
  timeoutMs: number;
};

export type RuntimeFact = {
  path: string;
  version: string;
  build: string;
  executableSha256: string | null;
  helpSha256: string;
  backend: string;
};

export type FileFact = {
  path: string;
  bytes: number;
  sha256: string | null;
};

export type ModelFact = {
  logicalId: string;
  architecture: string;
  shards: FileFact[];
  companions: FileFact[];
  ggufHeaderSha256: string;
};

export type HardwareFact = {
  adapterId: string;
  name: string;
  vendor: string;
  driver: string | null;
  backend: string | null;
  dedicatedBytes: Evidence<number>;
  sharedBytes: Evidence<number>;
  budgetBytes: Evidence<number>;
  currentUsageBytes: Evidence<number>;
};

export type LaunchFact = {
  requestedContext: number;
  effectiveContext: Evidence<number>;
  parallel: number;
  gpuLayers: string;
  batch: number;
  ubatch: number;
  cacheTypeK: string;
  cacheTypeV: string;
  splitMode: string;
  tensorSplit: string;
  mainGpu: number;
  commandArgs: string[];
  rejectedFlags: string[];
};

export type BenchmarkObservation = {
  trial: number;
  startedAtMs: number;
  durationMs: number;
  promptTokens: number;
  generatedTokens: number;
  prefillTps: number | null;
  decodeTps: number | null;
  firstTokenMs: number | null;
  derivedTtftMs: number | null;
  peakProcessRssBytes: Evidence<number>;
  outcome: AttemptOutcome;
  error: string | null;
};

export type AttemptOutcome = "succeeded" | "failed" | "timedOut" | "cancelled";

export type WarmupObservation = {
  warmup: number;
  startedAtMs: number;
  durationMs: number;
  outcome: AttemptOutcome;
  error: string | null;
};

export type BenchmarkManifest = {
  schema: number;
  harnessVersion: string;
  compatibilityKey: string | null;
  runtime: RuntimeFact | null;
  hardware: HardwareFact[];
  model: ModelFact | null;
  launch: LaunchFact | null;
  workload: Workload;
  warmups: WarmupObservation[];
  observations: BenchmarkObservation[];
  terminalOutcome: AttemptOutcome | null;
};

export type MetricStats = {
  count: number;
  mean: number;
  median: number;
  p50: number;
  p95: number;
  min: number;
  max: number;
  standardDeviation: number;
};

export type BenchmarkSummaryV2 = {
  resultClass: FitClass;
  successfulTrials: number;
  failedTrials: number;
  prefillTps: MetricStats | null;
  decodeTps: MetricStats;
  firstTokenMs: MetricStats | null;
  derivedTtftMs: MetricStats | null;
  failures: string[];
};

export type BenchmarkRunResult = {
  manifest: BenchmarkManifest;
  summary: BenchmarkSummaryV2 | null;
  manifestPath: string;
  compatibilityKey: string;
  resultClass: FitClass;
  failure: string | null;
};

export type QualityStatus = "notRun" | "passed" | "failed" | "error";

export type QualityCaseResult = {
  caseId: string;
  status: QualityStatus;
  detail: string;
};

export type QualitySuiteResult = {
  suiteId: string;
  seed: number;
  observedAtMs: number | null;
  modelLogicalId: string | null;
  runtimeSha256: string | null;
  // Full identity captured with the suite (audit MT-09). Optional so results
  // stored by older builds still load.
  modelContentSha256?: string | null;
  compatibilityKey?: string | null;
  suiteVersion?: string;
  casesPlanned?: number;
  status: QualityStatus;
  cases: QualityCaseResult[];
};

export type CandidateEvidence = {
  id: string;
  resultClass: FitClass;
  decodeTps: number | null;
  prefillTps: number | null;
  p95LatencyMs: number | null;
  peakMemoryBytes: number | null;
  qualityPassRate: number | null;
  storageBytes: number | null;
};

export type RecommendationConstraints = {
  minDecodeTps: number | null;
  maxP95LatencyMs: number | null;
  maxPeakMemoryBytes: number | null;
  minQualityPassRate: number | null;
  maxStorageBytes: number | null;
  requireMeasured: boolean;
};

export type ObjectiveWeights = {
  decodeTps: number;
  prefillTps: number;
  latency: number;
  memory: number;
  quality: number;
  storage: number;
};

export type ScoreComponent = {
  objective: string;
  normalized: number;
  weight: number;
  contribution: number;
};

export type RankedCandidate = {
  id: string;
  feasible: boolean;
  violations: string[];
  pareto: boolean;
  dominatedBy: string[];
  // Added with the complete-set dominance policy (audit MT-10): the fraction
  // of configured weight the candidate measured, and whether the bounded
  // dominator list hides further entries. Optional for stored results.
  dominatorsTruncated?: boolean;
  evidenceCoverage?: number;
  preferenceScore: number | null;
  scoreComponents: ScoreComponent[];
};

export type CalibrationAnchor = {
  compatibilityKey: string;
  estimatedValue: number;
  measuredValue: number;
  observedAtMs: number;
  // Identities added with the versioned execution snapshot (audit MT-08):
  // the persisted run behind the sample and the estimator that produced the
  // estimate. Empty strings appear on legacy records.
  sourceRunId?: string;
  estimator?: string;
  snapshotSchemaVersion?: string;
  unknownIdentities?: string[];
};

export type CalibrationModel = {
  compatibilityKey: string;
  factor: number;
  residualStandardDeviation: number;
  anchorCount: number;
  createdAtMs: number;
  expiresAtMs: number;
  // Provenance and freshness added with the unified validator (audit MT-14).
  sourceEvidenceAtMs?: number;
  estimator?: string;
  sourceRunIds?: string[];
};

export type CalibrationRecords = {
  anchors: CalibrationAnchor[];
  models: CalibrationModel[];
};

export type CalibratedEstimate = {
  value: number;
  lowerBound: number;
  upperBound: number;
  evidenceLevel: EvidenceLevel;
  compatibilityKey: string;
  expiresAtMs: number;
};

export type ExternalEvidenceState = "pending" | "verified" | "flagged" | "rejected";

export type ExternalObservation = {
  metric: string;
  value: number;
  unit: string;
  observedAtMs: number;
};

export type ExternalEvidenceBundle = {
  schema: number;
  source: string;
  compatibilityKey: string;
  state: ExternalEvidenceState;
  records: ExternalObservation[];
};

export type ShareFile = {
  bytes: number;
  sha256: string;
};

export type ShareRuntime = {
  version: string;
  build: string;
  backend: string;
  executableSha256: string;
  helpSha256: string;
};

export type ShareModel = {
  logicalId: string;
  architecture: string;
  ggufHeaderSha256: string;
  shards: ShareFile[];
  companions: ShareFile[];
};

export type ShareHardware = {
  vendor: string;
  backend: string | null;
  driver: string | null;
  dedicatedBytes: Evidence<number>;
  sharedBytes: Evidence<number>;
  budgetBytes: Evidence<number>;
};

export type ShareLaunch = {
  requestedContext: number;
  effectiveContext: Evidence<number>;
  parallel: number;
  gpuLayers: string;
  batch: number;
  ubatch: number;
  cacheTypeK: string;
  cacheTypeV: string;
  splitMode: string;
  tensorSplit: string;
  mainGpu: number;
  rejectedFlags: string[];
};

export type ShareObservation = {
  trial: number;
  durationMs: number;
  promptTokens: number;
  generatedTokens: number;
  prefillTps: number | null;
  decodeTps: number | null;
  firstTokenMs: number | null;
  derivedTtftMs: number | null;
  peakProcessRssBytes: Evidence<number>;
  outcome: AttemptOutcome;
  succeeded: boolean;
};

export type ShareQuality = {
  suiteId: string;
  seed: number;
  observedAtMs: number;
  modelLogicalId: string;
  runtimeSha256: string;
  status: QualityStatus;
  cases: Array<{ caseId: string; status: QualityStatus }>;
};

export type PrivacyReview = {
  omittedFields: string[];
  requiresUserConfirmation: boolean;
};

export type ShareBundle = {
  schema: number;
  createdAtMs: number;
  compatibilityKey: string;
  runtime: ShareRuntime;
  model: ShareModel;
  hardware: ShareHardware[];
  launch: ShareLaunch;
  workload: Workload;
  warmupOutcomes: AttemptOutcome[];
  observations: ShareObservation[];
  terminalOutcome: AttemptOutcome | null;
  summary: BenchmarkSummaryV2 | null;
  quality: ShareQuality | null;
  privacyReview: PrivacyReview;
};

export function qualityPassRate(result: QualitySuiteResult | null): number | null {
  if (!result || result.status === "notRun" || result.status === "error") return null;
  const scored = result.cases.filter((item) => item.status === "passed" || item.status === "failed");
  if (!scored.length) return null;
  return scored.filter((item) => item.status === "passed").length / scored.length;
}

export type CalibrationState =
  | "compatible"
  | "expired"
  | "incompatible"
  | "scheduled"
  | "staleEvidence"
  | "unavailable";

/** Mirrors the backend evaluation in calibration_model_state (audit MT-14):
 * exact expiry, a creation time ahead of the clock is not yet applicable,
 * and freshness is measured from the source-run evidence, not rebuild time. */
export function calibrationState(
  model: CalibrationModel | null,
  compatibilityKey: string,
  nowMs: number,
): CalibrationState {
  if (!model) return "unavailable";
  if (model.compatibilityKey !== compatibilityKey) return "incompatible";
  if (nowMs < model.createdAtMs) return "scheduled";
  if (nowMs >= model.expiresAtMs) return "expired";
  const evidence = model.sourceEvidenceAtMs ?? 0;
  const maxEvidenceAgeMs = 90 * 24 * 60 * 60 * 1_000;
  if (evidence > 0 && nowMs - evidence > maxEvidenceAgeMs) return "staleEvidence";
  return "compatible";
}

export function derivedEvidence<T>(
  value: T,
  requiredLevels: EvidenceLevel[],
  source: EvidenceSource,
  observedAtMs: number,
  notes: string[],
): Evidence<T> {
  if (requiredLevels.includes("unknown")) {
    return {
      value: null,
      level: "unknown",
      source,
      observedAtMs,
      notes: [...notes, "At least one required input is unknown."],
    };
  }
  return { value, level: "derived", source, observedAtMs, notes: [...notes] };
}

/// Parse a raw extra-arguments field into self-contained tokens (FE-08).
/// Double quotes group values that contain spaces; a backslash escapes a
/// quote inside a quoted value. The field is intentionally whitespace
/// separated, matching what llama-server receives from the command line.
export function parseExtraArgs(raw: string): string[] {
  const tokens: string[] = [];
  let current = "";
  let quoted = false;
  let started = false;
  for (let index = 0; index < raw.length; index += 1) {
    const character = raw[index];
    if (character === "\\" && quoted && index + 1 < raw.length) {
      current += raw[index + 1];
      index += 1;
      continue;
    }
    if (character === '"') {
      quoted = !quoted;
      started = true;
      continue;
    }
    if (!quoted && /\s/.test(character)) {
      if (started) {
        tokens.push(current);
        current = "";
        started = false;
      }
      continue;
    }
    current += character;
    started = true;
  }
  if (started) {
    tokens.push(current);
  }
  return tokens.filter((token) => token.length > 0);
}

/// Render tokens back into the raw field, quoting values that contain
/// spaces so a round trip through `parseExtraArgs` is stable (FE-08).
export function formatExtraArgs(tokens: readonly string[]): string {
  return tokens
    .map((token) =>
      /\s/.test(token) || token.includes('"')
        ? `"${token.replace(/"/g, '\\"')}"`
        : token,
    )
    .join(" ");
}

/// Parse persisted JSON without throwing (FE-09). Malformed text returns
/// `undefined` so a caller can quarantine the record instead of crashing
/// the render tree.
/// Validate a persisted tuning report (FE-09). A record that is not the
/// expected shape returns `undefined` so the caller can quarantine it
/// instead of letting `.reduce`/`.map` throw during render.
export function normalizeTuningReport(stored: unknown): TuningReport | undefined {
  if (typeof stored !== "object" || stored === null) return undefined;
  const report = stored as Record<string, unknown>;
  if (!Array.isArray(report.trials)) return undefined;
  const numberOrNull = (value: unknown): number | null =>
    typeof value === "number" && Number.isFinite(value) ? value : null;
  const stringOrNull = (value: unknown): string | null => (typeof value === "string" ? value : null);
  const trials: TuningTrial[] = [];
  for (const candidate of report.trials) {
    if (typeof candidate !== "object" || candidate === null) return undefined;
    const trial = candidate as Record<string, unknown>;
    const changes =
      typeof trial.changes === "object" && trial.changes !== null
        ? (trial.changes as Record<string, unknown>)
        : {};
    trials.push({
      index: typeof trial.index === "number" ? trial.index : trials.length,
      changes,
      rationale: typeof trial.rationale === "string" ? trial.rationale : "",
      meanTps: numberOrNull(trial.meanTps),
      medianTps: numberOrNull(trial.medianTps),
      error: stringOrNull(trial.error),
      command: typeof trial.command === "string" ? trial.command : "",
      effectiveContext: numberOrNull(trial.effectiveContext),
      stdDev: numberOrNull(trial.stdDev),
    });
  }
  if (report.bestIndex !== null && typeof report.bestIndex !== "number") return undefined;
  if (typeof report.bestProfile !== "object" || report.bestProfile === null) return undefined;
  return {
    baselineTps: numberOrNull(report.baselineTps),
    bestIndex: numberOrNull(report.bestIndex),
    bestTps: numberOrNull(report.bestTps),
    bestProfile: report.bestProfile as TuningReport["bestProfile"],
    trials,
    stoppedReason: typeof report.stoppedReason === "string" ? report.stoppedReason : "",
    objective: typeof report.objective === "string" ? report.objective : undefined,
    requiredEffectiveContext: numberOrNull(report.requiredEffectiveContext) ?? undefined,
    finalVerification: report.finalVerification as TuningReport["finalVerification"],
  };
}

/// Tone for an evidence result, from its actual status (audit FE-15):
/// measured/complete states are green, unknown or pending states amber,
/// blocked or failed states red. Colour is a second channel; the words
/// remain.
export function evidenceTone(value: string | null | undefined): "ok" | "pending" | "bad" {
  if (value === null || value === undefined || value.trim() === "") return "pending";
  const normalized = value.trim().toLowerCase();
  if (normalized === "unknown" || normalized === "notmeasured" || normalized === "not measured" || normalized === "pending") {
    return "pending";
  }
  if (normalized.includes("blocked") || normalized.includes("failed") || normalized.includes("rejected") || normalized.includes("cancelled") || normalized.includes("error")) {
    return "bad";
  }
  return "ok";
}

export function safeJsonParse<T>(raw: string): T | undefined {
  try {
    return JSON.parse(raw) as T;
  } catch {
    return undefined;
  }
}

export function defaultWorkload(): Workload {
  return {
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
}

export function validateWorkload(workload: Workload): DomainError[] {
  const errors: DomainError[] = [];
  const identity = workload.id.trim();
  if (!identity) {
    errors.push({
      code: "emptyIdentity",
      field: "workload.id",
      message: "Workload identity is required",
    });
  }
  // Rust deserializes these fields into integer types (u16/u32/u64), so the
  // validator enforces whole numbers as well as the ranges (audit FE-17 I3).
  const bounded = (field: string, value: number, minimum: number, maximum: number) => {
    if (!Number.isFinite(value) || !Number.isInteger(value)) {
      errors.push({
        code: "invalidRange",
        field,
        message: "Value must be a whole number",
      });
    } else if (value < minimum) {
      errors.push({ code: "invalidRange", field, message: `Value must be at least ${minimum}` });
    } else if (value > maximum) {
      errors.push({ code: "limitExceeded", field, message: `Value cannot exceed ${maximum}` });
    }
  };
  bounded("workload.promptTokens", workload.promptTokens, 1, 1_048_576);
  bounded("workload.generationTokens", workload.generationTokens, 1, 65_536);
  bounded("workload.warmups", workload.warmups, 0, 10);
  bounded("workload.trials", workload.trials, 1, 100);
  bounded("workload.concurrency", workload.concurrency, 1, 64);
  bounded("workload.timeoutMs", workload.timeoutMs, 1_000, 3_600_000);
  return errors;
}

export type Companion = {
  path: string;
  name: string;
  role: "mmproj" | "dspark" | "mtp" | "dflash" | "eagle3" | string;
  sizeBytes: number;
};

export type ArtifactFileFact = {
  path: string;
  name: string;
  sizeBytes: number;
  sha256: string | null;
  headerSha256: string | null;
  shardIndex: number | null;
  expectedShards: number | null;
};

export type ArtifactProblemCode =
  | "missingShard"
  | "duplicateShard"
  | "conflictingHeader"
  | "unreadableHeader";

export type ArtifactProblem = {
  code: ArtifactProblemCode;
  message: string;
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
  shards: ArtifactFileFact[];
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

export type MemoryMetric =
  | "dedicated"
  | "shared"
  | "budget"
  | "currentUsage"
  | "availableBudget"
  | "availableForReservation";

export type CapacityObservation = {
  metric: MemoryMetric;
  evidence: Evidence<number>;
};

export type SystemMemoryInfo = {
  totalPhysicalBytes: Evidence<number>;
  availablePhysicalBytes: Evidence<number>;
  memoryLoadPercent: Evidence<number>;
};

export type GpuAdapterInfo = {
  adapterId: string;
  compatibilityId: string;
  name: string;
  vendor: string;
  driver: Evidence<string>;
  backend: Evidence<string>;
  dedicatedBytes: Evidence<number>;
  sharedBytes: Evidence<number>;
  budgetBytes: Evidence<number>;
  currentUsageBytes: Evidence<number>;
  availableBudgetBytes: Evidence<number>;
  reservationBytes: Evidence<number>;
  availableForReservationBytes: Evidence<number>;
  capacityObservations: CapacityObservation[];
  // The stable physical identity recorded when a trustworthy join exists.
  physicalId?: Evidence<string> | null;
};

export type HardwareOverride = {
  adapterId: string;
  dedicatedBytes: number | null;
  sharedBytes: number | null;
  note: string;
};

export function manualGpuOverride(
  adapterId: string,
  dedicatedGiB: string,
  note: string,
): HardwareOverride | null {
  const gib = Number(dedicatedGiB);
  const id = adapterId.trim();
  if (!id || id.length > 256 || !Number.isFinite(gib) || gib <= 0 || gib > 1024 || note.length > 1024) {
    return null;
  }
  return {
    adapterId: id,
    dedicatedBytes: Math.round(gib * 1024 ** 3),
    sharedBytes: null,
    note: note.trim(),
  };
}

export type HardwareInfo = {
  architecture: string;
  gpuNames: string[];
  vendor: string;
  cudaMajor: number | null;
  driverVersion: string;
  detectionStatus: string;
  recommendation: string;
  systemMemory: SystemMemoryInfo;
  adapters: GpuAdapterInfo[];
  manualOverrides: HardwareOverride[];
  // NVIDIA telemetry without a trustworthy physical mapping (audit RT-09).
  unassignedNvidia?: Array<{
    name: string;
    uuid: string;
    pciBusId: string;
    driver: string;
    totalBytes: number | null;
    usedBytes: number | null;
    reason: string;
  }>;
};

export function conflictingCapacityMetrics(adapter: GpuAdapterInfo): MemoryMetric[] {
  const metricOrder: MemoryMetric[] = [
    "dedicated",
    "shared",
    "budget",
    "currentUsage",
    "availableBudget",
    "availableForReservation",
  ];
  return metricOrder.filter((metric) => {
    const values = adapter.capacityObservations
      .filter((item) => item.metric === metric && item.evidence.value !== null)
      .map((item) => item.evidence.value as number);
    return new Set(values).size > 1;
  });
}

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

export type RuntimeInstallRequest = {
  installKey: string;
  adapterId: string | null;
};

export type ManagedHealthRequest = RuntimeInstallRequest;

export type HealthModelProgress = {
  installKey: string;
  downloaded: number;
  total: number;
};

export type RuntimeInstallProgress = {
  installKey: string;
  assetName: string;
  downloaded: number;
  total: number;
};

export function runtimeInstallRequest(
  option: Pick<RuntimeOption, "backend" | "installKey">,
  selectedAdapterId: string,
): RuntimeInstallRequest {
  if (option.backend === "cpu") {
    return { installKey: option.installKey, adapterId: null };
  }
  const adapterId = selectedAdapterId.trim();
  if (!adapterId || adapterId.length > 256) {
    throw new Error("Select one detected GPU adapter before installing this runtime.");
  }
  return { installKey: option.installKey, adapterId };
}

export function managedHealthRequest(
  option: Pick<RuntimeOption, "backend" | "installKey">,
  selectedAdapterId: string,
): ManagedHealthRequest {
  return runtimeInstallRequest(option, selectedAdapterId);
}

export type RuntimeCatalog = {
  tag: string;
  publishedAt: string;
  options: RuntimeOption[];
  availability: Array<{
    installKey: string;
    backend: string;
    status: "available" | "blocked" | "dormant";
    reason: string;
    blockingJobs: string[];
    evidenceUrls: string[];
  }>;
  origin: "network" | "cache";
  warning: string | null;
  recommendationReason: string;
};

export type RuntimeCatalogError = {
  kind: "busy" | "timeout" | "rate_limited" | "http" | "body_too_large" | "invalid_response" | "trust_failure" | "cache_failure";
  message: string;
  retryAfterSeconds: number | null;
};

const runtimeCatalogErrorKinds = new Set<RuntimeCatalogError["kind"]>([
  "busy",
  "timeout",
  "rate_limited",
  "http",
  "body_too_large",
  "invalid_response",
  "trust_failure",
  "cache_failure",
]);

export function runtimeCatalogErrorFromUnknown(value: unknown): RuntimeCatalogError {
  if (typeof value === "string") {
    try {
      return runtimeCatalogErrorFromUnknown(JSON.parse(value));
    } catch {
      return { kind: "invalid_response", message: value, retryAfterSeconds: null };
    }
  }
  if (value && typeof value === "object") {
    const candidate = value as Partial<RuntimeCatalogError>;
    if (
      typeof candidate.kind === "string"
      && runtimeCatalogErrorKinds.has(candidate.kind as RuntimeCatalogError["kind"])
      && typeof candidate.message === "string"
      && candidate.message.trim()
      && (candidate.retryAfterSeconds === null
        || (Number.isInteger(candidate.retryAfterSeconds) && (candidate.retryAfterSeconds ?? -1) >= 0))
    ) {
      return {
        kind: candidate.kind as RuntimeCatalogError["kind"],
        message: candidate.message.trim(),
        retryAfterSeconds: candidate.retryAfterSeconds ?? null,
      };
    }
  }
  return {
    kind: "invalid_response",
    message: "The runtime catalog request failed without a structured error.",
    retryAfterSeconds: null,
  };
}

export type RuntimeCatalogViewState =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "ready"; catalog: RuntimeCatalog }
  | { kind: "empty"; catalog: RuntimeCatalog }
  | { kind: "error"; error: RuntimeCatalogError };

export function runtimeCatalogViewState(input: {
  loading: boolean;
  catalog: RuntimeCatalog | null;
  error: RuntimeCatalogError | null;
}): RuntimeCatalogViewState {
  if (input.loading) return { kind: "loading" };
  if (input.error) return { kind: "error", error: input.error };
  if (input.catalog) {
    return input.catalog.options.length > 0
      ? { kind: "ready", catalog: input.catalog }
      : { kind: "empty", catalog: input.catalog };
  }
  return { kind: "idle" };
}

export type InstalledRuntime = {
  tag: string;
  backend: string;
  runtimePath: string;
  installRoot: string;
  reused: boolean;
};

export type HealthStage =
  | "device_enumeration"
  | "backend_operations"
  | "pinned_model_load"
  | "loopback_server_health"
  | "deterministic_completion"
  | "cancellation"
  | "process_and_temporary_file_cleanup";

export type HealthFailureReason =
  | "timeout"
  | "spawn"
  | "nonzero_exit"
  | "output_limit"
  | "malformed_output"
  | "mismatch"
  | "cancelled"
  | "trust_failure";

export type HealthStageResult = {
  stage: HealthStage;
  status: "PASS" | "FAIL" | "SKIPPED";
  durationMs: number;
  failureReason: HealthFailureReason | null;
  detail: string;
  completion: {
    temperature: number;
    requestedTokens: number;
    observedTokens: number;
    expectedOutputSha256: string;
    observedOutputSha256: string;
  } | null;
};

export type ManagedHealthResult = {
  runtimeId: string;
  modelSha256: string;
  adapterId: string | null;
  startedAt: number;
  finishedAt: number;
  passed: boolean;
  stages: HealthStageResult[];
};

export type ManagedHealthOutcome = "passed" | "failed" | "cancelled";

export function managedHealthOutcome(result: ManagedHealthResult): ManagedHealthOutcome {
  if (result.passed) return "passed";
  return result.stages.some((stage) => stage.failureReason === "cancelled")
    ? "cancelled"
    : "failed";
}

export type RuntimeIdentity = {
  path: string;
  backend: string;
  cudaMajor: number | null;
  tag: string | null;
  installKey: string | null;
  source: "manifest" | "dlls" | "none" | string;
  managedVerified: boolean;
};

export type ManagedRuntimeRecord = {
  tag: string;
  backend: string;
  installKey: string;
  runtimePath: string;
  installRoot: string;
  /** Discovery never asserts verified content; true only after a real
   *  content verification (audit RT-06). */
  contentVerified: boolean;
};

export type RuntimeSetupResponse = {
  hardware: HardwareInfo;
  catalog: RuntimeCatalog | null;
  catalogError: RuntimeCatalogError | null;
  runtimeRoot: string;
  managedRuntimes: ManagedRuntimeRecord[];
};

export type RuntimeOptionState =
  | { kind: "install" }
  | { kind: "active" }
  | { kind: "update"; from: string }
  | { kind: "mismatch"; detail: string }
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
 * A `manifest-dll-mismatch` identity never matches: the installed files
 * disagree with the install record and the caller must reinstall.
 */
function activeMatchesVariant(option: RuntimeOption, active: RuntimeIdentity): boolean {
  if (active.source === "none") return false;
  if (active.source === "manifest-dll-mismatch") return false;
  if (active.backend === "mismatch") return false;
  if (active.installKey) return active.installKey === option.installKey;
  if (active.backend !== option.backend) return false;
  if (option.backend !== "cuda") return true;
  const optionMajor = tagBuild(option.installKey.replace(/^cuda-/, "").split(".")[0]);
  return active.cudaMajor !== null && optionMajor === active.cudaMajor;
}

/**
 * Whether the active executable is a manifest/DLL mismatch against a catalog
 * option with the same install key. The install record claims this variant
 * but the files beside the executable disagree, so the interface must offer
 * a reinstall instead of `use` or `active`.
 */
export function runtimeIdentityMismatch(
  option: RuntimeOption,
  active: RuntimeIdentity | null,
): string | null {
  if (!active) return null;
  if (active.installKey !== option.installKey) return null;
  if (active.source === "manifest-dll-mismatch" || active.backend === "mismatch") {
    return "Installed files disagree with the install record";
  }
  return null;
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

  const mismatch = runtimeIdentityMismatch(option, active);
  if (mismatch) return { kind: "mismatch", detail: mismatch };

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
  helpSha256: string;
  specTypes: string[];
  supportedFlags: string[];
  metrics: boolean;
  multimodal: boolean;
  fit: boolean;
};

export type ServerStatus = {
  running: boolean;
  /** Explicit lifecycle phase: idle, starting, running, or stopping. */
  phase: string;
  pid: number | null;
  profileName: string | null;
  alias: string | null;
  port: number | null;
  command: string | null;
  logPath: string | null;
  startedAt: number | null;
  exitCode: number | null;
  /** Strategy of the launched configuration, from the server snapshot. */
  specType?: string | null;
  companionLinked?: boolean | null;
  resultClass: FitClass;
  validation: LaunchValidation | null;
  failure: LaunchFailureEvidence | null;
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
  version: number;
  architecture: string;
  name: string;
  sizeLabel: string;
  fileType: number | null;
  blockCount: number | null;
  contextLength: number | null;
  embeddingLength: number | null;
  headCount: number | null;
  headCountKv: number | null;
  keyLength: number | null;
  valueLength: number | null;
  expertCount: number | null;
  expertUsedCount: number | null;
  vocabSize: number | null;
  ropeFreqBase: number | null;
  tensorCount: number;
  kvCount: number;
  // Optional shard metadata recorded by the producing tooling (audit S-02):
  // absent values mean the tensor-set completeness is unknown, never that
  // it is complete.
  splitNo?: number | null;
  splitCount?: number | null;
  metadataFacts: MetadataFact[];
  tensorDescriptors: TensorDescriptor[];
  tensorDescriptorsTruncated: boolean;
  headerBytes: number;
};

export type MetadataScalar =
  | { kind: "unsigned"; value: number }
  | { kind: "signed"; value: number }
  | { kind: "float"; value: number }
  | { kind: "boolean"; value: boolean }
  | { kind: "string"; value: string };

export type MetadataValue =
  | { shape: "scalar"; value: MetadataScalar }
  | {
      shape: "array";
      elementType: number;
      count: number;
      values: MetadataScalar[];
      truncated: boolean;
    };

export type MetadataFact = {
  key: string;
  value: MetadataValue;
};

export type TensorDescriptor = {
  name: string;
  dimensions: number[];
  ggmlType: number;
  offset: number;
};

export type ArtifactInspection = {
  logicalId: string;
  contentId: string | null;
  logicalName: string;
  firstShard: string;
  expectedShards: number;
  complete: boolean;
  headerConsistent: boolean;
  identityLevel: EvidenceLevel;
  shardBytes: number;
  companionBytes: number;
  shards: ArtifactFileFact[];
  companions: ArtifactFileFact[];
  summary: GgufSummary | null;
  problems: ArtifactProblem[];
};

export type RejectedLaunchArgument = {
  flag: string;
  reason: string;
};

export type LaunchArgumentValidation = {
  effectiveArgs: string[];
  rejected: RejectedLaunchArgument[];
  command: string;
};

export type LaunchValidation = {
  runtime: RuntimeCapabilities;
  artifacts: ArtifactInspection[];
  arguments: LaunchArgumentValidation;
  effectiveContext: Evidence<number>;
  unverifiedRequirements: string[];
};

export type LaunchFailureEvidence = {
  schema: number;
  phase: string;
  exitCode: number | null;
  timedOut: boolean;
  cancelled: boolean;
  message: string;
  logTail: string;
};

function structuredFailureText(value: unknown): string | null {
  if (typeof value !== "object" || value === null) return null;
  const failure = value as Record<string, unknown>;
  if (
    failure.schema !== 1 ||
    typeof failure.message !== "string" ||
    typeof failure.logTail !== "string"
  ) {
    return null;
  }
  return [failure.message, failure.logTail].filter(Boolean).join("\n");
}

export function errorText(error: unknown): string {
  const value = error instanceof Error ? error.message : error;
  const structured = structuredFailureText(value);
  if (structured !== null) return structured;
  const text = String(value);
  try {
    const parsed: unknown = JSON.parse(text);
    return structuredFailureText(parsed) ?? text;
  } catch {
    // Preserve non-structured Tauri and JavaScript errors.
  }
  return text;
}

export type KvCacheInputs = {
  blockCount: number | null;
  headCountKv: number | null;
  keyLength: number | null;
  valueLength: number | null;
  context: number;
  cacheTypeK: string;
  cacheTypeV: string;
  recurrentOrHybrid: boolean;
  observedAtMs: number;
};

export type MemoryAssessment = {
  class: FitClass;
  requiredBytes: Evidence<number>;
  availableBytes: Evidence<number>;
  policyReserveBytes: number;
  assumptions: string[];
};

export type StorageVolumeEvidence = {
  volumePath: Evidence<string>;
  residentBytes: Evidence<number>;
  availableBytes: Evidence<number>;
};

export type PreflightReport = {
  schema: number;
  class: FitClass;
  executionPath: ExecutionPath;
  requestedContext: Evidence<number>;
  nativeContext: Evidence<number>;
  effectiveContext: Evidence<number>;
  weightBytes: Evidence<number>;
  kvCacheBytes: Evidence<number>;
  storageRequiredBytes: Evidence<number>;
  diskAvailableBytes: Evidence<number>;
  storageVolumes: StorageVolumeEvidence[];
  memory: MemoryAssessment;
  assumptions: string[];
  unknowns: string[];
};

export type DeviceAllocationPlan = {
  adapterId: string;
  weightBytes: Evidence<number>;
  kvCacheBytes: Evidence<number>;
  totalBytes: Evidence<number>;
  availableBytes: Evidence<number>;
  note: string;
};

export type PreflightResult = {
  report: PreflightReport;
  devicePlan: DeviceAllocationPlan[];
  launch: LaunchValidation;
  hardware: HardwareInfo;
  selectedAdapterIds: string[];
};

export function artifactReadyForLaunch(artifact: ArtifactInspection): boolean {
  return artifact.complete && artifact.headerConsistent && artifact.problems.length === 0;
}

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
  effectiveContext?: number | null;
  stdDev?: number | null;
};

export type TuningReport = {
  baselineTps: number | null;
  bestIndex: number | null;
  bestTps: number | null;
  bestProfile: LaunchProfile;
  trials: TuningTrial[];
  stoppedReason: string;
  // Added with the requested-capacity objective (audit MT-11). Optional so a
  // report stored by an older build still loads.
  objective?: string;
  requiredEffectiveContext?: number;
  finalVerification?: {
    baselineTps: number;
    winnerTps: number;
    requiredImprovement: number;
    confirmed: boolean;
  } | null;
  qualityAffectingChanges?: string[];
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

/**
 * Presentation-only default proposal (audit FE-17 I5 boundary statement).
 *
 * Authority: Rust owns truth. `core::scan_models` ranks and attaches
 * companions, `core::validate_launch_profile`/`inspect_runtime` own flag
 * validity, and `lib.rs::preflight_model` re-derives the plan before launch.
 * This frontend factory only seeds an editable draft so the UI is useful
 * before the user changes anything; its output is preview, never authority,
 * and any divergence resolves in favour of the Rust decision.
 */
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
    // FE-09: a corrupt persisted record must not reach `.filter` as a
    // string; salvage what can be parsed and fall back to no arguments.
    extraArgs: (() => {
      const raw = merged.extraArgs as unknown;
      if (Array.isArray(raw)) {
        return raw.filter((arg): arg is string => typeof arg === "string" && arg !== "--jinja");
      }
      if (typeof raw === "string") {
        return parseExtraArgs(raw).filter((arg) => arg !== "--jinja");
      }
      return [];
    })(),
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
  lastModified?: string;
  createdAt?: string;
  /** True for files added locally: downloads use the stored exact digest. */
  userSourced?: boolean;
};

export function catalogRevision(file: CatalogFile): string {
  return file.revision || "main";
}

export function keepLatestRequest(sequence: number, current: number): boolean {
  return sequence === current;
}

/** Stable identity of a launch profile for deferred-response checks (FE-03). */
/** The selection a rescan keeps: the same model when it still exists,
 * otherwise the first remaining entry. `null` models means the scan failed
 * and the selection/profile must clear together (audit FE-01). */
export function selectionAfterRescan(
  models: LogicalModel[] | null,
  previousId: string,
): string | null {
  if (models === null) return null;
  if (models.some((model) => model.id === previousId)) return previousId;
  return models[0]?.id ?? null;
}

/** The model whose profile is on the profile screen: the committed selection
 * only. A missing selection is an explicit empty state, never a silent
 * fallback to another model (audit FE-01). */
export function displayedSelection(
  models: LogicalModel[],
  selectedId: string,
): LogicalModel | undefined {
  return models.find((model) => model.id === selectedId);
}

export function profileIdentity(profile: Pick<LaunchProfile, "name" | "model" | "runtime">): string {
  return `${profile.name}\u0000${profile.model}\u0000${profile.runtime}`;
}

/** True when a deferred response still matches both the latest request for a
 * resource and the resource's current identity: obsolete completions must not
 * replace current state (audit FE-03). */
export function responseIsCurrent(
  sequence: number,
  latest: number,
  identity: string,
  currentIdentity: string,
): boolean {
  return sequence === latest && identity === currentIdentity;
}

/** Apply a suggested port only while the profile is still the exact one the
 * suggestion was requested for and its port is unchanged: a later manual edit
 * or a newly selected profile is never overwritten (audit FE-03). */
/** App-wide label for the active evidence run shown outside Benchmark (FE-05). */
/** FE-02: one-line provenance of a tuning run or report, from its immutable
 * record rather than the editable selection. */
export function tuningRunSummary(run: {
  modelName: string;
  provider: string;
  advisor: string;
  context: number;
}): string {
  return `${run.modelName} · ${run.provider} · ${run.advisor} · ${run.context.toLocaleString()} ctx`;
}

export function evidenceRunLabel(kind: "benchmark" | "quality"): string {
  return kind === "benchmark" ? "Benchmark running" : "Quality suite running";
}

export function applySuggestedPort(
  profile: LaunchProfile,
  identity: string,
  expectedPort: number,
  suggested: number,
): LaunchProfile {
  if (profileIdentity(profile) !== identity || profile.port !== expectedPort) return profile;
  if (suggested === profile.port) return profile;
  return { ...profile, port: suggested };
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
  author?: string;
  summary: string;
  tags: string[];
  gated: boolean;
  downloads: number;
  likes: number;
  license?: string;
  pipeline_tag?: string;
  library_name?: string;
  architecture?: string;
  lastModified?: string;
  createdAt?: string;
  files: CatalogFile[];
  /** Local user-added rows carry true. Network rows always carry false. */
  userSourced?: boolean;
};

export type Catalog = {
  schemaVersion: number;
  updated: string;
  source: string;
  note: string;
  providers?: {
    source: string;
    cutoffDays: number;
    allowlist: string[];
  };
  models: CatalogModel[];
  /** Rows removed by runtime validation, with reasons (audit S-05). Absent
   * when nothing was dropped. */
  dropped?: { id: string; repo: string; reason: string }[];
};

export type CatalogSort = "downloads" | "likes" | "name" | "size";

export type CatalogQuery = {
  text: string;
  tag: string;
  quant: string;
  maxBytes: number;
  hideGated: boolean;
  sort: CatalogSort;
  author?: string;
  license?: string;
  pipeline_tag?: string;
  architecture?: string;
  fit_per_mille?: number;
  budget_bytes?: number;
};

export type CatalogFacets = {
  tags: string[];
  quants: string[];
  authors: string[];
  licenses: string[];
  // The Rust `CatalogFacets` serializes with rename_all = "camelCase": the
  // pipeline tag list arrives as `pipelineTags`. Reading a snake_case name
  // here left the state undefined and crashed the catalog render (GH-05).
  pipelineTags: string[];
  architectures: string[];
};

export type FitBudget = {
  budgetBytes: number;
  source: string;
};

/**
 * Pick the single budget the fit rule uses. Dedicated VRAM wins when any
 * adapter reports it; otherwise the largest shared figure; otherwise system
 * memory. Budgets are never summed. Mirrors catalog::hardware_fit_budget in
 * Rust, which owns truth; this copy lets the UI explain the auto-filter
 * without an IPC round trip, and tests pin both to the same answers.
 */
export function hardwareFitBudget(
  dedicated: (number | null)[],
  shared: (number | null)[],
  systemBytes: number,
): FitBudget {
  const best = (values: (number | null)[]): number => {
    let top = 0;
    for (const value of values) {
      if (typeof value === "number" && Number.isFinite(value) && value > top) top = value;
    }
    return top;
  };
  const dedicatedBest = best(dedicated);
  if (dedicatedBest > 0) return { budgetBytes: dedicatedBest, source: "dedicated" };
  const sharedBest = best(shared);
  if (sharedBest > 0) return { budgetBytes: sharedBest, source: "shared" };
  return { budgetBytes: systemBytes > 0 ? systemBytes : 0, source: "system" };
}

/**
 * Whether the hardware-fit rule hides a model with this smallest file.
 * Mirrors catalog::model_hidden_by_fit_rule in Rust, which owns truth.
 */
export function modelHiddenByFitRule(
  smallestBytes: number,
  fitPerMille: number | undefined,
  budgetBytes: number | undefined,
): boolean {
  const perMille = fitPerMille ?? 0;
  const budget = budgetBytes ?? 0;
  if (!(perMille > 0) || !(budget > 0)) return false;
  const clamped = Math.min(1000, Math.floor(perMille));
  return smallestBytes > Math.floor((budget * clamped) / 1000);
}

/**
 * The fit decision for one specific build (audit S-11): whether the SELECTED
 * file passes the size heuristic, whether the auto-filter keeps the row at
 * all (driven by the model's smallest file), and the threshold used. All
 * fields are null when the rule is off or no budget was observed: the
 * interface must say unknown instead of implying a fit.
 */
export type BuildFit = {
  thresholdBytes: number | null;
  selectedPasses: boolean | null;
  rowKept: boolean | null;
};

export function catalogBuildFit(
  selectedBytes: number,
  smallestBytes: number,
  fitPerMille: number | undefined,
  budgetBytes: number | undefined,
): BuildFit {
  const perMille = fitPerMille ?? 0;
  const budget = budgetBytes ?? 0;
  if (!(perMille > 0) || !(budget > 0)) {
    return { thresholdBytes: null, selectedPasses: null, rowKept: null };
  }
  const clamped = Math.min(1000, Math.floor(perMille));
  const thresholdBytes = Math.floor((budget * clamped) / 1000);
  return {
    thresholdBytes,
    selectedPasses: selectedBytes <= thresholdBytes,
    rowKept: smallestBytes <= thresholdBytes,
  };
}

/** Default auto-filter fraction: hide files above half the detected budget. */
export const DEFAULT_FIT_PER_MILLE = 500;

export type CatalogSnapshot = {
  catalog: Catalog;
  origin: "network" | "not-modified" | "cache" | "bundled";
  fetchedAt: string;
  url: string;
  /** Wall-clock second of the last successful network fill. Absent when never. */
  lastSuccessSecs?: string;
  /** Remaining cooldown in minutes: present when a refresh is throttled, and
   * on a local load that happened inside the window. */
  cooldownRemainingMinutes?: number;
  /** Set when a network refresh failed and local data was served instead. */
  refreshError?: string;
  /** Set when the verified catalog could not be persisted locally. */
  persistenceNotice?: string;
};

export type TokenStatus = {
  configured: boolean;
  masked: string;
  /** Present when a previous-version credential could not be removed from
   * Windows Credential Manager (audit S-07). Remove retries the cleanup. */
  cleanupNotice?: string;
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
export function downloadKey(
  repo: string,
  filename: string,
  destination: string,
  revision: string,
): string {
  // Mirrors `download_event_key` in lib.rs (audit FE-11): one job per
  // (repo, file, revision, destination); the frontend must not merge jobs
  // that the backend keeps apart.
  return `${repo}/${filename}@${revision}#${destination}`;
}

/// A download job as started from the catalog (audit FE-11).
export type DownloadJob = {
  key: string;
  repo: string;
  filename: string;
  revision: string;
  destination: string;
  startedAt: number;
};

function jobsForFile(
  jobs: readonly DownloadJob[],
  repo: string,
  filename: string,
): DownloadJob[] {
  return jobs
    .filter((job) => job.repo === repo && job.filename === filename)
    .sort((left, right) => right.startedAt - left.startedAt);
}

/// The job whose progress state a card should show: the newest job for the
/// file, whatever state it is in (FE-11).
export function newestDownloadJob(
  jobs: readonly DownloadJob[],
  repo: string,
  filename: string,
): DownloadJob | undefined {
  return jobsForFile(jobs, repo, filename)[0];
}

/// The job that is still running for the file — cancellation must bind to
/// this job, not to the currently edited destination (FE-11).
export function activeDownloadJob(
  jobs: readonly DownloadJob[],
  states: Readonly<Record<string, { state?: string } | undefined>>,
  repo: string,
  filename: string,
): DownloadJob | undefined {
  return jobsForFile(jobs, repo, filename).find((job) => {
    const state = states[job.key]?.state;
    return state === "downloading" || state === "verifying";
  });
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
