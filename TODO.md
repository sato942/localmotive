# Localmotive 0.6 — TODO

Single current tracker. Merged 2026-09-20 from the root `TODO-0.6.md` working
copy and `docs/history/TODO-0.6.md` (frozen; untouched). The root copy is
removed by this merge. Closed items keep their states from the working copy;
only open work and the merge register below carry forward.

Code comparison for this merge (2026-09-20, head `9436aec`):

- `cargo fmt --check`: clean. `cargo clippy --locked --all-targets -- -D
  warnings`: clean. `cargo test --locked`: 622 pass, 7 ignored.
  `release-gates` script tests: 164/164 pass.
- GitHub rulesets read live: `main-pr-check` (branch) and
  `immutable-release-tags` (tag) both active. This supports the closed
  GH-01.I3 and GH-02.I3 states.
- `reserve_operation` with `OperationOwner` exists in `src-tauri/src/lib.rs`.
  This supports the closed MT-05.I1 state.
- No `signtool`, `Get-AuthenticodeSignature`, or signing step exists in
  `.github/workflows/`. `scripts/sign-windows.ps1` and
  `src-tauri/tauri.signing.conf.json` are deleted. Releases ship unsigned as
  standing policy (owner order 2026-09-20). No open or closed box below
  requires signing work.
- `src-tauri/src/recommend.rs` and `src-tauri/src/sharing.rs` are deleted.
  No open box below requires them.
- Catalog Ed25519 signing (`catalog.yml`, `sign_catalog_candidate.mjs`) is
  untouched and unrelated to the unsigned-release policy.

Rule: a box closes only on its stated evidence. A checkbox is not evidence.

## Open work packages

- [ ] **U06-02 — OPEN.** Connect existing producers and artifact transfers before qualification; validate the entire manifest against the actual candidate.

