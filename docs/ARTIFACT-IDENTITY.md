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
- The **measurement snapshot** combines runtime identity, selected-artifact
  content identities, effective arguments and workload identity. A matching
  filename or header alone is insufficient. Unknown identity fields remain
  explicit. LoRA identity includes the ordered adapter contents and f32 scale
  values. No-adapter and single ordinary-adapter identities keep their earlier
  representation. Older multiple/scaled-adapter records may need a fresh
  benchmark; the application does not rewrite them. Identity coverage does
  not prove adapter-to-model or runtime launch compatibility.
- The **catalog mirror** stores verified rows plus user rows; user rows never
  authorize network files and are marked `USER ADDED · LOCAL ONLY`. The
  mirror is a convenience index over verified content, not an authority.

## Command arguments in local records

Managed model, draft and LoRA command-path values use SHA-256 fingerprints.
Replay accepts the current or prior argument encoding only when it matches
the current raw arguments. Model identity and compatibility-key checks still
apply. Replay does not rewrite old files. Ambiguous old values containing the
reserved `sha256:` marker require a fresh record.

This is not full anonymization. Other fields can contain local names or paths,
and a path fingerprint permits comparison and dictionary guesses. Do not
publish raw local records as redacted evidence.

## Consumer review (S-02.I1)

- `catalog_db.rs` — stores rows admitted by signature-verified catalogs or
  marked user rows; never promotes a user row into curated authority.
- `calibration.rs` / `evidence.rs` — bind runs to execution snapshots and retain
  known content identities plus explicit unknown fields.
- Calibration fitting, quality suites and share export were removed from the
  0.6 source build. `calibration.rs` retains execution-identity helpers only.
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

## Companion matching is a heuristic (audit S-13.I2)

Companion discovery uses filename and quantised-size evidence: the family
prefix, the role token in the name, and ranking by quantisation closeness to
the selected target build. It does **not** read companion GGUF headers during
a scan, so it can rank an unrelated file first in a mixed-family folder. The
available stronger signal is explicit provenance: the user's own selection in
the profile and the role shown next to each candidate. The runtime's `--help`
constrains method and flag selection; it does not prove target/companion
compatibility.

A retained draft path is reconciled with the selected speculative method:
`reconcileDraftCompanion` clears the path (and the interface says so) whenever
the newly selected method does not use a draft model, so a companion chosen
for a draft method is not retained when switching to an n-gram method.
Simple, EAGLE-3, DFlash and DSpark draft methods require a companion path.
MTP can use heads in the main model; a filename alone does not prove that a
separate MTP companion is compatible.
