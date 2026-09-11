// @vitest-environment jsdom
//
// Denied browser storage and rejected-IPC legs (audit FE-09 I3 / QD-02.I4):
// a persistence failure must never relabel a completed native operation, the
// app must keep working when storage is unavailable, and a rejected IPC
// response must surface honestly through the production callers.
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

type Handler = (args: unknown) => unknown | Promise<unknown>;

const invokeCalls: Array<{ command: string; args: unknown }> = [];
const handlers = new Map<string, Handler>();

const idleServerStatus = {
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

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (command: string, args?: unknown) => {
    invokeCalls.push({ command, args });
    const handler = handlers.get(command);
    if (handler) {
      try {
        return Promise.resolve(handler(args));
      } catch (error) {
        return Promise.reject(error);
      }
    }
    if (command === "server_status") return Promise.resolve(idleServerStatus);
    if (command === "list_managed_runtimes") return Promise.resolve([]);
    if (command === "load_runtime_setup" || command === "detect_hardware") {
      return Promise.resolve(runtimeSetup());
    }
    if (command === "cloud_providers") return Promise.resolve([]);
    if (command === "cloud_credential_status") return Promise.resolve({ provider: "openrouter", configured: false, masked: "" });
    if (command === "hf_token_status") return Promise.resolve({ configured: false });
    if (command === "catalog_facets") return Promise.resolve([[], []]);
    if (command === "catalog_rich_facets") {
      return Promise.resolve({ authors: [], licenses: [], pipeline_tags: [], architectures: [] });
    }
    if (command === "catalog_fit_budget") return Promise.resolve({ budgetBytes: 0, source: "unknown" });
    if (command === "tune_disclosure_list") return Promise.resolve([]);
    if (command === "about_info") return Promise.resolve(null);
    if (command === "scan_models_report") {
      const legacy = handlers.get("scan_models");
      if (legacy) {
        return Promise.resolve(legacy(args)).then((models) => ({
          models,
          problems: [],
          truncated: false,
        }));
      }
      return Promise.resolve({ models: [], problems: [], truncated: false });
    }
    return Promise.resolve(null);
  },
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => {}) }));
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: () => Promise.resolve(null),
  save: () => Promise.resolve(null),
}));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

import App from "./App";

function runtimeSetup(over: Record<string, unknown> = {}) {
  return {
    hardware: {
      architecture: "x86_64",
      gpuNames: [],
      vendor: "unknown",
      cudaMajor: null,
      driverVersion: "unknown",
      detectionStatus: "fixture",
      recommendation: "fixture",
      systemMemory: {
        totalPhysicalBytes: { value: null, level: "unknown", source: { kind: "unknown", detail: "fixture" }, observedAtMs: 0, notes: [] },
        availablePhysicalBytes: { value: null, level: "unknown", source: { kind: "unknown", detail: "fixture" }, observedAtMs: 0, notes: [] },
        memoryLoadPercent: { value: null, level: "unknown", source: { kind: "unknown", detail: "fixture" }, observedAtMs: 0, notes: [] },
      },
      adapters: [],
      manualOverrides: [],
    },
    catalog: null,
    catalogError: null,
    runtimeRoot: "",
    managedRuntimes: [],
    ...over,
  };
}

const scannedModel = {
  id: "fixture/model",
  name: "fixture",
  directory: "C:/models/fixture",
  firstShard: "C:/models/fixture/model-Q4_K_M.gguf",
  sizeBytes: 4_000_000_000,
  shardCount: 1,
  expectedShards: 1,
  complete: true,
  quant: "Q4_K_M",
  shards: [],
  companions: [],
};

const ggufSummary = {
  version: 3,
  architecture: "llama",
  name: "fixture",
  sizeLabel: "4 GB",
  fileType: 15,
  blockCount: 32,
  contextLength: 8192,
  embeddingLength: 4096,
  headCount: 32,
  headCountKv: 8,
  keyLength: 128,
  valueLength: 128,
  expertCount: null,
  expertUsedCount: null,
  vocabSize: 32000,
  ropeFreqBase: null,
  tensorCount: 10,
  kvCount: 20,
  metadataFacts: [],
};

