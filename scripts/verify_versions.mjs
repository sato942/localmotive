#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

async function json(path) {
  return JSON.parse(await readFile(path, "utf8"));
}

function packageVersion(text, packageName) {
  for (const block of text.split(/^\[\[package\]\]\s*$/m).slice(1)) {
    const name = block.match(/^name\s*=\s*"([^"]+)"\s*$/m)?.[1];
    if (name === packageName) return block.match(/^version\s*=\s*"([^"]+)"\s*$/m)?.[1] ?? null;
  }
  return null;
}

function cargoManifestVersion(text) {
  let inPackage = false;
  for (const line of text.split(/\r?\n/)) {
    if (/^\[.+\]\s*$/.test(line)) {
      inPackage = line.trim() === "[package]";
      continue;
    }
    if (inPackage) {
      const version = line.match(/^version\s*=\s*"([^"]+)"\s*$/)?.[1];
      if (version) return version;
    }
  }
  return null;
}

export async function verifyVersions(root, expected) {
  const packageJson = await json(resolve(root, "package.json"));
  const packageLock = await json(resolve(root, "package-lock.json"));
  const tauri = await json(resolve(root, "src-tauri", "tauri.conf.json"));
  const cargoToml = await readFile(resolve(root, "src-tauri", "Cargo.toml"), "utf8");
  const cargoLock = await readFile(resolve(root, "src-tauri", "Cargo.lock"), "utf8");
  const fields = [
    { name: "package.json", value: packageJson.version },
    { name: "package-lock.json", value: packageLock.version },
    { name: "package-lock.json packages['']", value: packageLock.packages?.[""]?.version },
    { name: "src-tauri/Cargo.toml", value: cargoManifestVersion(cargoToml) },
    { name: "src-tauri/Cargo.lock localmotive", value: packageVersion(cargoLock, "localmotive") },
    { name: "src-tauri/tauri.conf.json", value: tauri.version },
  ];
  const failures = fields
    .filter(({ value }) => value !== expected)
    .map(({ name, value }) => `${name} reports ${JSON.stringify(value)} instead of ${JSON.stringify(expected)}`);
  return { ok: failures.length === 0, expected, fields, failures };
}

async function main() {
  const expected = process.argv[2];
  if (!/^\d+\.\d+\.\d+$/.test(expected ?? "")) {
    throw new Error("Usage: node scripts/verify_versions.mjs MAJOR.MINOR.PATCH");
  }
  const result = await verifyVersions(process.cwd(), expected);
  console.log(JSON.stringify(result, null, 2));
  if (!result.ok) process.exitCode = 1;
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  main().catch((error) => {
    console.error(`version-gate: ${error.message}`);
    process.exitCode = 1;
  });
}
