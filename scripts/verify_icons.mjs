#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const REQUIRED_ICO_SIZES = [16, 24, 32, 48, 64, 256];

export function inspectIcoSizes(buffer) {
  if (!Buffer.isBuffer(buffer) || buffer.length < 6) throw new Error("ICO header is truncated");
  if (buffer.readUInt16LE(0) !== 0 || buffer.readUInt16LE(2) !== 1) throw new Error("ICO header is invalid");
  const count = buffer.readUInt16LE(4);
  if (count === 0 || buffer.length < 6 + (count * 16)) throw new Error("ICO directory is truncated");
  const sizes = [];
  for (let index = 0; index < count; index += 1) {
    const offset = 6 + (index * 16);
    const width = buffer[offset] || 256;
    const height = buffer[offset + 1] || 256;
    if (width !== height) throw new Error(`ICO layer ${index} is not square`);
    sizes.push(width);
  }
  return [...new Set(sizes)].sort((left, right) => left - right);
}

export function validateIconConfiguration({ config, svg, icoSizes, expectedPublisher }) {
  const failures = [];
  const bundle = config?.bundle ?? {};
  const nsis = bundle.windows?.nsis ?? {};
  if (bundle.publisher !== expectedPublisher) failures.push(`bundle.publisher must equal ${JSON.stringify(expectedPublisher)}`);
  if (!Array.isArray(bundle.icon) || !bundle.icon.includes("icons/icon.ico")) failures.push("bundle.icon must include icons/icon.ico");
  if (nsis.installerIcon !== "icons/icon.ico") failures.push("NSIS installerIcon must equal icons/icon.ico");
  if (nsis.uninstallerIcon !== "icons/icon.ico") failures.push("NSIS uninstallerIcon must equal icons/icon.ico");
  for (const size of REQUIRED_ICO_SIZES) {
    if (!icoSizes.includes(size)) failures.push(`icon.ico is missing its ${size}x${size} layer`);
  }
  if (/<text\b/i.test(svg)) failures.push("the canonical icon must not depend on a font");
  if ((svg.match(/<path\b/gi) ?? []).length < 2) failures.push("the canonical icon must contain vector paths for L and M");
  for (const color of ["#1d2122", "#d5d1c5", "#9edc72"]) {
    if (!svg.toLowerCase().includes(color)) failures.push(`the canonical icon is missing ${color}`);
  }
  return { ok: failures.length === 0, icoSizes, publisher: bundle.publisher ?? null, failures };
}

async function main() {
  const root = process.cwd();
  const expectedPublisher = process.argv[2] ?? "Localmotive contributors";
  const config = JSON.parse(await readFile(resolve(root, "src-tauri", "tauri.conf.json"), "utf8"));
  const svg = await readFile(resolve(root, "assets", "app-icon.svg"), "utf8");
  const icoSizes = inspectIcoSizes(await readFile(resolve(root, "src-tauri", "icons", "icon.ico")));
  const result = validateIconConfiguration({ config, svg, icoSizes, expectedPublisher });
  console.log(JSON.stringify(result, null, 2));
  if (!result.ok) process.exitCode = 1;
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  main().catch((error) => {
    console.error(`icon-gate: ${error.message}`);
    process.exitCode = 1;
  });
}
