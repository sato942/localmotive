// @vitest-environment jsdom
//
// Component tests for the calibration anchor flow (audit MT-08): the panel
// must create anchors from the persisted run identity via the backend
// command, show the unique-run count, and keep the build gate closed until
// three distinct runs exist. The IPC boundary is mocked; nothing else is.
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CalibrationAnchor, CalibrationRecords, LaunchProfile, LogicalModel } from "./model";
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

const KEY = `v2:${"c".repeat(64)}`;
const MANIFEST_PATH = "C:/runs/mt08-run.json";

const runningStatus = {
  running: true,
  phase: "healthy",
  pid: 4242,
  profileName: "fixture",
  alias: null,
  port: 8080,
  command: null,
  logPath: null,
  startedAt: 1000,
  exitCode: null,
  resultClass: "unknown",
  validation: null,
  failure: null,
};

const benchmarkResult = {
  manifest: {
    schema: 2,
    harnessVersion: "0.5.0",
    warmups: [],
    observations: [],
    terminalOutcome: null,
  } as never,
  summary: {
    resultClass: "fits",
    successfulTrials: 2,
    failedTrials: 0,
    prefillTps: null,
    decodeTps: { count: 2, mean: 50, median: 50, p50: 50, p95: 51, min: 49, max: 51, standardDeviation: 1 },
    firstTokenMs: null,
    derivedTtftMs: null,
    failures: [],
  },
  manifestPath: MANIFEST_PATH,
  compatibilityKey: KEY,
  resultClass: "fits",
  failure: null,
};

function anchorFixture(sourceRunId: string, estimate: number): CalibrationAnchor {
  return {
    compatibilityKey: KEY,
    estimatedValue: estimate,
    measuredValue: 50,
    observedAtMs: 1234,
    sourceRunId,
    estimator: "manual-estimate.v1",
    snapshotSchemaVersion: "localmotive.execution-snapshot.v2",
    unknownIdentities: [],
  };
}

let container: HTMLDivElement;
let root: Root;

function clickByText(text: string) {
  const button = Array.from(container.querySelectorAll("button")).find(
    (candidate) => candidate.textContent?.trim() === text,
  );
  if (!button) throw new Error(`button not found: ${text}`);
  button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
}

function buttonByText(text: string): HTMLButtonElement {
  const button = Array.from(container.querySelectorAll("button")).find(
    (candidate) => candidate.textContent?.trim() === text,
  );
  if (!button) throw new Error(`button not found: ${text}`);
  return button as HTMLButtonElement;
}

async function flush() {
  await act(async () => {
    await new Promise((resolve) => setTimeout(resolve, 0));
  });
}

beforeEach(() => {
  invokeCalls.length = 0;
  handlers.clear();
  container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);

  handlers.set("load_calibration_records", () => ({ anchors: [], models: [] }));
  handlers.set("benchmark_v2", () => benchmarkResult);
});

afterEach(() => {
  act(() => root.unmount());
  container.remove();
});

function renderPanel() {
  const model = {
    logicalId: "fixture-model",
    firstShard: "C:/models/fixture-00001-of-00001.gguf",
    companions: [],
    shards: [],
  } as unknown as LogicalModel;
  const profile = { runtime: "C:/runtime/llama-server.exe" } as unknown as LaunchProfile;
  act(() => {
    root.render(
      <V03EvidencePanel
        model={model}
        profile={profile}
        serverStatus={runningStatus as never}
        initialHardware={null}
        onRunStateChange={() => undefined}
      />,
    );
  });
}

async function runMeasuredBenchmark() {
  renderPanel();
  await flush();
  act(() => clickByText("Run v2 benchmark"));
  await flush();
}

function setEstimate(value: string) {
  const label = Array.from(container.querySelectorAll("label")).find((candidate) =>
    candidate.textContent?.includes("Uncalibrated estimate"),
  );
  const input = label?.querySelector<HTMLInputElement>('input[type="number"]')
    ?? container.querySelector<HTMLInputElement>('input[type="number"]');
  if (!input) throw new Error("estimate input not found");
  const setter = Object.getOwnPropertyDescriptor(
    window.HTMLInputElement.prototype,
    "value",
  )?.set;
  setter?.call(input, value);
  input.dispatchEvent(new Event("input", { bubbles: true }));
}

