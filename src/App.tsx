import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Activity, BadgeCheck, Braces, CircleStop, Cpu, Database, Download, ExternalLink, FolderOpen, Gauge, HardDrive, Info, KeyRound, MonitorCog, Play, RefreshCw, Save, Settings2, Sparkles, TestTube2 } from "lucide-react";
import "./App.css";
import {
  managedHealthOutcome,
  bytesLabel,
  errorText,
  catalogBuildFit,
  catalogRevision,
  type CommandPreview,
  type ScanReport,
  reconcileDraftCompanion,
  contextChoices,
  DEFAULT_FIT_PER_MILLE,
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
  selectionAfterRescan,
  displayedSelection,
  profileIdentity,
  applySuggestedPort,
  responseIsCurrent,
  managedHealthRequest,
  normalizeProfile,
  normalizeTuningReport,
  safeJsonParse,
  originLabel,
  rateLabel,
  retainOrDisposeListener,
  runtimeCatalogErrorFromUnknown,
  runtimeCatalogViewState,
  runtimeInstallRequest,
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
  type DisclosureSection,
  type BriefDisclosure,
} from "./model";
import { AboutScreen } from "./screens/AboutScreen";
import { RuntimeScreen } from "./screens/RuntimeScreen";
import { TuneScreen } from "./screens/TuneScreen";
import { DashboardScreen } from "./screens/DashboardScreen";
import { PathText } from "./screens/PathText";
import { InventoryScreen } from "./screens/InventoryScreen";
import { BenchmarkScreen } from "./screens/BenchmarkScreen";

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

/// S-25 I2: the catalog filter IPC carries the full snapshot (~510 KB
/// measured at the current 158-model size) and the Rust round trip plus
/// re-render costs ~22-113 ms per keystroke. A short pause after typing
/// collapses a burst into one request without delaying a single keystroke's
/// own feedback (the input value stays immediate).
function useDebouncedValue<T>(value: T, delayMs: number): T {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const handle = window.setTimeout(() => setDebounced(value), delayMs);
    return () => window.clearTimeout(handle);
  }, [value, delayMs]);
  return debounced;
}

