import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  Activity,
  ArrowUp,
  BadgeCheck,
  Box,
  Braces,
  CircleStop,
  Cpu,
  Database,
  Download,
  ExternalLink,
  FolderOpen,
  Gauge,
  HardDrive,
  Info,
  KeyRound,
  Link2,
  LogIn,
  MonitorCog,
  Play,
  RefreshCw,
  Save,
  Settings2,
  ShieldCheck,
  Sparkles,
  Square,
  SquareTerminal,
  TestTube2,
  TriangleAlert,
  Trophy,
  Unplug,
  Wrench,
} from "lucide-react";
import "./App.css";
import {
  bytesLabel,
  catalogRevision,
  conflictingCapacityMetrics,
  contextChoices,
  DEFAULT_FIT_PER_MILLE,
  describeChanges,
  downloadKey,
  newestDownloadJob,
  activeDownloadJob,
  formatExtraArgs,
  parseExtraArgs,
  downloadPercent,
  downloadReadiness,
  etaLabel,
  hardwareFitBudget,
  keepLatestRequest,
  evidenceRunLabel,
  tuningRunSummary,
  selectionAfterRescan,
  displayedSelection,
  profileIdentity,
  applySuggestedPort,
  responseIsCurrent,
  managedHealthRequest,
  managedHealthOutcome,
  normalizeProfile,
  normalizeTuningReport,
  safeJsonParse,
  originLabel,
  rateLabel,
  retainOrDisposeListener,
  runtimeCatalogErrorFromUnknown,
  runtimeCatalogViewState,
  runtimeInstallRequest,
  runtimeOptionState,
  suggestedProfile,
  type BenchmarkSummary,
  type CloudModel,
  type CloudProvider,
  type CredentialStatus,
  type GgufSummary,
  type HardwareInfo,
  type HealthModelProgress,
  type InstalledRuntime,
  type ManagedHealthResult,
  type LaunchProfile,
  type LogicalModel,
  type ManagedRuntimeRecord,
  type RuntimeCatalog,
  type RuntimeCatalogError,
  type RuntimeCapabilities,
  type RuntimeIdentity,
  type RuntimeInstallProgress,
  type RuntimeOption,
  type RuntimeSetupResponse,
  type ServerStatus,
  type TuningProgress,
  type TuningReport,
  type TuningTrial,
  type AboutInfo,
  type CatalogFile,
  type DownloadJob,
  type CatalogModel,
  type CatalogQuery,
  type CatalogSnapshot,
  type CatalogFacets,
  type DownloadEvent,
  type FitBudget,
  type TokenStatus,
} from "./model";
import { V03EvidencePanel } from "./V03EvidencePanel";

type View = "dashboard" | "models" | "catalog" | "runtime" | "profile" | "tune" | "benchmark" | "about";

const readRecord = (key: string): string | null => {
  // Upgrades read records saved under the previous product prefix once.
  // New writes use the current prefix; the old value stays for downgrade.
  try {
    const current = localStorage.getItem(`localmotive:${key}`);
    if (current !== null) return current;
    return localStorage.getItem(`gguf-pilot:${key}`);
  } catch {
    return null;
  }
};
/// Move a record that cannot be parsed or validated out of the way
/// (audit FE-09): the raw text is kept under a quarantine key, the live
/// key is cleared, and the caller shows a notice instead of crashing.
function quarantineRecord(key: string, raw: string) {
  try {
    localStorage.setItem(`localmotive:quarantine:${key}:${Date.now()}`, raw);
    localStorage.removeItem(`localmotive:${key}`);
    localStorage.removeItem(`gguf-pilot:${key}`);
  } catch {
    // Storage may be unavailable; the caller still falls back safely.
  }
}

const readSetting = (key: string): string => readRecord(key) ?? "";

const MODEL_ROOT = readSetting("model-root");
const RUNTIME = readSetting("runtime");
const CLOUD_PROVIDER = readSetting("cloud-provider") || "openrouter";
const CLOUD_MODEL = readSetting("cloud-model");
const inTauri = () => Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);
const idleStatus: ServerStatus = {
  running: false,
  phase: "idle",
  pid: null,
  profileName: null,
  alias: null,
  port: null,
  command: null,
  logPath: null,
  startedAt: null,
  exitCode: null,
  resultClass: "unknown",
  validation: null,
  failure: null,
};

