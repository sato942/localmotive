# Curated Hugging Face catalog

`catalog.json` is the public manifest consumed by Localmotive's **HF Catalog**
tab. It controls which GGUF builds appear; model files are always downloaded
directly from Hugging Face.

## Why this is not a database

The catalog is small, public, read-heavy, and curated by an allowlist builder.
A database and API would add hosting, authentication, migrations, backups,
uptime, and an attack surface without improving this workload. The manifest is
instead served by `raw.githubusercontent.com`, which provides CDN delivery and
ETag-based conditional requests. The app caches the last valid response and
embeds a release-time fallback.

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

Local SQLite lives on the PC only: the app mirrors the verified catalog into a
local database for first-start fill, refresh with cooldown, and fast
filter/sort. SQLite is never the network drop; the signed JSON stays the
network contract. No server is involved.

## Schema version 2

```json
{
  "schemaVersion": 2,
  "updated": "YYYY-MM-DD",
  "source": "https://…",
  "note": "…",
  "providers": {
    "source": "catalog/providers.json",
    "cutoffDays": 90,
    "allowlist": ["unsloth", "bartowski"]
  },
  "models": [
    {
      "id": "stable-lowercase-slug",
      "repo": "owner/repository",
      "family": "Display family",
      "parameters": "",
      "publisher": "owner",
      "author": "owner",
      "summary": "Optional factual summary",
      "tags": ["general", "code"],
      "gated": false,
      "downloads": 0,
      "likes": 0,
      "license": "apache-2.0",
      "pipeline_tag": "text-generation",
      "library_name": "",
      "architecture": "qwen35",
      "lastModified": "2026-08-20T12:04:25.000Z",
      "createdAt": "2026-08-13T08:28:40.000Z",
      "files": [{
        "quant": "Q4_K_M",
        "filename": "model-Q4_K_M.gguf",
        "sizeBytes": 1,
        "sha256": "64 hexadecimal digits from Hugging Face LFS",
        "lastModified": "2026-08-20T12:04:25.000Z",
        "createdAt": "2026-08-13T08:28:40.000Z"
      }]
    }
  ]
}
```

File keys stay snake_case (`pipeline_tag`, `library_name`) because the Ed25519
signature covers those exact bytes. The Rust parser also accepts camelCase
spellings for forward compatibility, proven by
`snake_file_keys_parse_to_real_values`.

Only direct single-file GGUF targets belong in `files`. Do not list `mmproj`,
draft, DSpark, DFlash, EAGLE, or MTP companions as standalone downloads.
Subdirectory files are excluded at build time. Filenames must be unique
case-insensitively across the whole catalog because the client downloads into
one shared folder; the builder dedupes by filename with the most-downloaded
publisher winning.

Every file requires its exact Hugging Face LFS SHA-256. The client verifies the
completed bytes against that curator-published digest; byte length or an optional
HTTP ETag is never sufficient integrity proof.

Version 1 files fail closed with an upgrade message: without rich fields the
0.5 filters cannot be honest about what they hide.

## Curating a change

1. Add or remove an author in `catalog/providers.json`. Keep the selection
   intentional: popularity alone is not acceptance. The builder reads this
   file; the app binary holds no author list.
2. Dispatch **Publish curated catalog** in GitHub Actions. The build job
   rebuilds `catalog.json` from the allowlist without secrets and validates
   structure with `--no-signature`. The sign job signs the exact candidate
   with the private Ed25519 key from the `CATALOG_SIGNING_KEY_PEM` repository
   secret, verifies the detached signature, and retains both artifacts. The
   private key never enters the tree.
3. Check the signed pair into `catalog/catalog.json` plus
   `catalog/catalog.json.sig`. Run `npm run catalog:validate` and
   `cargo test shipped_catalog_file_is_valid`.
4. Review the diff. Open every new Hugging Face repository and confirm its model
   card, licence, architecture, quant labels, and that the selected file is a
   directly serveable GGUF.
5. Clients receive the new signed document and ETag after the workflow commit
   reaches `main`. An invalid or missing signature falls back to the last signed
   cache, then to the catalog bundled with the app.

The builder filters repos by `lastModified` within `cutoffDays` (90), includes
all root-level single-file `.gguf` files, retries HTTP 429 with backoff, and
requires a per-file SHA-256. A repo that cannot be resolved is skipped without
replacing the last good catalog. Live dry-run: `node scripts/dryrun_catalog.mjs`.

Never invent repositories, filenames, sizes, download counts, licence status,
or benchmark claims. If the API cannot verify a value, omit the candidate.

## Operating alternatives

- **Current recommendation:** this repository + raw GitHub CDN. No infrastructure.
- **Stronger separation:** a private admin repository generates and validates the
  manifest, then an owner-approved workflow publishes only JSON to a public
  catalog repository or Cloudflare R2 bucket. This hides curation notes, not the
  final list, and requires a narrowly scoped deploy credential.
- **Database/API:** use only after requirements become dynamic. Put a CDN in
  front, return ETags, keep the v2 JSON response backward-compatible, and retain
  the bundled/cache fallbacks. Never proxy Hugging Face model bytes.
