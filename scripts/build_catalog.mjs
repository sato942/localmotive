// Build catalog entries from real Hugging Face metadata. Never invents values:
// every repo, filename and byte size below is read from the live HF API, and a
// repo that cannot be resolved aborts without replacing the last good catalog.
//   node scripts/build_catalog.mjs                       # write catalog/catalog.json (CI build job)
//   node scripts/build_catalog.mjs --write               # legacy alias for the same write
//   node scripts/build_catalog.mjs --dry-run             # author/repo/file counts only, no JSON
//   node scripts/build_catalog.mjs --stdout              # preview JSON on stdout, no write
import { rename, readFile, writeFile } from "node:fs/promises";

const providers = JSON.parse(await readFile("catalog/providers.json", "utf8"));
const ALLOWLIST = providers.allowlist ?? [];
const CUTOFF_DAYS = providers.cutoffDays ?? 90;
const MAX_REPOS = process.argv.includes("--full") ? 100 : (providers.maxReposPerAuthor ?? 20);
const DRY_RUN = process.argv.includes("--dry-run");
const ALLOW_EMPTY = process.argv.includes("--allow-empty");
const cutoff = new Date("2026-09-10T00:00:00Z");
cutoff.setUTCDate(cutoff.getUTCDate() - CUTOFF_DAYS);
const excludePatterns = (providers.excludeFilenamePatterns ?? []).map(
  (pattern) => new RegExp(pattern, "i"),
);

const api = async (url, token, retries = 5) => {
  const headers = { "User-Agent": "localmotive-catalog-builder" };
  if (token) headers.Authorization = `Bearer ${token}`;
  for (let attempt = 0; ; attempt += 1) {
    const r = await fetch(url, { headers });
    if (r.status === 429 && attempt < retries) {
      const wait = Math.min(30000, 2000 * 2 ** attempt);
      console.error(`429 ${url}, retry ${attempt + 1}/${retries} after ${wait}ms`);
      await new Promise((resolve) => setTimeout(resolve, wait));
      continue;
    }
    if (!r.ok) throw new Error(`${r.status} ${url}`);
    return r.json();
  }
};
const token = process.env.HF_TOKEN ?? process.env.HUGGING_FACE_TOKEN ?? "";

const licenseOf = (meta) => {
  const tags = meta.tags || [];
  const found = tags.find((tag) => tag.startsWith("license:"));
  if (found) return found.slice("license:".length);
  if (meta.cardData && typeof meta.cardData.license === "string") return meta.cardData.license;
  return "";
};

const entries = [];
const problems = [];
let scannedRepos = 0;
let scannedFiles = 0;
let skippedExcluded = 0;
let skippedNoSha = 0;
// Filenames collide across publishers with different bytes. The client
// downloads into one shared folder, so dedupe happens here at build time.
const seenFilenames = new Set();
const pending = [];

