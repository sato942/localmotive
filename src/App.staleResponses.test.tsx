// @vitest-environment jsdom
//
// Deferred-IPC interleavings through production callers (audit FE-03 V1/V3):
// a provider A response that resolves after a switch to B must not relabel B;
// a port suggestion that returns after a manual edit must not overwrite it;
// an older command preview must not replace a newer one. The IPC boundary is
// mocked with controllable deferred promises; nothing else is.
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

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

function runtimeSetup() {
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

const providers = [
  {
    id: "openrouter",
    label: "OpenRouter",
    baseUrl: "https://openrouter.ai/api/v1",
    keyPrefixHint: "sk-or-",
    consoleUrl: "https://openrouter.ai/settings/keys",
    supportsOauth: true,
    defaultModel: "advisor-alpha",
    listsModels: true,
  },
  {
    id: "anthropic",
    label: "Anthropic",
    baseUrl: "https://api.anthropic.com/v1",
    keyPrefixHint: "sk-ant-",
    consoleUrl: "https://platform.anthropic.com/settings/keys",
    supportsOauth: false,
    defaultModel: "advisor-beta",
    listsModels: false,
  },
];

let container: HTMLDivElement;
let root: Root;

beforeEach(() => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  vi.useFakeTimers();
  localStorage.clear();
  (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {};
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
  vi.useRealTimers();
  vi.unstubAllGlobals();
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

function tabButton(label: string) {
  return [...container.querySelectorAll('button[role="tab"]')].find(
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

function profileInput(labelText: string): HTMLInputElement {
  const label = [...container.querySelectorAll("label")].find(
    (candidate) => (candidate.textContent ?? "").trim().startsWith(labelText),
  );
  const input = label?.querySelector<HTMLInputElement>("input");
  if (!input) throw new Error(`input not found: ${labelText}`);
  return input;
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
  await act(async () => { await vi.advanceTimersByTimeAsync(300); });
}

describe("stale port suggestions and command previews (audit FE-03 V3)", () => {
  it("composes one preview after a typing burst and removes the obsolete command immediately", async () => {
    handlers.set("suggest_port", () => 8080);
    handlers.set("preview_command", () => ({ powerShell: "PREVIOUS-COMMAND", argv: "", cmd: null, cmdNotice: null }));
    await mount();
    await scanFixtureModel();
    expect(text()).toContain("PREVIOUS-COMMAND");
    invokeCalls.length = 0;
    for (const name of ["f", "fi", "finished"]) await setInputValue(profileInput("Profile name"), name);
    expect(profileInput("Profile name").value).toBe("finished");
    expect(text()).not.toContain("PREVIOUS-COMMAND");
    expect(invokeCalls.filter((call) => call.command === "preview_command")).toHaveLength(0);
    await act(async () => { await vi.advanceTimersByTimeAsync(300); });
    const previews = invokeCalls.filter((call) => call.command === "preview_command");
    expect(previews).toHaveLength(1);
    expect((previews[0].args as { profile: { name: string } }).profile.name).toBe("finished");
  });

  it("does not save or dispatch an out-of-range port", async () => {
    handlers.set("suggest_port", () => 8080);
    await mount();
    await scanFixtureModel();
    invokeCalls.length = 0;
    await setInputValue(profileInput("Port"), "65536");
    await act(async () => { await vi.advanceTimersByTimeAsync(300); });
    await click(navButton("Save"), "Save must remain visible");
    expect(localStorage.getItem(`localmotive:profile:${scannedModel.id}`)).toBeNull();
    expect(invokeCalls.filter((call) => call.command === "preview_command")).toHaveLength(0);
    await click(navButton("Control"), "Control navigation must exist");
    await click(navButton("Start profile"), "Control must offer Start profile");
    expect(invokeCalls.filter((call) => call.command === "start_server")).toHaveLength(0);
    expect(text()).toContain("Port");
    expect(text()).toContain("65535");
  });

  it("keeps a cleared numeric field blank instead of silently saving zero", async () => {
    handlers.set("suggest_port", () => 8080);
    await mount();
    await scanFixtureModel();
    await setInputValue(profileInput("Context tokens"), "");
    expect(profileInput("Context tokens").value).toBe("");
    await click(navButton("Save"), "Save must remain visible");
    expect(localStorage.getItem(`localmotive:profile:${scannedModel.id}`)).toBeNull();
    await setInputValue(profileInput("Context tokens"), "4096");
    await click(navButton("Save"), "a corrected profile can be saved");
    expect(JSON.parse(localStorage.getItem(`localmotive:profile:${scannedModel.id}`)!).context).toBe(4096);
  });

  it("never applies a port suggestion over a later manual edit", async () => {
    const suggestion = deferred<number>();
    handlers.set("suggest_port", () => suggestion.promise);
    await mount();
    await scanFixtureModel();
    expect(
      invokeCalls.some((call) => call.command === "suggest_port"),
      "the scan must have requested a port suggestion",
    ).toBe(true);
    // The user edits the port while the suggestion is still in flight.
    await setInputValue(profileInput("Port"), "9090");
    expect(profileInput("Port").value).toBe("9090");
    // The suggestion returns late; the manual edit must survive untouched.
    await act(async () => {
      suggestion.resolve(8081);
    });
    await settle();
    expect(profileInput("Port").value, "the manual port edit must not be overwritten").toBe("9090");
    expect(text(), "no apply notice may fire for a discarded suggestion").not.toContain("is in use. Trying");
  });

  it("discards an older command preview that resolves after a newer one", async () => {
    const previews: Array<ReturnType<typeof deferred<unknown>>> = [];
    handlers.set("suggest_port", () => 8080);
    handlers.set("preview_command", () => {
      const next = deferred<unknown>();
      previews.push(next);
      return next.promise;
    });
    await mount();
    await scanFixtureModel();
    // The profile loaded: preview A is in flight. Edit the name to dispatch
    // preview B for the newer revision.
    await setInputValue(profileInput("Profile name"), "fixture-edited");
    await act(async () => { await vi.advanceTimersByTimeAsync(300); });
    for (let attempt = 0; attempt < 10 && previews.length < 2; attempt += 1) {
      await settle();
    }
    expect(previews.length, "two preview requests must be in flight").toBe(2);
    const command = (marker: string) => ({
      powerShell: marker,
      argv: "",
      cmd: null,
      cmdNotice: null,
    });
    // The newer preview resolves first and becomes authoritative.
    await act(async () => {
      previews[1]!.resolve(command("NEWER-COMMAND"));
    });
    await settle();
    // The older preview resolves late and must be discarded.
    await act(async () => {
      previews[0]!.resolve(command("OLDER-COMMAND"));
    });
    await settle();
    expect(text(), "the newer command must stay displayed").toContain("NEWER-COMMAND");
    expect(text(), "the older command must be discarded").not.toContain("OLDER-COMMAND");
  });
});

describe("legacy benchmark numeric inputs", () => {
  beforeEach(async () => {
    handlers.set("server_status", () => ({ ...idleServerStatus, running: true, phase: "healthy", pid: 1234, port: 8080, alias: "fixture", profileName: "fixture" }));
    handlers.set("read_server_log", () => "");
    handlers.set("benchmark_server", () => { throw new Error("Fixture boundary: no inference run."); });
    await mount();
    await act(async () => { await vi.advanceTimersByTimeAsync(2000); });
    await click(navButton("Benchmark"), "Benchmark navigation must exist");
    expect(navButton("Run benchmark")?.disabled, "the server fixture must be ready before the input test").toBe(false);
    invokeCalls.length = 0;
  });

  it.each(["Forced output tokens", "Measured repeats"])("keeps a cleared %s field blank", async (label) => {
    await setInputValue(profileInput(label), "");
    expect(profileInput(label).value).toBe("");
  });

  it("keeps a legacy cancellation handle across navigation until the backend settles", async () => {
    const pending = deferred<unknown>();
    handlers.set("benchmark_server", () => pending.promise);
    handlers.set("cancel_benchmark", () => undefined);
    handlers.set("detect_hardware", () => { throw new Error("Fixture boundary: hardware unavailable"); });
    await click(navButton("Run benchmark"), "the legacy benchmark must start");
    try {
      await click(navButton("Inventory"), "navigation must remain available");
      let band = container.querySelector(".evidence-run-band");
      expect(band, "the legacy run must retain a visible cancel owner").not.toBeNull();
      expect(band?.textContent).toContain("Benchmark running");
      await click(navButton("Benchmark"), "return to the evidence panel");
      await click(navButton("Refresh hardware"), "an unrelated panel action stays available");
      band = container.querySelector(".evidence-run-band");
      expect(band, "the panel must not clear the legacy run's handle").not.toBeNull();
      const cancel = [...(band?.querySelectorAll("button") ?? [])].find((button) => button.textContent === "Cancel");
      await click(cancel, "the running legacy benchmark must expose Cancel");
      expect(invokeCalls.filter((call) => call.command === "cancel_benchmark")).toHaveLength(1);
      expect(container.querySelector(".evidence-run-band")).not.toBeNull();
      expect(text()).toContain("waiting for the current request");
    } finally {
      await act(async () => { pending.reject(new Error("The local request was cancelled")); });
      await settle();
    }
    expect(container.querySelector(".evidence-run-band")).toBeNull();
    expect(localStorage.getItem("localmotive:benchmark:fixture")).toBeNull();
  });

  it("rejects duplicate legacy dispatch before React publishes its busy state", async () => {
    const pending = deferred<unknown>();
    let requests = 0;
    handlers.set("benchmark_server", () => {
      requests += 1;
      if (requests > 1) throw new Error("A benchmark is already active");
      return pending.promise;
    });
    const run = navButton("Run benchmark");
    try {
      await act(async () => { run?.click(); run?.click(); });
      await settle();
      expect(invokeCalls.filter((call) => call.command === "benchmark_server")).toHaveLength(1);
      expect(container.querySelector(".evidence-run-band")).not.toBeNull();
    } finally {
      await act(async () => { pending.reject(new Error("The local request was cancelled")); });
      await settle();
    }
  });

  it.each([
    ["Forced output tokens", "63"], ["Forced output tokens", "4097"], ["Forced output tokens", "64.5"], ["Forced output tokens", ""], ["Forced output tokens", "1e309"],
    ["Measured repeats", "0"], ["Measured repeats", "11"], ["Measured repeats", "1.5"], ["Measured repeats", ""], ["Measured repeats", "1e309"],
  ])("does not dispatch invalid %s=%s", async (label, value) => {
    await setInputValue(profileInput(label), value);
    await click(navButton("Run benchmark"), "the benchmark action must remain visible");
    expect(invokeCalls.filter((call) => call.command === "benchmark_server")).toHaveLength(0);
    expect(navButton("Run benchmark")?.disabled).toBe(true);
    expect(container.querySelector('.benchmark-setup [role="status"]')?.textContent).toContain(label);
  });

  it.each(["Forced output tokens", "Measured repeats"])("marks blank %s invalid in the native control", async (label) => {
    await setInputValue(profileInput(label), "");
    expect(profileInput(label).checkValidity()).toBe(false);
  });

  it.each([
    ["Forced output tokens", "64", "tokens"], ["Forced output tokens", "65", "tokens"], ["Forced output tokens", "4096", "tokens"],
    ["Measured repeats", "1", "repeats"], ["Measured repeats", "10", "repeats"],
  ])("dispatches corrected %s=%s without clamping", async (label, value, field) => {
    await setInputValue(profileInput(label), "");
    expect(navButton("Run benchmark")?.disabled).toBe(true);
    await setInputValue(profileInput(label), value);
    expect(profileInput(label).value).toBe(value);
    expect(profileInput(label).checkValidity()).toBe(true);
    expect(container.querySelector('.benchmark-setup [role="status"]')).toBeNull();
    expect(navButton("Run benchmark")?.disabled).toBe(false);
    await click(navButton("Run benchmark"), "a corrected workload can be dispatched");
    const calls = invokeCalls.filter((call) => call.command === "benchmark_server");
    expect(calls).toHaveLength(1);
    expect(calls[0].args).toMatchObject({ [field]: Number(value) });
  });

  it("shows invalid input beside a prior result without replacing that result", async () => {
    handlers.set("benchmark_server", () => ({ samples: [1, 1, 1], meanTps: 1, medianTps: 1, minTps: 1, maxTps: 1, tokens: 512, repeats: 3 }));
    await click(navButton("Run benchmark"), "the fixture can record a prior result");
    const key = "localmotive:benchmark:fixture";
    const saved = localStorage.getItem(key);
    expect(saved).not.toBeNull();
    invokeCalls.length = 0;
    await setInputValue(profileInput("Forced output tokens"), "4097");
    await click(navButton("Run benchmark"), "the invalid action stays visible");
    expect(invokeCalls.filter((call) => call.command === "benchmark_server")).toHaveLength(0);
    expect(container.querySelector('.benchmark-setup [role="status"]')?.textContent).toContain("Forced output tokens");
    expect(container.querySelector('.benchmark-screen .result-main')?.textContent).toContain("1.00");
    expect(localStorage.getItem(key)).toBe(saved);
  });
});

describe("tuning numeric inputs", () => {
  beforeEach(async () => {
    handlers.set("suggest_port", () => 8080);
    handlers.set("cloud_providers", () => providers);
    handlers.set("cloud_credential_status", () => ({ provider: "openrouter", configured: true, masked: "fixture" }));
    handlers.set("cloud_list_models", () => [{ id: "advisor-alpha", label: "Fixture advisor" }]);
    handlers.set("start_tuning", () => { throw new Error("Fixture boundary: no cloud or inference runs."); });
    await mount();
    await scanFixtureModel();
    await click(navButton("AI Tune"), "AI Tune navigation must exist");
    expect(navButton("Auto-tune fixture")?.disabled, "the fixture must be ready before testing input errors").toBe(false);
    invokeCalls.length = 0;
  });

  it.each(["AI trials", "Tokens per measurement", "Repeats per trial"])("keeps a cleared %s field blank", async (label) => {
    await setInputValue(profileInput(label), "");
    expect(profileInput(label).value).toBe("");
  });

  it.each([
    ["AI trials", "0"], ["AI trials", "13"], ["AI trials", "1.5"], ["AI trials", ""],
    ["Tokens per measurement", "63"], ["Tokens per measurement", "2049"], ["Tokens per measurement", "64.5"], ["Tokens per measurement", ""],
    ["Repeats per trial", "0"], ["Repeats per trial", "6"], ["Repeats per trial", "1.5"], ["Repeats per trial", ""],
    ["AI trials", "1e309"], ["Tokens per measurement", "1e309"], ["Repeats per trial", "1e309"],
  ])("does not dispatch invalid %s=%s", async (label, value) => {
    await setInputValue(profileInput(label), value);
    await click(navButton("Auto-tune fixture"), "the Tune action must remain visible");
    expect(invokeCalls.filter((call) => call.command === "start_tuning")).toHaveLength(0);
    expect(navButton("Auto-tune fixture")?.disabled).toBe(true);
    expect(container.querySelector('.tune-screen [role="status"]')?.textContent).toContain(label);
    expect(text()).not.toContain("Choose a context length and start.");
    expect(text()).toContain("Correct tuning input");
  });

  it.each(["AI trials", "Tokens per measurement", "Repeats per trial"])("marks blank %s invalid in the native control", async (label) => {
    await setInputValue(profileInput(label), "");
    expect(profileInput(label).checkValidity()).toBe(false);
  });

  it.each([
    ["AI trials", "12", "maxTrials"],
    ["Tokens per measurement", "257", "tokens"],
    ["Repeats per trial", "5", "repeats"],
  ])("dispatches the corrected %s value without clamping", async (label, value, field) => {
    await setInputValue(profileInput(label), "");
    expect(navButton("Auto-tune fixture")?.disabled).toBe(true);
    await setInputValue(profileInput(label), value);
    expect(profileInput(label).value).toBe(value);
    expect(profileInput(label).checkValidity()).toBe(true);
    expect(container.querySelector('.tune-screen [role="status"]')).toBeNull();
    expect(navButton("Auto-tune fixture")?.disabled).toBe(false);
    await click(navButton("Auto-tune fixture"), "a corrected workload can be dispatched");
    const calls = invokeCalls.filter((call) => call.command === "start_tuning");
    expect(calls).toHaveLength(1);
    expect(calls[0].args).toMatchObject({ request: { [field]: Number(value) } });
  });
});

describe("provider responses that resolve after a switch (audit FE-03 V1)", () => {
  it("keeps the switched provider's credential and model when the previous provider resolves late", async () => {
    const alphaStatus = deferred<unknown>();
    const betaStatus = deferred<unknown>();
    const betaModels = deferred<unknown>();
    handlers.set("cloud_providers", () => providers);
    handlers.set("cloud_credential_status", (args) => {
      const provider = (args as { provider: string }).provider;
      if (provider === "openrouter") return alphaStatus.promise;
      if (provider === "anthropic") return betaStatus.promise;
      throw new Error(`unexpected provider ${provider}`);
    });
    handlers.set("cloud_list_models", () => betaModels.promise);
    await mount();
    await click(navButton("AI Tune"), "the AI Tune navigation must exist");
    // The OpenRouter status request is still pending; switch to Anthropic.
    await click(tabButton("Anthropic"), "the Anthropic provider tab must exist");
    await act(async () => {
      betaStatus.resolve({ provider: "anthropic", configured: true, masked: "sk-ant-…9999" });
    });
    await settle();
    await act(async () => {
      betaModels.resolve([{ id: "advisor-beta" }]);
    });
    await settle();
    expect(text()).toContain("Anthropic connected");
    expect(text()).toContain("advisor-beta");
    // The previous provider's late answer must not relabel the active tab.
    await act(async () => {
      alphaStatus.resolve({ provider: "openrouter", configured: false, masked: "" });
    });
    await settle();
    expect(text(), "the active provider's credential must stay").toContain("Anthropic connected");
    expect(text(), "a stale credential must not present as missing").not.toContain("Add a key or sign in");
    expect(text(), "the active provider's model must stay").toContain("advisor-beta");
  });

  it("discards a credential-save result that lands after the provider changed", async () => {
    const betaSave = deferred<unknown>();
    handlers.set("cloud_providers", () => providers);
    handlers.set("cloud_credential_status", (args) => {
      const provider = (args as { provider: string }).provider;
      if (provider === "anthropic") return { provider, configured: false, masked: "" };
      if (provider === "openrouter") return { provider, configured: true, masked: "sk-or-…1111" };
      throw new Error(`unexpected provider ${provider}`);
    });
    handlers.set("cloud_list_models", () => [{ id: "advisor-alpha" }]);
    handlers.set("cloud_save_credential", () => betaSave.promise);
    await mount();
    await click(navButton("AI Tune"), "the AI Tune navigation must exist");
    await click(tabButton("Anthropic"), "the Anthropic provider tab must exist");
    const keyInput = container.querySelector('input[aria-label="API key"]') as HTMLInputElement | null;
    expect(keyInput, "the API key input must render").toBeTruthy();
    await setInputValue(keyInput!, "sk-ant-test");
    await click(
      [...container.querySelectorAll("button")].find((button) => (button.textContent ?? "").trim() === "Store"),
      "the Store action must exist",
    );
    // The save is in flight; the user switches back to OpenRouter.
    await click(tabButton("OpenRouter"), "the OpenRouter provider tab must exist");
    expect(text()).toContain("OpenRouter connected");
    // The late save result must not be applied to the new provider.
    await act(async () => {
      betaSave.resolve({ provider: "anthropic", configured: true, masked: "sk-ant-…9999" });
    });
    await settle();
    expect(text(), "the active provider's status must stay").toContain("OpenRouter connected");
    expect(text(), "a stale save must not announce").not.toContain("Anthropic key stored");
    expect(text(), "a stale save must not replace the masked status").toContain("sk-or-…1111");
  });

  it("discards a probe reply that arrives after a provider switch", async () => {
    const probe = deferred<unknown>();
    handlers.set("cloud_providers", () => providers);
    handlers.set("cloud_credential_status", (args) => {
      const provider = (args as { provider: string }).provider;
      if (provider === "anthropic") return { provider, configured: true, masked: "sk-ant-…9999" };
      if (provider === "openrouter") return { provider, configured: true, masked: "sk-or-…1111" };
      throw new Error(`unexpected provider ${provider}`);
    });
    handlers.set("cloud_list_models", (args) => {
      const provider = (args as { provider: string }).provider;
      return provider === "anthropic" ? [{ id: "advisor-beta" }] : [{ id: "advisor-alpha" }];
    });
    handlers.set("cloud_probe", () => probe.promise);
    await mount();
    await click(navButton("AI Tune"), "the AI Tune navigation must exist");
    await click(tabButton("Anthropic"), "the Anthropic provider tab must exist");
    await click(
      [...container.querySelectorAll("button")].find((button) =>
        (button.textContent ?? "").includes("Test connection"),
      ),
      "the Test connection action must exist",
    );
    expect(text()).toContain("Contacting provider…");
    // Switch away while the probe is unanswered.
    await click(tabButton("OpenRouter"), "the OpenRouter provider tab must exist");
    expect(text()).not.toContain("Contacting provider…");
    await act(async () => {
      probe.resolve("PROBE-REPLY-FOR-BETA");
    });
    await settle();
    expect(text(), "the stale probe reply must not be shown").not.toContain("PROBE-REPLY-FOR-BETA");
    expect(text()).toContain("OpenRouter connected");
  });
});
