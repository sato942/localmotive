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
import { ErrorBoundary } from "./ErrorBoundary";

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

  it("renders the rich filter controls from the backend camelCase facet response (GH-05)", async () => {
    // The packaged matrix crash: the backend serializes CatalogFacets with
    // rename_all = camelCase, so pipeline_tags arrives as `pipelineTags`.
    // Reading the snake_case name set the pipeline state to undefined and
    // the render died on .map, blanking the whole app.
    const rows = [model("m1", "alpha")];
    useRows(rows);
    handlers.set("load_model_catalog", () => snapshot(rows));
    handlers.set("fetch_model_catalog", () => snapshot(rows));
    handlers.set("catalog_rich_facets", () => ({
      authors: ["fixture"],
      licenses: ["apache-2.0"],
      pipelineTags: ["text-generation"],
      architectures: ["qwen3"],
    }));
    await mount();
    await openCatalogTab();
    await settle();
    const navs = container.querySelectorAll("button.nav-item, button");
    expect(navs.length).toBeGreaterThan(0);
    const licenceOptions = [...container.querySelectorAll("option")].map((option) => option.textContent ?? "");
    expect(licenceOptions).toContain("apache-2.0");
    const pipelineOptions = [...container.querySelectorAll("option")].map((option) => option.textContent ?? "");
    expect(pipelineOptions).toContain("text-generation");
    expect(pipelineOptions).toContain("qwen3");
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

describe("Raw extra arguments field (audit FE-08)", () => {
  it("keeps typed separators while editing and commits the tokens on blur", async () => {
    await mount();
    handlers.set("scan_models", () => [
      {
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
      },
    ]);
    const navButton = (label: string) =>
      [...container.querySelectorAll("button")].find(
        (button) => (button.textContent ?? "").trim() === label,
      );
    const click = async (target: Element | undefined, why: string) => {
      expect(target, why).toBeTruthy();
      await act(async () => {
        target!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      });
      await settle();
    };
    await click(navButton("Inventory"), "the Inventory navigation must exist");
    const rootInput = container.querySelector(
      'input[aria-label="Model root"]',
    ) as HTMLInputElement | null;
    expect(rootInput, "the model root field must render").toBeTruthy();
    await act(async () => {
      const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
      proto.set!.call(rootInput, "C:/models/fixture-root");
      rootInput!.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await click(
      [...container.querySelectorAll("button")].find((button) =>
        (button.textContent ?? "").includes("Rescan"),
      ),
      "the Rescan action must exist",
    );
    for (let attempt = 0; attempt < 10 && !text().includes("fixture"); attempt += 1) {
      await settle();
    }
    expect(text(), "the scanned model must appear in the inventory").toContain("fixture");
    await click(navButton("Profile"), "the Profile navigation must exist");
    expect(text(), "the Profile screen must render after a selection").toContain("Start");
    const label = [...container.querySelectorAll("label")].find((candidate) =>
      (candidate.textContent ?? "").includes("Raw extra arguments"),
    );
    expect(label, "the raw arguments field must render on the Profile screen").toBeTruthy();
    const input = label!.querySelector("input") as HTMLInputElement;
    const setValue = (value: string) => {
      const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
      proto.set!.call(input, value);
      input.dispatchEvent(new Event("input", { bubbles: true }));
    };
    // The bug: trimming and re-splitting on every keystroke swallowed the
    // separating space, so a second token could never be typed.
    await act(async () => {
      setValue("--flash-attn ");
    });
    expect(input.value).toBe("--flash-attn ");
    await act(async () => {
      setValue("--flash-attn --ctx-size 4096");
    });
    expect(input.value).toBe("--flash-attn --ctx-size 4096");
    await act(async () => {
      input.dispatchEvent(new FocusEvent("blur", { bubbles: true }));
    });
    expect(input.value).toBe("--flash-attn --ctx-size 4096");
  });
});

describe("corrupt persisted records (audit FE-09)", () => {
  it("quarantines an unreadable profile instead of crashing the render", async () => {
    localStorage.setItem("localmotive:profile:fixture/model", "{not json at all");
    localStorage.setItem("localmotive:tuning:fixture/model", "[]");
    await mount();
    handlers.set("scan_models", () => [
      {
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
      },
    ]);
    const navButton = (label: string) =>
      [...container.querySelectorAll("button")].find(
        (button) => (button.textContent ?? "").trim() === label,
      );
    const click = async (target: Element | undefined, why: string) => {
      expect(target, why).toBeTruthy();
      await act(async () => {
        target!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      });
      await settle();
    };
    await click(navButton("Inventory"), "the Inventory navigation must exist");
    const rootInput = container.querySelector('input[aria-label="Model root"]') as HTMLInputElement;
    await act(async () => {
      const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
      proto.set!.call(rootInput, "C:/models/fixture-root");
      rootInput.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await click(
      [...container.querySelectorAll("button")].find((button) =>
        (button.textContent ?? "").includes("Rescan"),
      ),
      "the Rescan action must exist",
    );
    for (let attempt = 0; attempt < 10 && !text().includes("fixture"); attempt += 1) {
      await settle();
    }
    await click(navButton("Profile"), "the Profile navigation must exist");
    // The window still renders, the bad records are quarantined, and the
    // user is told what happened.
    expect(text()).toContain("Raw extra arguments");
    const quarantined = Object.keys(localStorage).filter((key) =>
      key.startsWith("localmotive:quarantine:profile:fixture/model"),
    );
    expect(quarantined, "the unreadable profile must be quarantined").toHaveLength(1);
    expect(localStorage.getItem("localmotive:profile:fixture/model")).toBeNull();
    expect(text().toLowerCase()).toContain("quarantine");
  });
});

describe("ErrorBoundary (audit FE-09)", () => {
  it("shows a recovery screen and clears application state", async () => {
    localStorage.setItem("localmotive:model-root", "C:/models");
    localStorage.setItem("localmotive:quarantine:profile:x:1", "keep me");
    const boundaryContainer = document.createElement("div");
    document.body.appendChild(boundaryContainer);
    const boundaryRoot = createRoot(boundaryContainer);
    const Boom = () => {
      throw new Error("synthetic render failure");
    };
    await act(async () => {
      boundaryRoot.render(
        <ErrorBoundary>
          <Boom />
        </ErrorBoundary>,
      );
    });
    expect(boundaryContainer.textContent).toContain("display error");
    expect(boundaryContainer.textContent).toContain("synthetic render failure");
    const reset = [...boundaryContainer.querySelectorAll("button")].find((button) =>
      (button.textContent ?? "").includes("Reset saved state"),
    );
    expect(reset, "the reset action must be offered").toBeTruthy();
    await act(async () => {
      reset!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(localStorage.getItem("localmotive:model-root")).toBeNull();
    expect(localStorage.getItem("localmotive:quarantine:profile:x:1")).toBe("keep me");
    await act(async () => {
      boundaryRoot.unmount();
    });
    boundaryContainer.remove();
  });
});

describe("download job identity (audit FE-11)", () => {
  it("keeps cancel bound to the running job after the destination is edited", async () => {
    localStorage.setItem("localmotive:model-root", "C:/models/first");
    const rows = [model("m1", "alpha")];
    useRows(rows);
    handlers.set("load_model_catalog", () => snapshot(rows));
    handlers.set("fetch_model_catalog", () => snapshot(rows, { origin: "network" }));
    // The transfer never settles: the job stays running for the assertions.
    handlers.set("download_catalog_file", () => new Promise(() => {}));
    handlers.set("cancel_download", () => false);

    await mount();
    // The destination lives on the Inventory screen; set it before the job.
    const inventoryTab = [...container.querySelectorAll("button")].find((button) =>
      (button.textContent ?? "").trim() === "Inventory",
    );
    await act(async () => {
      inventoryTab!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    const rootInput = document.querySelector('input[aria-label="Model root"]') as HTMLInputElement;
    expect(rootInput, "the model root field must render").toBeTruthy();
    await act(async () => {
      const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
      proto.set!.call(rootInput, "C:/models/first");
      rootInput.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await settle();
    await openCatalogTab();
    await settle();

    const downloadButton = [...container.querySelectorAll("button")].find((button) =>
      (button.textContent ?? "").trim() === "Download",
    );
    expect(downloadButton, "the Download action must render").toBeTruthy();
    expect((downloadButton as HTMLButtonElement).disabled, "Download must be enabled").toBe(false);
    await act(async () => {
      downloadButton!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    expect(invokeCalls.map((call) => call.command)).toContain("download_catalog_file");

    // The user edits the destination while the job runs: navigate back to
    // the live Inventory screen so the controlled input is attached.
    await act(async () => {
      inventoryTab!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    const liveRoot = document.querySelector('input[aria-label="Model root"]') as HTMLInputElement;
    expect(liveRoot, "the model root field must be live again").toBeTruthy();
    await act(async () => {
      const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
      proto.set!.call(liveRoot, "D:/elsewhere");
      liveRoot.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await settle();
    expect((liveRoot as HTMLInputElement).value).toBe("D:/elsewhere");
    await openCatalogTab();
    await settle();

    const stopButton = [...container.querySelectorAll("button")].find((button) =>
      (button.textContent ?? "").includes("Keep & stop"),
    );
    expect(stopButton, "the running job must still offer cancellation").toBeTruthy();
    await act(async () => {
      stopButton!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();

    const cancelCall = invokeCalls.find((call) => call.command === "cancel_download");
    expect(cancelCall, "cancel_download must be invoked").toBeTruthy();
    expect((cancelCall!.args as { destination: string }).destination).toBe("C:/models/first");
    // The boolean result is surfaced: false means nothing was stopped.
    expect(text()).toContain("not being downloaded");
  });
});
