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

import { TuneScreen, trialChosenLabel, trialOutcomeLabel, type TuneScreenProps } from "./TuneScreen";
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
    applyTuningTrial: noop,
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
    setUseAdvisor: noop,
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
    useAdvisor: false,
    ...overrides,
  };
}

describe("TuneScreen presentation contract", () => {
  it("states the same-prompt limit instead of claiming confirmation", () => {
    // P1-28 (MT-01): the final check re-measures on the fixed harness
    // prompt. The UI must present it as a same-prompt re-measurement, never
    // as an independent confirmation.
    const selected = {
      id: "fixture", name: "fixture", directory: "C:/models", firstShard: "C:/models/fixture.gguf",
      sizeBytes: 24, shardCount: 1, expectedShards: 1, complete: true, quant: "F16", shards: [], companions: [],
    };
    const trial = { index: 0, changes: {}, rationale: "Fixture report", meanTps: 1, medianTps: 1, error: null, command: "fixture" };
    const report = {
      baselineTps: 100, bestTps: 130, bestIndex: 0, bestProfile: suggestedProfile(selected, "C:/runtime/llama-server.exe"),
      trials: [trial], stoppedReason: "Fixture verification.",
      finalVerification: { baselineTps: 100, winnerTps: 130, requiredImprovement: 0.03, confirmed: true },
    };
    act(() => root.render(<TuneScreen {...props({ selected, bestLive: trial, trialsForDisplay: [trial], tuneReport: report })} />));
    const text = container.textContent ?? "";
    expect(text).toContain("harness prompt");
    expect(text).toContain("same-prompt re-measurement");
    expect(text).not.toContain("confirmed.");
  });
  it("keeps an input error visible while a prior report is displayed", () => {
    const selected = {
      id: "fixture", name: "fixture", directory: "C:/models", firstShard: "C:/models/fixture.gguf",
      sizeBytes: 24, shardCount: 1, expectedShards: 1, complete: true, quant: "F16", shards: [], companions: [],
    };
    const trial = { index: 0, changes: {}, rationale: "Fixture report", meanTps: 1, medianTps: 1, error: null, command: "fixture" };
    const error = "Search trials must be a whole number between 1 and 12.";
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

  it("offers the advisor as an explicit opt-in that stays off by default", () => {
    act(() => root.render(<TuneScreen {...props()} />));
    const checkbox = container.querySelector<HTMLInputElement>('input[type="checkbox"]');
    expect(checkbox?.checked).toBe(false);
    expect(container.textContent ?? "").toContain("one extra try");
    expect(container.textContent ?? "").toContain("Best measured in this session");
  });

  it("shows history-table badges and one apply control per measured row", () => {
    const onApply = vi.fn();
    const grid = {
      index: 1, changes: { flashAttention: "on" }, rationale: "Local grid", meanTps: 10,
      medianTps: 10, error: null, command: "c", effectiveContext: 8192, stdDev: 0.1,
      outcome: "ok", chosen: "grid", timestampMs: 1_700_000_000_000, configHash: "ab",
    } as const;
    const failed = {
      index: 2, changes: { batch: 512 }, rationale: "Local grid", meanTps: null,
      medianTps: null, error: "CUDA out of memory", command: "", effectiveContext: null,
      stdDev: null, outcome: "oom", chosen: "grid", timestampMs: 0, configHash: "cd",
    } as const;
    act(() => root.render(<TuneScreen {...props({ trialsForDisplay: [grid, failed], applyTuningTrial: onApply })} />));
    const text = container.textContent ?? "";
    expect(text).toContain("GRID");
    expect(text).toContain("OUT OF MEMORY");
    const applyButtons = Array.from(container.querySelectorAll("button")).filter((b) =>
      (b.textContent ?? "").includes("Apply this trial"),
    );
    expect(applyButtons.length).toBe(1);
    act(() => applyButtons[0].click());
    expect(onApply).toHaveBeenCalledTimes(1);
    expect(onApply.mock.calls[0][0]).toMatchObject({ index: 1 });
  });

  it("labels every trial outcome and choice with words", () => {
    expect(trialOutcomeLabel("ok")).toBe("OK");
    expect(trialOutcomeLabel("oom")).toBe("OUT OF MEMORY");
    expect(trialOutcomeLabel("launch-fail")).toBe("LAUNCH FAILED");
    expect(trialOutcomeLabel("cancelled")).toBe("CANCELLED");
    expect(trialOutcomeLabel("context-short")).toBe("CONTEXT SHORT");
    expect(trialOutcomeLabel(undefined)).toBe("UNKNOWN");
    expect(trialChosenLabel("baseline")).toBe("BASELINE");
    expect(trialChosenLabel("grid")).toBe("GRID");
    expect(trialChosenLabel("nudge")).toBe("NUDGE");
    expect(trialChosenLabel("confirm")).toBe("CONFIRM");
    expect(trialChosenLabel("advisor")).toBe("ADVISOR");
    expect(trialChosenLabel(undefined)).toBe("UNRECORDED");
  });
});

describe("TuneScreen bounded numeric input", () => {
  it("keeps incomplete typing visible and reports the bounds error as text", async () => {
    const { useState } = await import("react");
    const { tuningWorkloadError } = await import("../model");
    function Stateful() {
      const [trials, setTrials] = useState(6);
      const [tokens, setTokens] = useState(256);
      const [repeats, setRepeats] = useState(2);
      const error = tuningWorkloadError({ targetContext: 8192, maxTrials: trials, tokens, repeats });
      return (
        <TuneScreen
          {...props({
            tuneTrials: trials,
            setTuneTrials: setTrials,
            tuneTokens: tokens,
            setTuneTokens: setTokens,
            tuneRepeats: repeats,
            setTuneRepeats: setRepeats,
            tuneInputError: error,
          })}
        />
      );
    }
    act(() => {
      root.render(<Stateful />);
    });
    const trialsInput = Array.from(container.querySelectorAll("label")).find((label) =>
      (label.textContent ?? "").includes("Search trials"),
    )?.querySelector("input") as HTMLInputElement;
    expect(trialsInput.value).toBe("6");
    const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!;
    act(() => {
      proto.set!.call(trialsInput, "-");
      trialsInput.dispatchEvent(new Event("input", { bubbles: true }));
    });
    // The lone "-" stays visible instead of being eaten; the bounds error
    // reports the problem as text.
    expect(trialsInput.value).toBe("-");
    expect(container.querySelector('[role="status"]')?.textContent).toBe(
      "Search trials must be a whole number between 1 and 12.",
    );
    act(() => {
      proto.set!.call(trialsInput, "");
      trialsInput.dispatchEvent(new Event("input", { bubbles: true }));
    });
    expect(trialsInput.value).toBe("");
    act(() => {
      proto.set!.call(trialsInput, "0");
      trialsInput.dispatchEvent(new Event("input", { bubbles: true }));
    });
    // An explicit zero stays a visible zero — distinct from blank.
    expect(trialsInput.value).toBe("0");
  });
});
