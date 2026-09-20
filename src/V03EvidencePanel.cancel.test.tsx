// @vitest-environment jsdom
//
// Cancellation lifecycle through the real panel actions (audit FE-07 V1/V2/V3):
// a void acknowledgement is truthful, a rejected cancellation keeps the run
// and its affordance, a "no benchmark is running" race surfaces distinctly,
// and only the original run's terminal outcome releases ownership. The IPC
// boundary is mocked with controllable deferred promises; nothing else is.
// Numeric editing checks keep incomplete workloads away from that boundary.
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { HardwareInfo, LaunchProfile, LogicalModel } from "./model";
import { V03EvidencePanel } from "./V03EvidencePanel";

type Handler = (args: unknown) => unknown | Promise<unknown>;

const handlers = new Map<string, Handler>();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (command: string, args?: unknown) => {
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

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

function evidence(value: number | string) {
  return { value, level: "observed", source: null, observedAtMs: 1, notes: [] };
}

function hardwareFixture(): HardwareInfo {
  return {
    architecture: "x86_64",
    gpuNames: ["gpu-A"],
    vendor: "amd",
    cudaMajor: null,
    driverVersion: "1.0",
    detectionStatus: "detected",
    recommendation: "auto",
    systemMemory: {
      totalPhysicalBytes: evidence(64_000_000_000),
      availablePhysicalBytes: evidence(32_000_000_000),
      memoryLoadPercent: evidence(50),
    },
    adapters: [
      {
        adapterId: "gpu-A",
        compatibilityId: "compat-A",
        name: "gpu-A",
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
      },
    ],
    manualOverrides: [],
  } as unknown as HardwareInfo;
}

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

const runningStatus = {
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
};

function metricStats(mean: number) {
  return {
    count: 5,
    mean,
    median: mean,
    p50: mean,
    p95: mean + 1,
    min: mean - 1,
    max: mean + 1,
    standardDeviation: 0.5,
  };
}

const benchmarkResult = {
  manifest: { schema: 2, harnessVersion: "0.5.0", warmups: [], observations: [], terminalOutcome: null },
  summary: {
    resultClass: "measured",
    successfulTrials: 5,
    failedTrials: 0,
    prefillTps: null,
    decodeTps: metricStats(42),
    firstTokenMs: null,
    derivedTtftMs: null,
    failures: [],
  },
  manifestPath: "C:/runs/x.json",
  compatibilityKey: `v2:${"c".repeat(64)}`,
  resultClass: "measured",
  failure: null,
};

let container: HTMLDivElement;
let root: Root;

beforeEach(() => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  handlers.clear();
  container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);

});

afterEach(() => {
  act(() => root.unmount());
  container.remove();
  vi.unstubAllGlobals();
});

function render() {
  act(() => {
    root.render(
      <V03EvidencePanel
        model={model}
        profile={profile}
        serverStatus={runningStatus as never}
        initialHardware={hardwareFixture()}
        onRunStateChange={() => undefined}
      />,
    );
  });
}

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

function runButton(): HTMLButtonElement {
  const button = Array.from(container.querySelectorAll("button")).find((candidate) =>
    (candidate.textContent ?? "").includes("Run v2 benchmark") ||
    (candidate.textContent ?? "").includes("Benchmarking…"),
  );
  if (!button) throw new Error("run button not found");
  return button as HTMLButtonElement;
}

function text() {
  return container.textContent ?? "";
}

