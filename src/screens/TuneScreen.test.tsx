// @vitest-environment jsdom
//
// Contract tests for the extracted Tune screen (audit S-27.I2): the component
// is presentational, so these tests hand it fixture props and assert the
// rendered consequences a user would notice. They pin the parts the
// extraction could silently break: the disclosure radio pair (S-20), the
// readiness gating on the Auto-tune action, and the blocker line.
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { TuneScreen, type TuneScreenProps } from "./TuneScreen";
import { suggestedProfile, type ServerStatus } from "../model";

let container: HTMLDivElement;
let root: Root;

beforeEach(() => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);
});

afterEach(() => {
  act(() => root.unmount());
  container.remove();
  vi.unstubAllGlobals();
});

const idleStatus: ServerStatus = {
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
} as ServerStatus;

function props(overrides: Partial<TuneScreenProps> = {}): TuneScreenProps {
  const noop = () => {};
  return {
    adoptTunedProfile: noop,
    bestLive: null,
    briefDisclosure: "full",
    busy: "",
    canTune: true,
    cancelTuning: noop,
    changeDisclosure: noop,
    chooseCloudModel: noop,
    cloudCheck: "",
    cloudModel: "",
    cloudModels: [],
    credential: null,
    disclosureSections: [],
    forgetKey: noop,
    gguf: null,
    hardware: null,
    keyDraft: "",
    onProviderTabKey: noop,
    openExternal: noop,
    openRouterLogin: noop,
    probeCloud: noop,
    provider: null,
    providerId: "openrouter",
    providers: [],
    runtime: null,
    runtimeIdentity: null,
    runtimePath: "",
    saveKey: noop,
    selected: undefined,
    setKeyDraft: noop,
    setTuneContext: noop,
    setTuneRepeats: noop,
    setTuneTokens: noop,
    setTuneTrials: noop,
    startTuning: noop,
    status: idleStatus,
    switchProvider: noop,
    trialsForDisplay: [],
    tuneBlocker: "Add a provider key first.",
    tuneContext: 8192,
    tuneInputError: null,
    tuneLogRef: { current: null },
    tuneProgress: null,
    tuneRepeats: 2,
    tuneReport: null,
    tuneReportOrigin: null,
    tuneRun: null,
    tuneTokens: 256,
    tuneTrials: 6,
    tuning: false,
    ...overrides,
  };
}

describe("TuneScreen presentation contract", () => {
  it("keeps an input error visible while a prior report is displayed", () => {
    const selected = {
      id: "fixture", name: "fixture", directory: "C:/models", firstShard: "C:/models/fixture.gguf",
      sizeBytes: 24, shardCount: 1, expectedShards: 1, complete: true, quant: "F16", shards: [], companions: [],
    };
    const trial = { index: 0, changes: {}, rationale: "Fixture report", meanTps: 1, medianTps: 1, error: null, command: "fixture" };
    const error = "AI trials must be a whole number between 1 and 12.";
    act(() => root.render(<TuneScreen {...props({
      selected, canTune: false, tuneTrials: 13, tuneInputError: error, bestLive: trial, trialsForDisplay: [trial],
      tuneReport: {
        baselineTps: 1, bestTps: 1, bestIndex: 0, bestProfile: suggestedProfile(selected, "C:/runtime/llama-server.exe"),
        trials: [trial], stoppedReason: "Fixture prior report.",
      },
    })} />));
    expect(container.textContent).toContain("Fixture prior report.");
    expect(container.querySelector('[role="status"]')?.textContent).toBe(error);
  });

  it("reflects the persisted disclosure choice in the radio pair", () => {
    act(() => root.render(<TuneScreen {...props({ briefDisclosure: "minimal" })} />));
    const radios = Array.from(
      container.querySelectorAll<HTMLInputElement>('input[name="tune-disclosure"]'),
    );
    expect(radios.length).toBe(2);
    const checkedIndex = radios.findIndex((r) => r.checked);
    expect(checkedIndex).toBe(1);
    const checkedLabel = radios[1].closest("label")?.textContent ?? "";
    expect(checkedLabel).toContain("Minimal");
  });

  it("keeps the Auto-tune action gated while readiness is unmet", () => {
    act(() => root.render(<TuneScreen {...props({ canTune: false })} />));
    const buttons = Array.from(container.querySelectorAll("button"));
    const autotune = buttons.find((b) => (b.textContent ?? "").includes("Auto-tune"));
    expect(autotune).toBeTruthy();
    expect(autotune!.disabled).toBe(true);
  });

  it("shows the blocker line when not ready to tune", () => {
    act(() => root.render(<TuneScreen {...props()} />));
    expect(container.textContent ?? "").toContain("Add a provider key first.");
  });
});
