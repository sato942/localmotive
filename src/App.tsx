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
} from "lucide-react";
import "./App.css";
import {
  bytesLabel,
  catalogRevision,
  conflictingCapacityMetrics,
  contextChoices,
  describeChanges,
  downloadKey,
  downloadPercent,
  downloadReadiness,
  etaLabel,
  keepLatestRequest,
  normalizeProfile,
  originLabel,
  rateLabel,
  retainOrDisposeListener,
  runtimeOptionState,
  suggestedProfile,
  type BenchmarkSummary,
  type CloudModel,
  type CloudProvider,
  type CredentialStatus,
  type GgufSummary,
  type HardwareInfo,
  type InstalledRuntime,
  type LaunchProfile,
  type LogicalModel,
  type ManagedRuntimeRecord,
  type RuntimeCatalog,
  type RuntimeCapabilities,
  type RuntimeIdentity,
  type RuntimeOption,
  type ServerStatus,
  type TuningProgress,
  type TuningReport,
  type TuningTrial,
  type AboutInfo,
  type CatalogFile,
  type CatalogModel,
  type CatalogQuery,
  type CatalogSnapshot,
  type DownloadEvent,
  type TokenStatus,
} from "./model";
import { V03EvidencePanel } from "./V03EvidencePanel";

type View = "dashboard" | "models" | "catalog" | "runtime" | "profile" | "tune" | "benchmark" | "about";

