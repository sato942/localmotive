// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { normalizeProfile, suggestedProfile, type LogicalModel, type RuntimeCapabilities } from "../model";
import { ProfileScreen, type ProfileScreenProps } from "./ProfileScreen";

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

const model: LogicalModel = {
  id: "fixture", name: "fixture", directory: "C:/models", firstShard: "C:/models/model.gguf",
  sizeBytes: 100, shardCount: 1, expectedShards: 1, complete: true, quant: "Q4_K_M", shards: [], companions: [],
};
const runtime: RuntimeCapabilities = {
  path: "C:/llama/llama-server.exe", version: "fixture", build: "fixture", commit: "fixture",
  helpSha256: "fixture", specTypes: ["none", "ngram-simple"], supportedFlags: ["--spec-type"],
  metrics: false, multimodal: false, fit: false,
};
function render(overrides: Partial<ProfileScreenProps> = {}) {
  const noop = () => {};
  const props: ProfileScreenProps = {
    busy: "", command: null, evidenceRun: null, extraArgsDraft: null, inspect: noop,
    profile: suggestedProfile(model, runtime.path), runtime: null, saveProfile: noop, selected: model,
    setExtraArgsDraft: noop, setNotice: noop, setProfile: noop, setRuntime: noop, setRuntimePath: noop,
    start: noop, tuning: false,
    status: { running: false, phase: "idle", pid: null, profileName: null, alias: null, port: null,
      command: null, logPath: null, startedAt: null, exitCode: null, resultClass: "unknown", validation: null, failure: null },
    ...overrides,
  };
  act(() => root.render(<ProfileScreen {...props} />));
  return [...container.querySelectorAll("select")].find((select) => select.closest("label")?.textContent?.includes("Speculative method"))!;
}

it("does not offer guessed speculative methods before runtime inspection", () => {
  const select = render();
  expect(select.disabled).toBe(true);
  expect([...select.options].map((option) => option.value)).toEqual(["none"]);
  expect(select.querySelector("small")).toBeNull();
  expect(container.textContent).toContain("Inspect the selected runtime");
});

it("preserves a saved unsupported method without offering it as verified", () => {
  const profile = { ...suggestedProfile(model, runtime.path), specType: "draft-dspark", draftModel: "C:/models/draft.gguf" };
  const select = render({ runtime, profile });
  expect(select.value).toBe("draft-dspark");
  expect([...select.options].filter((option) => !option.disabled).map((option) => option.value)).toEqual(["none", "ngram-simple"]);
  expect(select.selectedOptions[0].textContent).toContain("not verified");
});

it("renders native numeric bounds without excluding valid fractional probabilities", () => {
  render();
  const input = (label: string) => [...container.querySelectorAll("label")].find((item) => item.textContent?.startsWith(label))!.querySelector("input")!;
  expect(input("Port").min).toBe("1");
  expect(input("Port").max).toBe("65535");
  expect(input("Port").required).toBe(true);
  input("Port").value = "65536";
  expect(input("Port").checkValidity()).toBe(false);
  input("Top P").value = "0.975";
  expect(input("Top P").checkValidity()).toBe(true);
});

it.each([null, {}, [], 42])("keeps a malformed stored method recoverable: %j", (specType) => {
  const profile = normalizeProfile(JSON.parse(JSON.stringify({ specType })), model, runtime.path);
  const select = render({ profile });
  expect(container.textContent).toContain("Speculative method must be text.");
  expect(select.value).toBe("");
  expect(container.textContent).toContain("Invalid saved method");
});
