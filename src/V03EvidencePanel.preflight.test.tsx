// @vitest-environment jsdom
//
// Component tests for preflight-input staleness and adapter-selection intent
// (audit FE-06): a displayed preflight result must go stale when its inputs
// change, and a deliberate empty adapter selection must persist. The IPC
// boundary is mocked; nothing else is.
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { HardwareInfo, LaunchProfile, LogicalModel } from "./model";
import { V03EvidencePanel } from "./V03EvidencePanel";

type Handler = (args: unknown) => unknown | Promise<unknown>;

const invokeCalls: Array<{ command: string; args: Record<string, unknown> }> = [];
const handlers = new Map<string, Handler>();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (command: string, args?: unknown) => {
    invokeCalls.push({ command, args: (args ?? {}) as Record<string, unknown> });
    const handler = handlers.get(command);
    if (handler) {
      try {
        return Promise.resolve(handler(args));
      } catch (error) {
        return Promise.reject(error);
      }
    }
    return Promise.reject(new Error(`no handler for ${command}`));
  },
}));

function adapter(adapterId: string) {
  const evidence = (value: number | string) => ({
    value,
    level: "observed",
    source: null,
    observedAtMs: 1,
    notes: [],
  });
  return {
    adapterId,
    compatibilityId: `compat-${adapterId}`,
    name: adapterId,
    vendor: "amd",
    driver: evidence("1.0"),
    backend: evidence("vulkan"),
    dedicatedBytes: evidence(24_000_000_000),
    sharedBytes: evidence(8_000_000_000),
    budgetBytes: evidence(24_000_000_000),
    currentUsageBytes: evidence(0),
    availableBudgetBytes: evidence(23_000_000_000),
    reservationBytes: evidence(0),
    availableForReservationBytes: evidence(23_000_000_000),
    capacityObservations: [],
  };
}

function hardwareFixture(adapterIds: string[]): HardwareInfo {
  return {
    architecture: "x86_64",
    gpuNames: adapterIds,
    vendor: "amd",
    cudaMajor: null,
    driverVersion: "1.0",
    detectionStatus: "detected",
    recommendation: "auto",
    systemMemory: {
      totalPhysicalBytes: { value: 64_000_000_000, level: "observed", source: null, observedAtMs: 1, notes: [] },
      availablePhysicalBytes: { value: 32_000_000_000, level: "observed", source: null, observedAtMs: 1, notes: [] },
      memoryLoadPercent: { value: 50, level: "observed", source: null, observedAtMs: 1, notes: [] },
    },
    adapters: adapterIds.map(adapter),
    manualOverrides: [],
  } as unknown as HardwareInfo;
}

function evidence(value: number) {
  return { value, level: "observed", source: null, observedAtMs: 1, notes: [] };
}

const preflightResult = {
  report: {
    schema: 1,
    class: "fits",
    executionPath: "fullOffload",
    requestedContext: evidence(4096),
    nativeContext: evidence(32768),
    effectiveContext: evidence(4096),
    weightBytes: evidence(1),
    kvCacheBytes: evidence(1),
    storageRequiredBytes: evidence(1),
    diskAvailableBytes: evidence(1),
    storageVolumes: [],
    memory: {
      requiredBytes: evidence(1),
      availableBytes: evidence(2),
      policyReserveBytes: 0,
    },
    assumptions: [],
    unknowns: [],
  },
  devicePlan: [],
  launch: { runtime: null, arguments: { command: "", arguments: [], effectiveArgs: [] }, artifacts: [], effectiveContext: evidence(4096), problems: [] },
  // The production preflight returns the hardware it inspected; the fixture
  // matches the initial observation so only real changes mark staleness.
  hardware: hardwareFixture(["gpu-A", "gpu-B"]),
  selectedAdapterIds: [],
};

let container: HTMLDivElement;
let root: Root;

async function flush() {
  await act(async () => {
    await new Promise((resolve) => setTimeout(resolve, 0));
  });
}

function buttonByText(text: string): HTMLButtonElement {
  const button = Array.from(container.querySelectorAll("button")).find(
    (candidate) => candidate.textContent?.trim() === text,
  );
  if (!button) throw new Error(`button not found: ${text}`);
  return button as HTMLButtonElement;
}

function clickByText(text: string) {
  buttonByText(text).dispatchEvent(new MouseEvent("click", { bubbles: true }));
}