**Trace:** V06-GH-02.V3, V06-GH-03.V1/V3, V06-GH-06.V3, V06-G-04/V06-G-05/V06-G-06, V06-G-09.I2/V1; audit [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02), [GH-06](docs/history/localmotive-comprehensive-audit.md#gh-06), [release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release). **Evidence:** E08.

Build a producer/consumer map for all 18 mandatory manifest records, naming executable, prerequisite, output path, source/byte binding, artifact upload/download, validation and any explicitly supported historical scope. Confirm that the files consumed are the files produced by this run. Eight source-bound historical inputs must be regenerated/staged:

| Records | Existing producer / prerequisite | Required binding |
|---|---|---|

**Map refresh 2026-09-21 (map check only — U06-02 stays OPEN).**
producer detail lives in `docs/QUALIFICATION-MAP-0.6.md` (refreshed same
day). Branch HEAD: `1dc0793`. All 18 `ATTESTATION_RECORDS` files are
present in `release-evidence/0.6.0/attestations/`; presence is not HEAD
binding — no record binds `1dc0793` (committed manifest binds `e9a36b3`,
working manifest binds `da091a4`).

| Record | Producer → consumer | Present? | Bound to HEAD? | Gap |
|---|---|---|---|---|
| `packaged_verification` | `verify_packaged_matrix.ps1` → manifest (staged) | yes | no (`da091a4` era) | regenerate per candidate |
| `lifecycle_upgrade_v0.4.0` / `v0.5.0` | `host-run-lifecycle.ps1` → manifest | yes | no | sha256 mismatch + not a parsed document (test 3169) |
| `lifecycle_preservation_v0.4.1` / `v0.5.0` | same harness → manifest | yes | no | sha256 mismatch + not a parsed document (test 3169) |
| `witness_missing_assets` / `timeout` / `malformed_result` / `preservation_missing` / `stale_lock` / `live_lock` | `test-fault-evidence.ps1` → manifest | yes | no (`85edee6` era) | regenerate 4 source-bound witness rows per candidate |
| `mt06_cancellation` | `g05_mt06_cycles.mjs` → manifest | yes | no | regenerate per candidate |
| `installer_payload_identity` | `verify_installer_payloads.mjs` → manifest + release-gates | yes | no | sha256 mismatch (test 3169) |
| `dc04_command_path` | `g05_dc04_override.mjs` + loopback → manifest | yes | log scope retained | re-run with the candidate |
| `rt04_delayed_download` | `run-rt04v2.sh` + packaged candidate → manifest | yes | log scope retained | re-run with the candidate |
| `a11y_packaged_verification` | `verify_a11y.mjs` → manifest | yes | A11Y_PASS stands | none for U06-02 |
| `rt06_all_backends` / `rt06_full_run_log` | `g05_rt06_all_backends.mjs` → manifest | yes | no | regenerate per candidate |
| release.yml digest | — | — | — | workflow file digest drifted (test 3169) |

Eight source-bound historical inputs (must be regenerated/staged, not
carried): `installer_payload_identity`, `mt06_cancellation`,
`rt06_all_backends` (+ log), `witness_timeout`, `witness_malformed_result`,
`witness_preservation_missing`, `witness_stale_lock`, `witness_live_lock`.

**U06-02 remains OPEN.** No release/qualify/promote run executed; no hash
rewritten; no attestation regenerated.

---

- [x] **U06-04 — DONE (2026-09-20).** Executed the four required current-candidate legs.

**Trace:** R06-01; V06-GH-04.V3, V06-G-06.I2, V06-GH-03.V1/V3, V06-GH-06.V3; audit [GH-04](docs/history/localmotive-comprehensive-audit.md#gh-04), [release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release).

| Leg | Baseline | Persistence |
| 1 | v0.4.0 | upgrade + cache |
| 2 | v0.4.1 | preservation / cache (profiles) |
| 3 | v0.5.0 | upgrade + mirror |
| 4 | v0.5.0 | preservation / mirror (SQLite + user-override) |

**Result (2026-09-20, candidate `fb0776b6b499fc4c26121664fe3b7162e0a15d75`):** all four legs PASS on the same installer bytes. Setup SHA-256 `fb21f7d17ab2d88d098fee596e3568b62bf9cf4bb68bc525610a8fb356383b25` (`5119018` bytes). MSI SHA-256 `a9015b926506536c14b993cf89a4b0fe4e7f469cfc24f33db56077f2921de77c` (`8798208` bytes). Inventory SHA-256 `58e82b4116ce9010aa9f58c8c876dc08b5975604776b8527fb1073646c8a7a84` (`artifacts/candidate-inventory-0.6.0.json`). Each leg records exact installed identity (`0.4.0`/`0.4.1`/`0.5.0` before update, `0.6.0` after), strict uninstall outcomes, and `preservation: PASS`. Both cache legs recovered `6/6` seeded settings keys with exact equality including the tuning value (retained digest `2b7ba23c5f06c8b7abef7d93a2c0114f50f838f3f725c97b9cb129d5ff8f8391`). Both mirror legs retained schema `1` with user-override rows (`user_sourced=1`) and the userdata canary. Evidence: `release-evidence/0.6.0/attestations/sandbox-clean-account-lifecycle-upgrade-v0.4.0.{json,log}`, `sandbox-clean-account-lifecycle-upgrade-v0.5.0.{json,log}`, `sandbox-clean-account-lifecycle-preservation-v0.4.1.{json,log,collected-settings.json}`, `sandbox-clean-account-lifecycle-preservation-v0.5.0.{json,log}`. Matrix log: `.hermes-0.6/u06-04-matrix-fb0776b.log`. No boot failure occurred, so no rerun was needed. Slot is free: `wsb.exe list` is empty and the harness removed its owning locks.

---

- [x] **U06-05 — CLOSED by owner disposition 2026-09-20.** Solo-repo gate cost reduction; the two-actor rehearsal requirement is superseded.

**Trace:** V06-GH-01.I3/V1/V2/V3, V06-GH-02.I3; audit [GH-01](docs/history/localmotive-comprehensive-audit.md#gh-01), [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02). **Evidence:** E03/E04/E05 as settings-and-policy records, not as a two-actor refusal.

Solo-maintainer policy: the owner may author and merge a PR without a second reviewer, but strict required checks still apply; normal and emergency fixes use the protected PR path unless the owner uses the documented owner-only bypass. No standing bypass actor exists for ordinary contributors. Any exceptional policy change needs a specifically recorded disposition and effective-settings readback.

U06-05 CLOSED by owner disposition 2026-09-20.

GH-01.I3: main-pr-check and immutable-release-tags are active.
GH-01.V1: pr-check already runs on pull_request to main under that stable job name on windows-latest. No new smoke PR required.
GH-01.V2: dedicated failing-PR campaign is not a release blocker for a solo repo. Optional. Do not merge a failing revision if executed.
GH-01.V3: waived as an access prerequisite. No Write collaborator exists and none will be created for this proof. Effective ruleset readback recorded. Owner is the sole documented bypass actor on main-pr-check. Ordinary-contributor direct-push refusal was not attempted and must not be fabricated. V3 reopens when the first Write collaborator is invited.

Evidence: live ruleset readback after owner-bypass edit on 2026-09-20 (main-pr-check 23218749: enforcement active, refs/heads/main, required context pr-check with strict policy, pull_request required_approving_review_count 0, non_fast_forward plus deletion, bypass_actors owner sato942 id 2147851 only; immutable-release-tags 23218751 unchanged and active, refs/tags/v*, update plus deletion, no bypass); this disposition; current collaborator state (owner only). E03/E04/E05 as settings-and-policy records, not as a two-actor refusal.

---

- [ ] **U06-06 — OPEN.** Select a fully reviewed source containing the corrections and obtain one successful, fully qualified final producer run.

**Trace:** V06-GH-02.I3/V3, V06-GH-03.V1/V3, V06-G-09.I1/I2; audit [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02), [GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).

Finish U06-01/02/03 and their actual preflight before another final tag attempt. The existing tag remains at 27349f9 while current main contains later fixes. Present the concrete source, checks, publication state and tag/version policy disposition if a different final SHA is needed. A new SHA under an already-used immutable tag is a real conflict, not another general release permission request. No silent temporary bypass or retargeting loop.

Current workflow facts: dispatching `release.yml` from main with the old tag still checks out that tag's source; re-running an old run does not load new committed workflow code; promotion requires a successful **push-event** Release verify at the tag's unchanged peeled SHA. Do not invent an RC-tag/version convention or weaken that contract to avoid reconciliation. An off-tag rehearsal is valid diagnostic work but not a substitute final producer run.

Acceptance: applicable required CI, resolve, audit, quality, package, native lifecycle and qualification all succeed for the identified candidate; qualified bundle, inventory and every record agree. Record tag object, peeled SHA, run/attempt ID, workflow/harness identity and each artifact digest. New outputs get their own digests even when product source is unchanged.

---

- [ ] **U06-07 — OPEN.** Complete FE-05 evidence-based acceptance and the release decision under standing owner delegation; persist precise residual scope.

**Trace:** R06-02, V06-G-08.V1, V06-FE-05.I1-I5/V1-V3, V06-G-05.I3, V06-S-25.I3; audit [FE-05](docs/history/localmotive-comprehensive-audit.md#fe-05), [release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance).

Reuse accepted FE-05 navigation, completion visibility, two-profile saved-manifest/anchor provenance and I1-I5 regression mappings. Do not invent a personal owner review or repeat the A/B walkthrough solely to generate a signature. Record who made the delegated evidence assessment and cite the actual owner authorization.

Hardware D06-01..D06-05 is CLOSED-DEFERRED-FAR-FUTURE 2026-09-21 (owner scope): 0.6 will not obtain other-vendor GPUs, a second adapter, HDD/SATA targets, or an OS-crash/power-loss rig, and U06-07 does not wait on those rows. Not executed, not PASS. For S-25, the split is: (1) independent benchmark distributions and baseline drift stay BLOCKED-INPUT; (2) held-out calibration error and interval coverage stay BLOCKED-INPUT; (3) release queue/failure/evidence-retention metrics are collected in V06-S-25.I3. The tuner history table is in-sample evidence and closes neither science third. For G-05, retain the automated a11y PASS scope; manual Narrator/NVDA listening and authorized live-account scenarios are CLOSED-DEFERRED-USER-REPORTS 2026-09-21 and U06-07 does not wait on them. If a non-hardware residual is eligible for deferral under the existing Medium/Low/supplemental policy, make a separately reasoned delegated decision with owner, risk, workaround, supported scope, milestone and evidence gap; do not label the current hardware instruction as that decision. Mandatory High/native gates stay required.

Acceptance: current evidence supports the stated release scope, no unresolved mandatory High/native gate is hidden, all unavailable hardware coverage is excluded honestly, every remaining non-hardware criterion has an executed result or policy-eligible explicit disposition. Final ship decision is based on prepublication evidence; publication/readback is the next action, not a circular prerequisite to that decision.

---

- [ ] **U06-08 — OPEN.** Complete the existing promotion controls, dry run, publication and public byte readback under standing authorization.

**Trace:** V06-G-09.I1/I2/I3/V1, V06-GH-02.V3, V06-GH-06.V3; audit [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02), [GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03), [stabilization exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release).

Use `release-promote.yml` with the final tag, selected successful verify run and exact confirm phrase `PUBLISH v0.6.0` (adjust only for an explicitly selected different version). Run `dry_run=true` first; actual publication uses `dry_run=false`. Promotion consumes the qualified bundle, never a fresh rebuild. Wrong source/tag, modified bytes, missing asset or missing lifecycle evidence must refuse publication in nonpublishing controls.

Acceptance: real release exists with the complete intended asset set; download public assets and compare SHA-256 and inventory/record identities to the qualified producer. Recheck tag, version, Latest/prerelease state and unsigned/SmartScreen/checksum wording. Inspect artifact contents, not merely successful upload-step labels. Preserve diagnostic and readback records. A successful dry run is not a published release.

---

- [ ] **U06-09 — OPEN.** Finish one current closeout record and update this tracker from evidence.

**Trace:** V06-G-10.I1/I2/I3/V1, V06-G-08.V1; audit [release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance), [QD-04](docs/history/localmotive-comprehensive-audit.md#qd-04), [GH-07](docs/history/localmotive-comprehensive-audit.md#gh-07).

Record final source/tag/release/run identities, exact public artifact digests, commands/results, verified task IDs, named deferrals and remaining limitations. Check only completed criteria; retain DEFERRED-OWNER boxes unchecked. Preserve original audit and frozen evidence. Historical main/CI/tag observations keep their own dates and SHAs. Confirm no unreconciled owned processes remain. Update this tracker and its handoff from the same evidence; avoid a new documentation-only loop for unchanged results.
## Open criterion ledger

- [x] **V06-RT-06.V3 (seven-installed-backend acceptance portion)** — CLOSED-DEFERRED-FAR-FUTURE 2026-09-21 (owner scope). Not executed. Not PASS. 0.6 will not obtain the required hardware. Supported evidence remains: single-vendor NVIDIA host (RTX 5090), NVMe only, process-crash tests only. Reopen only if the owner later provides the missing environment and asks for that campaign. A deferral is not substitute evidence that the missing configuration works. The acceptance wording asks for all seven backends installed on a representative host; this host runs one GPU vendor (NVIDIA, adapter `luid:0000000000014f4f`), so the maximum eligible set is four installed/selected/launched backends plus the three vendor refusals. **Trace:** [Audit RT-06](docs/history/localmotive-comprehensive-audit.md#rt-06).
  **Closed 2026-09-21.** D06-01 seven-installed hardware coverage is out of 0.6 scope, not an open gate. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L351).

---

- [x] **V06-DC-12.V3** — CLOSED-DEFERRED-FAR-FUTURE 2026-09-21 (owner scope). Not executed. Not PASS. 0.6 will not obtain the required hardware. Supported evidence remains: single-vendor NVIDIA host (RTX 5090), NVMe only, process-crash tests only. Reopen only if the owner later provides the missing environment and asks for that campaign. A deferral is not substitute evidence that the missing configuration works. Distinguishing ordinary process-crash testing from target-environment OS-crash/power-loss validation, and throughput for 1/4/8 connections on HDD, SATA SSD, and NVMe targets, needs a storage-class rig and a power-loss facility this project does not have. **Trace:** [Audit DC-12](docs/history/localmotive-comprehensive-audit.md#dc-12).
  **Closed 2026-09-21.** D06-02 storage-class and OS-crash/power-loss facility is out of 0.6 scope, not an open gate. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L743).

---

- [x] **V06-GH-01.V2** — Introduce a controlled failing check in the test PR; verify merge is blocked until corrected, then confirm the corrected commit obtains the required successful statuses. **Trace:** [Audit GH-01](docs/history/localmotive-comprehensive-audit.md#gh-01).
  **Current status: CLOSED by owner disposition 2026-09-20.** Dedicated failing-PR campaign is not a release blocker for a solo repo. A controlled failing PR remains optional owner work if a real regression needs it; do not merge a failing revision if executed. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1704).

---

- [x] **V06-GH-01.V3** — Read back effective branch/ruleset settings with appropriate access, record bypass actors and restrictions, and verify ordinary contributors cannot circumvent required checks through direct pushes. **Trace:** [Audit GH-01](docs/history/localmotive-comprehensive-audit.md#gh-01).
  **Current status: WAIVED by owner disposition 2026-09-20 as an access prerequisite.** No Write collaborator exists and none will be created for this proof. Effective readback recorded: main-pr-check 23218749 active on refs/heads/main with required context pr-check (strict), pull_request required_approving_review_count 0, non_fast_forward plus deletion, bypass_actors owner sato942 (id 2147851) only; classic branches/main protection is absent (expected; rulesets are the gate). Ordinary-contributor direct-push refusal was not attempted and must not be fabricated; owner bypass is not contributor proof. V3 reopens automatically when the first Write collaborator is invited; run a direct-push refusal with that identity before that grant takes effect. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1705).

---

- [x] **V06-GH-02.I3** — CLOSED 2026-09-20 on live readback + standing policy. Protect released version tags against updates and deletion, document one-time tag creation, and require a new prerelease or patch version when source changes after an earlier candidate. **Trace:** [Audit GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02).
  **Closed:** live `GET /repos/sato942/localmotive/rulesets/23218751` returns `immutable-release-tags`, enforcement `active`, target `refs/tags/v*`, rules `update` + `deletion`, `bypass_actors` empty, `current_user_can_bypass` never. Standing policy: released version tags (`refs/tags/v*`) are created once and never updated or deleted. If source changes after a candidate or a failed attempt, use a new prerelease or patch name. Do not retarget a used tag. `immutable-release-tags` is the gate. No bypass actor. No tag was published by this close. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1724).

---

- [ ] **V06-GH-02.V3** — Run a complete candidate flow and compare checkout revisions, inventory, packaged evidence and publication metadata; read back the effective tag-update/deletion protection. **Trace:** [Audit GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02).
  **Current status: OPEN.** Split 2026-09-20: the tag-protection readback sub-ask is satisfied by the same live JSON as V06-GH-02.I3 (`immutable-release-tags` 23218751, active, `refs/tags/v*`, `update` + `deletion`, no bypass). The complete candidate flow vs checkout / inventory / packaged evidence / publication metadata stays OPEN, blocked on U06-06 then U06-08. No fixture or older release stands in as publication proof. U06-02/U06-06/U06-08: full candidate identities and publication metadata must agree. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1731).

---

- [ ] **V06-GH-03.V1** — Exercise a fresh-version release in a single-runner configuration; the lifecycle job must start only after candidate artifacts exist, without release-not-found polling blocking the producer. **Trace:** [Audit GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: OPEN.** U06-02/U06-04/U06-06: complete final producer chain required. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1755).

---

- [ ] **V06-GH-03.V3** — Run the selected lifecycle path against the exact candidate bytes and confirm publication behavior matches the documented required/non-gating policy, including failure and cancellation outcomes. **Trace:** [Audit GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: OPEN.** U06-04/U06-06: native lifecycle and observed failure/cancellation gating required. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1757).

---

- [ ] **V06-GH-04.V3** — Run clean-account installer scenarios and both version-specific migration fixtures on Windows; compare exact installed identity, retained data, uninstall outcomes and structured per-scenario verdicts. **Trace:** [Audit GH-04](docs/history/localmotive-comprehensive-audit.md#gh-04).
  **Current status: OPEN.** R06-01 / U06-04: corrected current native installer and preservation observations remain required. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1783).

---

- [ ] **V06-GH-06.V3** — Inspect the completed workflow's artifact collection, not just its upload-step conclusion, and confirm the summary accurately reports both verification outcome and evidence availability. **Trace:** [Audit GH-06](docs/history/localmotive-comprehensive-audit.md#gh-06).
  **Current status: OPEN.** U06-02/U06-06/U06-08: inspect actual retained/downloaded evidence and publication asset contents. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1835).

---

- [x] **V06-GH-10.V3** — CLOSED 2026-09-21 on this-host comparison (existing Windows NVIDIA host; no new hardware, no fixtures, no invented MATCH). The recorded E07 parse defect (run 34819219650 trailing quote) is already repaired: `hardware qualify attestation block parses under pwsh (U06-03)` PASS 2026-09-21 under the real PowerShell parser, so no new parser fix was needed. Generator run 2026-09-21 with directly observed values (CPU `AMD Ryzen 9 9950X3D 16-Core Processor`, GPU `NVIDIA GeForce RTX 5090`, driver `32.0.16.1074`, OS `10.0.26100.0`, source `1dc0793`) emitted verdict MATCH: rows `amd-zen5-cpu`, `nvidia-blackwell-cuda`, `nvidia-blackwell-vulkan` all HOST_MATCH carrying the detected observations. Every row note requires packaged L4 checks; `supportClaimPolicy` forbids L4_PASS from host presence; `cannot claim L4 support` test PASS 2026-09-21. No host-only record claims CUDA/Vulkan inference success or complete L4. The comparison output was scratch-only, not filed as producer evidence. **Trace:** [Audit GH-10](docs/history/localmotive-comprehensive-audit.md#gh-10).
  **Current status: CLOSED 2026-09-21.** Historical generator tests stay accepted. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1941).

---

- [ ] **V06-S-25.I3 (split 2026-09-20)** — Three stapled asks, tracked separately. The search-first tuner history table is in-sample evidence only and closes neither science third. (1) Independent benchmark distributions / baseline drift — **BLOCKED-INPUT.** No external dataset, no multi-day drift program. A single tune session must not be synthesized into a distribution. Reopens only if the owner funds a measurement protocol. (2) Held-out calibration error / interval coverage — **BLOCKED-INPUT.** The on-screen calibration interval stays modest wording (Estimated interval). The tuner table is not a held-out study; no coverage percentages are reported from in-sample trials. (3) Release queue / failure / evidence-retention metrics — **DONE 2026-09-20** from existing GitHub data (window 2026-09-01..2026-09-20, no new workflows). Run counts by workflow and conclusion: CI 253 (push success 71, failure 37, cancelled 45; pull_request success 42, failure 52, cancelled 6 — the pull_request leg is the required `pr-check` context on `windows-latest`); Release verify 41 (push failure 21, success 12; workflow_dispatch failure 7; push cancelled 1); Release promote 0 runs in the window. Failure categories named from failed job/step names without guessing: CI — immutable workflow-action pin verification 30, deterministic release gates 18, frontend checks 18, Rust tests 13, dependency/security audit 11, manifest versions 2, Rust linting 2, toolchain/npm setup 2; 49 cancelled CI runs had no failed job. Release verify — packaged-executable verification 15 (plus 6 missing-evidence follow-on reports), tag-manifest match 3, Rust tests 2, frontend checks 2, prerelease checksum/inventory 1; 1 cancelled with no failed job. Durations (updated − started): CI median 383 s, p90 665 s; Release verify median 902 s, p90 1687 s. Queue delay (run_started_at − created_at) reads 0 s median/p90 on every run in the window; the API exposes no finer queue signal. Retention as configured today: verify qualified bundle 90 days and verify diagnostics 90 days on failure/cancel (`release.yml`); promote decision 90 days, public readback 90 days, publish diagnostics 30 days on failure (`release-promote.yml`). Long-term store is `release-evidence/` in git (committed); Actions artifacts expire. Sample runs: CI success `35505474440`/`35505071903`, CI failure `34912755547`/`34909564135`/`34905149184`; verify success `34514283693`/`34430058475`, verify failure `34819219725`/`34815989537`/`34813162097`. Avoid introducing virtualization or incompatible dependency unification purely from file size/counts. **Trace:** [Measurements after correctness](docs/history/localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored); [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations); [Rust dependency inventory](docs/history/localmotive-comprehensive-audit.md#rust).
  **Current status: PARTIAL.** (1)(2) blocked-input named above; (3) collected with the note above. Hardware-only coverage (D06-05) is CLOSED-DEFERRED-FAR-FUTURE 2026-09-21 (owner scope): not executed, not PASS, out of 0.6 scope. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2636).

---

- [x] **V06-G-05.I3** — CLOSED-DEFERRED-USER-REPORTS 2026-09-21 (owner scope). Manual Narrator/NVDA listening and authorized live cloud/HF credential scenarios are out of current 0.6 agent/owner campaign. Not executed. Not PASS. Automated a11y evidence stands as-is. Further accessibility and live-account defects will be handled when real users open issues; that is the accepted feedback path for this slice. Reopen only if the owner schedules a dedicated listening/live-account pass. **Covered in the packaged probe (unchanged PASS):** keyboard traversal with visible focus, reduced motion, high-DPI/zoom, forced-colors high contrast (`verify_a11y.mjs` A11Y_PASS; evidence `release-evidence/0.6.0/attestations/g05-a11y-packaged-verification.log`). Not upgraded to screen-reader verified or live HF verified. **Trace:** [Remaining target verification](docs/history/localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).
  **Current status: CLOSED 2026-09-21 (leftovers only).** U06-07 does not wait on manual listening or live-account scenarios. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2854).

---

- [ ] **V06-G-06.I2** — Test clean install/start and strict uninstall assertions for each supported installer. Test v0.4.1-to-v0.6 profile preservation separately from v0.5.0-to-v0.6 SQLite/user-override preservation. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-04](docs/history/localmotive-comprehensive-audit.md#gh-04); [GH-05](docs/history/localmotive-comprehensive-audit.md#gh-05); [GH-06](docs/history/localmotive-comprehensive-audit.md#gh-06).
  **Current status: OPEN.** R06-01 / U06-04: same native campaign; historical checkbox is superseded. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3149).

