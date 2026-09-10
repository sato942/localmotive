// @vitest-environment jsdom
//
// Component tests for the HF catalog presentation (audit QD-02/QD-03): the
// loading, empty, error and rate-limit presentation scenarios live here, on
// public React/DOM interfaces with the IPC boundary mocked and nothing else.
// The packaged verifier no longer reaches into private React state for them.
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
      return Promise.resolve({
        hardware: {
          architecture: "x86_64",
          gpuNames: [],
          vendor: "unknown",
          cudaMajor: null,
          driverVersion: "unknown",
          detectionStatus: "fixture",
          recommendation: "fixture",
          systemMemory: {
            totalPhysicalBytes: {
              value: null,
              level: "unknown",
              source: { kind: "unknown", detail: "fixture" },
              observedAtMs: 0,
              notes: [],
            },
            availablePhysicalBytes: {
              value: null,
              level: "unknown",
              source: { kind: "unknown", detail: "fixture" },
              observedAtMs: 0,
              notes: [],
            },
            memoryLoadPercent: {
              value: null,
              level: "unknown",
              source: { kind: "unknown", detail: "fixture" },
              observedAtMs: 0,
              notes: [],
            },
          },
          adapters: [],
          manualOverrides: [],
        },
        catalog: null,
        catalogError: null,
        runtimeRoot: "",
        managedRuntimes: [],
      });
    }
    if (command === "cloud_providers") return Promise.resolve([]);
    if (command === "cloud_credential_status") return Promise.resolve({ configured: false });
    if (command === "hf_token_status") return Promise.resolve({ configured: false });
    if (command === "catalog_facets") {
      return Promise.resolve([[], []]);
    }
    if (command === "catalog_rich_facets") {
      return Promise.resolve({ authors: [], licenses: [], pipeline_tags: [], architectures: [] });
    }
    if (command === "catalog_fit_budget") return Promise.resolve({ budgetBytes: 0, source: "unknown" });
    return Promise.resolve(null);
  },
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => {}) }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: () => Promise.resolve(null) }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

import App from "./App";

const file = (filename: string) => ({
  filename,
  quant: "Q4_K_M",
  sizeBytes: 4_000_000_000,
  sha256: "a".repeat(64),
  revision: "main",
});

const model = (id: string, family: string) => ({
  id,
  repo: `org/${family}-GGUF`,
  family,
  parameters: "7B",
  publisher: "org",
  summary: "Fixture model",
  tags: ["text-generation"],
  gated: false,
  downloads: 100,
  likes: 10,
  files: [file(`${family}-Q4_K_M.gguf`)],
});

const catalog = (models: unknown[]) => ({
  schemaVersion: 2,
  updated: "2026-09-01T00:00:00Z",
  source: "fixture",
  models,
});

const snapshot = (models: unknown[], over: Record<string, unknown> = {}) => ({
  catalog: catalog(models),
  origin: "cache",
  fetchedAt: "2026-09-11T00:00:00Z",
  url: "https://example.invalid/catalog.json",
  ...over,
});

let container: HTMLDivElement;
let root: Root;

beforeEach(() => {
  (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {};
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
});

async function mount() {
  await act(async () => {
    root.render(<App />);
  });
}

async function openCatalogTab() {
  const tab = [...container.querySelectorAll("button")].find((button) =>
    (button.textContent ?? "").includes("HF Catalog"),
  );
  expect(tab, "HF Catalog navigation must exist").toBeTruthy();
  await act(async () => {
    tab!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
}

function text() {
  return container.textContent ?? "";
}

function refreshButton() {
  return [...container.querySelectorAll("button")].find((button) =>
    (button.textContent ?? "").includes("Refresh list"),
  );
}

async function settle() {
  await act(async () => {
    for (let index = 0; index < 8; index += 1) {
      await Promise.resolve();
    }
  });
}

function useRows(rows: ReturnType<typeof model>[]) {
  handlers.set("catalog_local_models", () => rows);
  handlers.set("filter_catalog", () => rows);
}

describe("HF catalog presentation through public interfaces", () => {
  it("marks the fetched rows and shows the local snapshot after first fill", async () => {
    const rows = [model("m1", "alpha"), model("m2", "beta")];
    useRows(rows);
    handlers.set("load_model_catalog", () => snapshot(rows, { origin: "cache" }));
    handlers.set("fetch_model_catalog", () => snapshot(rows, { origin: "network" }));

    await mount();
    await openCatalogTab();
    await settle();

    expect(text()).toContain("alpha");
    expect(text()).toContain("beta");
    const commands = invokeCalls.map((call) => call.command);
    expect(commands.indexOf("load_model_catalog")).toBeGreaterThanOrEqual(0);
    expect(commands.indexOf("fetch_model_catalog")).toBeGreaterThan(commands.indexOf("load_model_catalog"));
  });

  it("shows the honest error state when the network refresh fails and the cache serves", async () => {
    const rows = [model("m1", "alpha")];
    useRows(rows);
    handlers.set("load_model_catalog", () => snapshot(rows));
    handlers.set("fetch_model_catalog", () => {
      throw new Error("GitHub rate-limited the catalog request");
    });

    await mount();
    await openCatalogTab();
    await settle();

    expect(text()).toContain("alpha");
    expect(text()).toContain("Refresh pending");
    expect(text()).toContain("rate-limited");
  });

  it("shows the empty state when no row matches the filters", async () => {
    useRows([]);
    handlers.set("load_model_catalog", () => snapshot([model("m1", "alpha")]));
    handlers.set("fetch_model_catalog", () => snapshot([model("m1", "alpha")]));

    await mount();
    await openCatalogTab();
    await settle();

    expect(text()).toContain("No curated model matches these filters");
  });

  it("keeps the loading state visible until the local load resolves, then renders rows", async () => {
    let release!: (value: unknown) => void;
    const pending = new Promise((resolvePromise) => {
      release = resolvePromise;
    });
    const rows = [model("m1", "alpha")];
    useRows(rows);
    handlers.set("load_model_catalog", () => pending);
    handlers.set("fetch_model_catalog", () => snapshot(rows));

    await mount();
    await openCatalogTab();
    expect(text()).toContain("Fetching curated catalog");

    await act(async () => {
      release(snapshot(rows));
    });
    await settle();
    expect(text()).toContain("alpha");
  });

  it("a manual refresh reuses the merged collection and reports cooldown honestly", async () => {
    const rows = [model("m1", "alpha")];
    useRows(rows);
    let localLoads = 0;
    handlers.set("load_model_catalog", () => {
      localLoads += 1;
      return snapshot(rows, localLoads > 1 ? { cooldownRemainingMinutes: 26, origin: "cache" } : {});
    });
    handlers.set("fetch_model_catalog", () =>
      snapshot(rows, { origin: "cache", cooldownRemainingMinutes: 26 }),
    );

    await mount();
    await openCatalogTab();
    await settle();

    const button = refreshButton();
    expect(button).toBeTruthy();
    await act(async () => {
      button!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    expect(text()).toContain("COOLDOWN 26 MIN LEFT");
    expect(text()).toContain("alpha");
  });
});
