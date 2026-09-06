import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import { readFile, stat, writeFile } from "node:fs/promises";
import { basename, join, resolve } from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const directory = resolve(process.argv[2] ?? "artifacts");
const version = process.argv[3] ?? "0.4.1";
const output = resolve(process.argv[4] ?? join(directory, "candidate-inventory.json"));

function gitRevision() {
  const configured = process.env.LOCALMOTIVE_SOURCE_REVISION;
  if (configured) return configured.toLowerCase();
  return execFileSync("git", ["rev-parse", "HEAD"], {
    cwd: process.cwd(),
    encoding: "utf8",
    windowsHide: true,
  }).trim().toLowerCase();
}

async function sha256File(path) {
  const digest = createHash("sha256");
  for await (const chunk of createReadStream(path)) digest.update(chunk);
  return digest.digest("hex");
}

function parseChecksums(text) {
  const entries = new Map();
  for (const line of text.trimEnd().split(/\r?\n/u)) {
    const match = /^([0-9a-f]{64}) [ *](.+)$/u.exec(line);
    if (!match || entries.has(match[2])) throw new Error("SHA256SUMS contains a malformed or duplicate row");
    entries.set(match[2], match[1]);
  }
  return entries;
}

export async function verifyCandidateInventory({
  artifactDirectory = directory,
  releaseVersion = version,
  outputPath = output,
  sourceRevision = gitRevision(),
} = {}) {
  if (!/^[0-9a-f]{40}$/u.test(sourceRevision)) throw new Error("Source revision must be one full Git commit");
  const names = [
    `Localmotive_${releaseVersion}_x64-setup.exe`,
    `Localmotive_${releaseVersion}_x64-portable.exe`,
    `Localmotive_${releaseVersion}_x64.msi`,
  ].sort();
  const checksumsPath = join(artifactDirectory, `SHA256SUMS-${releaseVersion}.txt`);
  const declared = parseChecksums(await readFile(checksumsPath, "utf8"));
  if (JSON.stringify([...declared.keys()].sort()) !== JSON.stringify(names)) {
    throw new Error("SHA256SUMS does not name the exact release asset set");
  }
  const artifacts = [];
  for (const name of names) {
    const path = join(artifactDirectory, name);
    const details = await stat(path);
    if (!details.isFile() || details.size <= 0) throw new Error(`${name} is missing or empty`);
    const sha256 = await sha256File(path);
    if (declared.get(name) !== sha256) throw new Error(`${name} does not match SHA256SUMS`);
    artifacts.push({ name, sizeBytes: details.size, sha256 });
  }
  const record = {
    schemaVersion: 1,
    release: releaseVersion,
    sourceRevision,
    generatedAt: new Date().toISOString(),
    checksumFile: basename(checksumsPath),
    artifacts,
  };
  await writeFile(outputPath, `${JSON.stringify(record, null, 2)}\n`, "utf8");
  return record;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const record = await verifyCandidateInventory();
  console.log(`PASS candidate inventory: ${record.artifacts.length} artifacts`);
}
