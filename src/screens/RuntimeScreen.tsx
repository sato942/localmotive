import type { Dispatch, SetStateAction } from "react";
import {
  ArrowUp, BadgeCheck, CircleStop, Cpu, Download, FolderOpen, MonitorCog, Play,
  RefreshCw, ShieldCheck, Wrench,
} from "lucide-react";
import {
  bytesLabel,
  conflictingCapacityMetrics,
  downloadPercent,
  managedHealthOutcome,
  runtimeOptionState,
  type HardwareInfo,
  type HealthModelProgress,
  type ManagedHealthResult,
  type ManagedRuntimeRecord,
  type RuntimeCapabilities,
  type RuntimeCatalog,
  type RuntimeCatalogViewState,
  type RuntimeIdentity,
  type RuntimeInstallProgress,
  type RuntimeOption,
} from "../model";

/// Presentational contract for the Runtime manager screen (audit S-27.I2).
/// The component renders; it does not acquire, persist, or decide anything.
export interface RuntimeScreenProps {
  activateRuntime: (path: string) => void;
  busy: string;
  cancelHealthRepair: () => void;
  cancelManagedHealth: () => void;
  cancelRuntimeInstall: () => void;
  chooseExistingRuntime: () => void;
  hardware: HardwareInfo | null;
  healthCancelling: boolean;
  healthModelProgress: HealthModelProgress | null;
  healthRepairNotice: string | null;
  healthRepairing: boolean;
  healthResult: ManagedHealthResult | null;
  healthRunning: string;
  inspect: (pathOverride?: string) => void;
  installRuntime: (option: RuntimeOption) => void;
  installing: string;
  isManagedPath: boolean;
  loadRuntimeSetup: () => void;
  managedRuntimes: ManagedRuntimeRecord[];
  modelRoot: string;
  openExternal: (url: string) => void;
  repairHealthModel: () => void;
  runManagedHealth: (option: RuntimeOption) => void;
  runtime: RuntimeCapabilities | null;
  runtimeCatalog: RuntimeCatalog | null;
  runtimeCatalogLoading: boolean;
  runtimeCatalogState: RuntimeCatalogViewState;
  runtimeIdentity: RuntimeIdentity | null;
  runtimeInstallCancelling: boolean;
  runtimeInstallProgress: RuntimeInstallProgress | null;
  runtimePath: string;
  runtimeRoot: string;
  selectRuntimeAdapter: (adapterId: string) => void;
  selectedRuntimeAdapterId: string;
  setRuntime: Dispatch<SetStateAction<RuntimeCapabilities | null>>;
  setRuntimePath: Dispatch<SetStateAction<string>>;
}

