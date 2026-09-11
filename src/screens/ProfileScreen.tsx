import type { Dispatch, SetStateAction } from "react";
import { Braces, Play, Save } from "lucide-react";
import {
  formatExtraArgs,
  parseExtraArgs,
  reconcileDraftCompanion,
  type CommandPreview,
  type LaunchProfile,
  type LogicalModel,
  type RuntimeCapabilities,
  type ServerStatus,
} from "../model";
import type { EvidenceRun } from "./evidence-run";

/// Presentational contract for the launch-profile editor (audit S-27.I2).
/// Acquisition, persistence and preview composition stay in `App.tsx`.
export interface ProfileScreenProps {
  busy: string;
  command: CommandPreview | null;
  evidenceRun: EvidenceRun | null;
  extraArgsDraft: string | null;
  inspect: () => void;
  profile: LaunchProfile;
  runtime: RuntimeCapabilities | null;
  saveProfile: () => void;
  selected: LogicalModel | undefined;
  setExtraArgsDraft: Dispatch<SetStateAction<string | null>>;
  setNotice: Dispatch<SetStateAction<string>>;
  setProfile: Dispatch<SetStateAction<LaunchProfile | null>>;
  setRuntime: Dispatch<SetStateAction<RuntimeCapabilities | null>>;
  setRuntimePath: Dispatch<SetStateAction<string>>;
  start: () => void;
  tuning: boolean;
  status: ServerStatus;
}