const tuningReport = {
  baselineTps: 100,
  bestIndex: 0,
  bestTps: 120,
  bestProfile: {
    name: "fixture / AI-tuned",
    runtime: "C:/runtime/llama-server.exe",
    model: "C:/models/fixture/model-Q4_K_M.gguf",
    port: 8080,
    host: "127.0.0.1",
    context: 4096,
    batch: 512,
    ubatch: 128,
    parallel: 1,
    gpuLayers: "all",
    flashAttention: "auto",
    kvOffload: true,
    extraArgs: [],
  },
  trials: [],
  stoppedReason: "Trial budget reached",
  objective: "short-prompt decode throughput",
};

let container: HTMLDivElement;
let root: Root;

beforeEach(() => {
  localStorage.clear();
  (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {};
  // jsdom does not implement Element.scrollTo; the tuning log effect calls it
  // through the production code path.
  (Element.prototype as unknown as { scrollTo: (options?: unknown) => void }).scrollTo = () => {};
  container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);
  invokeCalls.length = 0;
  handlers.clear();
});

afterEach(async () => {
  await act(async () => {
    root.unmount();
  });
  container.remove();
  vi.restoreAllMocks();
});

async function mount() {
  await act(async () => {
    root.render(<App />);
  });
}

async function settle() {
  await act(async () => {
    for (let index = 0; index < 8; index += 1) {
      await Promise.resolve();
    }
  });
}

function text() {
  return container.textContent ?? "";
}

function navButton(label: string) {
  return [...container.querySelectorAll("button")].find(
    (button) => (button.textContent ?? "").trim() === label,
  );
}

