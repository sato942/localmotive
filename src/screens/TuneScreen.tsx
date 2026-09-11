import type { Dispatch, KeyboardEvent, RefObject, SetStateAction } from "react";
import {
  Activity, KeyRound, LogIn, RefreshCw, Save, Settings2, Sparkles, Square, SquareTerminal, Trophy, Unplug,
} from "lucide-react";
import {
  contextChoices,
  describeChanges,
  tuningRunSummary,
  type BriefDisclosure,
  type CloudModel,
  type CloudProvider,
  type CredentialStatus,
  type DisclosureSection,
  type GgufSummary,
  type HardwareInfo,
  type LogicalModel,
  type RuntimeCapabilities,
  type RuntimeIdentity,
  type ServerStatus,
  type TuningProgress,
  type TuningReport,
  type TuningTrial,
} from "../model";

/// Identity of a tuning run or of the displayed report (FE-02).
export type TuneRunIdentity = {
  modelId: string;
  modelName: string;
  provider: string;
  advisor: string;
  context: number;
};

/// Presentational contract for the Tune screen (audit S-27.I2). Acquisition
/// and persistence stay in `App.tsx`; the component renders only.
export interface TuneScreenProps {
  adoptTunedProfile: () => void;
  bestLive: TuningTrial | null;
  briefDisclosure: BriefDisclosure;
  busy: string;
  canTune: boolean;
  cancelTuning: () => void;
  changeDisclosure: (mode: BriefDisclosure) => void;
  chooseCloudModel: (id: string) => void;
  cloudCheck: string;
  cloudModel: string;
  cloudModels: CloudModel[];
  credential: CredentialStatus | null;
  disclosureSections: DisclosureSection[];
  forgetKey: () => void;
  gguf: GgufSummary | null;
  hardware: HardwareInfo | null;
  keyDraft: string;
  onProviderTabKey: (event: KeyboardEvent<HTMLButtonElement>, index: number) => void;
  openExternal: (url: string) => void;
  openRouterLogin: () => void;
  probeCloud: () => void;
  provider: CloudProvider | null;
  providerId: string;
  providers: CloudProvider[];
  runtime: RuntimeCapabilities | null;
  runtimeIdentity: RuntimeIdentity | null;
  runtimePath: string;
  saveKey: () => void;
  selected: LogicalModel | undefined;
  setKeyDraft: Dispatch<SetStateAction<string>>;
  setTuneContext: Dispatch<SetStateAction<number>>;
  setTuneRepeats: Dispatch<SetStateAction<number>>;
  setTuneTokens: Dispatch<SetStateAction<number>>;
  setTuneTrials: Dispatch<SetStateAction<number>>;
  startTuning: () => void;
  status: ServerStatus;
  switchProvider: (next: string) => void;
  trialsForDisplay: TuningTrial[];
  tuneBlocker: string;
  tuneContext: number;
  tuneLogRef: RefObject<HTMLDivElement | null>;
  tuneProgress: TuningProgress | null;
  tuneRepeats: number;
  tuneReport: TuningReport | null;
  tuneReportOrigin: TuneRunIdentity | null;
  tuneRun: TuneRunIdentity | null;
  tuneTokens: number;
  tuneTrials: number;
  tuning: boolean;
}

