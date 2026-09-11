// Evidence acquisition boundary (audit S-27.I2): every Tauri invocation the
// evidence panel needs, behind one typed interface. The panel renders and
// keeps its operation state; this adapter owns the acquisition and
// persistence boundary so the presentation layer can be exercised without a
// Tauri process, and the wiring stays in `App.tsx`.
import { invoke } from "@tauri-apps/api/core";
import type {
  ArtifactInspection,
  BenchmarkManifest,
  BenchmarkRunResult,
  BenchmarkSummaryV2,
  CalibratedEstimate,
  CalibrationAnchor,
  CalibrationModel,
  CalibrationRecords,
  CandidateEvidence,
  ExternalEvidenceBundle,
  ExternalEvidenceState,
  HardwareInfo,
  HardwareOverride,
  LaunchProfile,
  ObjectiveWeights,
  PreflightResult,
  QualitySuiteResult,
  RankedCandidate,
  RecommendationConstraints,
  ShareBundle,
  Workload,
} from "./model";

export interface EvidenceAdapter {
  loadCalibrationRecords(compatibilityKey: string): Promise<CalibrationRecords>;
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
  runQualitySuite(): Promise<QualitySuiteResult>;
  joinQualityCandidate(args: {
    id: string;
    manifest: BenchmarkManifest;
    resultClass: string;
    quality: QualitySuiteResult | null;
  }): Promise<CandidateEvidence>;
  rankCandidates(args: {
    candidates: CandidateEvidence[];
    constraints: RecommendationConstraints | null;
    weights: ObjectiveWeights;
  }): Promise<RankedCandidate[]>;
  addBenchmarkCalibrationAnchor(args: {
    manifestPath: string;
    estimatedValue: number;
    estimator: string;
  }): Promise<CalibrationRecords>;
  buildCalibrationModel(args: {
    anchors: CalibrationAnchor[];
    createdAtMs: number;
    ttlMs: number;
  }): Promise<CalibrationModel>;
  storeCalibrationModel(model: CalibrationModel): Promise<CalibrationRecords>;
  applyCalibrationModel(args: {
    model: CalibrationModel;
    compatibilityKey: string;
    estimatedValue: number;
    nowMs: number;
  }): Promise<CalibratedEstimate>;
  clearCalibrationHistory(): Promise<number>;
  importExternalEvidence(bundle: unknown): Promise<ExternalEvidenceBundle>;
  reviewExternalEvidence(args: {
    bundle: ExternalEvidenceBundle;
    state: ExternalEvidenceState;
    confirmed: boolean;
  }): Promise<ExternalEvidenceBundle>;
  buildShareExport(args: {
    manifest: BenchmarkManifest;
    summary: BenchmarkSummaryV2 | null;
    quality: QualitySuiteResult | null;
    compatibilityKey: string;
    createdAtMs: number;
    confirmed: boolean;
  }): Promise<ShareBundle>;
  writeShareExport(args: {
    path: string;
    bundle: ShareBundle;
    confirmed: boolean;
  }): Promise<string>;
}

/// The real adapter used by the packaged application.
export const tauriEvidenceAdapter: EvidenceAdapter = {
  loadCalibrationRecords: (compatibilityKey) =>
    invoke("load_calibration_records", { compatibilityKey }),
  detectHardware: () => invoke("detect_hardware"),
  inspectModelArtifact: (args) => invoke("inspect_model_artifact", args),
  preflightModel: (args) => invoke("preflight_model", args),
  benchmarkV2: (workload) => invoke("benchmark_v2", { workload }),
  cancelBenchmark: () => invoke("cancel_benchmark"),
  replayBenchmarkManifest: (manifest) =>
    invoke("replay_benchmark_manifest", { manifest }),
  runQualitySuite: () => invoke("run_quality_suite"),
  joinQualityCandidate: (args) => invoke("join_quality_candidate", args),
  rankCandidates: (args) => invoke("rank_candidates", args),
  addBenchmarkCalibrationAnchor: (args) =>
    invoke("add_benchmark_calibration_anchor", args),
  buildCalibrationModel: (args) => invoke("build_calibration_model", args),
  storeCalibrationModel: (model) => invoke("store_calibration_model", { model }),
  applyCalibrationModel: (args) => invoke("apply_calibration_model", args),
  clearCalibrationHistory: () => invoke("clear_calibration_history", {}),
  importExternalEvidence: (bundle) =>
    invoke("import_external_evidence", { bundle }),
  reviewExternalEvidence: (args) => invoke("review_external_evidence", args),
  buildShareExport: (args) => invoke("build_share_export", args),
  writeShareExport: (args) => invoke("write_share_export", args),
};
