import { describe, expect, it } from "vitest";
import {
  normalizeProfile,
  runtimeOptionState,
  suggestedProfile,
  type GithubAsset,
  type LogicalModel,
  type ManagedRuntimeRecord,
  type RuntimeIdentity,
  type RuntimeOption,
} from "./model";

const model: LogicalModel = {
  id: "target",
  name: "LFM2.5-2.6B-Q8_0",
  directory: "C:\\models\\LFM",
  firstShard: "C:\\models\\LFM\\target.gguf",
  sizeBytes: 2_874_779_648,
  shardCount: 1,
  expectedShards: 1,
  complete: true,
  quant: "Q8_0",
  companions: [
    {
      path: "C:\\models\\LFM\\draft.gguf",
      name: "LFM2.5-DSpark-F16.gguf",
      role: "dspark",
      sizeBytes: 663_691_776,
    },
  ],
};

describe("normalizeProfile", () => {
  it("adopts the current runtime instead of the one saved with the profile", () => {
    // Bug: after installing a runtime update, opening a model whose profile was
    // saved against the old executable silently pinned the app back to it.
    const stored = {
      ...suggestedProfile(model, "C:\\llama\\llama-server.exe"),
      name: "My tuned profile",
      context: 16384,
    };
    const next = normalizeProfile(
      stored,
      model,
      "C:\\Users\\x\\AppData\\Local\\GGUF Pilot\\runtimes\\b10752\\cuda-13.3\\llama-server.exe",
    );
    expect(next.runtime).toBe(
      "C:\\Users\\x\\AppData\\Local\\GGUF Pilot\\runtimes\\b10752\\cuda-13.3\\llama-server.exe",
    );
    // Everything the user actually chose survives.
    expect(next.name).toBe("My tuned profile");
    expect(next.context).toBe(16384);
  });

  it("keeps the stored runtime when no runtime is currently selected", () => {
    const stored = { ...suggestedProfile(model, "C:\\llama\\llama-server.exe") };
    expect(normalizeProfile(stored, model, "").runtime).toBe("C:\\llama\\llama-server.exe");
  });
});

describe("suggestedProfile", () => {
  it("selects an explicit compatible DSpark companion", () => {
    const profile = suggestedProfile(model, "C:\\llama\\llama-server.exe");
    expect(profile.specType).toBe("draft-dspark");
    expect(profile.draftModel).toBe("C:\\models\\LFM\\draft.gguf");
    expect(profile.alias).toBe("lfm2-5-2-6b-q8-0");
    expect(profile.host).toBe("127.0.0.1");
    expect(profile.flashAttention).toBe("auto");
    expect(profile.cacheTypeK).toBe("f16");
    expect(profile.cachePrompt).toBe(true);
    expect(profile.continuousBatching).toBe(true);
    expect(profile.reasoning).toBe("auto");
    expect(profile.corsOrigins).toBe("localhost");
  });
});

const asset = (name: string): GithubAsset => ({ name, browserDownloadUrl: "https://example.invalid/" + name, size: 1, digest: null });
const option = (installKey: string, backend: string): RuntimeOption => ({
  id: `b10752:${installKey}`,
  label: installKey,
  backend,
  installKey,
  description: "",
  compatibility: "",
  asset: asset(`llama-b10752-bin-win-${installKey}-x64.zip`),
  companionAsset: null,
  recommended: false,
});
const cuda13 = option("cuda-13.3", "cuda");
const cuda12 = option("cuda-12.4", "cuda");
const cpu = option("cpu", "cpu");
const managedCuda13 = (tag: string): ManagedRuntimeRecord => ({
  tag,
  backend: "cuda",
  installKey: "cuda-13.3",
  runtimePath: `C:\\managed\\${tag}\\cuda-13.3\\llama-server.exe`,
  installRoot: `C:\\managed\\${tag}\\cuda-13.3`,
});
const identity = (over: Partial<RuntimeIdentity>): RuntimeIdentity => ({
  path: "C:\\llama\\llama-server.exe",
  backend: "unknown",
  cudaMajor: null,
  tag: null,
  installKey: null,
  source: "none",
  ...over,
});

describe("runtimeOptionState", () => {
  it("offers install when nothing matching is installed or active", () => {
    expect(runtimeOptionState(cuda13, "b10752", [], null, null)).toEqual({ kind: "install" });
  });

  it("marks the active managed build of the same variant as active", () => {
    const state = runtimeOptionState(cuda13, "b10752", [managedCuda13("b10752")], identity({ source: "manifest", backend: "cuda", cudaMajor: 13, tag: "b10752", installKey: "cuda-13.3" }), "10752");
    expect(state).toEqual({ kind: "active" });
  });

  it("offers an update when the active managed build of the same variant is older", () => {
    const state = runtimeOptionState(cuda13, "b10760", [managedCuda13("b10752")], identity({ source: "manifest", backend: "cuda", cudaMajor: 13, tag: "b10752", installKey: "cuda-13.3" }), "10752");
    expect(state).toEqual({ kind: "update", from: "b10752" });
  });

  it("offers to use an installed-but-inactive managed build of the same release", () => {
    const state = runtimeOptionState(cuda13, "b10752", [managedCuda13("b10752")], identity({ source: "manifest", backend: "cpu", tag: "b10752", installKey: "cpu" }), "10752");
    expect(state).toEqual({ kind: "use", runtimePath: "C:\\managed\\b10752\\cuda-13.3\\llama-server.exe" });
  });

  it("recognises a user-supplied CUDA 13 build by its DLLs and offers an update when the release is newer", () => {
    const state = runtimeOptionState(cuda13, "b10752", [], identity({ source: "dlls", backend: "cuda", cudaMajor: 13 }), "10679");
    expect(state).toEqual({ kind: "update", from: "b10679" });
  });

  it("does not treat a user-supplied CUDA 13 build as matching the CUDA 12 package", () => {
    expect(runtimeOptionState(cuda12, "b10752", [], identity({ source: "dlls", backend: "cuda", cudaMajor: 13 }), "10679")).toEqual({ kind: "install" });
  });

  it("marks a user-supplied build active when its build equals the release", () => {
    expect(runtimeOptionState(cpu, "b10752", [], identity({ source: "dlls", backend: "cpu" }), "10752")).toEqual({ kind: "active" });
  });

  it("still offers install when the active build cannot be identified", () => {
    expect(runtimeOptionState(cpu, "b10752", [], identity({ source: "none" }), "10752")).toEqual({ kind: "install" });
  });
});
