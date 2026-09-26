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
  detectHardware: () => invoke<HardwareInfo>("detect_hardware"),
  inspectModelArtifact: (args) => invoke<ArtifactInspection>("inspect_model_artifact", args),
  preflightModel: (args) => invoke<PreflightResult>("preflight_model", { request: args }),
  benchmarkV2: (workload) => invoke<BenchmarkRunResult>("benchmark_v2", { workload }),
  cancelBenchmark: () => invoke<void>("cancel_benchmark"),
  replayBenchmarkManifest: (manifest) =>
    invoke<Workload>("replay_benchmark_manifest", { manifest }),
};
