import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type ButtonHTMLAttributes,
  type InputHTMLAttributes,
  type ReactNode,
  type SelectHTMLAttributes,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import {
  calibrationState,
  errorText,
  manualGpuOverride,
  qualityPassRate,
  type ArtifactInspection,
  type BenchmarkRunResult,
  type CandidateEvidence,
  type CalibrationAnchor,
  type CalibrationModel,
  type CalibrationRecords,
  type CalibratedEstimate,
  type ExternalEvidenceBundle,
  type ExternalEvidenceState,
  type HardwareInfo,
  type HardwareOverride,
  type LaunchProfile,
  type LogicalModel,
  type PreflightResult,
  type QualitySuiteResult,
  type RankedCandidate,
  type RecommendationConstraints,
  type ObjectiveWeights,
  type ServerStatus,
  type ShareBundle,
  type Workload,
} from "./model";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: "primary" | "ghost";
};

function Button({ variant = "primary", className = "", ...props }: ButtonProps) {
  const visualClass = variant === "ghost" ? "secondary" : "primary";
  return <button className={`button ${visualClass} ${className}`.trim()} {...props} />;
}

function Card({ eyebrow, title, children }: { eyebrow: string; title: string; children: ReactNode }) {
  return (
    <article className="machine-panel evidence-panel">
      <div className="panel-title evidence-panel-title">
        <div>
          <span className="eyebrow">{eyebrow}</span>
          <h2>{title}</h2>
        </div>
      </div>
      {children}
    </article>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return <label className="evidence-field"><span>{label}</span>{children}</label>;
}

function Input(props: InputHTMLAttributes<HTMLInputElement>) {
  return <input {...props} />;
}

function Select(props: SelectHTMLAttributes<HTMLSelectElement>) {
  return <select {...props} />;
}

const DEFAULT_WORKLOAD: Workload = {
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

const DEFAULT_CONSTRAINTS: RecommendationConstraints = {
  minDecodeTps: null,
  maxP95LatencyMs: null,
  maxPeakMemoryBytes: null,
  minQualityPassRate: null,
  maxStorageBytes: null,
  requireMeasured: true,
};

const DEFAULT_WEIGHTS: ObjectiveWeights = {
  quality: 0.3,
  decodeTps: 0.25,
  prefillTps: 0.15,
  latency: 0.15,
  memory: 0.1,
  storage: 0.05,
};

// Audit FE-06: the full launch-profile fingerprint used for preflight-input
// identity. Every field that can change a preflight verdict is included.
function profileFingerprintOf(profile: LaunchProfile): Record<string, unknown> {
  const record = profile as unknown as Record<string, unknown>;
  const keys = Object.keys(record)
    .filter((key) => key !== "name")
    .sort();
  return Object.fromEntries(keys.map((key) => [key, record[key]]));
}

function hardwareSignatureOf(hardware: HardwareInfo | null): string | null {
  if (!hardware) return null;
  return JSON.stringify({
    architecture: hardware.architecture,
    vendor: hardware.vendor,
    driverVersion: hardware.driverVersion,
    detectionStatus: hardware.detectionStatus,
    adapters: hardware.adapters.map((adapter) => adapter.adapterId),
    systemMemoryBytes: hardware.systemMemory.totalPhysicalBytes.value ?? null,
  });
}

function preflightArtifactsOf(model: LogicalModel | null): unknown {
  if (!model) return null;
  return {
    firstShard: model.firstShard,
    companions: model.companions.map((item) => item.path),
  };
}

const PRIVACY_OMISSIONS = [
  "filesystem paths and hostnames",
  "prompts and model responses",
  "credentials and command arguments",
  "raw errors and logs",
  "stable adapter identifiers",
];

function formatBytes(value: number | null | undefined): string {
  if (value === null || value === undefined || !Number.isFinite(value)) return "Unknown";
  const units = ["B", "KiB", "MiB", "GiB", "TiB"];
  let amount = value;
  let unit = 0;
  while (amount >= 1024 && unit < units.length - 1) {
    amount /= 1024;
    unit += 1;
  }
  return `${amount.toFixed(unit === 0 ? 0 : 2)} ${units[unit]}`;
}

function numericInput(value: string, fallback: number): number {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : fallback;
}

type Props = {
  model: LogicalModel | null;
  profile: LaunchProfile | null;
  serverStatus: ServerStatus;
  initialHardware: HardwareInfo | null;
  /** FE-05: publishes the active evidence run so the whole app can show its
   * status and offer the same cancellation handle on any screen. */
  onRunStateChange?: (state: { kind: "benchmark" | "quality"; cancel: (() => void) | null } | null) => void;
};

export function V03EvidencePanel({
  model,
  profile,
  serverStatus,
  initialHardware,
  onRunStateChange,
}: Props) {
  const [artifact, setArtifact] = useState<ArtifactInspection | null>(null);
  const [hardware, setHardware] = useState<HardwareInfo | null>(initialHardware);
  const [selectedAdapterIds, setSelectedAdapterIds] = useState<string[]>([]);
  const [manualCapacityGiB, setManualCapacityGiB] = useState("");
  const [manualCapacityNote, setManualCapacityNote] = useState("");
  const [preflight, setPreflight] = useState<PreflightResult | null>(null);
  const [workload, setWorkload] = useState<Workload>(DEFAULT_WORKLOAD);
  const [benchmark, setBenchmark] = useState<BenchmarkRunResult | null>(null);
  const [history, setHistory] = useState<BenchmarkRunResult[]>([]);
  const [quality, setQuality] = useState<QualitySuiteResult | null>(null);
  const [ranking, setRanking] = useState<RankedCandidate[]>([]);
  const [constraints, setConstraints] = useState<RecommendationConstraints>(DEFAULT_CONSTRAINTS);
  const [weights] = useState<ObjectiveWeights>(DEFAULT_WEIGHTS);
  const [anchors, setAnchors] = useState<CalibrationAnchor[]>([]);
  const [estimatedTps, setEstimatedTps] = useState("");
  const [calibration, setCalibration] = useState<CalibrationModel | null>(null);
  const [calibrated, setCalibrated] = useState<CalibratedEstimate | null>(null);
  const [externalJson, setExternalJson] = useState("");
  const [externalEvidence, setExternalEvidence] = useState<ExternalEvidenceBundle | null>(null);
  const [shareConfirmed, setShareConfirmed] = useState(false);
  const [busy, setBusy] = useState<string | null>(null);
  // FE-07: a cancellation request in flight is distinct from the run itself.
  const [cancelPending, setCancelPending] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  // Audit FE-06 I3: the preflight-input identity covers the whole profile
  // (draft/projector, speculation, KV offload, fitting, attention, device
  // settings, extra args) plus the model, hardware observation, adapter
  // selection and manual capacity inputs — the material assumptions a
  // preflight result depends on.
  const preflightInputs = useMemo(
    () =>
      JSON.stringify({
        model: model?.id ?? null,
        profile: profile ? profileFingerprintOf(profile) : null,
        adapters: selectedAdapterIds,
        manualCapacityGiB,
        manualCapacityNote,
        hardwareSignature: hardwareSignatureOf(hardware),
        artifacts: preflightArtifactsOf(model),
      }),
    [model, profile, selectedAdapterIds, manualCapacityGiB, manualCapacityNote, hardware],
  );
  const inputRevisionRef = useRef(0);
  const [preflightInputsKey, setPreflightInputsKey] = useState<string | null>(null);

  useEffect(() => {
    setHardware(initialHardware);
  }, [initialHardware]);

  const profileIdentityKey = useMemo(
    () => (profile ? JSON.stringify(profileFingerprintOf(profile)) : null),
    [profile],
  );

  useEffect(() => {
    // FE-05: a different model or an edited profile invalidates the current
    // editable evidence (artifact, preflight, ranking input, export
    // approval) but completed runs stay in the history with their immutable
    // model/runtime/workload identities.
    setArtifact(null);
    setPreflight(null);
    setPreflightInputsKey(null);
    setRanking([]);
    setShareConfirmed(false);
    setMessage(null);
  }, [model?.id, profileIdentityKey]);

  // FE-05: a new reviewed payload (a different run or quality result)
  // requires a fresh export approval.
  useEffect(() => {
    setShareConfirmed(false);
  }, [benchmark?.manifestPath, quality?.observedAtMs]);

  // Audit FE-06 I4: adapter defaults initialize once, and a deliberate empty
  // selection (the user unchecked the final adapter) is preserved instead of
  // being silently repopulated.
  const [adaptersTouched, setAdaptersTouched] = useState(false);
  useEffect(() => {
    if (!hardware || adaptersTouched) return;
    if (selectedAdapterIds.length === 0 && hardware.adapters[0]) {
      setSelectedAdapterIds([hardware.adapters[0].adapterId]);
    }
  }, [hardware, adaptersTouched, selectedAdapterIds.length]);

  // A refreshed hardware observation reconciles an existing selection: kept
  // adapters stay, removed adapters drop out, and the result is never
  // repopulated behind the user's back.
  useEffect(() => {
    if (!hardware) return;
    setSelectedAdapterIds((current) => {
      if (current.length === 0) return current;
      const present = new Set(hardware.adapters.map((adapter) => adapter.adapterId));
      const reconciled = current.filter((adapterId) => present.has(adapterId));
      return reconciled.length === current.length ? current : reconciled;
    });
  }, [hardware]);

  // Audit FE-06 I1/I2: whether the displayed preflight result still matches
  // the inputs it was computed from.
  const preflightStale = preflight !== null && preflightInputsKey !== preflightInputs;

  // Async inspection/preflight responses carry the input revision they were
  // requested under; a response whose revision is no longer current is
  // discarded (audit FE-06 I5).
  useEffect(() => {
    inputRevisionRef.current += 1;
  }, [preflightInputs]);

  const latestCalibrationState = calibrationState(
    calibration,
    benchmark?.compatibilityKey ?? "",
    Date.now(),
  );

  useEffect(() => {
    const compatibilityKey = benchmark?.compatibilityKey;
    if (!compatibilityKey) return;
    let active = true;
    void invoke<CalibrationRecords>("load_calibration_records", { compatibilityKey })
      .then((records) => {
        if (!active) return;
        setAnchors(records.anchors);
        const now = Date.now();
        const activeModels = records.models
          .filter((model) => model.expiresAtMs > now)
          .sort((left, right) => left.createdAtMs - right.createdAtMs);
        const latest = activeModels[activeModels.length - 1] ?? null;
        setCalibration(latest);
      })
      .catch((error) => {
        if (active) setMessage(errorText(error));
      });
    return () => {
      active = false;
    };
  }, [benchmark?.compatibilityKey]);

  useEffect(() => {
    // FE-05 I2: navigation must not orphan a dispatched measurement; the app
    // keeps this run's status and cancel handle visible everywhere.
    if (busy === "benchmark") {
      onRunStateChange?.({ kind: "benchmark", cancel: () => void cancelBenchmark() });
      return () => onRunStateChange?.(null);
    }
    if (busy === "quality") {
      onRunStateChange?.({ kind: "quality", cancel: null });
      return () => onRunStateChange?.(null);
    }
    onRunStateChange?.(null);
    // `onRunStateChange` is a stable setState; the run state only depends on
    // which action is busy.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [busy]);

  async function runAction<T>(name: string, action: () => Promise<T>): Promise<T | null> {
    setBusy(name);
    setMessage(null);
    try {
      return await action();
    } catch (error) {
      setMessage(errorText(error));
      return null;
    } finally {
      setBusy(null);
    }
  }

  async function refreshHardware() {
    const result = await runAction("hardware", () => invoke<HardwareInfo>("detect_hardware"));
    if (result) {
      setHardware(result);
      const knownIds = new Set(result.adapters.map((adapter) => adapter.adapterId));
      setSelectedAdapterIds((current) => current.filter((id) => knownIds.has(id)));
      setMessage("Hardware evidence refreshed.");
    }
  }

  async function inspectArtifact() {
    if (!model) {
      setMessage("Select a model before artifact inspection.");
      return;
    }
    const capturedRevision = inputRevisionRef.current;
    const result = await runAction("artifact", () =>
      invoke<ArtifactInspection>("inspect_model_artifact", {
        firstShard: model.firstShard,
        companions: model.companions.map((item) => item.path),
        hashFiles: false,
      }),
    );
    if (result && inputRevisionRef.current !== capturedRevision) {
      setMessage("Artifact inspection discarded: the inputs changed while it ran.");
      return;
    }
    if (result) {
      setArtifact(result);
      setMessage(result.complete ? "Artifact structure is complete." : "Artifact inspection found blocking problems.");
    }
  }

  async function runPreflight() {
    if (!model || !profile) {
      setMessage("Select a model and profile before preflight.");
      return;
    }
    let manualOverrides: HardwareOverride[] = [];
    if (manualCapacityGiB.trim()) {
      if (selectedAdapterIds.length !== 1) {
        setMessage("Select exactly one adapter before you apply a manual capacity override.");
        return;
      }
      const override = manualGpuOverride(
        selectedAdapterIds[0],
        manualCapacityGiB,
        manualCapacityNote,
      );
      if (!override) {
        setMessage("Enter a finite manual capacity between 0 and 1024 GiB.");
        return;
      }
      manualOverrides = [override];
    }
    const capturedRevision = inputRevisionRef.current;
    const capturedInputs = preflightInputs;
    const result = await runAction("preflight", () =>
      invoke<PreflightResult>("preflight_model", {
        profile,
        selectedAdapterIds,
        manualOverrides,
      }),
    );
    if (result && inputRevisionRef.current !== capturedRevision) {
      setMessage(
        "Preflight result discarded: the model, profile, adapters or hardware changed while it ran. Re-run preflight.",
      );
      return;
    }
    if (result) {
      setPreflightInputsKey(capturedInputs);
      setPreflight(result);
      setArtifact(result.launch.artifacts[0] ?? null);
      setHardware(result.hardware);
      setMessage(`Preflight result: ${result.report.class}.`);
    }
  }

  async function executeBenchmark(nextWorkload: Workload) {
    if (!serverStatus.running) {
      setMessage("Start and health-validate the selected profile before benchmarking.");
      return;
    }
    setQuality(null);
    setShareConfirmed(false);
    const result = await runAction("benchmark", () =>
      invoke<BenchmarkRunResult>("benchmark_v2", { workload: nextWorkload }),
    );
    if (result) {
      setBenchmark(result);
      setHistory((current) => [...current, result]);
      setMessage(
        result.resultClass === "measured"
          ? "The measured benchmark manifest was saved."
          : `Benchmark finished as ${result.resultClass}.`,
      );
    }
  }

  async function cancelBenchmark() {
    // FE-07: cancellation request progress is its own state, separate from
    // the run's lifecycle (`busy` stays "benchmark" until the original
    // promise settles, so ownership is never cleared by the acknowledgement).
    // A void success is acknowledged truthfully; a failure keeps the run
    // record and the remaining affordance visible.
    if (cancelPending) return;
    setCancelPending(true);
    try {
      await invoke<void>("cancel_benchmark");
      setMessage("Benchmark cancellation requested; the run ends after the current attempt.");
    } catch (error) {
      setMessage(errorText(error));
    } finally {
      setCancelPending(false);
    }
  }

  async function replayBenchmark() {
    if (!benchmark) return;
    const replayed = await runAction("replay", () =>
      invoke<Workload>("replay_benchmark_manifest", {
        manifest: benchmark.manifest,
      }),
    );
    if (replayed) {
      setWorkload(replayed);
      await executeBenchmark(replayed);
    }
  }

  async function runQuality() {
    if (!serverStatus.running) {
      setMessage("Start and health-validate the selected profile before quality checks.");
      return;
    }
    const result = await runAction("quality", () =>
      invoke<QualitySuiteResult>("run_quality_suite"),
    );
    if (result) {
      setQuality(result);
      setMessage(`Quality suite result: ${result.status}.`);
    }
  }

  async function rankHistory() {
    if (history.length === 0) {
      setMessage("Run at least one measured benchmark before ranking.");
      return;
    }
    // The candidate join happens in Rust (audit MT-09 I3): quality evidence
    // attaches only when the launch identity, model content and runtime
    // match. A refused attachment is surfaced, never silently dropped.
    const candidates: CandidateEvidence[] = [];
    for (const [index, item] of history.entries()) {
      const attached = item === benchmark ? quality : null;
      try {
        const candidate = await invoke<CandidateEvidence>("join_quality_candidate", {
          id: `${item.compatibilityKey}:${index + 1}`,
          manifest: item.manifest,
          resultClass: item.resultClass,
          quality: attached,
        });
        candidates.push(candidate);
      } catch (error) {
        setMessage(`Quality evidence was not attached: ${errorText(error)}`);
        return;
      }
    }
    const result = await runAction("ranking", () =>
      invoke<RankedCandidate[]>("rank_candidates", {
        candidates,
        constraints,
        weights,
      }),
    );
    if (result) {
      setRanking(result);
      setMessage(
        "Candidates were ranked with hard constraints and Pareto dominance; a quality rate of 1.0 means both structural smoke cases passed.",
      );
    }
  }

  async function addCalibrationAnchor() {
    const estimated = Number(estimatedTps);
    if (!benchmark?.manifestPath || !Number.isFinite(estimated) || estimated <= 0) {
      setMessage("Enter a positive estimate after a successful measured benchmark.");
      return;
    }
    // The anchor is created in Rust from the persisted run identity: the
    // measured value and observation time come from the saved manifest, and
    // one run can contribute at most one sample (audit MT-08).
    const records = await runAction("calibration-anchor", () =>
      invoke<CalibrationRecords>("add_benchmark_calibration_anchor", {
        manifestPath: benchmark.manifestPath,
        estimatedValue: estimated,
        estimator: "manual-estimate.v1",
      }),
    );
    if (records) {
      setAnchors(records.anchors);
      setMessage(
        "Calibration anchor persisted from the saved run. Repeated adds on one run share one sample.",
      );
    }
  }

  function compatibleAnchors(): CalibrationAnchor[] {
    if (!benchmark) return [];
    return anchors.filter(
      (anchor) => anchor.compatibilityKey === benchmark.compatibilityKey,
    );
  }

  function compatibleRunCount(): number {
    const runs = new Set(
      compatibleAnchors()
        .map((anchor) => anchor.sourceRunId ?? "")
        .filter((identity) => identity.length > 0),
    );
    return runs.size;
  }

  async function buildCalibration() {
    if (!benchmark) return;
    const compatible = compatibleAnchors();
    const result = await runAction("calibration", () =>
      invoke<CalibrationModel>("build_calibration_model", {
        anchors: compatible,
        createdAtMs: Date.now(),
        ttlMs: 90 * 24 * 60 * 60 * 1_000,
      }),
    );
    if (result) {
      const records = await runAction("calibration-store", () =>
        invoke<CalibrationRecords>("store_calibration_model", { model: result }),
      );
      if (records) {
        setCalibration(result);
        setAnchors(records.anchors);
        setMessage("A local calibration model was built and persisted from compatible anchors.");
      }
    }
  }

  async function applyCalibration() {
    if (!benchmark || !calibration) return;
    const value = Number(estimatedTps);
    if (!Number.isFinite(value) || value <= 0) {
      setMessage("Enter a positive estimate before applying calibration.");
      return;
    }
    const result = await runAction("calibrated", () =>
      invoke<CalibratedEstimate>("apply_calibration_model", {
        model: calibration,
        compatibilityKey: benchmark.compatibilityKey,
        estimatedValue: value,
        nowMs: Date.now(),
      }),
    );
    if (result) {
      setCalibrated(result);
      setMessage("Calibration produced bounded estimated evidence, not a measured result.");
    }
  }

  async function importExternal() {
    let parsed: ExternalEvidenceBundle;
    try {
      parsed = JSON.parse(externalJson) as ExternalEvidenceBundle;
    } catch {
      setMessage("External evidence must be valid JSON.");
      return;
    }
    const result = await runAction("external", () =>
      invoke<ExternalEvidenceBundle>("import_external_evidence", { bundle: parsed }),
    );
    if (result) {
      setExternalEvidence(result);
      setMessage("External evidence passed schema checks and remains pending review.");
    }
  }

  async function reviewExternal(state: Exclude<ExternalEvidenceState, "pending">) {
    if (!externalEvidence) return;
    const result = await runAction("external-review", () =>
      invoke<ExternalEvidenceBundle>("review_external_evidence", {
        bundle: externalEvidence,
        state,
        confirmed: true,
      }),
    );
    if (result) {
      setExternalEvidence(result);
      setMessage(`External evidence review state: ${result.state}.`);
    }
  }

  async function exportShareBundle() {
    if (!benchmark || !shareConfirmed) {
      setMessage("Review the omissions and confirm the local export first.");
      return;
    }
    const target = await saveDialog({
      defaultPath: `localmotive-evidence-${benchmark.compatibilityKey.slice(0, 12)}.json`,
      filters: [{ name: "JSON evidence", extensions: ["json"] }],
    });
    if (!target) return;

    const bundle = await runAction("share-build", () =>
      invoke<ShareBundle>("build_share_export", {
        manifest: benchmark.manifest,
        summary: benchmark.summary,
        quality,
        compatibilityKey: benchmark.compatibilityKey,
        createdAtMs: Date.now(),
        confirmed: true,
      }),
    );
    if (!bundle) return;

    const written = await runAction("share-write", () =>
      invoke<string>("write_share_export", {
        path: target,
        bundle,
        confirmed: true,
      }),
    );
    if (written) {
      setMessage("The privacy-reviewed evidence bundle was written locally.");
    }
  }

  return (
    <Card eyebrow="v0.3 evidence" title="Fit, measure, compare, and share">
      <div className="evidence-workbench">
        <p className="muted evidence-intro">
          Estimates, launch proof, measurements, and imported evidence keep separate labels.
        </p>

        {message && <p className="evidence-message" role="status">{message}</p>}

        <section className="evidence-section" aria-labelledby="artifact-preflight-title">
          <div className="evidence-heading-row">
            <div>
              <h4 id="artifact-preflight-title">Artifact and preflight evidence</h4>
              <p className="muted">Choose adapters explicitly. Unknown inputs remain visible.</p>
            </div>
            <div className="actions compact-actions">
              <Button variant="ghost" onClick={() => void refreshHardware()} disabled={busy !== null}>
                Refresh hardware
              </Button>
              <Button variant="ghost" onClick={() => void inspectArtifact()} disabled={!model || busy !== null}>
                Inspect artifact
              </Button>
              <Button onClick={() => void runPreflight()} disabled={!model || !profile?.runtime || busy !== null}>
                Run preflight
              </Button>
            </div>
          </div>

          <div className="evidence-grid">
            <div className="evidence-status-card">
              <span className="muted">Artifact</span>
              <strong>{artifact ? (artifact.complete ? "Complete" : "Blocked") : "Unknown"}</strong>
              <span>{artifact?.summary?.architecture ?? "Architecture unknown"}</span>
              {artifact?.problems.map((problem) => (
                <span className="danger-text" key={`${problem.code}-${problem.message}`}>
                  {problem.message}
                </span>
              ))}
            </div>
            <div className="evidence-status-card">
              <span className="muted">Preflight class</span>
              <strong>{preflight?.report.class ?? "Unknown"}</strong>
              <span>{preflight?.report.executionPath ?? "Execution path unknown"}</span>
              {preflightStale ? (
                <span className="muted">
                  Stale: the model, profile, adapters, manual capacity or hardware
                  observation changed after this result. Re-run preflight.
                </span>
              ) : null}
              <span>
                Required: {formatBytes(preflight?.report.memory.requiredBytes.value)}
              </span>
              <span>
                Available: {formatBytes(preflight?.report.memory.availableBytes.value)}
              </span>
              <span>
                Policy reserve: {formatBytes(preflight?.report.memory.policyReserveBytes ?? null)}
              </span>
              <span>
                Resident storage: {formatBytes(preflight?.report.storageRequiredBytes.value)}
              </span>
            </div>
            <div className="evidence-status-card">
              <span className="muted">Launch proof</span>
              <strong>{serverStatus.resultClass}</strong>
              <span>{serverStatus.validation ? "Health endpoint validated" : "No launch validation"}</span>
              <span>
                Effective context: {serverStatus.validation?.effectiveContext.value ?? "Unknown"}
                {" "}({serverStatus.validation?.effectiveContext.level ?? "unknown"})
              </span>
              {serverStatus.validation?.arguments.rejected.map((item) => (
                <span key={`${item.flag}-${item.reason}`}>{item.flag}: {item.reason}</span>
              ))}
              {serverStatus.failure && (
                <span className="danger-text launch-failure" role="alert">
                  {errorText(serverStatus.failure)}
                </span>
              )}
            </div>
          </div>

          {preflight && preflight.devicePlan.length > 0 && (
            <div className="device-plan-grid" aria-label="Preflight device allocation plan">
              {preflight.devicePlan.map((plan) => (
                <article className="evidence-status-card" key={plan.adapterId}>
                  <span className="muted">Planned device</span>
                  <strong>{plan.adapterId}</strong>
                  <span>Weights: {formatBytes(plan.weightBytes.value)} ({plan.weightBytes.level})</span>
                  <span>KV cache: {formatBytes(plan.kvCacheBytes.value)} ({plan.kvCacheBytes.level})</span>
                  <span>Total: {formatBytes(plan.totalBytes.value)} ({plan.totalBytes.level})</span>
                  <span>Available: {formatBytes(plan.availableBytes.value)} ({plan.availableBytes.level})</span>
                  <small>{plan.note}</small>
                </article>
              ))}
            </div>
          )}

          {preflight && preflight.report.storageVolumes.length > 0 && (
            <div className="device-plan-grid" aria-label="Selected artifact storage by volume">
              {preflight.report.storageVolumes.map((volume, index) => (
                <article
                  className="evidence-status-card"
                  key={volume.volumePath.value ?? `unknown-volume-${index}`}
                >
                  <span className="muted">Artifact volume</span>
                  <strong>{volume.volumePath.value ?? "Unknown volume"}</strong>
                  <span>
                    Resident: {formatBytes(volume.residentBytes.value)} ({volume.residentBytes.level})
                  </span>
                  <span>
                    Free: {formatBytes(volume.availableBytes.value)} ({volume.availableBytes.level})
                  </span>
                </article>
              ))}
            </div>
          )}

          {hardware && (
            <fieldset className="adapter-picker">
              <legend>Adapters included in preflight</legend>
              {hardware.adapters.length === 0 && <span className="muted">No GPU adapter evidence is available.</span>}
              {hardware.adapters.map((adapter) => (
                <label key={adapter.adapterId}>
                  <input
                    type="checkbox"
                    checked={selectedAdapterIds.includes(adapter.adapterId)}
                    onChange={(event) => {
                      setAdaptersTouched(true);
                      setSelectedAdapterIds((current) =>
                        event.target.checked
                          ? [...new Set([...current, adapter.adapterId])]
                          : current.filter((id) => id !== adapter.adapterId),
                      );
                    }}
                  />
                  <span>{adapter.name}</span>
                  <small>{adapter.vendor} · {formatBytes(adapter.dedicatedBytes.value)} dedicated · {formatBytes(adapter.availableBudgetBytes.value)} available · {formatBytes(adapter.availableForReservationBytes.value)} reservable · {adapter.backend.value ?? "backend unknown"}</small>
                </label>
              ))}
            </fieldset>
          )}

          <div className="grid cols-3 evidence-controls manual-override-controls">
            <Field label="Manual dedicated capacity (GiB)">
              <Input
                inputMode="decimal"
                placeholder="Optional"
                value={manualCapacityGiB}
                onChange={(event) => setManualCapacityGiB(event.target.value)}
              />
            </Field>
            <Field label="Override note">
              <Input
                maxLength={1024}
                placeholder="Source or reason"
                value={manualCapacityNote}
                onChange={(event) => setManualCapacityNote(event.target.value)}
              />
            </Field>
            <p className="muted override-help">
              Manual capacity remains userOverride evidence. Localmotive does not combine dedicated and shared memory.
            </p>
          </div>

          {preflight && (preflight.report.unknowns.length > 0 || preflight.report.assumptions.length > 0) && (
            <details>
              <summary>Review assumptions and unknowns</summary>
              <ul className="evidence-list">
                {preflight.report.unknowns.map((item) => <li key={`unknown-${item}`}>Unknown: {item}</li>)}
                {preflight.report.assumptions.map((item) => <li key={`assumption-${item}`}>Assumption: {item}</li>)}
              </ul>
            </details>
          )}
        </section>

        <section className="evidence-section" aria-labelledby="benchmark-v2-title">
          <div className="evidence-heading-row">
            <div>
              <h4 id="benchmark-v2-title">Benchmark v2</h4>
              <p className="muted">Use one fixed workload. Raw failures remain in the saved manifest.</p>
            </div>
            <div className="actions compact-actions">
              <Button
                onClick={() => void executeBenchmark(workload)}
                disabled={!serverStatus.running || busy !== null}
              >
                {busy === "benchmark" ? "Benchmarking…" : "Run v2 benchmark"}
              </Button>
              <Button variant="ghost" onClick={() => void cancelBenchmark()} disabled={busy !== "benchmark" || cancelPending}>
                Cancel
              </Button>
              <Button variant="ghost" onClick={() => void replayBenchmark()} disabled={!benchmark || busy !== null}>
                Replay manifest
              </Button>
            </div>
          </div>

          <div className="grid cols-3 evidence-controls">
            <Field label="Workload ID"><Input value={workload.id} onChange={(event) => setWorkload({ ...workload, id: event.target.value })} aria-label="Benchmark workload ID" /></Field>
            <Field label="Prompt tokens"><Input type="number" min={1} value={workload.promptTokens} onChange={(event) => setWorkload({ ...workload, promptTokens: numericInput(event.target.value, 0) })} aria-label="Benchmark prompt tokens" /></Field>
            <Field label="Generated tokens"><Input type="number" min={1} value={workload.generationTokens} onChange={(event) => setWorkload({ ...workload, generationTokens: numericInput(event.target.value, 0) })} aria-label="Benchmark generated tokens" /></Field>
            <Field label="Warmups"><Input type="number" min={0} value={workload.warmups} onChange={(event) => setWorkload({ ...workload, warmups: numericInput(event.target.value, 0) })} aria-label="Benchmark warmups" /></Field>
            <Field label="Trials"><Input type="number" min={1} value={workload.trials} onChange={(event) => setWorkload({ ...workload, trials: numericInput(event.target.value, 0) })} aria-label="Benchmark trials" /></Field>
            <Field label="Cache mode"><Select value={workload.cacheMode} onChange={(event) => setWorkload({ ...workload, cacheMode: event.target.value as Workload["cacheMode"] })} aria-label="Benchmark cache mode"><option value="warm">Warm</option><option value="cold">Cold (requires a fresh runtime)</option></Select></Field>
          </div>

          <div className="evidence-grid">
            <div className="evidence-status-card">
              <span className="muted">Result class</span>
              <strong>{benchmark?.resultClass ?? "Unknown"}</strong>
              <span>{benchmark?.summary ? `${benchmark.summary.successfulTrials}/${benchmark.summary.successfulTrials + benchmark.summary.failedTrials} successful trials` : "No v2 result"}</span>
            </div>
            <div className="evidence-status-card">
              <span className="muted">Decode throughput</span>
              <strong>{benchmark?.summary?.decodeTps ? `${benchmark.summary.decodeTps.mean.toFixed(2)} tok/s` : "Unknown"}</strong>
              <span>{benchmark?.summary?.decodeTps ? `p50 ${benchmark.summary.decodeTps.p50.toFixed(2)} · p95 ${benchmark.summary.decodeTps.p95.toFixed(2)}` : "No measured statistic"}</span>
              {benchmark?.summary?.decodeTps && (
                <span>Median {benchmark.summary.decodeTps.median.toFixed(2)} tok/s</span>
              )}
            </div>
            <div className="evidence-status-card">
              <span className="muted">First-token latency</span>
              <strong>{benchmark?.summary?.firstTokenMs ? `${benchmark.summary.firstTokenMs.p50.toFixed(1)} ms p50` : "Unknown"}</strong>
              <span>{benchmark?.summary?.derivedTtftMs ? `Derived TTFT p50 ${benchmark.summary.derivedTtftMs.p50.toFixed(1)} ms` : "Derived TTFT unavailable"}</span>
            </div>
          </div>

          {benchmark && (
            <p className="muted">
              Warmups:{" "}
              {benchmark.manifest.warmups.length === 0
                ? "none"
                : benchmark.manifest.warmups
                    .map((warmup) => `${warmup.warmup}:${warmup.outcome}`)
                    .join(", ")}
              {benchmark.manifest.terminalOutcome
                ? ` · terminal ${benchmark.manifest.terminalOutcome}`
                : ""}
              {" "}· peak process memory: {formatBytes(
                benchmark.manifest.observations
                  .map((item) => item.peakProcessRssBytes.value ?? null)
                  .filter((value): value is number => value !== null)
                  .reduce<number | null>(
                    (maximum, value) => (maximum === null ? value : Math.max(maximum, value)),
                    null,
                  ),
              )}{" "}
              ({benchmark.manifest.observations.filter((item) => item.peakProcessRssBytes.value !== null).length}/
              {benchmark.manifest.observations.length} sampled)
            </p>
          )}
          {benchmark?.failure && <p className="danger-text">{benchmark.failure}</p>}
        </section>

        <section className="evidence-section" aria-labelledby="quality-ranking-title">
          <div className="evidence-heading-row">
            <div>
              <h4 id="quality-ranking-title">Quality and Pareto ranking</h4>
              <p className="muted">Quality is separate from speed. Hard failures remain excluded.</p>
            </div>
            <div className="actions compact-actions">
              <Button variant="ghost" onClick={() => void runQuality()} disabled={!serverStatus.running || busy !== null}>Run quality suite</Button>
              <Button variant="ghost" onClick={() => void rankHistory()} disabled={history.length === 0 || busy !== null}>Rank session results</Button>
            </div>
          </div>

          <div className="grid cols-3 evidence-controls">
            <Field label="Minimum decode tok/s"><Input type="number" min={0} value={constraints.minDecodeTps ?? ""} onChange={(event) => setConstraints({ ...constraints, minDecodeTps: event.target.value ? numericInput(event.target.value, 0) : null })} /></Field>
            <Field label="Maximum p95 latency (ms)"><Input type="number" min={0} value={constraints.maxP95LatencyMs ?? ""} onChange={(event) => setConstraints({ ...constraints, maxP95LatencyMs: event.target.value ? numericInput(event.target.value, 0) : null })} /></Field>
            <Field label="Minimum quality rate"><Input type="number" min={0} max={1} step={0.05} value={constraints.minQualityPassRate ?? ""} onChange={(event) => setConstraints({ ...constraints, minQualityPassRate: event.target.value ? numericInput(event.target.value, 0) : null })} /></Field>
          </div>

          <p className="muted">
            Quality: {quality ? `${quality.status} (${qualityPassRate(quality) ?? "unknown"})` : "not run"}.
            Session candidates: {history.length}.
          </p>
          {ranking.length > 0 && (
            <div className="ranking-list">
              {ranking.map((item, index) => (
                <div className="evidence-status-card" key={item.id}>
                  <strong>#{index + 1} · {item.id}</strong>
                  <span>{item.feasible ? "Feasible" : `Rejected: ${item.violations.join("; ")}`}</span>
                  <span>{item.pareto ? "Pareto frontier" : `Dominated by ${item.dominatedBy.join(", ") || "unknown"}`}</span>
                  <span>Preference score: {item.preferenceScore === null ? "Unknown" : item.preferenceScore.toFixed(3)}</span>
                </div>
              ))}
            </div>
          )}
        </section>

        <details className="evidence-section">
          <summary>Local calibration and external evidence</summary>
          <div className="grid cols-3 evidence-controls">
            <Field label="Uncalibrated estimate (tok/s)"><Input type="number" min={0} value={estimatedTps} onChange={(event) => setEstimatedTps(event.target.value)} /></Field>
            <div className="actions compact-actions evidence-action-cell">
              <Button variant="ghost" onClick={addCalibrationAnchor} disabled={!benchmark?.summary || busy !== null}>Add anchor</Button>
              <Button variant="ghost" onClick={() => void buildCalibration()} disabled={compatibleRunCount() < 3 || !benchmark || busy !== null}>Build calibration</Button>
              <Button variant="ghost" onClick={() => void applyCalibration()} disabled={!calibration || latestCalibrationState !== "compatible" || busy !== null}>Apply</Button>
            </div>
          </div>
          <p className="muted">
            Compatible anchors: {compatibleAnchors().length} from {compatibleRunCount()} unique run{compatibleRunCount() === 1 ? "" : "s"} (three distinct runs are required).
            Calibration: {latestCalibrationState}.
            {calibrated ? ` Estimated interval ${calibrated.lowerBound.toFixed(2)}–${calibrated.upperBound.toFixed(2)} tok/s.` : ""}
          </p>
          <p className="muted">
            Estimated interval = estimate × (mean measured/estimated ratio ± 1.96 × the ratio spread).
            It is a descriptive spread of saved runs, not a calibrated confidence interval.
          </p>

          <Field label="External evidence JSON">
            <textarea
              className="evidence-json-input"
              value={externalJson}
              onChange={(event) => setExternalJson(event.target.value)}
              placeholder='{"schema":1,"source":"...","compatibilityKey":"...","state":"pending","records":[]}'
            />
          </Field>
          <div className="actions compact-actions">
            <Button variant="ghost" onClick={() => void importExternal()} disabled={!externalJson.trim() || busy !== null}>Validate import</Button>
            <Button variant="ghost" onClick={() => void reviewExternal("verified")} disabled={externalEvidence?.state !== "pending" || busy !== null}>Mark verified</Button>
            <Button variant="ghost" onClick={() => void reviewExternal("flagged")} disabled={externalEvidence?.state !== "pending" || busy !== null}>Flag</Button>
            <Button variant="ghost" onClick={() => void reviewExternal("rejected")} disabled={externalEvidence?.state !== "pending" || busy !== null}>Reject</Button>
            <span className="muted">Imported state: {externalEvidence?.state ?? "none"}. Imported evidence never upgrades automatically.</span>
          </div>
        </details>

        <details className="evidence-section">
          <summary>Privacy-reviewed local export</summary>
          <p className="muted">The export omits:</p>
          <ul className="evidence-list">
            {PRIVACY_OMISSIONS.map((item) => <li key={item}>{item}</li>)}
          </ul>
          <label className="confirmation-row">
            <input type="checkbox" checked={shareConfirmed} onChange={(event) => setShareConfirmed(event.target.checked)} />
            <span>I reviewed these omissions and approve a local JSON export.</span>
          </label>
          <Button onClick={() => void exportShareBundle()} disabled={!benchmark || !shareConfirmed || busy !== null}>Choose export file</Button>
        </details>
      </div>
    </Card>
  );
}
