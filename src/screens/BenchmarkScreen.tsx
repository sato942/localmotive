import type { Dispatch, SetStateAction } from "react";
import {
  Activity, Gauge, TestTube2, TriangleAlert,
} from "lucide-react";
import { V03EvidencePanel } from "../V03EvidencePanel";
import type {
  BenchmarkSummary,
  HardwareInfo,
  LaunchProfile,
  LogicalModel,
  ServerStatus,
} from "../model";
import type { EvidenceRun } from "./evidence-run";
import type { EvidenceAdapter } from "../evidence-adapter";

/// Presentational contract for the benchmark screen (audit S-27.I2). The
/// screen stays mounted for the app's lifetime (audit FE-05); `App.tsx` keeps
/// the mount and visibility, this component renders the content.
export interface BenchmarkScreenProps {
  adapter?: EvidenceAdapter;
  benchmark: BenchmarkSummary | null;
  busy: string;
  evidenceRun: EvidenceRun | null;
  hardware: HardwareInfo | null;
  profile: LaunchProfile | null;
  repeats: number;
  runBenchmark: () => void;
  selected: LogicalModel | undefined;
  setEvidenceRun: Dispatch<SetStateAction<EvidenceRun | null>>;
  setRepeats: Dispatch<SetStateAction<number>>;
  setTokens: Dispatch<SetStateAction<number>>;
  status: ServerStatus;
  tokens: number;
}

export function BenchmarkScreen(props: BenchmarkScreenProps) {
  // Local alias: keeps TS narrowing inside map callbacks (the original App
  // body narrowed a const, not a property access).
  const benchmark: BenchmarkSummary | null = props.benchmark;
  return (
    <>
    <div className="section-heading">
      <div>
        <h1>Generation benchmark</h1>
        <p>One warmup, fixed deterministic workload, repeated server-reported throughput.</p>
      </div>
      <button className="button primary" onClick={props.runBenchmark} disabled={!props.status.running || props.busy === "benchmark" || props.evidenceRun !== null}>
        <Activity size={16} /> {props.busy === "benchmark" ? "Measuring…" : "Run benchmark"}
      </button>
    </div>
    {!props.status.running && <div className="warning-band"><TriangleAlert size={17} /><strong>Server required</strong><span>Start a profile before measuring it.</span></div>}
    <div className="benchmark-grid">
      <article className="machine-panel benchmark-setup">
        <div className="panel-title"><TestTube2 size={17} /><h2>Test setup</h2></div>
        <label>Forced output tokens<input type="number" min="64" max="4096" value={props.tokens} onChange={(e) => props.setTokens(Number(e.target.value))} /></label>
        <label>Measured repeats<input type="number" min="1" max="10" value={props.repeats} onChange={(e) => props.setRepeats(Number(e.target.value))} /></label>
        <dl className="spec-list">
          <div><dt>Sampling</dt><dd>GREEDY</dd></div>
          <div><dt>Seed</dt><dd>42</dd></div>
          <div><dt>Early EOS</dt><dd>IGNORED</dd></div>
          <div><dt>Warmup</dt><dd>64 TOKENS</dd></div>
        </dl>
      </article>
      <article className="machine-panel results-panel">
        <div className="panel-title"><Gauge size={17} /><h2>Measured result</h2></div>
        {benchmark ? (
          <>
            <div className="result-main"><strong>{benchmark.meanTps.toFixed(2)}</strong><span>generation tok/s mean</span></div>
            <div className="result-range"><span>Median {benchmark.medianTps.toFixed(2)}</span><span>Range {benchmark.minTps.toFixed(2)}–{benchmark.maxTps.toFixed(2)}</span></div>
            <div className="sample-bars">
              {benchmark.samples.map((sample, index) => <div key={index}><span>R{index + 1}</span><b style={{ width: `${(sample / benchmark.maxTps) * 100}%` }} /><em>{sample.toFixed(2)}</em></div>)}
            </div>
          </>
        ) : <div className="empty-result"><Activity size={28} /><p>No measurement recorded for this session.</p></div>}
      </article>
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
