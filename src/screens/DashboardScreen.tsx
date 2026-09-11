import type { Dispatch, SetStateAction } from "react";
import {
  Box, CircleStop, Play, SquareTerminal,
} from "lucide-react";
import type {
  BenchmarkSummary,
  LaunchProfile,
  LogicalModel,
  RuntimeCapabilities,
  ServerStatus,
} from "../model";
import { EvidenceRun } from "./evidence-run";

/// Presentational contract for the control (dashboard) screen (audit S-27.I2).
/// Acquisition and persistence stay in `App.tsx`; the component renders only.
export type AppView =
  | "dashboard"
  | "models"
  | "catalog"
  | "runtime"
  | "profile"
  | "tune"
  | "benchmark"
  | "about";

export interface DashboardScreenProps {
  benchmark: BenchmarkSummary | null;
  busy: string;
  evidenceRun: EvidenceRun | null;
  log: string;
  openExternal: (url: string) => void;
  profile: LaunchProfile | null;
  runtime: RuntimeCapabilities | null;
  selected: LogicalModel | undefined;
  setView: Dispatch<SetStateAction<AppView>>;
  start: () => void;
  status: ServerStatus;
  stop: () => void;
  tuning: boolean;
}

export function DashboardScreen(props: DashboardScreenProps) {
  return (
    <section className="screen dashboard-screen">
      <div className="section-heading">
        <div>
          <h1>Server control</h1>
          <p>One supervised process. Exact profile. No hidden defaults.</p>
        </div>
        <div className="actions">
          {props.status.running || props.status.phase === "starting" ? (
            <>
              {props.status.running && (
                <button className="button secondary" onClick={() => props.openExternal(`http://127.0.0.1:${props.status.port}`)}>
                  <SquareTerminal size={16} /> Open chat
                </button>
              )}
              <button className="button danger" onClick={props.stop} disabled={props.busy === "stop"}>
                <CircleStop size={16} /> {props.status.phase === "starting" ? "Cancel start" : "Stop server"}
              </button>
            </>
          ) : (
            <button className="button primary" onClick={props.start} disabled={!props.profile || !props.profile.runtime || props.busy === "start" || !props.selected?.complete || props.evidenceRun !== null || props.tuning}>
              <Play size={16} fill="currentColor" /> Start profile
            </button>
          )}
        </div>
      </div>

      <div className="instrument-strip">
        <div className={props.status.running ? "instrument primary-readout running" : "instrument primary-readout"}>
          <span className="instrument-label">PROCESS</span>
          <strong>{props.status.running ? "RUNNING" : props.status.phase === "starting" ? "STARTING" : "STANDBY"}</strong>
          <small>{props.status.running ? `PID ${props.status.pid}` : props.status.phase === "starting" ? "Waiting for readiness; Stop stays available" : "No owned child process"}</small>
        </div>
        <div className="instrument">
          <span className="instrument-label">ENDPOINT</span>
          <strong>{props.status.port ? `:${props.status.port}` : "—"}</strong>
          <small>{props.status.alias ?? "No alias assigned"}</small>
        </div>
        <div className="instrument">
          <span className="instrument-label">STRATEGY</span>
          {/* Running identity comes from the server snapshot; the
              editable draft only describes a not-yet-started profile
              (audit FE-16). */}
          <strong>{(props.status.running ? props.status.specType : props.profile?.specType)?.replace("draft-", "").toUpperCase() ?? "NONE"}</strong>
          <small>{props.status.running ? (props.status.companionLinked ? "Companion linked" : "Target only") : props.profile?.draftModel ? "Companion linked" : "Target only"}</small>
        </div>
        <div className="instrument">
          <span className="instrument-label">LAST TEST</span>
          <strong>{props.benchmark ? props.benchmark.meanTps.toFixed(1) : "—"}</strong>
          <small>{props.benchmark ? "generation tok/s mean" : "Not measured"}</small>
        </div>
      </div>

      <div className="control-grid">
        <article className="machine-panel current-profile">
          <div className="panel-title">
            <Box size={17} />
            <h2>Loaded profile</h2>
            <span className={props.selected?.complete ? "state-tag good" : "state-tag warning"}>
              {props.selected?.complete ? "SHARDS COMPLETE" : "SHARDS INCOMPLETE"}
            </span>
          </div>
          <div className="profile-identity">
            <strong>{props.profile?.name ?? "No profile"}</strong>
            <p>{props.selected?.firstShard ?? "Select a model from inventory"}</p>
          </div>
          <dl className="spec-list">
            <div><dt>Context</dt><dd>{props.profile?.context.toLocaleString() ?? "—"}</dd></div>
            <div><dt>GPU layers</dt><dd>{props.profile?.gpuLayers ?? "—"}</dd></div>
            <div><dt>Batch / uBatch</dt><dd>{props.profile ? `${props.profile.batch} / ${props.profile.ubatch}` : "—"}</dd></div>
            <div><dt>Draft depth</dt><dd>{props.profile?.specType !== "none" ? props.profile?.draftMax : "OFF"}</dd></div>
          </dl>
          <button className="text-button" onClick={() => props.setView("profile")}>Edit exact launch profile →</button>
        </article>

        <article className="machine-panel terminal-panel">
          <div className="panel-title">
            <SquareTerminal size={17} />
            <h2>Server log</h2>
            <span className="log-path">{props.status.logPath ?? "buffer offline"}</span>
          </div>
          {props.status.running ? (
            <pre>{props.log}</pre>
          ) : props.log ? (
            /* FE-16: after a stop or an unexpected exit the well keeps the
               last bounded output — evidence must not disappear with the
               process that produced it. */
            <div className="log-retained">
              <p className="log-retained-note"><strong>Server is stopped.</strong> The last bounded output remains visible until the next start.</p>
              <pre>{props.log}</pre>
            </div>
          ) : (
            <div className="log-empty"><SquareTerminal size={26} /><strong>Server is stopped</strong><span>Start this profile to stream llama-server output here.</span></div>
          )}
        </article>
      </div>
    </section>
  );
}