export function RuntimeScreen(props: RuntimeScreenProps) {
  const healthOutcome = props.healthResult ? managedHealthOutcome(props.healthResult) : null;
  const runtimeCatalogState: RuntimeCatalogViewState = props.runtimeCatalogState;
  return (
    <section className="screen runtime-screen">
      <div className="section-heading">
        <div>
          <h1>Runtime manager</h1>
          <p>Detect this Windows PC, choose the right backend, and keep llama.cpp updateable.</p>
        </div>
        <button className="button secondary" onClick={props.loadRuntimeSetup} disabled={props.runtimeCatalogLoading}>
          <RefreshCw size={16} className={props.runtimeCatalogLoading ? "spin" : ""} /> Check latest release
        </button>
      </div>

      <div className="setup-steps" aria-label="First-run setup">
        <div className={props.runtimePath ? "setup-step done" : "setup-step active"}><span>1</span><strong>Runtime</strong><small>{props.runtimePath ? "Path selected" : "Install or choose"}</small></div>
        <div className={props.modelRoot ? "setup-step done" : "setup-step"}><span>2</span><strong>Models</strong><small>{props.modelRoot ? "Folder selected" : "Choose in Inventory"}</small></div>
        <div className={props.runtimePath && props.modelRoot ? "setup-step done" : "setup-step"}><span>3</span><strong>Serve</strong><small>{props.runtimePath && props.modelRoot ? "Ready to validate" : "Complete setup"}</small></div>
      </div>

      <div className="hardware-panel">
        <div className="hardware-icon"><MonitorCog size={26} /></div>
        <div>
          <span className="instrument-label">DETECTED WINDOWS HARDWARE</span>
          <strong>{props.hardware?.gpuNames.length ? props.hardware.gpuNames.join(" · ") : "CPU / GPU detection pending"}</strong>
          <p>{props.runtimeCatalog?.recommendationReason ?? "Waiting for the approved catalog to evaluate exact product evidence…"}</p>
          <small className="hardware-source">{props.hardware?.detectionStatus}</small>
        </div>
        <div className="hardware-meta"><span>{props.hardware?.architecture ?? "—"}</span><span>{props.hardware?.vendor.toUpperCase() ?? "—"}</span>{props.hardware?.cudaMajor && <span>CUDA {props.hardware.cudaMajor}</span>}{props.hardware?.driverVersion && <span>Driver {props.hardware.driverVersion}</span>}</div>
      </div>

      {props.hardware && (
        <div className="hardware-evidence-grid" aria-label="Hardware evidence">
          <article className="hardware-evidence-card">
            <span className="instrument-label">SYSTEM MEMORY</span>
            <strong>{props.hardware.systemMemory.totalPhysicalBytes.value === null ? "UNKNOWN" : bytesLabel(props.hardware.systemMemory.totalPhysicalBytes.value)}</strong>
            <dl>
              <div><dt>Available</dt><dd>{props.hardware.systemMemory.availablePhysicalBytes.value === null ? "Unknown" : bytesLabel(props.hardware.systemMemory.availablePhysicalBytes.value)}<small>{props.hardware.systemMemory.availablePhysicalBytes.source.detail} · {props.hardware.systemMemory.availablePhysicalBytes.observedAtMs ? new Date(props.hardware.systemMemory.availablePhysicalBytes.observedAtMs).toISOString() : "not observed"}</small></dd></div>
              <div><dt>Load</dt><dd>{props.hardware.systemMemory.memoryLoadPercent.value === null ? "Unknown" : `${props.hardware.systemMemory.memoryLoadPercent.value}%`}<small>{props.hardware.systemMemory.memoryLoadPercent.source.detail} · {props.hardware.systemMemory.memoryLoadPercent.observedAtMs ? new Date(props.hardware.systemMemory.memoryLoadPercent.observedAtMs).toISOString() : "not observed"}</small></dd></div>
            </dl>
            <small>{props.hardware.systemMemory.totalPhysicalBytes.source.detail} · {props.hardware.systemMemory.totalPhysicalBytes.observedAtMs ? new Date(props.hardware.systemMemory.totalPhysicalBytes.observedAtMs).toISOString() : "not observed"}</small>
          </article>
          {props.hardware.adapters.map((adapter) => {
            const conflicts = conflictingCapacityMetrics(adapter);
            return (
              <article className="hardware-evidence-card" key={adapter.adapterId}>
                <span className="instrument-label">GPU ADAPTER · {adapter.adapterId}</span>
                <strong>{adapter.name}</strong>
                <dl>
                  <div><dt>Compatibility ID</dt><dd><code>{adapter.compatibilityId}</code></dd></div>
                  <div><dt>Driver</dt><dd>{adapter.driver.value ?? "Unknown"} ({adapter.driver.level})</dd></div>
                  <div><dt>Dedicated</dt><dd>{adapter.dedicatedBytes.value === null ? "Unknown" : bytesLabel(adapter.dedicatedBytes.value)}<small>{adapter.dedicatedBytes.source.detail} · {adapter.dedicatedBytes.observedAtMs ? new Date(adapter.dedicatedBytes.observedAtMs).toISOString() : "not observed"}</small></dd></div>
                  <div><dt>Shared</dt><dd>{adapter.sharedBytes.value === null ? "Unknown" : bytesLabel(adapter.sharedBytes.value)}<small>{adapter.sharedBytes.source.detail} · {adapter.sharedBytes.observedAtMs ? new Date(adapter.sharedBytes.observedAtMs).toISOString() : "not observed"}</small></dd></div>
                  <div><dt>Budget</dt><dd>{adapter.budgetBytes.value === null ? "Unknown" : bytesLabel(adapter.budgetBytes.value)}<small>{adapter.budgetBytes.source.detail} · {adapter.budgetBytes.observedAtMs ? new Date(adapter.budgetBytes.observedAtMs).toISOString() : "not observed"}</small></dd></div>
                  <div><dt>Current use</dt><dd>{adapter.currentUsageBytes.value === null ? "Unknown" : bytesLabel(adapter.currentUsageBytes.value)}<small>{adapter.currentUsageBytes.source.detail} · {adapter.currentUsageBytes.observedAtMs ? new Date(adapter.currentUsageBytes.observedAtMs).toISOString() : "not observed"}</small></dd></div>
                  <div><dt>Available budget</dt><dd>{adapter.availableBudgetBytes.value === null ? "Unknown" : bytesLabel(adapter.availableBudgetBytes.value)}<small>{adapter.availableBudgetBytes.source.detail} · {adapter.availableBudgetBytes.observedAtMs ? new Date(adapter.availableBudgetBytes.observedAtMs).toISOString() : "not observed"}</small></dd></div>
                  <div><dt>Backend</dt><dd>{adapter.backend.value?.toUpperCase() ?? "Unknown"} ({adapter.backend.level})</dd></div>
                </dl>
                <small>{adapter.budgetBytes.source.detail} · {adapter.budgetBytes.observedAtMs ? new Date(adapter.budgetBytes.observedAtMs).toISOString() : "not observed"}</small>
                {conflicts.length > 0 && <p className="hardware-conflict">Conflicting probes: {conflicts.join(", ")}</p>}
              </article>
            );
          })}
          {props.hardware.manualOverrides.map((override) => (
            <article className="hardware-evidence-card hardware-override" key={`override-${override.adapterId}`}>
              <span className="instrument-label">MANUAL OVERRIDE · {override.adapterId}</span>
              <strong>User supplied</strong>
              <p>{override.note || "No note supplied."}</p>
            </article>
          ))}
        </div>
      )}

      {props.hardware && props.hardware.adapters.length > 0 && (
        <label className="runtime-adapter-picker">
          GPU adapter for managed runtime installation
          <select
            value={props.selectedRuntimeAdapterId}
            onChange={(event) => void props.selectRuntimeAdapter(event.target.value)}
            disabled={props.runtimeCatalogLoading}
          >
            {props.hardware.adapters.length > 1 && <option value="">Select one detected adapter</option>}
            {props.hardware.adapters.map((adapter) => (
              <option value={adapter.adapterId} key={adapter.adapterId}>
                {adapter.name} · {adapter.adapterId}
              </option>
            ))}
          </select>
        </label>
      )}

      {props.installing && (
        <div className="download-progress downloading" role="status" aria-live="polite">
          {props.runtimeInstallProgress ? (
            <>
              <div><b style={{ width: `${downloadPercent(props.runtimeInstallProgress.downloaded, props.runtimeInstallProgress.total)}%` }} /></div>
              <span>
                {props.runtimeInstallProgress.assetName} · {bytesLabel(props.runtimeInstallProgress.downloaded)} / {bytesLabel(props.runtimeInstallProgress.total)}
              </span>
            </>
          ) : (
            <span>Validating hardware and preparing the approved runtime download…</span>
          )}
          <button className="button danger" onClick={props.cancelRuntimeInstall} disabled={props.runtimeInstallCancelling}>
            <CircleStop size={15} /> {props.runtimeInstallCancelling ? "Stopping…" : "Keep partial download and stop"}
          </button>
        </div>
      )}

      <div className="runtime-layout">
        <div className="runtime-main">
          <div className="runtime-section-title">
            <div>
              <span className="instrument-label">DIRECT RUNTIME · L2 EVIDENCE CEILING</span>
              <h2>Official Windows builds</h2>
              <p>
                {runtimeCatalogState.kind === "ready" || runtimeCatalogState.kind === "empty"
                  ? `Approved binary release ${runtimeCatalogState.catalog.tag}${runtimeCatalogState.catalog.publishedAt ? ` · ${runtimeCatalogState.catalog.publishedAt.slice(0, 10)}` : ""}`
                  : runtimeCatalogState.kind === "loading"
                    ? "Loading the approved release from ggml-org/llama.cpp"
                    : runtimeCatalogState.kind === "error"
                      ? `Could not load the approved release: ${runtimeCatalogState.error.message}`
                      : "Runtime catalog not loaded"}
              </p>
            </div>
            <div className="runtime-section-actions">
              <button
                className="text-button runtime-source"
                aria-label="Refresh approved runtime catalog"
                disabled={runtimeCatalogState.kind === "loading"}
                onClick={props.loadRuntimeSetup}
              >
                <RefreshCw size={14} /> Refresh catalog
              </button>
              <button className="text-button runtime-source" onClick={() => props.openExternal("https://github.com/ggml-org/llama.cpp/releases")}><Download size={14} /> GitHub releases</button>
            </div>
          </div>
          <div className="runtime-options">
            {(runtimeCatalogState.kind === "ready" || runtimeCatalogState.kind === "empty") && (
              <p className="runtime-recommendation-reason" role="status">
                {runtimeCatalogState.catalog.recommendationReason}
              </p>
            )}
            {runtimeCatalogState.kind === "loading" && (
              <div className="runtime-loading" role="status" aria-label="Loading approved runtime catalog">
                <RefreshCw size={22} className="spin" aria-hidden="true" />
                <span>Reading official release assets…</span>
              </div>
            )}
            {runtimeCatalogState.kind === "error" && (
              <div className="runtime-catalog-message error" role="alert">
                <strong>Runtime catalog unavailable</strong>
                <span>{runtimeCatalogState.error.message}</span>
                {runtimeCatalogState.error.retryAfterSeconds !== null && (
                  <span>Retry after {runtimeCatalogState.error.retryAfterSeconds} seconds.</span>
                )}
                <button className="button secondary" onClick={props.loadRuntimeSetup}>Retry</button>
              </div>
            )}
            {runtimeCatalogState.kind === "idle" && (
              <div className="runtime-catalog-message">
                <span>The runtime catalog is not loaded.</span>
                <button className="button secondary" onClick={props.loadRuntimeSetup}>Load catalog</button>
              </div>
            )}
            {runtimeCatalogState.kind === "empty" && (
              <div className="runtime-catalog-message" role="status">
                <strong>No approved runtime matches this system.</strong>
                <span>Review the blocked backend evidence or retry catalog retrieval.</span>
                <button className="button secondary" onClick={props.loadRuntimeSetup}>Retry</button>
              </div>
            )}
            {runtimeCatalogState.kind === "ready" && runtimeCatalogState.catalog.options.map((option) => {
              const state = runtimeOptionState(option, runtimeCatalogState.catalog.tag, props.managedRuntimes, props.runtimeIdentity, props.runtime?.build ?? null);
              const downloading = props.installing === option.id;
              const adapterRequired = option.backend !== "cpu" && !props.selectedRuntimeAdapterId;
              const managedInstalled = props.managedRuntimes.some((record) => record.installKey === option.installKey);
              const supportLabel = `DIRECT RUNTIME · UPSTREAM EVIDENCE ONLY · ${props.hardware?.architecture ?? "unknown architecture"} · ${option.backend} · ${runtimeCatalogState.catalog.tag}`;
              return (
              <article className={`runtime-option${option.recommended ? " recommended" : ""}${state.kind === "active" ? " is-active" : ""}`} key={`${option.id}:${option.asset.name}`} aria-label={`${option.label} runtime option, ${option.recommended ? "recommended" : "not recommended"}, ${state.kind}`}>
                <div className="runtime-option-main">
                  <div className="runtime-option-title"><strong>{option.label}</strong>{state.kind === "active" && <span className="state-tag good"><BadgeCheck size={10} /> ACTIVE · b{props.runtime?.build}</span>}{state.kind === "update" && <span className="state-tag warning"><ArrowUp size={10} /> UPDATE FROM {state.from.toUpperCase()}</span>}{state.kind === "mismatch" && <span className="state-tag warning">MISMATCH · REINSTALL</span>}<span className="state-tag warning" title="Product support requires an exact L4 compatibility record">PRODUCT SUPPORT NOT VALIDATED</span>{state.kind !== "active" && state.kind !== "update" && state.kind !== "mismatch" && option.recommended && <span className="state-tag good">RECOMMENDED</span>}</div>
                  <span
                    className="runtime-role runtime-scope"
                    data-support-architecture={props.hardware?.architecture ?? "unknown"}
                    data-support-backend={option.backend}
                    data-runtime-revision={runtimeCatalogState.catalog.tag}
                  >
                    {supportLabel} · exact product configuration untested
                  </span>
                  <span className="runtime-role">{state.kind === "active" ? (props.isManagedPath ? "This is the runtime in use" : "Your existing executable matches this package") : state.kind === "update" ? `Newer official build available for the runtime in use` : state.kind === "mismatch" ? "Installed files disagree with the install record · reinstall before launch" : state.kind === "use" ? "Already downloaded · not the active runtime" : option.recommended ? "Recommended for this PC" : option.backend === "cpu" ? "CPU fallback" : option.backend === "vulkan" ? "Compatibility fallback" : option.backend === "cuda" ? "Alternative CUDA package" : "Optional backend"}</span>
                  <p>{option.description}</p>
                  <small>{option.compatibility}</small>
                  <code>{option.asset.name}</code>
                  <small className="runtime-provenance"><ShieldCheck size={12} /> Official ggml-org asset · {runtimeCatalogState.catalog.tag} · {option.asset.digest ? "digest available" : "size metadata available"}</small>
                  <details className="asset-proof"><summary>Artifact verification</summary><code>{option.asset.digest ?? "No digest published for this asset"}</code><span>Localmotive verifies published size and SHA-256 before extraction. GitHub release metadata does not provide a Windows code-signing result.</span></details>
                </div>
                <div className="runtime-option-action">
                  <span>{bytesLabel(option.asset.size + (option.companionAsset?.size ?? 0))}</span>
                  {state.kind === "active" && (
                    <button className="button secondary is-current" disabled aria-disabled="true"><BadgeCheck size={15} /> Up to date</button>
                  )}
                  {state.kind === "update" && (
                    <button className="button primary" onClick={() => props.installRuntime(option)} disabled={Boolean(props.installing) || adapterRequired}>
                      <ArrowUp size={15} /> {downloading ? "Downloading…" : `Update to ${runtimeCatalogState.catalog.tag}`}
                    </button>
                  )}
                  {state.kind === "use" && (
                    <button className="button secondary" onClick={() => props.activateRuntime(state.runtimePath)} disabled={Boolean(props.installing) || props.busy === "runtime"}>
                      <Play size={15} /> Use this build
                    </button>
                  )}
                  {state.kind === "install" && (
                    <button className={option.recommended ? "button primary" : "button secondary"} onClick={() => props.installRuntime(option)} disabled={Boolean(props.installing) || adapterRequired}>
                      <Download size={15} /> {downloading ? "Downloading…" : "Install"}
                    </button>
                  )}
                  {state.kind === "mismatch" && (
                    <button className="button primary" onClick={() => props.installRuntime(option)} disabled={Boolean(props.installing) || adapterRequired}>
                      <Download size={15} /> {downloading ? "Downloading…" : "Reinstall"}
                    </button>
                  )}
                  {managedInstalled && (
                    <button
                      className="button secondary"
                      onClick={() => props.runManagedHealth(option)}
                      disabled={Boolean(props.installing) || Boolean(props.healthRunning) || adapterRequired}
                    >
                      <ShieldCheck size={15} /> {props.healthRunning === option.installKey ? "Checking…" : "Run 7-stage health"}
                    </button>
                  )}
                </div>
              </article>
              );
            })}
            {(runtimeCatalogState.kind === "ready" || runtimeCatalogState.kind === "empty") && runtimeCatalogState.catalog.availability
              .filter((entry) => entry.status === "blocked")
              .map((entry) => (
                <article className="runtime-option blocked" key={`blocked:${entry.installKey}`} aria-label={`${entry.backend} runtime blocked`}>
                  <div className="runtime-option-main">
                    <div className="runtime-option-title">
                      <strong>{entry.backend.toUpperCase()}</strong>
                      <span className="state-tag warning">BLOCKED</span>
                    </div>
                    <p>{entry.reason}</p>
                    {entry.blockingJobs.map((jobName, index) => (
                      <span className="runtime-role" key={`${entry.installKey}:${jobName}`}>
                        Blocking job: <code>{jobName}</code>
                        {entry.evidenceUrls[index] && (
                          <button className="text-button runtime-source" onClick={() => props.openExternal(entry.evidenceUrls[index])}>
                            <ShieldCheck size={12} /> View {jobName} evidence
                          </button>
                        )}
                      </span>
                    ))}
                    {entry.evidenceUrls.slice(entry.blockingJobs.length).map((url) => (
                      <button className="text-button runtime-source" key={url} onClick={() => props.openExternal(url)}>
                        <ShieldCheck size={12} /> View blocking evidence
                      </button>
                    ))}
                  </div>
                </article>
              ))}
          </div>
        </div>

        <aside className="runtime-sidebar">
          <div className="panel-title"><ShieldCheck size={17} /><h2>Managed runtime</h2></div>
          <div className="managed-status">
            <span className={props.runtime ? "plate-light ok" : "plate-light"} />
            <div><strong>{props.runtime ? `llama.cpp b${props.runtime.build}` : "Not configured"}</strong>{props.runtime && <span className={props.isManagedPath ? "runtime-trust managed" : "runtime-trust supplied"}>{props.isManagedPath ? "MANAGED · VERIFIED DOWNLOAD" : "USER-SUPPLIED · VERSION INSPECTED"}</span>}{props.runtimeIdentity && props.runtimeIdentity.backend !== "unknown" && <span className="runtime-trust neutral">{props.runtimeIdentity.backend.toUpperCase()}{props.runtimeIdentity.cudaMajor ? ` ${props.runtimeIdentity.cudaMajor}` : ""} · {props.runtimeIdentity.source === "manifest" ? "FROM MANIFEST" : "FROM DLLS"}</span>}<small>{props.runtimePath || "Install a recommended build or choose an existing executable."}</small></div>
          </div>
          <label>Existing llama-server.exe<input value={props.runtimePath} onChange={(event) => {
            // The inspected identity belongs to the old path (FE-15):
            // clear stale capabilities until this path is inspected.
            if (event.target.value !== props.runtimePath) props.setRuntime(null);
            props.setRuntimePath(event.target.value);
          }} placeholder="Path to llama-server.exe" /></label>
          <div className="runtime-side-actions">
            <button className="button secondary" onClick={props.chooseExistingRuntime}><FolderOpen size={15} /> Browse</button>
            <button className="button secondary" onClick={() => props.inspect()} disabled={!props.runtimePath}><Cpu size={15} /> Inspect</button>
          </div>
          {props.managedRuntimes.length > 0 && (
            <dl className="runtime-facts managed-list">
              <div><dt>Installed managed builds</dt><dd>{props.managedRuntimes.map((record) => <button key={record.runtimePath} className={`managed-entry${record.runtimePath === props.runtimePath ? " current" : ""}`} onClick={() => props.activateRuntime(record.runtimePath)} disabled={record.runtimePath === props.runtimePath}>{record.tag} · {record.installKey}{record.runtimePath === props.runtimePath ? " · active" : ""}</button>)}</dd></div>
            </dl>
          )}
          <div className="managed-health" aria-live="polite">
            <span className="instrument-label">SEVEN-STAGE HEALTH CONTRACT</span>
            <p>Localmotive retrieves and verifies the immutable pinned SmolLM2 health model before each run.</p>
            {props.healthRunning && (
              <>
                {props.healthModelProgress && (
                  <div className="download-progress downloading" role="status">
                    <div><b style={{ width: `${downloadPercent(props.healthModelProgress.downloaded, props.healthModelProgress.total)}%` }} /></div>
                    <span>Pinned health model · {bytesLabel(props.healthModelProgress.downloaded)} / {bytesLabel(props.healthModelProgress.total)}</span>
                  </div>
                )}
                <button className="button danger" onClick={props.cancelManagedHealth} disabled={props.healthCancelling}>
                  <CircleStop size={15} /> {props.healthCancelling ? "Stopping…" : "Cancel and clean up"}
                </button>
              </>
            )}
            {props.healthResult && (
              <div className={`health-result ${healthOutcome}`}>
                <strong>{healthOutcome?.toUpperCase()}</strong>
                <small>Runtime {props.healthResult.runtimeId} · model SHA-256 {props.healthResult.modelSha256}</small>
                <ol>
                  {props.healthResult.stages.map((stage) => (
                    <li key={stage.stage} className={stage.status.toLowerCase()}>
                      <span>{stage.stage.replace(/_/g, " ")}</span>
                      <b>{stage.status}</b>
                      <small>{stage.detail}{stage.failureReason ? ` · ${stage.failureReason}` : ""}</small>
                    </li>
                  ))}
                </ol>
              </div>
            )}
            <div className="health-repair">
              <button className="button secondary" onClick={props.repairHealthModel} disabled={props.healthRepairing || Boolean(props.healthRunning)}>
                <Wrench size={15} /> {props.healthRepairing ? "Repairing…" : "Repair cached model"}
              </button>
              {props.healthRepairing && (
                <button className="button danger" onClick={props.cancelHealthRepair}>
                  <CircleStop size={15} /> Cancel repair
                </button>
              )}
              {props.healthRepairNotice && <small role="status">{props.healthRepairNotice}</small>}
            </div>
          </div>
          <dl className="runtime-facts">
            <div><dt>Managed folder</dt><dd>{props.runtimeRoot || "Resolving…"}</dd></div>
            <div><dt>Verification</dt><dd>Size always; SHA-256 before activation when published</dd></div>
            <div><dt>CUDA packages</dt><dd>Binary + matching cudart DLLs</dd></div>
            <div><dt>Existing files</dt><dd>User-supplied; version/help inspected, not download-verified</dd></div>
            <div><dt>Updates</dt><dd>Versioned; existing installs preserved</dd></div>
          </dl>
        </aside>
      </div>
    </section>
  );
}
