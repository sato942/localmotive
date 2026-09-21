# Qualification producer/consumer map — 0.6.0 (U06-02)

Single map for all 18 mandatory manifest records (`ATTESTATION_RECORDS` in
`scripts/build_qualification_manifest.mjs`). Each row names the producer
executable, its prerequisites, output path, source/byte binding, artifact
transfer, and consuming validation. Status is per-record at map date.

Map date: 2026-09-14. Map source: `3a79a54`.
Committed manifest source: `e9a36b3aa075d72713d88af4cc74d64d561f12f8`
(`release-evidence/0.6.0/qualification-manifest-0.6.0.json`).
Current main: `3a79a54` — the eight source-bound records below still bind
`e9a36b3` and MUST be regenerated against the final candidate before
qualification. Nothing here re-labels them.

## Records regenerated per candidate (8 source-bound)

| Record | Producer / prerequisites | Output path | Binding | Transfer | Validation |
|---|---|---|---|---|---|
| `installer_payload_identity` | `node scripts/verify_installer_payloads.mjs <artifactDir> <inventoryPath> [evidencePath]`; needs the 3 staged artifacts + producer inventory; 7-Zip for extraction; `--functional` for payload launch probe | `release-evidence/0.6.0/attestations/installer-payload-identity.json` | `sourceRevision` + `inventorySha256` inside the record; verifier checks both against manifest/inventory | Committed file; qualification consumes the checkout copy | `verify_qualification_manifest.mjs` payload-identity contract |
| `mt06_cancellation` | `node scripts/g05_mt06_cycles.mjs <debugPort> [cycles] <appPid> [sourceRevision] [portableDigest]`; needs packaged app over CDP + `MT06_EVIDENCE_PATH` + `.hermes-0.6/g05-tls/api-key.txt` + managed runtime/model | `MT06_EVIDENCE_PATH` (staged to `release-evidence/0.6.0/attestations/mt06-cycles-result.json`) | `sourceRevision` + `portableDigest` args recorded in the document; verifier checks both | Committed file; qualification consumes the checkout copy | Manifest mt06 contract (schema, source, digest, checks, replacement record) |
| `rt06_all_backends` | `node scripts/g05_rt06_all_backends.mjs <cdpPort> <adapterId> [installBatch]`; needs packaged app over CDP + selected adapter + evidence/log paths | `release-evidence/0.6.0/attestations/rt06-all-backends.json` (+ `rt06-full-run.log`) | `sourceRevision` + `portableDigest` from env recorded in the document; installed/refused scope lists | Committed files; qualification consumes the checkout copies | Manifest rt06 contract (schema, source, digest, installed/refused counts) |
| `witness_timeout` | `scripts/sandbox/test-fault-evidence.ps1` (leg 2, `-FaultSimulation timeout`); needs staged candidate installers in `-CandidateDir` | `release-evidence/0.6.0/attestations/witness-timeout.json` | `sourceRevision` + `candidateDigests` + `candidateInventorySha256` bound by `host-run-lifecycle.ps1` | Committed file | Manifest witness contract (TIMEOUT at sandbox-timeout) |
| `witness_malformed_result` | `scripts/sandbox/test-fault-evidence.ps1` (leg 3, `-FaultSimulation malformed-result`); same installer prerequisite | `release-evidence/0.6.0/attestations/witness-malformed-result.json` | Same harness binding | Committed file | Manifest witness contract (FAIL at sandbox-run) |
| `witness_preservation_missing` | `scripts/sandbox/test-fault-evidence.ps1` (leg 5, `-FaultSimulation preservation-missing`); same installer prerequisite | `release-evidence/0.6.0/attestations/witness-preservation-missing.json` | Same harness binding | Committed file | Manifest witness contract (FAIL at preservation-verification) |
| `witness_stale_lock` | `scripts/sandbox/test-fault-evidence.ps1` (stale-lock leg); same installer prerequisite | `release-evidence/0.6.0/attestations/witness-stale-lock.json` | Same harness binding | Committed file | Manifest witness contract (FAIL at resolve-installers) |
| `witness_live_lock` | `scripts/sandbox/test-fault-evidence.ps1` (live-lock leg); same installer prerequisite | `release-evidence/0.6.0/attestations/witness-live-lock.json` | Same harness binding | Committed file | Manifest witness contract (FAIL at resolve-installers) |

## Records with supported carry-forward or log scope (10)

