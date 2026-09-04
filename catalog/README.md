# Curated Hugging Face catalog

`catalog.json` is the public manifest consumed by Localmotive's **HF Catalog**
tab. It controls which GGUF builds appear; model files are always downloaded
directly from Hugging Face.

## Why this is not a database

The catalog is small, public, read-heavy, and manually curated. A database and
API would add hosting, authentication, migrations, backups, uptime, and an
attack surface without improving this workload. The manifest is instead served
by `raw.githubusercontent.com`, which provides CDN delivery and ETag-based
conditional requests. The app caches the last valid response and embeds a
release-time fallback.

Write access is already private: only repository collaborators can change the
manifest. Everyone can read it because every app user must receive the list.
The git history is the audit log and a bad edit is one revert away.

For a private *curation workflow*, keep candidate notes or source data in a
separate private repository, then publish only the validated `catalog.json`
here. Do not put tokens, unreleased plans, or private notes in this public file.
A separate database becomes useful only when the catalog needs per-user data,
real-time writes, or thousands of records with server-side queries. Network
catalogs are accepted only with a detached Ed25519 signature from the maintainer
key embedded in the application; repository or CDN compromise cannot authorize
an unsigned replacement.

## Schema version 1

```json
{
  "schemaVersion": 1,
  "updated": "YYYY-MM-DD",
  "source": "https://…",
  "note": "…",
  "models": [
    {
      "id": "stable-lowercase-slug",
      "repo": "owner/repository",
      "family": "Display family",
      "parameters": "8B",
      "publisher": "owner",
      "summary": "Optional factual summary",
      "tags": ["general", "code"],
      "gated": false,
      "downloads": 0,
      "likes": 0,
      "files": [{
        "quant": "Q4_K_M",
        "filename": "model-Q4_K_M.gguf",
        "sizeBytes": 1,
        "sha256": "64 hexadecimal digits from Hugging Face LFS"
      }]
    }
  ]
}
```

Only direct GGUF targets belong in `files`. Do not list `mmproj`, draft,
DSpark, DFlash, EAGLE, or MTP companions as standalone downloads. Version 1
lists single-file builds; add an explicit shard-group schema before listing a
split model.

Every file requires its exact Hugging Face LFS SHA-256. The client verifies the
completed bytes against that curator-published digest; byte length or an optional
HTTP ETag is never sufficient integrity proof.

## Curating a change

1. Add or remove a repository in `scripts/build_catalog.mjs`. Keep the selection
   intentional: popularity alone is not acceptance.
2. Dispatch **Publish curated catalog** in GitHub Actions. The private Ed25519
   key is stored as the `CATALOG_SIGNING_KEY_PEM` repository secret; the workflow
   resolves live metadata, writes `catalog.json` plus `catalog.json.sig`, validates
   both, and commits only the public pair. The private key never enters the tree.
3. Run `npm run catalog:validate` and `cargo test shipped_catalog_file_is_valid`.
4. Review the diff. Open every new Hugging Face repository and confirm its model
   card, licence, architecture, quant labels, and that the selected file is a
   directly serveable GGUF.
5. Clients receive the new signed document and ETag after the workflow commit
   reaches `main`. An invalid or missing signature falls back to the last signed
   cache, then to the catalog bundled with the app.

Never invent repositories, filenames, sizes, download counts, licence status,
or benchmark claims. If the API cannot verify a value, omit the candidate.

## Operating alternatives

- **Current recommendation:** this repository + raw GitHub CDN. No infrastructure.
- **Stronger separation:** a private admin repository generates and validates the
  manifest, then an owner-approved workflow publishes only JSON to a public
  catalog repository or Cloudflare R2 bucket. This hides curation notes, not the
  final list, and requires a narrowly scoped deploy credential.
- **Database/API:** use only after requirements become dynamic. Put a CDN in
  front, return ETags, keep the v1 JSON response backward-compatible, and retain
  the bundled/cache fallbacks. Never proxy Hugging Face model bytes.
