#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import { createPublicKey, verify } from "node:crypto";
import { validateCatalogRows } from "./lib/catalog_schema.mjs";

const path = process.argv[2] ?? "catalog/catalog.json";
// The build job writes an unsigned candidate; the sign job signs it right
// after. Structure checks must pass on the unsigned candidate, so signature
// verification is opt-out via --no-signature instead of the default.
//   node scripts/validate_catalog.mjs catalog/catalog.json
//   node scripts/validate_catalog.mjs catalog/catalog.json --no-signature
const skipSignature = process.argv.includes("--no-signature");
const fail = (message) => {
  console.error(`catalog: ${message}`);
  process.exitCode = 1;
};

let catalog;
try {
  catalog = JSON.parse(await readFile(path, "utf8"));
} catch (error) {
  fail(`cannot parse ${path}: ${error.message}`);
  process.exit();
}

if (catalog.schemaVersion !== 2) fail("schemaVersion must be 2");
if (!/^\d{4}-\d{2}-\d{2}$/.test(catalog.updated ?? "")) fail("updated must be YYYY-MM-DD");
if (!Array.isArray(catalog.models) || catalog.models.length === 0) fail("models must be a non-empty array");

// The allowlist ships inside the signed artifact so adding an author is a
// catalog publish, never an app release. The app binary must not hardcode it.
if (catalog.providers == null || !Array.isArray(catalog.providers.allowlist) || catalog.providers.allowlist.length === 0) {
  fail("providers.allowlist must be a non-empty array inside the signed artifact");
}
if (typeof catalog.providers?.cutoffDays !== "number" || catalog.providers.cutoffDays <= 0) {
  fail("providers.cutoffDays must be a positive number");
}

if (!skipSignature) try {
  const rawKey = Buffer.from([234, 194, 139, 46, 191, 202, 36, 78, 104, 245, 230, 170, 90, 67, 238, 61, 1, 162, 242, 207, 116, 254, 217, 3, 74, 69, 78, 102, 199, 170, 8, 119]);
  const spkiPrefix = Buffer.from("302a300506032b6570032100", "hex");
  const publicKey = createPublicKey({ key: Buffer.concat([spkiPrefix, rawKey]), format: "der", type: "spki" });
  const signature = Buffer.from((await readFile(`${path}.sig`, "utf8")).trim(), "base64");
  const body = await readFile(path);
  if (signature.length !== 64 || !verify(null, body, publicKey, signature)) fail("detached Ed25519 signature is invalid");
} catch (error) {
  fail(`cannot verify ${path}.sig: ${error.message}`);
}

const ids = new Set();
const targets = new Set();
const isoDate = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$/;
for (const [index, model] of (catalog.models ?? []).entries()) {
  const at = `models[${index}]`;
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(model.id ?? "")) fail(`${at}.id is not a stable slug`);
  if (ids.has(model.id)) fail(`${at}.id duplicates ${model.id}`);
  ids.add(model.id);
  if (!/^[A-Za-z0-9._-]+\/[A-Za-z0-9._-]+$/.test(model.repo ?? "")) fail(`${at}.repo must be owner/name`);
  if (model.lastModified && !isoDate.test(model.lastModified)) fail(`${at}.lastModified must be ISO-8601`);
  if (model.createdAt && !isoDate.test(model.createdAt)) fail(`${at}.createdAt must be ISO-8601`);
  if (!Array.isArray(model.files) || model.files.length === 0) fail(`${at}.files must not be empty`);
  for (const [fileIndex, file] of (model.files ?? []).entries()) {
    const fileAt = `${at}.files[${fileIndex}]`;
    const filename = file.filename ?? "";
    if (!filename.toLowerCase().endsWith(".gguf") || filename.includes("/") || filename.includes("\\") || filename.includes(":")) {
      fail(`${fileAt}.filename is unsafe or is not GGUF`);
    }
    if (!Number.isSafeInteger(file.sizeBytes) || file.sizeBytes <= 0) fail(`${fileAt}.sizeBytes must be a positive safe integer`);
    if (!/^[a-f0-9]{64}$/i.test(file.sha256 ?? "")) fail(`${fileAt}.sha256 must be a 64-digit SHA-256`);
    if (typeof file.quant !== "string" || !file.quant.trim()) fail(`${fileAt}.quant is required`);
    if (file.lastModified && !isoDate.test(file.lastModified)) fail(`${fileAt}.lastModified must be ISO-8601`);
    // Every model writes into the same chosen folder on case-insensitive Windows.
    const target = filename.toLocaleLowerCase("en-US");
    if (targets.has(target)) fail(`${fileAt} duplicates ${target}`);
    targets.add(target);
  }
}

// The shared contract (audit S-05) must accept every curated row: a drop at
// validation time would make that row's files unreachable in the app.
const shared = validateCatalogRows(catalog.models ?? []);
if (shared.error) fail(`shared schema contract: ${shared.error}`);
if (shared.dropped.length > 0) {
  fail(`shared schema contract drops ${shared.dropped.length} row(s): ${JSON.stringify(shared.dropped.slice(0, 3))}`);
}

if (!process.exitCode) console.log(`catalog: valid v2 (${catalog.models.length} models, ${targets.size} files) · shared contract 0 drops`);
