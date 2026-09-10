// Dry-run the v2 allowlist against Hugging Face. Reads live HF metadata and
// reports repo count, file count, and artifact bytes. Never writes the catalog
// and never needs the signing key.
//   node scripts/dryrun_catalog.mjs
//   node scripts/dryrun_catalog.mjs --config catalog/providers.json
import { readFile } from "node:fs/promises";

const configPath = process.argv.find((arg) => arg === "--config")
  ? process.argv[process.argv.indexOf("--config") + 1]
  : "catalog/providers.json";

const config = JSON.parse(await readFile(configPath, "utf8"));
const authors = config.allowlist ?? [];
const cutoffDays = config.cutoffDays ?? 90;
const maxRepos = config.maxReposPerAuthor ?? 20;
const cutoff = new Date("2026-09-10T00:00:00Z");
cutoff.setUTCDate(cutoff.getUTCDate() - cutoffDays);
const excludes = (config.excludeFilenamePatterns ?? []).map(
  (pattern) => new RegExp(pattern, "i"),
);

const api = async (url) => {
  const response = await fetch(url, {
    headers: { "User-Agent": "localmotive-dryrun" },
  });
  if (!response.ok) throw new Error(`${response.status} ${url}`);
  return response.json();
};

let repos = 0;
let files = 0;
let bytes = 0;
let skippedNoSha = 0;
let skippedExcluded = 0;
const perAuthor = [];

for (const author of authors) {
  let list = [];
  try {
    list = await api(
      `https://huggingface.co/api/models?author=${encodeURIComponent(author)}&filter=gguf&sort=lastModified&direction=-1&limit=100`,
    );
  } catch (error) {
    perAuthor.push({ author, error: String(error) });
    continue;
  }
  const recent = list.filter(
    (model) => new Date(model.lastModified) >= cutoff,
  );
  let authorFiles = 0;
  let authorBytes = 0;
  for (const model of recent.slice(0, maxRepos)) {
    let meta;
    try {
      meta = await api(`https://huggingface.co/api/models/${model.id}?blobs=true`);
    } catch {
      continue;
    }
    const ggufs = (meta.siblings || []).filter((sibling) =>
      sibling.rfilename.toLowerCase().endsWith(".gguf"),
    );
    for (const sibling of ggufs) {
      if (excludes.some((pattern) => pattern.test(sibling.rfilename))) {
        skippedExcluded += 1;
        continue;
      }
      const sha = sibling.lfs?.sha256 ?? sibling.lfs?.oid;
      if (!/^[a-f0-9]{64}$/i.test(sha ?? "")) {
        skippedNoSha += 1;
        continue;
      }
      if (typeof sibling.size !== "number" || sibling.size <= 0) {
        skippedNoSha += 1;
        continue;
      }
      files += 1;
      authorFiles += 1;
      bytes += sibling.size;
      authorBytes += sibling.size;
    }
    repos += 1;
    await new Promise((resolve) => setTimeout(resolve, 200));
  }
  perAuthor.push({
    author,
    scanned: Math.min(recent.length, maxRepos),
    recent90: recent.length,
    files: authorFiles,
    bytes: authorBytes,
  });
  await new Promise((resolve) => setTimeout(resolve, 300));
}

console.log(
  JSON.stringify(
    {
      cutoff: cutoff.toISOString(),
      authors: authors.length,
      repos,
      files,
      bytes,
      skippedNoSha,
      skippedExcluded,
      perAuthor,
    },
    null,
    1,
  ),
);
