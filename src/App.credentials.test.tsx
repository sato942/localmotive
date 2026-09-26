// @vitest-environment jsdom
//
// Credential-draft hygiene (audit FE-01): a typed secret must not survive a
// failed save or a screen unmount. These tests drive the real App through its
// public DOM with the IPC boundary mocked.
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
    if (command === "cloud_credential_status") return Promise.resolve({ configured: false });
    if (command === "hf_token_status") return Promise.resolve({ configured: false });
    if (command === "catalog_facets") return Promise.resolve([[], []]);
    if (command === "catalog_rich_facets") {
      return Promise.resolve({ authors: [], licenses: [], pipeline_tags: [], architectures: [] });
    }
    if (command === "catalog_fit_budget") return Promise.resolve({ budgetBytes: 0, source: "unknown" });
    if (command === "scan_models_report") {
      return Promise.resolve({ models: [], problems: [], truncated: false });
    }
    return Promise.resolve(null);
  },
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => {}) }));
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: () => Promise.resolve(null as unknown),
  save: () => Promise.resolve(null),
}));
vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: (_url: string) => Promise.resolve(),
}));

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

let container: HTMLDivElement;
let root: Root;

beforeEach(() => {
  window.__TAURI_INTERNALS__ = {};
  container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);
  invokeCalls.length = 0;
  handlers.clear();
  handlers.set("catalog_local_models", () => []);
  handlers.set("filter_catalog", () => []);
  handlers.set("load_model_catalog", () => ({
    catalog: {
      schemaVersion: 2,
      updated: "2026-09-01T00:00:00Z",
      source: "fixture",
      models: [],
    },
    origin: "cache",
    fetchedAt: "2026-09-11T00:00:00Z",
    url: "https://example.invalid/catalog.json",
  }));
});

afterEach(async () => {
  await act(async () => {
    root.unmount();
  });
  container.remove();
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

async function navTo(label: string) {
  const tab = [...container.querySelectorAll("button")].find((button) =>
    (button.textContent ?? "").includes(label),
  );
  expect(tab, `${label} navigation must exist`).toBeTruthy();
  await act(async () => {
    tab!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
  await settle();
}

async function typeInto(input: HTMLInputElement, value: string) {
  await act(async () => {
    const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
    proto.set!.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
  await settle();
}

async function clickButton(button: HTMLButtonElement) {
  await act(async () => {
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
  await settle();
}

function passwordInputs() {
  return [...container.querySelectorAll('input[type="password"]')] as HTMLInputElement[];
}

function storeButtonFor(input: HTMLInputElement) {
  const row = input.closest("div");
  const button = row?.querySelector("button") as HTMLButtonElement | null;
  expect(button, "the Store button next to the secret field must exist").toBeTruthy();
  return button!;
}

describe("credential drafts clear on every exit path", () => {
  it("clears the HF token draft when the save fails", async () => {
    handlers.set("save_hf_token", () => {
      throw new Error("keyring unavailable in fixture");
    });
    await mount();
    await navTo("HF Catalog");
    const [tokenInput] = passwordInputs();
    expect(tokenInput, "the HF token field must render").toBeTruthy();
    await typeInto(tokenInput, "hf_fixture_secret");
    expect(tokenInput.value).toBe("hf_fixture_secret");
    await clickButton(storeButtonFor(tokenInput));
    expect(tokenInput.value).toBe("");
  });

  it("clears the HF token draft when the catalog screen unmounts", async () => {
    await mount();
    await navTo("HF Catalog");
    const [tokenInput] = passwordInputs();
    await typeInto(tokenInput, "hf_fixture_secret");
    await navTo("Control");
    await navTo("HF Catalog");
    const [tokenAgain] = passwordInputs();
    expect(tokenAgain.value).toBe("");
  });

  it("clears the cloud key draft when the save fails", async () => {
    handlers.set("cloud_providers", () => [
      {
        id: "openrouter",
        label: "OpenRouter",
        baseUrl: "https://example.invalid",
        keyPrefixHint: "sk-or-",
        consoleUrl: "https://example.invalid",
        supportsOauth: false,
        defaultModel: "m",
        listsModels: false,
      },
    ]);
    handlers.set("cloud_save_credential", () => {
      throw new Error("keyring unavailable in fixture");
    });
    await mount();
    await navTo("AI Tune");
    const keyInput = container.querySelector('input[aria-label="API key"]') as HTMLInputElement | null;
    expect(keyInput, "the cloud API key field must render").toBeTruthy();
    await typeInto(keyInput!, "sk-or-fixture-secret");
    await clickButton(storeButtonFor(keyInput!));
    expect(keyInput!.value).toBe("");
  });

  it("clears the cloud key draft when the tune screen unmounts", async () => {
    handlers.set("cloud_providers", () => [
      {
        id: "openrouter",
        label: "OpenRouter",
        baseUrl: "https://example.invalid",
        keyPrefixHint: "sk-or-",
        consoleUrl: "https://example.invalid",
        supportsOauth: false,
        defaultModel: "m",
        listsModels: false,
      },
    ]);
    await mount();
    await navTo("AI Tune");
    const keyInput = container.querySelector('input[aria-label="API key"]') as HTMLInputElement | null;
    await typeInto(keyInput!, "sk-or-fixture-secret");
    await navTo("Control");
    await navTo("AI Tune");
    const keyAgain = container.querySelector('input[aria-label="API key"]') as HTMLInputElement | null;
    expect(keyAgain!.value).toBe("");
  });
});
