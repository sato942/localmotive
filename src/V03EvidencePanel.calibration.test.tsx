// @vitest-environment jsdom
// Component regression for the reduced evidence panel. The panel keeps
// benchmark, preflight, and replay controls while retired controls stay absent.
// The IPC boundary is mocked; nothing else is.
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { LaunchProfile, LogicalModel } from "./model";
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
  handlers.clear();
  container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);

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

describe("reduced evidence panel", () => {
  it("exposes benchmark, preflight, and replay without retired controls", async () => {
    renderPanel();
    await flush();

    for (const retained of ["Run v2 benchmark", "Run preflight", "Replay manifest"]) {
      expect(buttonByText(retained), `${retained} must remain available`).toBeTruthy();
    }

    const buttons = Array.from(container.querySelectorAll("button"), (button) => button.textContent?.trim() ?? "");
    for (const retired of [
      "Run quality suite",
      "Rank session results",
      "Add anchor",
      "Build calibration",
      "Apply",
      "Clear local history",
      "Validate import",
      "Mark verified",
      "Flag",
      "Reject",
      "Choose export file",
    ]) {
      expect(buttons, `${retired} must be removed`).not.toContain(retired);
    }
    expect(container.querySelector("textarea.evidence-json-input")).toBeNull();
    expect(container.textContent).not.toMatch(/Quality and Pareto ranking|Local calibration and external evidence|Privacy-reviewed local export/);
  });
});

describe("retained evidence metrics", () => {
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