| Record | Producer / prerequisite | Current status |
|---|---|---|
| `packaged_verification` | `scripts/verify_packaged_matrix.ps1` in release.yml `package` job; staged via `--stage-packaged-verification` into the manifest (F9-04, never the stale committed copy) | Fresh per candidate by construction |
| `lifecycle_upgrade_v0.4.0` | `scripts/sandbox/host-run-lifecycle.ps1 -PreviousTag v0.4.0 -PreservationFlavor cache` in release.yml `clean-account-lifecycle` job; needs Windows Sandbox + published baseline | Carried from freeze8-da091a4 with ledger justification; MUST be re-executed per U06-04 (carry-forward is not a waiver) |
| `lifecycle_upgrade_v0.5.0` | Same harness, `-PreviousTag v0.5.0 -PreservationFlavor mirror` | Carried from freeze9-eb01bc9 with justification; MUST be re-executed per U06-04 |
| `lifecycle_preservation_v0.4.1` | Same harness, `-PreviousTag v0.4.1 -PreservationFlavor cache` | Carried from freeze9-eb01bc9 with justification; MUST be re-executed per U06-04 |
| `lifecycle_preservation_v0.5.0` | Same harness, `-PreviousTag v0.5.0 -PreservationFlavor mirror` | Carried from freeze8-da091a4 with justification; MUST be re-executed per U06-04 |
| `witness_missing_assets` | `scripts/sandbox/test-fault-evidence.ps1` (leg 1, missing-assets fixture dir) | Current; re-run with the candidate |
| `rt06_full_run_log` | Console log of the rt06 producer run | Current; re-captured with the rt06 run |
| `a11y_packaged_verification` | `node scripts/verify_a11y.mjs [exePath]` against the packaged binary | A11Y_PASS re-executed 2026-09-14 on current binary, byte-identical |
| `dc04_command_path` | `node scripts/g05_dc04_override.mjs <cdpPort> <fixturePort> <destinationRoot>` + loopback fixture | Committed log scope retained; re-run with the candidate |
| `rt04_delayed_download` | `bash .hermes-0.6/run-rt04v2.sh` + packaged candidate | Committed log scope retained; re-run with the candidate |

## Validation rehearsal (2026-09-14, current checkout)

- `node scripts/verify_qualification_manifest.mjs --expect-source e9a36b3…`: all 18 records PASS + 4 carry-forward notes; exactly 1 FAIL — `workflow file digest drifted: .github/workflows/release.yml` (manifest pins `091a0d16…`, checkout is `6cbf7f99…` after the U06-01 extraction). This is the expected staleness signal, not a waiver.
- Negative controls (missing record, old-source/current-inventory, wrong bytes) are covered by `scripts/tests/qualification_manifest.test.mjs` (28/28 pass) — no historical bytes were edited for this map.
- Full assembly validation (all 18 transferred + manifest + promotion validators green) is due with the final candidate's own records; this map does not claim it.

## Map refresh (2026-09-21, U06-02 map check only — box stays OPEN)

Map source (branch HEAD): `1dc0793`. Committed manifest
(`release-evidence/0.6.0/qualification-manifest-0.6.0.json`) binds source
`e9a36b3aa075d72713d88af4cc74d64d561f12f8`. Working manifest
(`.hermes-0.6/qualification-manifest-0.6.0.json`) binds `da091a47ca6595ec37690ab5ae63542671c5f7f6`.
Neither binds the current HEAD. All 18 `ATTESTATION_RECORDS` files are
present in `release-evidence/0.6.0/attestations/` (checked 2026-09-21);
presence is not HEAD binding.

Per-record HEAD binding at `1dc0793`: none. Every source-bound record still
carries an older revision (lifecycle/mt06/negative rows `da091a4`; witness
rows `85edee6`; negatives at `7c32fa9`/`cb8ab64`). The eight source-bound
records named above MUST still be regenerated against the final candidate;
nothing here re-labels them.

Gaps (exact text from `release-gates.test.mjs` "the qualification manifest
validates and references only committed records", run 2026-09-21 — listed,
not repaired in this pass): workflow file digest drifted
(`.github/workflows/release.yml`); `lifecycle_upgrade_v0.4.0`,
`lifecycle_upgrade_v0.5.0`, `lifecycle_preservation_v0.4.1`,
`lifecycle_preservation_v0.5.0`, `installer_payload_identity`: sha256
mismatch at their `release-evidence/0.6.0/attestations/` paths; the four
lifecycle rows: carried-forward record is not a parsed document.

**U06-02 remains OPEN.** No release/qualify/promote run was executed for
this refresh; no hash was rewritten and no attestation regenerated.