for (const author of ALLOWLIST) {
  let list;
  try {
    list = await api(
      `https://huggingface.co/api/models?author=${encodeURIComponent(author)}&filter=gguf&sort=lastModified&direction=-1&limit=100`,
      token,
    );
  } catch (error) {
    problems.push(`${author}: list failed: ${error.message}`);
    continue;
  }
  const recent = list.filter((model) => new Date(model.lastModified) >= cutoff);
  for (const summary of recent.slice(0, MAX_REPOS)) {
    const repo = summary.id;
    try {
      const meta = await api(`https://huggingface.co/api/models/${repo}?blobs=true`, token);
      const ggufs = (meta.siblings || []).filter((s) =>
        s.rfilename.toLowerCase().endsWith(".gguf"),
      );
      if (!ggufs.length) {
        continue;
      }
      const files = [];
      for (const sibling of ggufs) {
        scannedFiles += 1;
        const name = sibling.rfilename;
        // Only root-level files: subdirectories hold MTP drafts, DSpark
        // variants, imatrix data, and experiments, none of which are direct
        // single-file download targets. A slash also fails the client
        // filename guard, so exclude here instead of emitting dead rows.
        if (name.includes("/") || name.includes("\\")) {
          skippedExcluded += 1;
          continue;
        }
        if (excludePatterns.some((pattern) => pattern.test(name))) {
          skippedExcluded += 1;
          continue;
        }
        if (typeof sibling.size !== "number" || sibling.size <= 0) {
          skippedNoSha += 1;
          problems.push(`${repo}/${name}: no size`);
          continue;
        }
        const sha256 = sibling.lfs?.sha256 ?? sibling.lfs?.oid;
        if (!/^[a-f0-9]{64}$/i.test(sha256 ?? "")) {
          skippedNoSha += 1;
          problems.push(`${repo}/${name}: no SHA-256`);
          continue;
        }
        const quant = (name.match(/-([A-Za-z0-9_]+)\.gguf$/i) || [])[1] || "UNKNOWN";
        files.push({
          quant,
          filename: name,
          sizeBytes: sibling.size,
          sha256,
          lastModified: meta.lastModified ?? "",
          createdAt: meta.createdAt ?? "",
        });
      }
      if (!files.length) {
        continue;
      }
      scannedRepos += 1;
      pending.push({
        id: repo.toLowerCase().replace(/[^a-z0-9]+/g, "-"),
        repo,
        family: repo.split("/")[1].replace(/-GGUF$/i, "").replace(/-/g, " "),
        parameters: "",
        publisher: repo.split("/")[0],
        author: meta.author ?? repo.split("/")[0],
        summary: (meta.cardData && meta.cardData.summary) || "",
        tags: (meta.tags || []).filter((tag) => !tag.includes(":")),
        gated: Boolean(meta.gated),
        downloads: meta.downloads ?? 0,
        likes: meta.likes ?? 0,
        license: licenseOf(meta),
        // Wire keys stay snake_case: the checked-in v2 contract (signed run
        // 34482317368) uses pipeline_tag/library_name, and the signature
        // covers those exact bytes. camelCase rename_all covers the Tauri IPC
        // boundary, not the file format.
        pipeline_tag: meta.pipeline_tag ?? "",
        library_name: meta.library_name ?? "",
        architecture: meta.gguf?.architecture ?? "",
        lastModified: meta.lastModified ?? "",
        createdAt: meta.createdAt ?? "",
        files: files.sort((a, b) => a.sizeBytes - b.sizeBytes),
      });
    } catch (error) {
      problems.push(`${repo}: ${error.message}`);
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  await new Promise((resolve) => setTimeout(resolve, 800));
}

if (DRY_RUN) {
  for (const entry of pending.sort((a, b) => b.downloads - a.downloads)) {
    const kept = [];
    for (const file of entry.files) {
      const key = file.filename.toLowerCase();
      if (seenFilenames.has(key)) {
        skippedExcluded += 1;
        continue;
      }
      seenFilenames.add(key);
      kept.push(file);
    }
    if (!kept.length) continue;
    entries.push({ ...entry, files: kept });
  }
  console.log(
    JSON.stringify(
      {
        cutoff: cutoff.toISOString().slice(0, 10),
        authors: ALLOWLIST.length,
        repos: scannedRepos,
        files: entries.reduce((n, e) => n + e.files.length, 0),
        bytes: entries.reduce((n, e) => n + e.files.reduce((a, f) => a + f.sizeBytes, 0), 0),
        scannedFiles,
        skippedExcluded,
        skippedNoSha,
        problems: problems.slice(0, 20),
      },
      null,
      1,
    ),
  );
  process.exit(0);
}

// Two publishers may ship the same filename with different bytes (seen live:
// 61 colliding names with different SHA-256 across unsloth,
// lmstudio-community, AtomicChat, LiquidAI). The client downloads into one
// shared folder, so the second write would clobber or race the first. Sort
// repos by downloads first so the most-used publisher wins each filename,
// then drop colliding files from later repos.
for (const entry of pending.sort((a, b) => b.downloads - a.downloads)) {
  const kept = [];
  for (const file of entry.files) {
    const key = file.filename.toLowerCase();
    if (seenFilenames.has(key)) {
      skippedExcluded += 1;
      continue;
    }
    seenFilenames.add(key);
    kept.push(file);
  }
  if (!kept.length) continue;
  entries.push({ ...entry, files: kept });
}

const catalog = {
  schemaVersion: 2,
  updated: new Date().toISOString().slice(0, 10),
  source: "https://github.com/sato942/localmotive/blob/main/catalog/catalog.json",
  note: "Curated list of GGUF builds. Localmotive downloads directly from Hugging Face; this file only decides what is offered.",
  providers: {
    source: "catalog/providers.json",
    cutoffDays: CUTOFF_DAYS,
    allowlist: ALLOWLIST,
  },
  models: entries.sort((a, b) => b.downloads - a.downloads),
};
const rendered = JSON.stringify(catalog, null, 2) + "\n";
console.error(
  `resolved ${entries.length} repos, ${entries.reduce((n, e) => n + e.files.length, 0)} files (${scannedRepos} scanned, ${skippedExcluded} excluded, ${skippedNoSha} no-sha)`,
);

if (problems.length && !ALLOW_EMPTY) {
  console.error("UNRESOLVED (catalog not replaced):\n  " + problems.join("\n  "));
  process.exitCode = 1;
} else if (process.argv.includes("--stdout") && !process.argv.includes("--write")) {
  process.stdout.write(rendered);
} else {
  // Default and --write: atomically replace catalog/catalog.json without
  // touching the signature. The sign job signs the exact committed candidate
  // afterwards, so build never needs the private key.
  const temporary = "catalog/catalog.json.next";
  await writeFile(temporary, rendered, "utf8");
  await rename(temporary, "catalog/catalog.json");
  console.error("wrote catalog/catalog.json (unsigned; the sign job signs it)");
}
