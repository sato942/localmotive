# Catalog candidate promotion and recovery (audit S-29)

The catalog workflow (`.github/workflows/catalog.yml`) is **manually
dispatched**. It builds, validates and signs a catalog candidate. It does
**not** publish to `main`. Promotion is a separate, human commit.

## 1. The handoff, step by step

1. The owner dispatches the workflow (Actions -> "Publish curated catalog" ->
   Run workflow). The `sign_only` input signs the checked-in catalog instead
   of rebuilding from Hugging Face.
2. `build` job: checks out the dispatch revision, runs the pinned action
   verification (`verify_workflow_pins.mjs`, `verify_workflow_gates.mjs`),
   rebuilds the catalog from the allowlist (`node scripts/build_catalog.mjs`),
   validates the structure without a signature
   (`node scripts/validate_catalog.mjs catalog/catalog.json --no-signature`),
   and uploads the **unsigned candidate** as the artifact
   `localmotive-catalog-candidate` (retention 7 days).
3. `sign` job: downloads that exact artifact, signs the exact bytes with the
   private Ed25519 key from the repository secret `CATALOG_SIGNING_KEY_PEM`
   (`node scripts/sign_catalog_candidate.mjs`), re-verifies the detached
   signature and the structure against the embedded public key
   (`node scripts/validate_catalog.mjs catalog/catalog.json`), records
   `catalog/SHA256SUMS-candidate.txt` over the body and the signature, and
   uploads the **signed pair** as the artifact `localmotive-catalog-signature`
   (retention 7 days).
4. Review: a human downloads the signed artifacts, reviews the diff of
   `catalog/catalog.json` against `main`, and confirms the recorded digests.
5. Promotion: the human commits `catalog/catalog.json` and
   `catalog/catalog.json.sig` **together** (for example with `catalog.yml`
   copy-paste or `gh run download`). The repository has no step that performs
   this commit automatically.
6. Consumption: the app fetches `catalog.json` and `catalog.json.sig`, and
   verifies the signature over the fetched body bytes against the embedded
   public key (the same key hardcoded in `scripts/validate_catalog.mjs` and in
   the Rust fetch path). A body whose signature does not match is refused and
   the previous verified copy is served instead.

The private key exists only as the CI secret. A local machine cannot sign the
catalog; that is why the unsigned candidate must travel through the workflow.

## 2. Retention, retrieval and expiry

- Retrieval window: **7 days** (the artifact retention set in the workflow).
- Retainer: the repository owner. Retrieval is `gh run download <run-id> -n
  localmotive-catalog-signature` (or the run page), before expiry.
- An **expired or never-downloaded candidate** is not recovered in place:
  artifacts are immutable and retention is enforced by GitHub. Recovery is a
  **new dispatch**, which regenerates the candidate bytes and signs again.
  Because a rebuild produces new bytes (`updated` date and live allowlist
  content), a stale signature can never be paired with rebuilt bytes - the
  verification refuses the pair (proved in the S-29 rehearsal).
- A **failed `build` job** stops before the `sign` job (`needs: build`); a
  **failed `sign` job** leaves `main` untouched. In both cases the remedy is a
  re-dispatch after fixing the cause, or `sign_only` when only the signature
  is missing for checked-in bytes.
- Preview without writing: `node scripts/dryrun_catalog.mjs` reads live HF
  metadata, reports repo/file/byte counts, and never writes the catalog and
  never needs the signing key.

## 3. How the promoted pair is checked together

- `node scripts/validate_catalog.mjs catalog/catalog.json` verifies the
  detached Ed25519 signature over the **exact file bytes** and then the
  structure (schema version, allowlist presence, row rules). It fails when
  the body changed, when the signature changed, or when the signature file is
  missing (rehearsal evidence: `.hermes-0.6/s29-rehearsal.log`).
- `catalog/SHA256SUMS-candidate.txt` (recorded in the `sign` job) binds the
  digests of both files, so review can confirm the pair before promotion.
- The app repeats the signature verification at fetch time against the same
  embedded public key; a mismatched pair is refused, and the cached verified
  copy is served with a visible refresh error (DC-01/DC-07 behavior).

## 4. Limits

- The workflow name alone proves nothing: nothing in `.github/workflows/`
  commits to `main`; promotion is the human step above.
- This document covers the catalog artifact. Release binaries, installers and
  their signatures are covered by `docs/SUPPLY-CHAIN.md` and the release
  workflow gates instead.
