// @vitest-environment jsdom
//
// Run-ownership callback contract (FE-07): the panel must notify the
// current owner, not the callback that happened to be installed first.
// A consumer that replaces onRunStateChange mid-run sees the release
// notice; the stale callback gets no second call. The IPC boundary is
// mocked with a controllable deferred promise; nothing else is.
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { HardwareInfo, LaunchProfile, LogicalModel } from "./model";
import type { EvidenceRun } from "./screens/evidence-run";
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

function hardwareFixture(): HardwareInfo {
  const evidence = (value: number | string) => ({
    value,
    level: "observed",
    source: null,
    observedAtMs: 1,
    notes: [],
  });
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
} as never;

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

function renderPanel(notify: (state: EvidenceRun | null) => void) {
  act(() => {
    root.render(
      <V03EvidencePanel
        model={model}
        profile={profile}
        serverStatus={runningStatus}
        initialHardware={hardwareFixture()}
        onRunStateChange={notify}
      />,
    );
  });
}

async function flush() {
  await act(async () => {
    await new Promise((resolve) => setTimeout(resolve, 0));
  });
}

function clickByText(text: string) {
  const button = Array.from(container.querySelectorAll("button")).find((candidate) =>
    (candidate.textContent ?? "").includes(text),
  );
  if (!button) throw new Error(`button not found: ${text}`);
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
}

describe("run-ownership callback (FE-07)", () => {
  it("notifies the latest owner when a run is released mid-flight", async () => {
    const run = deferred<unknown>();
    handlers.set("benchmark_v2", () => run.promise);
    handlers.set("cancel_benchmark", () => undefined);
    const first = vi.fn();
    const second = vi.fn();
    renderPanel(first);
    await flush();

    act(() => clickByText("Run v2 benchmark"));
    await flush();
    // The run is active and owned by the first callback: mount released
    // null, then the run start published the owned run.
    expect(first).toHaveBeenCalledTimes(2);
    expect(first.mock.calls[0][0]).toBe(null);
    expect(first.mock.calls[1][0]).toMatchObject({ kind: "benchmark" });

    // Swap owners mid-run, then release via unmount.
    renderPanel(second);
    act(() => root.unmount());
    expect(second).toHaveBeenCalledWith(null);
    expect(first).toHaveBeenCalledTimes(2);
  });
});
