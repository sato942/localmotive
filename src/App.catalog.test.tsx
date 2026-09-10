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
      return Promise.resolve(runtimeSetup());
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

  describe("runtime catalog presentation through public interfaces", () => {
    async function openRuntimeTab() {
      const tab = [...container.querySelectorAll("button")].find((button) =>
        (button.textContent ?? "").trim() === "Runtime",
      );
      expect(tab, "Runtime navigation must exist").toBeTruthy();
      await act(async () => {
        tab!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      });
    }

    function refreshCatalogButton() {
      return container.querySelector('[aria-label="Refresh approved runtime catalog"]');
    }

    it("shows one loading state and disables refresh until the retrieval resolves", async () => {
      let release!: (value: unknown) => void;
      const pending = new Promise((resolvePromise) => {
        release = resolvePromise;
      });
      // The refresh action re-runs the runtime setup load; the first call
      // (mount) resolves, the click re-run stays pending until released.
      let calls = 0;
      handlers.set("load_runtime_setup", () => {
        calls += 1;
        return calls === 1 ? runtimeSetup() : pending;
      });
      await mount();
      await openRuntimeTab();
      await settle();

      const button = refreshCatalogButton();
      expect(button, "the runtime catalog refresh action must exist").toBeTruthy();
      await act(async () => {
        (button as HTMLButtonElement).dispatchEvent(new MouseEvent("click", { bubbles: true }));
      });
      await settle();
      expect(container.querySelectorAll(".runtime-loading").length).toBe(1);
      expect(refreshCatalogButton()?.getAttribute("disabled")).not.toBeNull();

      await act(async () => {
        release(
          runtimeSetup({
            catalog: { tag: "b1", publishedAt: "", options: [], availability: [], origin: "cache", recommendationReason: "" },
          }),
        );
      });
      await settle();
      expect(container.querySelectorAll(".runtime-loading").length).toBe(0);
      expect(container.querySelectorAll(".runtime-catalog-message").length).toBeGreaterThanOrEqual(1);
    });

    it("shows the explicit empty state without a loading indicator", async () => {
      handlers.set("load_runtime_setup", () =>
        runtimeSetup({
          catalog: { tag: "b1", publishedAt: "", options: [], availability: [], origin: "cache", recommendationReason: "" },
        }),
      );
      await mount();
      await openRuntimeTab();
      await settle();
      const message = container.querySelectorAll(".runtime-catalog-message");
      expect(message.length).toBeGreaterThanOrEqual(1);
      expect(message[0].textContent ?? "").toContain("No approved runtime");
      expect(container.querySelectorAll(".runtime-loading").length).toBe(0);
    });

    it("shows a terminal error with one retry action", async () => {
      handlers.set("load_runtime_setup", () =>
        runtimeSetup({
          catalogError: { kind: "network", message: "Verifier-injected catalog failure" },
        }),
      );
      await mount();
      await openRuntimeTab();
      await settle();
      const errored = container.querySelectorAll(".runtime-catalog-message.error");
      expect(errored.length).toBe(1);
      expect(errored[0].textContent ?? "").toContain("Verifier-injected catalog failure");
      const retries = [...errored[0].querySelectorAll("button")].filter(
        (button) => (button.textContent ?? "").trim() === "Retry",
      );
      expect(retries.length).toBe(1);
      expect(container.querySelectorAll(".runtime-loading").length).toBe(0);
    });

    it("shows the rate-limit retry delay honestly", async () => {
      handlers.set("load_runtime_setup", () =>
        runtimeSetup({
          catalogError: {
            kind: "rate_limited",
            message: "GitHub rate limit hit; retry after 60 seconds",
            retryAfterSeconds: 60,
          },
        }),
      );
      await mount();
      await openRuntimeTab();
      await settle();
      const errored = container.querySelectorAll(".runtime-catalog-message.error");
      expect(errored.length).toBe(1);
      expect((errored[0].textContent ?? "").toLowerCase()).toContain("retry after 60 seconds");
    });
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