describe("calibration anchors (MT-08)", () => {
  it("creates anchors from the persisted run and shows the unique-run count", async () => {
    let stored: CalibrationAnchor[] = [];
    handlers.set("add_benchmark_calibration_anchor", (args) => {
      const request = args as { manifestPath: string; estimatedValue: number; estimator: string };
      // Worst case: the record list contains three anchors that all share
      // one source run. The panel must still count one unique run.
      stored = [
        anchorFixture("run-1", request.estimatedValue),
        anchorFixture("run-1", request.estimatedValue + 1),
        anchorFixture("run-1", request.estimatedValue + 2),
      ];
      return { anchors: stored, models: [] };
    });

    await runMeasuredBenchmark();
    setEstimate("120");
    await flush();

    for (let click = 0; click < 3; click += 1) {
      act(() => clickByText("Add anchor"));
      await flush();
    }

    const adds = invokeCalls.filter((call) => call.command === "add_benchmark_calibration_anchor");
    expect(adds).toHaveLength(3);
    for (const call of adds) {
      expect(call.args.manifestPath).toBe(MANIFEST_PATH);
      expect(call.args.estimator).toBe("manual-estimate.v1");
      expect(call.args.estimatedValue).toBe(120);
    }
    expect(container.textContent).toContain("1 unique run");
    expect(buttonByText("Build calibration").disabled).toBe(true);
  });

  it("enables the build gate only for three distinct runs", async () => {
    const threeRuns: CalibrationRecords = {
      anchors: [
        anchorFixture("run-1", 100),
        anchorFixture("run-1", 101),
        anchorFixture("run-2", 110),
        anchorFixture("run-3", 120),
      ],
      models: [],
    };
    handlers.set("add_benchmark_calibration_anchor", () => threeRuns);
    handlers.set("build_calibration_model", () => ({
      compatibilityKey: KEY,
      factor: 1.0,
      residualStandardDeviation: 0.05,
      anchorCount: 3,
      createdAtMs: 10,
      expiresAtMs: 100,
    }));

    await runMeasuredBenchmark();
    setEstimate("100");
    await flush();
    act(() => clickByText("Add anchor"));
    await flush();

    expect(container.textContent).toContain("3 unique runs");
    const build = buttonByText("Build calibration");
    expect(build.disabled).toBe(false);
    act(() => clickByText("Build calibration"));
    await flush();

    const buildCall = invokeCalls.find((call) => call.command === "build_calibration_model");
    expect(buildCall).toBeTruthy();
    const sent = (buildCall?.args.anchors ?? []) as CalibrationAnchor[];
    expect(sent.map((anchor) => anchor.sourceRunId)).toEqual(["run-1", "run-1", "run-2", "run-3"]);
  });

  it("labels sample counts, derived TTFT and working-set scope honestly (S-12)", async () => {
    // A five-trial-style fixture: nearest-rank p95 with a small sample base
    // is the sample maximum, derived TTFT is not a streamed observation, and
    // the working set is process-lifetime CPU evidence.
    handlers.set("benchmark_v2", () => ({
      ...benchmarkResult,
      manifest: {
        ...(benchmarkResult.manifest as Record<string, unknown>),
        scopeNote: "Controlled greedy microbenchmark: one fixed prompt.",
        observations: [
          { peakProcessRssBytes: { value: 1_200_000_000 } },
          { peakProcessRssBytes: { value: 1_400_000_000 } },
        ],
      } as never,
      summary: {
        ...benchmarkResult.summary,
        decodeTps: { count: 5, mean: 50, median: 50, p50: 49, p95: 51, min: 49, max: 51, standardDeviation: 1 },
        firstTokenMs: { count: 5, mean: 120, median: 120, p50: 120, p95: 130, min: 110, max: 130, standardDeviation: 5 },
        derivedTtftMs: { count: 5, mean: 92, median: 92, p50: 92, p95: 95, min: 90, max: 95, standardDeviation: 2 },
      },
    }));
    await runMeasuredBenchmark();

    const rendered = container.textContent ?? "";
    // Decode throughput AND first-token latency each carry the sample line.
    expect(rendered.match(/n=5 · nearest-rank p95 is the sample maximum/g) ?? []).toHaveLength(2);
    expect(rendered).toContain("not an observed first token");
    expect(rendered).toContain("CPU peak working set (process lifetime, excludes GPU)");
    expect(rendered).toContain("2/2 sampled");
    expect(rendered).toContain("Controlled greedy microbenchmark: one fixed prompt.");
    expect(rendered).toContain("excludes dedicated GPU memory");
  });
});
