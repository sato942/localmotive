// @vitest-environment jsdom
//
// F9-05: v0.4.1 profile/settings recovery through the application's own load
// path. The fixture is scripts/sandbox/canary-settings.json - the same file the
// lifecycle sandbox seeds into a real v0.4.1 install - derived from the released
// v0.4.1 persistence keys (`git show v0.4.1:src/App.tsx`) and profile shape
// (`git show v0.4.1:src/model.ts`). This test proves the current build READS
// those values back; the sandbox leg proves a real upgraded install does.
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
// The fixture the lifecycle sandbox seeds into a real v0.4.1 install
// (scripts/sandbox/canary-settings.json): importing it keeps this test and the
// sandbox leg reading the same released-schema records.
import fixtureSource from "../scripts/sandbox/canary-settings.json";

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
    if (command === "cloud_credential_status") {
      return Promise.resolve({ provider: "openrouter", configured: false, masked: "" });
    }
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
    managedRuntimes: [],
    ...over,
  };
}

function fixture() {
  return fixtureSource as {
    schema: string;
    keys: Record<string, string>;
    expect: { settings: Record<string, string>; profile: Record<string, string | number> };
  };
}

const scannedModel = {
  id: "fixture/v041-legacy-model",
  name: "v041-legacy",
  directory: "C:/fixtures/v041/models",
  firstShard: "C:/fixtures/v041/models/legacy-Q4_K_M.gguf",
  sizeBytes: 4_000_000_000,
  shardCount: 1,
  expectedShards: 1,
  complete: true,
  quant: "Q4_K_M",
  shards: [],
  companions: [],
};

let container: HTMLDivElement;
let root: Root | null = null;
let App: (typeof import("./App"))["default"];

beforeEach(async () => {
  localStorage.clear();
  (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {};
  (Element.prototype as unknown as { scrollTo: (options?: unknown) => void }).scrollTo = () => {};
  container = document.createElement("div");
  document.body.appendChild(container);
  invokeCalls.length = 0;
  handlers.clear();
  handlers.set("scan_models", () => [scannedModel]);
  // The application reads its settings when the module loads: seed the released
  // records first, then import a fresh module instance.
  const seeded = fixture();
  for (const [key, value] of Object.entries(seeded.keys)) {
    localStorage.setItem(key, value);
  }
  vi.resetModules();
  App = (await import("./App")).default;
  root = createRoot(container);
});

afterEach(async () => {
  if (root) {
    await act(async () => {
      root!.unmount();
    });
  }
  container.remove();
  vi.restoreAllMocks();
});

async function settle() {
  await act(async () => {
    for (let index = 0; index < 12; index += 1) {
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

function inputByLabel(label: string) {
  const field = [...container.querySelectorAll("label")].find(
    (candidate) => (candidate.textContent ?? "").startsWith(label),
  );
  return field?.querySelector("input") as HTMLInputElement | undefined;
}

it("recovers the v0.4.1 model root and uses it for the scan", async () => {
  const seeded = fixture();
  await act(async () => {
    root!.render(<App />);
  });
  await settle();
  const scans = invokeCalls.filter((call) => call.command === "scan_models_report");
  expect(scans.length, "mounting with a recovered root must scan it").toBeGreaterThan(0);
  expect(
    (scans[0].args as { root?: string }).root,
    "the scan must use the recovered model root",
  ).toBe(seeded.keys["localmotive:model-root"]);
  await click(navButton("Inventory"), "Inventory navigation must exist");
  const rootInput = container.querySelector('input[aria-label="Model root"]') as HTMLInputElement | null;
  expect(rootInput?.value, "the model root field must show the recovered path").toBe(
    seeded.keys["localmotive:model-root"],
  );
  expect(text()).toContain("v041-legacy");
});

it("recovers the stored v0.4.1 profile values for the scanned model", async () => {
  const seeded = fixture();
  await act(async () => {
    root!.render(<App />);
  });
  await settle();
  await click(navButton("Inventory"), "Inventory navigation must exist");
  // The user opens the scanned model; the app must load the STORED v0.4.1
  // record for it rather than a suggested profile.
  const row = [...container.querySelectorAll("button")].find((button) =>
    (button.textContent ?? "").includes("v041-legacy"),
  );
  await click(row, "the scanned model row must exist");
  for (let attempt = 0; attempt < 12 && !text().includes("Launch profile"); attempt += 1) {
    await settle();
  }
  expect(text(), "the stored profile must load for the scanned model").toContain("Launch profile");
  expect(inputByLabel("Port")?.value, "the recovered port").toBe("8123");
  expect(inputByLabel("Context tokens")?.value, "the recovered context").toBe("6144");
  expect(inputByLabel("Seed")?.value, "the recovered seed").toBe("424242");
  expect(inputByLabel("Temperature")?.value, "the recovered temperature").toBe("0.73");
  const nameValue = [...container.querySelectorAll("input")].some(
    (input) => (input as HTMLInputElement).value === seeded.expect.profile.name,
  );
  expect(nameValue, "the recovered profile name must render").toBe(true);
  // Nothing was quarantined: the app parsed both records successfully.
  const quarantined = Object.keys(localStorage).filter((key) =>
    key.startsWith("localmotive:quarantine:"),
  );
  expect(quarantined, "recovered records must not be quarantined").toEqual([]);
  expect(
    localStorage.getItem("localmotive:profile:fixture/v041-legacy-model"),
    "the stored profile record must survive the load",
  ).toBe(seeded.keys["localmotive:profile:fixture/v041-legacy-model"]);
});

it("keeps working when the recovered settings are lost entirely", async () => {
  // Loss must be observable as a fallback (an empty root), not a crash: the
  // sandbox assertion is what fails when values are lost, and this proves the
  // application's behavior in that case is a defined default.
  localStorage.clear();
  await act(async () => {
    root!.unmount();
  });
  root = null;
  container.remove();
  container = document.createElement("div");
  document.body.appendChild(container);
  vi.resetModules();
  const FreshApp = (await import("./App")).default;
  const freshRoot = createRoot(container);
  await act(async () => {
    freshRoot.render(<FreshApp />);
  });
  await settle();
  const scans = invokeCalls.filter((call) => call.command === "scan_models_report");
  expect(scans, "an empty root must not trigger a scan").toEqual([]);
  expect(navButton("Control"), "the app must still render").toBeTruthy();
  await act(async () => {
    freshRoot.unmount();
  });
});
