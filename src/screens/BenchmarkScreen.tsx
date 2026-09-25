import { V03EvidencePanel } from "../V03EvidencePanel";
import type {
  HardwareInfo,
  LaunchProfile,
  LogicalModel,
  ServerStatus,
} from "../model";
import type { EvidenceRun } from "./evidence-run";
import type { EvidenceAdapter } from "../evidence-adapter";
import type { Dispatch, SetStateAction } from "react";

/// Presentational contract for the benchmark screen (audit S-27.I2). The
/// screen stays mounted for the app's lifetime (audit FE-05); `App.tsx` keeps
/// the mount and visibility, this component renders the content. The legacy
/// benchmark runner was removed (audit MT-09/FE-05); the v2 evidence panel is
/// the only benchmark runner.
export interface BenchmarkScreenProps {
  adapter?: EvidenceAdapter;
  hardware: HardwareInfo | null;
  profile: LaunchProfile | null;
  selected: LogicalModel | undefined;
  setEvidenceRun: Dispatch<SetStateAction<EvidenceRun | null>>;
  status: ServerStatus;
}

export function BenchmarkScreen(props: BenchmarkScreenProps) {
  return (
    <>
    <div className="section-heading">
      <div>
        <h1>Generation benchmark</h1>
        <p>Measured runs with persisted evidence, calibration, and replay.</p>
      </div>
    </div>
    <V03EvidencePanel
      adapter={props.adapter}
      model={props.selected ?? null}
      profile={props.profile}
      serverStatus={props.status}
      initialHardware={props.hardware}
      onRunStateChange={props.setEvidenceRun}
    />
    </>
  );
}
