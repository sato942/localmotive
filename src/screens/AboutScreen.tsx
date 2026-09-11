// Extracted from App.tsx (audit S-27 I2): the About screen is presentational
// only — it receives the facts and an external-link opener as props, so its
// rendering cannot acquire data or persistence of its own.
import { FolderOpen, Info, Link2, MonitorCog, ShieldCheck } from "lucide-react";
import {
  bytesLabel,
  type AboutInfo,
  type HardwareInfo,
  type LogicalModel,
  type RuntimeIdentity,
} from "../model";

export function AboutScreen({
  about,
  models,
  totalBytes,
  runtime,
  runtimeIdentity,
  hardware,
  modelRoot,
  onOpenExternal,
}: {
  about: AboutInfo | null;
  models: LogicalModel[];
  totalBytes: number;
  runtime: { build: string } | null;
  runtimeIdentity: RuntimeIdentity | null;
  hardware: HardwareInfo | null;
  modelRoot: string;
  onOpenExternal: (url: string) => void;
}) {
  return (
<section className="screen about-screen">
  <div className="section-heading">
    <div>
      <h1>About</h1>
      <p>What this build is, where it keeps things, and what it is made of.</p>
    </div>
    <div className="actions">
      <button className="button secondary" onClick={() => onOpenExternal(about?.repository ?? "https://github.com/sato942/localmotive")}><Link2 size={15} /> Project on GitHub</button>
    </div>
  </div>

  <div className="about-layout">
    <article className="machine-panel about-identity">
      <div className="about-mark">LM</div>
      <div>
        <strong>{about?.name ?? "Localmotive"}</strong>
        <span>Version {about?.version ?? "—"}</span>
        <p>A Windows control plane for local GGUF inference: it manages official llama.cpp runtimes, discovers models, builds exact launch profiles, supervises the server, measures throughput, and tunes settings with a cloud advisor.</p>
      </div>
    </article>

    <article className="machine-panel">
      <div className="panel-title"><Info size={17} /><h2>Build</h2></div>
      <dl className="spec-list">
        <div><dt>Version</dt><dd>{about?.version ?? "—"}</dd></div>
        <div><dt>Bundle identifier</dt><dd>{about?.identifier ?? "—"}</dd></div>
        <div><dt>Platform</dt><dd>{about?.os.toUpperCase() ?? "—"}</dd></div>
        <div><dt>Framework</dt><dd>TAURI {about?.tauriVersion ?? "2"} · RUST · REACT</dd></div>
        <div><dt>Licence</dt><dd>{about?.license || "MIT"}</dd></div>
      </dl>
    </article>

    <article className="machine-panel">
      <div className="panel-title"><MonitorCog size={17} /><h2>This machine</h2></div>
      <dl className="spec-list">
        <div><dt>Runtime in use</dt><dd>{runtime ? `LLAMA.CPP B${runtime.build}` : "NONE"}</dd></div>
        <div><dt>Backend</dt><dd>{runtimeIdentity && runtimeIdentity.backend !== "unknown" ? `${runtimeIdentity.backend.toUpperCase()}${runtimeIdentity.cudaMajor ? ` ${runtimeIdentity.cudaMajor}` : ""}` : "—"}</dd></div>
        <div><dt>Accelerator</dt><dd>{hardware?.gpuNames[0]?.toUpperCase() ?? hardware?.vendor.toUpperCase() ?? "—"}</dd></div>
        <div><dt>Models found</dt><dd>{models.length} ({bytesLabel(totalBytes)})</dd></div>
      </dl>
    </article>

    <article className="machine-panel about-paths">
      <div className="panel-title"><FolderOpen size={17} /><h2>Where things live</h2></div>
      <dl className="runtime-facts">
        <div><dt>Model folder</dt><dd>{modelRoot || "Not selected"}</dd></div>
        <div><dt>Managed runtimes</dt><dd>{about?.runtimeRoot || "—"}</dd></div>
        <div><dt>Server logs</dt><dd>{about?.logDir || "—"}</dd></div>
        <div><dt>Cloud keys</dt><dd>Windows Credential Manager, service “Localmotive” — never in settings or logs</dd></div>
        <div><dt>Settings</dt><dd>Browser local storage in this app (model folder, runtime, profiles, tuning reports)</dd></div>
      </dl>
    </article>

    <article className="machine-panel about-credits">
      <div className="panel-title"><ShieldCheck size={17} /><h2>Built on</h2></div>
      <div className="credit-list">
        <button className="credit" onClick={() => onOpenExternal("https://github.com/ggml-org/llama.cpp")}><strong>llama.cpp</strong><span>ggml-org · MIT — the inference engine and every managed runtime binary</span></button>
        <button className="credit" onClick={() => onOpenExternal("https://tauri.app")}><strong>Tauri {about?.tauriVersion ?? "2"}</strong><span>Apache-2.0 / MIT — desktop shell</span></button>
        <button className="credit" onClick={() => onOpenExternal("https://react.dev")}><strong>React + TypeScript + Vite</strong><span>MIT — interface</span></button>
        <button className="credit" onClick={() => onOpenExternal("https://lucide.dev")}><strong>Lucide</strong><span>ISC — icons</span></button>
      </div>
      <p className="group-note about-note">Localmotive is not affiliated with ggml-org. Runtime binaries are downloaded directly from official llama.cpp GitHub releases and verified against their published size and SHA-256 before use.</p>
    </article>
  </div>
  </section>
  );
}
