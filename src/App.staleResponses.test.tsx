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
}

describe("stale port suggestions and command previews (audit FE-03 V3)", () => {
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
