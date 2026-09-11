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
    // The app path uses the bounded report command (audit S-15); the legacy
    // array handlers stay the source for tests.
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
const dialogBehavior = { open: () => Promise.resolve(null as unknown) };
const openerBehavior = { openUrl: (_url: string) => Promise.resolve() };
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: () => dialogBehavior.open(),
  save: () => Promise.resolve(null),
}));
vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: (url: string) => openerBehavior.openUrl(url),
}));

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


  it("labels the fit of the selected build, never the smallest variant (S-11)", async () => {
    // A 10 GB dedicated budget with the default 50% fraction is a 5 GB
    // threshold: the 2 GB build passes, the 8 GB build does not. The row
    // stays listed (the smallest build fits), but the label for the selected
    // larger build must fail and must say why the row survives.
    const evidence = (value: number | string | null) => ({
      value,
      level: value === null ? "unknown" : "observed",
      source: { kind: "vendorApi", detail: "fixture" },
      observedAtMs: 1,
      notes: [],
    });
    const hardware = {
      ...runtimeSetup().hardware,
      adapters: [
        {
          adapterId: "gpu-0",
          compatibilityId: "compat-gpu-0",
          name: "GPU 0",
          vendor: "nvidia",
          driver: evidence("580.0"),
          backend: evidence("cuda"),
          dedicatedBytes: evidence(10_000_000_000),
          sharedBytes: evidence(null),
          budgetBytes: evidence(10_000_000_000),
          currentUsageBytes: evidence(0),
          availableBudgetBytes: evidence(9_000_000_000),
          reservationBytes: evidence(0),
          availableForReservationBytes: evidence(null),
          capacityObservations: [],
        },
      ],
    };
    handlers.set("load_runtime_setup", () => runtimeSetup({ hardware }));
    handlers.set("detect_hardware", () => runtimeSetup({ hardware }));

    const twoBuild = {
      ...model("m1", "alpha"),
      files: [
        { filename: "alpha-Q4_K_M.gguf", quant: "Q4_K_M", sizeBytes: 2_000_000_000, sha256: "a".repeat(64), revision: "main" },
        { filename: "alpha-Q8_0.gguf", quant: "Q8_0", sizeBytes: 8_000_000_000, sha256: "b".repeat(64), revision: "main" },
      ],
    };
    const rows = [twoBuild];
    useRows(rows);
    handlers.set("load_model_catalog", () => snapshot(rows));
    handlers.set("fetch_model_catalog", () => snapshot(rows));

    await mount();
    await openCatalogTab();
    await settle();

    expect(text()).toContain("SIZE CHECK PASSES");
    expect(text()).not.toContain("SIZE CHECK FAILS");

    const buildSelect = [...container.querySelectorAll("select")].find((select) =>
      [...select.options].some((option) => (option.textContent ?? "").includes("Q8_0")),
    );
    expect(buildSelect, "the build selector must list both quants").toBeTruthy();
    await act(async () => {
      buildSelect!.value = "alpha-Q8_0.gguf";
      buildSelect!.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await settle();

    expect(text()).toContain("SIZE CHECK FAILS FOR THIS BUILD");
    expect(text()).toContain("smaller build fits");
    // The passing claim for the small build must be gone for the selection.
    expect(text()).not.toContain("SIZE CHECK PASSES");
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
    // The consequence of a successful scan is the loaded selection, not the
    // transient notice string: a concurrent port probe owns the single notice
    // line and may win the last write (the scan promise gained one hop with
    // the S-15 report shape).
    for (let attempt = 0; attempt < 10 && !text().includes("Launch profile"); attempt += 1) {
      await settle();
    }
    expect(text(), "the scan must load the profile for the scanned model").toContain("Launch profile");
    const nameField = [...container.querySelectorAll("input")].find(
      (input) => (input as HTMLInputElement).value.includes("fixture"),
    );
    expect(nameField, "the scanned model must be selected and loaded").toBeTruthy();
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

describe("assistive-technology structure (audit FE-13)", () => {
  const fixtureScan = () => [
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
  ];
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

  it("inventory is a real table with column headers and named row actions", async () => {
    handlers.set("scan_models", fixtureScan);
    await mount();
    await click(navButton("Inventory"), "Inventory navigation");
    const rootInput = document.querySelector('input[aria-label="Model root"]') as HTMLInputElement;
    await act(async () => {
      const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
      proto.set!.call(rootInput, "C:/models/fixture-root");
      rootInput.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await click(
      [...container.querySelectorAll("button")].find((button) =>
        (button.textContent ?? "").includes("Rescan"),
      ),
      "Rescan action",
    );
    for (let attempt = 0; attempt < 10 && !text().includes("rescan"); attempt += 1) {
      await settle();
    }
    // The scan selects the model and moves to the Profile screen; return to
    // the inventory to inspect the table.
    await click(navButton("Inventory"), "Inventory navigation again");
    const table = container.querySelector("table.inventory-table");
    expect(table, "the inventory must render a table").toBeTruthy();
    const headers = [...table!.querySelectorAll("th")];
    expect(headers).toHaveLength(6);
    expect(headers.every((th) => th.getAttribute("scope") === "col")).toBe(true);
    const rowButton = table!.querySelector("tbody .row-target") as HTMLButtonElement | null;
    expect(rowButton, "each row must expose a named selection control").toBeTruthy();
    expect(rowButton!.textContent).toContain("fixture");
    // The row control is reachable and selects on Enter like any button.
    rowButton!.focus();
    expect(document.activeElement).toBe(rowButton);
  });

  it("paired numeric inputs each carry their own label", async () => {
    handlers.set("scan_models", fixtureScan);
    await mount();
    await click(navButton("Inventory"), "Inventory navigation");
    const rootField = document.querySelector('input[aria-label="Model root"]') as HTMLInputElement;
    await act(async () => {
      const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
      proto.set!.call(rootField, "C:/models/fixture-root");
      rootField.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await click(
      [...container.querySelectorAll("button")].find((button) =>
        (button.textContent ?? "").includes("Rescan"),
      ),
      "Rescan action",
    );
    for (let attempt = 0; attempt < 10 && !container.querySelector("table.inventory-table"); attempt += 1) {
      await settle();
    }
    await click(navButton("Profile"), "Profile navigation");
    const labelWith = (needle: string) =>
      [...container.querySelectorAll("label")].find((candidate) =>
        (candidate.textContent ?? "").includes(needle),
      );
    const min = labelWith("N-gram draft min");
    const max = labelWith("N-gram draft max");
    const sizeN = labelWith("N-gram map size n");
    const sizeM = labelWith("N-gram map size m");
    for (const [label, name] of [
      [min, "min"],
      [max, "max"],
      [sizeN, "n"],
      [sizeM, "m"],
    ] as const) {
      expect(label, `the ${name} field must have its own label`).toBeTruthy();
      expect(label!.querySelectorAll("input")).toHaveLength(1);
    }
    expect(min).not.toBe(max);
  });

  it("provider tabs follow the WAI-ARIA keyboard pattern with linked tabpanel", async () => {
    handlers.set("cloud_providers", () => [
      { id: "openrouter", label: "OpenRouter", supportsOauth: true, keyPrefixHint: "sk-or", consoleUrl: "https://openrouter.ai", defaultModel: "m" },
      { id: "second", label: "Second Cloud", supportsOauth: false, keyPrefixHint: "sk-2", consoleUrl: "https://example.invalid", defaultModel: "m2" },
    ]);
    await mount();
    await click(navButton("AI Tune"), "AI Tune navigation");
    const tablist = container.querySelector('[role="tablist"]');
    expect(tablist, "the provider tablist must render").toBeTruthy();
    const tabs = [...tablist!.querySelectorAll('[role="tab"]')] as HTMLButtonElement[];
    expect(tabs.length).toBeGreaterThanOrEqual(2);
    const selectedIndex = tabs.findIndex((tab) => tab.getAttribute("aria-selected") === "true");
    expect(selectedIndex).toBeGreaterThanOrEqual(0);
    // Roving tabindex: exactly one tab is in the tab order.
    expect(tabs.filter((tab) => tab.tabIndex === 0)).toHaveLength(1);
    expect(tabs.every((tab) => tab.getAttribute("aria-controls") === "provider-panel")).toBe(true);
    const panel = container.querySelector('[role="tabpanel"]');
    expect(panel?.getAttribute("aria-labelledby")).toBe(tabs[selectedIndex].id);
    // ArrowRight moves selection and focus to the next tab.
    const next = tabs[(selectedIndex + 1) % tabs.length];
    await act(async () => {
      tabs[selectedIndex].dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true }));
    });
    await settle();
    const refreshed = [...tablist!.querySelectorAll('[role="tab"]')];
    const nowSelected = refreshed.find((tab) => tab.getAttribute("aria-selected") === "true");
    expect(nowSelected?.id).toBe(next.id);
    expect(document.activeElement?.id).toBe(next.id);
  });
});

describe("capability and status words stay honest (audit FE-15)", () => {
  it("says shards complete, path selected and not inspected instead of overstating", async () => {
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
    await mount();
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
    // The runtime path is chosen before the scan so the suggested profile
    // carries it and the speculation select can be inspected.
    await click(navButton("Runtime"), "Runtime navigation");
    const runtimeField = [...container.querySelectorAll("input")].find(
      (input) => (input as HTMLInputElement).placeholder === "Path to llama-server.exe",
    ) as HTMLInputElement | undefined;
    expect(runtimeField, "the runtime path field must render").toBeTruthy();
    await act(async () => {
      const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
      proto.set!.call(runtimeField, "C:/runtime/llama-server.exe");
      runtimeField!.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await settle();
    await click(navButton("Inventory"), "Inventory navigation");
    const rootInput = document.querySelector('input[aria-label="Model root"]') as HTMLInputElement;
    await act(async () => {
      const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
      proto.set!.call(rootInput, "C:/models/fixture-root");
      rootInput.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await click(
      [...container.querySelectorAll("button")].find((button) =>
        (button.textContent ?? "").includes("Rescan"),
      ),
      "Rescan action",
    );
    for (let attempt = 0; attempt < 10 && !text().includes("SHARDS") && !text().includes("Complete"); attempt += 1) {
      await settle();
    }
    // The loaded-profile tag states the shard fact, not a validation claim.
    await click(navButton("Control"), "Control navigation");
    expect(text()).toContain("SHARDS COMPLETE");
    expect(text()).not.toContain("VALID");
    // The profile's speculation list is marked provisional until inspection.
    await click(navButton("Profile"), "Profile navigation");
    expect(text(), "provisional before inspection").toContain("provisional");
    // Inspecting binds the list to this executable…
    handlers.set("inspect_runtime", () => ({ build: "b9999", specTypes: ["none", "draft-mtp"], supportedFlags: [] }));
    handlers.set("describe_runtime", () => ({ path: "C:/runtime/llama-server.exe", backend: "cuda", cudaMajor: 13, tag: "b9999", installKey: null, source: "manifest", managedVerified: false }));
    await click(
      [...container.querySelectorAll("button")].find((button) =>
        (button.textContent ?? "").includes("Inspect selected runtime"),
      ),
      "Inspect action",
    );
    expect(text()).not.toContain("provisional");
    // …and editing the path clears the stale capabilities (FE-15).
    const execInput = [...container.querySelectorAll("input")].find(
      (input) => (input as HTMLInputElement).value === "C:/runtime/llama-server.exe"
        && input.getAttribute("type") !== "password",
    ) as HTMLInputElement | undefined;
    expect(execInput, "the profile executable field must render").toBeTruthy();
    await act(async () => {
      const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
      proto.set!.call(execInput, "C:/runtime/other.exe");
      execInput!.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await settle();
    expect(text(), "provisional after path edit").toContain("provisional");
    // The Runtime screen's own path field clears stale capabilities too.
    await click(
      [...container.querySelectorAll("button")].find((button) =>
        (button.textContent ?? "").includes("Inspect selected runtime"),
      ),
      "Inspect action again",
    );
    expect(text()).not.toContain("provisional");
    await click(navButton("Runtime"), "Runtime navigation");
    const liveRuntimeField = [...container.querySelectorAll("input")].find(
      (input) => (input as HTMLInputElement).placeholder === "Path to llama-server.exe",
    ) as HTMLInputElement;
    await act(async () => {
      const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
      proto.set!.call(liveRuntimeField, "C:/runtime/third.exe");
      liveRuntimeField.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await settle();
    await click(navButton("Profile"), "Profile navigation again");
    expect(text(), "provisional after Runtime-screen path edit").toContain("provisional");
    // First-run readiness does not claim validation.
    await click(navButton("Runtime"), "Runtime navigation");
    expect(text()).toContain("Path selected");
    expect(text()).toContain("Ready to validate");
  });
});

describe("Bounded discovery diagnostics (audit S-15)", () => {
  it("renders bounded scan diagnostics while keeping discovered models", async () => {
    const fixtureScan = [
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
    ];
    handlers.set("scan_models", () => fixtureScan);
    handlers.set("scan_models_report", () => ({
      models: fixtureScan,
      problems: [
        { path: "C:/models/locked", reason: "Could not read this directory: access denied" },
        { path: "C:/models/deep", reason: "depth limit 8 reached" },
      ],
      truncated: true,
    }));

    await mount();
    const nav = [...container.querySelectorAll("button")].find(
      (button) => (button.textContent ?? "").trim() === "Inventory",
    );
    await act(async () => {
      nav!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    const rootInput = container.querySelector(
      'input[aria-label="Model root"]',
    ) as HTMLInputElement | null;
    await act(async () => {
      const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
      proto.set!.call(rootInput, "C:/models/fixture-root");
      rootInput!.dispatchEvent(new Event("input", { bubbles: true }));
    });
    const rescan = [...container.querySelectorAll("button")].find((button) =>
      (button.textContent ?? "").includes("Rescan"),
    );
    await act(async () => {
      rescan!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    for (let attempt = 0; attempt < 10 && !text().includes("scan diagnostics"); attempt += 1) {
      await settle();
    }
    const diagnostics = container.querySelector(".scan-diagnostics");
    expect(diagnostics, "bounded scan diagnostics must render").toBeTruthy();
    const rendered = diagnostics!.textContent ?? "";
    expect(rendered).toContain("stopped early");
    expect(rendered).toContain("access denied");
    expect(rendered).toContain("depth limit 8 reached");
    // Valid discovered models survive alongside the diagnostics.
    expect(text()).toContain("fixture");
  });

  it("shows the cloud data disclosure before tuning and records the chosen mode (S-20)", async () => {
    handlers.set("tune_disclosure_list", () =>
      [
        {
          category: "Hardware",
          detail: "CPU architecture, GPU names and VRAM, driver version, system memory.",
          fields: ["hardware", "systemRamBytes"],
          sentInMinimal: true,
        },
        {
          category: "Launch profile",
          detail: "Every launch flag and value, including model, runtime and companion paths.",
          fields: ["baselineProfile"],
          sentInMinimal: false,
        },
      ],
    );
    await mount();
    const nav = [...container.querySelectorAll("button")].find(
      (button) => (button.textContent ?? "").trim() === "AI Tune",
    );
    await act(async () => {
      nav!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    const rendered = text();
    expect(
      container.querySelector('[aria-label="Cloud data disclosure"]'),
      "the disclosure block must be visible on the tuning screen",
    ).toBeTruthy();
    expect(rendered, "the disclosure block content must render").toContain(
      "What the brief carries",
    );
    expect(rendered, "local inference and export must be named separately").toContain(
      "Local inference and local share export never send data anywhere",
    );
    expect(rendered, "the section list comes from Rust").toContain("Hardware");
    expect(rendered, "full-only sections must say so").toContain("(full mode only)");
    expect(rendered, "credentials must be explicitly excluded").toContain(
      "Stored cloud credentials are never part of the brief",
    );

    const radios = [
      ...container.querySelectorAll('input[name="tune-disclosure"]'),
    ] as HTMLInputElement[];
    expect(radios.length, "both disclosure modes must be selectable").toBe(2);
    expect(radios[0].checked, "full is the default").toBe(true);
    await act(async () => {
      radios[1].dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    expect(localStorage.getItem("localmotive:tune-disclosure")).toBe("minimal");
    const minimalRadio = [
      ...container.querySelectorAll('input[name="tune-disclosure"]'),
    ][1] as HTMLInputElement;
    expect(minimalRadio.checked, "the chosen mode must stay selected").toBe(true);
  });

  it("presents dialog and opener rejections with recovery text and a copyable diagnostic (S-22)", async () => {
    dialogBehavior.open = () => Promise.reject("dialog backend unavailable");
    openerBehavior.openUrl = () => Promise.reject({ code: "no-handler", message: "no browser registered" });
    await mount();

    // The folder picker rejects: the current root stands, the failure is
    // visible with recovery text, and a diagnostic affordance appears.
    const inventoryNav = [...container.querySelectorAll("button")].find(
      (button) => (button.textContent ?? "").trim() === "Inventory",
    );
    await act(async () => {
      inventoryNav!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    const before = (
      container.querySelector('input[aria-label="Model root"]') as HTMLInputElement | null
    )?.value;
    const chooseButton = [...container.querySelectorAll("button")].find((button) =>
      (button.textContent ?? "").toLowerCase().includes("choose"),
    );
    await act(async () => {
      chooseButton!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    const notice = container.querySelector(".notice-line");
    expect(notice!.textContent).toContain("Could not open the folder picker");
    expect(notice!.textContent).toContain("dialog backend unavailable");
    expect(notice!.textContent).toContain("unchanged");
    const rootField = container.querySelector(
      'input[aria-label="Model root"]',
    ) as HTMLInputElement | null;
    if (rootField) expect(rootField.value).toBe(before);
    const details = container.querySelector(".notice-diagnostic details pre");
    expect(details, "the raw diagnostic must be disclosed, not spilled into the notice").toBeTruthy();
    expect(details!.textContent).toContain("dialog backend unavailable");

    // The browser opener rejects on a link: same contract, no unhandled
    // rejection and no loss of the current screen.
    const link = [...container.querySelectorAll("button")].find((button) =>
      (button.textContent ?? "").includes("GitHub releases"),
    ) ?? [...container.querySelectorAll("button")].find((button) =>
      (button.textContent ?? "").includes("Project on GitHub"),
    );
    if (link) {
      await act(async () => {
        link.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      });
      await settle();
      const after = container.querySelector(".notice-line")!.textContent ?? "";
      expect(after).toContain("Could not open the browser");
    }
  });

  it("first-run Profile and Inventory empty states give an accurate next action (S-23)", async () => {
    handlers.set("scan_models_report", () => ({ models: [], problems: [], truncated: false }));
    await mount();
    const navFor = (label: string) =>
      [...container.querySelectorAll("button")].find(
        (button) => (button.textContent ?? "").trim() === label,
      );
    // Profile before any selection: not a blank screen; routes to Inventory.
    await act(async () => {
      navFor("Profile")!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    let rendered = text();
    expect(rendered).toContain("No models to profile yet");
    expect(rendered).toContain("Open Inventory");
    await act(async () => {
      [...container.querySelectorAll("button")]
        .find((button) => (button.textContent ?? "").includes("Open Inventory"))!
        .dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    // Inventory with no root: asks for a folder and offers both actions.
    rendered = text();
    expect(rendered).toContain("Choose your GGUF model folder");
    expect(rendered).toContain("Choose folder");
    expect(rendered).toContain("Rescan this folder");

    // An empty-but-valid folder scan is distinguished from a failure.
    const rootInput = container.querySelector(
      'input[aria-label="Model root"]',
    ) as HTMLInputElement;
    await act(async () => {
      const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
      proto.set!.call(rootInput, "C:/models/empty-fixture");
      rootInput.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await act(async () => {
      [...container.querySelectorAll("button")]
        .find((button) => (button.textContent ?? "").includes("Rescan this folder"))!
        .dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    rendered = text();
    expect(rendered).toContain("No GGUF models in this folder yet");
    expect(rendered).toContain("Add .gguf files or choose a different folder");

    // A failed scan says so and keeps the recovery actions.
    handlers.set("scan_models_report", () => {
      throw new Error("Access to the folder was denied");
    });
    await act(async () => {
      [...container.querySelectorAll("button")]
        .find((button) => (button.textContent ?? "").includes("Rescan"))!
        .dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    rendered = text();
    expect(rendered).toContain("The scan could not complete");
    expect(rendered).toContain("Access to the folder was denied");
    expect(rendered).toContain("Choose folder");
  });

  it("long paths stay retrievable and Jump to Advanced respects motion and focus (S-24)", async () => {
    const longFolder = "C:/Users/fixture-user/Documents/very/deep/folder/tree/with/many/segments/models";
    handlers.set("scan_models_report", () => ({
      models: [
        {
          id: "fixture-alpha",
          name: "fixture-alpha",
          directory: longFolder,
          firstShard: `${longFolder}/fixture-alpha-00001-of-00001.gguf`,
          sizeBytes: 2048,
          shardCount: 1,
          expectedShards: 1,
          complete: true,
          quant: "Q4_K_M",
          shards: [],
          companions: [],
        },
      ],
      problems: [],
      truncated: false,
    }));
    await mount();
    const navFor = (label: string) =>
      [...container.querySelectorAll("button")].find(
        (button) => (button.textContent ?? "").trim() === label,
      );
    await act(async () => {
      navFor("Inventory")!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    const rootInput = container.querySelector(
      'input[aria-label="Model root"]',
    ) as HTMLInputElement;
    await act(async () => {
      const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
      proto.set!.call(rootInput, "C:/models/fixture-root");
      rootInput.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await act(async () => {
      [...container.querySelectorAll("button")]
        .find((button) => (button.textContent ?? "").includes("Rescan"))!
        .dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    // A successful rescan opens the loaded profile (FE-01); return to the
    // inventory to inspect the row presentation.
    await act(async () => {
      navFor("Inventory")!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();

    // The truncated folder value keeps the full path in title and aria-label,
    // and one adjacent control copies it.
    const value = container.querySelector(".path-text-value") as HTMLElement | null;
    expect(value, "the directory must render through the PathText pattern").toBeTruthy();
    expect(value!.getAttribute("title")).toBe(longFolder);
    expect(value!.getAttribute("aria-label")).toContain(longFolder);
    expect(value!.tabIndex).toBe(0);
    const copy = container.querySelector(".path-copy") as HTMLButtonElement | null;
    expect(copy, "a copy control must sit beside the truncated value").toBeTruthy();
    const writes: string[] = [];
    Object.assign(navigator, {
      clipboard: { writeText: (text: string) => (writes.push(text), Promise.resolve()) },
    });
    await act(async () => {
      copy!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    expect(writes).toContain(longFolder);
    // The row's own click opens the profile; the copy control must not
    // trigger that navigation (real packaged defect found by the S-24 probe).
    expect(text(), "copying must not leave the inventory").toContain("Logical inventory");
    expect(
      (container.querySelector(".path-copy") as HTMLButtonElement)?.textContent,
      "the copy control must report the write",
    ).toBe("Copied");
  });

  it("Jump to Advanced honours reduced motion and moves focus into the region (S-24)", async () => {
    const behaviorCalls: Array<{ behavior?: string }> = [];
    const originalScroll = Element.prototype.scrollIntoView;
    Element.prototype.scrollIntoView = function (options?: ScrollIntoViewOptions) {
      behaviorCalls.push((options ?? {}) as { behavior?: string });
    };
    const originalMatchMedia = window.matchMedia;
    window.matchMedia = ((query: string) =>
      ({
        matches: query.includes("prefers-reduced-motion"),
        media: query,
        onchange: null,
        addListener: () => undefined,
        removeListener: () => undefined,
        addEventListener: () => undefined,
        removeEventListener: () => undefined,
        dispatchEvent: () => false,
      }) as MediaQueryList) as typeof window.matchMedia;
    try {
      handlers.set("scan_models_report", () => ({
        models: [
          {
            id: "fixture-alpha",
            name: "fixture-alpha",
            directory: "C:/models/fixture-root",
            firstShard: "C:/models/fixture-root/fixture-alpha-00001-of-00001.gguf",
            sizeBytes: 2048,
            shardCount: 1,
            expectedShards: 1,
            complete: true,
            quant: "Q4_K_M",
            shards: [],
            companions: [],
          },
        ],
        problems: [],
        truncated: false,
      }));
      await mount();
      const navFor = (label: string) =>
        [...container.querySelectorAll("button")].find(
          (button) => (button.textContent ?? "").trim() === label,
        );
      // Scan to load a profile — the advanced zone lives on the profile screen.
      await act(async () => {
        navFor("Inventory")!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      });
      await settle();
      const rootInput = container.querySelector(
        'input[aria-label="Model root"]',
      ) as HTMLInputElement;
      await act(async () => {
        const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
        proto.set!.call(rootInput, "C:/models/fixture-root");
        rootInput.dispatchEvent(new Event("input", { bubbles: true }));
      });
      await act(async () => {
        [...container.querySelectorAll("button")]
          .find((button) => (button.textContent ?? "").includes("Rescan"))!
          .dispatchEvent(new MouseEvent("click", { bubbles: true }));
      });
      await settle();
      await act(async () => {
        navFor("Profile")!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      });
      await settle();
      const jump = [...container.querySelectorAll("button")].find((button) =>
        (button.textContent ?? "").includes("Jump to Advanced"),
      );
      expect(jump, "the Jump to Advanced action must exist on the profile screen").toBeTruthy();
      await act(async () => {
        jump!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      });
      await settle();
      expect(
        behaviorCalls[behaviorCalls.length - 1]?.behavior,
        "reduced motion must not force smooth scrolling",
      ).toBe("auto");
      const advanced = container.querySelector(".advanced-zone") as HTMLDetailsElement;
      expect(advanced.open, "the region must be expanded").toBe(true);
      expect((document.activeElement as HTMLElement)?.tagName).toBe("SUMMARY");
    } finally {
      Element.prototype.scrollIntoView = originalScroll;
      window.matchMedia = originalMatchMedia;
    }
  });

  it("catalog download rows do not each claim the primary action (S-24)", async () => {
    useRows([model("m1", "alpha"), model("m2", "beta")]);
    await mount();
    await openCatalogTab();
    await settle();
    const catalog = container.querySelector(".catalog-screen") ?? container;
    const primariesInCatalog = catalog.querySelectorAll("button.button.primary").length;
    expect(
      primariesInCatalog,
      "the one-primary-per-screen rule forbids a primary on every download row",
    ).toBe(0);
    const downloadButtons = [...catalog.querySelectorAll("button")].filter((button) =>
      /download|verify file/i.test(button.textContent ?? ""),
    );
    expect(downloadButtons.length).toBeGreaterThanOrEqual(2);
    for (const button of downloadButtons) {
      expect(button.className).toContain("secondary");
    }
  });

  it("typing bursts collapse into one filter request after the pause (S-25)", async () => {
    useRows([model("m1", "alpha")]);
    handlers.set("load_model_catalog", () => snapshot([model("m1", "alpha")]));
    handlers.set("fetch_model_catalog", () => snapshot([model("m1", "alpha")]));
    handlers.set("filter_catalog", () => [model("m1", "alpha")]);
    await mount();
    await openCatalogTab();
    await settle();
    const search = container.querySelector(
      '.catalog-search input, input[placeholder*="earch"]',
    ) as HTMLInputElement;
    expect(search, "the catalog search input must render").toBeTruthy();
    const filterCalls = () =>
      invokeCalls.filter((call) => call.command === "filter_catalog").length;
    const before = filterCalls();
    const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
    // Seven keystrokes inside the debounce window.
    for (const value of ["m", "mi", "mis", "mist", "mistr", "mistra", "mistral"]) {
      await act(async () => {
        proto.set!.call(search, value);
        search.dispatchEvent(new Event("input", { bubbles: true }));
      });
      await act(async () => {
        await new Promise((resolve) => setTimeout(resolve, 12));
      });
    }
    // Let the debounce fire.
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 350));
    });
    const after = filterCalls();
    expect(
      after - before,
      "a typing burst must produce exactly one filter request, not one per keystroke",
    ).toBe(1);
  });
});

describe("delayed and rejected catalog IPC (audit QD-02 I4)", () => {
  function deferred<T>() {
    let resolve!: (value: T) => void;
    let reject!: (reason?: unknown) => void;
    const promise = new Promise<T>((res, rej) => {
      resolve = res;
      reject = rej;
    });
    return { promise, resolve, reject };
  }

  function orderSelect(): HTMLSelectElement {
    const select = [...container.querySelectorAll("select")].find((candidate) =>
      [...candidate.options].some((option) => (option.textContent ?? "").includes("Smallest file")),
    );
    expect(select, "the order selector must exist").toBeTruthy();
    return select as HTMLSelectElement;
  }

  async function changeOrder(value: string) {
    await act(async () => {
      orderSelect().value = value;
      orderSelect().dispatchEvent(new Event("change", { bubbles: true }));
    });
    await settle();
  }

  it("surfaces a rejected filter application and keeps the previously listed rows", async () => {
    const rows = [model("m1", "alpha")];
    useRows(rows);
    handlers.set("load_model_catalog", () => snapshot(rows));
    handlers.set("fetch_model_catalog", () => snapshot(rows));

    await mount();
    await openCatalogTab();
    await settle();
    expect(text()).toContain("alpha");

    // The backend refuses the next filter application.
    handlers.set("filter_catalog", () => {
      throw new Error("filter refused by fixture");
    });
    await changeOrder("name");
    expect(text()).toContain("filter refused by fixture");
    expect(text(), "the previous rows stay visible").toContain("alpha");
  });

  it("keeps the newest filter result when an older application resolves late", async () => {
    const initial = [model("m1", "alpha"), model("m2", "beta")];
    handlers.set("catalog_local_models", () => initial);
    handlers.set("load_model_catalog", () => snapshot(initial));
    handlers.set("fetch_model_catalog", () => snapshot(initial));
    const filters: Array<ReturnType<typeof deferred<unknown>>> = [];
    handlers.set("filter_catalog", () => {
      const next = deferred<unknown>();
      filters.push(next);
      return next.promise;
    });

    await mount();
    await openCatalogTab();
    await settle();
    // Resolve every application dispatched so far with the full list.
    await act(async () => {
      for (const filter of filters.splice(0)) filter.resolve(initial);
    });
    await settle();
    expect(text()).toContain("alpha");
    expect(text()).toContain("beta");

    // Two further applications; the newer resolves first with a different list.
    await changeOrder("name");
    await changeOrder("likes");
    expect(filters.length, "two further filter applications must be dispatched").toBe(2);
    await act(async () => {
      filters[1]!.resolve([model("m3", "gamma")]);
    });
    await settle();
    expect(text()).toContain("gamma");
    // The older application resolves late; it must be discarded.
    await act(async () => {
      filters[0]!.resolve([model("m1", "alpha")]);
    });
    await settle();
    expect(text(), "the newest filter result must stay").toContain("gamma");
    expect(text(), "the stale filter result must be discarded").not.toContain("alpha");
  });

  it("keeps the previous hardware evidence when a refresh is rejected", async () => {
    const evidence = (value: number | string | null) => ({
      value,
      level: value === null ? "unknown" : "observed",
      source: { kind: "vendorApi", detail: "fixture" },
      observedAtMs: 1,
      notes: [],
    });
    const hardware = {
      ...runtimeSetup().hardware,
      adapters: [
        {
          adapterId: "gpu-0",
          compatibilityId: "compat-gpu-0",
          name: "GPU 0",
          vendor: "nvidia",
          driver: evidence("580.0"),
          backend: evidence("cuda"),
          dedicatedBytes: evidence(10_000_000_000),
          sharedBytes: evidence(null),
          budgetBytes: evidence(10_000_000_000),
          currentUsageBytes: evidence(0),
          availableBudgetBytes: evidence(9_000_000_000),
          reservationBytes: evidence(0),
          availableForReservationBytes: evidence(null),
          capacityObservations: [],
        },
      ],
    };
    handlers.set("load_runtime_setup", () => runtimeSetup({ hardware }));
    await mount();
    await settle();
    expect(text(), "the initial adapter evidence must render").toContain("gpu-0");

    handlers.set("detect_hardware", () => {
      throw new Error("hardware probe refused by fixture");
    });
    const refresh = [...container.querySelectorAll("button")].find((button) =>
      (button.textContent ?? "").includes("Refresh hardware"),
    );
    expect(refresh, "the Refresh hardware action must exist").toBeTruthy();
    await act(async () => {
      refresh!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await settle();
    expect(text()).toContain("hardware probe refused by fixture");
    expect(text(), "the previous adapter evidence must stay").toContain("gpu-0");
  });
});