export function ProfileScreen(props: ProfileScreenProps) {
  const {
    busy,
    command,
    evidenceRun,
    extraArgsDraft,
    inspect,
    profile,
    runtime,
    saveProfile,
    selected,
    setExtraArgsDraft,
    setNotice,
    setProfile,
    setRuntime,
    setRuntimePath,
    start,
    tuning,
  } = props;
  return (
    <section className="screen profile-screen">
      <div className="section-heading">
        <div>
          <h1>Launch profile</h1>
          <p>Every field becomes an explicit llama-server argument.</p>
        </div>
        <div className="actions">
          <button className="button secondary" onClick={saveProfile}><Save size={16} /> Save</button>
          <button className="button primary" onClick={start} disabled={!selected?.complete || !profile.runtime || busy === "start" || evidenceRun !== null || tuning}><Play size={16} fill="currentColor" /> Start</button>
        </div>
      </div>

      <div className="profile-layout">
        <div className="settings-stack">
          <div className="profile-guide">
            <div><strong>Essentials first</strong><span>Common fit, speed, acceleration, and chat controls stay visible.</span></div>
            <button
            onClick={() => {
              const advanced = document.querySelector(".advanced-zone") as HTMLDetailsElement | null;
              if (!advanced) return;
              advanced.open = true;
              // S-24 I3: honour the reduced-motion preference instead of
              // forcing a smooth scroll, and put keyboard focus on the
              // revealed region so the next Tab stop is inside it.
              const reduced =
                window.matchMedia?.("(prefers-reduced-motion: reduce)").matches ?? false;
              advanced.scrollIntoView({ behavior: reduced ? "auto" : "smooth", block: "start" });
              // The region is open synchronously, so the summary can
              // take focus immediately (no animation frame needed).
              const summary = advanced.querySelector("summary") as HTMLElement | null;
              summary?.focus();
            }}
          >
            Jump to Advanced
          </button>
          </div>
          <fieldset>
            <legend>Identity & runtime</legend>
            <label className="wide">Profile name<input value={profile.name} onChange={(e) => setProfile({ ...profile, name: e.target.value })} /></label>
            <label className="wide">llama-server executable<input value={profile.runtime} onChange={(e) => {
              // Editing the committed path clears stale capabilities
              // until the new path is inspected (audit FE-15).
              if (e.target.value !== profile.runtime) setRuntime(null);
              setRuntimePath(e.target.value);
              setProfile({ ...profile, runtime: e.target.value });
            }} /></label>
            <button className="inline-action" onClick={() => inspect()} disabled={busy === "runtime"}>Inspect selected runtime</button>
            <label>Host<input value={profile.host} onChange={(e) => setProfile({ ...profile, host: e.target.value })} /></label>
            <label>Port<input type="number" value={profile.port} onChange={(e) => setProfile({ ...profile, port: Number(e.target.value) })} /></label>
            <label className="wide">API alias<input value={profile.alias} onChange={(e) => setProfile({ ...profile, alias: e.target.value })} /></label>
          </fieldset>

          <fieldset>
            <legend>Everyday performance</legend>
            <p className="group-note">The settings most likely to affect whether a model fits and how fast it runs.</p>
            <label>Context tokens<input type="number" value={profile.context} onChange={(e) => setProfile({ ...profile, context: Number(e.target.value) })} /><small className="field-help">Shared by prompt and output.</small></label>
            <label>Parallel slots<input type="number" value={profile.parallel} onChange={(e) => setProfile({ ...profile, parallel: Number(e.target.value) })} /><small className="field-help">Use 1 for best single-user latency.</small></label>
            <label>GPU layers<input value={profile.gpuLayers} onChange={(e) => setProfile({ ...profile, gpuLayers: e.target.value })} /><small className="field-help">auto, all, or a layer count.</small></label>
            <label>CPU MoE layers<input type="number" value={profile.cpuMoe} onChange={(e) => setProfile({ ...profile, cpuMoe: Number(e.target.value) })} /><small className="field-help">Useful for large mixture-of-experts models.</small></label>
            <label>Flash attention
              <select value={profile.flashAttention} onChange={(e) => setProfile({ ...profile, flashAttention: e.target.value as LaunchProfile["flashAttention"] })}>
                <option value="auto">Automatic</option><option value="on">On</option><option value="off">Off</option>
              </select>
            </label>
            <label>Model loading
              <select value={profile.loadMode} onChange={(e) => setProfile({ ...profile, loadMode: e.target.value })}>
                <option value="auto">Automatic</option><option value="mmap">Memory map</option><option value="mmap+mlock">Map + lock in RAM</option><option value="mlock">Lock in RAM</option><option value="none">Load normally</option><option value="dio">Direct I/O</option>
              </select>
            </label>
            <label>KV cache K
              <select value={profile.cacheTypeK} onChange={(e) => setProfile({ ...profile, cacheTypeK: e.target.value })}>{["f16","bf16","q8_0","q4_0","q4_1","iq4_nl","q5_0","q5_1","f32"].map((type) => <option key={type}>{type}</option>)}</select>
            </label>
            <label>KV cache V
              <select value={profile.cacheTypeV} onChange={(e) => setProfile({ ...profile, cacheTypeV: e.target.value })}>{["f16","bf16","q8_0","q4_0","q4_1","iq4_nl","q5_0","q5_1","f32"].map((type) => <option key={type}>{type}</option>)}</select>
            </label>
            <label className="toggle-line"><input type="checkbox" checked={profile.fit} onChange={(e) => setProfile({ ...profile, fit: e.target.checked })} /> Smart memory fit</label>
            <label className="toggle-line"><input type="checkbox" checked={profile.kvOffload} onChange={(e) => setProfile({ ...profile, kvOffload: e.target.checked })} /> Keep KV cache on GPU</label>
          </fieldset>

          <fieldset>
            <legend>Acceleration & model features</legend>
            <label className="wide">Speculative method
              <select value={profile.specType} onChange={(e) => {
                const reconciled = reconcileDraftCompanion(e.target.value, profile.draftModel ?? null);
                setProfile({ ...profile, specType: e.target.value, draftModel: reconciled.draftModelPath });
                if (reconciled.cleared) {
                  setNotice(`${e.target.value} does not use a draft model; the retained companion path was cleared.`);
                }
              }}>
                {(runtime?.specTypes ?? ["none", "draft-mtp", "draft-dspark", "ngram-mod", "ngram-simple", "ngram-map-k", "ngram-map-k4v"]).map((type) => <option key={type}>{type}</option>)}
                {!runtime && <small className="field-help">Not inspected — this list is provisional until the selected runtime is inspected.</small>}
              </select>
            </label>
            <label className="wide">Draft / companion GGUF
              {(() => {
                const matching = (selected?.companions ?? []).filter((c) => c.role !== "mmproj");
                if (matching.length < 2) {
                  return <input value={profile.draftModel ?? ""} onChange={(e) => setProfile({ ...profile, draftModel: e.target.value || null })} />;
                }
                const known = matching.some((c) => c.path === profile.draftModel);
                return (
                  <select value={known ? (profile.draftModel ?? "") : ""} onChange={(e) => setProfile({ ...profile, draftModel: e.target.value || null })}>
                    <option value="">None</option>
                    {matching.map((c) => <option key={c.path} value={c.path}>{c.role.toUpperCase()} · {c.name}</option>)}
                  </select>
                );
              })()}
              <small className="field-help">Required by DSpark, DFlash, EAGLE-3, and simple draft models.{(selected?.companions ?? []).filter((c) => c.role !== "mmproj").length > 1 ? ` This family ships ${(selected?.companions ?? []).filter((c) => c.role !== "mmproj").length} drafts; the closest quantisation to ${selected?.quant ?? "the target"} is listed first.` : ""}</small>
            </label>
            <label>Maximum draft tokens<input type="number" value={profile.draftMax} onChange={(e) => setProfile({ ...profile, draftMax: Number(e.target.value) })} /></label>
            <label>Vision projector<input value={profile.mmproj ?? ""} onChange={(e) => setProfile({ ...profile, mmproj: e.target.value || null })} /></label>
          </fieldset>

          <fieldset>
            <legend>Chat behavior</legend>
            <label>Reasoning
              <select value={profile.reasoning} onChange={(e) => setProfile({ ...profile, reasoning: e.target.value })}><option value="auto">Automatic</option><option value="on">On</option><option value="off">Off</option></select>
            </label>
            <label>Reasoning effort
              <select value={profile.reasoningEffort} onChange={(e) => setProfile({ ...profile, reasoningEffort: e.target.value })}>{["default","minimal","low","medium","high","xhigh","max"].map((level) => <option key={level}>{level}</option>)}</select>
            </label>
            <label className="toggle-line"><input type="checkbox" checked={profile.jinja} onChange={(e) => setProfile({ ...profile, jinja: e.target.checked })} /> Jinja chat templates</label>
            <label className="toggle-line"><input type="checkbox" checked={profile.cachePrompt} onChange={(e) => setProfile({ ...profile, cachePrompt: e.target.checked })} /> Reuse prompt cache</label>
          </fieldset>

          <details className="advanced-zone">
            <summary><span>Advanced options</span><small>Fine-tune placement, cache, sampling, security, vision, and adapters</small></summary>
            <div className="advanced-content">
              <details className="option-group">
                <summary>CPU, batching & throughput</summary>
                <div className="option-grid">
                  <label>Generation threads<input type="number" value={profile.threads} onChange={(e) => setProfile({ ...profile, threads: Number(e.target.value) })} /><small className="field-help">-1 lets llama.cpp decide.</small></label>
                  <label>Prompt threads<input type="number" value={profile.threadsBatch} onChange={(e) => setProfile({ ...profile, threadsBatch: Number(e.target.value) })} /></label>
                  <label>Batch size<input type="number" value={profile.batch} onChange={(e) => setProfile({ ...profile, batch: Number(e.target.value) })} /></label>
                  <label>Physical uBatch<input type="number" value={profile.ubatch} onChange={(e) => setProfile({ ...profile, ubatch: Number(e.target.value) })} /></label>
                  <label>Dense CPU FFN layers<input type="number" value={profile.cpuFfn} onChange={(e) => setProfile({ ...profile, cpuFfn: Number(e.target.value) })} /></label>
                  <label>HTTP threads<input type="number" value={profile.threadsHttp} onChange={(e) => setProfile({ ...profile, threadsHttp: Number(e.target.value) })} /></label>
                  <label className="toggle-line"><input type="checkbox" checked={profile.continuousBatching} onChange={(e) => setProfile({ ...profile, continuousBatching: e.target.checked })} /> Continuous batching</label>
                  <label className="toggle-line"><input type="checkbox" checked={profile.warmup} onChange={(e) => setProfile({ ...profile, warmup: e.target.checked })} /> Startup warmup</label>
                </div>
              </details>

              <details className="option-group">
                <summary>GPU placement & memory fitting</summary>
                <div className="option-grid">
                  <label>Split mode<select value={profile.splitMode} onChange={(e) => setProfile({ ...profile, splitMode: e.target.value })}><option value="none">Single GPU</option><option value="layer">Layer split</option><option value="row">Row split</option><option value="tensor">Tensor split (experimental)</option></select></label>
                  <label>Tensor split<input value={profile.tensorSplit} onChange={(e) => setProfile({ ...profile, tensorSplit: e.target.value })} placeholder="e.g. 3,1" /></label>
                  <label>Main GPU<input type="number" value={profile.mainGpu} onChange={(e) => setProfile({ ...profile, mainGpu: Number(e.target.value) })} /></label>
                  <label>Devices<input value={profile.device} onChange={(e) => setProfile({ ...profile, device: e.target.value })} placeholder="blank = automatic" /></label>
                  <label>Fit margin MiB<input value={profile.fitTarget} onChange={(e) => setProfile({ ...profile, fitTarget: e.target.value })} /></label>
                  <label>Minimum fit context<input type="number" value={profile.fitCtx} onChange={(e) => setProfile({ ...profile, fitCtx: Number(e.target.value) })} /></label>
                  {runtime?.supportedFlags.includes("--lazy-mode") && <label>Lazy tensor loading<select value={profile.lazyMode} onChange={(e) => setProfile({ ...profile, lazyMode: e.target.value })}><option value="auto">Automatic</option><option value="on">On</option><option value="off">Off</option></select></label>}
                </div>
              </details>

              <details className="option-group">
                <summary>Prompt cache & server lifecycle</summary>
                <div className="option-grid">
                  <label>Cache RAM MiB<input type="number" value={profile.cacheRam} onChange={(e) => setProfile({ ...profile, cacheRam: Number(e.target.value) })} /></label>
                  <label>Cache reuse chunk<input type="number" value={profile.cacheReuse} onChange={(e) => setProfile({ ...profile, cacheReuse: Number(e.target.value) })} /></label>
                  <label>Context checkpoints<input type="number" value={profile.contextCheckpoints} onChange={(e) => setProfile({ ...profile, contextCheckpoints: Number(e.target.value) })} /></label>
                  <label>Sleep after idle seconds<input type="number" value={profile.sleepIdleSeconds} onChange={(e) => setProfile({ ...profile, sleepIdleSeconds: Number(e.target.value) })} /></label>
                  <label>Request timeout seconds<input type="number" value={profile.timeout} onChange={(e) => setProfile({ ...profile, timeout: Number(e.target.value) })} /></label>
                  <label>SSE ping seconds<input type="number" value={profile.ssePingInterval} onChange={(e) => setProfile({ ...profile, ssePingInterval: Number(e.target.value) })} /></label>
                  <label className="toggle-line"><input type="checkbox" checked={profile.contextShift} onChange={(e) => setProfile({ ...profile, contextShift: e.target.checked })} /> Infinite-generation context shift</label>
                  <label className="toggle-line"><input type="checkbox" checked={profile.slots} onChange={(e) => setProfile({ ...profile, slots: e.target.checked })} /> Slots monitoring endpoint</label>
                  <label className="toggle-line"><input type="checkbox" checked={profile.metrics} onChange={(e) => setProfile({ ...profile, metrics: e.target.checked })} /> Prometheus metrics</label>
                  <label className="toggle-line"><input type="checkbox" checked={profile.webUi} onChange={(e) => setProfile({ ...profile, webUi: e.target.checked })} /> llama.cpp WebUI</label>
                </div>
              </details>

              <details className="option-group">
                <summary>Speculative fine tuning</summary>
                <div className="option-grid">
                  <label>Minimum draft tokens<input type="number" value={profile.draftMin} onChange={(e) => setProfile({ ...profile, draftMin: Number(e.target.value) })} /></label>
                  <label>Draft probability floor<input type="number" step="0.01" value={profile.draftPMin} onChange={(e) => setProfile({ ...profile, draftPMin: Number(e.target.value) })} /></label>
                  <label>Draft split probability<input type="number" step="0.01" value={profile.draftPSplit} onChange={(e) => setProfile({ ...profile, draftPSplit: Number(e.target.value) })} /></label>
                  <label>Draft GPU layers<input value={profile.draftGpuLayers} onChange={(e) => setProfile({ ...profile, draftGpuLayers: e.target.value })} /></label>
                  <label>Draft cache K<select value={profile.draftCacheTypeK} onChange={(e) => setProfile({ ...profile, draftCacheTypeK: e.target.value })}>{["f16","bf16","q8_0","q4_0","q4_1"].map((type) => <option key={type}>{type}</option>)}</select></label>
                  <label>Draft cache V<select value={profile.draftCacheTypeV} onChange={(e) => setProfile({ ...profile, draftCacheTypeV: e.target.value })}>{["f16","bf16","q8_0","q4_0","q4_1"].map((type) => <option key={type}>{type}</option>)}</select></label>
                  <label>N-gram match length<input type="number" value={profile.ngramMatch} onChange={(e) => setProfile({ ...profile, ngramMatch: Number(e.target.value) })} /></label>
                  <div className="paired-inputs">
                    <label>N-gram draft min<input type="number" value={profile.ngramMin} onChange={(e) => setProfile({ ...profile, ngramMin: Number(e.target.value) })} /></label>
                    <label>N-gram draft max<input type="number" value={profile.ngramMax} onChange={(e) => setProfile({ ...profile, ngramMax: Number(e.target.value) })} /></label>
                  </div>
                  <div className="paired-inputs">
                    <label>N-gram map size n<input type="number" value={profile.ngramSizeN} onChange={(e) => setProfile({ ...profile, ngramSizeN: Number(e.target.value) })} /></label>
                    <label>N-gram map size m<input type="number" value={profile.ngramSizeM} onChange={(e) => setProfile({ ...profile, ngramSizeM: Number(e.target.value) })} /></label>
                  </div>
                  <label>Map minimum hits<input type="number" value={profile.ngramMinHits} onChange={(e) => setProfile({ ...profile, ngramMinHits: Number(e.target.value) })} /></label>
                </div>
              </details>

              <details className="option-group">
                <summary>Sampling defaults</summary>
                <div className="option-grid">
                  <label>Temperature<input type="number" step="0.05" value={profile.temperature} onChange={(e) => setProfile({ ...profile, temperature: Number(e.target.value) })} /></label>
                  <label>Top K<input type="number" value={profile.topK} onChange={(e) => setProfile({ ...profile, topK: Number(e.target.value) })} /></label>
                  <label>Top P<input type="number" step="0.01" value={profile.topP} onChange={(e) => setProfile({ ...profile, topP: Number(e.target.value) })} /></label>
                  <label>Min P<input type="number" step="0.01" value={profile.minP} onChange={(e) => setProfile({ ...profile, minP: Number(e.target.value) })} /></label>
                  <label>Repeat penalty<input type="number" step="0.05" value={profile.repeatPenalty} onChange={(e) => setProfile({ ...profile, repeatPenalty: Number(e.target.value) })} /></label>
                  <label>Repeat window<input type="number" value={profile.repeatLastN} onChange={(e) => setProfile({ ...profile, repeatLastN: Number(e.target.value) })} /></label>
                  <label>Seed<input type="number" value={profile.seed} onChange={(e) => setProfile({ ...profile, seed: Number(e.target.value) })} /></label>
                  <label>DRY multiplier<input type="number" step="0.05" value={profile.dryMultiplier} onChange={(e) => setProfile({ ...profile, dryMultiplier: Number(e.target.value) })} /></label>
                  <label>DRY base<input type="number" step="0.05" value={profile.dryBase} onChange={(e) => setProfile({ ...profile, dryBase: Number(e.target.value) })} /></label>
                </div>
              </details>

              <details className="option-group">
                <summary>Vision & multimodal</summary>
                <div className="option-grid">
                  <label className="toggle-line"><input type="checkbox" checked={profile.mmprojOffload} onChange={(e) => setProfile({ ...profile, mmprojOffload: e.target.checked })} /> Offload projector to GPU</label>
                  <label>Projector device<input value={profile.mmprojDevice} onChange={(e) => setProfile({ ...profile, mmprojDevice: e.target.value })} /></label>
                  <label>Minimum image tokens<input type="number" value={profile.imageMinTokens} onChange={(e) => setProfile({ ...profile, imageMinTokens: Number(e.target.value) })} /></label>
                  <label>Maximum image tokens<input type="number" value={profile.imageMaxTokens} onChange={(e) => setProfile({ ...profile, imageMaxTokens: Number(e.target.value) })} /></label>
                </div>
              </details>

              <details className="option-group security-group">
                <summary>Network & security</summary>
                <p className="security-note">LAN binding requires an API key file and restricted CORS origins. Secrets are read from a file rather than exposed in the command line.</p>
                <div className="option-grid">
                  <label className="wide">Allowed CORS origins<input value={profile.corsOrigins} onChange={(e) => setProfile({ ...profile, corsOrigins: e.target.value })} /></label>
                  <label className="wide">API key file<input value={profile.apiKeyFile} onChange={(e) => setProfile({ ...profile, apiKeyFile: e.target.value })} /></label>
                  <label>SSL private key<input value={profile.sslKeyFile} onChange={(e) => setProfile({ ...profile, sslKeyFile: e.target.value })} /></label>
                  <label>SSL certificate<input value={profile.sslCertFile} onChange={(e) => setProfile({ ...profile, sslCertFile: e.target.value })} /></label>
                </div>
              </details>

              <details className="option-group">
                <summary>Templates, adapters & low-level overrides</summary>
                <div className="option-grid">
                  <label>Reasoning token budget<input type="number" value={profile.reasoningBudget} onChange={(e) => setProfile({ ...profile, reasoningBudget: Number(e.target.value) })} /></label>
                  <label className="toggle-line"><input type="checkbox" checked={profile.reasoningPreserve} onChange={(e) => setProfile({ ...profile, reasoningPreserve: e.target.checked })} /> Preserve reasoning history</label>
                  <label className="wide">Custom chat template file<input value={profile.chatTemplateFile} onChange={(e) => setProfile({ ...profile, chatTemplateFile: e.target.value })} /></label>
                  <label className="wide">LoRA files<input value={profile.lora} onChange={(e) => setProfile({ ...profile, lora: e.target.value })} placeholder="comma-separated paths" /></label>
                  <label className="wide">Scaled LoRAs<input value={profile.loraScaled} onChange={(e) => setProfile({ ...profile, loraScaled: e.target.value })} placeholder="path:scale,..." /></label>
                  <label className="wide">Tensor overrides<input value={profile.overrideTensor} onChange={(e) => setProfile({ ...profile, overrideTensor: e.target.value })} /></label>
                  <label className="wide">Model metadata overrides<input value={profile.overrideKv} onChange={(e) => setProfile({ ...profile, overrideKv: e.target.value })} /></label>
                  <label>Log verbosity<input type="number" min="0" max="5" value={profile.verbosity} onChange={(e) => setProfile({ ...profile, verbosity: Number(e.target.value) })} /></label>
                  <label className="toggle-line"><input type="checkbox" checked={profile.logTimestamps} onChange={(e) => setProfile({ ...profile, logTimestamps: e.target.checked })} /> Log timestamps</label>
                  <label className="wide">
                    Raw extra arguments
                    <input
                      value={extraArgsDraft ?? formatExtraArgs(profile.extraArgs)}
                      onChange={(event) => setExtraArgsDraft(event.target.value)}
                      onFocus={() => setExtraArgsDraft(formatExtraArgs(profile.extraArgs))}
                      onBlur={() => {
                        if (extraArgsDraft !== null) {
                          setProfile({ ...profile, extraArgs: parseExtraArgs(extraArgsDraft) });
                          setExtraArgsDraft(null);
                        }
                      }}
                      onKeyDown={(event) => {
                        if (event.key === "Enter" && extraArgsDraft !== null) {
                          setProfile({ ...profile, extraArgs: parseExtraArgs(extraArgsDraft) });
                          setExtraArgsDraft(null);
                        }
                      }}
                    />
                    <small className="field-help">Use self-contained `--flag` or `--flag=value` tokens; quote values that contain spaces. The list is applied when you leave the field. Typed-field overrides and privileged capabilities are rejected.</small>
                  </label>
                </div>
              </details>
            </div>
          </details>
        </div>

        <aside className="command-preview">
          <div className="panel-title"><Braces size={17} /><h2>Provisional command</h2></div>
          {command ? (
            <>
              <p className="field-help">PowerShell form (paste-ready):</p>
              <pre>{command.powerShell}</pre>
              {command.cmd && (
                <>
                  <p className="field-help">cmd.exe form:</p>
                  <pre>{command.cmd}</pre>
                </>
              )}
              {command.cmdNotice && <small className="field-help">{command.cmdNotice}</small>}
              {command.argv && (
                <>
                  <p className="field-help">argv (JSON array — lossless for any wrapper):</p>
                  <pre>{command.argv}</pre>
                </>
              )}
            </>
          ) : (
            <pre>Edit a setting to compose the command.</pre>
          )}
          <p className="security-note">Composed without probing the runtime: capability filtering, artifact checks, and managed-runtime trust are enforced when the profile is validated and launched.</p>
          <div className="capability-list">
            <span>RUNTIME CAPABILITIES</span>
            {(runtime?.specTypes ?? []).map((type) => <i key={type}>{type}</i>)}
          </div>
          <p>Only methods advertised by this executable are offered. Raw arguments remain available for new llama.cpp features.</p>
        </aside>
      </div>
    </section>
  );
}