async function click(target: Element | undefined, why: string) {
  expect(target, why).toBeTruthy();
  await act(async () => {
    target!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
  await settle();
}

async function setInputValue(input: HTMLInputElement, value: string) {
  await act(async () => {
    const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
    proto.set!.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
  await settle();
}

/// Drive the Runtime screen's path field, the Inventory root field and a
/// rescan so a complete model with a runtime path is selected and loaded.
async function scanFixtureModel() {
  await click(navButton("Runtime"), "Runtime navigation must exist");
  const runtimeField = [...container.querySelectorAll("input")].find(
    (input) => (input as HTMLInputElement).placeholder === "Path to llama-server.exe",
  ) as HTMLInputElement | undefined;
  expect(runtimeField, "the runtime path field must render").toBeTruthy();
  await setInputValue(runtimeField!, "C:/runtime/llama-server.exe");
  await click(navButton("Inventory"), "Inventory navigation must exist");
  const rootInput = container.querySelector('input[aria-label="Model root"]') as HTMLInputElement | null;
  expect(rootInput, "the model root field must render").toBeTruthy();
  await setInputValue(rootInput!, "C:/models/fixture-root");
  handlers.set("scan_models", () => [scannedModel]);
  await click(
    [...container.querySelectorAll("button")].find((button) =>
      (button.textContent ?? "").includes("Rescan"),
    ),
    "the Rescan action must exist",
  );
  for (let attempt = 0; attempt < 10 && !text().includes("Launch profile"); attempt += 1) {
    await settle();
  }
  expect(text(), "the scan must load the profile for the scanned model").toContain("Launch profile");
}

function denyStorage() {
  return vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
    throw new Error("storage denied by fixture");
  });
}

describe("denied browser storage (audit FE-09 I3)", () => {
  it("keeps the app usable when storage is denied from the first read", async () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("storage denied by fixture");
    });
    const setItem = denyStorage();
    await mount();
    await settle();
    // The app renders with defaults instead of crashing; navigation exists.
    expect(navButton("Control"), "the app must survive denied storage reads").toBeTruthy();
    expect(text()).toContain("Local inference control plane");
    setItem.mockRestore();
  });

  it("reports a failed profile save without losing the edit (save, not crash)", async () => {
    await mount();
    await scanFixtureModel();
    const nameField = [...container.querySelectorAll("input")].find(
      (input) => (input as HTMLInputElement).value.includes("fixture"),
    ) as HTMLInputElement | undefined;
    expect(nameField, "the profile name field must render").toBeTruthy();
    await setInputValue(nameField!, "fixture-renamed");
    const setItem = denyStorage();
    try {
      await click(
        [...container.querySelectorAll("button")].find(
          (button) => (button.textContent ?? "").trim() === "Save",
        ),
        "the Save action must exist",
      );
      expect(text()).toContain("Could not save fixture-renamed");
      expect(text()).toContain("could not be saved to browser storage");
      // The edit is still visible; the app did not navigate away or crash.
      expect((nameField!.value ?? "")).toBe("fixture-renamed");
      expect(navButton("Benchmark"), "navigation must still work").toBeTruthy();
    } finally {
      setItem.mockRestore();
    }
    // With storage restored the same action succeeds again.
    await click(
      [...container.querySelectorAll("button")].find(
        (button) => (button.textContent ?? "").trim() === "Save",
      ),
      "the Save action must exist",
    );
    expect(text()).toContain("Saved fixture-renamed");
  });

  it("keeps a completed benchmark result visible when its save fails", async () => {
    handlers.set("server_status", () => ({
      ...idleServerStatus,
      running: true,
      phase: "running",
      pid: 42,
      profileName: "fixture / Baseline",
      alias: "fixture-baseline",
      port: 8080,
      startedAt: 1,
      resultClass: "measured",
    }));
    handlers.set("benchmark_server", () => ({
      samples: [41, 42, 43],
      meanTps: 42,
      medianTps: 42,
      minTps: 41,
      maxTps: 43,
      tokens: 256,
      repeats: 3,
    }));
    // The status poll is a 2-second interval; advance it so the app observes
    // the running server before the click.
    vi.useFakeTimers();
    try {
      await mount();
      await act(async () => {
        await vi.advanceTimersByTimeAsync(2100);
      });
      await click(navButton("Benchmark"), "Benchmark navigation must exist");
      const setItem = denyStorage();
      try {
        // Scope to the benchmark screen: the dashboard keeps its own controls
        // in the DOM with this screen hidden.
        const section = container.querySelector(".benchmark-screen") as HTMLElement | null;
        expect(section, "the benchmark screen section must exist").toBeTruthy();
        await click(
          [...section!.querySelectorAll("button")].find((button) =>
            (button.textContent ?? "").includes("Run benchmark"),
          ),
          "the Run benchmark action must exist",
        );
        // The completed native operation is reported as completed; the save
        // failure is a separate sentence, not an operation failure.
        expect(text()).toContain("Benchmark complete: 42.00 generation tok/s mean.");
        expect(text()).toContain("could not be saved to browser storage");
      } finally {
        setItem.mockRestore();
      }
    } finally {
      vi.useRealTimers();
    }
  });

  it("keeps a finished tuning report visible when its save fails", async () => {
    handlers.set("cloud_credential_status", () => ({ provider: "openrouter", configured: true, masked: "sk-or-…1234" }));
    handlers.set("cloud_list_models", () => [{ id: "advisor-1", label: "Advisor One" }]);
    handlers.set("read_gguf_summary", () => ggufSummary);
    handlers.set("start_tuning", () => tuningReport);
    await mount();
    await scanFixtureModel();
    await click(navButton("AI Tune"), "the AI Tune navigation must exist");
    const autoTune = [...container.querySelectorAll("button")].find((button) =>
      (button.textContent ?? "").includes("Auto-tune"),
    );
    expect(autoTune, "the Auto-tune action must render").toBeTruthy();
    expect((autoTune as HTMLButtonElement).disabled, "readiness must enable Auto-tune")
      .toBe(false);
    const setItem = denyStorage();
    try {
      await click(autoTune, "the Auto-tune action must exist");
      // The run finished; the report is visible and the phase is not an error.
      expect(text()).toContain("Tuning finished");
      expect(text()).toContain("Report ready");
      expect(text()).toContain("could not be saved to browser storage");
      expect(text()).not.toContain("storage denied by fixture");
    } finally {
      setItem.mockRestore();
    }
  });
});