---

- [ ] **V06-G-08.V1** — Reconcile task status, required checks, lifecycle results, immutable source/digests and public claims; a green aggregate summary must not hide an unresolved High finding or missing required evidence. **Kept open (2026-09-12, owner directive):** the reconciliation is maintained continuously in the third-pass record, but it closes only with the release decision - the unresolved High rows (GH-01, GH-02, GH-03, GH-06.V3) and the G-09/G-10 gates remain, and a written deferral is not treated as passing evidence. **Reconciled (2026-09-12 third pass):** the open boxes are individually categorised (owner-gated / environment-blocked) with completion criteria and unblock actions; unresolved High rows (GH-01, GH-02, GH-03, GH-06.V3) carry their criteria and are not marked verified; a written deferral is not treated as passing evidence anywhere in this record. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Audit priorities](docs/history/localmotive-comprehensive-audit.md#priority-scale).
  **Current status: OPEN.** U06-07: current evidence-based release acceptance, under standing authorization and explicit hardware deferrals. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3198).

---

- [ ] **V06-G-09.I1** — After the recorded ship decision, create/protect the v0.6.0 tag at the verified immutable SHA and promote the exact tested candidate assets; do not rebuild or retarget a used tag. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02); [GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: OPEN.** U06-06/U06-08: final source/tag disposition and exact-byte promotion; old tag is not the repaired candidate. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3214).

