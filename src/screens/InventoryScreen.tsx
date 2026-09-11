import type { Dispatch, SetStateAction } from "react";
import { invoke } from "@tauri-apps/api/core";
import { CircleStop, FolderOpen, HardDrive, RefreshCw, TriangleAlert } from "lucide-react";
import { PathText } from "./PathText";
import {
  bytesLabel,
  type LogicalModel,
  type ScanReport,
} from "../model";

/// Presentational contract for the logical inventory screen (audit S-27.I2).
/// Acquisition (scanning, picking folders, loading profiles) stays in
/// `App.tsx`; the component renders and reports.
export interface InventoryScreenProps {
  busy: string;
  chooseModelFolder: () => void;
  invalidCount: number;
  lastScan: ScanReport | null;
  loadProfile: (model: LogicalModel) => void;
  modelRoot: string;
  models: LogicalModel[];
  scan: () => void;
  scanFailed: boolean;
  selectedId: string;
  setModelRoot: Dispatch<SetStateAction<string>>;
  totalBytes: number;
}

export function InventoryScreen(props: InventoryScreenProps) {
  const {
    busy,
    chooseModelFolder,
    invalidCount,
    lastScan,
    loadProfile,
    modelRoot,
    models,
    scan,
    scanFailed,
    selectedId,
    setModelRoot,
    totalBytes,
  } = props;
  return (
    <section className="screen inventory-screen">
      <div className="section-heading">
        <div>
          <h1>Logical inventory</h1>
          <p>Split shards grouped; companions kept separate from targets.</p>
        </div>
        <button className="button secondary" onClick={scan} disabled={busy === "scan"}>
          <RefreshCw size={16} className={busy === "scan" ? "spin" : ""} /> Rescan
        </button>
        {busy === "scan" && (
          <button className="button danger" onClick={() => void invoke("cancel_scan")}>
            <CircleStop size={15} /> Cancel scan
          </button>
        )}
      </div>
      <div className="path-bar">
        <HardDrive size={16} />
        <input value={modelRoot} onChange={(event) => setModelRoot(event.target.value)} aria-label="Model root" placeholder="Choose a folder containing GGUF files" />
        <button className="path-action" onClick={chooseModelFolder}><FolderOpen size={15} /> Choose</button>
        <span>{models.length} targets · {bytesLabel(totalBytes)}</span>
      </div>
      {models.length === 0 ? (
        <div className="empty-state" role="status">
          <h2>
            {scanFailed
              ? "The scan could not complete"
              : lastScan
                ? "No GGUF models in this folder yet"
                : "Choose your GGUF model folder"}
          </h2>
          <p>
            {scanFailed
              ? "Fix access to the folder or pick another one, then scan again. The previous inventory was cleared to avoid showing stale targets."
              : lastScan
                ? `The folder was scanned (${lastScan.problems.length} diagnostic${lastScan.problems.length === 1 ? "" : "s"}) and contains no GGUF model shards. Add .gguf files or choose a different folder.`
                : "Localmotive reads GGUF files directly from a folder on this PC. Nothing is uploaded or moved."}
          </p>
          <div className="empty-state-actions">
            <button className="button secondary" onClick={chooseModelFolder}>
              <FolderOpen size={15} /> Choose folder
            </button>
            <button className="button primary" onClick={scan} disabled={busy === "scan" || !modelRoot.trim()}>
              <RefreshCw size={15} /> Rescan this folder
            </button>
          </div>
        </div>
      ) : null}
      <table className="inventory-table" aria-label="Model inventory">
        <thead>
          <tr>
            <th scope="col">Target</th><th scope="col">Quant</th><th scope="col">Footprint</th><th scope="col">Shards</th><th scope="col">Companions</th><th scope="col">Status</th>
          </tr>
        </thead>
        <tbody>
          {models.map((model) => (
            <tr
              className={selectedId === model.id ? "selected" : ""}
              key={model.id}
              onClick={() => loadProfile(model)}
            >
              <td className="model-cell">
                <button
                  type="button"
                  className="row-target"
                  onClick={() => loadProfile(model)}
                >
                  <strong>{model.name}</strong>
                </button>
                <small>
                  <PathText value={model.directory} label="Model folder" />
                </small>
              </td>
              <td>{model.quant}</td>
              <td className="numeric">{bytesLabel(model.sizeBytes)}</td>
              <td className="numeric">{model.shardCount}/{model.expectedShards}</td>
              <td className="companion-stack">
                {model.companions.length ? (() => {
                  const counts = new Map<string, number>();
                  model.companions.forEach((c) => counts.set(c.role, (counts.get(c.role) ?? 0) + 1));
                  return [...counts].map(([role, count]) => <i key={role}>{role.toUpperCase()}{count > 1 ? ` ×${count}` : ""}</i>);
                })() : <small>None</small>}
              </td>
              <td className={model.complete ? "state-tag good" : "state-tag warning"}>
                {model.complete ? "READY" : "INCOMPLETE"}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      {invalidCount > 0 && (
        <div className="warning-band"><TriangleAlert size={17} /><strong>{invalidCount} target blocked</strong><span>Missing shards must be restored before launch.</span></div>
      )}
    </section>
  );
}
