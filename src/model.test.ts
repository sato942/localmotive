import { describe, expect, it } from "vitest";
import {
  catalogRevision,
  downloadKey,
  downloadPercent,
  downloadReadiness,
  keepLatestRequest,
  retainOrDisposeListener,
  etaLabel,
  normalizeProfile,
  originLabel,
  rateLabel,
  runtimeOptionState,
  suggestedProfile,
  type GithubAsset,
  type LogicalModel,
  type ManagedRuntimeRecord,
  type RuntimeIdentity,
  type RuntimeOption,
} from "./model";

describe("asynchronous UI race guards", () => {
  it("accepts only the newest request result", () => {
    expect(keepLatestRequest(3, 3)).toBe(true);
    expect(keepLatestRequest(2, 3)).toBe(false);
  });

  it("immediately disposes a listener that resolves after unmount", () => {
    let stopped = 0;
    expect(retainOrDisposeListener(true, () => { stopped += 1; })).toBeNull();
    expect(stopped).toBe(1);
    const stop = () => { stopped += 1; };
    expect(retainOrDisposeListener(false, stop)).toBe(stop);
  });
});

describe("catalog revisions", () => {
  it("preserves a pinned revision and defaults only omitted values", () => {
    const base = { quant: "Q4", filename: "x.gguf", sizeBytes: 1, sha256: "a".repeat(64) };
    expect(catalogRevision({ ...base, revision: "refs/pr/7" })).toBe("refs/pr/7");
    expect(catalogRevision(base)).toBe("main");
  });
});

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


describe("HF catalog download rules", () => {
  const base = {
    destination: "C:/models",
    running: false,
    alreadyOnDisk: false,
    gated: false,
    hasToken: false,
  };

  it("refuses to start without a destination folder and says so", () => {
    const result = downloadReadiness({ ...base, destination: "" });
    expect(result.canStart).toBe(false);
    expect(result.reason).toMatch(/model folder/i);
    expect(downloadReadiness({ ...base, destination: "   " }).canStart).toBe(false);
  });

  it("refuses a second download of a file already in flight", () => {
    const result = downloadReadiness({ ...base, running: true });
    expect(result.canStart).toBe(false);
    expect(result.reason).toMatch(/already downloading/i);
  });

  it("lets the backend verify and reuse a file that is already on disk", () => {
    // A same-named local file may be stale or corrupt. The backend checks size
    // and a Hugging Face SHA-256 ETag instead of trusting the filename alone.
    expect(downloadReadiness({ ...base, alreadyOnDisk: true }).canStart).toBe(true);
  });

  it("blocks a gated repo until a token exists, then allows it", () => {
    // Gated repos return 401/403 without a token; catching it here gives a
    // better message than letting the download fail after it starts.
    const blocked = downloadReadiness({ ...base, gated: true, hasToken: false });
    expect(blocked.canStart).toBe(false);
    expect(blocked.reason).toMatch(/token/i);
    expect(downloadReadiness({ ...base, gated: true, hasToken: true }).canStart).toBe(true);
  });

  it("allows a normal download and gives no reason text", () => {
    const result = downloadReadiness(base);
    expect(result.canStart).toBe(true);
    expect(result.reason).toBe("");
  });

  it("builds a stable key per repo and file", () => {
    expect(downloadKey("unsloth/Qwen3.8-27B-GGUF", "a.gguf")).toBe(
      "unsloth/Qwen3.8-27B-GGUF/a.gguf",
    );
    expect(downloadKey("a/b", "x.gguf")).not.toBe(downloadKey("a/b", "y.gguf"));
  });
});

describe("download progress display", () => {
  it("clamps the percentage instead of overflowing the bar", () => {
    expect(downloadPercent(0, 100)).toBe(0);
    expect(downloadPercent(50, 100)).toBe(50);
    expect(downloadPercent(100, 100)).toBe(100);
    expect(downloadPercent(500, 100)).toBe(100);
    expect(downloadPercent(-5, 100)).toBe(0);
  });

  it("returns zero rather than NaN when the total is unknown", () => {
    // A NaN width silently breaks the progress bar, so guard every bad input.
    expect(downloadPercent(10, 0)).toBe(0);
    expect(downloadPercent(10, Number.NaN)).toBe(0);
    expect(downloadPercent(Number.NaN, 100)).toBe(0);
    expect(downloadPercent(10, Number.POSITIVE_INFINITY)).toBe(0);
  });

  it("formats the remaining time in units a person reads", () => {
    expect(etaLabel(0, 1000, 100)).toBe("10s left");
    expect(etaLabel(0, 100_000, 100)).toBe("16m 40s left");
    expect(etaLabel(0, 10_000_000, 1000)).toBe("2h 46m left");
  });

  it("shows no estimate when there is nothing to estimate from", () => {
    expect(etaLabel(0, 1000, 0)).toBe("");
    expect(etaLabel(1000, 1000, 100)).toBe("");
    expect(etaLabel(0, 0, 100)).toBe("");
  });

  it("labels transfer rate only when it is known", () => {
    expect(rateLabel(0)).toBe("");
    expect(rateLabel(1024 ** 3)).toBe("1.00 GiB/s");
  });
});

describe("catalog origin honesty", () => {
  it("marks a cached list as possibly stale rather than live", () => {
    // Showing an offline cache as live would misrepresent the catalog.
    expect(originLabel("cache").tone).toBe("warn");
    expect(originLabel("cache").label).toMatch(/OFFLINE/);
    expect(originLabel("network").tone).toBe("ok");
    expect(originLabel("not-modified").tone).toBe("ok");
  });

  it("always includes words, never colour alone", () => {
    for (const origin of ["network", "not-modified", "cache", "bundled"] as const) {
      expect(originLabel(origin).label.trim().length).toBeGreaterThan(3);
    }
  });
});