const MODEL_ROOT = localStorage.getItem("gguf-pilot:model-root") ?? "";
const RUNTIME = localStorage.getItem("gguf-pilot:runtime") ?? "";
const CLOUD_PROVIDER = localStorage.getItem("gguf-pilot:cloud-provider") ?? "openrouter";
const CLOUD_MODEL = localStorage.getItem("gguf-pilot:cloud-model") ?? "";
const inTauri = () => Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);
const idleStatus: ServerStatus = {
  running: false,
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

const previewModels: LogicalModel[] = [
  {
    id: "example-chat",
    name: "Example-8B-Instruct-Q4_K_M",
    directory: "C:\\Models\\Example-8B-Instruct",
    firstShard: "C:\\Models\\Example-8B-Instruct\\Example-8B-Instruct-Q4_K_M.gguf",
    sizeBytes: 5_120_000_000,
    shardCount: 1,
    expectedShards: 1,
    complete: true,
    quant: "Q4_K_M",
    shards: [],
    companions: [],
  },
  {
    id: "example-speculative",
    name: "Example-MoE-Q4_K_M",
    directory: "D:\\AI\\GGUF\\Example-MoE",
    firstShard: "D:\\AI\\GGUF\\Example-MoE\\Example-MoE-Q4_K_M-00001-of-00002.gguf",
    sizeBytes: 96_000_000_000,
    shardCount: 2,
    expectedShards: 2,
    complete: true,
    quant: "Q4_K_M",
    shards: [],
    companions: [
      {
        name: "Example-MoE-DSpark-F16.gguf",
        path: "D:\\AI\\GGUF\\Example-MoE\\draft\\Example-MoE-DSpark-F16.gguf",
        role: "dspark",
        sizeBytes: 1_200_000_000,
      },
    ],
  },
];

function App() {
  const [view, setView] = useState<View>(RUNTIME ? "dashboard" : "runtime");
  const [modelRoot, setModelRoot] = useState(MODEL_ROOT);
  const [runtimePath, setRuntimePath] = useState(RUNTIME);
  const [hardware, setHardware] = useState<HardwareInfo | null>(null);
  const [runtimeCatalog, setRuntimeCatalog] = useState<RuntimeCatalog | null>(null);
  const [runtimeRoot, setRuntimeRoot] = useState("");
  const [installing, setInstalling] = useState("");
  const [models, setModels] = useState<LogicalModel[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [profile, setProfile] = useState<LaunchProfile | null>(null);
  const [runtime, setRuntime] = useState<RuntimeCapabilities | null>(null);
  const [status, setStatus] = useState<ServerStatus>(idleStatus);
  const [command, setCommand] = useState("");
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
  const [credential, setCredential] = useState<CredentialStatus | null>(null);
  const [keyDraft, setKeyDraft] = useState("");
  const [cloudModels, setCloudModels] = useState<CloudModel[]>([]);
  const [cloudModel, setCloudModel] = useState(CLOUD_MODEL);
  const [cloudCheck, setCloudCheck] = useState("");
  const [gguf, setGguf] = useState<GgufSummary | null>(null);
  const [about, setAbout] = useState<AboutInfo | null>(null);
  const [catalogSnapshot, setCatalogSnapshot] = useState<CatalogSnapshot | null>(null);
  const [catalogRows, setCatalogRows] = useState<CatalogModel[]>([]);
  const [catalogTags, setCatalogTags] = useState<string[]>([]);
  const [catalogQuants, setCatalogQuants] = useState<string[]>([]);
  const [catalogSearch, setCatalogSearch] = useState("");
  const [catalogTag, setCatalogTag] = useState("");
  const [catalogQuant, setCatalogQuant] = useState("");
  const [catalogMaxGiB, setCatalogMaxGiB] = useState(0);
  const [catalogHideGated, setCatalogHideGated] = useState(false);
  const [catalogSort, setCatalogSort] = useState<CatalogQuery["sort"]>("downloads");
  const [catalogBusy, setCatalogBusy] = useState(false);
  const [hfToken, setHfToken] = useState<TokenStatus>({ configured: false, masked: "" });
  const [hfTokenDraft, setHfTokenDraft] = useState("");
  const [catalogFiles, setCatalogFiles] = useState<Record<string, string>>({});
  const [downloads, setDownloads] = useState<Record<string, DownloadEvent>>({});
  const catalogFilterSeq = useRef(0);
  const catalogLoadSeq = useRef(0);
  const [tuneContext, setTuneContext] = useState(8192);
  const [tuneTrials, setTuneTrials] = useState(6);
  const [tuneTokens, setTuneTokens] = useState(256);
  const [tuneRepeats, setTuneRepeats] = useState(2);
  const [tuning, setTuning] = useState(false);
  const [tuneProgress, setTuneProgress] = useState<TuningProgress | null>(null);
  const [tuneLive, setTuneLive] = useState<TuningTrial[]>([]);
  const [tuneReport, setTuneReport] = useState<TuningReport | null>(null);
  const tuneLogRef = useRef<HTMLDivElement | null>(null);

  const selected = models.find((model) => model.id === selectedId) ?? models[0];
  const totalBytes = useMemo(() => models.reduce((sum, model) => sum + model.sizeBytes, 0), [models]);
  const invalidCount = models.filter((model) => !model.complete).length;
  const provider = providers.find((entry) => entry.id === providerId) ?? null;
  const isManagedPath = Boolean(runtimeRoot && runtimePath.startsWith(runtimeRoot));

  async function scan() {
    if (!modelRoot.trim()) {
      setNotice("Choose the folder that contains your GGUF models.");
      return;
    }
    setBusy("scan");
    try {
      const result = await invoke<LogicalModel[]>("scan_models", { root: modelRoot });
      setModels(result);
      setSelectedId(result[0]?.id ?? "");
      localStorage.setItem("gguf-pilot:model-root", modelRoot);
      setNotice(`${result.length} logical targets indexed from ${modelRoot}`);
    } catch (error) {
      if (!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__) {
        setModels(previewModels);
        setSelectedId(previewModels[0].id);
        setNotice("Browser preview: showing representative local inventory.");
      } else {
        setNotice(String(error));
      }
    } finally {
      setBusy("");
    }
  }

  async function inspect(pathOverride?: string) {
    const path = (pathOverride ?? runtimePath).trim();
    if (!path) {
      setNotice("Install a managed runtime or choose an existing llama-server.exe.");
      return;
    }
    setBusy("runtime");
    try {
      const caps = await invoke<RuntimeCapabilities>("inspect_runtime", { path });
      setRuntime(caps);
      setRuntimeIdentity(await invoke<RuntimeIdentity>("describe_runtime", { path }));
      setManagedRuntimes(await invoke<ManagedRuntimeRecord[]>("list_managed_runtimes"));
      localStorage.setItem("gguf-pilot:runtime", path);
      setNotice(`Runtime build ${caps.build} inspected; ${caps.specTypes.length} speculation modes exposed.`);
    } catch (error) {
      if (!inTauri()) {
        setRuntime({
          path,
          version: "browser preview",
          build: "10679",
          commit: "preview",
          helpSha256: "0".repeat(64),
          specTypes: ["none", "draft-simple", "draft-eagle3", "draft-mtp", "draft-dflash", "draft-dspark", "ngram-simple", "ngram-map-k", "ngram-map-k4v", "ngram-mod", "ngram-cache"],
          supportedFlags: ["--lazy-mode"],
          metrics: true,
          multimodal: true,
          fit: true,
        });
        setRuntimeIdentity({ path, backend: "cuda", cudaMajor: 13, tag: null, installKey: null, source: "dlls" });
        setNotice("Browser preview: capabilities represented from the inspected local runtime.");
      } else {
        setNotice(String(error));
      }
    } finally {
      setBusy("");
    }
  }

  /** Point the app at a different executable and inspect it in one step. */
  async function activateRuntime(path: string) {
    setRuntimePath(path);
    localStorage.setItem("gguf-pilot:runtime", path);
    if (profile) setProfile({ ...profile, runtime: path });
    await inspect(path);
  }

  async function loadRuntimeSetup() {
    setBusy("catalog");
    try {
      const detected = await invoke<HardwareInfo>("detect_hardware");
      setHardware(detected);
      const root = await invoke<string>("managed_runtime_root");
      setRuntimeRoot(root);
      const catalog = await invoke<RuntimeCatalog>("fetch_runtime_catalog");
      setRuntimeCatalog(catalog);
      setManagedRuntimes(await invoke<ManagedRuntimeRecord[]>("list_managed_runtimes"));
      if (!runtimePath) setNotice("Hardware detected. Install the recommended llama.cpp runtime to continue.");
    } catch (error) {
      if (!inTauri()) {
        const sampleAsset = (name: string): RuntimeOption["asset"] => ({
          name,
          browserDownloadUrl: "https://github.com/ggml-org/llama.cpp/releases",
          size: 128 * 1024 * 1024,
          digest: null,
        });
        setHardware({
          architecture: "x64",
          gpuNames: ["Detected Windows GPU"],
          vendor: "nvidia",
          cudaMajor: 13,
          driverVersion: "browser preview",
          detectionStatus: "Browser preview — real app reads GPU and driver locally",
          recommendation: "CUDA 13 is the best match for this NVIDIA driver",
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
        setRuntimeCatalog({
          tag: "b10752",
          publishedAt: "",
          options: [
            {
              id: "latest:cuda",
              label: "NVIDIA CUDA 13",
              backend: "cuda",
              installKey: "cuda-13.3",
              description: "Fastest path for supported NVIDIA GPUs",
              compatibility: "Includes the matching CUDA runtime DLL archive",
              asset: sampleAsset("llama-latest-bin-win-cuda-13.3-x64.zip"),
              companionAsset: sampleAsset("cudart-llama-bin-win-cuda-13.3-x64.zip"),
              recommended: true,
            },
            {
              id: "latest:vulkan",
              label: "Vulkan",
              backend: "vulkan",
              installKey: "vulkan",
              description: "Broad Windows GPU compatibility",
              compatibility: "Works across current AMD, Intel, and NVIDIA drivers",
              asset: sampleAsset("llama-latest-bin-win-vulkan-x64.zip"),
              companionAsset: null,
              recommended: false,
            },
            {
              id: "latest:cpu",
              label: "CPU",
              backend: "cpu",
              installKey: "cpu",
              description: "Portable CPU-only Windows build",
              compatibility: "No supported GPU is required",
              asset: sampleAsset("llama-latest-bin-win-cpu-x64.zip"),
              companionAsset: null,
              recommended: false,
            },
          ],
        });
        setRuntimeRoot("%LOCALAPPDATA%\\GGUF Pilot\\runtimes");
        setNotice("Browser preview: the packaged app detects local hardware and reads live GitHub releases.");
      } else {
        setNotice(String(error));
      }
    } finally {
      setBusy("");
    }
  }

  async function installRuntime(option: RuntimeOption) {
    if (!runtimeCatalog) return;
    setInstalling(option.id);
    setNotice(`Downloading ${option.label} from the official llama.cpp release…`);
    try {
      const installed = await invoke<InstalledRuntime>("install_managed_runtime", {
        tag: runtimeCatalog.tag,
        option,
      });
      await activateRuntime(installed.runtimePath);
      setNotice(`${option.label} ${installed.reused ? "was already installed and is now active" : "installed, verified, and active"}.`);
    } catch (error) {
      setNotice(String(error));
    } finally {
      setInstalling("");
    }
  }

  async function chooseModelFolder() {
    const selected = await openDialog({ directory: true, multiple: false, title: "Choose your GGUF model folder" });
    if (typeof selected === "string") {
      setModelRoot(selected);
      localStorage.setItem("gguf-pilot:model-root", selected);
    }
  }

  async function loadModelCatalog() {
    const sequence = ++catalogLoadSeq.current;
    setCatalogBusy(true);
    try {
      const snapshot = await invoke<CatalogSnapshot>("fetch_model_catalog");
      const [tags, quants] = await invoke<[string[], string[]]>("catalog_facets", {
        models: snapshot.catalog.models,
      });
      const token = await invoke<TokenStatus>("hf_token_status");
      if (!keepLatestRequest(sequence, catalogLoadSeq.current)) return;
      setCatalogSnapshot(snapshot);
      setCatalogRows(snapshot.catalog.models);
      setCatalogTags(tags);
      setCatalogQuants(quants);
      setHfToken(token);
      setNotice(`${snapshot.catalog.models.length} curated Hugging Face models loaded from ${snapshot.origin}.`);
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
    const key = downloadKey(model.repo, file.filename);
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
        revision: catalogRevision(file),
        destination: modelRoot,
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

  async function cancelCatalogDownload(file: CatalogFile) {
    await invoke<boolean>("cancel_download", { destination: modelRoot, filename: file.filename });
    setNotice(`Stopping ${file.filename}; downloaded chunks will be kept for resume.`);
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
    try {
      const list = providers.length ? providers : await invoke<CloudProvider[]>("cloud_providers");
      if (!providers.length) setProviders(list);
      const status = await invoke<CredentialStatus>("cloud_credential_status", { provider: nextProvider });
      setCredential(status);
      setCloudModels([]);
      setCloudCheck("");
      if (status.configured) {
        try {
          const modelsList = await invoke<CloudModel[]>("cloud_list_models", { provider: nextProvider });
          setCloudModels(modelsList);
          const fallback = list.find((entry) => entry.id === nextProvider)?.defaultModel ?? "";
          const stored = localStorage.getItem(`gguf-pilot:cloud-model:${nextProvider}`);
          const chosen = stored && modelsList.some((m) => m.id === stored) ? stored : modelsList.some((m) => m.id === fallback) ? fallback : (modelsList[0]?.id ?? fallback);
          setCloudModel(chosen);
        } catch (error) {
          setCloudCheck(String(error));
        }
      }
    } catch (error) {
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
    localStorage.setItem("gguf-pilot:cloud-provider", next);
    setKeyDraft("");
    await loadCloud(next);
  }

  async function saveKey() {
    if (!keyDraft.trim()) return;
    setBusy("cloud");
    try {
      const status = await invoke<CredentialStatus>("cloud_save_credential", { provider: providerId, secret: keyDraft });
      setKeyDraft("");
      setCredential(status);
      setNotice(`${provider?.label ?? providerId} key stored in Windows Credential Manager.`);
      await loadCloud(providerId);
    } catch (error) {
      setNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function forgetKey() {
    setBusy("cloud");
    try {
      setCredential(await invoke<CredentialStatus>("cloud_clear_credential", { provider: providerId }));
      setCloudModels([]);
      setCloudCheck("");
      setNotice(`${provider?.label ?? providerId} key removed from Windows Credential Manager.`);
    } catch (error) {
      setNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function openRouterLogin() {
    setBusy("oauth");
    setNotice("Finish signing in to OpenRouter in your browser. GGUF Pilot is waiting on a local callback.");
    try {
      const status = await invoke<CredentialStatus>("cloud_openrouter_login");
      setCredential(status);
      setNotice("OpenRouter connected. A user-controlled key was issued and stored in Windows Credential Manager.");
      await loadCloud("openrouter");
    } catch (error) {
      setNotice(String(error));
    } finally {
      setBusy("");
    }
  }

  async function probeCloud() {
    setBusy("probe");
    setCloudCheck("Contacting provider…");
    try {
      const reply = await invoke<string>("cloud_probe", { provider: providerId, model: cloudModel });
      setCloudCheck(`Connected · ${cloudModel} replied “${reply.trim().slice(0, 40)}”`);
    } catch (error) {
      setCloudCheck(String(error));
    } finally {
      setBusy("");
    }
  }

  function chooseCloudModel(id: string) {
    setCloudModel(id);
    localStorage.setItem("gguf-pilot:cloud-model", id);
    localStorage.setItem(`gguf-pilot:cloud-model:${providerId}`, id);
  }

  // ---- AI tuning -------------------------------------------------------------

  async function loadGguf(model: LogicalModel | undefined) {
    if (!model) return;
    try {
      const summary = await invoke<GgufSummary>("read_gguf_summary", { path: model.firstShard });
      setGguf(summary);
      if (summary.contextLength && tuneContext > summary.contextLength) {
        const choices = contextChoices(summary.contextLength);
        setTuneContext(choices[choices.length - 1] ?? 2048);
      }
    } catch {
      setGguf(null);
    }
  }

  async function startTuning() {
    if (!profile || !selected) return;
    setTuning(true);
    setTuneReport(null);
    setTuneLive([]);
    setTuneProgress({ phase: "prepare", message: "Preparing…", trial: null });
    setNotice(`AI tuning started: ${tuneTrials} trials at ${tuneContext.toLocaleString()} context via ${provider?.label ?? providerId}.`);
    try {
      const report = await invoke<TuningReport>("start_tuning", {
        request: {
          profile,
          provider: providerId,
          model: cloudModel,
          targetContext: tuneContext,
          maxTrials: tuneTrials,
          tokens: tuneTokens,
          repeats: tuneRepeats,
          companions: selected.companions.map((c) => `${c.role}: ${c.path}`),
        },
      });
      setTuneReport(report);
      localStorage.setItem(`gguf-pilot:tuning:${selected.id}`, JSON.stringify(report));
      const gain = report.baselineTps && report.bestTps ? ((report.bestTps / report.baselineTps - 1) * 100).toFixed(1) : null;
      setNotice(gain ? `Tuning finished: best ${report.bestTps?.toFixed(2)} tok/s (${Number(gain) >= 0 ? "+" : ""}${gain}% vs baseline). ${report.stoppedReason}.` : `Tuning finished. ${report.stoppedReason}.`);
    } catch (error) {
      setNotice(String(error));
      setTuneProgress({ phase: "error", message: String(error), trial: null });
    } finally {
      setTuning(false);
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
    if (!tuneReport || !selected) return;
    const adopted = { ...tuneReport.bestProfile, name: `${selected.name} / AI-tuned @${tuneContext.toLocaleString()}` };
    setProfile(adopted);
    localStorage.setItem(`gguf-pilot:profile:${selected.id}`, JSON.stringify(adopted));
    setNotice(`Adopted the best configuration as the saved profile for ${selected.name}.`);
    setView("profile");
  }

  function loadProfile(model: LogicalModel) {
    const stored = localStorage.getItem(`gguf-pilot:profile:${model.id}`);
    const next = stored
      ? normalizeProfile(JSON.parse(stored), model, runtimePath)
      : suggestedProfile(model, runtimePath);
    setProfile(next);
    setSelectedId(model.id);
    setView("profile");
    // Only move an unsaved candidate off a currently busy port. Launch remains authoritative.
    if (!stored) void suggestPortCandidate(next);
  }

  /** Suggest a currently unused port without claiming that the released socket stays available. */
  async function suggestPortCandidate(candidate: LaunchProfile) {
    try {
      const free = await invoke<number>("suggest_port", { host: candidate.host, preferred: candidate.port });
      if (free !== candidate.port) {
        setProfile((current) => (current ? { ...current, port: free } : current));
        setNotice(`Port ${candidate.port} is in use. Trying ${free}; launch validation will confirm it.`);
      }
    } catch {
      // Browser preview, or no free port in range: keep the requested one and
      // let child startup and health validation report the authoritative result.
    }
  }

  function saveProfile() {
    if (!profile || !selected) return;
    localStorage.setItem(`gguf-pilot:profile:${selected.id}`, JSON.stringify(profile));
    setNotice(`Saved ${profile.name}`);
  }

  async function preview() {
    if (!profile) return;
    try {
      setCommand(await invoke<string>("preview_command", { profile }));
    } catch (error) {
      if (!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__) {
        const draft = profile.draftModel ? ` -md \"${profile.draftModel}\"` : "";
        setCommand(`\"${profile.runtime}\" -m \"${profile.model}\"${draft} --spec-type ${profile.specType} -c ${profile.context} -ngl ${profile.gpuLayers} --host ${profile.host} --port ${profile.port} --alias ${profile.alias}`);
      } else {
        setCommand(String(error));
      }
    }
  }

  async function start() {
    if (!profile) return;
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
      localStorage.setItem(`gguf-pilot:benchmark:${status.alias}`, JSON.stringify(result));
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
    const sequence = ++catalogFilterSeq.current;
    const query: CatalogQuery = {
      text: catalogSearch,
      tag: catalogTag,
      quant: catalogQuant,
      maxBytes: catalogMaxGiB > 0 ? catalogMaxGiB * 1024 ** 3 : 0,
      hideGated: catalogHideGated,
      sort: catalogSort,
    };
    invoke<CatalogModel[]>("filter_catalog", {
      models: catalogSnapshot.catalog.models,
      query,
    })
      .then((rows) => {
        if (keepLatestRequest(sequence, catalogFilterSeq.current)) setCatalogRows(rows);
      })
      .catch((error) => {
        if (keepLatestRequest(sequence, catalogFilterSeq.current)) setNotice(String(error));
      });
  }, [catalogSnapshot, catalogSearch, catalogTag, catalogQuant, catalogMaxGiB, catalogHideGated, catalogSort]);

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
    loadRuntimeSetup();
    loadCloud();
    invoke<AboutInfo>("about_info").then(setAbout).catch(() => {
      setAbout({ name: "GGUF Pilot", version: "0.2.4", tauriVersion: "2", identifier: "io.github.ggufpilot.app", os: "windows x86_64", runtimeRoot: "", logDir: "", repository: "https://github.com/sato942/gguf-pilot", license: "MIT" });
    });
    if (modelRoot) scan();
    if (runtimePath) inspect();
  }, []);

  useEffect(() => {
    if (selected && !profile) setProfile(suggestedProfile(selected, runtimePath));
  }, [selected, runtimePath]);

  useEffect(() => {
    loadGguf(selected);
    if (selected) {
      const stored = localStorage.getItem(`gguf-pilot:tuning:${selected.id}`);
      setTuneReport(stored ? (JSON.parse(stored) as TuningReport) : null);
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
    const timer = window.setInterval(async () => {
      try {
        const next = await invoke<ServerStatus>("server_status");
        setStatus(next);
        if (next.running) setLog(await invoke<string>("read_server_log"));
      } catch {
        // Browser preview has no Tauri bridge.
      }
    }, 2000);
    return () => window.clearInterval(timer);
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
        <div className="brand-mark" aria-label="GGUF Pilot">
          <span>GP</span>
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
        <header className="topbar">
          <div>
            <p className="product-name">GGUF PILOT</p>
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
                {status.running ? (
                  <>
                    <button className="button secondary" onClick={() => openUrl(`http://127.0.0.1:${status.port}`)}>
                      <SquareTerminal size={16} /> Open chat
                    </button>
                    <button className="button danger" onClick={stop} disabled={busy === "stop"}>
                      <CircleStop size={16} /> Stop server
                    </button>
                  </>
                ) : (
                  <button className="button primary" onClick={start} disabled={!profile || !profile.runtime || busy === "start" || !selected?.complete}>
                    <Play size={16} fill="currentColor" /> Start profile
                  </button>
                )}
              </div>
            </div>

            <div className="instrument-strip">
              <div className={status.running ? "instrument primary-readout running" : "instrument primary-readout"}>
                <span className="instrument-label">PROCESS</span>
                <strong>{status.running ? "RUNNING" : "STANDBY"}</strong>
                <small>{status.running ? `PID ${status.pid}` : "No owned child process"}</small>
              </div>
              <div className="instrument">
                <span className="instrument-label">ENDPOINT</span>
                <strong>{status.port ? `:${status.port}` : "—"}</strong>
                <small>{status.alias ?? "No alias assigned"}</small>
              </div>
              <div className="instrument">
                <span className="instrument-label">STRATEGY</span>
                <strong>{profile?.specType.replace("draft-", "").toUpperCase() ?? "NONE"}</strong>
                <small>{profile?.draftModel ? "Companion linked" : "Target only"}</small>
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
                    {selected?.complete ? "VALID" : "BLOCKED"}
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
            <div className="inventory-table" role="table" aria-label="Model inventory">
              <div className="table-row table-head" role="row">
                <span>Target</span><span>Quant</span><span>Footprint</span><span>Shards</span><span>Companions</span><span>Status</span>
              </div>
              {models.map((model) => (
                <button className={selectedId === model.id ? "table-row selected" : "table-row"} key={model.id} onClick={() => loadProfile(model)}>
                  <span className="model-cell"><strong>{model.name}</strong><small>{model.directory}</small></span>
                  <span>{model.quant}</span>
                  <span className="numeric">{bytesLabel(model.sizeBytes)}</span>
                  <span className="numeric">{model.shardCount}/{model.expectedShards}</span>
                  <span className="companion-stack">
                    {model.companions.length ? (() => {
                      const counts = new Map<string, number>();
                      model.companions.forEach((c) => counts.set(c.role, (counts.get(c.role) ?? 0) + 1));
                      return [...counts].map(([role, count]) => <i key={role}>{role.toUpperCase()}{count > 1 ? ` ×${count}` : ""}</i>);
                    })() : <small>None</small>}
                  </span>
                  <span className={model.complete ? "state-tag good" : "state-tag warning"}>
                    {model.complete ? "READY" : "INCOMPLETE"}
                  </span>
                </button>
              ))}
            </div>
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
              <label>Available under<select value={catalogMaxGiB} onChange={(event) => setCatalogMaxGiB(Number(event.target.value))}><option value={0}>Any size</option>{[2, 4, 8, 16, 32, 64].map((size) => <option key={size} value={size}>≤ {size} GiB</option>)}</select></label>
              <label>Order<select value={catalogSort} onChange={(event) => setCatalogSort(event.target.value as CatalogQuery["sort"])}><option value="downloads">Downloads</option><option value="likes">Likes</option><option value="name">Name</option><option value="size">Smallest file</option></select></label>
              <label className="toggle-line catalog-toggle"><input type="checkbox" checked={catalogHideGated} onChange={(event) => setCatalogHideGated(event.target.checked)} /> Hide gated</label>
            </div>

            <div className="catalog-layout">
              <div className="catalog-results">
                <div className="catalog-result-count">
                  <strong>{catalogRows.length}</strong>
                  <span>of {catalogSnapshot?.catalog.models.length ?? 0} curated models</span>
                  {catalogSnapshot && <small>LIST UPDATED {catalogSnapshot.catalog.updated || "UNKNOWN"}</small>}
                </div>
                {catalogBusy && !catalogSnapshot && <div className="catalog-empty machine-panel"><RefreshCw size={24} className="spin" /><strong>Fetching curated catalog</strong></div>}
                {!catalogBusy && catalogSnapshot && catalogRows.length === 0 && <div className="catalog-empty machine-panel"><Database size={24} /><strong>No curated model matches these filters</strong><span>Clear one or more filters.</span></div>}
                {catalogRows.map((model) => {
                  const filename = catalogFiles[model.id] ?? model.files[0]?.filename ?? "";
                  const file = model.files.find((entry) => entry.filename === filename) ?? model.files[0];
                  if (!file) return null;
                  const key = downloadKey(model.repo, file.filename);
                  const progress = downloads[key];
                  const running = progress?.state === "downloading" || progress?.state === "verifying";
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
                      </div>
                      {model.summary && <p className="catalog-summary">{model.summary}</p>}
                      <div className="catalog-meta">
                        <span>{model.downloads.toLocaleString()} downloads</span>
                        <span>{model.likes.toLocaleString()} likes</span>
                        {model.tags.map((tag) => <i key={tag}>{tag.toUpperCase()}</i>)}
                      </div>
                      <div className="catalog-file-row">
                        <label>Build<select value={file.filename} onChange={(event) => setCatalogFiles((current) => ({ ...current, [model.id]: event.target.value }))}>{model.files.map((entry) => <option key={entry.filename} value={entry.filename}>{entry.quant} · {bytesLabel(entry.sizeBytes)}</option>)}</select></label>
                        <div className="catalog-filename"><span>{file.filename}</span><small>{bytesLabel(file.sizeBytes)} · 4 PARALLEL RANGES</small></div>
                        {running ? (
                          <button className="button danger" onClick={() => cancelCatalogDownload(file)}><CircleStop size={15} /> Keep & stop</button>
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
                  <button className="text-link" onClick={() => openUrl("https://huggingface.co/settings/tokens")}>Create a read token on Hugging Face ↗</button>
                </div>
                <dl className="runtime-facts catalog-transfer-facts">
                  <div><dt>Data route</dt><dd>Hugging Face → this PC. GGUF Pilot never proxies model bytes.</dd></div>
                  <div><dt>Resume</dt><dd>Per-chunk progress survives interruption in .part metadata.</dd></div>
                  <div><dt>Parallelism</dt><dd>Four ranged HTTPS connections; one when the CDN does not support ranges.</dd></div>
                  <div><dt>Integrity</dt><dd>Final size always checked; SHA-256 verified when Hugging Face publishes it as the object ETag.</dd></div>
                  <div><dt>Secret storage</dt><dd>Windows Credential Manager service “GGUF Pilot HF”; never local storage or logs.</dd></div>
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
              <button className="button secondary" onClick={loadRuntimeSetup} disabled={busy === "catalog"}>
                <RefreshCw size={16} className={busy === "catalog" ? "spin" : ""} /> Check latest release
              </button>
            </div>

            <div className="setup-steps" aria-label="First-run setup">
              <div className={runtimePath ? "setup-step done" : "setup-step active"}><span>1</span><strong>Runtime</strong><small>{runtimePath ? "Configured" : "Install or choose"}</small></div>
              <div className={modelRoot ? "setup-step done" : "setup-step"}><span>2</span><strong>Models</strong><small>{modelRoot ? "Folder selected" : "Choose in Inventory"}</small></div>
              <div className={runtimePath && modelRoot ? "setup-step done" : "setup-step"}><span>3</span><strong>Serve</strong><small>{runtimePath && modelRoot ? "Ready" : "Complete setup"}</small></div>
            </div>

            <div className="hardware-panel">
              <div className="hardware-icon"><MonitorCog size={26} /></div>
              <div>
                <span className="instrument-label">DETECTED WINDOWS HARDWARE</span>
                <strong>{hardware?.gpuNames.length ? hardware.gpuNames.join(" · ") : "CPU / GPU detection pending"}</strong>
                <p>{hardware?.recommendation ?? "Checking architecture and graphics drivers…"}</p>
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

            <div className="runtime-layout">
              <div className="runtime-main">
                <div className="runtime-section-title">
                  <div><h2>Official Windows builds</h2><p>{runtimeCatalog ? `Latest binary release ${runtimeCatalog.tag}${runtimeCatalog.publishedAt ? ` · ${runtimeCatalog.publishedAt.slice(0, 10)}` : ""}` : "Loading releases from ggml-org/llama.cpp"}</p></div>
                  <button className="text-button runtime-source" onClick={() => openUrl("https://github.com/ggml-org/llama.cpp/releases")}><Download size={14} /> GitHub releases</button>
                </div>
                <div className="runtime-options">
                  {runtimeCatalog?.options.map((option) => {
                    const state = runtimeOptionState(option, runtimeCatalog.tag, managedRuntimes, runtimeIdentity, runtime?.build ?? null);
                    const downloading = installing === option.id;
                    return (
                    <article className={`runtime-option${option.recommended ? " recommended" : ""}${state.kind === "active" ? " is-active" : ""}`} key={`${option.id}:${option.asset.name}`}>
                      <div className="runtime-option-main">
                        <div className="runtime-option-title"><strong>{option.label}</strong>{state.kind === "active" && <span className="state-tag good"><BadgeCheck size={10} /> ACTIVE · b{runtime?.build}</span>}{state.kind === "update" && <span className="state-tag warning"><ArrowUp size={10} /> UPDATE FROM {state.from.toUpperCase()}</span>}{state.kind !== "active" && state.kind !== "update" && option.recommended && <span className="state-tag good">RECOMMENDED</span>}</div>
                        <span className="runtime-role">{state.kind === "active" ? (isManagedPath ? "This is the runtime in use" : "Your existing executable matches this package") : state.kind === "update" ? `Newer official build available for the runtime in use` : state.kind === "use" ? "Already downloaded · not the active runtime" : option.recommended ? "Recommended for this PC" : option.backend === "cpu" ? "CPU fallback" : option.backend === "vulkan" ? "Compatibility fallback" : option.backend === "cuda" ? "Alternative CUDA package" : "Optional backend"}</span>
                        <p>{option.description}</p>
                        <small>{option.compatibility}</small>
                        <code>{option.asset.name}</code>
                        <small className="runtime-provenance"><ShieldCheck size={12} /> Official ggml-org asset · {runtimeCatalog.tag} · {option.asset.digest ? "digest available" : "size metadata available"}</small>
                        <details className="asset-proof"><summary>Artifact verification</summary><code>{option.asset.digest ?? "No digest published for this asset"}</code><span>GGUF Pilot verifies published size and SHA-256 before extraction. GitHub release metadata does not provide a Windows code-signing result.</span></details>
                      </div>
                      <div className="runtime-option-action">
                        <span>{bytesLabel(option.asset.size + (option.companionAsset?.size ?? 0))}</span>
                        {state.kind === "active" && (
                          <button className="button secondary is-current" disabled aria-disabled="true"><BadgeCheck size={15} /> Up to date</button>
                        )}
                        {state.kind === "update" && (
                          <button className="button primary" onClick={() => installRuntime(option)} disabled={Boolean(installing)}>
                            <ArrowUp size={15} /> {downloading ? "Downloading…" : `Update to ${runtimeCatalog.tag}`}
                          </button>
                        )}
                        {state.kind === "use" && (
                          <button className="button secondary" onClick={() => activateRuntime(state.runtimePath)} disabled={Boolean(installing) || busy === "runtime"}>
                            <Play size={15} /> Use this build
                          </button>
                        )}
                        {state.kind === "install" && (
                          <button className={option.recommended ? "button primary" : "button secondary"} onClick={() => installRuntime(option)} disabled={Boolean(installing)}>
                            <Download size={15} /> {downloading ? "Downloading…" : "Install"}
                          </button>
                        )}
                      </div>
                    </article>
                    );
                  }) ?? <div className="runtime-loading"><RefreshCw size={22} className="spin" /><span>Reading official release assets…</span></div>}
                </div>
              </div>

              <aside className="runtime-sidebar">
                <div className="panel-title"><ShieldCheck size={17} /><h2>Managed runtime</h2></div>
                <div className="managed-status">
                  <span className={runtime ? "plate-light ok" : "plate-light"} />
                  <div><strong>{runtime ? `llama.cpp b${runtime.build}` : "Not configured"}</strong>{runtime && <span className={isManagedPath ? "runtime-trust managed" : "runtime-trust supplied"}>{isManagedPath ? "MANAGED · VERIFIED DOWNLOAD" : "USER-SUPPLIED · VERSION INSPECTED"}</span>}{runtimeIdentity && runtimeIdentity.backend !== "unknown" && <span className="runtime-trust neutral">{runtimeIdentity.backend.toUpperCase()}{runtimeIdentity.cudaMajor ? ` ${runtimeIdentity.cudaMajor}` : ""} · {runtimeIdentity.source === "manifest" ? "FROM MANIFEST" : "FROM DLLS"}</span>}<small>{runtimePath || "Install a recommended build or choose an existing executable."}</small></div>
                </div>
                <label>Existing llama-server.exe<input value={runtimePath} onChange={(event) => setRuntimePath(event.target.value)} placeholder="Path to llama-server.exe" /></label>
                <div className="runtime-side-actions">
                  <button className="button secondary" onClick={chooseExistingRuntime}><FolderOpen size={15} /> Browse</button>
                  <button className="button secondary" onClick={() => inspect()} disabled={!runtimePath}><Cpu size={15} /> Inspect</button>
                </div>
                {managedRuntimes.length > 0 && (
                  <dl className="runtime-facts managed-list">
                    <div><dt>Installed managed builds</dt><dd>{managedRuntimes.map((record) => <button key={record.runtimePath} className={`managed-entry${record.runtimePath === runtimePath ? " current" : ""}`} onClick={() => activateRuntime(record.runtimePath)} disabled={record.runtimePath === runtimePath}>{record.tag} · {record.installKey}{record.runtimePath === runtimePath ? " · active" : ""}</button>)}</dd></div>
                  </dl>
                )}
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
                <button className="button primary" onClick={start} disabled={!selected?.complete || !profile.runtime || busy === "start"}><Play size={16} fill="currentColor" /> Start</button>
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
                  <label className="wide">llama-server executable<input value={profile.runtime} onChange={(e) => { setRuntimePath(e.target.value); setProfile({ ...profile, runtime: e.target.value }); }} /></label>
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
                        <label>N-gram draft min / max<div className="paired-inputs"><input type="number" value={profile.ngramMin} onChange={(e) => setProfile({ ...profile, ngramMin: Number(e.target.value) })} /><input type="number" value={profile.ngramMax} onChange={(e) => setProfile({ ...profile, ngramMax: Number(e.target.value) })} /></div></label>
                        <label>Map lookup / draft size<div className="paired-inputs"><input type="number" value={profile.ngramSizeN} onChange={(e) => setProfile({ ...profile, ngramSizeN: Number(e.target.value) })} /><input type="number" value={profile.ngramSizeM} onChange={(e) => setProfile({ ...profile, ngramSizeM: Number(e.target.value) })} /></div></label>
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
                          <input value={profile.extraArgs.join(" ")} onChange={(e) => setProfile({ ...profile, extraArgs: e.target.value.trim() ? e.target.value.trim().split(/\s+/) : [] })} />
                          <small className="field-help">Use self-contained `--flag` or `--flag=value` tokens. Typed-field overrides and privileged capabilities are rejected.</small>
                        </label>
                      </div>
                    </details>
                  </div>
                </details>
              </div>

              <aside className="command-preview">
                <div className="panel-title"><Braces size={17} /><h2>Exact command</h2></div>
                <pre>{command || "Edit a setting to build the command."}</pre>
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
                    {providers.map((entry) => (
                      <button key={entry.id} role="tab" aria-selected={entry.id === providerId} className={entry.id === providerId ? "provider-tab active" : "provider-tab"} onClick={() => switchProvider(entry.id)}>{entry.label}</button>
                    ))}
                  </div>
                  <div className="provider-body">
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
                      <small className="field-help">Stored under “GGUF Pilot” in Windows Credential Manager, not in this app’s settings. {provider && <button className="text-link" onClick={() => openUrl(provider.consoleUrl)}>Get a key from {provider.label} →</button>}</small>
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
                  <div className="panel-title"><Trophy size={17} /><h2>Result</h2>{tuneReport && <span className="state-tag good">FINISHED</span>}{tuning && <span className="state-tag warning">RUNNING</span>}</div>
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
                      {tuneReport && <div className="tune-actions"><button className="button primary" onClick={adoptTunedProfile}><Save size={15} /> Adopt best as profile</button><small>{tuneReport.stoppedReason}</small></div>}
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
                <button className="button secondary" onClick={() => openUrl(about?.repository ?? "https://github.com/sato942/gguf-pilot")}><Link2 size={15} /> Project on GitHub</button>
              </div>
            </div>

            <div className="about-layout">
              <article className="machine-panel about-identity">
                <div className="about-mark">GP</div>
                <div>
                  <strong>{about?.name ?? "GGUF Pilot"}</strong>
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
                  <div><dt>Cloud keys</dt><dd>Windows Credential Manager, service “GGUF Pilot” — never in settings or logs</dd></div>
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
                <p className="group-note about-note">GGUF Pilot is not affiliated with ggml-org. Runtime binaries are downloaded directly from official llama.cpp GitHub releases and verified against their published size and SHA-256 before use.</p>
              </article>
            </div>
          </section>
        )}

        {view === "benchmark" && (
          <section className="screen benchmark-screen">
            <div className="section-heading">
              <div>
                <h1>Generation benchmark</h1>
                <p>One warmup, fixed deterministic workload, repeated server-reported throughput.</p>
              </div>
              <button className="button primary" onClick={runBenchmark} disabled={!status.running || busy === "benchmark"}>
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
            />
          </section>
        )}
      </main>
    </div>
  );
}

export default App;