function App() {
  const [view, setView] = useState<View>(RUNTIME ? "dashboard" : "runtime");
  // FE-05: the active evidence run's status and cancel handle, published by
  // the always-mounted evidence panel so any screen can show it.
  const [evidenceRun, setEvidenceRun] = useState<{ kind: "benchmark" | "quality"; cancel: (() => void) | null } | null>(null);
  const [extraArgsDraft, setExtraArgsDraft] = useState<string | null>(null);
  // FE-11: every download carries the destination and revision captured at
  // start, so cancel and progress always address the job itself.
  const [downloadJobs, setDownloadJobs] = useState<DownloadJob[]>([]);
  const [modelRoot, setModelRoot] = useState(MODEL_ROOT);
  const [runtimePath, setRuntimePath] = useState(RUNTIME);
  const [hardware, setHardware] = useState<HardwareInfo | null>(null);
  const [selectedRuntimeAdapterId, setSelectedRuntimeAdapterId] = useState("");
  const [runtimeCatalog, setRuntimeCatalog] = useState<RuntimeCatalog | null>(null);
  const [runtimeCatalogError, setRuntimeCatalogError] = useState<RuntimeCatalogError | null>(null);
  const [runtimeCatalogLoading, setRuntimeCatalogLoading] = useState(false);
  const [runtimeRoot, setRuntimeRoot] = useState("");
  const [installing, setInstalling] = useState("");
  const [runtimeInstallCancelling, setRuntimeInstallCancelling] = useState(false);
  const [runtimeInstallProgress, setRuntimeInstallProgress] = useState<RuntimeInstallProgress | null>(null);
  const [healthRunning, setHealthRunning] = useState("");
  const [healthCancelling, setHealthCancelling] = useState(false);
  const [healthModelProgress, setHealthModelProgress] = useState<HealthModelProgress | null>(null);
  const [healthRepairing, setHealthRepairing] = useState(false);
  const [healthRepairNotice, setHealthRepairNotice] = useState<string | null>(null);
  const [healthResult, setHealthResult] = useState<ManagedHealthResult | null>(null);
  const [models, setModels] = useState<LogicalModel[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [profile, setProfile] = useState<LaunchProfile | null>(null);
  const [runtime, setRuntime] = useState<RuntimeCapabilities | null>(null);
  const [status, setStatus] = useState<ServerStatus>(idleStatus);
  const [command, setCommand] = useState("");
  // FE-03: per-resource request sequences and live identity mirrors, so a
  // deferred response can never commit against a newer resource state.
  const cloudSeq = useRef(0);
  const ggufSeq = useRef(0);
  const previewSeq = useRef(0);
  const [log, setLog] = useState("Waiting for a managed server.");
  const [notice, setNotice] = useState("Inventory not scanned yet.");
  const [busy, setBusy] = useState("");
  const [benchmark, setBenchmark] = useState<BenchmarkSummary | null>(null);
  const [tokens, setTokens] = useState(512);
  const [repeats, setRepeats] = useState(3);
  const [runtimeIdentity, setRuntimeIdentity] = useState<RuntimeIdentity | null>(null);
  const [managedRuntimes, setManagedRuntimes] = useState<ManagedRuntimeRecord[]>([]);
  const [providers, setProviders] = useState<CloudProvider[]>([]);
  const [providerId, setProviderId] = useState(CLOUD_PROVIDER);
  // Live mirrors for stale-response checks (audit FE-03): refs track the
  // committed values without re-rendering on assignment.
  // FE-16: single-flight status polling with a sequence so a delayed poll
  // can never overwrite a newer start/stop snapshot.
  const statusPollSeq = useRef(0);
  const providerIdRef = useRef(providerId);
  providerIdRef.current = providerId;
  const profileRef = useRef<LaunchProfile | null>(profile);
  profileRef.current = profile;
  const [credential, setCredential] = useState<CredentialStatus | null>(null);
  const [keyDraft, setKeyDraft] = useState("");
  const [cloudModels, setCloudModels] = useState<CloudModel[]>([]);
  const [cloudModel, setCloudModel] = useState(CLOUD_MODEL);
  const [cloudCheck, setCloudCheck] = useState("");
  const [gguf, setGguf] = useState<GgufSummary | null>(null);
  const [about, setAbout] = useState<AboutInfo | null>(null);
  const [catalogSnapshot, setCatalogSnapshot] = useState<CatalogSnapshot | null>(null);
  const [catalogRows, setCatalogRows] = useState<CatalogModel[]>([]);
  // One merged browse collection: verified curated rows plus user rows.
  // Every filter, sort, and facet input reads this, never the snapshot alone
  // (audit DC-04).
  const [catalogAllRows, setCatalogAllRows] = useState<CatalogModel[]>([]);
  const [catalogTags, setCatalogTags] = useState<string[]>([]);
  const [catalogQuants, setCatalogQuants] = useState<string[]>([]);
  const [catalogAuthors, setCatalogAuthors] = useState<string[]>([]);
  const [catalogLicenses, setCatalogLicenses] = useState<string[]>([]);
  const [catalogPipelines, setCatalogPipelines] = useState<string[]>([]);
  const [catalogArchitectures, setCatalogArchitectures] = useState<string[]>([]);
  const [catalogSearch, setCatalogSearch] = useState("");
  const [catalogTag, setCatalogTag] = useState("");
  const [catalogQuant, setCatalogQuant] = useState("");
  const [catalogAuthor, setCatalogAuthor] = useState("");
  const [catalogLicense, setCatalogLicense] = useState("");
  const [catalogPipeline, setCatalogPipeline] = useState("");
  const [catalogArchitecture, setCatalogArchitecture] = useState("");
  const [catalogMaxGiB, setCatalogMaxGiB] = useState(0);
  const [catalogHideGated, setCatalogHideGated] = useState(false);
  const [catalogSort, setCatalogSort] = useState<CatalogQuery["sort"]>("downloads");
  const [catalogFitEnabled, setCatalogFitEnabled] = useState(true);
  const [catalogFitPerMille, setCatalogFitPerMille] = useState(DEFAULT_FIT_PER_MILLE);
  const [catalogFitBudget, setCatalogFitBudget] = useState<FitBudget | null>(null);
  const [catalogBusy, setCatalogBusy] = useState(false);
  const [hfToken, setHfToken] = useState<TokenStatus>({ configured: false, masked: "" });
  const [hfTokenDraft, setHfTokenDraft] = useState("");
  const [catalogFiles, setCatalogFiles] = useState<Record<string, string>>({});
  const [downloads, setDownloads] = useState<Record<string, DownloadEvent>>({});
  const catalogFilterSeq = useRef(0);
  const catalogLoadSeq = useRef(0);
  const runtimeCatalogSeq = useRef(0);
  const runtimeInspectSeq = useRef(0);
  const installingRef = useRef("");
  const healthRunningRef = useRef("");
  const [tuneContext, setTuneContext] = useState(8192);
  const [tuneTrials, setTuneTrials] = useState(6);
  const [tuneTokens, setTuneTokens] = useState(256);
  const [tuneRepeats, setTuneRepeats] = useState(2);
  const [tuning, setTuning] = useState(false);
  const [tuneProgress, setTuneProgress] = useState<TuningProgress | null>(null);
  const [tuneLive, setTuneLive] = useState<TuningTrial[]>([]);
  const [tuneReport, setTuneReport] = useState<TuningReport | null>(null);
  // FE-02: immutable identity of the run in flight and of the displayed
  // report, so navigation or draft edits can never relabel work.
  const [tuneRun, setTuneRun] = useState<{
    modelId: string;
    modelName: string;
    provider: string;
    advisor: string;
    context: number;
  } | null>(null);
  const [tuneReportOrigin, setTuneReportOrigin] = useState<{
    modelId: string;
    modelName: string;
    provider: string;
    advisor: string;
    context: number;
  } | null>(null);
  const tuneLogRef = useRef<HTMLDivElement | null>(null);

  const selected = displayedSelection(models, selectedId);
  const totalBytes = useMemo(() => models.reduce((sum, model) => sum + model.sizeBytes, 0), [models]);
  const invalidCount = models.filter((model) => !model.complete).length;
  const provider = providers.find((entry) => entry.id === providerId) ?? null;
  const isManagedPath = runtimeIdentity?.managedVerified === true;
  const healthOutcome = healthResult ? managedHealthOutcome(healthResult) : null;

  async function scan() {
    if (!modelRoot.trim()) {
      setNotice("Choose the folder that contains your GGUF models.");
      return;
    }
    setBusy("scan");
    try {
      const result = await invoke<LogicalModel[]>("scan_models", { root: modelRoot });
      // FE-01: a rescan preserves the committed selection while the model
      // still exists; otherwise the replacement is loaded together with its
      // profile. An empty inventory clears both together.
      const nextId = selectionAfterRescan(result, selectedId);
      setModels(result);
      if (nextId === null) {
        clearCommittedSelection();
      } else if (nextId !== selectedId || selectedId === "") {
        const replacement = result.find((model) => model.id === nextId);
        if (replacement) loadProfile(replacement);
      }
      localStorage.setItem("localmotive:model-root", modelRoot);
      setNotice(`${result.length} logical targets indexed from ${modelRoot}`);
    } catch (error) {
      // A failed scan clears the selection and its editable profile as one
      // transition, so a stale draft can never masquerade as current.
      setModels([]);
      clearCommittedSelection();
      setNotice(inTauri() ? String(error) : "Browser preview cannot scan local model files. Use the packaged app.");
    } finally {
      setBusy("");
    }
  }

  /// FE-01: clearing the committed selection also clears everything derived
  /// from it (profile draft, GGUF metadata, pending previews).
  function clearCommittedSelection() {
    previewSeq.current += 1;
    ggufSeq.current += 1;
    setSelectedId("");
    setProfile(null);
    setGguf(null);
  }

  async function inspect(pathOverride?: string) {
    const path = (pathOverride ?? runtimePath).trim();
    if (!path) {
      setNotice("Install a managed runtime or choose an existing llama-server.exe.");
      return;
    }
    const sequence = ++runtimeInspectSeq.current;
    setBusy("runtime");
    try {
      const [caps, identity, managed] = await Promise.all([
        invoke<RuntimeCapabilities>("inspect_runtime", { path }),
        invoke<RuntimeIdentity>("describe_runtime", { path }),
        invoke<ManagedRuntimeRecord[]>("list_managed_runtimes"),
      ]);
      if (!keepLatestRequest(sequence, runtimeInspectSeq.current)) return;
      commitRuntime(path, caps, identity);
      setManagedRuntimes(managed);
      setNotice(`Runtime build ${caps.build} inspected; ${caps.specTypes.length} speculation modes exposed.`);
    } catch (error) {
      if (!keepLatestRequest(sequence, runtimeInspectSeq.current)) return;
      setRuntime(null);
      setRuntimeIdentity(null);
      setNotice(inTauri() ? String(error) : "Browser preview cannot inspect local runtime files. Use the packaged app.");
    } finally {
      if (keepLatestRequest(sequence, runtimeInspectSeq.current)) setBusy("");
    }
  }

  /** Point the app at a different executable and inspect it in one step. */
  /// FE-01: one committed-runtime transition. The editable profile follows
  /// the runtime only after its inspection succeeded; a failed inspection
  /// leaves the previous committed identity and the draft untouched, and the
  /// typed path stays uncommitted input.
  function commitRuntime(path: string, caps: RuntimeCapabilities, identity: RuntimeIdentity) {
    setRuntimePath(path);
    setRuntime(caps);
    setRuntimeIdentity(identity);
    localStorage.setItem("localmotive:runtime", path);
    setProfile((current) => (current ? { ...current, runtime: path } : current));
  }

  async function activateRuntime(path: string) {
    await inspect(path);
  }

  async function loadRuntimeSetup() {
    const sequence = ++runtimeCatalogSeq.current;
    setRuntimeCatalogLoading(true);
    setRuntimeCatalogError(null);
    try {
      const setup = await invoke<RuntimeSetupResponse>("load_runtime_setup", {
        adapterId: selectedRuntimeAdapterId || null,
      });
      if (!keepLatestRequest(sequence, runtimeCatalogSeq.current)) return;
      setHardware(setup.hardware);
      setSelectedRuntimeAdapterId((current) => {
        if (setup.hardware.adapters.some((adapter) => adapter.adapterId === current)) return current;
        return setup.hardware.adapters.length === 1 ? setup.hardware.adapters[0].adapterId : "";
      });
      setRuntimeRoot(setup.runtimeRoot);
      setRuntimeCatalog(setup.catalog);
      setRuntimeCatalogError(setup.catalogError);
      setManagedRuntimes(setup.managedRuntimes);
      if (!runtimePath) setNotice("Hardware detected. Install the recommended llama.cpp runtime to continue.");
    } catch (error) {
      if (!keepLatestRequest(sequence, runtimeCatalogSeq.current)) return;
      if (!inTauri()) {
        setHardware({
          architecture: "unknown",
          gpuNames: [],
          vendor: "unknown",
          cudaMajor: null,
          driverVersion: "unknown",
          detectionStatus: "Browser preview has no native Windows hardware evidence",
          recommendation: "CPU fallback until exact L4 product compatibility evidence is available",
          systemMemory: {
            totalPhysicalBytes: {
              value: null,
              level: "unknown",
              source: { kind: "unknown", detail: "browser preview" },
              observedAtMs: 0,
              notes: ["Native Windows evidence is unavailable in browser preview."],
            },
            availablePhysicalBytes: {
              value: null,
              level: "unknown",
              source: { kind: "unknown", detail: "browser preview" },
              observedAtMs: 0,
              notes: ["Native Windows evidence is unavailable in browser preview."],
            },
            memoryLoadPercent: {
              value: null,
              level: "unknown",
              source: { kind: "unknown", detail: "browser preview" },
              observedAtMs: 0,
              notes: ["Native Windows evidence is unavailable in browser preview."],
            },
          },
          adapters: [],
          manualOverrides: [],
        });
        setRuntimeCatalog(null);
        setRuntimeCatalogError({
          kind: "invalid_response",
          message: "Browser preview cannot retrieve the approved runtime catalog.",
          retryAfterSeconds: null,
        });
        setRuntimeRoot("");
        setNotice("Browser preview cannot retrieve the approved runtime catalog. Use the packaged app for native evidence.");
      } else {
        const catalogError = runtimeCatalogErrorFromUnknown(error);
        setRuntimeCatalogError(catalogError);
        setNotice(catalogError.message);
      }
    } finally {
      if (keepLatestRequest(sequence, runtimeCatalogSeq.current)) {
        setRuntimeCatalogLoading(false);
      }
    }
  }

  async function selectRuntimeAdapter(adapterId: string) {
    setSelectedRuntimeAdapterId(adapterId);
    if (!hardware || !inTauri()) return;
    const sequence = ++runtimeCatalogSeq.current;
    setRuntimeCatalogLoading(true);
    setRuntimeCatalogError(null);
    try {
      const catalog = await invoke<RuntimeCatalog>("fetch_runtime_catalog", {
        adapterId: adapterId || null,
      });
      if (keepLatestRequest(sequence, runtimeCatalogSeq.current)) setRuntimeCatalog(catalog);
    } catch (error) {
      if (keepLatestRequest(sequence, runtimeCatalogSeq.current)) {
        setRuntimeCatalogError(runtimeCatalogErrorFromUnknown(error));
      }
    } finally {
      if (keepLatestRequest(sequence, runtimeCatalogSeq.current)) {
        setRuntimeCatalogLoading(false);
      }
    }
  }

  const runtimeCatalogState = runtimeCatalogViewState({
    loading: runtimeCatalogLoading,
    catalog: runtimeCatalog,
    error: runtimeCatalogError,
  });

  async function installRuntime(option: RuntimeOption) {
    if (!runtimeCatalog) return;
    installingRef.current = option.installKey;
    setInstalling(option.id);
    setRuntimeInstallProgress(null);
    setNotice(`Downloading ${option.label} from the official llama.cpp release…`);
    try {
      const installed = await invoke<InstalledRuntime>("install_managed_runtime", {
        request: runtimeInstallRequest(option, selectedRuntimeAdapterId),
      });
      await activateRuntime(installed.runtimePath);
      setNotice(`${option.label} ${installed.reused ? "was already installed and is now active" : "installed, verified, and active"}.`);
    } catch (error) {
      setNotice(String(error));
    } finally {
      installingRef.current = "";
      setRuntimeInstallCancelling(false);
      setInstalling("");
    }
  }

  async function cancelRuntimeInstall() {
    if (runtimeInstallCancelling) return;
    setRuntimeInstallCancelling(true);
    const cancelled = await invoke<boolean>("cancel_managed_runtime_install").catch(() => false);
    if (cancelled) {
      setNotice("Stopping the runtime installation. Verified partial download state will remain available for resume.");
    } else {
      setRuntimeInstallCancelling(false);
    }
  }

  async function runManagedHealth(option: RuntimeOption) {
    healthRunningRef.current = option.installKey;
    setHealthRunning(option.installKey);
    setHealthModelProgress(null);
    setHealthResult(null);
    setNotice(`Running the seven-stage health contract for ${option.label}…`);
    try {
      const result = await invoke<ManagedHealthResult>("check_managed_runtime_health", {
        request: managedHealthRequest(option, selectedRuntimeAdapterId),
      });
      setHealthResult(result);
      const outcome = managedHealthOutcome(result);
      setNotice(outcome === "passed"
        ? `${option.label} passed all seven managed-runtime health stages.`
        : outcome === "cancelled"
          ? `${option.label} health verification was cancelled and cleaned up.`
          : `${option.label} failed managed-runtime health verification.`);
    } catch (error) {
      setNotice(String(error));
    } finally {
      healthRunningRef.current = "";
      setHealthCancelling(false);
      setHealthRunning("");
    }
  }

  async function cancelManagedHealth() {
    if (healthCancelling) return;
    setHealthCancelling(true);
    const cancelled = await invoke<boolean>("cancel_managed_runtime_health").catch(() => false);
    if (cancelled) {
      setNotice("Stopping the managed-runtime health run and cleaning up its process.");
    } else {
      setHealthCancelling(false);
    }
  }

  // Repair the cached pinned health model (audit S-04): a corrupt or
  // truncated cache is quarantined with its diagnostics preserved, and the
  // immutable pinned revision is downloaded again through the size/hash
  // checks. A healthy cache returns immediately without touching the network.
  async function repairHealthModel() {
    if (healthRepairing) return;
    setHealthRepairNotice(null);
    setHealthRepairing(true);
    try {
      await invoke("repair_health_model");
      setHealthRepairNotice("Cached health model is verified.");
    } catch (error) {
      setHealthRepairNotice(String(error));
    } finally {
      setHealthRepairing(false);
    }
  }

  async function cancelHealthRepair() {
    try {
      await invoke("cancel_health_model_repair");
    } catch (error) {
      setHealthRepairNotice(String(error));
    }
  }

  async function chooseModelFolder() {
    const selected = await openDialog({ directory: true, multiple: false, title: "Choose your GGUF model folder" });
    if (typeof selected === "string") {
      setModelRoot(selected);
      localStorage.setItem("localmotive:model-root", selected);
    }
  }

  async function loadModelCatalog() {
    const sequence = ++catalogLoadSeq.current;
    setCatalogBusy(true);
    try {
      // Local load first: the signature-verified cache or the bundled
      // snapshot, without a network request and without the refresh cooldown.
      // Rows must appear when the app restarts inside the cooldown or offline
      // (audit DC-01). Only an explicit refresh can be throttled.
      const local = await invoke<CatalogSnapshot>("load_model_catalog");
      if (!keepLatestRequest(sequence, catalogLoadSeq.current)) return;
      setCatalogSnapshot(local);

      // Then attempt the network refresh. When it is throttled or fails, the
      // loaded rows stay and the notice explains the refresh state.
      let snapshot = local;
      let refreshNote = "";
      try {
        snapshot = await invoke<CatalogSnapshot>("fetch_model_catalog");
        if (!keepLatestRequest(sequence, catalogLoadSeq.current)) return;
        setCatalogSnapshot(snapshot);
      } catch (error) {
        refreshNote = ` Refresh pending: ${String(error)}`;
      }
      // Local mirror: verified rows plus marked user rows. Falls back to the
      // snapshot when the mirror is unavailable, so the tab never goes empty
      // because of a local database problem.
      const localModels = await invoke<CatalogModel[]>("catalog_local_models").catch(
        () => snapshot.catalog.models,
      );
      const [tags, quants] = await invoke<[string[], string[]]>("catalog_facets", {
        models: localModels,
      });
      const rich = await invoke<CatalogFacets>("catalog_rich_facets", {
        models: localModels,
      }).catch(() => null);
      const token = await invoke<TokenStatus>("hf_token_status");
      if (!keepLatestRequest(sequence, catalogLoadSeq.current)) return;
      setCatalogSnapshot(snapshot);
      setCatalogRows(localModels);
      setCatalogAllRows(localModels);
      setCatalogTags(tags);
      setCatalogQuants(quants);
      if (rich) {
        // Defensive defaults: a partial facet payload must degrade to empty
        // filters, never crash the render on `.map` (GH-05).
        setCatalogAuthors(rich.authors ?? []);
        setCatalogLicenses(rich.licenses ?? []);
        setCatalogPipelines(rich.pipelineTags ?? []);
        setCatalogArchitectures(rich.architectures ?? []);
      }
      setHfToken(token);
      const userCount = localModels.filter((model) => model.userSourced).length;
      const userNote = userCount > 0 ? ` (includes ${userCount} USER ADDED local row${userCount === 1 ? "" : "s"})` : "";
      const droppedRows = snapshot.catalog.dropped ?? [];
      const droppedNote =
        droppedRows.length > 0
          ? ` ${droppedRows.length} catalog row${droppedRows.length === 1 ? "" : "s"} dropped by validation: ${droppedRows
              .slice(0, 2)
              .map((drop) => drop.reason)
              .join("; ")}.`
          : "";
      const warningNote = [snapshot.refreshError, snapshot.persistenceNotice]
        .filter(Boolean)
        .join(" ");
      setNotice(
        `${localModels.length} curated Hugging Face models loaded from ${snapshot.origin}${userNote}.${refreshNote}${warningNote ? ` ${warningNote}` : ""}${droppedNote}`,
      );
    } catch (error) {
      if (keepLatestRequest(sequence, catalogLoadSeq.current)) setNotice(String(error));
    } finally {
      if (keepLatestRequest(sequence, catalogLoadSeq.current)) setCatalogBusy(false);
    }
  }

  async function saveHfToken() {
    if (!hfTokenDraft.trim()) return;
    setCatalogBusy(true);
    try {
      const status = await invoke<TokenStatus>("save_hf_token", { token: hfTokenDraft });
      setHfTokenDraft("");
      setHfToken(status);
      setNotice(`Hugging Face token ${status.masked} stored in Windows Credential Manager.`);
    } catch (error) {
      setNotice(String(error));
    } finally {
      setCatalogBusy(false);
    }
  }

  async function removeHfToken() {
    setCatalogBusy(true);
    try {
      setHfToken(await invoke<TokenStatus>("clear_hf_token"));
      setHfTokenDraft("");
      setNotice("Hugging Face token removed from Windows Credential Manager.");
    } catch (error) {
      setNotice(String(error));
    } finally {
      setCatalogBusy(false);
    }
  }

  function inventoryHasFile(filename: string): boolean {
    const wanted = filename.replace(/\//g, "\\").toLocaleLowerCase();
    return models.some((model) =>
      model.firstShard.toLocaleLowerCase().endsWith(wanted)
      || model.companions.some((entry) => entry.path.toLocaleLowerCase().endsWith(wanted)),
    );
  }

  async function startCatalogDownload(model: CatalogModel, file: CatalogFile) {
    // Capture the job identity now: a later destination edit must not
    // retarget this download's progress or cancellation (audit FE-11).
    const revision = catalogRevision(file);
    const destination = modelRoot;
    const key = downloadKey(model.repo, file.filename, destination, revision);
    setDownloadJobs((current) => [
      ...current.filter((job) => job.key !== key),
      { key, repo: model.repo, filename: file.filename, revision, destination, startedAt: Date.now() },
    ]);
    setDownloads((current) => ({
      ...current,
      [key]: {
        key,
        downloaded: 0,
        total: file.sizeBytes,
        bytesPerSecond: 0,
        state: "downloading",
        message: "Connecting to Hugging Face…",
        path: "",
      },
    }));
    try {
      const path = await invoke<string>("download_catalog_file", {
        repo: model.repo,
        filename: file.filename,
        revision,
        destination,
        connections: 4,
      });
      setNotice(`Downloaded and verified ${file.filename} to ${path}`);
    } catch (error) {
      setDownloads((current) => ({
        ...current,
        [key]: {
          ...(current[key] ?? {
            key,
            downloaded: 0,
            total: file.sizeBytes,
            bytesPerSecond: 0,
            path: "",
          }),
          state: "error",
          message: String(error),
        },
      }));
      setNotice(String(error));
    }
  }

  async function cancelCatalogDownload(job: DownloadJob) {
    try {
      const stopped = await invoke<boolean>("cancel_download", {
        destination: job.destination,
        filename: job.filename,
      });
      setNotice(
        stopped
          ? `Stopping ${job.filename}; downloaded chunks will be kept for resume.`
          : `${job.filename} is not being downloaded (it may have finished); nothing was stopped.`,
      );
    } catch (error) {
      setNotice(`Could not stop ${job.filename}: ${String(error)}`);
    }
  }

  async function chooseExistingRuntime() {
    const selected = await openDialog({
      directory: false,
      multiple: false,
      title: "Choose llama-server.exe",
      filters: [{ name: "llama-server", extensions: ["exe"] }],
    });
    if (typeof selected === "string") await activateRuntime(selected);
  }

  // ---- Cloud provider & credentials ------------------------------------------

  async function loadCloud(nextProvider = providerId) {
    const sequence = ++cloudSeq.current;
    try {
      const list = providers.length ? providers : await invoke<CloudProvider[]>("cloud_providers");
      if (sequence !== cloudSeq.current) return;
      if (!providers.length) setProviders(list);
      const status = await invoke<CredentialStatus>("cloud_credential_status", { provider: nextProvider });
      if (!responseIsCurrent(sequence, cloudSeq.current, nextProvider, providerIdRef.current)) return;
      setCredential(status);
      setCloudModels([]);
      setCloudCheck("");
      setCloudModel("");
      if (status.configured) {
        try {
          const modelsList = await invoke<CloudModel[]>("cloud_list_models", { provider: nextProvider });
          if (!responseIsCurrent(sequence, cloudSeq.current, nextProvider, providerIdRef.current)) return;
          setCloudModels(modelsList);
          const fallback = list.find((entry) => entry.id === nextProvider)?.defaultModel ?? "";
          const stored = readRecord(`cloud-model:${nextProvider}`);
          const chosen = stored && modelsList.some((m) => m.id === stored) ? stored : modelsList.some((m) => m.id === fallback) ? fallback : (modelsList[0]?.id ?? fallback);
          setCloudModel(chosen);
        } catch (error) {
          if (responseIsCurrent(sequence, cloudSeq.current, nextProvider, providerIdRef.current)) setCloudCheck(String(error));
        }
      }
    } catch (error) {
      if (sequence !== cloudSeq.current) return;
      if (!inTauri()) {
        setProviders([
          { id: "openrouter", label: "OpenRouter", baseUrl: "https://openrouter.ai/api/v1", keyPrefixHint: "sk-or-", consoleUrl: "https://openrouter.ai/settings/keys", supportsOauth: true, defaultModel: "anthropic/claude-sonnet-4.6", listsModels: true },
          { id: "anthropic", label: "Anthropic", baseUrl: "https://api.anthropic.com/v1", keyPrefixHint: "sk-ant-", consoleUrl: "https://platform.claude.com/settings/keys", supportsOauth: false, defaultModel: "claude-sonnet-4-6", listsModels: true },
          { id: "openai", label: "OpenAI", baseUrl: "https://api.openai.com/v1", keyPrefixHint: "sk-", consoleUrl: "https://platform.openai.com/api-keys", supportsOauth: false, defaultModel: "gpt-5", listsModels: true },
          { id: "gemini", label: "Google Gemini", baseUrl: "https://generativelanguage.googleapis.com/v1beta/openai", keyPrefixHint: "AIza", consoleUrl: "https://aistudio.google.com/apikey", supportsOauth: false, defaultModel: "gemini-2.5-pro", listsModels: true },
        ]);
        setCredential({ provider: nextProvider, configured: false, masked: "" });
      } else {
        setNotice(String(error));
      }
    }
  }

  async function switchProvider(next: string) {
    setProviderId(next);
    localStorage.setItem("localmotive:cloud-provider", next);
    setKeyDraft("");
    // FE-03: clear provider-specific presentation at switch start; a slow
    // previous provider can never relabel the new tab while it loads.
    cloudSeq.current += 1;
    setCredential(null);
    setCloudModels([]);
    setCloudCheck("");
    await loadCloud(next);
  }

  /// WAI-ARIA tabs keyboard pattern (audit FE-13): arrows cycle, Home/End
  /// jump, and focus follows the selection.
  function onProviderTabKey(
    event: React.KeyboardEvent<HTMLButtonElement>,
    index: number,
  ) {
    let next = index;
    if (event.key === "ArrowRight") next = (index + 1) % providers.length;
    else if (event.key === "ArrowLeft") next = (index - 1 + providers.length) % providers.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = providers.length - 1;
    else return;
    event.preventDefault();
    const target = providers[next];
    if (!target) return;
    switchProvider(target.id);
    document.getElementById(`provider-tab-${target.id}`)?.focus();
  }

  async function saveKey() {
    if (!keyDraft.trim()) return;
    const forProvider = providerId;
    const sequence = ++cloudSeq.current;
    setBusy("cloud");
    try {
      const status = await invoke<CredentialStatus>("cloud_save_credential", { provider: forProvider, secret: keyDraft });
      if (!responseIsCurrent(sequence, cloudSeq.current, forProvider, providerIdRef.current)) return;
      setKeyDraft("");
      setCredential(status);
      setNotice(`${providers.find((entry) => entry.id === forProvider)?.label ?? forProvider} key stored in Windows Credential Manager.`);
      await loadCloud(forProvider);
    } catch (error) {
      if (responseIsCurrent(sequence, cloudSeq.current, forProvider, providerIdRef.current)) setNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function forgetKey() {
    const forProvider = providerId;
    const sequence = ++cloudSeq.current;
    setBusy("cloud");
    try {
      const status = await invoke<CredentialStatus>("cloud_clear_credential", { provider: forProvider });
      if (!responseIsCurrent(sequence, cloudSeq.current, forProvider, providerIdRef.current)) return;
      setCredential(status);
      setCloudModels([]);
      setCloudCheck("");
      setNotice(`${providers.find((entry) => entry.id === forProvider)?.label ?? forProvider} key removed from Windows Credential Manager.`);
    } catch (error) {
      if (responseIsCurrent(sequence, cloudSeq.current, forProvider, providerIdRef.current)) setNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function openRouterLogin() {
    const sequence = ++cloudSeq.current;
    setBusy("oauth");
    setNotice("Finish signing in to OpenRouter in your browser. Localmotive is waiting on a local callback.");
    try {
      const status = await invoke<CredentialStatus>("cloud_openrouter_login");
      if (!responseIsCurrent(sequence, cloudSeq.current, "openrouter", providerIdRef.current)) return;
      setCredential(status);
      setNotice("OpenRouter connected. A user-controlled key was issued and stored in Windows Credential Manager.");
      await loadCloud("openrouter");
    } catch (error) {
      // A cancelled/timed-out sign-in must not relabel another provider's tab.
      if (responseIsCurrent(sequence, cloudSeq.current, "openrouter", providerIdRef.current)) setNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function probeCloud() {
    const forProvider = providerId;
    const forModel = cloudModel;
    const sequence = ++cloudSeq.current;
    setBusy("probe");
    setCloudCheck("Contacting provider…");
    try {
      const reply = await invoke<string>("cloud_probe", { provider: forProvider, model: forModel });
      if (!responseIsCurrent(sequence, cloudSeq.current, forProvider, providerIdRef.current)) return;
      setCloudCheck(`Connected · ${forModel} replied “${reply.trim().slice(0, 40)}”`);
    } catch (error) {
      if (responseIsCurrent(sequence, cloudSeq.current, forProvider, providerIdRef.current)) setCloudCheck(String(error));
    } finally {
      setBusy("");
    }
  }

  function chooseCloudModel(id: string) {
    setCloudModel(id);
    localStorage.setItem("localmotive:cloud-model", id);
    localStorage.setItem(`localmotive:cloud-model:${providerId}`, id);
  }

  // ---- AI tuning -------------------------------------------------------------

  async function loadGguf(model: LogicalModel | undefined) {
    const sequence = ++ggufSeq.current;
    if (!model) {
      // FE-03: an absent selection clears metadata instead of retaining
      // another model's facts.
      setGguf(null);
      return;
    }
    try {
      const summary = await invoke<GgufSummary>("read_gguf_summary", { path: model.firstShard });
      if (!keepLatestRequest(sequence, ggufSeq.current)) return;
      setGguf(summary);
      if (summary.contextLength && tuneContext > summary.contextLength) {
        const choices = contextChoices(summary.contextLength);
        setTuneContext(choices[choices.length - 1] ?? 2048);
      }
    } catch {
      if (keepLatestRequest(sequence, ggufSeq.current)) setGguf(null);
    }
  }

  async function startTuning() {
    if (!profile || !selected) return;
    // FE-02: capture the originating identity at dispatch; nothing later can
    // change whose run this is.
    const run = {
      modelId: selected.id,
      modelName: selected.name,
      provider: providerId,
      advisor: cloudModel,
      context: tuneContext,
    };
    setTuning(true);
    setTuneRun(run);
    setTuneReport(null);
    setTuneReportOrigin(null);
    setTuneLive([]);
    setTuneProgress({ phase: "prepare", message: "Preparing…", trial: null });
    setNotice(`AI tuning started: ${tuneTrials} trials at ${run.context.toLocaleString()} context via ${providers.find((entry) => entry.id === run.provider)?.label ?? run.provider} (${run.advisor}).`);
    try {
      const report = await invoke<TuningReport>("start_tuning", {
        request: {
          profile,
          provider: run.provider,
          model: run.advisor,
          targetContext: run.context,
          maxTrials: tuneTrials,
          tokens: tuneTokens,
          repeats: tuneRepeats,
          companions: selected.companions.map((c) => `${c.role}: ${c.path}`),
        },
      });
      setTuneReport(report);
      setTuneReportOrigin(run);
      localStorage.setItem(`localmotive:tuning:${run.modelId}`, JSON.stringify(report));
      const gain = report.baselineTps && report.bestTps ? ((report.bestTps / report.baselineTps - 1) * 100).toFixed(1) : null;
      setNotice(gain ? `Tuning finished: best ${report.bestTps?.toFixed(2)} tok/s (${Number(gain) >= 0 ? "+" : ""}${gain}% vs baseline). ${report.stoppedReason}.` : `Tuning finished. ${report.stoppedReason}.`);
    } catch (error) {
      setNotice(String(error));
      setTuneProgress({ phase: "error", message: String(error), trial: null });
    } finally {
      setTuning(false);
      setTuneRun(null);
    }
  }

  async function cancelTuning() {
    try {
      await invoke<boolean>("cancel_tuning");
      setNotice("Cancelling after the current measurement…");
    } catch (error) {
      setNotice(String(error));
    }
  }

  function adoptTunedProfile() {
    // FE-02: adoption binds to the report's originating model, never to the
    // currently selected one; the runtime in the adopted profile follows the
    // established normalization policy implicitly (it is the measured
    // runtime, applied through the same editable draft the loader uses).
    if (!tuneReport || !tuneReportOrigin) return;
    const origin = tuneReportOrigin;
    const adopted = { ...tuneReport.bestProfile, name: `${origin.modelName} / AI-tuned @${origin.context.toLocaleString()}` };
    localStorage.setItem(`localmotive:profile:${origin.modelId}`, JSON.stringify(adopted));
    if (selectedId === origin.modelId) {
      setProfile(adopted);
      setNotice(`Adopted the best configuration as the saved profile for ${origin.modelName}.`);
      setView("profile");
    } else {
      setNotice(`Saved the tuned profile for ${origin.modelName}; it applies when that model is selected.`);
    }
  }

  function loadProfile(model: LogicalModel) {
    // A new selection invalidates any pending preview for the old profile.
    previewSeq.current += 1;
    const stored = readRecord(`profile:${model.id}`);
    const parsedStored = stored === null ? undefined : safeJsonParse<Partial<LaunchProfile>>(stored);
    if (stored !== null && parsedStored === undefined) {
      quarantineRecord(`profile:${model.id}`, stored);
      setNotice(
        `The saved profile for ${model.name} was unreadable and was moved to quarantine; recommended defaults were loaded.`,
      );
    }
    const next = parsedStored !== undefined
      ? normalizeProfile(parsedStored, model, runtimePath)
      : suggestedProfile(model, runtimePath);
    setProfile(next);
    setSelectedId(model.id);
    setView("profile");
    // Only move an unsaved candidate off a currently busy port. Launch remains authoritative.
    if (!stored) void suggestPortCandidate(next);
  }

  /** Suggest a currently unused port without claiming that the released socket stays available. */
  async function suggestPortCandidate(candidate: LaunchProfile) {
    const identity = profileIdentity(candidate);
    try {
      const free = await invoke<number>("suggest_port", { host: candidate.host, preferred: candidate.port });
      if (free === candidate.port) return;
      // FE-03: apply only while the profile is still the exact candidate and
      // its port is unchanged; a later manual edit or a newly selected
      // profile is never overwritten.
      const current = profileRef.current;
      if (!current) return;
      const nextProfile = applySuggestedPort(current, identity, candidate.port, free);
      if (nextProfile === current) return;
      setProfile(nextProfile);
      setNotice(`Port ${candidate.port} is in use. Trying ${free}; launch validation will confirm it.`);
    } catch {
      // Browser preview, or no free port in range: keep the requested one and
      // let child startup and health validation report the authoritative result.
    }
  }

  function saveProfile() {
    if (!profile || !selected) return;
    localStorage.setItem(`localmotive:profile:${selected.id}`, JSON.stringify(profile));
    setNotice(`Saved ${profile.name}`);
  }

  async function preview() {
    if (!profile) return;
    const identity = profileIdentity(profile);
    const sequence = ++previewSeq.current;
    try {
      const command = await invoke<string>("preview_command", { profile });
      // FE-03: an old preview must not overwrite a newer one; an old profile
      // must not label the current one's command.
      const currentIdentity = profileRef.current ? profileIdentity(profileRef.current) : "";
      if (!responseIsCurrent(sequence, previewSeq.current, identity, currentIdentity)) return;
      setCommand(command);
    } catch (error) {
      if (keepLatestRequest(sequence, previewSeq.current)) {
        setCommand(inTauri() ? String(error) : "Browser preview cannot compose the command. Use the packaged app.");
      }
    }
  }

  async function start() {
    if (!profile) return;
    statusPollSeq.current += 1;
    setBusy("start");
    try {
      const next = await invoke<ServerStatus>("start_server", { profile });
      setStatus(next);
      setNotice(`Started ${profile.alias} on port ${profile.port}`);
      setView("dashboard");
    } catch (error) {
      setNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function stop() {
    statusPollSeq.current += 1;
    setBusy("stop");
    try {
      setStatus(await invoke<ServerStatus>("stop_server"));
      setNotice("Managed server stopped cleanly.");
    } catch (error) {
      setNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function runBenchmark() {
    if (!status.port) return;
    setBusy("benchmark");
    try {
      const result = await invoke<BenchmarkSummary>("benchmark_server", {
        host: profile?.host ?? "127.0.0.1",
        port: status.port,
        tokens,
        repeats,
      });
      setBenchmark(result);
      localStorage.setItem(`localmotive:benchmark:${status.alias}`, JSON.stringify(result));
      setNotice(`Benchmark complete: ${result.meanTps.toFixed(2)} generation tok/s mean.`);
    } catch (error) {
      setNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  useEffect(() => {
    if (view === "catalog" && !catalogSnapshot && !catalogBusy) loadModelCatalog();
  }, [view]);

  useEffect(() => {
    if (!catalogSnapshot) return;
    // Auto-fit budget follows detected hardware: dedicated VRAM wins, else
    // shared, else system memory. Never sums budgets. Disabled by toggle.
    const dedicated = (hardware?.adapters ?? []).map((adapter) => adapter.dedicatedBytes.value);
    const shared = (hardware?.adapters ?? []).map((adapter) => adapter.sharedBytes.value);
    const system = hardware?.systemMemory.availablePhysicalBytes.value
      ?? hardware?.systemMemory.totalPhysicalBytes.value ?? null;
    setCatalogFitBudget(hardwareFitBudget(dedicated, shared, system ?? 0));
  }, [hardware, catalogSnapshot]);

  useEffect(() => {
    if (!catalogSnapshot) return;
    const sequence = ++catalogFilterSeq.current;
    const query: CatalogQuery = {
      text: catalogSearch,
      tag: catalogTag,
      quant: catalogQuant,
      author: catalogAuthor,
      license: catalogLicense,
      pipeline_tag: catalogPipeline,
      architecture: catalogArchitecture,
      maxBytes: catalogMaxGiB > 0 ? catalogMaxGiB * 1024 ** 3 : 0,
      hideGated: catalogHideGated,
      sort: catalogSort,
      fit_per_mille: catalogFitEnabled ? catalogFitPerMille : 0,
      budget_bytes: catalogFitEnabled ? (catalogFitBudget?.budgetBytes ?? 0) : 0,
    };
    invoke<CatalogModel[]>("filter_catalog", {
      models: catalogAllRows,
      query,
    })
      .then((rows) => {
        if (keepLatestRequest(sequence, catalogFilterSeq.current)) setCatalogRows(rows);
      })
      .catch((error) => {
        if (keepLatestRequest(sequence, catalogFilterSeq.current)) setNotice(String(error));
      });
  }, [catalogSnapshot, catalogAllRows, catalogSearch, catalogTag, catalogQuant, catalogAuthor, catalogLicense, catalogPipeline, catalogArchitecture, catalogMaxGiB, catalogHideGated, catalogSort, catalogFitEnabled, catalogFitPerMille, catalogFitBudget]);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let disposed = false;
    listen<DownloadEvent>("download:progress", (event) => {
      setDownloads((current) => ({ ...current, [event.payload.key]: event.payload }));
    })
      .then((stop) => { unlisten = retainOrDisposeListener(disposed, stop); })
      .catch(() => { /* Browser preview has no Tauri bridge. */ });
    return () => { disposed = true; unlisten?.(); };
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let disposed = false;
    listen<RuntimeInstallProgress>("runtime-install-progress", (event) => {
      if (event.payload.installKey === installingRef.current) {
        setRuntimeInstallProgress(event.payload);
      }
    })
      .then((stop) => { unlisten = retainOrDisposeListener(disposed, stop); })
      .catch(() => { /* Browser preview has no Tauri bridge. */ });
    return () => { disposed = true; unlisten?.(); };
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let disposed = false;
    listen<HealthModelProgress>("health-model-progress", (event) => {
      if (event.payload.installKey === healthRunningRef.current) {
        setHealthModelProgress(event.payload);
      }
    })
      .then((stop) => { unlisten = retainOrDisposeListener(disposed, stop); })
      .catch(() => { /* Browser preview has no Tauri bridge. */ });
    return () => { disposed = true; unlisten?.(); };
  }, []);

  useEffect(() => {
    loadRuntimeSetup();
    loadCloud();
    invoke<AboutInfo>("about_info").then(setAbout).catch(() => setAbout(null));
    if (modelRoot) scan();
    if (runtimePath) inspect();
    return () => {
      runtimeCatalogSeq.current += 1;
      runtimeInspectSeq.current += 1;
    };
  }, []);

  useEffect(() => {
    if (selected && !profile) setProfile(suggestedProfile(selected, runtimePath));
  }, [selected, runtimePath]);

  useEffect(() => {
    loadGguf(selected);
    if (selected) {
      const stored = readRecord(`tuning:${selected.id}`);
      const parsed = stored === null ? undefined : normalizeTuningReport(safeJsonParse(stored));
      if (stored !== null && parsed === undefined) {
        quarantineRecord(`tuning:${selected.id}`, stored);
        setNotice(
          `The saved tuning report for ${selected.name} did not match the expected shape and was moved to quarantine.`,
        );
      }
      setTuneReport(parsed ?? null);
      setTuneLive([]);
    }
  }, [selected?.id]);

  useEffect(() => {
    if (profile) preview();
  }, [profile]);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let disposed = false;
    listen<TuningProgress>("tuning-progress", (event) => {
      setTuneProgress(event.payload);
      if (event.payload.trial) setTuneLive((prev) => [...prev.filter((t) => t.index !== event.payload.trial!.index), event.payload.trial!]);
    })
      .then((stop) => { unlisten = retainOrDisposeListener(disposed, stop); })
      .catch(() => { /* Browser preview has no Tauri bridge. */ });
    return () => { disposed = true; unlisten?.(); };
  }, []);

  useEffect(() => {
    tuneLogRef.current?.scrollTo({ top: tuneLogRef.current.scrollHeight });
  }, [tuneLive.length, tuneProgress?.message]);

  useEffect(() => {
    if (!inTauri()) return;
    let disposed = false;
    let inFlight = false;
    const timer = window.setInterval(async () => {
      if (inFlight) return; // single-flight: slow native calls cannot pile up
      inFlight = true;
      const requested = ++statusPollSeq.current;
      try {
        const next = await invoke<ServerStatus>("server_status");
        if (disposed || requested !== statusPollSeq.current) return; // obsolete
        setStatus(next);
        if (next.running) {
          const logText = await invoke<string>("read_server_log");
          if (!disposed && requested === statusPollSeq.current) setLog(logText);
        }
      } catch {
        // Browser preview has no Tauri bridge.
      } finally {
        inFlight = false;
      }
    }, 2000);
    return () => {
      disposed = true;
      window.clearInterval(timer);
    };
  }, []);

  const nav: Array<{ id: View; label: string; icon: typeof Gauge }> = [
    { id: "dashboard", label: "Control", icon: Gauge },
    { id: "models", label: "Inventory", icon: Database },
    { id: "catalog", label: "HF Catalog", icon: Download },
    { id: "runtime", label: "Runtime", icon: MonitorCog },
    { id: "profile", label: "Profile", icon: Settings2 },
    { id: "tune", label: "AI Tune", icon: Sparkles },
    { id: "benchmark", label: "Benchmark", icon: TestTube2 },
    { id: "about", label: "About", icon: Info },
  ];

  const trialsForDisplay: TuningTrial[] = tuneReport ? tuneReport.trials : tuneLive;
  const bestLive = trialsForDisplay.reduce<TuningTrial | null>((best, trial) => (trial.meanTps !== null && (best === null || (best.meanTps ?? 0) < trial.meanTps) ? trial : best), null);
  const canTune = Boolean(profile && selected?.complete && runtimePath && credential?.configured && cloudModel && !status.running && !tuning);
  const tuneBlocker = !credential?.configured
    ? `Connect ${provider?.label ?? "a cloud provider"} to begin: sign in or store an API key.`
    : !cloudModel
      ? "Choose an advisor model."
      : !selected
        ? "Select a model in Inventory."
        : !selected.complete
          ? `${selected.name} is incomplete; choose a model with every shard present.`
          : !runtimePath
            ? "Install or choose a llama-server runtime first."
            : status.running
              ? "Stop the running server first; the tuner launches its own."
              : `No tuning run for ${selected.name} yet. Choose a context length and start.`;

  return (
    <div className="app-shell">
      <aside className="rail">
        <div className="brand-mark" aria-label="Localmotive">
          <span>LM</span>
        </div>
        <nav aria-label="Primary">
          {nav.map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              className={view === id ? "nav-item active" : "nav-item"}
              onClick={() => setView(id)}
              title={label}
            >
              <Icon size={19} strokeWidth={1.8} />
              <span>{label}</span>
            </button>
          ))}
        </nav>
        <div className="rail-foot">
          <span className={status.running ? "signal live" : "signal"} />
          <small>{status.running ? "LIVE" : "IDLE"}</small>
        </div>
      </aside>

      <main className="workspace">
        {evidenceRun && (
          <div className="warning-band evidence-run-band" role="status">
            <Activity size={17} />
            <strong>{evidenceRunLabel(evidenceRun.kind)}</strong>
            <span>The run continues while you work on other screens.</span>
            {evidenceRun.cancel && (
              <button className="button secondary" onClick={evidenceRun.cancel}>Cancel</button>
            )}
            <button className="button secondary" onClick={() => setView("benchmark")}>Open Benchmark</button>
          </div>
        )}
        <header className="topbar">
          <div>
            <p className="product-name">LOCALMOTIVE</p>
            <p className="product-sub">Local inference control plane</p>
          </div>
          <div className="runtime-plate">
            <Cpu size={16} />
            <span>{runtime ? `llama.cpp b${runtime.build}` : "No runtime configured"}</span>
            <span className={runtime ? "plate-light ok" : "plate-light"} />
          </div>
        </header>

        <div className="notice-line" role="status">
          <span className="notice-code">SYS</span>
          <span>{notice}</span>
        </div>

        {view === "dashboard" && (
          <section className="screen dashboard-screen">
            <div className="section-heading">
              <div>
                <h1>Server control</h1>
                <p>One supervised process. Exact profile. No hidden defaults.</p>
              </div>
              <div className="actions">
                {status.running || status.phase === "starting" ? (
                  <>
                    {status.running && (
                      <button className="button secondary" onClick={() => openUrl(`http://127.0.0.1:${status.port}`)}>
                        <SquareTerminal size={16} /> Open chat
                      </button>
                    )}
                    <button className="button danger" onClick={stop} disabled={busy === "stop"}>
                      <CircleStop size={16} /> {status.phase === "starting" ? "Cancel start" : "Stop server"}
                    </button>
                  </>
                ) : (
                  <button className="button primary" onClick={start} disabled={!profile || !profile.runtime || busy === "start" || !selected?.complete || evidenceRun !== null || tuning}>
                    <Play size={16} fill="currentColor" /> Start profile
                  </button>
                )}
              </div>
            </div>

            <div className="instrument-strip">
              <div className={status.running ? "instrument primary-readout running" : "instrument primary-readout"}>
                <span className="instrument-label">PROCESS</span>
                <strong>{status.running ? "RUNNING" : status.phase === "starting" ? "STARTING" : "STANDBY"}</strong>
                <small>{status.running ? `PID ${status.pid}` : status.phase === "starting" ? "Waiting for readiness; Stop stays available" : "No owned child process"}</small>
              </div>
              <div className="instrument">
                <span className="instrument-label">ENDPOINT</span>
                <strong>{status.port ? `:${status.port}` : "—"}</strong>
                <small>{status.alias ?? "No alias assigned"}</small>
              </div>
              <div className="instrument">
                <span className="instrument-label">STRATEGY</span>
                {/* Running identity comes from the server snapshot; the
                    editable draft only describes a not-yet-started profile
                    (audit FE-16). */}
                <strong>{(status.running ? status.specType : profile?.specType)?.replace("draft-", "").toUpperCase() ?? "NONE"}</strong>
                <small>{status.running ? (status.companionLinked ? "Companion linked" : "Target only") : profile?.draftModel ? "Companion linked" : "Target only"}</small>
              </div>
              <div className="instrument">
                <span className="instrument-label">LAST TEST</span>
                <strong>{benchmark ? benchmark.meanTps.toFixed(1) : "—"}</strong>
                <small>{benchmark ? "generation tok/s mean" : "Not measured"}</small>
              </div>
            </div>

            <div className="control-grid">
              <article className="machine-panel current-profile">
                <div className="panel-title">
                  <Box size={17} />
                  <h2>Loaded profile</h2>
                  <span className={selected?.complete ? "state-tag good" : "state-tag warning"}>
                    {selected?.complete ? "SHARDS COMPLETE" : "SHARDS INCOMPLETE"}
                  </span>
                </div>
                <div className="profile-identity">
                  <strong>{profile?.name ?? "No profile"}</strong>
                  <p>{selected?.firstShard ?? "Select a model from inventory"}</p>
                </div>
                <dl className="spec-list">
                  <div><dt>Context</dt><dd>{profile?.context.toLocaleString() ?? "—"}</dd></div>
                  <div><dt>GPU layers</dt><dd>{profile?.gpuLayers ?? "—"}</dd></div>
                  <div><dt>Batch / uBatch</dt><dd>{profile ? `${profile.batch} / ${profile.ubatch}` : "—"}</dd></div>
                  <div><dt>Draft depth</dt><dd>{profile?.specType !== "none" ? profile?.draftMax : "OFF"}</dd></div>
                </dl>
                <button className="text-button" onClick={() => setView("profile")}>Edit exact launch profile →</button>
              </article>

              <article className="machine-panel terminal-panel">
                <div className="panel-title">
                  <SquareTerminal size={17} />
                  <h2>Server log</h2>
                  <span className="log-path">{status.logPath ?? "buffer offline"}</span>
                </div>
                {status.running ? <pre>{log}</pre> : <div className="log-empty"><SquareTerminal size={26} /><strong>Server is stopped</strong><span>Start this profile to stream llama-server output here.</span></div>}
              </article>
            </div>
          </section>
        )}

        {view === "models" && (
          <section className="screen inventory-screen">
            <div className="section-heading">
              <div>
                <h1>Logical inventory</h1>
                <p>Split shards grouped; companions kept separate from targets.</p>
              </div>
              <button className="button secondary" onClick={scan} disabled={busy === "scan"}>
                <RefreshCw size={16} className={busy === "scan" ? "spin" : ""} /> Rescan
              </button>
            </div>
            <div className="path-bar">
              <HardDrive size={16} />
              <input value={modelRoot} onChange={(event) => setModelRoot(event.target.value)} aria-label="Model root" placeholder="Choose a folder containing GGUF files" />
              <button className="path-action" onClick={chooseModelFolder}><FolderOpen size={15} /> Choose</button>
              <span>{models.length} targets · {bytesLabel(totalBytes)}</span>
            </div>
            <table className="inventory-table" aria-label="Model inventory">
              <thead>
                <tr>
                  <th scope="col">Target</th><th scope="col">Quant</th><th scope="col">Footprint</th><th scope="col">Shards</th><th scope="col">Companions</th><th scope="col">Status</th>
                </tr>
              </thead>
              <tbody>
                {models.map((model) => (
                  <tr
                    className={selectedId === model.id ? "selected" : ""}
                    key={model.id}
                    onClick={() => loadProfile(model)}
                  >
                    <td className="model-cell">
                      <button
                        type="button"
                        className="row-target"
                        onClick={() => loadProfile(model)}
                      >
                        <strong>{model.name}</strong><small>{model.directory}</small>
                      </button>
                    </td>
                    <td>{model.quant}</td>
                    <td className="numeric">{bytesLabel(model.sizeBytes)}</td>
                    <td className="numeric">{model.shardCount}/{model.expectedShards}</td>
                    <td className="companion-stack">
                      {model.companions.length ? (() => {
                        const counts = new Map<string, number>();
                        model.companions.forEach((c) => counts.set(c.role, (counts.get(c.role) ?? 0) + 1));
                        return [...counts].map(([role, count]) => <i key={role}>{role.toUpperCase()}{count > 1 ? ` ×${count}` : ""}</i>);
                      })() : <small>None</small>}
                    </td>
                    <td className={model.complete ? "state-tag good" : "state-tag warning"}>
                      {model.complete ? "READY" : "INCOMPLETE"}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            {invalidCount > 0 && (
              <div className="warning-band"><TriangleAlert size={17} /><strong>{invalidCount} target blocked</strong><span>Missing shards must be restored before launch.</span></div>
            )}
          </section>
        )}

        {view === "catalog" && (
          <section className="screen catalog-screen">
            <div className="section-heading">
              <div>
                <h1>HF Catalog</h1>
                <p>A developer-curated list. Model bytes travel directly from Hugging Face to this machine.</p>
              </div>
              <div className="actions">
                {catalogSnapshot && (() => {
                  const origin = originLabel(catalogSnapshot.origin);
                  return <span className={origin.tone === "ok" ? "state-tag good" : "state-tag warning"}>{origin.label}</span>;
                })()}
                <button className="button secondary" onClick={loadModelCatalog} disabled={catalogBusy}>
                  <RefreshCw size={16} className={catalogBusy ? "spin" : ""} /> Refresh list
                </button>
              </div>
            </div>

            <div className="path-bar catalog-destination">
              <HardDrive size={16} />
              <input value={modelRoot} onChange={(event) => setModelRoot(event.target.value)} aria-label="Download destination" placeholder="Choose where downloaded GGUF files should go" />
              <button className="path-action" onClick={chooseModelFolder}><FolderOpen size={15} /> Choose</button>
              <span>DIRECT TO MODEL FOLDER</span>
            </div>

            <div className="catalog-filters machine-panel" aria-label="Catalog filters">
              <label className="catalog-search">Search<input value={catalogSearch} onChange={(event) => setCatalogSearch(event.target.value)} placeholder="Family, publisher, tag, repository…" /></label>
              <label>Use<select value={catalogTag} onChange={(event) => setCatalogTag(event.target.value)}><option value="">Any use</option>{catalogTags.map((tag) => <option key={tag}>{tag}</option>)}</select></label>
              <label>Quant<select value={catalogQuant} onChange={(event) => setCatalogQuant(event.target.value)}><option value="">Any quant</option>{catalogQuants.map((quant) => <option key={quant}>{quant}</option>)}</select></label>
              <label>Author<select value={catalogAuthor} onChange={(event) => setCatalogAuthor(event.target.value)}><option value="">Any author</option>{catalogAuthors.map((author) => <option key={author}>{author}</option>)}</select></label>
              <label>Licence<select value={catalogLicense} onChange={(event) => setCatalogLicense(event.target.value)}><option value="">Any licence</option>{catalogLicenses.map((licence) => <option key={licence}>{licence}</option>)}</select></label>
              <label>Pipeline<select value={catalogPipeline} onChange={(event) => setCatalogPipeline(event.target.value)}><option value="">Any pipeline</option>{catalogPipelines.map((pipeline) => <option key={pipeline}>{pipeline}</option>)}</select></label>
              <label>Architecture<select value={catalogArchitecture} onChange={(event) => setCatalogArchitecture(event.target.value)}><option value="">Any architecture</option>{catalogArchitectures.map((architecture) => <option key={architecture}>{architecture}</option>)}</select></label>
              <label>Available under<select value={catalogMaxGiB} onChange={(event) => setCatalogMaxGiB(Number(event.target.value))}><option value={0}>Any size</option>{[2, 4, 8, 16, 32, 64].map((size) => <option key={size} value={size}>≤ {size} GiB</option>)}</select></label>
              <label>Order<select value={catalogSort} onChange={(event) => setCatalogSort(event.target.value as CatalogQuery["sort"])}><option value="downloads">Downloads</option><option value="likes">Likes</option><option value="name">Name</option><option value="size">Smallest file</option></select></label>
              <label className="toggle-line catalog-toggle"><input type="checkbox" checked={catalogHideGated} onChange={(event) => setCatalogHideGated(event.target.checked)} /> Hide gated</label>
              <label className="toggle-line catalog-toggle"><input type="checkbox" checked={catalogFitEnabled} onChange={(event) => setCatalogFitEnabled(event.target.checked)} /> Hardware fit{
                catalogFitBudget && catalogFitBudget.budgetBytes > 0
                  ? ` · ≤ ${bytesLabel(Math.floor((catalogFitBudget.budgetBytes * catalogFitPerMille) / 1000))} on ${catalogFitBudget.source} budget`
                  : " · budget unknown"
              }</label>
              {catalogFitEnabled && (
                <label>Fit budget<select value={catalogFitPerMille} onChange={(event) => setCatalogFitPerMille(Number(event.target.value))}>{[250, 500, 750, 1000].map((perMille) => <option key={perMille} value={perMille}>{perMille / 10}% of budget</option>)}</select></label>
              )}
            </div>

            <div className="catalog-layout">
              <div className="catalog-results">
                <div className="catalog-result-count">
                  <strong>{catalogRows.length}</strong>
                  <span>of {catalogSnapshot?.catalog.models.length ?? 0} curated models</span>
                  {catalogSnapshot && <small>LIST UPDATED {catalogSnapshot.catalog.updated || "UNKNOWN"}</small>}
                  {catalogSnapshot?.lastSuccessSecs && (
                    <small>LAST SUCCESS {new Date(Number(catalogSnapshot.lastSuccessSecs) * 1000).toLocaleString()}</small>
                  )}
                  {typeof catalogSnapshot?.cooldownRemainingMinutes === "number" && (
                    <small>COOLDOWN {catalogSnapshot.cooldownRemainingMinutes} MIN LEFT</small>
                  )}
                </div>
                {catalogBusy && !catalogSnapshot && <div className="catalog-empty machine-panel"><RefreshCw size={24} className="spin" /><strong>Fetching curated catalog</strong></div>}
                {!catalogBusy && catalogSnapshot && catalogRows.length === 0 && <div className="catalog-empty machine-panel"><Database size={24} /><strong>No curated model matches these filters</strong><span>Clear one or more filters.</span></div>}
                {catalogRows.map((model) => {
                  const filename = catalogFiles[model.id] ?? model.files[0]?.filename ?? "";
                  const file = model.files.find((entry) => entry.filename === filename) ?? model.files[0];
                  if (!file) return null;
                  // Progress and cancellation follow the running job for
                  // this file, not the currently edited destination (FE-11).
                  const activeJob = activeDownloadJob(downloadJobs, downloads, model.repo, file.filename);
                  const latestJob = newestDownloadJob(downloadJobs, model.repo, file.filename);
                  const progressKey = (activeJob ?? latestJob)?.key
                    ?? downloadKey(model.repo, file.filename, modelRoot, catalogRevision(file));
                  const progress = downloads[progressKey];
                  const running = activeJob !== undefined;
                  const alreadyOnDisk = inventoryHasFile(file.filename) || progress?.state === "done";
                  const readiness = downloadReadiness({ destination: modelRoot, running, alreadyOnDisk, gated: model.gated, hasToken: hfToken.configured });
                  return (
                    <article className="machine-panel catalog-model" key={model.id}>
                      <div className="catalog-model-head">
                        <div>
                          <button className="catalog-repo" onClick={() => openUrl(`https://huggingface.co/${model.repo}`)}>{model.repo}<ExternalLink size={12} /></button>
                          <strong>{model.family || model.repo.split("/")[1]}</strong>
                          <span>{model.parameters || "PARAMETERS UNKNOWN"} · BY {model.publisher || model.repo.split("/")[0]}</span>
                        </div>
                        {model.gated && <span className="state-tag warning">GATED · TOKEN + LICENCE</span>}
                        {model.userSourced && <span className="state-tag">USER ADDED · LOCAL ONLY</span>}
                      </div>
                      {model.summary && <p className="catalog-summary">{model.summary}</p>}
                      <div className="catalog-meta">
                        <span>{model.downloads.toLocaleString()} downloads</span>
                        <span>{model.likes.toLocaleString()} likes</span>
                        {model.tags.map((tag) => <i key={tag}>{tag.toUpperCase()}</i>)}
                      </div>
                      <div className="catalog-file-row">
                        <label>Build<select value={file.filename} onChange={(event) => setCatalogFiles((current) => ({ ...current, [model.id]: event.target.value }))}>{model.files.map((entry) => <option key={entry.filename} value={entry.filename}>{entry.quant} · {bytesLabel(entry.sizeBytes)}</option>)}</select></label>
                        <div className="catalog-filename"><span>{file.filename}</span>{file.userSourced && <span className="state-tag">USER FILE · LOCAL DIGEST</span>}<small>{bytesLabel(file.sizeBytes)} · 4 PARALLEL RANGES</small></div>
                        {running ? (
                          <button className="button danger" onClick={() => cancelCatalogDownload(activeJob!)}><CircleStop size={15} /> Keep & stop</button>
                        ) : (
                          <button className={alreadyOnDisk ? "button is-current" : "button primary"} disabled={!readiness.canStart} title={readiness.reason} onClick={() => startCatalogDownload(model, file)}>
                            {alreadyOnDisk ? <><BadgeCheck size={15} /> Verify file</> : <><Download size={15} /> {progress?.state === "error" ? "Resume" : "Download"}</>}
                          </button>
                        )}
                      </div>
                      {progress && (
                        <div className={`download-progress ${progress.state}`}>
                          <div><b style={{ width: `${downloadPercent(progress.downloaded, progress.total)}%` }} /></div>
                          <span>{progress.state.toUpperCase()} · {bytesLabel(progress.downloaded)} / {bytesLabel(progress.total)}</span>
                          <small>{rateLabel(progress.bytesPerSecond)} {etaLabel(progress.downloaded, progress.total, progress.bytesPerSecond)}</small>
                          {progress.message && <p>{progress.message}</p>}
                        </div>
                      )}
                      {!readiness.canStart && !running && !alreadyOnDisk && <p className="catalog-blocker">{readiness.reason}</p>}
                    </article>
                  );
                })}
              </div>

              <aside className="machine-panel catalog-sidebar">
                <div className="panel-title"><KeyRound size={17} /><h2>Hugging Face access</h2><span className={hfToken.configured ? "state-tag good" : "state-tag warning"}>{hfToken.configured ? `TOKEN ${hfToken.masked}` : "ANONYMOUS"}</span></div>
                <div className="catalog-token-body">
                  <p>A token is still useful: it enables gated repositories after you accept their licence and applies your account’s higher resolver rate limits. It does not guarantee higher raw bandwidth.</p>
                  <label>Read token<div className="key-row"><input type="password" autoComplete="off" value={hfTokenDraft} onChange={(event) => setHfTokenDraft(event.target.value)} placeholder={hfToken.configured ? `Stored ${hfToken.masked}` : "hf_…"} /><button className="button secondary" disabled={!hfTokenDraft.trim() || catalogBusy} onClick={saveHfToken}>Store</button></div></label>
                  {hfToken.configured && <button className="text-link" onClick={removeHfToken}>Remove stored token</button>}
                  {hfToken.cleanupNotice && (
                    <p role="status" className="token-cleanup-notice">{hfToken.cleanupNotice}</p>
                  )}
                  <button className="text-link" onClick={() => openUrl("https://huggingface.co/settings/tokens")}>Create a read token on Hugging Face ↗</button>
                </div>
                <dl className="runtime-facts catalog-transfer-facts">
                  <div><dt>Data route</dt><dd>Hugging Face → this PC. Localmotive never proxies model bytes.</dd></div>
                  <div><dt>Resume</dt><dd>Per-chunk progress survives interruption in .part metadata.</dd></div>
                  <div><dt>Parallelism</dt><dd>Four ranged HTTPS connections; one when the CDN does not support ranges.</dd></div>
                  <div><dt>Integrity</dt><dd>Final size always checked; SHA-256 verified when Hugging Face publishes it as the object ETag.</dd></div>
                  <div><dt>Secret storage</dt><dd>Windows Credential Manager service “Localmotive HF”; never local storage or logs.</dd></div>
                </dl>
              </aside>
            </div>
          </section>
        )}

        {view === "runtime" && (
          <section className="screen runtime-screen">
            <div className="section-heading">
              <div>
                <h1>Runtime manager</h1>
                <p>Detect this Windows PC, choose the right backend, and keep llama.cpp updateable.</p>
              </div>
              <button className="button secondary" onClick={loadRuntimeSetup} disabled={runtimeCatalogLoading}>
                <RefreshCw size={16} className={runtimeCatalogLoading ? "spin" : ""} /> Check latest release
              </button>
            </div>

            <div className="setup-steps" aria-label="First-run setup">
              <div className={runtimePath ? "setup-step done" : "setup-step active"}><span>1</span><strong>Runtime</strong><small>{runtimePath ? "Path selected" : "Install or choose"}</small></div>
              <div className={modelRoot ? "setup-step done" : "setup-step"}><span>2</span><strong>Models</strong><small>{modelRoot ? "Folder selected" : "Choose in Inventory"}</small></div>
              <div className={runtimePath && modelRoot ? "setup-step done" : "setup-step"}><span>3</span><strong>Serve</strong><small>{runtimePath && modelRoot ? "Ready to validate" : "Complete setup"}</small></div>
            </div>

            <div className="hardware-panel">
              <div className="hardware-icon"><MonitorCog size={26} /></div>
              <div>
                <span className="instrument-label">DETECTED WINDOWS HARDWARE</span>
                <strong>{hardware?.gpuNames.length ? hardware.gpuNames.join(" · ") : "CPU / GPU detection pending"}</strong>
                <p>{runtimeCatalog?.recommendationReason ?? "Waiting for the approved catalog to evaluate exact product evidence…"}</p>
                <small className="hardware-source">{hardware?.detectionStatus}</small>
              </div>
              <div className="hardware-meta"><span>{hardware?.architecture ?? "—"}</span><span>{hardware?.vendor.toUpperCase() ?? "—"}</span>{hardware?.cudaMajor && <span>CUDA {hardware.cudaMajor}</span>}{hardware?.driverVersion && <span>Driver {hardware.driverVersion}</span>}</div>
            </div>

            {hardware && (
              <div className="hardware-evidence-grid" aria-label="Hardware evidence">
                <article className="hardware-evidence-card">
                  <span className="instrument-label">SYSTEM MEMORY</span>
                  <strong>{hardware.systemMemory.totalPhysicalBytes.value === null ? "UNKNOWN" : bytesLabel(hardware.systemMemory.totalPhysicalBytes.value)}</strong>
                  <dl>
                    <div><dt>Available</dt><dd>{hardware.systemMemory.availablePhysicalBytes.value === null ? "Unknown" : bytesLabel(hardware.systemMemory.availablePhysicalBytes.value)}<small>{hardware.systemMemory.availablePhysicalBytes.source.detail} · {hardware.systemMemory.availablePhysicalBytes.observedAtMs ? new Date(hardware.systemMemory.availablePhysicalBytes.observedAtMs).toISOString() : "not observed"}</small></dd></div>
                    <div><dt>Load</dt><dd>{hardware.systemMemory.memoryLoadPercent.value === null ? "Unknown" : `${hardware.systemMemory.memoryLoadPercent.value}%`}<small>{hardware.systemMemory.memoryLoadPercent.source.detail} · {hardware.systemMemory.memoryLoadPercent.observedAtMs ? new Date(hardware.systemMemory.memoryLoadPercent.observedAtMs).toISOString() : "not observed"}</small></dd></div>
                  </dl>
                  <small>{hardware.systemMemory.totalPhysicalBytes.source.detail} · {hardware.systemMemory.totalPhysicalBytes.observedAtMs ? new Date(hardware.systemMemory.totalPhysicalBytes.observedAtMs).toISOString() : "not observed"}</small>
                </article>
                {hardware.adapters.map((adapter) => {
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
                {hardware.manualOverrides.map((override) => (
                  <article className="hardware-evidence-card hardware-override" key={`override-${override.adapterId}`}>
                    <span className="instrument-label">MANUAL OVERRIDE · {override.adapterId}</span>
                    <strong>User supplied</strong>
                    <p>{override.note || "No note supplied."}</p>
                  </article>
                ))}
              </div>
            )}

            {hardware && hardware.adapters.length > 0 && (
              <label className="runtime-adapter-picker">
                GPU adapter for managed runtime installation
                <select
                  value={selectedRuntimeAdapterId}
                  onChange={(event) => void selectRuntimeAdapter(event.target.value)}
                  disabled={runtimeCatalogLoading}
                >
                  {hardware.adapters.length > 1 && <option value="">Select one detected adapter</option>}
                  {hardware.adapters.map((adapter) => (
                    <option value={adapter.adapterId} key={adapter.adapterId}>
                      {adapter.name} · {adapter.adapterId}
                    </option>
                  ))}
                </select>
              </label>
            )}

            {installing && (
              <div className="download-progress downloading" role="status" aria-live="polite">
                {runtimeInstallProgress ? (
                  <>
                    <div><b style={{ width: `${downloadPercent(runtimeInstallProgress.downloaded, runtimeInstallProgress.total)}%` }} /></div>
                    <span>
                      {runtimeInstallProgress.assetName} · {bytesLabel(runtimeInstallProgress.downloaded)} / {bytesLabel(runtimeInstallProgress.total)}
                    </span>
                  </>
                ) : (
                  <span>Validating hardware and preparing the approved runtime download…</span>
                )}
                <button className="button danger" onClick={cancelRuntimeInstall} disabled={runtimeInstallCancelling}>
                  <CircleStop size={15} /> {runtimeInstallCancelling ? "Stopping…" : "Keep partial download and stop"}
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
                      onClick={loadRuntimeSetup}
                    >
                      <RefreshCw size={14} /> Refresh catalog
                    </button>
                    <button className="text-button runtime-source" onClick={() => openUrl("https://github.com/ggml-org/llama.cpp/releases")}><Download size={14} /> GitHub releases</button>
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
                      <button className="button secondary" onClick={loadRuntimeSetup}>Retry</button>
                    </div>
                  )}
                  {runtimeCatalogState.kind === "idle" && (
                    <div className="runtime-catalog-message">
                      <span>The runtime catalog is not loaded.</span>
                      <button className="button secondary" onClick={loadRuntimeSetup}>Load catalog</button>
                    </div>
                  )}
                  {runtimeCatalogState.kind === "empty" && (
                    <div className="runtime-catalog-message" role="status">
                      <strong>No approved runtime matches this system.</strong>
                      <span>Review the blocked backend evidence or retry catalog retrieval.</span>
                      <button className="button secondary" onClick={loadRuntimeSetup}>Retry</button>
                    </div>
                  )}
                  {runtimeCatalogState.kind === "ready" && runtimeCatalogState.catalog.options.map((option) => {
                    const state = runtimeOptionState(option, runtimeCatalogState.catalog.tag, managedRuntimes, runtimeIdentity, runtime?.build ?? null);
                    const downloading = installing === option.id;
                    const adapterRequired = option.backend !== "cpu" && !selectedRuntimeAdapterId;
                    const managedInstalled = managedRuntimes.some((record) => record.installKey === option.installKey);
                    const supportLabel = `DIRECT RUNTIME · UPSTREAM EVIDENCE ONLY · ${hardware?.architecture ?? "unknown architecture"} · ${option.backend} · ${runtimeCatalogState.catalog.tag}`;
                    return (
                    <article className={`runtime-option${option.recommended ? " recommended" : ""}${state.kind === "active" ? " is-active" : ""}`} key={`${option.id}:${option.asset.name}`} aria-label={`${option.label} runtime option, ${option.recommended ? "recommended" : "not recommended"}, ${state.kind}`}>
                      <div className="runtime-option-main">
                        <div className="runtime-option-title"><strong>{option.label}</strong>{state.kind === "active" && <span className="state-tag good"><BadgeCheck size={10} /> ACTIVE · b{runtime?.build}</span>}{state.kind === "update" && <span className="state-tag warning"><ArrowUp size={10} /> UPDATE FROM {state.from.toUpperCase()}</span>}{state.kind === "mismatch" && <span className="state-tag warning">MISMATCH · REINSTALL</span>}<span className="state-tag warning" title="Product support requires an exact L4 compatibility record">PRODUCT SUPPORT NOT VALIDATED</span>{state.kind !== "active" && state.kind !== "update" && state.kind !== "mismatch" && option.recommended && <span className="state-tag good">RECOMMENDED</span>}</div>
                        <span
                          className="runtime-role runtime-scope"
                          data-support-architecture={hardware?.architecture ?? "unknown"}
                          data-support-backend={option.backend}
                          data-runtime-revision={runtimeCatalogState.catalog.tag}
                        >
                          {supportLabel} · exact product configuration untested
                        </span>
                        <span className="runtime-role">{state.kind === "active" ? (isManagedPath ? "This is the runtime in use" : "Your existing executable matches this package") : state.kind === "update" ? `Newer official build available for the runtime in use` : state.kind === "mismatch" ? "Installed files disagree with the install record · reinstall before launch" : state.kind === "use" ? "Already downloaded · not the active runtime" : option.recommended ? "Recommended for this PC" : option.backend === "cpu" ? "CPU fallback" : option.backend === "vulkan" ? "Compatibility fallback" : option.backend === "cuda" ? "Alternative CUDA package" : "Optional backend"}</span>
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
                          <button className="button primary" onClick={() => installRuntime(option)} disabled={Boolean(installing) || adapterRequired}>
                            <ArrowUp size={15} /> {downloading ? "Downloading…" : `Update to ${runtimeCatalogState.catalog.tag}`}
                          </button>
                        )}
                        {state.kind === "use" && (
                          <button className="button secondary" onClick={() => activateRuntime(state.runtimePath)} disabled={Boolean(installing) || busy === "runtime"}>
                            <Play size={15} /> Use this build
                          </button>
                        )}
                        {state.kind === "install" && (
                          <button className={option.recommended ? "button primary" : "button secondary"} onClick={() => installRuntime(option)} disabled={Boolean(installing) || adapterRequired}>
                            <Download size={15} /> {downloading ? "Downloading…" : "Install"}
                          </button>
                        )}
                        {state.kind === "mismatch" && (
                          <button className="button primary" onClick={() => installRuntime(option)} disabled={Boolean(installing) || adapterRequired}>
                            <Download size={15} /> {downloading ? "Downloading…" : "Reinstall"}
                          </button>
                        )}
                        {managedInstalled && (
                          <button
                            className="button secondary"
                            onClick={() => runManagedHealth(option)}
                            disabled={Boolean(installing) || Boolean(healthRunning) || adapterRequired}
                          >
                            <ShieldCheck size={15} /> {healthRunning === option.installKey ? "Checking…" : "Run 7-stage health"}
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
                                <button className="text-button runtime-source" onClick={() => openUrl(entry.evidenceUrls[index])}>
                                  <ShieldCheck size={12} /> View {jobName} evidence
                                </button>
                              )}
                            </span>
                          ))}
                          {entry.evidenceUrls.slice(entry.blockingJobs.length).map((url) => (
                            <button className="text-button runtime-source" key={url} onClick={() => openUrl(url)}>
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
                  <span className={runtime ? "plate-light ok" : "plate-light"} />
                  <div><strong>{runtime ? `llama.cpp b${runtime.build}` : "Not configured"}</strong>{runtime && <span className={isManagedPath ? "runtime-trust managed" : "runtime-trust supplied"}>{isManagedPath ? "MANAGED · VERIFIED DOWNLOAD" : "USER-SUPPLIED · VERSION INSPECTED"}</span>}{runtimeIdentity && runtimeIdentity.backend !== "unknown" && <span className="runtime-trust neutral">{runtimeIdentity.backend.toUpperCase()}{runtimeIdentity.cudaMajor ? ` ${runtimeIdentity.cudaMajor}` : ""} · {runtimeIdentity.source === "manifest" ? "FROM MANIFEST" : "FROM DLLS"}</span>}<small>{runtimePath || "Install a recommended build or choose an existing executable."}</small></div>
                </div>
                <label>Existing llama-server.exe<input value={runtimePath} onChange={(event) => {
                  // The inspected identity belongs to the old path (FE-15):
                  // clear stale capabilities until this path is inspected.
                  if (event.target.value !== runtimePath) setRuntime(null);
                  setRuntimePath(event.target.value);
                }} placeholder="Path to llama-server.exe" /></label>
                <div className="runtime-side-actions">
                  <button className="button secondary" onClick={chooseExistingRuntime}><FolderOpen size={15} /> Browse</button>
                  <button className="button secondary" onClick={() => inspect()} disabled={!runtimePath}><Cpu size={15} /> Inspect</button>
                </div>
                {managedRuntimes.length > 0 && (
                  <dl className="runtime-facts managed-list">
                    <div><dt>Installed managed builds</dt><dd>{managedRuntimes.map((record) => <button key={record.runtimePath} className={`managed-entry${record.runtimePath === runtimePath ? " current" : ""}`} onClick={() => activateRuntime(record.runtimePath)} disabled={record.runtimePath === runtimePath}>{record.tag} · {record.installKey}{record.runtimePath === runtimePath ? " · active" : ""}</button>)}</dd></div>
                  </dl>
                )}
                <div className="managed-health" aria-live="polite">
                  <span className="instrument-label">SEVEN-STAGE HEALTH CONTRACT</span>
                  <p>Localmotive retrieves and verifies the immutable pinned SmolLM2 health model before each run.</p>
                  {healthRunning && (
                    <>
                      {healthModelProgress && (
                        <div className="download-progress downloading" role="status">
                          <div><b style={{ width: `${downloadPercent(healthModelProgress.downloaded, healthModelProgress.total)}%` }} /></div>
                          <span>Pinned health model · {bytesLabel(healthModelProgress.downloaded)} / {bytesLabel(healthModelProgress.total)}</span>
                        </div>
                      )}
                      <button className="button danger" onClick={cancelManagedHealth} disabled={healthCancelling}>
                        <CircleStop size={15} /> {healthCancelling ? "Stopping…" : "Cancel and clean up"}
                      </button>
                    </>
                  )}
                  {healthResult && (
                    <div className={`health-result ${healthOutcome}`}>
                      <strong>{healthOutcome?.toUpperCase()}</strong>
                      <small>Runtime {healthResult.runtimeId} · model SHA-256 {healthResult.modelSha256}</small>
                      <ol>
                        {healthResult.stages.map((stage) => (
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
                    <button className="button secondary" onClick={repairHealthModel} disabled={healthRepairing || Boolean(healthRunning)}>
                      <Wrench size={15} /> {healthRepairing ? "Repairing…" : "Repair cached model"}
                    </button>
                    {healthRepairing && (
                      <button className="button danger" onClick={cancelHealthRepair}>
                        <CircleStop size={15} /> Cancel repair
                      </button>
                    )}
                    {healthRepairNotice && <small role="status">{healthRepairNotice}</small>}
                  </div>
                </div>
                <dl className="runtime-facts">
                  <div><dt>Managed folder</dt><dd>{runtimeRoot || "Resolving…"}</dd></div>
                  <div><dt>Verification</dt><dd>Size always; SHA-256 before activation when published</dd></div>
                  <div><dt>CUDA packages</dt><dd>Binary + matching cudart DLLs</dd></div>
                  <div><dt>Existing files</dt><dd>User-supplied; version/help inspected, not download-verified</dd></div>
                  <div><dt>Updates</dt><dd>Versioned; existing installs preserved</dd></div>
                </dl>
              </aside>
            </div>
          </section>
        )}

        {view === "profile" && profile && (
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
                  <button onClick={() => { const advanced = document.querySelector(".advanced-zone") as HTMLDetailsElement | null; if (advanced) { advanced.open = true; advanced.scrollIntoView({ behavior: "smooth", block: "start" }); } }}>Jump to Advanced</button>
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
                    <select value={profile.specType} onChange={(e) => setProfile({ ...profile, specType: e.target.value })}>
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
                <pre>{command || "Edit a setting to compose the command."}</pre>
                <p className="security-note">Composed without probing the runtime: capability filtering, artifact checks, and managed-runtime trust are enforced when the profile is validated and launched.</p>
                <div className="capability-list">
                  <span>RUNTIME CAPABILITIES</span>
                  {(runtime?.specTypes ?? []).map((type) => <i key={type}>{type}</i>)}
                </div>
                <p>Only methods advertised by this executable are offered. Raw arguments remain available for new llama.cpp features.</p>
              </aside>
            </div>
          </section>
        )}

        {view === "tune" && (
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
                      <small className="field-help">Stored under “Localmotive” in Windows Credential Manager, not in this app’s settings. {provider && <button className="text-link" onClick={() => openUrl(provider.consoleUrl)}>Get a key from {provider.label} →</button>}</small>
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
        )}

        {view === "about" && (
          <section className="screen about-screen">
            <div className="section-heading">
              <div>
                <h1>About</h1>
                <p>What this build is, where it keeps things, and what it is made of.</p>
              </div>
              <div className="actions">
                <button className="button secondary" onClick={() => openUrl(about?.repository ?? "https://github.com/sato942/localmotive")}><Link2 size={15} /> Project on GitHub</button>
              </div>
            </div>

            <div className="about-layout">
              <article className="machine-panel about-identity">
                <div className="about-mark">LM</div>
                <div>
                  <strong>{about?.name ?? "Localmotive"}</strong>
                  <span>Version {about?.version ?? "—"}</span>
                  <p>A Windows control plane for local GGUF inference: it manages official llama.cpp runtimes, discovers models, builds exact launch profiles, supervises the server, measures throughput, and tunes settings with a cloud advisor.</p>
                </div>
              </article>

              <article className="machine-panel">
                <div className="panel-title"><Info size={17} /><h2>Build</h2></div>
                <dl className="spec-list">
                  <div><dt>Version</dt><dd>{about?.version ?? "—"}</dd></div>
                  <div><dt>Bundle identifier</dt><dd>{about?.identifier ?? "—"}</dd></div>
                  <div><dt>Platform</dt><dd>{about?.os.toUpperCase() ?? "—"}</dd></div>
                  <div><dt>Framework</dt><dd>TAURI {about?.tauriVersion ?? "2"} · RUST · REACT</dd></div>
                  <div><dt>Licence</dt><dd>{about?.license || "MIT"}</dd></div>
                </dl>
              </article>

              <article className="machine-panel">
                <div className="panel-title"><MonitorCog size={17} /><h2>This machine</h2></div>
                <dl className="spec-list">
                  <div><dt>Runtime in use</dt><dd>{runtime ? `LLAMA.CPP B${runtime.build}` : "NONE"}</dd></div>
                  <div><dt>Backend</dt><dd>{runtimeIdentity && runtimeIdentity.backend !== "unknown" ? `${runtimeIdentity.backend.toUpperCase()}${runtimeIdentity.cudaMajor ? ` ${runtimeIdentity.cudaMajor}` : ""}` : "—"}</dd></div>
                  <div><dt>Accelerator</dt><dd>{hardware?.gpuNames[0]?.toUpperCase() ?? hardware?.vendor.toUpperCase() ?? "—"}</dd></div>
                  <div><dt>Models found</dt><dd>{models.length} ({bytesLabel(totalBytes)})</dd></div>
                </dl>
              </article>

              <article className="machine-panel about-paths">
                <div className="panel-title"><FolderOpen size={17} /><h2>Where things live</h2></div>
                <dl className="runtime-facts">
                  <div><dt>Model folder</dt><dd>{modelRoot || "Not selected"}</dd></div>
                  <div><dt>Managed runtimes</dt><dd>{about?.runtimeRoot || "—"}</dd></div>
                  <div><dt>Server logs</dt><dd>{about?.logDir || "—"}</dd></div>
                  <div><dt>Cloud keys</dt><dd>Windows Credential Manager, service “Localmotive” — never in settings or logs</dd></div>
                  <div><dt>Settings</dt><dd>Browser local storage in this app (model folder, runtime, profiles, tuning reports)</dd></div>
                </dl>
              </article>

              <article className="machine-panel about-credits">
                <div className="panel-title"><ShieldCheck size={17} /><h2>Built on</h2></div>
                <div className="credit-list">
                  <button className="credit" onClick={() => openUrl("https://github.com/ggml-org/llama.cpp")}><strong>llama.cpp</strong><span>ggml-org · MIT — the inference engine and every managed runtime binary</span></button>
                  <button className="credit" onClick={() => openUrl("https://tauri.app")}><strong>Tauri {about?.tauriVersion ?? "2"}</strong><span>Apache-2.0 / MIT — desktop shell</span></button>
                  <button className="credit" onClick={() => openUrl("https://react.dev")}><strong>React + TypeScript + Vite</strong><span>MIT — interface</span></button>
                  <button className="credit" onClick={() => openUrl("https://lucide.dev")}><strong>Lucide</strong><span>ISC — icons</span></button>
                </div>
                <p className="group-note about-note">Localmotive is not affiliated with ggml-org. Runtime binaries are downloaded directly from official llama.cpp GitHub releases and verified against their published size and SHA-256 before use.</p>
              </article>
            </div>
          </section>
        )}

        {/* FE-05: the evidence panel stays mounted for the app's lifetime so
            navigation cannot erase active runs or completed evidence. */}
        <section
          className="screen benchmark-screen"
          style={view === "benchmark" ? undefined : { display: "none" }}
        >
            <div className="section-heading">
              <div>
                <h1>Generation benchmark</h1>
                <p>One warmup, fixed deterministic workload, repeated server-reported throughput.</p>
              </div>
              <button className="button primary" onClick={runBenchmark} disabled={!status.running || busy === "benchmark" || evidenceRun !== null}>
                <Activity size={16} /> {busy === "benchmark" ? "Measuring…" : "Run benchmark"}
              </button>
            </div>
            {!status.running && <div className="warning-band"><TriangleAlert size={17} /><strong>Server required</strong><span>Start a profile before measuring it.</span></div>}
            <div className="benchmark-grid">
              <article className="machine-panel benchmark-setup">
                <div className="panel-title"><TestTube2 size={17} /><h2>Test setup</h2></div>
                <label>Forced output tokens<input type="number" min="64" max="4096" value={tokens} onChange={(e) => setTokens(Number(e.target.value))} /></label>
                <label>Measured repeats<input type="number" min="1" max="10" value={repeats} onChange={(e) => setRepeats(Number(e.target.value))} /></label>
                <dl className="spec-list">
                  <div><dt>Sampling</dt><dd>GREEDY</dd></div>
                  <div><dt>Seed</dt><dd>42</dd></div>
                  <div><dt>Early EOS</dt><dd>IGNORED</dd></div>
                  <div><dt>Warmup</dt><dd>64 TOKENS</dd></div>
                </dl>
              </article>
              <article className="machine-panel results-panel">
                <div className="panel-title"><Gauge size={17} /><h2>Measured result</h2></div>
                {benchmark ? (
                  <>
                    <div className="result-main"><strong>{benchmark.meanTps.toFixed(2)}</strong><span>generation tok/s mean</span></div>
                    <div className="result-range"><span>Median {benchmark.medianTps.toFixed(2)}</span><span>Range {benchmark.minTps.toFixed(2)}–{benchmark.maxTps.toFixed(2)}</span></div>
                    <div className="sample-bars">
                      {benchmark.samples.map((sample, index) => <div key={index}><span>R{index + 1}</span><b style={{ width: `${(sample / benchmark.maxTps) * 100}%` }} /><em>{sample.toFixed(2)}</em></div>)}
                    </div>
                  </>
                ) : <div className="empty-result"><Activity size={28} /><p>No measurement recorded for this session.</p></div>}
              </article>
            </div>
            <V03EvidencePanel
              model={selected ?? null}
              profile={profile}
              serverStatus={status}
              initialHardware={hardware}
              onRunStateChange={setEvidenceRun}
            />
          </section>
      </main>
    </div>
  );
}

export default App;
