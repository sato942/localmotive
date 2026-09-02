#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import { createPublicKey, verify } from "node:crypto";

const path = process.argv[2] ?? "catalog/catalog.json";
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

if (catalog.schemaVersion !== 1) fail("schemaVersion must be 1");
if (!/^\d{4}-\d{2}-\d{2}$/.test(catalog.updated ?? "")) fail("updated must be YYYY-MM-DD");
if (!Array.isArray(catalog.models) || catalog.models.length === 0) fail("models must be a non-empty array");

try {
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
for (const [index, model] of (catalog.models ?? []).entries()) {
  const at = `models[${index}]`;
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(model.id ?? "")) fail(`${at}.id is not a stable slug`);
  if (ids.has(model.id)) fail(`${at}.id duplicates ${model.id}`);
  ids.add(model.id);
  if (!/^[A-Za-z0-9._-]+\/[A-Za-z0-9._-]+$/.test(model.repo ?? "")) fail(`${at}.repo must be owner/name`);
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
    // Every model writes into the same chosen folder on case-insensitive Windows.
    const target = filename.toLocaleLowerCase("en-US");
    if (targets.has(target)) fail(`${fileAt} duplicates ${target}`);
    targets.add(target);
  }
}

if (!process.exitCode) console.log(`catalog: valid (${catalog.models.length} models, ${targets.size} files)`);
