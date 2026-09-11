# Artifact identity levels

Localmotive names four levels of identity for a model artifact set. Each
consumer binds to exactly one level; a check at a weaker level never stands in
for a stronger one (audit S-02).

| Level | What it is | Where it is established |
|---|---|---|
| **Logical/header identity** | Architecture, parameter counts, quantization and other facts read from the GGUF header metadata (never tensor data). | `gguf.rs` (`GgufSummary`); shown as artifact facts. |
| **Named artifact-set identity** | The filename plan: shard indices and counts (`-00002-of-00004`), the logical name, companion ordering and roles. | `artifact.rs` (`analyze_shard_names`); the inventory's completeness status. |
| **Recorded split metadata** | Optional `split.no`/`split.count` keys inside each shard's header, when the producing tooling wrote them. | `gguf.rs` optional fields + `artifact.rs::split_metadata_verdict`. |
| **Content identity** | SHA-256 digests: per-file digests from the catalog (verified at download completion) and header SHA-256 for managed content. | `download.rs` final verification; `runtime.rs` managed verification. |

## What each level does and does not cover

- The **catalog download path** verifies content identity: the published
  per-file SHA-256 must match the finalized bytes before a file is exposed
  under its final name. Renaming identical bytes keeps the digest; changing
  any byte, including after the header, changes it.
- The **inventory** establishes named artifact-set identity from filenames
  only. It is complete when the planned indices exist; this is a naming
  claim, not a content claim.
- The **split metadata verdict** cross-checks the header against the plan
  when both sides are present. A mismatch marks the set untrustworthy; absent
  metadata leaves tensor-set completeness **unknown** — metadata alone never
  upgrades it to a claim.
- The **calibration and measurement identity chain** (execution snapshots,
  launch compatibility keys, quality suites) binds to logical/header identity
  plus launch arguments — the level needed to say "these numbers belong to
  this configuration". It does not re-hash model bytes on every run; content
  identity is enforced where bytes enter the machine (download, managed
  install) and re-checked when a managed file is executed.
  A model edited in place with the same name and header facts is therefore
  the same configuration identity but not the same content; the digest-level
  check is the download/installation gate, not the calibration key.
- The **catalog mirror** stores verified rows plus user rows; user rows never
  authorize network files and are marked `USER ADDED · LOCAL ONLY`. The
  mirror is a convenience index over verified content, not an authority.

## Consumer review (S-02.I1)

- `catalog_db.rs` — stores rows admitted by signature-verified catalogs or
  marked user rows; never promotes a user row into curated authority.
- `calibration.rs` / `evidence.rs` / `sharing.rs` — bind runs to execution
  snapshots (logical identity + effective arguments); they make no content
  claim beyond what the download/install gates established.
- `measurement.rs` — records measurements against a run identity; a changed
  runtime digest invalidates the snapshot key (MT-07/MT-09).
- `core.rs` inventory — naming-level completeness (`SHARDS COMPLETE`), with
  header-level facts surfaced separately and split metadata cross-checked
  when available.

## Tests

- `gguf::tests::s02_split_metadata_is_read_when_present_and_stays_unknown_when_absent`
- `artifact::tests::s02_split_verdict_agrees_or_reports_mismatch_and_unknown`
- MT-15 parity tests (filename plan vs shard analyzer) and DC-11 publication
  identity tests (content digest + file index).
