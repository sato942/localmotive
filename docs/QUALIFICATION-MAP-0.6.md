# Qualification producer/consumer map — 0.6.0 (U06-02)

Single map for all 18 mandatory manifest records (`ATTESTATION_RECORDS` in
`scripts/build_qualification_manifest.mjs`). Each row names the producer
executable, its prerequisites, output path, source/byte binding, artifact
transfer, and consuming validation. Status is per-record at map date.

Map date: 2026-09-21. Map source: `c0ae53874b54fda90616f2c8afc291ae002d32d9`.
Committed manifest source: `c0ae53874b54fda90616f2c8afc291ae002d32d9`
(`release-evidence/0.6.0/qualification-manifest-0.6.0.json`).
Current branch: `fix/u06-stabilization` — all 18 records below are new
parses from the 2026-09-21 qualify campaign on that SHA (U06-02 CLOSED).
Nothing here re-labels older bytes.

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
| `lifecycle_upgrade_v0.4.0` | `scripts/sandbox/host-run-lifecycle.ps1 -PreviousTag v0.4.0 -PreservationFlavor cache` in release.yml `clean-account-lifecycle` job; needs Windows Sandbox + published baseline | New parse 2026-09-21 on `c0ae538` (Sandbox PASS, bound to freeze SHA + inventory) |
| `lifecycle_upgrade_v0.5.0` | Same harness, `-PreviousTag v0.5.0 -PreservationFlavor mirror` | New parse 2026-09-21 on `c0ae538` (Sandbox PASS) |
| `lifecycle_preservation_v0.4.1` | Same harness, `-PreviousTag v0.4.1 -PreservationFlavor cache` | New parse 2026-09-21 on `c0ae538` (Sandbox PASS) |
| `lifecycle_preservation_v0.5.0` | Same harness, `-PreviousTag v0.5.0 -PreservationFlavor mirror` | New parse 2026-09-21 on `c0ae538` (Sandbox PASS) |
| `witness_missing_assets` | `scripts/sandbox/test-fault-evidence.ps1` (leg 1, missing-assets fixture dir) | New parse 2026-09-21 on `c0ae538` (ALL WITNESS LEGS PASS) |
| `rt06_full_run_log` | Console log of the rt06 producer run | Re-captured 2026-09-21 on `c0ae538` (22/22) |
| `a11y_packaged_verification` | `node scripts/verify_a11y.mjs [exePath]` against the packaged binary | A11Y_PASS re-executed 2026-09-21 on the `c0ae538` binary |
| `dc04_command_path` | `node scripts/g05_dc04_override.mjs <cdpPort> <fixturePort> <destinationRoot>` + loopback fixture | Re-run 2026-09-21 on `c0ae538` (13/13 PASS) |
| `rt04_delayed_download` | `bash .hermes-0.6/run-rt04v2.sh` + packaged candidate | Re-run 2026-09-21 on `c0ae538` (17/17 PASS) |

## Validation (2026-09-21 qualify campaign, U06-02 CLOSED)

- `node scripts/verify_qualification_manifest.mjs --expect-source c0ae538…`: MANIFEST VERIFY PASS — source, 3 artifacts, producer inventory, workflow file, all 18 records exact, no carry-forward.
- `node --test scripts/tests/release-gates.test.mjs`: 164/164 PASS (the manifest test that drifted on stale 0.6.0 evidence is green on the new files).
- `node --test scripts/tests/qualification_manifest.test.mjs`: 28/28 PASS.
- `node scripts/verify_release_promotion.mjs --qualified . --tag v0.6.0 --expect-source c0ae538…`: local contract PASS (no publish performed; tag untouched).
- Candidate: portable `e3a7804269ef9396355af3fca9a5cd448951b65587e2377a825fb354e8bdb04d`, setup `900ac1074a526156a51ea5f315daa61f0c1ff99f814e62bf63ef2e721438e58c`, MSI `f411280b889c89177b097657ef55146ab811c65a06bec7c853b6f1b10c490a73`; inventory `ac21f7ea5ea1e58b1549006647c8c5e86df3d406a555b2478ffd93a4eb4e1353`.

## Four-identity rule (2026-09-22; stops the v0.6.1 recurrence)

Product version, producer SHA, tag, and evidence are distinct. Version `0.6.1`
was bumped once, before the freeze. Producer `4b31431` is the tree that was
built. Tag `v0.6.1` peels to that SHA and never moves. Evidence lives in
`d3085ff` and CI artifacts under `release-evidence/0.6.1/`, never in the tag
tree. A record counts only when its `sourceRevision` and candidate digests
match the producer. Digest-bound records cannot transfer between builds
(three builds of `4b31431` gave three portable digests); a green CI manifest
needs all digest-bound legs produced in that run. No 0.6.2 exists for this.

## Earlier map refresh (2026-09-21, superseded — retained for history)

Superseded by the qualify campaign above (U06-02 CLOSED 2026-09-21). Retained for history: map source was `1dc0793`; committed manifest bound `e9a36b3`; working manifest bound `da091a4`; gaps were workflow digest drift + sha256 mismatches + unparsed lifecycle rows. All resolved on the new files; no hash was rewritten to silence a gate.
