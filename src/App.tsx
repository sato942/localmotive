import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Activity, Cpu, Database, Download, Gauge, Info, MonitorCog, Settings2, Sparkles, TestTube2 } from "lucide-react";
import "./App.css";
import {
  managedHealthOutcome,
  errorText,
  catalogRevision,
  type CommandPreview,
  type ScanReport,
  contextChoices,
  DEFAULT_FIT_PER_MILLE,
  downloadKey,
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
import { InventoryScreen } from "./screens/InventoryScreen";
import { CatalogScreen } from "./screens/CatalogScreen";
import { ProfileEmptyScreen } from "./screens/ProfileEmptyScreen";
import { ProfileScreen } from "./screens/ProfileScreen";
import { BenchmarkScreen } from "./screens/BenchmarkScreen";
import { tauriEvidenceAdapter } from "./evidence-adapter";

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
          <CatalogScreen
            cancelCatalogDownload={cancelCatalogDownload}
            catalogArchitecture={catalogArchitecture}
            catalogArchitectures={catalogArchitectures}
            catalogAuthor={catalogAuthor}
            catalogAuthors={catalogAuthors}
            catalogBusy={catalogBusy}
            catalogFiles={catalogFiles}
            catalogFitBudget={catalogFitBudget}
            catalogFitEnabled={catalogFitEnabled}
            catalogFitPerMille={catalogFitPerMille}
            catalogHideGated={catalogHideGated}
            catalogLicense={catalogLicense}
            catalogLicenses={catalogLicenses}
            catalogMaxGiB={catalogMaxGiB}
            catalogPipeline={catalogPipeline}
            catalogPipelines={catalogPipelines}
            catalogQuant={catalogQuant}
            catalogQuants={catalogQuants}
            catalogRows={catalogRows}
            catalogSearch={catalogSearch}
            catalogSnapshot={catalogSnapshot}
            catalogSort={catalogSort}
            catalogTag={catalogTag}
            catalogTags={catalogTags}
            chooseModelFolder={chooseModelFolder}
            downloadJobs={downloadJobs}
            downloads={downloads}
            hfToken={hfToken}
            hfTokenDraft={hfTokenDraft}
            inventoryHasFile={inventoryHasFile}
            loadModelCatalog={loadModelCatalog}
            modelRoot={modelRoot}
            openExternal={openExternal}
            removeHfToken={removeHfToken}
            saveHfToken={saveHfToken}
            setCatalogArchitecture={setCatalogArchitecture}
            setCatalogAuthor={setCatalogAuthor}
            setCatalogFiles={setCatalogFiles}
            setCatalogFitEnabled={setCatalogFitEnabled}
            setCatalogFitPerMille={setCatalogFitPerMille}
            setCatalogHideGated={setCatalogHideGated}
            setCatalogLicense={setCatalogLicense}
            setCatalogMaxGiB={setCatalogMaxGiB}
            setCatalogPipeline={setCatalogPipeline}
            setCatalogQuant={setCatalogQuant}
            setCatalogSearch={setCatalogSearch}
            setCatalogSort={setCatalogSort}
            setCatalogTag={setCatalogTag}
            setHfTokenDraft={setHfTokenDraft}
            setModelRoot={setModelRoot}
            startCatalogDownload={startCatalogDownload}
          />
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
          <ProfileEmptyScreen models={models} setView={setView} />
        )}
        {view === "profile" && profile && (
          <ProfileScreen
            busy={busy}
            command={command}
            evidenceRun={evidenceRun}
            extraArgsDraft={extraArgsDraft}
            inspect={inspect}
            profile={profile}
            runtime={runtime}
            saveProfile={saveProfile}
            selected={selected}
            setExtraArgsDraft={setExtraArgsDraft}
            setNotice={setNotice}
            setProfile={setProfile}
            setRuntime={setRuntime}
            setRuntimePath={setRuntimePath}
            start={start}
            status={status}
            tuning={tuning}
          />
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
            adapter={tauriEvidenceAdapter}
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
