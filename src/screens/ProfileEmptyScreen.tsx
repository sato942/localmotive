import type { Dispatch, SetStateAction } from "react";
import { FolderOpen } from "lucide-react";
import type { LogicalModel } from "../model";
import type { AppView } from "./DashboardScreen";

/// Presentational contract for the profile empty state (audit S-27.I2).
export interface ProfileEmptyScreenProps {
  models: LogicalModel[];
  setView: Dispatch<SetStateAction<AppView>>;
}

export function ProfileEmptyScreen(props: ProfileEmptyScreenProps) {
  return (
    <section className="screen profile-screen">
      <div className="section-heading">
        <div>
          <h1>Launch profile</h1>
          <p>Every field becomes an explicit llama-server argument.</p>
        </div>
      </div>
      <div className="empty-state" role="status">
        <h2>{props.models.length === 0 ? "No models to profile yet" : "No model selected"}</h2>
        <p>
          {props.models.length === 0
            ? "A launch profile is built from a scanned model. Choose a folder of GGUF files first — the Inventory screen walks through it."
            : "Choose a model in the Inventory screen; its profile, runtime check and Start action appear here."}
        </p>
        <div className="empty-state-actions">
          <button
            className="button primary"
            onClick={() => props.setView("models")}
          >
            <FolderOpen size={15} /> Open Inventory
          </button>
        </div>
      </div>
    </section>
  );
}