async function setWorkloadInput(label: string, value: string) {
  const input = container.querySelector<HTMLInputElement>(`input[aria-label="${label}"]`);
  if (!input) throw new Error(`workload input not found: ${label}`);
  await act(async () => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
  await flush();
  return input;
}

describe("benchmark workload editing", () => {
  it.each(["Benchmark prompt tokens", "Benchmark generated tokens", "Benchmark warmups", "Benchmark trials"])("keeps a cleared %s field blank", async (label) => {
    render();
    await flush();
    expect(runButton().disabled, "the running-server fixture must be ready before the edit").toBe(false);
    const input = await setWorkloadInput(label, "");
    expect(input.value).toBe("");
    expect(input.checkValidity()).toBe(false);
    expect(runButton().disabled).toBe(true);
  });

  it("does not treat a blank warmup as permission to skip warmup", async () => {
    const benchmark = vi.fn(() => { throw new Error("Fixture boundary: no inference run."); });
    handlers.set("benchmark_v2", benchmark);
    render();
    await flush();
    expect(runButton().disabled).toBe(false);
    await setWorkloadInput("Benchmark warmups", "");
    act(() => clickByText("Run v2 benchmark"));
    await flush();
    expect(benchmark).not.toHaveBeenCalled();
    expect(runButton().disabled).toBe(true);
    expect(container.querySelector('[aria-label="Workload validation errors"]')?.textContent).toContain("workload.warmups");
  });

  it.each([
    ["Benchmark prompt tokens", "promptTokens", "1"],
    ["Benchmark prompt tokens", "promptTokens", "1048576"],
    ["Benchmark generated tokens", "generationTokens", "1"],
    ["Benchmark generated tokens", "generationTokens", "65536"],
    ["Benchmark warmups", "warmups", "0"],
    ["Benchmark warmups", "warmups", "10"],
    ["Benchmark trials", "trials", "1"],
    ["Benchmark trials", "trials", "100"],
  ])("dispatches corrected %s (%s=%s) without clamping", async (label, field, value) => {
    const benchmark = vi.fn(() => { throw new Error("Fixture boundary: no inference run."); });
    handlers.set("benchmark_v2", benchmark);
    render();
    await flush();
    await setWorkloadInput(label, "");
    expect(runButton().disabled).toBe(true);
    const input = await setWorkloadInput(label, value);
    expect(input.value).toBe(value);
    expect(input.checkValidity()).toBe(true);
    expect(runButton().disabled).toBe(false);
    expect(container.querySelector('[aria-label="Workload validation errors"]')).toBeNull();
    act(() => clickByText("Run v2 benchmark"));
    await flush();
    expect(benchmark).toHaveBeenCalledTimes(1);
    expect(benchmark).toHaveBeenCalledWith({ workload: expect.objectContaining({ [field]: Number(value) }) });
  });

  it.each([
    ["Benchmark prompt tokens", "0"], ["Benchmark prompt tokens", "1048577"], ["Benchmark prompt tokens", "1.5"],
    ["Benchmark generated tokens", "0"], ["Benchmark generated tokens", "65537"], ["Benchmark generated tokens", "1.5"],
    ["Benchmark warmups", "-1"], ["Benchmark warmups", "11"], ["Benchmark warmups", "0.5"],
    ["Benchmark trials", "0"], ["Benchmark trials", "101"], ["Benchmark trials", "1.5"],
  ])("does not dispatch invalid %s=%s", async (label, value) => {
    const benchmark = vi.fn(() => { throw new Error("Fixture boundary: no inference run."); });
    handlers.set("benchmark_v2", benchmark);
    render();
    await flush();
    expect(runButton().disabled).toBe(false);
    const input = await setWorkloadInput(label, value);
    expect(input.value).toBe(value);
    expect(input.checkValidity()).toBe(false);
    act(() => clickByText("Run v2 benchmark"));
    await flush();
    expect(benchmark).not.toHaveBeenCalled();
    expect(runButton().disabled).toBe(true);
  });
});

describe("benchmark cancellation lifecycle (audit FE-07)", () => {
  it("recognizes a void acknowledgement as success and keeps the run owned (V1/V2)", async () => {
    const run = deferred<unknown>();
    handlers.set("benchmark_v2", () => run.promise);
    handlers.set("cancel_benchmark", () => undefined); // a Rust unit response
    render();
    await flush();

    act(() => clickByText("Run v2 benchmark"));
    await flush();
    expect(runButton().textContent).toContain("Benchmarking…");
    expect(runButton().disabled).toBe(true);
    expect(buttonByText("Cancel").disabled).toBe(false);

    act(() => clickByText("Cancel"));
    await flush();
    // A void success is acknowledged truthfully, and the acknowledgement does
    // not release the run: the lifecycle stays owned by the original promise.
    expect(text()).toContain("Benchmark cancellation requested; the run ends after the current attempt.");
    expect(runButton().textContent).toContain("Benchmarking…");
    expect(runButton().disabled).toBe(true);
    expect(buttonByText("Replay manifest").disabled).toBe(true);

    // Only the original run's terminal outcome releases ownership.
    await act(async () => {
      run.resolve(benchmarkResult);
    });
    await flush();
    expect(text()).toContain("The measured benchmark manifest was saved.");
    expect(runButton().textContent).toContain("Run v2 benchmark");
    expect(runButton().disabled).toBe(false);
  });

  it("keeps the run and the cancel affordance when cancellation is rejected (V2)", async () => {
    const run = deferred<unknown>();
    handlers.set("benchmark_v2", () => run.promise);
    handlers.set("cancel_benchmark", () => {
      throw new Error("Benchmark state is unavailable");
    });
    render();
    await flush();

    act(() => clickByText("Run v2 benchmark"));
    await flush();
    act(() => clickByText("Cancel"));
    await flush();
    // The rejection is surfaced; the run record is retained and the cancel
    // affordance remains available instead of pretending termination.
    expect(text()).toContain("Benchmark state is unavailable");
    expect(runButton().textContent).toContain("Benchmarking…");
    expect(buttonByText("Cancel").disabled).toBe(false);

    await act(async () => {
      run.resolve(benchmarkResult);
    });
    await flush();
    expect(text()).toContain("The measured benchmark manifest was saved.");
    expect(runButton().disabled).toBe(false);
  });

  it("surfaces a no-active-benchmark race distinctly and keeps the completed result (V2)", async () => {
    const run = deferred<unknown>();
    const cancel = deferred<unknown>();
    handlers.set("benchmark_v2", () => run.promise);
    handlers.set("cancel_benchmark", () => cancel.promise);
    render();
    await flush();

    act(() => clickByText("Run v2 benchmark"));
    await flush();
    act(() => clickByText("Cancel"));
    await flush();
    // The original run finishes before the cancel lands (the race the backend
    // rejects with "No benchmark is running").
    await act(async () => {
      run.resolve(benchmarkResult);
    });
    await flush();
    await act(async () => {
      cancel.reject(new Error("No benchmark is running"));
    });
    await flush();
    // The distinct race message is visible; the completed result is retained
    // (the card keeps its measured statistics) and the run control is free.
    expect(text()).toContain("No benchmark is running");
    expect(text()).toContain("5/5 successful trials");
    expect(text()).not.toContain("No v2 result");
    expect(runButton().disabled).toBe(false);
  });

  it("releases ownership when the original run fails after acknowledgement (V3)", async () => {
    const run = deferred<unknown>();
    handlers.set("benchmark_v2", () => run.promise);
    handlers.set("cancel_benchmark", () => undefined);
    render();
    await flush();

    act(() => clickByText("Run v2 benchmark"));
    await flush();
    act(() => clickByText("Cancel"));
    await flush();
    expect(text()).toContain("Benchmark cancellation requested; the run ends after the current attempt.");

    await act(async () => {
      run.reject(new Error("health timed out"));
    });
    await flush();
    expect(text()).toContain("health timed out");
    expect(runButton().textContent).toContain("Run v2 benchmark");
    expect(runButton().disabled).toBe(false);
    // No result was recorded for the failed run.
    expect(text()).toContain("No v2 result");
  });
});
