// Sign the exact catalog candidate in CI. The private Ed25519 key lives only
// in the CATALOG_SIGNING_KEY_PEM repository secret and never enters the tree.
// Local builds cannot sign: run the catalog workflow instead.
//   node scripts/sign_catalog_candidate.mjs   (CI only, needs the secret env)
import { readFile, writeFile } from "node:fs/promises";
import { createPrivateKey, sign } from "node:crypto";

const key = process.env.LOCALMOTIVE_CATALOG_SIGNING_KEY_PEM ?? "";
if (!key) {
  console.error("LOCALMOTIVE_CATALOG_SIGNING_KEY_PEM is required to sign the catalog");
  process.exit(1);
}
const body = await readFile("catalog/catalog.json");
const signature = sign(null, body, createPrivateKey(key)).toString("base64");
await writeFile("catalog/catalog.json.sig", signature + "\n", "utf8");
console.error("signed catalog/catalog.json with the maintainer Ed25519 key");