export function TuneScreen(props: TuneScreenProps) {
  const {
    adoptTunedProfile,
    bestLive,
    briefDisclosure,
    busy,
    canTune,
    cancelTuning,
    changeDisclosure,
    chooseCloudModel,
    cloudCheck,
    cloudModel,
    cloudModels,
    credential,
    disclosureSections,
    forgetKey,
    gguf,
    hardware,
    keyDraft,
    onProviderTabKey,
    openExternal,
    openRouterLogin,
    probeCloud,
    provider,
    providerId,
    providers,
    runtime,
    runtimeIdentity,
    runtimePath,
    saveKey,
    selected,
    setKeyDraft,
    setTuneContext,
    setTuneRepeats,
    setTuneTokens,
    setTuneTrials,
    startTuning,
    status,
    switchProvider,
    trialsForDisplay,
    tuneBlocker,
    tuneContext,
    tuneLogRef,
    tuneProgress,
    tuneRepeats,
    tuneReport,
    tuneReportOrigin,
    tuneRun,
    tuneTokens,
    tuneTrials,
    tuning,
  } = props;
  return (
    <section className="screen tune-screen">
      <div className="section-heading">
        <div>
          <h1>AI tuning</h1>
          <p>A cloud model proposes llama-server settings; this PC measures them. The best measured configuration wins.</p>
        </div>
        <div className="actions">
          {tuning ? (
            <button className="button danger" onClick={cancelTuning}><Square size={15} fill="currentColor" /> Stop after this trial</button>
          ) : (
            <button className="button primary" onClick={startTuning} disabled={!canTune}><Sparkles size={16} /> Auto-tune {selected ? selected.name : "model"}</button>
          )}
        </div>
      </div>

      <div className="panel" aria-label="Cloud data disclosure">
        <p className="muted">
          Cloud tuning sends a brief to the selected provider. Local inference and local
          share export never send data anywhere — this setting affects cloud tuning only.
        </p>
        <fieldset className="disclosure-choice">
          <legend>What the brief carries</legend>
          <label>
            <input
              type="radio"
              name="tune-disclosure"
              checked={briefDisclosure === "full"}
              onChange={() => changeDisclosure("full")}
            />
            Full — every measured value, including local paths
          </label>
          <label>
            <input
              type="radio"
              name="tune-disclosure"
              checked={briefDisclosure === "minimal"}
              onChange={() => changeDisclosure("minimal")}
            />
            Minimal — directories and user names removed; file names, flags and measurements stay
          </label>
        </fieldset>
        <ul className="disclosure-list">
          {disclosureSections.map((section) => (
            <li key={section.category}>
              <strong>{section.category}</strong> — {section.detail}
              {!section.sentInMinimal ? " (full mode only)" : ""}
            </li>
          ))}
        </ul>
        <p className="muted">
          Stored cloud credentials are never part of the brief, in either mode.
        </p>
      </div>

      <div className="setup-steps" aria-label="Tuning readiness">
        <div className={credential?.configured ? "setup-step done" : "setup-step active"}><span>1</span><strong>Cloud provider</strong><small>{credential?.configured ? `${provider?.label ?? providerId} connected` : "Add a key or sign in"}</small></div>
        <div className={selected?.complete && runtimePath ? "setup-step done" : "setup-step"}><span>2</span><strong>Local model</strong><small>{selected ? selected.name : "Select in Inventory"}</small></div>
        <div className={tuneReport ? "setup-step done" : canTune ? "setup-step active" : "setup-step"}><span>3</span><strong>Tune</strong><small>{tuning ? "Running…" : tuneReport ? "Report ready" : status.running ? "Stop the server first" : "Choose context, start"}</small></div>
      </div>

      <div className="tune-layout">
        <div className="settings-stack">
          <article className="machine-panel">
            <div className="panel-title"><KeyRound size={17} /><h2>Cloud provider</h2>{credential && <span className={credential.configured ? "state-tag good" : "state-tag warning"}>{credential.configured ? `CONNECTED ${credential.masked}` : "NO KEY"}</span>}</div>
            <div className="provider-tabs" role="tablist" aria-label="Cloud providers">
              {providers.map((entry, index) => (
                <button
                  key={entry.id}
                  id={`provider-tab-${entry.id}`}
                  role="tab"
                  aria-selected={entry.id === providerId}
                  aria-controls="provider-panel"
                  tabIndex={entry.id === providerId ? 0 : -1}
                  className={entry.id === providerId ? "provider-tab active" : "provider-tab"}
                  onClick={() => switchProvider(entry.id)}
                  onKeyDown={(event) => onProviderTabKey(event, index)}
                >{entry.label}</button>
              ))}
            </div>
            <div
              className="provider-body"
              id="provider-panel"
              role="tabpanel"
              aria-labelledby={`provider-tab-${providerId}`}
              tabIndex={0}
            >
              {provider?.supportsOauth && (
                <div className="oauth-row">
                  <button className="button secondary" onClick={openRouterLogin} disabled={busy === "oauth" || tuning}><LogIn size={15} /> {busy === "oauth" ? "Waiting for browser…" : credential?.configured ? "Sign in again" : "Sign in with OpenRouter"}</button>
                  <small>Opens your browser. OpenRouter issues a key you control (OAuth PKCE); it is stored in Windows Credential Manager and never shown here.</small>
                </div>
              )}
              <label className="wide">{provider?.supportsOauth ? "Or paste an API key" : "API key"}
                <div className="key-row">
                  <input type="password" autoComplete="off" spellCheck={false} value={keyDraft} onChange={(e) => setKeyDraft(e.target.value)} placeholder={provider ? `${provider.keyPrefixHint}…` : ""} aria-label="API key" />
                  <button className="button secondary" onClick={saveKey} disabled={!keyDraft.trim() || busy === "cloud"}><Save size={15} /> Store</button>
                </div>
                <small className="field-help">Stored under “Localmotive” in Windows Credential Manager, not in this app’s settings. {provider && <button className="text-link" onClick={() => openExternal(provider.consoleUrl)}>Get a key from {provider.label} →</button>}</small>
              </label>
              {credential?.configured && (
                <>
                  <label className="wide">Advisor model
                    {cloudModels.length ? (
                      <select value={cloudModel} onChange={(e) => chooseCloudModel(e.target.value)}>
                        {cloudModels.map((m) => <option key={m.id} value={m.id}>{m.id}</option>)}
                      </select>
                    ) : (
                      <input value={cloudModel} onChange={(e) => chooseCloudModel(e.target.value)} placeholder={provider?.defaultModel} />
                    )}
                    <small className="field-help">Pick a strong reasoning model; each trial costs one short request with the full measurement history.</small>
                  </label>
                  <div className="runtime-side-actions">
                    <button className="button secondary" onClick={probeCloud} disabled={busy === "probe" || !cloudModel}><Activity size={15} /> Test connection</button>
                    <button className="button secondary" onClick={forgetKey} disabled={busy === "cloud" || tuning}><Unplug size={15} /> Forget key</button>
                  </div>
                  {cloudCheck && <p className={cloudCheck.startsWith("Connected") ? "cloud-check ok" : "cloud-check"}>{cloudCheck}</p>}
                </>
              )}
            </div>
          </article>

          <article className="machine-panel">
            <div className="panel-title"><Settings2 size={17} /><h2>Tuning target</h2></div>
            <div className="option-grid">
              <label>Context length
                <select value={tuneContext} onChange={(e) => setTuneContext(Number(e.target.value))} disabled={tuning}>
                  {contextChoices(gguf?.contextLength ?? null).map((value) => <option key={value} value={value}>{value.toLocaleString()}{gguf?.contextLength === value ? " (native max)" : ""}</option>)}
                </select>
                <small className="field-help">Every trial runs at exactly this context; the KV cache is sized for it.</small>
              </label>
              <label>AI trials
                <input type="number" min="1" max="12" value={tuneTrials} onChange={(e) => setTuneTrials(Number(e.target.value))} disabled={tuning} />
                <small className="field-help">Proposals measured after the baseline (T0). Each is one server launch.</small>
              </label>
              <label>Tokens per measurement<input type="number" min="64" max="2048" step="64" value={tuneTokens} onChange={(e) => setTuneTokens(Number(e.target.value))} disabled={tuning} /></label>
              <label>Repeats per trial<input type="number" min="1" max="5" value={tuneRepeats} onChange={(e) => setTuneRepeats(Number(e.target.value))} disabled={tuning} /></label>
            </div>
            <dl className="spec-list">
              <div><dt>Model</dt><dd>{selected?.name ?? "—"}</dd></div>
              <div><dt>Architecture</dt><dd>{gguf ? `${gguf.architecture.toUpperCase()} · ${gguf.sizeLabel || "?"}${gguf.expertCount ? ` · ${gguf.expertCount} EXPERTS` : ""}` : "—"}</dd></div>
              <div><dt>Layers / native context</dt><dd>{gguf ? `${gguf.blockCount ?? "?"} / ${gguf.contextLength?.toLocaleString() ?? "?"}` : "—"}</dd></div>
              <div><dt>Companions</dt><dd>{selected?.companions.length ? selected.companions.map((c) => c.role.toUpperCase()).join(" · ") : "NONE"}</dd></div>
              <div><dt>Runtime</dt><dd>{runtime ? `B${runtime.build} · ${runtimeIdentity?.backend.toUpperCase() ?? "?"}` : "—"}</dd></div>
              <div><dt>Hardware</dt><dd>{hardware?.gpuNames[0]?.toUpperCase() ?? hardware?.vendor.toUpperCase() ?? "—"}</dd></div>
            </dl>
            <p className="group-note tune-note">The advisor may change only throughput-relevant fields (offload, threads, batching, KV cache type, flash attention, speculative settings). Host, port, alias, paths, and security settings are never touched. Nothing runs on this PC except llama-server with the proposed flags.</p>
          </article>
        </div>

        <aside className="tune-results">
          <article className="machine-panel">
            <div className="panel-title"><Trophy size={17} /><h2>Result</h2>{tuneReport && <span className="state-tag good">FINISHED</span>}{tuning && <span className="state-tag warning">RUNNING</span>}{(tuning ? tuneRun : tuneReportOrigin) && <small>{tuningRunSummary((tuning ? tuneRun : tuneReportOrigin)!)}</small>}</div>
            {bestLive ? (
              <>
                <div className="result-main"><strong>{bestLive.meanTps?.toFixed(2)}</strong><span>best generation tok/s{trialsForDisplay[0]?.meanTps && bestLive.index !== 0 ? ` · ${(bestLive.meanTps ?? 0) >= (trialsForDisplay[0].meanTps ?? 0) ? "+" : ""}${(((bestLive.meanTps ?? 0) / (trialsForDisplay[0].meanTps ?? 1) - 1) * 100).toFixed(1)}% vs baseline` : " · baseline holds"}</span></div>
                <div className="result-range"><span>Baseline {trialsForDisplay[0]?.meanTps?.toFixed(2) ?? "—"}</span><span>Best: trial {bestLive.index} of {trialsForDisplay.length - 1}</span><span>{tuneContext.toLocaleString()} ctx</span></div>
                <div className="sample-bars trial-bars">
                  {trialsForDisplay.map((trial) => (
                    <div key={trial.index} className={trial.index === bestLive.index ? "best" : trial.meanTps === null ? "failed" : ""} title={trial.error ?? trial.rationale}>
                      <span>T{trial.index}</span>
                      <b style={{ width: `${trial.meanTps ? (trial.meanTps / (bestLive.meanTps ?? 1)) * 100 : 2}%` }} />
                      <em>{trial.meanTps ? trial.meanTps.toFixed(1) : "FAIL"}</em>
                    </div>
                  ))}
                </div>
                {tuneReport && <div className="tune-actions"><button className="button primary" onClick={adoptTunedProfile} disabled={tuneReport.bestIndex === null}><Save size={15} /> Adopt best as profile</button><small>{tuneReport.stoppedReason}</small>
                  <small>{tuneReport.objective ?? "Measured objective: short-prompt decode throughput at the allocated context."}</small>
                  {tuneReport.finalVerification ? <small>Final verification: baseline {tuneReport.finalVerification.baselineTps.toFixed(2)} tok/s, winner {tuneReport.finalVerification.winnerTps.toFixed(2)} tok/s, required +{(tuneReport.finalVerification.requiredImprovement * 100).toFixed(1)}% — {tuneReport.finalVerification.confirmed ? "confirmed" : "not confirmed"}.</small> : null}
                  {tuneReport.qualityAffectingChanges && tuneReport.qualityAffectingChanges.length > 0 ? <small>Quality not measured for: {tuneReport.qualityAffectingChanges.join(", ")}. Run the quality suite before adopting output-quality-sensitive changes.</small> : null}</div>}
              </>
            ) : (
              <div className="empty-result tune-empty"><Sparkles size={28} /><p>{tuning ? tuneProgress?.message ?? "Starting…" : tuneBlocker}</p></div>
            )}
          </article>

          <article className="machine-panel terminal-panel tune-log">
            <div className="panel-title"><SquareTerminal size={17} /><h2>Trial log</h2>{tuning && tuneProgress && <span className="log-path">{tuneProgress.message}</span>}</div>
            <div className="trial-list" ref={tuneLogRef}>
              {trialsForDisplay.length === 0 && <div className="log-empty"><Sparkles size={26} /><strong>No trials yet</strong><span>Start tuning to see each proposal, its rationale, and its measurement.</span></div>}
              {trialsForDisplay.map((trial) => (
                <div key={trial.index} className={`trial-entry${trial.meanTps === null ? " failed" : ""}${bestLive?.index === trial.index ? " best" : ""}`}>
                  <div className="trial-head"><span className="trial-index">T{trial.index}</span><code>{describeChanges(trial.changes)}</code><strong>{trial.meanTps !== null ? `${trial.meanTps.toFixed(2)} tok/s` : "FAILED"}</strong></div>
                  <p>{trial.rationale}</p>
                  {trial.error && <pre className="trial-error">{trial.error}</pre>}
                </div>
              ))}
              {tuning && tuneProgress && !tuneProgress.trial && <div className="trial-entry pending"><div className="trial-head"><RefreshCw size={12} className="spin" /><code>{tuneProgress.message}</code></div></div>}
            </div>
          </article>
        </aside>
      </div>
    </section>
  );
}