function adapterCheckbox(adapterId: string): HTMLInputElement {
  const label = Array.from(container.querySelectorAll("label")).find(
    (candidate) => candidate.textContent?.includes(adapterId),
  );
  const input = label?.querySelector<HTMLInputElement>('input[type="checkbox"]');
  if (!input) throw new Error(`adapter checkbox not found: ${adapterId}`);
  return input;
}

function toggleAdapter(adapterId: string) {
  adapterCheckbox(adapterId).dispatchEvent(new MouseEvent("click", { bubbles: true }));
}

function preflightCalls() {
  return invokeCalls.filter((call) => call.command === "preflight_model");
}

function preflightRequest(call: { args: unknown }): { selectedAdapterIds: string[] } {
  return (call.args as { request: { selectedAdapterIds: string[] } }).request;
}

beforeEach(() => {
  invokeCalls.length = 0;
  handlers.clear();
  container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);

  handlers.set("preflight_model", () => preflightResult);
  handlers.set("load_calibration_records", () => ({ anchors: [], models: [] }));
});

afterEach(() => {
  act(() => root.unmount());
  container.remove();
});

const model = {
  id: "fixture-model",
  logicalId: "fixture-model",
  firstShard: "C:/models/fixture-00001-of-00001.gguf",
  companions: [],
} as unknown as LogicalModel;

const profile = {
  name: "fixture",
  runtime: "C:/runtime/llama-server.exe",
  model: "C:/models/fixture-00001-of-00001.gguf",
  port: 8080,
  host: "127.0.0.1",
  context: 4096,
  batch: 512,
  ubatch: 128,
  parallel: 1,
  gpuLayers: "all",
  flashAttention: "auto",
  kvOffload: true,
} as unknown as LaunchProfile;

function render(initialHardware: HardwareInfo | null) {
  act(() => {
    root.render(
      <V03EvidencePanel
        model={model}
        profile={profile}
        serverStatus={{ running: false } as never}
        initialHardware={initialHardware}
        onRunStateChange={() => undefined}
      />,
    );
  });
}

describe("workload drafts and the production validator (FE-17)", () => {
  function renderRunning() {
    act(() => {
      root.render(
        <V03EvidencePanel
          model={model}
          profile={profile}
          serverStatus={
            {
              running: true,
              phase: "healthy",
              pid: 1,
              profileName: "fixture",
              alias: null,
              port: 8080,
              command: null,
              logPath: null,
              startedAt: 1,
              exitCode: null,
              resultClass: "unknown",
              validation: null,
              failure: null,
            } as never
          }
          initialHardware={hardwareFixture(["gpu-A"])}
          onRunStateChange={() => undefined}
        />,
      );
    });
  }

  function fieldInput(label: string): HTMLInputElement {
    const field = Array.from(container.querySelectorAll("label")).find((candidate) =>
      candidate.textContent?.includes(label),
    );
    const input = field?.querySelector<HTMLInputElement>("input");
    if (!input) throw new Error(`input not found: ${label}`);
    return input;
  }

  function setInputValue(input: HTMLInputElement, value: string) {
    const setter = Object.getOwnPropertyDescriptor(
      window.HTMLInputElement.prototype,
      "value",
    )?.set;
    setter?.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  }

  it("shows field errors, blocks dispatch, and dispatches once the draft is valid", async () => {
    handlers.set("benchmark_v2", () => ({
      manifest: { schema: 2, harnessVersion: "0.5.0", warmups: [], observations: [], terminalOutcome: null },
      summary: null,
      manifestPath: "C:/runs/x.json",
      compatibilityKey: `v2:${"c".repeat(64)}`,
      resultClass: "unknown",
      failure: null,
    }));
    renderRunning();
    await flush();

    // The tested factory seeds the panel: default values, no errors.
    expect(fieldInput("Prompt tokens").value).toBe("512");
    expect(fieldInput("Trials").value).toBe("5");

    act(() => setInputValue(fieldInput("Trials"), "2.5"));
    await flush();
    expect(container.textContent).toContain("workload.trials");
    expect(container.textContent).toContain("whole number");
    const runButton = buttonByText("Run v2 benchmark");
    expect(runButton.disabled).toBe(true);
    runButton.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    await flush();
    expect(invokeCalls.some((call) => call.command === "benchmark_v2")).toBe(false);

    act(() => setInputValue(fieldInput("Trials"), "3"));
    await flush();
    expect(container.textContent).not.toContain("whole number");
    expect(buttonByText("Run v2 benchmark").disabled).toBe(false);
    act(() => clickByText("Run v2 benchmark"));
    await flush();
    const dispatched = invokeCalls.find((call) => call.command === "benchmark_v2");
    expect(dispatched).toBeTruthy();
    const workload = (dispatched?.args.workload ?? {}) as { trials?: number };
    expect(workload.trials).toBe(3);
  });
});

