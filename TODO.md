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
|

---

- [ ] **U06-04 — OPEN; BLOCKED-ENVIRONMENT only if no suitable isolated Windows facility can actually run it.** Execute the four required current-candidate legs.

**Trace:** R06-01; V06-GH-04.V3, V06-G-06.I2, V06-GH-03.V1/V3, V06-GH-06.V3; audit [GH-04](docs/history/localmotive-comprehensive-audit.md#gh-04), [release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release).

| Leg | Baseline | Persistence |
|

---

- [ ] **U06-05 — OPEN.** Preserve verified configuration and hosted execution; finish only the missing controlled-failure and contributor-enforcement proof.

**Trace:** V06-GH-01.I3/V1/V2/V3, V06-GH-02.I3; audit [GH-01](docs/history/localmotive-comprehensive-audit.md#gh-01), [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02). **Evidence:** E03/E04/E05.

Solo-maintainer policy: the owner may author and merge a PR without a second reviewer, but strict required checks still apply; normal and emergency fixes use the protected PR path. No standing bypass actor exists. Any exceptional policy change needs a specifically recorded disposition and effective-settings readback.

GH-01.I3 and V1 are checked in the criterion ledger now. Find an existing deliberate green/red/green campaign with actual merge-blocking evidence; if absent, execute the already authorized controlled campaign without merging the failing revision. Record effective rules, bypass actors and ordinary-contributor direct-push refusal through an available appropriately scoped identity. Readback alone does not fabricate an attempted contributor refusal; if that identity is unavailable, name that specific access prerequisite. Do not use a generic historical “needs approval” stop. No settings rewrite is needed where existing effective settings already satisfy the criterion.

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

Hardware D06-01..05 is deferred now. For S-25, collect available queue/failure/evidence-retention metrics and separately identify missing held-out data/program inputs. For G-05, retain the automated a11y PASS scope; obtain a human listening pass and use only actually authorized test accounts when available. If a non-hardware residual is eligible for deferral under the existing Medium/Low/supplemental policy, make a separately reasoned delegated decision with owner, risk, workaround, supported scope, milestone and evidence gap; do not label the current hardware instruction as that decision. Mandatory High/native gates stay required.

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

- [ ] **V06-RT-06.V3 (seven-installed-backend acceptance portion) — environment-blocked:** the acceptance wording asks for all seven backends installed on a representative host. This host runs one GPU vendor (NVIDIA, adapter `luid:0000000000014f4f`), so the maximum eligible set is four installed/selected/launched backends plus the three vendor refusals; a seven-simultaneous-backend host needs three GPU vendors. Unblock action: run `scripts/g05_rt06_all_backends.mjs` (candidate bound from the committed inventory) on a three-GPU-vendor Windows host. This box must not be closed from fixture evidence or relabelled from the historical 12/12. **Trace:** [Audit RT-06](docs/history/localmotive-comprehensive-audit.md#rt-06).
  **Current status: DEFERRED-OWNER.** D06-01: owner-deferred seven-installed hardware coverage; available-host record is still required by U06-02. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L351).

---

- [ ] **V06-DC-12.V3** — Distinguish ordinary process-crash testing from target-environment OS-crash/power-loss validation; record throughput for 1/4/8 connections on available HDD, SATA SSD, and NVMe targets before optimizing. **Trace:** [Audit DC-12](docs/history/localmotive-comprehensive-audit.md#dc-12).
  **Current status: DEFERRED-OWNER.** D06-02: owner-deferred storage-class and OS-crash/power-loss facility. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L743).

---

- [ ] **V06-GH-01.V2** — Introduce a controlled failing check in the test PR; verify merge is blocked until corrected, then confirm the corrected commit obtains the required successful statuses. **Trace:** [Audit GH-01](docs/history/localmotive-comprehensive-audit.md#gh-01).
  **Current status: OPEN.** U06-05: controlled failing PR, observed merge block and green recovery evidence still required. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1704).

---

- [ ] **V06-GH-01.V3** — Read back effective branch/ruleset settings with appropriate access, record bypass actors and restrictions, and verify ordinary contributors cannot circumvent required checks through direct pushes. **Trace:** [Audit GH-01](docs/history/localmotive-comprehensive-audit.md#gh-01).
  **Current status: OPEN.** U06-05: effective rules readback exists; ordinary-contributor noncircumvention proof remains unconfirmed. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1705).

---

- [ ] **V06-GH-02.I3** — Protect released version tags against updates and deletion, document one-time tag creation, and require a new prerelease or patch version when source changes after an earlier candidate. **Trace:** [Audit GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02).
  **Current status: OPEN.** PARTIAL / U06-05 and U06-06: active immutable-tag settings accepted; tag-history/final-identity policy disposition remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1724).

---

- [ ] **V06-GH-02.V3** — Run a complete candidate flow and compare checkout revisions, inventory, packaged evidence and publication metadata; read back the effective tag-update/deletion protection. **Trace:** [Audit GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02).
  **Current status: OPEN.** U06-02/U06-06/U06-08: full candidate identities and publication metadata must agree. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1731).

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

- [ ] **V06-GH-10.V3** — Run on the intended Windows host and compare emitted identity fields to directly observed hardware/driver/source values; verify no host-only record claims successful CUDA/Vulkan inference or complete L4 support. **Trace:** [Audit GH-10](docs/history/localmotive-comprehensive-audit.md#gh-10).
  **Current status: OPEN.** E07 / U06-03: current host-attestation workflow fails to parse; historical generator tests stay accepted. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1941).

---

- [ ] **V06-S-25.I3** — Collect independent benchmark distributions/baseline drift, held-out calibration error and interval coverage, and release queue/failure/evidence-retention metrics. Avoid introducing virtualization or incompatible dependency unification purely from file size/counts. **Trace:** [Measurements after correctness](docs/history/localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored); [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations); [Rust dependency inventory](docs/history/localmotive-comprehensive-audit.md#rust).
  **Current status: OPEN.** PARTIAL / U06-07: hardware-only coverage deferred by D06-05; available release metrics remain executable and independent datasets are a separate input. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2636).

---

- [ ] **V06-G-05.I3** — Perform keyboard/Narrator or NVDA, high-DPI/zoom/high-contrast and reduced-motion checks. Perform live cloud/HF credential scenarios only where an explicitly authorized test account and suitable environment are available. **Covered in the packaged probe:** keyboard traversal with visible focus, reduced motion, high-DPI/zoom, forced-colors high contrast (`verify_a11y.mjs` A11Y_PASS; evidence `release-evidence/0.6.0/attestations/g05-a11y-packaged-verification.log`). **Remaining (blocked):** a manual Narrator or NVDA pass on the packaged app, and live cloud/HF credential scenarios, which require an explicitly authorized test account and suitable environment. **Trace:** [Remaining target verification](docs/history/localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).
  **Current status: OPEN.** PARTIAL / U06-07: automated a11y accepted; manual listening and authorized live-account scenarios remain distinct from hardware deferral. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2854).

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
- Owner hardware deferrals D06-01..D06-05 stand: seven-vendor GPU host,
  storage-class and OS-crash facility, unprovided physical configurations,
  same-model multi-GPU rig, and hardware-dependent metrics. They gate only
  the criteria named above.
