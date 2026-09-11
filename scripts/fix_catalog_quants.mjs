// One-time migration of the shipped catalog's quant labels (audit DC-09).
//
// Rewrites each file's `quant` field from the file's own quantisation token
// using the shared extractor. Run once; the catalog build pipeline re-signs
// the candidate afterwards (the signature lives in catalog/catalog.json.sig
// and is produced only by the sign job that holds the private key).
import { readFile, rename, writeFile } from "node:fs/promises";
import { quantFromFilename } from "./lib/quant_label.mjs";

const path = process.argv[2] ?? "catalog/catalog.json";
const catalog = JSON.parse(await readFile(path, "utf8"));

let changed = 0;
const samples = [];
for (const model of catalog.models ?? []) {
  for (const file of model.files ?? []) {
    const next = quantFromFilename(file.filename);
    if (next !== file.quant) {
      changed += 1;
      if (samples.length < 12) samples.push(`${file.filename}: ${file.quant} -> ${next}`);
      file.quant = next;
    }
  }
}

const temporary = `${path}.next`;
await writeFile(temporary, `${JSON.stringify(catalog, null, 2)}\n`, "utf8");
await rename(temporary, path);
console.error(`rewrote ${changed} quant labels in ${path}`);
for (const sample of samples) console.error(`  ${sample}`);