---

- [ ] **V06-G-09.I2** — Bind release inventory, checksums, packaged/lifecycle records and provenance to that SHA and each artifact digest. Refuse publication when source, version, candidate or required evidence differs. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02); [GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: OPEN.** U06-02/U06-06/U06-08: actual candidate inventory/records and promoted bytes agree. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3215).

---

- [ ] **V06-G-09.I3** — Read back the published tag/release metadata and complete asset set. Verify downloaded/public byte identity against the candidate where the release gate promises it, and verify Latest/prerelease/unsigned wording matches the intended release policy. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02); [GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: OPEN.** U06-08: actual public metadata and downloaded asset identity readback. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3216).

---

- [ ] **V06-G-09.V1** — Exercise wrong SHA, moved tag, modified bytes, missing assets and absent lifecycle evidence as negative publication controls before using the real release path. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02); [GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: OPEN.** U06-08: nonpublishing negative controls on the actual promotion contract. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3220).

---

- [ ] **V06-G-10.I1** — Record final source/release IDs, exact asset digests, successful and skipped verification, closure evidence for completed findings and the remaining risk register. **Trace:** [Remediation and release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [QD-04](docs/history/localmotive-comprehensive-audit.md#qd-04); [GH-07](docs/history/localmotive-comprehensive-audit.md#gh-07); [What to measure next](docs/history/localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored).
  **Current status: OPEN.** U06-09: actual final release/evidence/deferral closeout. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3236).
## Closed and not-applicable register for this merge

- All 650 closed boxes from the working copy carry forward as closed with
  their recorded evidence. No closed state changed in this merge.
- Signing-deferred wording is superseded by the standing unsigned policy.
  Closed boxes that mention the old deferral stay closed; the policy, not a
  future signing step, is now the terminal state.
- History-only duplicates (per-pass checkpoints, frozen ledgers, repeated
  evidence rows) do not carry forward. The frozen original stays readable at
  `docs/history/TODO-0.6.md`.
- Owner hardware deferrals D06-01..D06-05 are CLOSED-DEFERRED-FAR-FUTURE
  2026-09-21 (owner scope, not executed, not PASS): seven-vendor GPU host,
  storage-class and OS-crash facility, unprovided physical configurations,
  same-model multi-GPU rig, and hardware-dependent metrics. They are out of
  0.6 scope, not open gates.
- V06-MT-07.V2 live portability matrix is CLOSED-DEFERRED-FAR-FUTURE
  2026-09-21 (owner scope): not executed, not PASS; 0.6 will not obtain a
  CPU-only (or other extra) machine to prove calibration identity on that
  class. Unit-level snapshot identity stays accepted; no claim that
  calibration works on CPU-only hosts.
- Search-first tuner CLOSED 2026-09-20 by owner acceptance: advisor-off
  default session (local grid + one-axis nudge, persisted trial table with
  outcome/choice rows, confirm bar, one optional advisor try after the
  table). Evidence: 35 tune Rust tests (6 new, mutation-proved), 282 Vitest,
  `npm run tauri build` exit 0 with exe + MSI + NSIS. Held-out/drift
  science items are not closed by this work.
