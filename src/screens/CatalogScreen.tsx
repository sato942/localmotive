import type { Dispatch, SetStateAction } from "react";
import {
  BadgeCheck, CircleStop, Database, Download, ExternalLink, FolderOpen, HardDrive, KeyRound, RefreshCw,
} from "lucide-react";
import { PathText } from "./PathText";
import {
  activeDownloadJob,
  bytesLabel,
  catalogBuildFit,
  catalogRevision,
  type CatalogFile,
  type CatalogModel,
  type CatalogQuery,
  type CatalogSnapshot,
  downloadPercent,
  downloadReadiness,
  downloadKey,
  etaLabel,
  newestDownloadJob,
  originLabel,
  rateLabel,
  type DownloadEvent,
  type DownloadJob,
  type FitBudget,
  type TokenStatus,
} from "../model";

/// Presentational contract for the HF catalog screen (audit S-27.I2).
/// Acquisition (catalog fetch, downloads, token storage) stays in `App.tsx`.
export interface CatalogScreenProps {
  cancelCatalogDownload: (job: DownloadJob) => void;
  catalogArchitecture: string;
  catalogAuthor: string;
  catalogBusy: boolean;
  catalogFitEnabled: boolean;
  catalogHideGated: boolean;
  catalogLicense: string;
  catalogMaxGiB: number;
  catalogPipeline: string;
  catalogQuant: string;
  catalogSearch: string;
  catalogTag: string;
  hfTokenDraft: string;
  modelRoot: string;
  catalogArchitectures: string[];
  catalogAuthors: string[];
  catalogLicenses: string[];
  catalogPipelines: string[];
  catalogQuants: string[];
  catalogTags: string[];
  catalogFiles: Record<string, string>;
  catalogFitBudget: FitBudget | null;
  catalogFitPerMille: number;
  catalogRows: CatalogModel[];
  catalogSnapshot: CatalogSnapshot | null;
  catalogSort: CatalogQuery["sort"];
  chooseModelFolder: () => void;
  downloads: Record<string, DownloadEvent>;
  downloadJobs: DownloadJob[];
  hfToken: TokenStatus;
  inventoryHasFile: (filename: string) => boolean;
  loadModelCatalog: () => void;
  openExternal: (url: string) => void;
  removeHfToken: () => void;
  saveHfToken: () => void;
  setCatalogArchitecture: Dispatch<SetStateAction<string>>;
  setCatalogAuthor: Dispatch<SetStateAction<string>>;
  setCatalogFiles: Dispatch<SetStateAction<Record<string, string>>>;
  setCatalogFitEnabled: Dispatch<SetStateAction<boolean>>;
  setCatalogFitPerMille: Dispatch<SetStateAction<number>>;
  setCatalogHideGated: Dispatch<SetStateAction<boolean>>;
  setCatalogLicense: Dispatch<SetStateAction<string>>;
  setCatalogMaxGiB: Dispatch<SetStateAction<number>>;
  setCatalogPipeline: Dispatch<SetStateAction<string>>;
  setCatalogQuant: Dispatch<SetStateAction<string>>;
  setCatalogSearch: Dispatch<SetStateAction<string>>;
  setCatalogSort: Dispatch<SetStateAction<CatalogQuery["sort"]>>;
  setCatalogTag: Dispatch<SetStateAction<string>>;
  setHfTokenDraft: Dispatch<SetStateAction<string>>;
  setModelRoot: Dispatch<SetStateAction<string>>;
  startCatalogDownload: (model: CatalogModel, file: CatalogFile) => void;
}

