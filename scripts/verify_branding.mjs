import { readdir, readFile } from "node:fs/promises";
import { extname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const LEGACY_NAMES = [
  ["GGUF", "Pilot"].join(" "),
  ["gguf", "pilot"].join("-"),
];

const HISTORICAL_FILES = new Set([
  "CHANGELOG.md",
  "Future_branding.md",
  "TODO.md",
  "TODO-0.4.md",
  "TODO-0.4.1.md",
  "release-evidence/0.4.1/research-freeze-manifest.json",
]);

const MIGRATION_LINES = new Map([
  ["src/App.tsx", [["localStorage.getItem(`", LEGACY_NAMES[1], ":${key}`)"].join("")]],
  ["src-tauri/src/catalog.rs", ["LEGACY_HF_KEYRING_SERVICE", `${LEGACY_NAMES[0]} HF`]],
  ["src-tauri/src/cloud.rs", ["LEGACY_KEYRING_SERVICE", LEGACY_NAMES[0]]],
  ["src-tauri/src/runtime.rs", [
    `runtime_data_dir(\"${LEGACY_NAMES[0]}\")`,
    `base.join(\"${LEGACY_NAMES[0]}\")`,
  ]],
  ["scripts/verify_041.mjs", [`content.text.includes(\"${LEGACY_NAMES[0]}\")`]],
]);

const TEXT_EXTENSIONS = new Set([
  ".css",
  ".html",
  ".js",
  ".json",
  ".jsx",
  ".md",
  ".mjs",
  ".rs",
  ".toml",
  ".ts",
  ".tsx",
  ".txt",
  ".yaml",
  ".yml",
]);

const SKIPPED_DIRECTORIES = new Set([
  ".git",
  "dist",
  "node_modules",
  "research",
  "target",
]);

function normalizedPath(path) {
  return path.split(sep).join("/");
}

function lineIsApproved(path, line) {
  if (HISTORICAL_FILES.has(path)) return true;
  const snippets = MIGRATION_LINES.get(path) ?? [];
  return snippets.some((snippet) => line.includes(snippet));
}

export function validateBrandingEntries(entries) {
  const failures = [];
  for (const entry of entries) {
    const path = normalizedPath(entry.path);
    const lines = entry.content.split(/\r?\n/u);
    const hasUnapprovedLegacyName = lines.some((line) =>
      LEGACY_NAMES.some((legacy) => line.toLowerCase().includes(legacy.toLowerCase()))
      && !lineIsApproved(path, line));
    if (hasUnapprovedLegacyName) {
      failures.push(`${path}:legacy-product-name`);
    }
  }
  return { ok: failures.length === 0, failures };
}

async function collectTextEntries(root, directory = root) {
  const entries = [];
  const directoryEntries = await readdir(directory, { withFileTypes: true });
  for (const entry of directoryEntries) {
    if (entry.name === ".env" || entry.name.startsWith(".env.")) continue;
    const absolute = join(directory, entry.name);
    if (entry.isDirectory()) {
      if (!SKIPPED_DIRECTORIES.has(entry.name)) {
        entries.push(...await collectTextEntries(root, absolute));
      }
      continue;
    }
    if (!entry.isFile() || !TEXT_EXTENSIONS.has(extname(entry.name).toLowerCase())) continue;
    entries.push({
      path: normalizedPath(relative(root, absolute)),
      content: await readFile(absolute, "utf8"),
    });
  }
  return entries;
}

export async function verifyBranding(root = process.cwd()) {
  return validateBrandingEntries(await collectTextEntries(resolve(root)));
}

const invokedPath = process.argv[1] ? resolve(process.argv[1]) : null;
if (invokedPath === fileURLToPath(import.meta.url)) {
  const result = await verifyBranding(process.cwd());
  if (!result.ok) {
    for (const failure of result.failures) console.error(`FAIL ${failure}`);
    process.exitCode = 1;
  } else {
    console.log("PASS branding: no unapproved legacy product names");
  }
}