/// S-24 I1: a path that is truncated for layout keeps its FULL value
/// retrievable — the title carries it, the control is keyboard reachable, and
/// one click copies it. Identity is never hidden by ellipsis alone.

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
  const [command, setCommand] = useState<CommandPreview | null>(null);
  const [lastScan, setLastScan] = useState<ScanReport | null>(null);
  // S-23 I2: distinguish an empty-but-valid folder from a failed scan so the
  // empty inventory offers the right next action.
  const [scanFailed, setScanFailed] = useState(false);
  // FE-03: per-resource request sequences and live identity mirrors, so a
  // deferred response can never commit against a newer resource state.
  const cloudSeq = useRef(0);
  const ggufSeq = useRef(0);
  const previewSeq = useRef(0);
  const [log, setLog] = useState("Waiting for a managed server.");
  const [notice, setNotice] = useState("Inventory not scanned yet.");
  // S-22: secondary-action failures (dialogs, browser opener, local saves)
  // get a concise recovery line plus a copyable diagnostic, and never
  // replace a successful primary result.
  const [diagnostic, setDiagnostic] = useState<{ message: string; detail: string } | null>(null);

  function reportFailure(recovery: string, error: unknown) {
    const message = errorText(error);
    setNotice(`${recovery} ${message}`);
    let detail = message;
    if (error !== null && typeof error === "object") {
      try {
        detail = JSON.stringify(error);
      } catch {
        detail = message;
      }
    }
    setDiagnostic({ message, detail });
  }

  async function openExternal(url: string) {
    try {
      await openUrl(url);
    } catch (error) {
      reportFailure(
        `Could not open the browser. Copy this link instead: ${url}.`,
        error,
      );
    }
  }

  async function copyDiagnostic() {
    if (!diagnostic) return;
    try {
      await navigator.clipboard.writeText(diagnostic.detail);
      setNotice("Diagnostic copied to the clipboard.");
    } catch (error) {
      setNotice(`Copy failed; select the text manually. ${errorText(error)}`);
    }
  }
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
  const catalogSearchDebounced = useDebouncedValue(catalogSearch, 120);
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
  // Cloud disclosure (audit S-20): what the brief carries and how much of it
  // leaves the machine. Local inference and local exports are unaffected.
  const [disclosureSections, setDisclosureSections] = useState<DisclosureSection[]>([]);
  const [briefDisclosure, setBriefDisclosure] = useState<BriefDisclosure>(() => {
    const stored = localStorage.getItem("localmotive:tune-disclosure");
    return stored === "minimal" ? "minimal" : "full";
  });
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

  async function scan() {
    if (!modelRoot.trim()) {
      setNotice("Choose the folder that contains your GGUF models.");
      return;
    }
    setBusy("scan");
    try {
      const report = await invoke<ScanReport>("scan_models_report", { root: modelRoot });
      setLastScan(report);
      setScanFailed(false);
      const result = report.models;
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
      const problemNote =
        report.problems.length > 0 || report.truncated
          ? ` · ${report.problems.length} scan diagnostic${report.problems.length === 1 ? "" : "s"}${report.truncated ? " (bounded scan stopped early)" : ""}: ${report.problems
              .slice(0, 2)
              .map((problem) => problem.reason)
              .join("; ")}`
          : "";
      setNotice(`${result.length} logical targets indexed from ${modelRoot}${problemNote}`);
    } catch (error) {
      // A failed scan clears the selection and its editable profile as one
      // transition, so a stale draft can never masquerade as current.
      setModels([]);
      setScanFailed(true);
      clearCommittedSelection();
      setNotice(inTauri() ? errorText(error) : "Browser preview cannot scan local model files. Use the packaged app.");
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
      setNotice(inTauri() ? errorText(error) : "Browser preview cannot inspect local runtime files. Use the packaged app.");
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
      setNotice(errorText(error));
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
      setNotice(errorText(error));
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
      setHealthRepairNotice(errorText(error));
    } finally {
      setHealthRepairing(false);
    }
  }

  async function cancelHealthRepair() {
    try {
      await invoke("cancel_health_model_repair");
    } catch (error) {
      setHealthRepairNotice(errorText(error));
    }
  }

  async function chooseModelFolder() {
    // S-22: a rejected or cancelled dialog keeps the previous selection and
    // reports a recoverable failure instead of an unhandled rejection.
    const selected = await openDialog({ directory: true, multiple: false, title: "Choose your GGUF model folder" }).catch(
      (error) => {
        reportFailure("Could not open the folder picker. The current root is unchanged.", error);
        return null;
      },
    );
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
        refreshNote = ` Refresh pending: ${errorText(error)}`;
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
      if (keepLatestRequest(sequence, catalogLoadSeq.current)) setNotice(errorText(error));
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
      setNotice(errorText(error));
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
      setNotice(errorText(error));
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
          message: errorText(error),
        },
      }));
      setNotice(errorText(error));
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
      setNotice(`Could not stop ${job.filename}: ${errorText(error)}`);
    }
  }

  async function chooseExistingRuntime() {
    const selected = await openDialog({
      directory: false,
      multiple: false,
      title: "Choose llama-server.exe",
      filters: [{ name: "llama-server", extensions: ["exe"] }],
    }).catch((error) => {
      reportFailure("Could not open the file picker. No runtime was changed.", error);
      return null;
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
          if (responseIsCurrent(sequence, cloudSeq.current, nextProvider, providerIdRef.current)) setCloudCheck(errorText(error));
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
        setNotice(errorText(error));
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
      if (responseIsCurrent(sequence, cloudSeq.current, forProvider, providerIdRef.current)) setNotice(errorText(error));
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
      if (responseIsCurrent(sequence, cloudSeq.current, forProvider, providerIdRef.current)) setNotice(errorText(error));
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
      if (responseIsCurrent(sequence, cloudSeq.current, "openrouter", providerIdRef.current)) setNotice(errorText(error));
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
      if (responseIsCurrent(sequence, cloudSeq.current, forProvider, providerIdRef.current)) setCloudCheck(errorText(error));
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

  useEffect(() => {
    let cancelled = false;
    invoke<DisclosureSection[]>("tune_disclosure_list")
      .then((sections) => {
        if (!cancelled) setDisclosureSections(Array.isArray(sections) ? sections : []);
      })
      .catch(() => {
        if (!cancelled) setDisclosureSections([]);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  function changeDisclosure(mode: BriefDisclosure) {
    setBriefDisclosure(mode);
    localStorage.setItem("localmotive:tune-disclosure", mode);
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
          disclosure: briefDisclosure,
        },
      });
      setTuneReport(report);
      setTuneReportOrigin(run);
      localStorage.setItem(`localmotive:tuning:${run.modelId}`, JSON.stringify(report));
      const gain = report.baselineTps && report.bestTps ? ((report.bestTps / report.baselineTps - 1) * 100).toFixed(1) : null;
      setNotice(gain ? `Tuning finished: best ${report.bestTps?.toFixed(2)} tok/s (${Number(gain) >= 0 ? "+" : ""}${gain}% vs baseline). ${report.stoppedReason}.` : `Tuning finished. ${report.stoppedReason}.`);
    } catch (error) {
      setNotice(errorText(error));
      setTuneProgress({ phase: "error", message: errorText(error), trial: null });
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
      setNotice(errorText(error));
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
      const command = await invoke<CommandPreview>("preview_command", { profile });
      // FE-03: an old preview must not overwrite a newer one; an old profile
      // must not label the current one's command.
      const currentIdentity = profileRef.current ? profileIdentity(profileRef.current) : "";
      if (!responseIsCurrent(sequence, previewSeq.current, identity, currentIdentity)) return;
      setCommand(command);
    } catch (error) {
      if (keepLatestRequest(sequence, previewSeq.current)) {
        setCommand(
          inTauri()
            ? { powerShell: errorText(error), argv: "", cmd: null, cmdNotice: null }
            : { powerShell: "Browser preview cannot compose the command. Use the packaged app.", argv: "", cmd: null, cmdNotice: null },
        );
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
      setNotice(errorText(error));
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
      setNotice(errorText(error));
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
      setNotice(errorText(error));
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
      text: catalogSearchDebounced,
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
        if (keepLatestRequest(sequence, catalogFilterSeq.current)) setNotice(errorText(error));
      });
  }, [catalogSnapshot, catalogAllRows, catalogSearchDebounced, catalogTag, catalogQuant, catalogAuthor, catalogLicense, catalogPipeline, catalogArchitecture, catalogMaxGiB, catalogHideGated, catalogSort, catalogFitEnabled, catalogFitPerMille, catalogFitBudget]);

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
          {diagnostic ? (
            <span className="notice-diagnostic">
              <button className="text-link" onClick={() => void copyDiagnostic()}>
                Copy diagnostic
              </button>
              <details>
                <summary>Details</summary>
                <pre>{diagnostic.detail}</pre>
              </details>
            </span>
          ) : null}
        </div>
        {lastScan && (lastScan.problems.length > 0 || lastScan.truncated) && (
          <p className="scan-diagnostics" role="status">
            {lastScan.truncated
              ? `Bounded scan stopped early after reporting ${lastScan.problems.length} diagnostic${lastScan.problems.length === 1 ? "" : "s"}. `
              : `${lastScan.problems.length} scan diagnostic${lastScan.problems.length === 1 ? "" : "s"}. `}
            {lastScan.problems
              .slice(0, 3)
              .map((problem) => `${problem.path}: ${problem.reason}`)
              .join(" · ")}
          </p>
        )}

        {view === "dashboard" && (
          <DashboardScreen
            benchmark={benchmark}
            busy={busy}
            evidenceRun={evidenceRun}
            log={log}
            openExternal={openExternal}
            profile={profile}
            runtime={runtime}
            selected={selected}
            setView={setView}
            start={start}
            status={status}
            stop={stop}
            tuning={tuning}
          />
        )}
        {view === "models" && (
          <InventoryScreen
            busy={busy}
            chooseModelFolder={chooseModelFolder}
            invalidCount={invalidCount}
            lastScan={lastScan}
            loadProfile={loadProfile}
            modelRoot={modelRoot}
            models={models}
            scan={scan}
            scanFailed={scanFailed}
            selectedId={selectedId}
            setModelRoot={setModelRoot}
            totalBytes={totalBytes}
          />
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
                  ? ` · filter keeps models whose smallest build is ≤ ${bytesLabel(Math.floor((catalogFitBudget.budgetBytes * catalogFitPerMille) / 1000))} on ${catalogFitBudget.source}`
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
                  const smallestBytes = Math.min(...model.files.map((entry) => entry.sizeBytes));
                  const buildFit = catalogBuildFit(
                    file.sizeBytes,
                    smallestBytes,
                    catalogFitEnabled ? catalogFitPerMille : 0,
                    catalogFitBudget?.budgetBytes,
                  );
                  const alreadyOnDisk = inventoryHasFile(file.filename) || progress?.state === "done";
                  const readiness = downloadReadiness({ destination: modelRoot, running, alreadyOnDisk, gated: model.gated, hasToken: hfToken.configured });
                  return (
                    <article className="machine-panel catalog-model" key={model.id}>
                      <div className="catalog-model-head">
                        <div>
                          <button className="catalog-repo" onClick={() => openExternal(`https://huggingface.co/${model.repo}`)}>{model.repo}<ExternalLink size={12} /></button>
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
                        <div className="catalog-filename"><PathText value={file.filename} label="Model file" />{file.userSourced && <span className="state-tag">USER FILE · LOCAL DIGEST</span>}<small>{bytesLabel(file.sizeBytes)} · 4 PARALLEL RANGES</small></div>
                        {running ? (
                          <button className="button danger" onClick={() => cancelCatalogDownload(activeJob!)}><CircleStop size={15} /> Keep & stop</button>
                        ) : (
                          <button className={alreadyOnDisk ? "button is-current" : "button secondary"} disabled={!readiness.canStart} title={readiness.reason} onClick={() => startCatalogDownload(model, file)}>
                            {alreadyOnDisk ? <><BadgeCheck size={15} /> Verify file</> : <><Download size={15} /> {progress?.state === "error" ? "Resume" : "Download"}</>}
                          </button>
                        )}
                      </div>
                      {catalogFitEnabled && (
                        <p className="catalog-fit-note">
                          {buildFit.selectedPasses === false
                            ? `SIZE CHECK FAILS FOR THIS BUILD · ${bytesLabel(file.sizeBytes)} > ${bytesLabel(buildFit.thresholdBytes ?? 0)} threshold on ${catalogFitBudget?.source ?? "unknown"} budget${buildFit.rowKept ? ". The model stays listed because a smaller build fits; runtime memory is not measured." : ""}`
                            : buildFit.selectedPasses === true
                              ? `SIZE CHECK PASSES · ${bytesLabel(file.sizeBytes)} ≤ ${bytesLabel(buildFit.thresholdBytes ?? 0)} on ${catalogFitBudget?.source ?? "unknown"} budget · size only, runtime memory not measured`
                              : "SIZE CHECK UNKNOWN · no measured memory budget; open the build to check allocation"}
                        </p>
                      )}
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
                  <button className="text-link" onClick={() => openExternal("https://huggingface.co/settings/tokens")}>Create a read token on Hugging Face ↗</button>
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
          <RuntimeScreen
            activateRuntime={activateRuntime}
            busy={busy}
            cancelHealthRepair={cancelHealthRepair}
            cancelManagedHealth={cancelManagedHealth}
            cancelRuntimeInstall={cancelRuntimeInstall}
            chooseExistingRuntime={chooseExistingRuntime}
            hardware={hardware}
            healthCancelling={healthCancelling}
            healthModelProgress={healthModelProgress}
            healthRepairNotice={healthRepairNotice}
            healthRepairing={healthRepairing}
            healthResult={healthResult}
            healthRunning={healthRunning}
            inspect={inspect}
            installRuntime={installRuntime}
            installing={installing}
            isManagedPath={isManagedPath}
            loadRuntimeSetup={loadRuntimeSetup}
            managedRuntimes={managedRuntimes}
            modelRoot={modelRoot}
            openExternal={openExternal}
            repairHealthModel={repairHealthModel}
            runManagedHealth={runManagedHealth}
            runtime={runtime}
            runtimeCatalog={runtimeCatalog}
            runtimeCatalogLoading={runtimeCatalogLoading}
            runtimeCatalogState={runtimeCatalogState}
            runtimeIdentity={runtimeIdentity}
            runtimeInstallCancelling={runtimeInstallCancelling}
            runtimeInstallProgress={runtimeInstallProgress}
            runtimePath={runtimePath}
            runtimeRoot={runtimeRoot}
            selectRuntimeAdapter={selectRuntimeAdapter}
            selectedRuntimeAdapterId={selectedRuntimeAdapterId}
            setRuntime={setRuntime}
            setRuntimePath={setRuntimePath}
          />
        )}
        {view === "profile" && !profile && (
          <section className="screen profile-screen">
            <div className="section-heading">
              <div>
                <h1>Launch profile</h1>
                <p>Every field becomes an explicit llama-server argument.</p>
              </div>
            </div>
            <div className="empty-state" role="status">
              <h2>{models.length === 0 ? "No models to profile yet" : "No model selected"}</h2>
              <p>
                {models.length === 0
                  ? "A launch profile is built from a scanned model. Choose a folder of GGUF files first — the Inventory screen walks through it."
                  : "Choose a model in the Inventory screen; its profile, runtime check and Start action appear here."}
              </p>
              <div className="empty-state-actions">
                <button
                  className="button primary"
                  onClick={() => setView("models")}
                >
                  <FolderOpen size={15} /> Open Inventory
                </button>
              </div>
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
        )}

        {view === "tune" && (
          <TuneScreen
            adoptTunedProfile={adoptTunedProfile}
            bestLive={bestLive}
            briefDisclosure={briefDisclosure}
            busy={busy}
            canTune={canTune}
            cancelTuning={cancelTuning}
            changeDisclosure={changeDisclosure}
            chooseCloudModel={chooseCloudModel}
            cloudCheck={cloudCheck}
            cloudModel={cloudModel}
            cloudModels={cloudModels}
            credential={credential}
            disclosureSections={disclosureSections}
            forgetKey={forgetKey}
            gguf={gguf}
            hardware={hardware}
            keyDraft={keyDraft}
            onProviderTabKey={onProviderTabKey}
            openExternal={openExternal}
            openRouterLogin={openRouterLogin}
            probeCloud={probeCloud}
            provider={provider}
            providerId={providerId}
            providers={providers}
            runtime={runtime}
            runtimeIdentity={runtimeIdentity}
            runtimePath={runtimePath}
            saveKey={saveKey}
            selected={selected}
            setKeyDraft={setKeyDraft}
            setTuneContext={setTuneContext}
            setTuneRepeats={setTuneRepeats}
            setTuneTokens={setTuneTokens}
            setTuneTrials={setTuneTrials}
            startTuning={startTuning}
            status={status}
            switchProvider={switchProvider}
            trialsForDisplay={trialsForDisplay}
            tuneBlocker={tuneBlocker}
            tuneContext={tuneContext}
            tuneLogRef={tuneLogRef}
            tuneProgress={tuneProgress}
            tuneRepeats={tuneRepeats}
            tuneReport={tuneReport}
            tuneReportOrigin={tuneReportOrigin}
            tuneRun={tuneRun}
            tuneTokens={tuneTokens}
            tuneTrials={tuneTrials}
            tuning={tuning}
          />
        )}
        {view === "about" && (
          <AboutScreen
            about={about}
            models={models}
            totalBytes={totalBytes}
            runtime={runtime}
            runtimeIdentity={runtimeIdentity}
            hardware={hardware}
            modelRoot={modelRoot}
            onOpenExternal={(url) => void openExternal(url)}
          />
        )}

        {/* FE-05: the evidence panel stays mounted for the app's lifetime so
            navigation cannot erase active runs or completed evidence. */}
        <section
          className="screen benchmark-screen"
          style={view === "benchmark" ? undefined : { display: "none" }}
        >
          <BenchmarkScreen
            benchmark={benchmark}
            busy={busy}
            evidenceRun={evidenceRun}
            hardware={hardware}
            profile={profile}
            repeats={repeats}
            runBenchmark={runBenchmark}
            selected={selected}
            setEvidenceRun={setEvidenceRun}
            setRepeats={setRepeats}
            setTokens={setTokens}
            status={status}
            tokens={tokens}
          />
          </section>
      </main>
    </div>
  );
}

export default App;