export function CatalogScreen(props: CatalogScreenProps) {
  const {
    cancelCatalogDownload,
    catalogArchitecture,
    catalogArchitectures,
    catalogAuthor,
    catalogAuthors,
    catalogBusy,
    catalogFiles,
    catalogFitBudget,
    catalogFitEnabled,
    catalogFitPerMille,
    catalogHideGated,
    catalogLicense,
    catalogLicenses,
    catalogMaxGiB,
    catalogPipeline,
    catalogPipelines,
    catalogQuant,
    catalogQuants,
    catalogRows,
    catalogSearch,
    catalogSnapshot,
    catalogSort,
    catalogTag,
    catalogTags,
    chooseModelFolder,
    downloadJobs,
    downloads,
    hfToken,
    hfTokenDraft,
    inventoryHasFile,
    loadModelCatalog,
    modelRoot,
    openExternal,
    removeHfToken,
    saveHfToken,
    setCatalogArchitecture,
    setCatalogAuthor,
    setCatalogFiles,
    setCatalogFitEnabled,
    setCatalogFitPerMille,
    setCatalogHideGated,
    setCatalogLicense,
    setCatalogMaxGiB,
    setCatalogPipeline,
    setCatalogQuant,
    setCatalogSearch,
    setCatalogSort,
    setCatalogTag,
    setHfTokenDraft,
    setModelRoot,
    startCatalogDownload,
  } = props;
  return (
    <section className="screen catalog-screen">
      <div className="section-heading">
        <div>
          <h1>HF Catalog</h1>
          <p>A developer-curated list. Model bytes travel directly from Hugging Face to this machine.</p>
        </div>
        <div className="actions">
          {catalogSnapshot && (() => {
            const origin = originLabel(catalogSnapshot.origin);
            return <span className={origin.tone === "ok" ? "state-tag good" : "state-tag warning"}>{origin.label}</span>;
          })()}
          <button className="button secondary" onClick={loadModelCatalog} disabled={catalogBusy}>
            <RefreshCw size={16} className={catalogBusy ? "spin" : ""} /> Refresh list
          </button>
        </div>
      </div>

      <div className="path-bar catalog-destination">
        <HardDrive size={16} />
        <input value={modelRoot} onChange={(event) => setModelRoot(event.target.value)} aria-label="Download destination" placeholder="Choose where downloaded GGUF files should go" />
        <button className="path-action" onClick={chooseModelFolder}><FolderOpen size={15} /> Choose</button>
        <span>DIRECT TO MODEL FOLDER</span>
      </div>

      <div className="catalog-filters machine-panel" aria-label="Catalog filters">
        <label className="catalog-search">Search<input value={catalogSearch} onChange={(event) => setCatalogSearch(event.target.value)} placeholder="Family, publisher, tag, repository…" /></label>
        <label>Use<select value={catalogTag} onChange={(event) => setCatalogTag(event.target.value)}><option value="">Any use</option>{catalogTags.map((tag) => <option key={tag}>{tag}</option>)}</select></label>
        <label>Quant<select value={catalogQuant} onChange={(event) => setCatalogQuant(event.target.value)}><option value="">Any quant</option>{catalogQuants.map((quant) => <option key={quant}>{quant}</option>)}</select></label>
        <label>Author<select value={catalogAuthor} onChange={(event) => setCatalogAuthor(event.target.value)}><option value="">Any author</option>{catalogAuthors.map((author) => <option key={author}>{author}</option>)}</select></label>
        <label>Licence<select value={catalogLicense} onChange={(event) => setCatalogLicense(event.target.value)}><option value="">Any licence</option>{catalogLicenses.map((licence) => <option key={licence}>{licence}</option>)}</select></label>
        <label>Pipeline<select value={catalogPipeline} onChange={(event) => setCatalogPipeline(event.target.value)}><option value="">Any pipeline</option>{catalogPipelines.map((pipeline) => <option key={pipeline}>{pipeline}</option>)}</select></label>
        <label>Architecture<select value={catalogArchitecture} onChange={(event) => setCatalogArchitecture(event.target.value)}><option value="">Any architecture</option>{catalogArchitectures.map((architecture) => <option key={architecture}>{architecture}</option>)}</select></label>
        <label>Available under<select value={catalogMaxGiB} onChange={(event) => setCatalogMaxGiB(Number(event.target.value))}><option value={0}>Any size</option>{[2, 4, 8, 16, 32, 64].map((size) => <option key={size} value={size}>≤ {size} GiB</option>)}</select></label>
        <label>Order<select value={catalogSort} onChange={(event) => setCatalogSort(event.target.value as CatalogQuery["sort"])}><option value="downloads">Downloads</option><option value="likes">Likes</option><option value="name">Name</option><option value="size">Smallest file</option></select></label>
        <label className="toggle-line catalog-toggle"><input type="checkbox" checked={catalogHideGated} onChange={(event) => setCatalogHideGated(event.target.checked)} /> Hide gated</label>
        <label className="toggle-line catalog-toggle"><input type="checkbox" checked={catalogFitEnabled} onChange={(event) => setCatalogFitEnabled(event.target.checked)} /> Hardware fit{
          catalogFitBudget && catalogFitBudget.budgetBytes > 0
            ? ` · filter keeps models whose smallest build is ≤ ${bytesLabel(Math.floor((catalogFitBudget.budgetBytes * catalogFitPerMille) / 1000))} on ${catalogFitBudget.source}`
            : " · budget unknown"
        }</label>
        {catalogFitEnabled && (
          <label>Fit budget<select value={catalogFitPerMille} onChange={(event) => setCatalogFitPerMille(Number(event.target.value))}>{[250, 500, 750, 1000].map((perMille) => <option key={perMille} value={perMille}>{perMille / 10}% of budget</option>)}</select></label>
        )}
      </div>

      <div className="catalog-layout">
        <div className="catalog-results">
          <div className="catalog-result-count">
            <strong>{catalogRows.length}</strong>
            <span>of {catalogSnapshot?.catalog.models.length ?? 0} curated models</span>
            {catalogSnapshot && <small>LIST UPDATED {catalogSnapshot.catalog.updated || "UNKNOWN"}</small>}
            {catalogSnapshot?.lastSuccessSecs && (
              <small>LAST SUCCESS {new Date(Number(catalogSnapshot.lastSuccessSecs) * 1000).toLocaleString()}</small>
            )}
            {typeof catalogSnapshot?.cooldownRemainingMinutes === "number" && (
              <small>COOLDOWN {catalogSnapshot.cooldownRemainingMinutes} MIN LEFT</small>
            )}
          </div>
          {catalogBusy && !catalogSnapshot && <div className="catalog-empty machine-panel"><RefreshCw size={24} className="spin" /><strong>Fetching curated catalog</strong></div>}
          {!catalogBusy && catalogSnapshot && catalogRows.length === 0 && <div className="catalog-empty machine-panel"><Database size={24} /><strong>No curated model matches these filters</strong><span>Clear one or more filters.</span></div>}
          {catalogRows.map((model) => {
            const filename = catalogFiles[model.id] ?? model.files[0]?.filename ?? "";
            const file = model.files.find((entry) => entry.filename === filename) ?? model.files[0];
            if (!file) return null;
            // Progress and cancellation follow the running job for
            // this file, not the currently edited destination (FE-11).
            const activeJob = activeDownloadJob(downloadJobs, downloads, model.repo, file.filename);
            const latestJob = newestDownloadJob(downloadJobs, model.repo, file.filename);
            const progressKey = (activeJob ?? latestJob)?.key
              ?? downloadKey(model.repo, file.filename, modelRoot, catalogRevision(file));
            const progress = downloads[progressKey];
            const running = activeJob !== undefined;
            const smallestBytes = Math.min(...model.files.map((entry) => entry.sizeBytes));
            const buildFit = catalogBuildFit(
              file.sizeBytes,
              smallestBytes,
              catalogFitEnabled ? catalogFitPerMille : 0,
              catalogFitBudget?.budgetBytes,
            );
            const alreadyOnDisk = inventoryHasFile(file.filename) || progress?.state === "done";
            const readiness = downloadReadiness({ destination: modelRoot, running, alreadyOnDisk, gated: model.gated, hasToken: hfToken.configured });
            return (
              <article className="machine-panel catalog-model" key={model.id}>
                <div className="catalog-model-head">
                  <div>
                    <button className="catalog-repo" onClick={() => openExternal(`https://huggingface.co/${model.repo}`)}>{model.repo}<ExternalLink size={12} /></button>
                    <strong>{model.family || model.repo.split("/")[1]}</strong>
                    <span>{model.parameters || "PARAMETERS UNKNOWN"} · BY {model.publisher || model.repo.split("/")[0]}</span>
                  </div>
                  {model.gated && <span className="state-tag warning">GATED · TOKEN + LICENCE</span>}
                  {model.userSourced && <span className="state-tag">USER ADDED · LOCAL ONLY</span>}
                </div>
                {model.summary && <p className="catalog-summary">{model.summary}</p>}
                <div className="catalog-meta">
                  <span>{model.downloads.toLocaleString()} downloads</span>
                  <span>{model.likes.toLocaleString()} likes</span>
                  {model.tags.map((tag) => <i key={tag}>{tag.toUpperCase()}</i>)}
                </div>
                <div className="catalog-file-row">
                  <label>Build<select value={file.filename} onChange={(event) => setCatalogFiles((current) => ({ ...current, [model.id]: event.target.value }))}>{model.files.map((entry) => <option key={entry.filename} value={entry.filename}>{entry.quant} · {bytesLabel(entry.sizeBytes)}</option>)}</select></label>
                  <div className="catalog-filename"><PathText value={file.filename} label="Model file" />{file.userSourced && <span className="state-tag">USER FILE · LOCAL DIGEST</span>}<small>{bytesLabel(file.sizeBytes)} · 4 PARALLEL RANGES</small></div>
                  {running ? (
                    <button className="button danger" onClick={() => cancelCatalogDownload(activeJob!)}><CircleStop size={15} /> Keep & stop</button>
                  ) : (
                    <button className={alreadyOnDisk ? "button is-current" : "button secondary"} disabled={!readiness.canStart} title={readiness.reason} onClick={() => startCatalogDownload(model, file)}>
                      {alreadyOnDisk ? <><BadgeCheck size={15} /> Verify file</> : <><Download size={15} /> {progress?.state === "error" ? "Resume" : "Download"}</>}
                    </button>
                  )}
                </div>
                {catalogFitEnabled && (
                  <p className="catalog-fit-note">
                    {buildFit.selectedPasses === false
                      ? `SIZE CHECK FAILS FOR THIS BUILD · ${bytesLabel(file.sizeBytes)} > ${bytesLabel(buildFit.thresholdBytes ?? 0)} threshold on ${catalogFitBudget?.source ?? "unknown"} budget${buildFit.rowKept ? ". The model stays listed because a smaller build fits; runtime memory is not measured." : ""}`
                      : buildFit.selectedPasses === true
                        ? `SIZE CHECK PASSES · ${bytesLabel(file.sizeBytes)} ≤ ${bytesLabel(buildFit.thresholdBytes ?? 0)} on ${catalogFitBudget?.source ?? "unknown"} budget · size only, runtime memory not measured`
                        : "SIZE CHECK UNKNOWN · no measured memory budget; open the build to check allocation"}
                  </p>
                )}
                {progress && (
                  <div className={`download-progress ${progress.state}`}>
                    <div><b style={{ width: `${downloadPercent(progress.downloaded, progress.total)}%` }} /></div>
                    <span>{progress.state.toUpperCase()} · {bytesLabel(progress.downloaded)} / {bytesLabel(progress.total)}</span>
                    <small>{rateLabel(progress.bytesPerSecond)} {etaLabel(progress.downloaded, progress.total, progress.bytesPerSecond)}</small>
                    {progress.message && <p>{progress.message}</p>}
                  </div>
                )}
                {!readiness.canStart && !running && !alreadyOnDisk && <p className="catalog-blocker">{readiness.reason}</p>}
              </article>
            );
          })}
        </div>

        <aside className="machine-panel catalog-sidebar">
          <div className="panel-title"><KeyRound size={17} /><h2>Hugging Face access</h2><span className={hfToken.configured ? "state-tag good" : "state-tag warning"}>{hfToken.configured ? `TOKEN ${hfToken.masked}` : "ANONYMOUS"}</span></div>
          <div className="catalog-token-body">
            <p>A token is still useful: it enables gated repositories after you accept their licence and applies your account’s higher resolver rate limits. It does not guarantee higher raw bandwidth.</p>
            <label>Read token<div className="key-row"><input type="password" autoComplete="off" value={hfTokenDraft} onChange={(event) => setHfTokenDraft(event.target.value)} placeholder={hfToken.configured ? `Stored ${hfToken.masked}` : "hf_…"} /><button className="button secondary" disabled={!hfTokenDraft.trim() || catalogBusy} onClick={saveHfToken}>Store</button></div></label>
            {hfToken.configured && <button className="text-link" onClick={removeHfToken}>Remove stored token</button>}
            {hfToken.cleanupNotice && (
              <p role="status" className="token-cleanup-notice">{hfToken.cleanupNotice}</p>
            )}
            <button className="text-link" onClick={() => openExternal("https://huggingface.co/settings/tokens")}>Create a read token on Hugging Face ↗</button>
          </div>
          <dl className="runtime-facts catalog-transfer-facts">
            <div><dt>Data route</dt><dd>Hugging Face → this PC. Localmotive never proxies model bytes.</dd></div>
            <div><dt>Resume</dt><dd>Per-chunk progress survives interruption in .part metadata.</dd></div>
            <div><dt>Parallelism</dt><dd>Four ranged HTTPS connections; one when the CDN does not support ranges.</dd></div>
            <div><dt>Integrity</dt><dd>Final size always checked; SHA-256 verified when Hugging Face publishes it as the object ETag.</dd></div>
            <div><dt>Secret storage</dt><dd>Windows Credential Manager service “Localmotive HF”; never local storage or logs.</dd></div>
          </dl>
        </aside>
      </div>
    </section>
  );
}