describe("preflight staleness and adapter selection (FE-06)", () => {
  it("marks a preflight result stale after an adapter change and clears it on re-run", async () => {
    render(hardwareFixture(["gpu-A", "gpu-B"]));
    await flush();

    // The default selection initializes to the first adapter.
    expect(adapterCheckbox("gpu-A").checked).toBe(true);
    act(() => clickByText("Run preflight"));
    await flush();
    expect(preflightCalls()).toHaveLength(1);
    expect(preflightRequest(preflightCalls()[0]).selectedAdapterIds).toEqual(["gpu-A"]);
    expect(container.textContent).toContain("Preflight class");
    expect(container.textContent).not.toContain("Stale:");

    // Changing the adapter set makes the displayed result stale.
    act(() => toggleAdapter("gpu-B"));
    await flush();
    expect(container.textContent).toContain("Stale:");

    // Re-running preflight clears the stale marker and sends the new inputs.
    act(() => clickByText("Run preflight"));
    await flush();
    expect(preflightCalls()).toHaveLength(2);
    expect(preflightRequest(preflightCalls()[1]).selectedAdapterIds).toEqual(["gpu-A", "gpu-B"]);
    expect(container.textContent).not.toContain("Stale:");
  });

  it("discards a preflight response whose inputs changed while it ran", async () => {
    render(hardwareFixture(["gpu-A", "gpu-B"]));
    await flush();
    expect(adapterCheckbox("gpu-A").checked).toBe(true);

    // Hold the response until after an input change.
    let release: ((value: unknown) => void) | null = null;
    handlers.set(
      "preflight_model",
      () => new Promise((resolve) => {
        release = resolve;
      }),
    );

    act(() => clickByText("Run preflight"));
    await flush();
    act(() => toggleAdapter("gpu-B"));
    await flush();
    expect(release).not.toBeNull();
    await act(async () => {
      release?.(preflightResult);
      await new Promise((resolve) => setTimeout(resolve, 0));
    });

    // The obsolete response is not displayed as a current result.
    expect(container.textContent).toContain("Preflight result discarded");
    expect(container.textContent).toContain("Preflight classUnknown");
    expect(container.textContent).not.toContain("Stale:");
  });

  it("preserves a deliberate empty adapter selection across preflight and hardware refresh", async () => {
    render(hardwareFixture(["gpu-A", "gpu-B"]));
    await flush();
    expect(adapterCheckbox("gpu-A").checked).toBe(true);

    // Uncheck the final selected adapter: the empty choice must persist and
    // must not be silently repopulated.
    act(() => toggleAdapter("gpu-A"));
    await flush();
    expect(adapterCheckbox("gpu-A").checked).toBe(false);

    act(() => clickByText("Run preflight"));
    await flush();
    expect(preflightRequest(preflightCalls()[0]).selectedAdapterIds).toEqual([]);

    // A hardware refresh after the deliberate clearing keeps the selection
    // empty instead of re-selecting a default.
    render(hardwareFixture(["gpu-A", "gpu-B", "gpu-C"]));
    await flush();
    expect(adapterCheckbox("gpu-A").checked).toBe(false);
    expect(adapterCheckbox("gpu-C").checked).toBe(false);
    act(() => clickByText("Run preflight"));
    await flush();
    expect(preflightRequest(preflightCalls()[1]).selectedAdapterIds).toEqual([]);

    // The refreshed observation still counts as a new input revision for the
    // stale marker when a result was displayed.
    expect(container.textContent).toContain("Stale:");
  });
});

describe("evidence tone follows the status (FE-15)", () => {
  it("does not paint unknown results green", () => {
    render(null);
    const strongs = [...container.querySelectorAll(".evidence-status-card strong")];
    expect(strongs.length).toBeGreaterThanOrEqual(3);
    const unknowns = strongs.filter((strong) => (strong.textContent ?? "").trim() === "Unknown");
    expect(unknowns.length).toBeGreaterThanOrEqual(3);
    for (const strong of unknowns) {
      expect(strong.className).toContain("tone-pending");
    }
    // Nothing in the panel may present an unknown or empty result as green.
    for (const strong of strongs) {
      expect(strong.className).not.toContain("tone-ok");
    }
  });
});
