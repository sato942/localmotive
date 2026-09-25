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
import {
  tauriEvidenceAdapter,
  type EvidenceAdapter,
} from "./evidence-adapter";
import {
  evidenceTone,
  describeSamples,
  WORKLOAD_SCOPE_NOTE,
  WORKING_SET_SCOPE_NOTE,
  defaultWorkload,
  errorText,
  manualGpuOverride,
  type ArtifactInspection,
  type BenchmarkRunResult,
  type HardwareInfo,
  type HardwareOverride,
  type LaunchProfile,
  type LogicalModel,
  type PreflightResult,
  validateWorkload,
  type ServerStatus,
  type Workload,
} from "./model";
import type { EvidenceRun } from "./screens/evidence-run";

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

type Props = {
  model: LogicalModel | null;
  profile: LaunchProfile | null;
  serverStatus: ServerStatus;
  initialHardware: HardwareInfo | null;
  /** FE-05: publishes the active evidence run so the whole app can show its
   * status and offer the same cancellation handle on any screen. */
  onRunStateChange?: (state: EvidenceRun | null) => void;
  /// Injection seam (audit S-27.I2): acquisition and persistence arrive
  /// through this adapter; tests can substitute it freely.
  adapter?: EvidenceAdapter;
};

export function V03EvidencePanel({
  model,
  profile,
  serverStatus,
  initialHardware,
  onRunStateChange,
  adapter: injectedAdapter,
}: Props) {
  const adapter = injectedAdapter ?? tauriEvidenceAdapter;
  const [artifact, setArtifact] = useState<ArtifactInspection | null>(null);
  const [hardware, setHardware] = useState<HardwareInfo | null>(initialHardware);
  const [selectedAdapterIds, setSelectedAdapterIds] = useState<string[]>([]);
  const [manualCapacityGiB, setManualCapacityGiB] = useState("");
  const [manualCapacityNote, setManualCapacityNote] = useState("");
  const [preflight, setPreflight] = useState<PreflightResult | null>(null);
  // Audit FE-17 I1: the tested factory is the single default source.
  const [workload, setWorkload] = useState<Workload>(() => defaultWorkload());
  const [benchmark, setBenchmark] = useState<BenchmarkRunResult | null>(null);
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
    // A different model or edited profile invalidates the current editable
    // evidence; completed manifests remain available for replay.
    setArtifact(null);
    setPreflight(null);
    setPreflightInputsKey(null);
    setMessage(null);
  }, [model?.id, profileIdentityKey]);

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

  // Audit FE-17 I2: the production-used validator gates dispatch and shows
  // field-level errors; dispatch never relies on input attributes alone.
  const workloadErrors = useMemo(() => validateWorkload(workload), [workload]);

  // Audit FE-06 I1/I2: whether the displayed preflight result still matches
  // the inputs it was computed from.
  const preflightStale = preflight !== null && preflightInputsKey !== preflightInputs;

  // Async inspection/preflight responses carry the input revision they were
  // requested under; a response whose revision is no longer current is
  // discarded (audit FE-06 I5).
  useEffect(() => {
    inputRevisionRef.current += 1;
  }, [preflightInputs]);

  useEffect(() => {
    // FE-05 I2: navigation must not orphan a dispatched measurement; the app
    // keeps this run's status and cancel handle visible everywhere.
    if (busy === "benchmark") {
      onRunStateChange?.({ kind: "benchmark", cancel: () => void cancelBenchmark() });
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
    const result = await runAction("hardware", () => adapter.detectHardware());
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
      adapter.inspectModelArtifact({
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
      adapter.preflightModel({
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
    const issues = validateWorkload(nextWorkload);
    if (issues.length > 0) {
      setMessage(
        `Fix the workload before benchmarking: ${issues
          .map((issue) => `${issue.field} — ${issue.message}`)
          .join("; ")}`,
      );
      return;
    }
    const result = await runAction("benchmark", () =>
      adapter.benchmarkV2(nextWorkload),
    );
    if (result) {
      setBenchmark(result);
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
      await adapter.cancelBenchmark();
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
      adapter.replayBenchmarkManifest(benchmark.manifest),
    );
    if (replayed) {
      setWorkload(replayed);
      await executeBenchmark(replayed);
    }
  }

  return (
    <Card eyebrow="v0.3 evidence" title="Fit and measure">
      <div className="evidence-workbench">
        <p className="muted evidence-intro">
          Artifact inspection, preflight proof, controlled benchmarks, and saved-manifest replay keep measured evidence explicit.
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
              <strong className={`tone-${evidenceTone(artifact ? (artifact.complete ? "Complete" : "Blocked") : "Unknown")}`}>{artifact ? (artifact.complete ? "Complete" : "Blocked") : "Unknown"}</strong>
              <span>{artifact?.summary?.architecture ?? "Architecture unknown"}</span>
              {artifact?.problems.map((problem) => (
                <span className="danger-text" key={`${problem.code}-${problem.message}`}>
                  {problem.message}
                </span>
              ))}
            </div>
            <div className="evidence-status-card">
              <span className="muted">Preflight class</span>
              <strong className={`tone-${evidenceTone(preflight?.report.class ?? "Unknown")}`}>{preflight?.report.class ?? "Unknown"}</strong>
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
              {(hardware.unassignedNvidia?.length ?? 0) > 0 && (
                <span className="muted">
                  Unassigned NVIDIA telemetry: {hardware.unassignedNvidia?.length} row(s) could not be
                  mapped to one adapter by physical identity and stay unknown.
                </span>
              )}
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
                disabled={!serverStatus.running || busy !== null || workloadErrors.length > 0}
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
            <Field label="Prompt tokens"><Input type="number" min={1} max={1_048_576} step={1} required value={Number.isFinite(workload.promptTokens) ? workload.promptTokens : ""} onChange={(event) => setWorkload({ ...workload, promptTokens: event.target.valueAsNumber })} aria-label="Benchmark prompt tokens" /></Field>
            <Field label="Generated tokens"><Input type="number" min={1} max={65_536} step={1} required value={Number.isFinite(workload.generationTokens) ? workload.generationTokens : ""} onChange={(event) => setWorkload({ ...workload, generationTokens: event.target.valueAsNumber })} aria-label="Benchmark generated tokens" /></Field>
            <Field label="Warmups"><Input type="number" min={0} max={10} step={1} required value={Number.isFinite(workload.warmups) ? workload.warmups : ""} onChange={(event) => setWorkload({ ...workload, warmups: event.target.valueAsNumber })} aria-label="Benchmark warmups" /></Field>
            <Field label="Trials"><Input type="number" min={1} max={100} step={1} required value={Number.isFinite(workload.trials) ? workload.trials : ""} onChange={(event) => setWorkload({ ...workload, trials: event.target.valueAsNumber })} aria-label="Benchmark trials" /></Field>
            <Field label="Cache mode"><Select value={workload.cacheMode} onChange={(event) => setWorkload({ ...workload, cacheMode: event.target.value as Workload["cacheMode"] })} aria-label="Benchmark cache mode"><option value="warm">Warm</option><option value="cold">Cold (requires a fresh runtime)</option></Select></Field>
          </div>
          {workloadErrors.length > 0 && (
            <ul className="evidence-list" role="alert" aria-label="Workload validation errors">
              {workloadErrors.map((issue) => (
                <li key={`${issue.field}-${issue.code}`}>{issue.field}: {issue.message}</li>
              ))}
            </ul>
          )}

          <div className="evidence-grid">
            <div className="evidence-status-card">
              <span className="muted">Result class</span>
              <strong className={`tone-${evidenceTone(benchmark?.resultClass ?? "Unknown")}`}>{benchmark?.resultClass ?? "Unknown"}</strong>
              <span>{benchmark?.summary ? `${benchmark.summary.successfulTrials}/${benchmark.summary.successfulTrials + benchmark.summary.failedTrials} successful trials` : "No v2 result"}</span>
            </div>
            <div className="evidence-status-card">
              <span className="muted">Decode throughput</span>
              <strong className={`tone-${evidenceTone(benchmark?.summary?.decodeTps ? "measured" : "Unknown")}`}>{benchmark?.summary?.decodeTps ? `${benchmark.summary.decodeTps.mean.toFixed(2)} tok/s` : "Unknown"}</strong>
              <span>{benchmark?.summary?.decodeTps ? `p50 ${benchmark.summary.decodeTps.p50.toFixed(2)} · p95 ${benchmark.summary.decodeTps.p95.toFixed(2)}` : "No measured statistic"}</span>
              {benchmark?.summary?.decodeTps && (
                <span>{describeSamples(benchmark.summary.decodeTps)}</span>
              )}
              {benchmark?.summary?.decodeTps && (
                <span>Median {benchmark.summary.decodeTps.median.toFixed(2)} tok/s</span>
              )}
            </div>
            <div className="evidence-status-card">
              <span className="muted">First-token latency</span>
              <strong className={`tone-${evidenceTone(benchmark?.summary?.firstTokenMs ? "measured" : "Unknown")}`}>{benchmark?.summary?.firstTokenMs ? `${benchmark.summary.firstTokenMs.p50.toFixed(1)} ms p50` : "Unknown"}</strong>
              <span>{benchmark?.summary?.derivedTtftMs ? `Derived TTFT p50 ${benchmark.summary.derivedTtftMs.p50.toFixed(1)} ms (prefill + per-token decode; not an observed first token)` : "Derived TTFT unavailable"}</span>
              {benchmark?.summary?.firstTokenMs && (
                <span>{describeSamples(benchmark.summary.firstTokenMs)}</span>
              )}
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
              {benchmark.serverRestored ? " · server restored after the cold run" : ""}
              {benchmark.serverRestoreError
                ? ` · server restore failed: ${benchmark.serverRestoreError}`
                : ""}
              {" "}· CPU peak working set (process lifetime, excludes GPU): {formatBytes(
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
          <p className="muted evidence-scope-note">
            {benchmark?.manifest.scopeNote || WORKLOAD_SCOPE_NOTE}
          </p>
          <p className="muted evidence-scope-note">{WORKING_SET_SCOPE_NOTE}</p>
          {benchmark?.failure && <p className="danger-text">{benchmark.failure}</p>}
        </section>

      </div>
    </Card>
  );
}
