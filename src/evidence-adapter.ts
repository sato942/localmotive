// Evidence acquisition boundary (audit S-27.I2): every Tauri invocation the
// evidence panel needs, behind one typed interface. The panel renders and
// keeps its operation state; this adapter owns the acquisition boundary so the
// presentation layer can be exercised without a Tauri process, and the wiring
// stays in `App.tsx`.
import { invoke } from "@tauri-apps/api/core";
import type {
  ArtifactInspection,
  BenchmarkManifest,
  BenchmarkRunResult,
  HardwareInfo,
  HardwareOverride,
  LaunchProfile,
  PreflightResult,
  Workload,
} from "./model";

export interface EvidenceAdapter {
  detectHardware(): Promise<HardwareInfo>;
  inspectModelArtifact(args: {
    firstShard: string;
    companions: string[];
    hashFiles: boolean;
  }): Promise<ArtifactInspection>;
  preflightModel(args: {
    profile: LaunchProfile;
    selectedAdapterIds: string[];
    manualOverrides: HardwareOverride[];
  }): Promise<PreflightResult>;
  benchmarkV2(workload: Workload): Promise<BenchmarkRunResult>;
  cancelBenchmark(): Promise<void>;
  replayBenchmarkManifest(manifest: BenchmarkManifest): Promise<Workload>;
}

/// The real adapter used by the packaged application.
export const tauriEvidenceAdapter: EvidenceAdapter = {
  detectHardware: () => invoke("detect_hardware"),
  inspectModelArtifact: (args) => invoke("inspect_model_artifact", args),
  preflightModel: (args) => invoke("preflight_model", { request: args }),
  benchmarkV2: (workload) => invoke("benchmark_v2", { workload }),
  cancelBenchmark: () => invoke("cancel_benchmark"),
  replayBenchmarkManifest: (manifest) =>
    invoke("replay_benchmark_manifest", { manifest }),
};
