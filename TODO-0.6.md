# Localmotive 0.6 — universal TODO

**Status: IN PROGRESS — release integration and qualification remain. Unprovided hardware coverage is DEFERRED-OWNER.**

**Target:** published `v0.6.0`, with exact candidate identity, required evidence, asset readback, and explicit deferrals.

**Required outcomes:** existing supported workflows work reliably; user data survives upgrades; security, download integrity and process ownership remain enforced; one identified candidate passes the existing qualification and promotion path; public assets match the qualified bytes and disclose limitations. Features are frozen. The audit remains the traced backlog, not an expanding release boundary.

**Deferred scope:** unavailable hardware remains D06-01..05 under the recorded owner decision; optional signing retains its unsigned-release disclosure. Human listening, account-dependent scenarios, independent datasets and promised compatibility retain their actual statuses below. Optional improvements do not enter 0.6 without a demonstrated defect or required-release-path failure.

**Current blocker / next action:** cancellation/resume remains accepted at `4ff042e` with trusted-main CI `34922732310`. PR 47's pushed head `725062b` passed CI `34930008573`. The newer local repair `7887e5d` passed a complete native 0.4.0-to-0.6.0 installer/preservation rehearsal, but its second default-parallel Rust run failed in `dc02_an_interrupted_no_range_transfer_retries_from_zero`. Diagnose that distinct failure before pushing or merging the new repair. The qualification worktree now wires transferred records and retained settings into manifest assembly and promotion; remaining producer execution and final-source qualification stay open. Intensity: ultra for data preservation and release integrity. No tag changes during debugging.

**Consolidated:** 2026-09-14. **Reviewed source:** `1881db93c54f1c4a181ed5070d7c1449c50a7d74`.

**Engineer handoff:** [AGENT-ENGINEER-0.6.md](AGENT-ENGINEER-0.6.md).

This is the single operative 0.6 tracker. It consolidates the original audit TODO and the post-076a3ee residual tracker. Update this file as evidence changes; do not create another residual TODO or a new goal hierarchy. The original audit, original TODO, and prior release evidence remain historical sources. Their old approval wording and aggregate completion claims do not override the current decisions below.

## Ideas audit checkpoint — 2026-09-20

Work resumed at the owner's request. All three legacy ownership findings are corrected locally: a final cancellation gate discards late-cancelled successes, the benchmark slot publishes before client construction, and server/tuning start refuse while an abandoned benchmark slot is held. Verification: cargo fmt clean, warning-free Clippy, 622 Rust tests with 7 ignored, tsc clean, diff-check clean. Each new regression proved RED before GREEN with mutation proof. The working tree stays uncommitted. The closing handoff is `artifacts/stop-ownership-audit-20260920-0624/HANDOFF.md`.

The owner requested research, paper-cut fixes and simplification without increasing publishing requirements. This pass leaves AGENTS.md, `.github/`, permissions, tags and publication unchanged from the starting dirty tree at `bdb3c14c9170f0342ba0384738e2ffd23062970d`.

Implemented: shared bounds for 41 numeric profile fields; pre-save/preview/start/tune validation; blank-edit preservation; debounced, immediately invalidated command previews; inspected-only speculative choices with preserved unverified selections; manual release navigation; corrected release-file and credit-link documentation. Review follow-up adds Rust workload bounds, spec-value validation against inspected help, pre-read GGUF/artifact reparse checks, safe malformed-method rendering and shared evidence types. RED-before-GREEN logs and the failing 65536-port mutation are retained under `artifacts/ideas-audit-20260919-2202/`. Paper cuts are append-only in `agents_feedback.md`; research and complete review dispositions are in ignored `research/ideas-audit-20260919/`.

Current observed checks: the queued Stop correction passed `npm run check` (274 script tests, 278 Vitest tests, TypeScript and production frontend build), fmt, Clippy and 619 default-parallel Rust tests. The same seven Rust entries remain ignored; the legacy child-server fixture still executes through its parent tests. Nine Stop cases passed in release mode from the repository-required `src-tauri` directory. Eight default-parallel Stop repetitions passed. Earlier statistics and LoRA corrections retain their own release-mode records. npm audit, version/workflow checks and the cleanup matrix passed during the initial audit phase.

The rebuilt version 0.6.0 executable, MSI and NSIS remain unsigned. Executable SHA-256: `79702eb23b1a04de0d223b2ca00dc98e9b1ecf91b6696b63d01b760446b80e49`. Its isolated native diagnostic passed 26 checks plus cleanup; packaged accessibility returned `A11Y_PASS`. Current records are under `artifacts/stop-ownership-audit-20260920-0624/`. Native checks cover a UI-only cancellation handle and real idle Stop IPC. The process-replacement regressions use contained stand-ins, not models. Earlier records retain their original source and artifact identities.

All source-review messages are reconciled against their recorded candidates. Legacy ownership review deleg_16c0e72a reported FAIL with three logic defects; all three are now corrected locally with RED-before-GREEN regressions and mutation proof (see research/ideas-audit-20260919/legacy-ownership.md). An independent re-review and a packaged-matrix run remain pending. The correction touches only the five reviewed Rust files plus two regression tests. Queued Stop review deleg_060e7c42 reports PASS for the unchanged server_service.rs correction, with no material security or logic findings. Its additional wraparound test is optional follow-up. The startup-state cleanup concern remains unconfirmed; ContainedProcess::drop already attempts process termination and reap. No implementation change follows while work is paused. No clean-checkout release matrix, native upgrade campaign, public release, or new inference benchmark is claimed. No release checkbox changes follow from these local results.

LoRA follow-up: snapshots now parse scaled paths through the shared validator and fingerprint the ordered adapter contents plus f32 scales. No-adapter and single ordinary-adapter keys matched before/after the correction. A scale-removal mutation failed the regression. Optional file flags now remain reserved when their profile fields are empty; Rust and native IPC reproduced the raw-argument bypass before the correction. Existing records remain untouched; older multiple/scaled keys can require fresh measurement. The pinned upstream scaled parser has a separate apparent Windows drive-colon limitation, which these snapshot tests do not resolve. Companion-list/preflight bounds and other scoped findings remain in `agents_feedback.md` and the research dispositions.

Manifest follow-up: new command records fingerprint managed draft/LoRA paths and known aliases. Replay compares the current and prior encodings against raw current arguments without rewriting records, while retaining model and exact compatibility-key checks. Privacy and persistence regressions passed; bypassing key equality failed the strengthened test. Other local-record fields remain outside full anonymization.

Tune input follow-up: all three editable workload fields keep blank edits visible. One pure check now controls readiness, request refusal and the field status. Native constraints match the existing Rust integer domains, including valid 257-token workloads. RED cases reproduced invalid dispatch and misleading readiness text. A 13-trial mutation failed before restoration. The packaged checks use isolated readiness metadata and never dispatch a valid tuning request; real-App integration tests verify refusal and exact corrected payloads with IPC mocked. The Tune checkpoint recorded 433 unchanged tracked files outside that correction.

Benchmark v2 input follow-up: four controls now retain blank edits instead of substituting zero. A RED request proved that clearing Warmups previously dispatched warmups:0. Explicit zero stays valid. The existing validator rejects incomplete input, and its cancellation ownership tests still pass. Reintroducing the zero fallback failed the action-level regression. Native checks cover editing, visible errors and recovery with an idle server; real-panel tests cover dispatch with IPC mocked. The v2 checkpoint recorded AGENTS.md and 437 tracked files outside that correction unchanged.

Legacy benchmark input follow-up: both controls retain blank edits and share a pure bounds check across readiness, dispatch and visible status. Ten invalid-edit cases reproduced an IPC request before the correction. Valid endpoints and 65 tokens pass unchanged; invalid edits preserve prior results. A deliberate 4097-token ceiling failed the fixed action test before restoration. Native editing/error checks pass with an idle server. That checkpoint recorded AGENTS.md and 434 tracked files outside its scope unchanged. The later ownership and Stop corrections below supersede its open-concurrency note.

Benchmark statistics follow-up: RED tests observed null JSON summary fields, infinite tuning deviation and a zero median for positive subnormal samples. Maximum-value normalization and `f64::midpoint` correct those cases without a new dependency or IPC change. Restoring the unscaled mean and substituting the population denominator each failed the fixed regressions. Both mutations were restored before full verification. These are synthetic numeric robustness checks, not performance measurements.

Legacy ownership follow-up: an owned synthetic HTTP child reproduced concurrent legacy requests and the missing cancellation slot. The command now reserves the shared operation owner, uses the cancellable sampler and retains ownership through request drain. The UI keeps an independent cancellation handle across navigation and refuses a second dispatch before render. Removing drain and mixing the UI owner slots each failed the regressions. Client-construction failure and cancellation both allow a later valid run. The unused non-cancellable sampler was removed. A failed Tauri mock-runtime setup was reverted rather than changing production manifests; the state-taking tests need no new dependency. Research, limitations and timestamped paper cuts remain in the existing audit records.

Queued Stop follow-up: RED tests reproduced a later benchmark-owner bypass and termination of a replacement process after its startup reservation ended. Stop now captures the generation, signals only that startup, waits for its ownership to end and acquires the existing reservation before accessing the server slot. The post-acquisition generation check rejects intervening completed work. Nine regressions cover replacement preservation, original-start cancellation, reservation lifetime, normal Stop and wraparound. Three controlled mutations failed before exact restoration. A separate post-spawn client-construction cleanup path remains a source watchpoint and needs reproduction; this correction does not claim it fixed.

## Owner decisions and completion meaning

The owner has authorized continued release work, including necessary commits, protected PRs, approved hosted checks, repository settings, verification, promotion and publication. Do not ask again for these same general permissions. This document records the latest instruction: **“hardware we dont provide yet has to be deferred.”** Its exact scope is in [hardware deferrals](#hardware-deferrals).

Deferral means excluded from the present release's required hardware coverage, visibly untested, and retained for follow-up. It never means PASS. Do not acquire missing hardware, invent measurements, or wait indefinitely for an unprovided GPU/storage configuration. Existing supported-host records must still be generated honestly. Native installer/preservation verification on a suitable isolated Windows environment remains a release gate; it is not a request to obtain three GPU vendors or a power-loss laboratory.

Human screen-reader work, live accounts and independent datasets are different prerequisites from unavailable hardware. Identify their actual remaining scenarios and eligible dispositions under U06-07; this hardware instruction does not silently waive them or authorize spending, obtaining new accounts, or use of somebody else's credentials.

The standing authorization does not silently change the immutable-tag contract. Prepare and verify the actual final candidate before resolving the concrete conflict between the existing `v0.6.0` tag and a newer required source. Do not use tag deletion/recreation or bypass actors as the debugging loop. U06-06 records the single final identity decision if it is necessary.

**Release completion:** U06-01 through U06-09 have their acceptance evidence, no mandatory gate remains unresolved, authorized publication/readback/closeout have occurred, and every eligible deferral has an honest scope and follow-up. The truthful result is **“0.6 released; current release scope complete with named hardware deferrals”** when that is what happened. **Every original criterion verified** is a stronger claim and remains false while any original criterion is deferred or otherwise unexecuted.

## Reading order and status rules

1. Read this decision section and the current checkpoint.
2. Execute the finite packages U06-01 through U06-09. Their order and dependencies are explicit; one campaign may satisfy several source criteria.
3. Consult the complete original-criterion ledger only for the affected IDs and their audit/evidence links.
4. Append a concise evidence record and update the corresponding package and source statuses after an actual result.

| State | Checkbox | Meaning |
|---|---|---|
| VERIFIED | checked | Observed completion at the stated source/run or accepted historical evidence within its stated scope |
| OPEN / IN-PROGRESS | unchecked | Required implementation, execution, reconciliation or publication is unfinished |
| BLOCKED-ENVIRONMENT / BLOCKED-INPUT | unchecked | A specific execution facility, person, account or dataset is missing; identify it |
| DEFERRED-OWNER | unchecked | Owner-authorized exclusion from current release scope; not a passing test |

Imported checked criteria retain accepted implementation and historical verification. They do not assign old artifact hashes to a new build or claim a fresh candidate campaign. The original wording is preserved for traceability; the adjacent **Current status** annotation governs any historical “closed,” “blocked,” candidate SHA or approval statement in that wording. Broad original assertions that all findings were verified are not imported as a current completion claim.

U06 labels are execution packages mapped to existing criteria and newly observed release-path regressions, not additional audit findings. Package boxes and source-criterion boxes are two views of overlapping work; do not add them as an effort estimate.

## Current checkpoint and evidence index

| Evidence | Observed result and precise limit |
|---|---|
| E01 — [main CI 34828965478](https://github.com/sato942/localmotive/actions/runs/34828965478) | SUCCESS for `1881db9`: check, rust-audit, Security audit, package-smoke; pr-check skipped on push. This is not Release verify qualification. |
| E02 — [PR 26](https://github.com/sato942/localmotive/pull/26), [repair CI 34826767627](https://github.com/sato942/localmotive/actions/runs/34826767627), [bounded repair ledger](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/TODO-0.6-post-076a3eeecbdf.md#L454) | Owned-process orchestration source landed at `353491e`; ledger reports 8 helper scenarios under Windows PowerShell 5.1 and a fixture init/prep rehearsal. The new matrix is not wired into CI and is not complete execution of the main orchestration under workflow pwsh. U06-01 remains open. |
| E03 — [PR 18 hosted check](https://github.com/sato942/localmotive/actions/runs/34801833268/job/103845880922), [PR 18](https://github.com/sato942/localmotive/pull/18) | `pr-check` SUCCESS for `d4a2e4ac961776f20d80edc8afe9640fb72cf623` on `windows-latest`; self-hosted jobs skipped. Check completed before merge `938b995`. Closes GH-01.V1; not deliberate-red or contributor-bypass proof. |
| E04 — [main ruleset](https://github.com/sato942/localmotive/rules/23218749) | Read back 2026-09-14: active main-only pull-request rule, strict required `pr-check`, non-fast-forward and deletion protection, zero required reviewers for solo-maintainer operation, no bypass actors. Closes configuration GH-01.I3 together with the existing solo-maintainer policy. |
| E05 — [tag ruleset](https://github.com/sato942/localmotive/rules/23218751) | Read back 2026-09-14: active `refs/tags/v*`, update/deletion rules, no bypass actors. Current protection is verified; earlier tag recreation means full GH-02.I3 policy/identity disposition is still open. |
| E06 — [Release verify 34819219725](https://github.com/sato942/localmotive/actions/runs/34819219725) | FAILED at tagged source `27349f93d7d17eac9b15a6895cd94e850a122a8b`. Final catalog PASS was followed about 282 ms later by exit 1. That old taskkill/LASTEXITCODE mechanism is removed in new source; current complete release execution is still unproven. |
| E07 — [hardware run 34819219650](https://github.com/sato942/localmotive/actions/runs/34819219650) | FAILED: PowerShell parser error caused by the trailing quote in hardware-qualify.yml. Existing host is present; this is a software defect, not deferred hardware coverage. |
| E08 — [required manifest records](scripts/build_qualification_manifest.mjs), [manifest verifier](scripts/verify_qualification_manifest.mjs) | Eight required source-bound records remain historical and are not regenerated by release.yml. An isolated contract probe preserving historical record bytes reproduced source mismatch refusals for a new candidate. This was a controlled validator probe, not a candidate execution. U06-02 must connect producers and transfers. |
| E09 — [published releases](https://github.com/sato942/localmotive/releases) | At checkpoint, latest public release is v0.5.0. Tag v0.6.0 is object `92385b2da503d761fa2ba1eaae62911f2c0e98d2`, peeled to `27349f9`; it excludes the current repair. No 0.6 publication claimed. |
| E10 — [PR 34](https://github.com/sato942/localmotive/pull/34), [pr-check 34867277657](https://github.com/sato942/localmotive/actions/runs/34867277657) | Cancellation-test race fixed and merged at `8d80896`: observed-progress cancellation replaces the 100 ms timer; fixture bounded. `pr-check` SUCCESS in 10m48s (Rust tests incl. corrected test green). Full local Rust suite 617/617 at fix commit; [map](docs/QUALIFICATION-MAP-0.6.md) recovered from unmerged PR 33 branch `5e36537` unchanged. |

The source and artifact identities above are historical observations, not commands to tag an old revision. Re-read current state at execution time. A documentation merge after this checkpoint needs its own CI reference; it does not change the identity of existing evidence.

## Completed preparation and accepted repairs

- [x] **P06-01 — State and residual reconciliation prepared.** Historical completion preserved from the residual ledger; this universal tracker replaces its stale current-status wording. Trace: source release closeout/G-08 and the [residual preparation](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/TODO-0.6-post-076a3eeecbdf.md#L68).
- [x] **P06-02 — Release decision packet prepared.** Standing authorization is now present; preparation does not substitute for actual release execution. Trace: GH-01/GH-02/GH-03/G-08/G-09; [packet](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/TODO-0.6-post-076a3eeecbdf.md#L84).
- [x] **P06-03 — Environment and pending-scenario matrix prepared.** Missing hardware now has an explicit owner deferral below. Trace: RT-06/DC-12/S-25/G-05/GH-04; [matrix](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/TODO-0.6-post-076a3eeecbdf.md#L140).
- [x] **P06-04 — Preparation handoff completed.** This is not G-10.I1 or proof of a published release. Trace: G-10; [historical handoff](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/TODO-0.6-post-076a3eeecbdf.md#L153).

| Accepted correction | Preserved disposition |
|---|---|
| F9-01 | Classifier correction accepted at 076a3ee; record replay remains separate from historical packaged MT-06 evidence. |
| F9-02 / F9-03 / F9-06 | Closed per owner review; no new audit or repeat campaign solely to refresh prose. |
| F9-04 | Staged packaged-record integration accepted; U06-02 addresses other producers, not a reopening of the accepted stage flag. |
| F9-05 | Verifier implementation accepted; current native assertions remain required through R06-01/U06-04. |

Trace: [accepted F9 dispositions](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/TODO-0.6-post-076a3eeecbdf.md#L23), [reviewed correction CI](https://github.com/sato942/localmotive/actions/runs/34774614565).

<a id="hardware-deferrals"></a>
## Hardware deferrals — owner decision of 2026-09-14

These exclusions are effective now. They do not depend on another general approval. Responsible owner for future provision: Sato; engineer maintains the scope/evidence record. Reactivate only when the owner actually supplies the required configuration and schedules its test. Follow-up milestone: first authorized qualification campaign on that newly provided configuration, before claiming support for it. Do not invent a calendar date.

| ID | Original trace / deferred scope | State and release effect | Residual risk and honest workaround |
|---|---|---|---|
| D06-01 | V06-RT-06.V3 **seven-installed portion**; [RT-06 audit](docs/history/localmotive-comprehensive-audit.md#rt-06) | DEFERRED-OWNER; three-vendor hardware coverage is outside current release exit scope | No claim of seven simultaneously installed/exercised backends. Generate the current available-host installed/refused matrix anyway and publish that exact scope. This needs three vendor configurations, not seven GPUs. |
| D06-02 | V06-DC-12.V3; [DC-12 audit](docs/history/localmotive-comprehensive-audit.md#dc-12) | DEFERRED-OWNER; unprovided HDD/SATA SSD/NVMe targets and real OS-crash/power-loss facility do not block current release | Storage-class throughput and crash/power-loss durability remain unverified. Existing interruption tests do not prove OS-crash durability. Retain checksum/atomic-publication controls and disclose the gap. |
| D06-03 | Physical CPU-only/changed-CPU/GPU portability beyond accepted V06-MT-07.V2 identity tests; [MT-07 audit](docs/history/localmotive-comprehensive-audit.md#mt-07), V06-S-25.I3 | DEFERRED-OWNER for unprovided physical configurations | Five accepted canonical identity scenarios remain checked. Do not carry throughput/support claims to unmeasured hardware; remeasure on the destination configuration before performance claims. This is a scoped limitation, not a new product defect. |
| D06-04 | Physical same-model multi-GPU coverage in V06-RT-09.V3 and V06-G-05.I2 when hardware is available; [RT-09 audit](docs/history/localmotive-comprehensive-audit.md#rt-09) | DEFERRED-OWNER for unprovided multi-GPU rigs | Existing deterministic identity tests and supported-host evidence remain accepted. No claim that physical identical-card mapping was exercised; use stable adapter identity and display unmeasured scope. |
| D06-05 | Hardware-dependent independent distributions within V06-S-25.I3 | DEFERRED-OWNER only for measurements requiring unprovided hosts | Do not invent population accuracy or portability. The independent dataset/program itself remains BLOCKED-INPUT if missing; available release queue/failure/retention metrics are executable under U06-07. |

Human Narrator/NVDA listening and authorized cloud/HF account scenarios in G-05.I3 are **not** reclassified as hardware coverage. Their automated subset is accepted; their remaining scope must be executed or receive a policy-eligible, separately reasoned disposition. Optional SignPath/Authenticode remains the existing deferred signing decision, with unsigned/SmartScreen/checksum disclosure. No new signing prerequisite is introduced.

## Finite execution packages

Every package below maps to original criteria; close it only on its stated evidence. U06-01/02/03 are one release-integration correction pass. Use the existing scripts, complete producer-to-consumer wiring, and actual Windows execution before another final tag run. Avoid a sequence of cleanup-only patches that each discover the next missing gate.

<a id="u06-01"></a>
### U06-01 — Complete packaged orchestration and executable coverage

- [x] **U06-01 — DONE 2026-09-14 at `ae448a7` (PR 29).** Corrected and proved the full verification/cleanup result, under the workflow's actual `pwsh` mode. Evidence: ledger rows above; matrix 12/12; PR 29 pr-check pass; main CI 34845858794 SUCCESS.

**Trace:** V06-GH-05.I4/V1/V2/V3, V06-GH-06.I1/V1/V2, V06-G-05.V1; audit [GH-05](docs/history/localmotive-comprehensive-audit.md#gh-05) and [GH-06](docs/history/localmotive-comprehensive-audit.md#gh-06). **Evidence:** E02/E06; [orchestrator](scripts/verify_packaged_matrix.ps1), [current matrix](scripts/tests/verify_cleanup_matrix.ps1).

Required correction: `Stop-Candidate` must propagate a still-live owned candidate into a failing final result while preserving an earlier verifier error. Validate this attempt's fixture identity; do not accept any `node` process on CIM failure or generic `__serve` alone. Bound lookup/termination and keep process ownership through cleanup. Exceptions inspecting a process are not proof it exited. Cleanup after partial initialization must run. No unconditional success reset, ignored mandatory cleanup, or broad process/port kill.

Acceptance: executable tests invoke actual adoption/orchestration/final-outcome behavior, covering successful phases with already-stopped/live fixture, repeated cleanup, verifier failure, early failure, candidate/fixture unable to stop, foreign process and unavailable/slow identity lookup. The negative controls fail the prior implementation for the expected reason. Wire these tests into required Windows CI. The current helper-only matrix, copied predicate and after-the-fact elapsed-time assertion are insufficient. Run the real two-launch packaged/catalog matrix to completion against identified artifact bytes; capture final process exit and no owned leftovers. Record producer SHA, harness SHA and artifact digests separately.

<a id="u06-02"></a>
### U06-02 — Produce every required record for the actual candidate

- [ ] **U06-02 — OPEN.** Connect existing producers and artifact transfers before qualification; validate the entire manifest against the actual candidate.

**Trace:** V06-GH-02.V3, V06-GH-03.V1/V3, V06-GH-06.V3, V06-G-04/V06-G-05/V06-G-06, V06-G-09.I2/V1; audit [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02), [GH-06](docs/history/localmotive-comprehensive-audit.md#gh-06), [release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release). **Evidence:** E08.

Build a producer/consumer map for all 18 mandatory manifest records, naming executable, prerequisite, output path, source/byte binding, artifact upload/download, validation and any explicitly supported historical scope. Confirm that the files consumed are the files produced by this run. Eight source-bound historical inputs must be regenerated/staged:

| Records | Existing producer / prerequisite | Required binding |
|---|---|---|
| `installer_payload_identity` | `scripts/verify_installer_payloads.mjs` with current artifacts/inventory and functional extraction | Current source, inventory digest, portable and extracted installer payload identities |
| `mt06_cancellation` | `scripts/g05_mt06_cycles.mjs`; existing TLS/model/server preparation and `MT06_EVIDENCE_PATH` | Current source and portable digest; real terminal outcomes and separate UI/API boundaries |
| `rt06_all_backends` | `scripts/g05_rt06_all_backends.mjs`; exact selected adapter, evidence/log paths | Current source/portable digest and actual installed/refused backend scope; D06-01 remains deferred |
| `witness_timeout`, `witness_malformed_result`, `witness_preservation_missing`, `witness_stale_lock`, `witness_live_lock` | Existing `scripts/sandbox/test-fault-evidence.ps1` with current candidate installers | This candidate's identities and expected bounded negative outcomes |

The other mandatory records are packaged verification, four native lifecycle legs, `witness_missing_assets`, RT-06 run log, a11y log, DC-04 log and RT-04 log. Audit their existing supported carry-forward rules explicitly; do not claim old logs are newly executed. The catalog matrix is additionally required by the packaged gate even though it is not one of the manifest's 18 keys.

Acceptance: one complete candidate-to-qualified-bundle rehearsal succeeds through manifest and promotion validation, with every required record available in a fresh checkout plus that run's downloaded bundle. Old-source/current-inventory, wrong bytes and missing mandatory record controls must refuse qualification. Do not satisfy this by editing historical SHAs/digests or weakening schemas. MT-06 is not a standalone command without its existing controlled TLS/runtime/model setup. Historical lifecycle carry-forward is not a waiver for the eight current-source records or U06-04 native execution.

<a id="u06-03"></a>
### U06-03 — Repair and execute the existing host-attestation workflow

- [x] **U06-03 — DONE 2026-09-14 at `9cbb9fd` (PR 30 + run 34847400771).** Removed the trailing PowerShell quote in `hardware-qualify.yml` and verified the actual block parses and runs on the already available host. Evidence: ledger rows above; attestation MATCH downloaded.

**Trace:** V06-GH-10.I2/I3/V3; audit [GH-10](docs/history/localmotive-comprehensive-audit.md#gh-10). **Evidence:** E07; [workflow](.github/workflows/hardware-qualify.yml).

Acceptance: actual emitted source/CPU/GPU/driver fields agree with observed values; mismatch and unknown controls remain honest. Host presence must not be labeled inference qualification or proof of deferred vendor coverage. This workflow is separate from the 18-record qualification manifest; do not invent an additional release dependency solely to connect it. Execute through its existing authorized entry point after the repair.

<a id="u06-04"></a>
### U06-04 — Complete mandatory native lifecycle (R06-01)

- [ ] **U06-04 — OPEN; BLOCKED-ENVIRONMENT only if no suitable isolated Windows facility can actually run it.** Execute the four required current-candidate legs.

**Trace:** R06-01; V06-GH-04.V3, V06-G-06.I2, V06-GH-03.V1/V3, V06-GH-06.V3; audit [GH-04](docs/history/localmotive-comprehensive-audit.md#gh-04), [release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release).

| Leg | Baseline | Persistence |
|---|---|---|
| Upgrade | v0.4.0 | cache |
| Upgrade | v0.5.0 | mirror |
| Preservation | v0.4.1 | cache |
| Preservation | v0.5.0 | mirror |

Use `scripts/sandbox/host-run-lifecycle.ps1 -Tag <selected-tag> -Version 0.6.0 -PreviousTag <baseline> -CandidateDir artifacts -EvidenceName <leg> -PreservationFlavor <cache-or-mirror>` and the existing controlled fixtures. Current installers come from this candidate; previous baselines come from their published assets. This same campaign closes all criteria it actually covers without repeating it per ID.

Acceptance: actual clean install/start, expected version/digest after upgrade, required profile/settings/cache or SQLite/user-override preservation, strict uninstall and retained per-leg results. Loss/corruption controls must fail. A workflow success label or historical carried-forward record cannot replace these observations. Preserve BLOCKED if the facility cannot run; diagnose a concrete boot failure rather than retrying indefinitely. Do not procure unrelated GPUs or storage for this gate. The original audit calls GH-04 High; an inconsistent historical Medium label cannot waive native acceptance.

<a id="u06-05"></a>
### U06-05 — Finish governance evidence once

- [ ] **U06-05 — OPEN.** Preserve verified configuration and hosted execution; finish only the missing controlled-failure and contributor-enforcement proof.

**Trace:** V06-GH-01.I3/V1/V2/V3, V06-GH-02.I3; audit [GH-01](docs/history/localmotive-comprehensive-audit.md#gh-01), [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02). **Evidence:** E03/E04/E05.

Solo-maintainer policy: the owner may author and merge a PR without a second reviewer, but strict required checks still apply; normal and emergency fixes use the protected PR path. No standing bypass actor exists. Any exceptional policy change needs a specifically recorded disposition and effective-settings readback.

GH-01.I3 and V1 are checked in the criterion ledger now. Find an existing deliberate green/red/green campaign with actual merge-blocking evidence; if absent, execute the already authorized controlled campaign without merging the failing revision. Record effective rules, bypass actors and ordinary-contributor direct-push refusal through an available appropriately scoped identity. Readback alone does not fabricate an attempted contributor refusal; if that identity is unavailable, name that specific access prerequisite. Do not use a generic historical “needs approval” stop. No settings rewrite is needed where existing effective settings already satisfy the criterion.

<a id="u06-06"></a>
### U06-06 — Resolve final identity and finish Release verify

- [ ] **U06-06 — OPEN.** Select a fully reviewed source containing the corrections and obtain one successful, fully qualified final producer run.

**Trace:** V06-GH-02.I3/V3, V06-GH-03.V1/V3, V06-G-09.I1/I2; audit [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02), [GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).

Finish U06-01/02/03 and their actual preflight before another final tag attempt. The existing tag remains at 27349f9 while current main contains later fixes. Present the concrete source, checks, publication state and tag/version policy disposition if a different final SHA is needed. A new SHA under an already-used immutable tag is a real conflict, not another general release permission request. No silent temporary bypass or retargeting loop.

Current workflow facts: dispatching `release.yml` from main with the old tag still checks out that tag's source; re-running an old run does not load new committed workflow code; promotion requires a successful **push-event** Release verify at the tag's unchanged peeled SHA. Do not invent an RC-tag/version convention or weaken that contract to avoid reconciliation. An off-tag rehearsal is valid diagnostic work but not a substitute final producer run.

Acceptance: applicable required CI, resolve, audit, quality, package, native lifecycle and qualification all succeed for the identified candidate; qualified bundle, inventory and every record agree. Record tag object, peeled SHA, run/attempt ID, workflow/harness identity and each artifact digest. New outputs get their own digests even when product source is unchanged.

<a id="u06-07"></a>
### U06-07 — Reconcile release acceptance and eligible residuals (R06-02)

- [ ] **U06-07 — OPEN.** Complete FE-05 evidence-based acceptance and the release decision under standing owner delegation; persist precise residual scope.

**Trace:** R06-02, V06-G-08.V1, V06-FE-05.I1-I5/V1-V3, V06-G-05.I3, V06-S-25.I3; audit [FE-05](docs/history/localmotive-comprehensive-audit.md#fe-05), [release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance).

Reuse accepted FE-05 navigation, completion visibility, two-profile saved-manifest/anchor provenance and I1-I5 regression mappings. Do not invent a personal owner review or repeat the A/B walkthrough solely to generate a signature. Record who made the delegated evidence assessment and cite the actual owner authorization.

Hardware D06-01..05 is deferred now. For S-25, collect available queue/failure/evidence-retention metrics and separately identify missing held-out data/program inputs. For G-05, retain the automated a11y PASS scope; obtain a human listening pass and use only actually authorized test accounts when available. If a non-hardware residual is eligible for deferral under the existing Medium/Low/supplemental policy, make a separately reasoned delegated decision with owner, risk, workaround, supported scope, milestone and evidence gap; do not label the current hardware instruction as that decision. Mandatory High/native gates stay required.

Acceptance: current evidence supports the stated release scope, no unresolved mandatory High/native gate is hidden, all unavailable hardware coverage is excluded honestly, every remaining non-hardware criterion has an executed result or policy-eligible explicit disposition. Final ship decision is based on prepublication evidence; publication/readback is the next action, not a circular prerequisite to that decision.

<a id="u06-08"></a>
### U06-08 — Promote, publish and read back exact bytes

- [ ] **U06-08 — OPEN.** Complete the existing promotion controls, dry run, publication and public byte readback under standing authorization.

**Trace:** V06-G-09.I1/I2/I3/V1, V06-GH-02.V3, V06-GH-06.V3; audit [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02), [GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03), [stabilization exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release).

Use `release-promote.yml` with the final tag, selected successful verify run and exact confirm phrase `PUBLISH v0.6.0` (adjust only for an explicitly selected different version). Run `dry_run=true` first; actual publication uses `dry_run=false`. Promotion consumes the qualified bundle, never a fresh rebuild. Wrong source/tag, modified bytes, missing asset or missing lifecycle evidence must refuse publication in nonpublishing controls.

Acceptance: real release exists with the complete intended asset set; download public assets and compare SHA-256 and inventory/record identities to the qualified producer. Recheck tag, version, Latest/prerelease state and unsigned/SmartScreen/checksum wording. Inspect artifact contents, not merely successful upload-step labels. Preserve diagnostic and readback records. A successful dry run is not a published release.

<a id="u06-09"></a>
### U06-09 — Close current release scope and retain deferred coverage

- [ ] **U06-09 — OPEN.** Finish one current closeout record and update this tracker from evidence.

**Trace:** V06-G-10.I1/I2/I3/V1, V06-G-08.V1; audit [release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance), [QD-04](docs/history/localmotive-comprehensive-audit.md#qd-04), [GH-07](docs/history/localmotive-comprehensive-audit.md#gh-07).

Record final source/tag/release/run identities, exact public artifact digests, commands/results, verified task IDs, named deferrals and remaining limitations. Check only completed criteria; retain DEFERRED-OWNER boxes unchecked. Preserve original audit and frozen evidence. Historical main/CI/tag observations keep their own dates and SHAs. Confirm no unreconciled owned processes remain. Update this tracker and its handoff from the same evidence; avoid a new documentation-only loop for unchanged results.

## Status changes made by this consolidation

| Original criterion | Old checkbox | Current disposition | Evidence / reason |
|---|---|---|---|
| V06-GH-01.I3 | unchecked | VERIFIED, checked | E04 effective main ruleset and existing solo-maintainer policy |
| V06-GH-01.V1 | unchecked | VERIFIED, checked | E03 real isolated hosted PR check before merge |
| V06-GH-04.V3 | checked | OPEN, unchecked | R06-01 requires still-missing corrected native observations; U06-04 |
| V06-G-06.I2 | checked | OPEN, unchecked | Same R06-01 native obligation; one campaign, not two |
| V06-GH-10.V3 | checked | OPEN, unchecked | E07 current host-proof execution fails to parse; U06-03; historical generator controls remain checked |
| V06-RT-06.V3 seven-installed portion | unchecked | DEFERRED-OWNER, unchecked | Owner hardware instruction; D06-01 |
| V06-DC-12.V3 | unchecked | DEFERRED-OWNER, unchecked | Owner hardware instruction; D06-02 |

The accepted RT-06 available-host portion uses the same original `V06-RT-06.V3` ID as the seven-installed portion. Both entries are deliberately retained with distinct registry anchors; the narrower completed scope stays checked. The physical caveats in RT-09/G-05.I2/MT-07 remain scoped as D06-03/04 without discarding completed identity tests.

## Counts and source integrity

| View | Checked | Unchecked | Scope |
|---|---:|---:|---|
| Original criteria, current dispositions | 644 | 20 | 664 original scoped lines; 2 unchecked entries explicitly hardware-deferred |
| Completed preparation | 4 | 0 | P06-01..04, inherited preparation only |
| Remaining execution packages | 0 | 9 | U06-01..09; these map to overlapping source criteria |

Original source counts were 645 checked / 19 unchecked. This consolidation checks two demonstrated governance criteria and reopens three specific execution claims; it does not force totals to a target. Counts describe checklist lines, not remaining effort.

Source imports: `TODO-0.6(2).md` is byte-identical to [the original TODO at the reviewed source](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md), SHA-256 `7fc68e59e27d32fdd447f9cdfd9402b2cf3691ef32e9a90f9b1a5ba030a25d3d`; `TODO-0.6-post-076a3eeecbdf(1).md` matches [the reviewed residual tracker](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/TODO-0.6-post-076a3eeecbdf.md), SHA-256 `52110dea71fd0c86a6961722a54a9dcf11b8c79c56fca4a580fd8bd40062dcbb`.

There are 664 original criterion lines, 663 original IDs, 72 primary findings, 29 supplemental packages and 10 release-gate groups. Every criterion appears below, once per original scoped line, with its audit link and an immutable original-line/evidence pointer. The two scoped RT-06.V3 entries account for the repeated original ID. Nothing is silently deduplicated. `scripts/check_tracker.mjs` still validates the historical source tracker only; it is not claimed to validate this new file.

## Evidence ledger for subsequent execution

2026-09-19 / owner-authorized second reduction (in progress):

- The owner authorized the protected guide edit and continued product/pipeline reduction. `AGENTS.md` now has 179 lines instead of 431. The guide retains the safety rules and documents the maintained packaged matrix. Removed the now-unreferenced `scripts/verify_024.mjs`; `removed-guide-script.zip` retains its exact working-tree bytes. The first archive check exposed CRLF/LF conversion; the corrected archive passed byte comparison. This resolves the protected-guide Deferred Setup marker below after its stated authorization trigger.
- Promotion now invokes the complete validator once per downloaded bundle. Removed separate manifest/checksum commands at those same boundaries; retained independent validation in the read-only gate and write-authorized publish job. No validator protection was removed. The new workflow regression failed before this cut and passed afterward. All 10 promotion tests passed, including real CLI controls for missing manifests, invalid checksums, and changed installer bytes. Workflow gate and syntax checks passed. Logs: `.hermes-0.6/barebone/promotion-{dedup-red,dedup-green,validator-cli}.log`.
- The workspace-consolidation regression currently fails as intended against the old six-job verification chain. Consolidation and optional feature removal remain in progress. No new full-suite, packaged, hosted-CI, or release result is claimed. No tag, signing, or publication action occurred.

2026-09-19 / owner-requested simplification, local working tree based on `bdb3c14`:

- Static review covered all 188 inventoried code/configuration files (87,557 original physical lines). Four review reports and the reconciled per-file inventory are in `.hermes-0.6/barebone/`. Static review is not exhaustive behavioral verification.
- Removed 24 unused diagnostic scripts and four unreachable Rust test helpers. Retained `verify_024.mjs` because the protected `AGENTS.md` update timed out without approval. The deleted script bytes match the recovery archive `removed-scripts.zip`. Historical records and existing untracked release evidence remain unchanged.
- Removed duplicate Node/TypeScript/doctest execution, the unused verify-workflow publication input, the unused quality tag output, and a report-only qualification step. Full Rust tests now enforce `RUSTDOCFLAGS=-D warnings`. Source, digest, preservation, signing, promotion, and post-transfer checks remain. No product feature or IPC command was removed.
- RED/GREEN: the publish-input and single-test-path assertions failed before their corrections. Script deletion exposed an `ENOENT` in the line-ending guard; the guard now excludes deleted files. Clippy exposed one fixture used only by deleted aliases; that unreachable fixture was also removed. Logs retain each initial failure. Three disposable workflow mutations then failed for the intended assertions; the unchanged control passed. The initial mutation helper changed line endings and was corrected before accepting its evidence.
- PASS: `npm run check` (269 script tests, 144 Vitest tests, strict types, repository verifiers, frontend build); workflow version/pin/gate/syntax checks; packaged cleanup matrix; `cargo fmt --check`; `cargo clippy --locked --all-targets -- -D warnings`; both full Cargo test command forms (619 passed, 0 failed, 7 ignored each); `npm audit --audit-level=moderate` (0 vulnerabilities).
- PASS: `npm run tauri build` produced the Windows executable, MSI, and NSIS installer. The isolated packaged smoke checked version, eight navigation controls, idle status/Stop, and 158 local catalog rows. The owned application exited; process readback found no probe processes. All three artifacts are unsigned. `package-files.json`, `package-metadata.json`, and `packaged-smoke.json` retain exact identities and scope.
- Limits: no commit, push, tag change, signing, or publication occurred. Native upgrade/preservation and final-source release qualification were not rerun. Existing flaky-test findings remain open. The live comparison/calibration/import/share subsystem and parallel benchmark paths need a product-scope decision before removal. This first reduction is not a barebone conversion or release approval.
- Deferred Setup: `tauridev: the protected guide update is blocked and verify_024.mjs remains, revisit when the owner approves the packaged-verification guide update.`

2026-09-15 / PR 47 remaining review findings and native CDP configuration:

- Required `pr-check` [34930008573](https://github.com/sato942/localmotive/actions/runs/34930008573) passed at `725062be79cd4dea17de570be147f40531bc1177`. The read-only review batch is reconciled. The owned-process leak was fixed in that commit; subsequent retention/preflight/configuration changes need their own exact-head checks.
- Retention RED/GREEN: `pwsh -NoProfile -File scripts/tests/verify_settings_retention.ps1` first failed because the host dropped the actual settings reads. The host now copies the actual settings JSON under lock ownership and binds its filename and SHA-256 into the lifecycle document. The real copy/digest checks and lost-lock refusal passed under PowerShell 7 and 5.1. The workflow's retained artifact pattern includes the settings file. Logs: `.hermes-0.6/pr47-retention-{red,green,ps51}.log`.
- CLI preflight RED/GREEN: the actual workflow step incorrectly passed when the Windows Sandbox executable existed but `wsb.exe` was unavailable. The step now requires the CLI and a successful nonempty `--version` result. Executed controls cover missing CLI, failing CLI and working CLI. Logs: `.hermes-0.6/pr47-cli-preflight-red.log`, `.hermes-0.6/pr47-review-final-green.log`.
- Native controlled probe: the guest shell and installed v0.4.0 application both reported High integrity (`S-1-16-12288`). With environment arguments, the app exposed zero CDP pages. With the documented application-scoped HKLM `AdditionalBrowserArguments` override, the same executable and storage folder exposed one page and the browser command line included `--remote-debugging-port=10093`. Baseline payload SHA-256 remained `af9f01ac31730798b584efd1aee8f8854e97cc894573106c6935acb0461ebb93`; guest WebView2 was `153.0.4234.32`. The probe stopped its owned Sandbox and readback showed no remaining environments. Logs: `.hermes-0.6/integrity-probe.{json,log}`, `.hermes-0.6/u06-integrity-host.log`; scripts: `.hermes-0.6/u06-{settings-integrity-probe,run-integrity-probe}.ps1`. Primary explanation: [Microsoft WebView2 security guidance at `4964cdf`](https://github.com/MicrosoftDocs/edge-developer/blob/4964cdf121506b05015539b0cb87404f3a45edd3/microsoft-edge/webview2/concepts/security.md).
- Guest configuration regression: the elevated path now creates only the Localmotive override inside the disposable guest. The non-elevated path changes no machine configuration. Existing application overrides are refused; existing keys with other application values are not recreated. RED cases recorded the absent configuration and unsafe key recreation; focused settings/configuration controls passed. Logs: `.hermes-0.6/pr47-{elevated-red,registry-retention-red,native-config-green}.log`. Host policy and security enforcement are unchanged.
- Native rehearsal PASS at harness `7887e5de0ac1a1ccaa9ad3a289a8e305e555cbe3`: `LOCALMOTIVE_SOURCE_REVISION=eb9ce5828aeb4b2b284860a45371e8de35ebf353 pwsh -NoProfile -File scripts/sandbox/host-run-lifecycle.ps1 -Tag v0.6.0 -Version 0.6.0 -PreviousTag v0.4.0 -CandidateDir .hermes-0.6/witness-candidates -EvidenceName sandbox-upg040-machine-override -PreservationFlavor cache -AllowStoredGitHubLogin` exited 0. NSIS and MSI fresh install/launch/uninstall, NSIS update/uninstall and native preservation all passed. Each installed-application smoke probe rendered the shell with eight navigation buttons. The installed baseline seeded six keys; the installed candidate recovered all six. Independent comparison of retained reads to the original fixture confirmed exact equality for all six values, including the tuning value. The retained settings digest is `2b7ba23c5f06c8b7abef7d93a2c0114f50f838f3f725c97b9cb129d5ff8f8391`. The owned Sandbox was removed. Evidence: `release-evidence/0.6.0/attestations/sandbox-upg040-machine-override*`; archive `.hermes-0.6/native-preservation-7887e5d.zip` preserves nine files byte-for-byte (SHA-256 `1b7d55d5ab182a17f3cda3d5383def75c2b5bfe167bba187e7394bb2bccede55`). Candidate source remains `eb9ce5828aeb4b2b284860a45371e8de35ebf353`; no final-source qualification is claimed.
- Local verification at `7887e5d`: `npm run check` passed 144 Vitest tests and 269 script tests, strict types, repository verifiers and the production frontend build. Formatting, locked all-target clippy, version/workflow gates and the cleanup matrix passed. The first default-parallel Rust run passed 619/0/7; the second failed 618/1/7 at `download.rs:4045` in `dc02_an_interrupted_no_range_transfer_retries_from_zero` with a loopback connection failure after the probe and two zero-offset transfer requests. The third run did not execute. Logs: `.hermes-0.6/pr47-final-{check,clippy,rust-1,rust-2,cleanup}.log`. This is an unresolved failure, not a passing soak; no new push or merge followed. Follow-up `t_db6f22cd` is assigned to `lead` for isolated investigation. Cancellation source remains byte-identical to accepted main.
- Four disposable-copy mutations disabled settings retention, elevated-guest configuration, registry-key preservation and CLI preflight. Each failed for the intended reason; the unchanged committed source passed the focused suite afterward. Logs: `.hermes-0.6/pr47-final-mutant-{retention,elevated,registry,cli}.log`, `.hermes-0.6/pr47-final-restored.log`.
- Scope: the native-runner deferral below was revisited and resolved for the tested configuration. No alternative VM is needed for this rehearsal. The final candidate and its required native matrix still need qualification. The existing cache verifier checks presence/parseability, not exact cache contents; the native tuning equality above has an independent value comparison because the existing verifier checks only tuning-value presence. These limits are not waivers.
- Integration handoff: qualification transfer is committed locally as `b1945763a7c78dbb8cc8e0493315bda509fa518c` on `fix/u06-02-candidate-evidence-transfer`. Its committed-head `npm run check` passed 144 Vitest tests and 301 script tests. The actual workflow assembly command consumes the separate transfer directory; declared settings reads travel with their lifecycle record and receive digest checks before assembly and during promotion. Producer execution remains incomplete; no final candidate is qualified. The branch remains unpushed.
- Rust follow-up state: implementation `t_a90d91e7` and gated review `t_35dc844d` own the DC02 correction. A diagnostic run also exposed `proc::tests::dropping_a_contained_process_terminates_descendants` at `proc.rs:806`: `the descendant fixture did not start (spawned=true, parent_exit=Some(ExitStatus(ExitStatus(0))), elapsed=45.0144615s)`. The raw log was inspected at `C:/Users/Mubarak/AppData/Local/hermes/kanban/boards/fallen-king-salhadaar-replay/workspaces/t_a90d91e7/evidence/dc02-diagnostic-full-7.log`; follow-up `t_0151e9c4` retains that distinct failure. Neither failure is waived by later passing runs. No correction has been integrated into PR 47 yet.

2026-09-15 / PR 47 committed verification and cleanup correction:

- Updated PR 47 head `ca5310e6b11c140686172e9efd62b6c15cb19f8c` includes the coherent settings/harness repair and accepted main `4ff042e`. Required `pr-check` run [34928698434](https://github.com/sato942/localmotive/actions/runs/34928698434) passed on that exact head. Local `npm run check`, version/workflow verifiers, cleanup matrix, formatting, clippy, documentation tests and dependency audit passed. Three default-parallel Rust runs each returned 619 passed, 0 failed, 7 ignored. The live host payload sequence recovered six settings keys and rejected deliberate loss/corruption under PowerShell 5.1. Logs: `.hermes-0.6/pr47-{check,clippy,rust-1,rust-2,rust-3,doc,npm-audit,packaged-cleanup,host-settings,ci-complete}.log`.
- Follow-up regression: a throwing `CloseMainWindow()` skipped forced termination and left the owned child alive. The new assertion failed with `Failed graceful close orphaned owned child` before the fix. The helper now attempts forced termination even after graceful close throws, joins the owned child, and then reports the original verifier/close failures. The regression passed under PowerShell 7 and 5.1; live host settings/loss/corruption controls also passed. Logs: `.hermes-0.6/pr47-close-error-{red,green,ps51,host}.log`. This correction needs its own committed-head checks; the successful `ca5310e` CI is not reused for changed code.
- Native facility discovery: `wsb.exe` is available and `wsb.exe list --raw` reports no running environments. The hypervisor is present. `Get-VM`, `vmrun` and `VBoxManage` are unavailable in the inspected shell; no alternative isolated test session is verified. This does not prove that no VM exists. Native settings access remains the recorded CDP blocker, not a hardware deferral or a waiver.

2026-09-15 settings correction evidence (original working-tree base `e7feee726d887bfe50ed747019afb4ea3eb22770`; committed as `c396a78f75602525fbdaf81b74db4511d2bef004` before merging current main. PR 47's earlier `pr-check` [34916604366](https://github.com/sato942/localmotive/actions/runs/34916604366) covers only the original base):

- Cancellation RED: `cargo test --locked --lib -- --nocapture` reproduced the resumed loopback refusal. `.hermes-0.6/u06-cancel-diagnostic-parallel.log` records `bytes=0-0`, `bytes=0-65535`, observed progress `4096/65536`, listener exit at `30.0040188s` after two requests, and then resume probe error `Os { code: 10061, kind: ConnectionRefused }`. This is a fixture-lifetime failure, not an external Hugging Face failure.
- Cancellation GREEN: scoped fixture ownership replaces the fixed lifetime/request-count exit; concurrent connections prevent an idle connection from blocking the resumed probe; observed-progress cancellation releases the partial response through a channel. `cargo test --manifest-path src-tauri/Cargo.toml --locked cancellation_ -- --nocapture`: 17 passed. Download module: 57 passed, 1 ignored. Three default-parallel `cargo test --manifest-path src-tauri/Cargo.toml --locked` runs each returned 619 passed, 0 failed, 7 ignored (`.hermes-0.6/u06-cancel-default-{1,2,3}.log`). `cargo fmt --check` and locked all-target clippy with `-D warnings` passed. Production download behavior is unchanged; diagnostics are test-only.
- Cancellation mutation: disabling resume-state reuse made the scenario fail at `resume must use the saved offset`, with a restarted `bytes=0-65535` request. The mutation was removed. Restored cancellation tests passed 17/17 (`.hermes-0.6/u06-cancel-reset-mutant.log`, `.hermes-0.6/u06-cancel-restored.log`).
- Settings helper RED/GREEN: `pwsh -NoProfile -File scripts/tests/verify_settings_session.ps1` failed before the repair because baseline storage was deleted. The corrected helper preserves the scenario directory across sessions, closes its owned application, and restores the environment. The executable helper regression passed, including verifier failure cleanup. CDP discovery is stubbed in this regression; it does not prove application-level preservation. Native leg `sandbox-upg040-session-lifetime` is running on actual inventory-bound candidate `eb9ce5828aeb4b2b284860a45371e8de35ebf353`; it is a correction rehearsal, not final-source qualification.
- Required local frontend/gate suite: `npm run check < /dev/null` passed (144 Vitest tests, 260 script tests, type checks, repository verifiers and production frontend build). Log: `.hermes-0.6/u06-required-check.log`. Required new-head CI, native settings readback, and final-candidate qualification remain OPEN.
- Native result: `sandbox-upg040-session-lifetime` failed before baseline seeding with `Preservation baseline exposed no CDP page target within 90 seconds`. NSIS/MSI install, executable identity, upgrade identity and uninstall observations passed; process survival did not establish a rendered shell. Settings preservation was not exercised. The retained FAIL record/log are `release-evidence/0.6.0/attestations/sandbox-upg040-session-lifetime.{json,log}`. These records name the harness base; the uncommitted session-lifetime correction was staged for this diagnostic rehearsal.
- Isolated startup diagnosis: the installed v0.4.0 application had a responsive `Localmotive` window and a WebView2 renderer. Its browser command line honored the user-data folder but omitted the requested debugger flag; no listener appeared at 10093. Removing the unsupported `--user-data-dir` flag, using direct process creation, and using the documented per-application debugging configuration did not resolve discovery. Selecting the host's signed WebView2 152.0.4191.66 instead of the guest's 153.0.4234.32 also did not resolve it. None of these diagnostic variants changed release source or security policy enforcement. Logs: `.hermes-0.6/u06-settings-{diagnostic,arguments-diagnostic,direct-launch,registry-diagnostic,runtime152-diagnostic}.log`. The diagnostic Sandbox was stopped. The specific reason the debugger flag is omitted remains UNKNOWN; no longer timeout or full lifecycle retry was used.
- Host comparison: the exact extracted baseline payload `af9f01ac31730798b584efd1aee8f8854e97cc894573106c6935acb0461ebb93` and candidate NSIS payload `7891d815756d646963b9e529da261be0b8f505e603c1b57b1926a7c657c0f18b` successfully seeded and recovered all 6 settings keys in an isolated host data root. Both digests match the native installation observations. This demonstrates the helper's real application behavior, not the required installer lifecycle. The diagnostic script is `.hermes-0.6/u06-settings-host-payload.ps1`.
- Windows PowerShell 5.1 exposed a second verifier defect after successful readback: `Unexpected UTF-8 BOM (decode using utf-8-sig)`. Three new controls failed for that exact reason before changing the collected-settings reader to `utf-8-sig`. The corrected reader accepts the guest shell's output encoding without relaxing value checks. Real application readback under Windows PowerShell 5.1 then passed. Deliberate live deletion of the runtime setting and corruption of the stored profile each produced the expected verifier failure. Logs: `.hermes-0.6/u06-settings-{host-ps51-console,bom-red,bom-green,host-ps51-green,live-mutations}.log`. Native Sandbox preservation remains unverified.
- Cancellation correction merged through [PR 48](https://github.com/sato942/localmotive/pull/48): PR source `9fd4781f5aeb58334b8106cbf449191393d77826`, merge `4ff042e33f5e58531b1232510e160af57fe0abca`. Exact-head `pr-check` [34921834667](https://github.com/sato942/localmotive/actions/runs/34921834667) passed before the protected merge. Trusted-main CI [34922732310](https://github.com/sato942/localmotive/actions/runs/34922732310) then passed `check`, `rust-audit` and `package-smoke` on `DESKTOP-HPTF57N-zen5-blackwell`. Its default-parallel Rust run passed the complete cancellation/resume scenario, idle-connection case and unwind-join case: 619 passed, 0 failed, 7 ignored. Logs: `.hermes-0.6/u06-cancel-main-{check,package-smoke}.log`. The local Rust file is byte-identical to the merged file. Settings changes remain uncommitted on PR 47's branch; no native preservation or release completion is claimed.
- Follow-up read-only review found relevant harness defects. RED/GREEN controls now cover explicit local opt-in for stored GitHub credentials (never allowed in CI), retained authentication-failure documents, owned-ID Sandbox cleanup, visible combined verifier/cleanup failures, joined smoke processes, and cache-only/mirror-only collection with missing or locked-file rejection. Logs: `.hermes-0.6/u06-{auth-boundary-red,auth-boundary-green,sandbox-ownership-red,sandbox-ownership-green,session-cleanup-error-red,smoke-cleanup-red,preservation-collection-red,collection-copy-failure-red,review-regressions-green}.log`. The plain-file helper test remains helper evidence only. Optional fixture expansions from the review do not change the release boundary.
- Real Sandbox ownership proof: this host rejects a second Sandbox with `0x800401F6 (CO_E_APPSINGLEUSE)`. The actual lifecycle wrapper retained a candidate-bound FAIL document without stopping the existing fixture Sandbox. The real cleanup function then removed only that owned fixture ID. Readback showed no remaining Sandboxes. `.hermes-0.6/u06-sandbox-ownership-refusal.log` and `release-evidence/0.6.0/attestations/sandbox-ownership-refusal.{json,log}` retain the observations; the failed two-instance experiment is `.hermes-0.6/u06-sandbox-owned-cleanup-live.log`, not a PASS.
- Harness implementation contract: native lifecycle execution now requires the installed `wsb.exe` CLI, whose successful start returns an owned environment ID. The harness no longer terminates Sandboxes by process name. Owned Sandbox destruction clears guest-local temporary settings storage; the guest no longer recursively deletes a potentially redirected profile path. This changes the harness mechanism, not product compatibility or preservation acceptance.
- Post-review verification: `npm run check < /dev/null` passed (144 Vitest tests, 266 script tests, type checks, repository verifiers and production frontend build). All three PowerShell regression harnesses also passed under Windows PowerShell 5.1. Real baseline-to-candidate payload readback and live loss/corruption controls passed again with the reviewed helper. Logs: `.hermes-0.6/u06-reviewed-settings-required-check.log`, `.hermes-0.6/u06-review-{session,collection,ownership}-ps51.log`, `.hermes-0.6/u06-reviewed-settings-live-mutations.log`. Full native installer preservation remains OPEN; these results do not substitute for it.
- Native-runner setup: `tauridev: Sandbox native settings access remains unavailable and no alternative isolated Windows session is verified, revisit on a new testable CDP hypothesis or availability of an isolated test session.` A new VM is one possible method, not a release requirement. A second account alone does not isolate machine-wide installer effects. The owner directed independent producer/qualification work to continue; no general authorization request or native waiver applies.
- PR 47 preservation checkpoint: the working source and three new PowerShell regressions were archived in `.hermes-0.6/pr47-preserved-20260915T041657Z.zip`. The local cancellation source matches accepted main byte-for-byte. The focused settings/authentication/cleanup/collection and PowerShell UTF-8 tests passed in `.hermes-0.6/pr47-current-focused.log`. These results cover the repaired working tree, not the stale pushed `e7feee7` checks. The native limitation and host-payload evidence above remain unchanged.

Historical pre-merge cancellation evidence from `4ff042e` follows. Its pending-CI wording describes that earlier checkpoint; the accepted closure and successful trusted-main CI above supersede it.

2026-09-15 cancellation correction (base `7b924f31cd83dee8051d7baec7f76a0d638ff5cf`; settings corrections remain separate):

- RED: the default-parallel diagnostic run recorded `bytes=0-0`, `bytes=0-65535`, observed progress `4096/65536`, and listener exit at `30.0040188s` after two requests. The resumed probe then failed with `Os { code: 10061, kind: ConnectionRefused }`. This is a loopback fixture-lifetime failure, not an external Hugging Face failure. Log: `.hermes-0.6/u06-cancel-diagnostic-parallel.log` in the main development checkout.
- GREEN: scenario-owned scoped threads, concurrent connection handling, cancellation-triggered response release and explicit socket shutdown replace timing/request-count lifetime assumptions. The scenario verifies partial bytes, valid retained state, exact resumed offset, complete payload and final cleanup. Separate tests cover an idle connection, more than four probes, early return and client panic. Production download behavior is unchanged; nested loopback errors are test-only diagnostics.
- `cargo test --manifest-path src-tauri/Cargo.toml --locked cancellation_ -- --nocapture`: 17 passed. Download module: 57 passed, 1 ignored. Three default-parallel full runs each returned 619 passed, 0 failed, 7 ignored. Formatting and locked all-target clippy with `-D warnings` passed. Logs: `.hermes-0.6/u06-cancel-{green-focused,module,default-1,default-2,default-3,clippy}.log` in the main development checkout.
- Mutation: disabling resume-state reuse failed at `resume must use the saved offset`; the captured sequence showed the erroneous restart at byte zero. The mutation was removed and the restored cancellation suite passed 17/17. Logs: `.hermes-0.6/u06-cancel-{reset-mutant,restored}.log`. Required new-head CI remains pending; neither this correction nor the separate settings diagnostic closes final-candidate qualification.
- Cancellation-only branch verification: `npm run check < /dev/null` passed (144 Vitest tests, 257 script tests, type checks, repository verifiers and production frontend build). Formatting, locked all-target clippy with `-D warnings`, and the default-parallel full Rust suite passed (619 passed, 0 failed, 7 ignored). Logs: `.hermes-0.6/u06-cancel-pr-{check,clippy,rust}.log` in the main development checkout. No installer or release rebuild is needed for this test-only correction.


| Date / package | Status | Source / harness / artifacts | Actual command or observation | Result and retained evidence | Remaining scope |
|---|---|---|---|---|---|
| 2026-09-14 / consolidation | Documentation complete; release OPEN | Reviewed 1881db9; original and residual input hashes above | Full criterion import, live CI/ruleset/PR readback, bounded source review and isolated manifest contract probe | E01..E09; no product code or historical artifact/evidence rewrite | U06-01..09; D06 hardware exclusions; non-hardware inputs as stated |
| 2026-09-14 / U06-01 | Packaged-orchestration correction merged (PR 29, `9832131`); CI-wired pwsh matrix green | Producer `ae448a7`; harness `scripts/verify_packaged_impl.ps1` + wrapper `scripts/verify_packaged_matrix.ps1`; matrix `scripts/tests/verify_cleanup_matrix.ps1` | `pwsh matrix 12/12` (also 5.1 12/12 informative); PR 29 `pr-check` run 34844472885 pass 10m49s incl. new CI step; release-gates 147/147 + workflow-integration 10/10 locally; RED proof recorded in PR body (old Stop-Candidate never sets functionalFailed; old adoption falls back to process-name acceptance) | Merged to main `ae448a7`; main push CI 34845858794 SUCCESS (check, rust-audit, Security audit, package-smoke; pr-check skipped on push) | U06-01 done at this harness/product revision; U06-02/04/06 still need the candidate produced from this source |
| 2026-09-14 / U06-03 | hardware-qualify trailing-quote repair merged (PR 30, `a4710b9`); host attestation executed on the available host | Workflow `.github/workflows/hardware-qualify.yml` at `9cbb9fd`; generator `scripts/qualification/build_host_attestation.mjs`; mismatch/unknown controls covered by generator tests | PR 30 `pr-check` run 34844686066 pass 11m5s; dispatched hardware-qualify run 34847400771 SUCCESS at source `9cbb9fd` (workflow_dispatch tag v0.6.0); attestation artifact `hardware-attestation-0.6.0` downloaded: MATCH (Ryzen 9 9950X3D, RTX 5090, driver 32.0.16.1074, Win 10.0.26100.0); RED proof: new gate test fails on prior source (stray quote + pwsh parse error), passes after | Attestation `.hermes-0.6/hw-attest-34847400771/host-zen5-blackwell.json`; U06-03 repair+execution done | Host proof is not inference qualification and not deferred-vendor coverage; lifecycle/qualification still open |
| 2026-09-14 / U06-07 | S-25 available-scope metrics collected; a11y automated probe re-executed on current binary | Probe `scripts/verify_a11y.mjs` against `src-tauri/target/release/localmotive.exe` (sha `d6a6ef66…`, source `e9a36b3` build); queue/failure/retention reads from live CI + local stores | a11y A11Y_PASS, byte-identical to committed `g05-a11y-packaged-verification.log`; benchmark store 213 records / anchors 13 (bound 4000); main-branch last-20 CI: 19 success + 1 cancelled (34845625575, superseded push, not a failure); artifact retention 14/30/90 days | `.hermes-0.6/u06-07/a11y-new.log`, `.hermes-0.6/u06-07/s25-metrics.log` | Held-out data/program inputs still missing (BLOCKED-INPUT); manual Narrator/NVDA + live accounts still open per G-05.I3 |
| 2026-09-15 / U06-00 | Cancellation-test race fixed and merged (PR 34, `c4746b7`); U06-02 map recovered | Test `download::tests::cancellation_retains_valid_state_and_the_next_attempt_resumes` in `src-tauri/src/download.rs`; map `docs/QUALIFICATION-MAP-0.6.md` from unmerged PR 33 branch `5e36537` unchanged | RED proof: old test with a 2 s probe hold fails at `part.is_file` (download.rs:2003) — the PR 33 failure mode. GREEN: corrected test passes solo x3; sibling early-cancel green; download module 55/55 x2 at 16 threads; full lib 617/617; fmt clean; clippy zero warnings | Merged to main `8d80896`; PR 34 `pr-check` run 34867277657 SUCCESS in 10m48s (all steps incl. Rust tests green) | U06-02 producer execution, U06-04 native legs, U06-05/06/07/08/09 still open |

Add actual results here; do not prefill success templates. Every new record names the producing revision and candidate bytes. Current release decisions may use inherited evidence only within the contract's explicit scope.

## Complete original-criterion ledger

The following is the complete criterion text from the original tracker, grouped by package. Checkbox changes are limited to the explicit status-change table. Original wording, historical claims and source links are retained as evidence; current status annotations above/below govern execution. Checked entries mean accepted completion at the original evidence's scope, not that this consolidation reran hundreds of tests.

### V06-RT-01

<a id="criterion-v06-rt-01-i1"></a>
- [x] **V06-RT-01.I1** — Separate the destination policy for new approved installations from discovery of existing GGUF Pilot installations; select the primary Localmotive runtime root for new installs even when only the legacy directory exists. **Trace:** [Audit RT-01](docs/history/localmotive-comprehensive-audit.md#rt-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L207).

<a id="criterion-v06-rt-01-i2"></a>
- [x] **V06-RT-01.I2** — Keep legacy discovery and any migration explicit, preserving existing files until the replacement has passed compiled-content verification and publication has succeeded. **Trace:** [Audit RT-01](docs/history/localmotive-comprehensive-audit.md#rt-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L208).

<a id="criterion-v06-rt-01-i3"></a>
- [x] **V06-RT-01.I3** — Align install, repair, reuse, public inspection, and launch validation so a successful installation result cannot return a path that the same application categorically rejects as legacy. **Trace:** [Audit RT-01](docs/history/localmotive-comprehensive-audit.md#rt-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L209).

<a id="criterion-v06-rt-01-i4"></a>
- [x] **V06-RT-01.I4** — Update the existing root-selection regression and recovery messages to express the selected policy; retain architecture, content-manifest, reparse, backup, and rollback protections. **Trace:** [Audit RT-01](docs/history/localmotive-comprehensive-audit.md#rt-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L210).

<a id="criterion-v06-rt-01-v1"></a>
- [x] **V06-RT-01.V1** — Compose root selection, installation/publication using an inert verified fixture, and the public inspection trust gate with an empty legacy directory and absent primary directory; assert that the returned runtime is accepted. **Trace:** [Audit RT-01](docs/history/localmotive-comprehensive-audit.md#rt-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L214).

<a id="criterion-v06-rt-01-v2"></a>
- [x] **V06-RT-01.V2** — Repeat the scenario with corrupt legacy records, a populated primary root, and repair/reuse requests; verify that reinstall does not reproduce the legacy rejection loop. **Trace:** [Audit RT-01](docs/history/localmotive-comprehensive-audit.md#rt-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L215).

<a id="criterion-v06-rt-01-v3"></a>
- [x] **V06-RT-01.V3** — On the supported packaged Windows application, reproduce the upgrade layout and record a successful install followed by inspection and launch, including preservation of prior files on a failed migration. **Trace:** [Audit RT-01](docs/history/localmotive-comprehensive-audit.md#rt-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L216).

### V06-RT-02

<a id="criterion-v06-rt-02-i1"></a>
- [x] **V06-RT-02.I1** — Centralize managed-runtime content authorization at the lowest shared execution or probe boundary so callers cannot bypass it by directly invoking core runtime inspection. **Trace:** [Audit RT-02](docs/history/localmotive-comprehensive-audit.md#rt-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L237).

<a id="criterion-v06-rt-02-i2"></a>
- [x] **V06-RT-02.I2** — Route tuning preparation, the older public runtime-health command, normal inspection, and launch through that boundary before executing version, help, or device-discovery probes. **Trace:** [Audit RT-02](docs/history/localmotive-comprehensive-audit.md#rt-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L238).

<a id="criterion-v06-rt-02-i3"></a>
- [x] **V06-RT-02.I3** — Authorize the complete managed installation and the selected companion executable, including llama-cli.exe and its approved DLLs, rather than validating only that the server path is a regular file. **Trace:** [Audit RT-02](docs/history/localmotive-comprehensive-audit.md#rt-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L239).

<a id="criterion-v06-rt-02-i4"></a>
- [x] **V06-RT-02.I4** — Introduce an explicit verified-runtime representation or equally enforceable execution interface; keep intentionally selected external runtimes governed by an explicit separate policy and retain compiled inventory, digest, and filesystem protections. **Trace:** [Audit RT-02](docs/history/localmotive-comprehensive-audit.md#rt-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L240).

<a id="criterion-v06-rt-02-v1"></a>
- [x] **V06-RT-02.V1** — Use inert replaced managed server and CLI binaries that write a sentinel whenever invoked; call tuning preparation and the older health workflow and assert rejection before any sentinel appears. **Trace:** [Audit RT-02](docs/history/localmotive-comprehensive-audit.md#rt-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L244).

<a id="criterion-v06-rt-02-v2"></a>
- [x] **V06-RT-02.V2** — Tamper with an approved DLL while leaving the server executable unchanged and require rejection through the same user-facing workflows. **Trace:** [Audit RT-02](docs/history/localmotive-comprehensive-audit.md#rt-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L245).

<a id="criterion-v06-rt-02-v3"></a>
- [x] **V06-RT-02.V3** — Exercise ordinary managed inspection/launch and intentional external-runtime selection to verify that the centralized boundary preserves their documented behavior and does not rely on frontend validation. **Trace:** [Audit RT-02](docs/history/localmotive-comprehensive-audit.md#rt-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L246).

### V06-RT-03

<a id="criterion-v06-rt-03-i1"></a>
- [x] **V06-RT-03.I1** — Carry the health cancellation signal into completion request execution and response-body reading instead of checking it only immediately before a blocking request. **Trace:** [Audit RT-03](docs/history/localmotive-comprehensive-audit.md#rt-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L263).

<a id="criterion-v06-rt-03-i2"></a>
- [x] **V06-RT-03.I2** — Use an asynchronous cancellation selection or a supervised request worker that can terminate the contained server and interrupt pending completion work within a documented cancellation deadline. **Trace:** [Audit RT-03](docs/history/localmotive-comprehensive-audit.md#rt-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L264).

<a id="criterion-v06-rt-03-i3"></a>
- [x] **V06-RT-03.I3** — Propagate Cancelled consistently when cancellation arrives during response waiting, body reading, or response completion; resolve response-versus-cancel races before recording a successful deterministic completion or overall pass. **Trace:** [Audit RT-03](docs/history/localmotive-comprehensive-audit.md#rt-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L265).

<a id="criterion-v06-rt-03-i4"></a>
- [x] **V06-RT-03.I4** — Keep request/body limits, proxy-free loopback transport, process ownership checks, exact pinned completion comparison, and process/listener cleanup intact; distinguish the existing idle-server shutdown stage from verification of active-request cancellation. **Trace:** [Audit RT-03](docs/history/localmotive-comprehensive-audit.md#rt-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L266).

<a id="criterion-v06-rt-03-v1"></a>
- [x] **V06-RT-03.V1** — Run an owned loopback fixture that accepts the completion request and withholds its response, then cancel after acceptance; assert bounded return with a Cancelled result and no passing completion stage. **Trace:** [Audit RT-03](docs/history/localmotive-comprehensive-audit.md#rt-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L270).

<a id="criterion-v06-rt-03-v2"></a>
- [x] **V06-RT-03.V2** — Cancel during response-body reading and at the response-completion boundary; require deterministic terminal status and verify that a user-cancelled run cannot become an overall pass. **Trace:** [Audit RT-03](docs/history/localmotive-comprehensive-audit.md#rt-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L271).

<a id="criterion-v06-rt-03-v3"></a>
- [x] **V06-RT-03.V3** — Confirm that the contained child and loopback listener are stopped and temporary files are cleaned after cancellation, and verify that the ordinary successful health sequence still passes. **Trace:** [Audit RT-03](docs/history/localmotive-comprehensive-audit.md#rt-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L272).

### V06-RT-04

<a id="criterion-v06-rt-04-i1"></a>
- [x] **V06-RT-04.I1** — Extend the shared verified-runtime execution boundary to return a lease that retains restrictive read-sharing handles for the approved executable and DLL inventory, rather than discarding each handle after hashing. **Trace:** [Audit RT-04](docs/history/localmotive-comprehensive-audit.md#rt-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L289).

<a id="criterion-v06-rt-04-i2"></a>
- [x] **V06-RT-04.I2** — Retain and validate the relevant directory identities with the lease, and keep protection effective through process creation and loading of the files covered by the execution-identity guarantee. **Trace:** [Audit RT-04](docs/history/localmotive-comprehensive-audit.md#rt-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L290).

<a id="criterion-v06-rt-04-i3"></a>
- [x] **V06-RT-04.I3** — Carry the lease through managed health context preparation, any pinned health-model download, and the subsequent CLI, benchmark, and server launches so the long preparation interval does not reopen a modification window. **Trace:** [Audit RT-04](docs/history/localmotive-comprehensive-audit.md#rt-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L291).

<a id="criterion-v06-rt-04-i4"></a>
- [x] **V06-RT-04.I4** — Coordinate runtime repair and replacement with active leases, defining a clear wait or rejection outcome; preserve backup/rollback behavior and avoid replacing cryptographic approval with a last-moment path recheck. **Trace:** [Audit RT-04](docs/history/localmotive-comprehensive-audit.md#rt-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L292).

<a id="criterion-v06-rt-04-v1"></a>
- [x] **V06-RT-04.V1** — Use a synchronized writer that attempts to replace an approved executable after verification succeeds but before spawn; require replacement to fail while protected or rejection before an inert marker executable runs. **Trace:** [Audit RT-04](docs/history/localmotive-comprehensive-audit.md#rt-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L296).

<a id="criterion-v06-rt-04-v2"></a>
- [x] **V06-RT-04.V2** — Repeat with DLL replacement and with the health-model download deliberately delayed between context preparation and runtime execution. **Trace:** [Audit RT-04](docs/history/localmotive-comprehensive-audit.md#rt-04). **Closed (2026-09-12, fourth pass):** packaged 17/17 PASS with a controlled slow fixture (≈4 MB/s loopback server streaming the pinned health model): the download is observed in flight, an in-window swap attempt is DENIED by the read-shared execution lease (EBUSY, the RT-04 no-time-based-release mechanism), tampering both verified runtime copies outside the window yields `trust_failure` — `Managed runtime health trust verification failed: Managed runtime cuda-13.3 is not installed or fails content verification` — with zero spawned children, and restoring the exact bytes (`87c4e9d0…`) returns a full seven-stage PASS. Evidence: `release-evidence/0.6.0/attestations/rt04v2-delayed-download-verification.log` (run log `.hermes-0.6/freeze5-rt04v2-g.log`; driver `scripts/g05_rt04v2_delay.mjs`; seam `download::rebase_download_url` gated to verifier-profile loopback HTTP).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L297).

<a id="criterion-v06-rt-04-v3"></a>
- [x] **V06-RT-04.V3** — Exercise repair/replacement while a lease is active and after release; verify the documented outcome, intact approved content, and no abandoned locks or handles. **Trace:** [Audit RT-04](docs/history/localmotive-comprehensive-audit.md#rt-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L298).

### V06-RT-05

<a id="criterion-v06-rt-05-i1"></a>
- [x] **V06-RT-05.I1** — Reuse a bounded streaming response-body reader for runtime catalog success and error statuses, including HTTP 403, enforcing the 2 MiB limit before retaining or formatting oversized content. **Trace:** [Audit RT-05](docs/history/localmotive-comprehensive-audit.md#rt-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L315).

<a id="criterion-v06-rt-05-i2"></a>
- [x] **V06-RT-05.I2** — Open discovered runtime.json records as regular protected files, check metadata from the opened handle, and read at most the 64 KiB limit plus one detection byte before parsing. **Trace:** [Audit RT-05](docs/history/localmotive-comprehensive-audit.md#rt-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L316).

<a id="criterion-v06-rt-05-i3"></a>
- [x] **V06-RT-05.I3** — Count every visited filesystem entry, including empty directories, during managed-install collection; add a bounded traversal depth and reject excessive work before growing the pending traversal without limit. **Trace:** [Audit RT-05](docs/history/localmotive-comprehensive-audit.md#rt-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L317).

<a id="criterion-v06-rt-05-i4"></a>
- [x] **V06-RT-05.I4** — Preserve typed failure categories, compiled installation verification, and useful bounded error messages; do not confuse the HTTP timeout or final payload validation with allocation and traversal limits. **Trace:** [Audit RT-05](docs/history/localmotive-comprehensive-audit.md#rt-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L318).

<a id="criterion-v06-rt-05-v1"></a>
- [x] **V06-RT-05.V1** — Serve chunked HTTP 403 bodies at the configured byte limit and one byte beyond it; assert bounded retained data, early oversized-body rejection, and normal handling of a small rate-limit response. **Trace:** [Audit RT-05](docs/history/localmotive-comprehensive-audit.md#rt-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L322).

<a id="criterion-v06-rt-05-v2"></a>
- [x] **V06-RT-05.V2** — Inspect an oversized sparse runtime.json record and exact-boundary valid/malformed records; verify that discovery cannot allocate the entire oversized file before rejection. **Trace:** [Audit RT-05](docs/history/localmotive-comprehensive-audit.md#rt-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L323).

<a id="criterion-v06-rt-05-v3"></a>
- [x] **V06-RT-05.V3** — Build trees with excessive empty directories and excessive nesting, plus valid boundary inventories; assert bounded traversal and unchanged reparse/non-regular-entry rejection. **Trace:** [Audit RT-05](docs/history/localmotive-comprehensive-audit.md#rt-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L324).

### V06-RT-06

<a id="criterion-v06-rt-06-i1"></a>
- [x] **V06-RT-06.I1** — Use the corrected primary-root policy to make root lookup and cheap installation discovery independent of full payload hashing; expose discovery without prematurely asserting cryptographically verified status. **Trace:** [Audit RT-06](docs/history/localmotive-comprehensive-audit.md#rt-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L341).

<a id="criterion-v06-rt-06-i2"></a>
- [x] **V06-RT-06.I2** — Move expensive content verification to blocking workers with cancellation checks and progress reporting, including preparatory work used by installation, description, selection, and managed health. **Trace:** [Audit RT-06](docs/history/localmotive-comprehensive-audit.md#rt-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L342).

<a id="criterion-v06-rt-06-i3"></a>
- [x] **V06-RT-06.I3** — Coalesce simultaneous verification requests for the same installation and retain results through the verified-install lease, with invalidation tied to the lifetime and identity guarantees established for execution. **Trace:** [Audit RT-06](docs/history/localmotive-comprehensive-audit.md#rt-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L343).

<a id="criterion-v06-rt-06-i4"></a>
- [x] **V06-RT-06.I4** — Instrument verification jobs and bytes hashed across listing, selection, and launch so repeated work is visible; never substitute mtime/size equality alone for compiled-content integrity checks. **Trace:** [Audit RT-06](docs/history/localmotive-comprehensive-audit.md#rt-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L344).

<a id="criterion-v06-rt-06-v1"></a>
- [x] **V06-RT-06.V1** — Assert that root lookup performs no full content scan and that simultaneous requests for one installation share a verification job without incorrectly sharing work across different installations. **Trace:** [Audit RT-06](docs/history/localmotive-comprehensive-audit.md#rt-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L348).

<a id="criterion-v06-rt-06-v2"></a>
- [x] **V06-RT-06.V2** — Cancel a long verification operation and assert prompt worker termination and a truthful unverified/cancelled result; verify that changed content cannot inherit a stale verified label. **Trace:** [Audit RT-06](docs/history/localmotive-comprehensive-audit.md#rt-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L349).

<a id="criterion-v06-rt-06-v3-available"></a>
- [x] **V06-RT-06.V3 (achievable four-backend scope)** — Measure bytes hashed, job counts, and elapsed time for listing, selecting, and launching on a representative supported Windows host with all seven backends installed; record the environment and results. Run on this host (Ryzen 9 9950X3D + RTX 5090, adapter `luid:0000000000014f4f`, catalog `b10816`): the catalog exposes all seven Windows x64 backends; four are eligible on a single-vendor host and were exercised (cpu reused 169-181 ms; cuda-12.4 reused 717-752 ms; cuda-13.3 reused 506-527 ms; vulkan freshly installed 2,916 ms at 35,228,033 bytes); the three non-NVIDIA backends are refused by the runtime-compatibility guard with named reasons (`GPU adapter luid:... (nvidia) is incompatible with the {openvino,rocm,sycl} runtime`; with no adapter, `The selected runtime requires one bounded GPU adapter ID`) - a seven-simultaneous-backend host would need three GPU vendors, so the maximum eligible set was qualified and the refusals proven. Listing: 4 records, second pass hashed 0 bytes (the verified-install lease/cache works); selecting: `describe_runtime` 379-388 ms with `managedVerified=true`; launching: managed health run PASS in 3,336-3,438 ms; instrumentation counters (jobs started/coalesced/bytes hashed/cancelled) captured around every step. Driver `scripts/g05_rt06_all_backends.mjs` **22/22 checks PASS** on the packaged binary (freeze-9: per-backend installed/selected/launched detail plus the three named vendor refusals; evidence `release-evidence/0.6.0/attestations/rt06-all-backends.json` + `rt06-full-run.log`). The earlier freeze-8 statement "RT-06.V3 closed" with "12/12" is **superseded** by the scoped entries here and by the scoped review; it is preserved as history, not rewritten. **Trace:** [Audit RT-06](docs/history/localmotive-comprehensive-audit.md#rt-06).
  **Current status: VERIFIED.** Accepted available-host installed/refused scope only; the separate seven-installed line below is deferred. Regenerate this host-scope record for the final candidate under U06-02. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L350).

<a id="criterion-v06-rt-06-v3-deferred-hardware"></a>
- [ ] **V06-RT-06.V3 (seven-installed-backend acceptance portion) — environment-blocked:** the acceptance wording asks for all seven backends installed on a representative host. This host runs one GPU vendor (NVIDIA, adapter `luid:0000000000014f4f`), so the maximum eligible set is four installed/selected/launched backends plus the three vendor refusals; a seven-simultaneous-backend host needs three GPU vendors. Unblock action: run `scripts/g05_rt06_all_backends.mjs` (candidate bound from the committed inventory) on a three-GPU-vendor Windows host. This box must not be closed from fixture evidence or relabelled from the historical 12/12. **Trace:** [Audit RT-06](docs/history/localmotive-comprehensive-audit.md#rt-06).
  **Current status: DEFERRED-OWNER.** D06-01: owner-deferred seven-installed hardware coverage; available-host record is still required by U06-02. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L351).

### V06-RT-07

<a id="criterion-v06-rt-07-i1"></a>
- [x] **V06-RT-07.I1** — Replace the test-only archive-verification implementation with a production verification/extraction path that the actual installer uses after download completion. **Trace:** [Audit RT-07](docs/history/localmotive-comprehensive-audit.md#rt-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L368).

<a id="criterion-v06-rt-07-i2"></a>
- [x] **V06-RT-07.I2** — Open the archive with appropriate identity and sharing protections, verify approved size and SHA-256, and extract from that same opened file so reopening cannot introduce another check/use gap. **Trace:** [Audit RT-07](docs/history/localmotive-comprehensive-audit.md#rt-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L369).

<a id="criterion-v06-rt-07-i3"></a>
- [x] **V06-RT-07.I3** — Remove duplicate security logic guarded solely by cfg(test), and route archive mutation regressions through the production installer or its real extraction boundary. **Trace:** [Audit RT-07](docs/history/localmotive-comprehensive-audit.md#rt-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L370).

<a id="criterion-v06-rt-07-i4"></a>
- [x] **V06-RT-07.I4** — Preserve capability-relative create-new extraction, traversal/ADS/symlink rejection, decompression limits, cancellation, final compiled per-file inventory verification, and verified publication/rollback. **Trace:** [Audit RT-07](docs/history/localmotive-comprehensive-audit.md#rt-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L371).

<a id="criterion-v06-rt-07-v1"></a>
- [x] **V06-RT-07.V1** — Replace the archive between completed download and actual production extraction with same-size changed content and with a changed-size file; assert rejection before any archive entry is extracted. **Trace:** [Audit RT-07](docs/history/localmotive-comprehensive-audit.md#rt-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L375).

<a id="criterion-v06-rt-07-v2"></a>
- [x] **V06-RT-07.V2** — Attempt symlink/reparse replacement and a path change after the archive is opened; verify that the protected original handle remains authoritative or the operation fails safely. **Closed 2026-09-12:** `runtime::tests::rt07_path_and_reparse` (real NTFS: junction swap after open, symlink-payload archive open+verify refusal, reparse-ancestor refusal); mutant MR1 (share-mode narrowed) failed the test and passed after restore. **Trace:** [Audit RT-07](docs/history/localmotive-comprehensive-audit.md#rt-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L376).

<a id="criterion-v06-rt-07-v3"></a>
- [x] **V06-RT-07.V3** — Cancel during archive verification and confirm bounded cancellation/cleanup, then run an intact approved fixture through the same production path to exercise successful extraction and final content checks. **Trace:** [Audit RT-07](docs/history/localmotive-comprehensive-audit.md#rt-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L377).

### V06-RT-08

<a id="criterion-v06-rt-08-i1"></a>
- [x] **V06-RT-08.I1** — Capture GetLastError immediately after a failed FindNextStreamW call, before FindClose or any other Win32 operation, and use the captured value to recognize ERROR_HANDLE_EOF. **Trace:** [Audit RT-08](docs/history/localmotive-comprehensive-audit.md#rt-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L394).

<a id="criterion-v06-rt-08-i2"></a>
- [x] **V06-RT-08.I2** — Capture the failure status from GetProcessMemoryInfo before CloseHandle, preserving the actual measurement error in the returned unknown-evidence diagnostic. **Trace:** [Audit RT-08](docs/history/localmotive-comprehensive-audit.md#rt-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L395).

<a id="criterion-v06-rt-08-i3"></a>
- [x] **V06-RT-08.I3** — Use scoped RAII handle ownership or an equivalent cleanup structure that closes each acquired handle exactly once without replacing the original operation result or error. **Trace:** [Audit RT-08](docs/history/localmotive-comprehensive-audit.md#rt-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L396).

<a id="criterion-v06-rt-08-i4"></a>
- [x] **V06-RT-08.I4** — Keep alternate-stream rejection and valid-file verification semantics unchanged, distinguishing ordinary enumeration completion from extra streams and genuine enumeration failures. **Trace:** [Audit RT-08](docs/history/localmotive-comprehensive-audit.md#rt-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L397).

<a id="criterion-v06-rt-08-v1"></a>
- [x] **V06-RT-08.V1** — Add a Windows API-boundary fixture whose cleanup operation deliberately changes thread-local last-error state; assert that stream enumeration retains the pre-cleanup result. **Trace:** [Audit RT-08](docs/history/localmotive-comprehensive-audit.md#rt-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L401).

<a id="criterion-v06-rt-08-v2"></a>
- [x] **V06-RT-08.V2** — Exercise a normal file with only the default data stream and a file with an extra stream; require acceptance of the former and rejection of the latter without false enumeration errors. **Trace:** [Audit RT-08](docs/history/localmotive-comprehensive-audit.md#rt-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L402).

<a id="criterion-v06-rt-08-v3"></a>
- [x] **V06-RT-08.V3** — Force a process-memory measurement failure followed by cleanup that changes last error; verify that the reported diagnostic identifies the original measurement failure and handles are released. **Trace:** [Audit RT-08](docs/history/localmotive-comprehensive-audit.md#rt-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L403).

### V06-RT-09

<a id="criterion-v06-rt-09-i1"></a>
- [x] **V06-RT-09.I1** — Extend NVIDIA probing to obtain a stable physical identifier, such as PCI location or UUID with an appropriate Windows mapping, and retain the identifier alongside driver/capacity/usage observations. **Trace:** [Audit RT-09](docs/history/localmotive-comprehensive-audit.md#rt-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L420).

<a id="criterion-v06-rt-09-i2"></a>
- [x] **V06-RT-09.I2** — Join NVIDIA observations to DXGI adapters using a verified physical mapping rather than pairing the first unmatched adapter with the same display name. **Trace:** [Audit RT-09](docs/history/localmotive-comprehensive-audit.md#rt-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L421).

<a id="criterion-v06-rt-09-i3"></a>
- [x] **V06-RT-09.I3** — When a trustworthy mapping is unavailable or ambiguous, keep observations unassigned or explicitly unknown instead of attaching device-specific usage/capacity evidence to an arbitrary LUID. **Trace:** [Audit RT-09](docs/history/localmotive-comprehensive-audit.md#rt-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L422).

<a id="criterion-v06-rt-09-i4"></a>
- [x] **V06-RT-09.I4** — Use an explicit runtime-device-to-DXGI identity mapping for managed health selection so a requested adapter ID selects the intended physical device; preserve refusal on unresolved ambiguity and keep existing DXGI budget evidence separate. **Trace:** [Audit RT-09](docs/history/localmotive-comprehensive-audit.md#rt-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L423).

<a id="criterion-v06-rt-09-v1"></a>
- [x] **V06-RT-09.V1** — Provide two identically named adapters with reversed DXGI and NVIDIA enumeration orders and distinct usage values; assert correct stable mapping or explicit unknown/unassigned evidence. **Trace:** [Audit RT-09](docs/history/localmotive-comprehensive-audit.md#rt-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L427).

<a id="criterion-v06-rt-09-v2"></a>
- [x] **V06-RT-09.V2** — Exercise missing, duplicate, and conflicting physical identifiers and confirm that matching does not fall back to arbitrary enumeration order or name-only assignment. **Trace:** [Audit RT-09](docs/history/localmotive-comprehensive-audit.md#rt-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L428).

<a id="criterion-v06-rt-09-v3"></a>
- [x] **V06-RT-09.V3** — Test same-name managed health selection against explicit runtime-device mappings, then validate the mapping on representative supported Windows hardware with multiple identical GPUs when available. **Trace:** [Audit RT-09](docs/history/localmotive-comprehensive-audit.md#rt-09).
  **Current status: VERIFIED.** Accepted implementation/fixture/available-host scope; unprovided physical hardware portions are DEFERRED-OWNER under D06-03/04. No broader hardware observation is claimed. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L429).

### V06-DC-01

<a id="criterion-v06-dc-01-i1"></a>
- [x] **V06-DC-01.I1** — Separate loading an existing catalog from requesting a network refresh. Resolve a supported, signature-verified local snapshot or the bundled fallback before applying the persisted 1,560-minute refresh throttle. **Trace:** [Audit DC-01](docs/history/localmotive-comprehensive-audit.md#dc-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L448).

<a id="criterion-v06-dc-01-i2"></a>
- [x] **V06-DC-01.I2** — Populate the backend's authoritative curated catalog state during local loading, including a fresh app instance, and return origin, last-success time, and remaining cooldown without turning a valid local read into an error. **Trace:** [Audit DC-01](docs/history/localmotive-comprehensive-audit.md#dc-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L449).

<a id="criterion-v06-dc-01-i3"></a>
- [x] **V06-DC-01.I3** — Update the frontend load sequence to display available rows independently of network-refresh success, retain them when refresh is throttled, and show the cooldown alongside the refresh control. **Trace:** [Audit DC-01](docs/history/localmotive-comprehensive-audit.md#dc-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L450).

<a id="criterion-v06-dc-01-i4"></a>
- [x] **V06-DC-01.I4** — Preserve the in-flight refresh guard and network throttling, while keeping signed curated authorization distinct from mutable SQLite browsing data. **Trace:** [Audit DC-01](docs/history/localmotive-comprehensive-audit.md#dc-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L451).

<a id="criterion-v06-dc-01-v1"></a>
- [x] **V06-DC-01.V1** — Add a failing command/UI regression that starts with a signed cache, populated mirror, fresh persisted stamp, and new AppState; require populated rows and no network request. **Trace:** [Audit DC-01](docs/history/localmotive-comprehensive-audit.md#dc-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L455).

<a id="criterion-v06-dc-01-v2"></a>
- [x] **V06-DC-01.V2** — Exercise missing and corrupt caches, bundled fallback, unavailable network, repeated Refresh, and clock rollback; verify that the selected supported snapshot initializes backend authorization. **Trace:** [Audit DC-01](docs/history/localmotive-comprehensive-audit.md#dc-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L456).

<a id="criterion-v06-dc-01-v3"></a>
- [x] **V06-DC-01.V3** — Run the packaged Windows sequence: successful refresh, close, restart within 26 hours, open HF Catalog, and browse/filter a cached entry; record the displayed cooldown and request behavior. **Trace:** [Audit DC-01](docs/history/localmotive-comprehensive-audit.md#dc-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L457).

### V06-DC-02

<a id="criterion-v06-dc-02-i1"></a>
- [x] **V06-DC-02.I1** — Introduce an explicit sequential whole-response transfer path when the probe establishes that Range is ignored, using the complete expected object size rather than the 8 MiB ranged-request span. **Trace:** [Audit DC-02](docs/history/localmotive-comprehensive-audit.md#dc-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L474).

<a id="criterion-v06-dc-02-i2"></a>
- [x] **V06-DC-02.I2** — Retain a bounded streaming buffer and read or idle deadline for the sequential path; validate the full expected length and mandatory final SHA-256 before publication. **Trace:** [Audit DC-02](docs/history/localmotive-comprehensive-audit.md#dc-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L475).

<a id="criterion-v06-dc-02-i3"></a>
- [x] **V06-DC-02.I3** — Restart non-range transfers from zero after interruption or retry, and ensure misleading Accept-Ranges headers cannot cause unsafe reuse of a partially downloaded object. **Trace:** [Audit DC-02](docs/history/localmotive-comprehensive-audit.md#dc-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L476).

<a id="criterion-v06-dc-02-i4"></a>
- [x] **V06-DC-02.I4** — Keep exact Content-Range, remote-validator, response-length, and overrun checks on the existing ranged path; do not weaken parallel integrity checks to accommodate HTTP 200. **Trace:** [Audit DC-02](docs/history/localmotive-comprehensive-audit.md#dc-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L477).

<a id="criterion-v06-dc-02-v1"></a>
- [x] **V06-DC-02.V1** — Extend the existing 16 KiB no-range regression with 8 MiB+1 and a realistic larger fixture; test HTTP 200 with both Content-Length and chunked response bodies. **Trace:** [Audit DC-02](docs/history/localmotive-comprehensive-audit.md#dc-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L481).

<a id="criterion-v06-dc-02-v2"></a>
- [x] **V06-DC-02.V2** — Test cancellation, stalled reads, truncated bodies, retry from zero, misleading Accept-Ranges, excess bytes, and incorrect digests against a local server. **Trace:** [Audit DC-02](docs/history/localmotive-comprehensive-audit.md#dc-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L482).

<a id="criterion-v06-dc-02-v3"></a>
- [x] **V06-DC-02.V3** — Re-run parallel range, resume-identity, and checksum regressions, and verify a runtime-artifact caller remains compatible because runtime and model transfers share this downloader. **Trace:** [Audit DC-02](docs/history/localmotive-comprehensive-audit.md#dc-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L483).

### V06-DC-03

<a id="criterion-v06-dc-03-i1"></a>
- [x] **V06-DC-03.I1** — Replace the healthy-migration rebuild branch with a transactional call to mirror_verified_catalog on the valid connection, preserving existing user records during ordinary refresh. **Trace:** [Audit DC-03](docs/history/localmotive-comprehensive-audit.md#dc-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L500).

<a id="criterion-v06-dc-03-i2"></a>
- [x] **V06-DC-03.I2** — Route confirmed migration or corruption failures into controlled recovery, closing relevant connections before quarantine or replacement instead of depending on removal of an open SQLite file. **Trace:** [Audit DC-03](docs/history/localmotive-comprehensive-audit.md#dc-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L501).

<a id="criterion-v06-dc-03-i3"></a>
- [x] **V06-DC-03.I3** — Preserve or export readable user overrides before rebuilding; define a non-destructive downgrade policy for unknown-newer schemas and report any irrecoverable loss rather than deleting silently. **Trace:** [Audit DC-03](docs/history/localmotive-comprehensive-audit.md#dc-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L502).

<a id="criterion-v06-dc-03-i4"></a>
- [x] **V06-DC-03.I4** — Propagate database-open, migration, mirror, and publication failures into an explicit persistence notice while continuing to serve an available verified in-memory catalog. **Trace:** [Audit DC-03](docs/history/localmotive-comprehensive-audit.md#dc-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L503).

<a id="criterion-v06-dc-03-v1"></a>
- [x] **V06-DC-03.V1** — Create a real temporary database with curated and user records; refresh repeatedly and assert both provenance and user contents survive. **Trace:** [Audit DC-03](docs/history/localmotive-comprehensive-audit.md#dc-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L507).

<a id="criterion-v06-dc-03-v2"></a>
- [x] **V06-DC-03.V2** — Exercise corrupt databases, unknown-newer schemas, unavailable database paths, held locks, and injected write failures; verify recovery policy and preservation of the prior readable state. **Trace:** [Audit DC-03](docs/history/localmotive-comprehensive-audit.md#dc-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L508).

<a id="criterion-v06-dc-03-v3"></a>
- [x] **V06-DC-03.V3** — Run the open-handle and recovery cases in the packaged Windows application, recording actual NTFS/SQLite sharing outcomes and confirming that every failed persistence action is visible. **Trace:** [Audit DC-03](docs/history/localmotive-comprehensive-audit.md#dc-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L509).

### V06-DC-04

<a id="criterion-v06-dc-04-i1"></a>
- [x] **V06-DC-04.I1** — Define the supported local-override trust contract explicitly: signed curated entries remain authorized by their signed snapshot, while supported override downloads require a separately validated, explicitly user-approved record and exact digest. **Trace:** [Audit DC-04](docs/history/localmotive-comprehensive-audit.md#dc-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L526).

<a id="criterion-v06-dc-04-i2"></a>
- [x] **V06-DC-04.I2** — Implement the chosen override download route without promoting arbitrary mutable SQLite rows into curator-signed authority; ensure save, reload, and removal update the appropriate authorization source. **Trace:** [Audit DC-04](docs/history/localmotive-comprehensive-audit.md#dc-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L527).

<a id="criterion-v06-dc-04-i3"></a>
- [x] **V06-DC-04.I3** — Use one merged browse collection for initial rows, filtering, sorting, and facets so the filter effect no longer replaces local entries with only snapshot.catalog.models. **Trace:** [Audit DC-04](docs/history/localmotive-comprehensive-audit.md#dc-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L528).

<a id="criterion-v06-dc-04-i4"></a>
- [x] **V06-DC-04.I4** — Keep provenance visible throughout the workflow and document the product decision if the incomplete add/edit/download functionality is deferred instead of enabled end to end. **Trace:** [Audit DC-04](docs/history/localmotive-comprehensive-audit.md#dc-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L529).

<a id="criterion-v06-dc-04-v1"></a>
- [x] **V06-DC-04.V1** — Save a unique override, reload the app, change every relevant filter/sort, and verify rows and facet values continue to refer to the same merged collection. **Trace:** [Audit DC-04](docs/history/localmotive-comprehensive-audit.md#dc-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L533).

<a id="criterion-v06-dc-04-v2"></a>
- [x] **V06-DC-04.V2** — Against a controlled server, exercise an approved override with correct and incorrect SHA-256, then remove it and verify subsequent download authorization fails. **Trace:** [Audit DC-04](docs/history/localmotive-comprehensive-audit.md#dc-04). **Closed (2026-09-12):** supported-command-path run 13/13 against controlled loopback bytes (correct digest publishes, wrong digest refuses and publishes nothing, removal revokes with no network touch); evidence `release-evidence/0.6.0/attestations/dc04-v2-command-path-verification.log`; record below.
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L534).

<a id="criterion-v06-dc-04-v3"></a>
- [x] **V06-DC-04.V3** — Attempt downloads from unknown or directly modified database rows and verify they never acquire signed curated status; retain tests for ordinary curated downloads. **Trace:** [Audit DC-04](docs/history/localmotive-comprehensive-audit.md#dc-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L535).

### V06-DC-05

<a id="criterion-v06-dc-05-i1"></a>
- [x] **V06-DC-05.I1** — Reserve curator-owned model identifiers and choose either a separate user namespace or explicit rejection of mixed-origin ID collisions before any UPSERT can mutate ownership. **Trace:** [Audit DC-05](docs/history/localmotive-comprehensive-audit.md#dc-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L552).

<a id="criterion-v06-dc-05-i2"></a>
- [x] **V06-DC-05.I2** — Define and enforce a deterministic case-insensitive filename collision policy that preserves existing files and requires explicit resolution rather than silently moving a catalog_file row to another model. **Trace:** [Audit DC-05](docs/history/localmotive-comprehensive-audit.md#dc-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L553).

<a id="criterion-v06-dc-05-i3"></a>
- [x] **V06-DC-05.I3** — Carry provenance at the file/entity level used for display and authorization; update database reads so a user file cannot inherit a curator label merely because its model row was refreshed. **Trace:** [Audit DC-05](docs/history/localmotive-comprehensive-audit.md#dc-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L554).

<a id="criterion-v06-dc-05-i4"></a>
- [x] **V06-DC-05.I4** — Change curated mirror refresh to preserve origin boundaries, and enable or strengthen relational constraints where they enforce the chosen ownership rules. **Trace:** [Audit DC-05](docs/history/localmotive-comprehensive-audit.md#dc-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L555).

<a id="criterion-v06-dc-05-v1"></a>
- [x] **V06-DC-05.V1** — Turn the executed SQL reproductions into application-level regressions for same ID/different repo and different IDs/same case-insensitive filename; require the original curated records to remain unchanged. **Trace:** [Audit DC-05](docs/history/localmotive-comprehensive-audit.md#dc-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L559).

<a id="criterion-v06-dc-05-v2"></a>
- [x] **V06-DC-05.V2** — Exercise refresh, edit, and removal after each collision attempt; verify no empty curated model or stale user file becomes relabeled as curator-sourced. **Trace:** [Audit DC-05](docs/history/localmotive-comprehensive-audit.md#dc-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L560).

<a id="criterion-v06-dc-05-v3"></a>
- [x] **V06-DC-05.V3** — Test disjoint user and curator entries alongside the rejection cases so normal additions and refresh preservation remain supported. **Trace:** [Audit DC-05](docs/history/localmotive-comprehensive-audit.md#dc-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L561).

### V06-DC-06

<a id="criterion-v06-dc-06-i1"></a>
- [x] **V06-DC-06.I1** — Wrap replacement of one user's model and complete file set in a transaction, removing files omitted by the new version while preserving the previous version if any step fails. **Trace:** [Audit DC-06](docs/history/localmotive-comprehensive-audit.md#dc-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L578).

<a id="criterion-v06-dc-06-i2"></a>
- [x] **V06-DC-06.I2** — Make override removal transactional so deleting file rows and the model row either succeeds as one operation or leaves the previous state intact. **Trace:** [Audit DC-06](docs/history/localmotive-comprehensive-audit.md#dc-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L579).

<a id="criterion-v06-dc-06-i3"></a>
- [x] **V06-DC-06.I3** — Apply shared full validation to user writes and parsed records: bound serialized bytes, tags and nested arrays, text/date/quant/revision lengths, file counts, and case-insensitive duplicate targets. **Trace:** [Audit DC-06](docs/history/localmotive-comprehensive-audit.md#dc-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L580).

<a id="criterion-v06-dc-06-i4"></a>
- [x] **V06-DC-06.I4** — Enforce the 200-user limit with origin-aware accounting under concurrent writes, and replace unchecked u64-to-i64 and reverse casts with validated, checked conversions. **Trace:** [Audit DC-06](docs/history/localmotive-comprehensive-audit.md#dc-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L581).

<a id="criterion-v06-dc-06-v1"></a>
- [x] **V06-DC-06.V1** — Add regressions for changing two files to one and a.gguf to b.gguf, then inject failure on the second insert and between removal statements; assert exact replacement or full rollback. **Trace:** [Audit DC-06](docs/history/localmotive-comprehensive-audit.md#dc-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L585).

<a id="criterion-v06-dc-06-v2"></a>
- [x] **V06-DC-06.V2** — Exercise concurrent saves near the user cap and attempted conversion of a curated ID, coordinating the ownership rules from DC-05. **Trace:** [Audit DC-06](docs/history/localmotive-comprehensive-audit.md#dc-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L586).

<a id="criterion-v06-dc-06-v3"></a>
- [x] **V06-DC-06.V3** — Test excessive tags, long revisions/quants/dates, duplicate filenames, serialized payload limits, and integer values at i64::MAX and beyond; require actionable rejections before mutation. **Trace:** [Audit DC-06](docs/history/localmotive-comprehensive-audit.md#dc-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L587).

### V06-DC-07

<a id="criterion-v06-dc-07-i1"></a>
- [x] **V06-DC-07.I1** — Route candidate body-read, streamed size-limit, UTF-8, signature-body, signature-validation, and catalog-parse failures through one consistent fallback path instead of early propagation that suppresses usable data. **Trace:** [Audit DC-07](docs/history/localmotive-comprehensive-audit.md#dc-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L604).

<a id="criterion-v06-dc-07-i2"></a>
- [x] **V06-DC-07.I2** — Return the last supported signature-verified cache or bundled snapshot with explicit refresh-error status; preserve the distinction between a successful local load and a successful network refresh. **Trace:** [Audit DC-07](docs/history/localmotive-comprehensive-audit.md#dc-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L605).

<a id="criterion-v06-dc-07-i3"></a>
- [x] **V06-DC-07.I3** — Keep unsupported future-schema and otherwise invalid candidates from replacing the last valid cache or becoming the authoritative curated download state. **Trace:** [Audit DC-07](docs/history/localmotive-comprehensive-audit.md#dc-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L606).

<a id="criterion-v06-dc-07-i4"></a>
- [x] **V06-DC-07.I4** — Handle catalog database-open failure consistently with migration/read failure by selecting verified memory or bundled browsing data rather than depending solely on a frontend catch. **Trace:** [Audit DC-07](docs/history/localmotive-comprehensive-audit.md#dc-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L607).

<a id="criterion-v06-dc-07-v1"></a>
- [x] **V06-DC-07.V1** — Seed a valid signed cache and test truncated streaming responses, missing Content-Length with limit+1 data, invalid UTF-8, malformed signature bodies, and network/send failures. **Trace:** [Audit DC-07](docs/history/localmotive-comprehensive-audit.md#dc-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L611).

<a id="criterion-v06-dc-07-v2"></a>
- [x] **V06-DC-07.V2** — Serve a validly signed unsupported schema and confirm the previous supported catalog remains visible and unchanged on disk. **Trace:** [Audit DC-07](docs/history/localmotive-comprehensive-audit.md#dc-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L612).

<a id="criterion-v06-dc-07-v3"></a>
- [x] **V06-DC-07.V3** — Repeat fallback cases without a usable cache and with a database-open failure; verify a bundled snapshot, clear refresh status, and no unsigned authorization. **Trace:** [Audit DC-07](docs/history/localmotive-comprehensive-audit.md#dc-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L613).

### V06-DC-08

<a id="criterion-v06-dc-08-i1"></a>
- [x] **V06-DC-08.I1** — Introduce explicit limits for KV count, string/key length, aggregate retained bytes/elements, and total parser work across nested values; enforce budgets before allocation rather than relying only on the 256 MiB input limit. **Trace:** [Audit DC-08](docs/history/localmotive-comprehensive-audit.md#dc-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L630).

<a id="criterion-v06-dc-08-i2"></a>
- [x] **V06-DC-08.I2** — Skip irrelevant fixed-size arrays using checked byte counts, stream-discard unneeded strings, and avoid allocating captured-value vectors when capture is false; retain clear truncation metadata for supported summaries. **Trace:** [Audit DC-08](docs/history/localmotive-comprehensive-audit.md#dc-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L631).

<a id="criterion-v06-dc-08-i3"></a>
- [x] **V06-DC-08.I3** — Use buffered file reads and stack arrays for fixed-width primitives, validate remaining file bytes early, and avoid unnecessary cloning of retained metadata. **Trace:** [Audit DC-08](docs/history/localmotive-comprehensive-audit.md#dc-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L632).

<a id="criterion-v06-dc-08-i4"></a>
- [x] **V06-DC-08.I4** — Move synchronous GGUF parsing into cancellable background work and apply a defined budget across artifact inspection so multiple shards cannot create unbounded repeated work. **Trace:** [Audit DC-08](docs/history/localmotive-comprehensive-audit.md#dc-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L633).

<a id="criterion-v06-dc-08-v1"></a>
- [x] **V06-DC-08.V1** — Add deterministic resource regressions for short files declaring giant strings, many tiny KVs, wide nested arrays, noncaptured tokenizer arrays, oversized keys, and counts just beyond each limit. **Trace:** [Audit DC-08](docs/history/localmotive-comprehensive-audit.md#dc-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L637).

<a id="criterion-v06-dc-08-v2"></a>
- [x] **V06-DC-08.V2** — Test files shortened during parsing and cancellation during large summaries or multi-shard inspection; assert prompt errors and preserved UI responsiveness. **Trace:** [Audit DC-08](docs/history/localmotive-comprehensive-audit.md#dc-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L638).

<a id="criterion-v06-dc-08-v3"></a>
- [x] **V06-DC-08.V3** — Fuzz malformed/truncated headers with explicit allocation/work budgets, and measure representative valid headers to ensure useful metadata and header-only behavior remain intact. **Trace:** [Audit DC-08](docs/history/localmotive-comprehensive-audit.md#dc-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L639).

### V06-DC-09

<a id="criterion-v06-dc-09-i1"></a>
- [x] **V06-DC-09.I1** — Replace the arbitrary final-filename-suffix extraction with parsing of actual recognized quant tokens in the full basename, preserving provenance or variant suffixes separately and returning unknown when evidence is ambiguous. **Trace:** [Audit DC-09](docs/history/localmotive-comprehensive-audit.md#dc-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L656).

<a id="criterion-v06-dc-09-i2"></a>
- [x] **V06-DC-09.I2** — Prefer verified structured upstream quant metadata when available, normalize canonical casing, and make facet deduplication agree with filter comparison semantics. **Trace:** [Audit DC-09](docs/history/localmotive-comprehensive-audit.md#dc-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L657).

<a id="criterion-v06-dc-09-i3"></a>
- [x] **V06-DC-09.I3** — Extend curation validation and shared fixtures to reject unsupported provenance/date/instruction labels masquerading as quants; rebuild corrected candidate metadata through the existing signature-validation process. **Trace:** [Audit DC-09](docs/history/localmotive-comprehensive-audit.md#dc-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L658).

<a id="criterion-v06-dc-09-i4"></a>
- [x] **V06-DC-09.I4** — Review MTP-related candidates using upstream or header evidence to distinguish integrated-MTP main models from companion-only files before changing exclusion rules. **Trace:** [Audit DC-09](docs/history/localmotive-comprehensive-audit.md#dc-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L659).

<a id="criterion-v06-dc-09-v1"></a>
- [x] **V06-DC-09.V1** — Add fixtures covering IQ2_S-MTP, Q4_K_M-imatrix, dates, instruction-tuning suffixes, lowercase labels, combined base/draft quants, unknown formats, and quant-looking model names. **Trace:** [Audit DC-09](docs/history/localmotive-comprehensive-audit.md#dc-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L663).

<a id="criterion-v06-dc-09-v2"></a>
- [x] **V06-DC-09.V2** — Use the audit's 46 suffix-label rows as a correction checklist, preserving unknown where underlying quant evidence is insufficient rather than guessing. **Trace:** [Audit DC-09](docs/history/localmotive-comprehensive-audit.md#dc-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L664).

<a id="criterion-v06-dc-09-v3"></a>
- [x] **V06-DC-09.V3** — Verify one semantic facet per normalized quant and confirm selecting a quant finds corrected builds; run catalog validation on the resulting signed candidate. **Trace:** [Audit DC-09](docs/history/localmotive-comprehensive-audit.md#dc-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L665).

### V06-DC-10

<a id="criterion-v06-dc-10-i1"></a>
- [x] **V06-DC-10.I1** — Capture an immutable upstream commit identity for each offered file during catalog construction and serialize it as the file revision alongside the exact verified SHA-256 and size. **Trace:** [Audit DC-10](docs/history/localmotive-comprehensive-audit.md#dc-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L682).

<a id="criterion-v06-dc-10-i2"></a>
- [x] **V06-DC-10.I2** — Preserve immutable revisions through catalog parsing, cached/bundled representation, and download authorization so resolution does not silently return to mutable main. **Trace:** [Audit DC-10](docs/history/localmotive-comprehensive-audit.md#dc-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L683).

<a id="criterion-v06-dc-10-i3"></a>
- [x] **V06-DC-10.I3** — Return a clear recoverable error when an approved historical object is unavailable, retaining strict length/digest checks instead of substituting current upstream bytes. **Trace:** [Audit DC-10](docs/history/localmotive-comprehensive-audit.md#dc-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L684).

<a id="criterion-v06-dc-10-i4"></a>
- [x] **V06-DC-10.I4** — Document whether rollback of a previously signed manifest belongs in the freshness threat model; if it does, define and implement signed sequence/expiry acceptance and recovery behavior. **Trace:** [Audit DC-10](docs/history/localmotive-comprehensive-audit.md#dc-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L685).

<a id="criterion-v06-dc-10-v1"></a>
- [x] **V06-DC-10.V1** — Add immutable revision serialization and authorization round-trip fixtures, including cache/bundle loading and an entry whose filename remains unchanged across revisions. **Trace:** [Audit DC-10](docs/history/localmotive-comprehensive-audit.md#dc-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L689).

<a id="criterion-v06-dc-10-v2"></a>
- [x] **V06-DC-10.V2** — Use controlled upstream responses to replace main while keeping the approved revision available; require the pinned request to resolve the approved bytes and reject unintended substitution. **Trace:** [Audit DC-10](docs/history/localmotive-comprehensive-audit.md#dc-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L690).

<a id="criterion-v06-dc-10-v3"></a>
- [x] **V06-DC-10.V3** — Test unavailable historical objects and enforce the documented old-signed-manifest replay policy, including any required expiry or rollback recovery scenario. **Trace:** [Audit DC-10](docs/history/localmotive-comprehensive-audit.md#dc-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L691).

### V06-DC-11

<a id="criterion-v06-dc-11-i1"></a>
- [x] **V06-DC-11.I1** — Add a per-target cross-process reservation or lock so separate app instances cannot share and mutate the same partial transfer merely because the in-process registry is clear. **Trace:** [Audit DC-11](docs/history/localmotive-comprehensive-audit.md#dc-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L708).

<a id="criterion-v06-dc-11-i2"></a>
- [x] **V06-DC-11.I2** — Use a unique private partial filename and preserve stable file identity from writing through hash verification and final publication where platform APIs permit. **Trace:** [Audit DC-11](docs/history/localmotive-comprehensive-audit.md#dc-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L709).

<a id="criterion-v06-dc-11-i3"></a>
- [x] **V06-DC-11.I3** — Publish with no-replace semantics, checking that the entry being committed is the verified object; an ordinary target appearing during transfer must produce a conflict instead of being overwritten. **Trace:** [Audit DC-11](docs/history/localmotive-comprehensive-audit.md#dc-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L710).

<a id="criterion-v06-dc-11-i4"></a>
- [x] **V06-DC-11.I4** — Preserve both the verified artifact and the conflicting existing file on collision, report actionable recovery information, and retain directory-capability, reparse-point, and hard-link protections. **Trace:** [Audit DC-11](docs/history/localmotive-comprehensive-audit.md#dc-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L711).

<a id="criterion-v06-dc-11-v1"></a>
- [x] **V06-DC-11.V1** — Run two app instances targeting the same filename and introduce an unrelated final file during transfer; assert neither pre-existing file is modified. **Trace:** [Audit DC-11](docs/history/localmotive-comprehensive-audit.md#dc-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L715).

<a id="criterion-v06-dc-11-v2"></a>
- [x] **V06-DC-11.V2** — Inject partial-entry replacement immediately after hashing and directory/root renaming during transfer; require safe failure or verified stable-object publication. **Trace:** [Audit DC-11](docs/history/localmotive-comprehensive-audit.md#dc-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L716).

<a id="criterion-v06-dc-11-v3"></a>
- [x] **V06-DC-11.V3** — Exercise the cases on NTFS in the packaged application, record actual handle/rename behavior, and re-run existing directory-capability and link-safety regressions. **Trace:** [Audit DC-11](docs/history/localmotive-comprehensive-audit.md#dc-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L717).

### V06-DC-12

<a id="criterion-v06-dc-12-i1"></a>
- [x] **V06-DC-12.I1** — Define the promised crash and power-loss recovery guarantees, then order partial-data durability before publishing resume checkpoints that claim those bytes are complete. **Trace:** [Audit DC-12](docs/history/localmotive-comprehensive-audit.md#dc-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L734).

<a id="criterion-v06-dc-12-i2"></a>
- [x] **V06-DC-12.I2** — Choose documented byte/time checkpoint intervals that maintain the guarantee without syncing tiny sidecars unnecessarily; preserve the previous valid checkpoint when data or metadata persistence fails. **Trace:** [Audit DC-12](docs/history/localmotive-comprehensive-audit.md#dc-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L735).

<a id="criterion-v06-dc-12-i3"></a>
- [x] **V06-DC-12.I3** — Add cancellation checks and progress reporting to both existing-file and completed-part SHA-256 verification, preserving recoverable state when Keep & stop is requested during hashing. **Trace:** [Audit DC-12](docs/history/localmotive-comprehensive-audit.md#dc-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L736).

<a id="criterion-v06-dc-12-i4"></a>
- [x] **V06-DC-12.I4** — Align frontend verification controls with backend cancellation semantics and measure the shared write-mutex/seek path before considering positional-write optimization. **Trace:** [Audit DC-12](docs/history/localmotive-comprehensive-audit.md#dc-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L737).

<a id="criterion-v06-dc-12-v1"></a>
- [x] **V06-DC-12.V1** — Inject data-write, synchronization, checkpoint-publication, and recovery failures; assert that resumed progress never knowingly claims bytes outside the selected durability guarantee. **Trace:** [Audit DC-12](docs/history/localmotive-comprehensive-audit.md#dc-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L741).

<a id="criterion-v06-dc-12-v2"></a>
- [x] **V06-DC-12.V2** — Cancel large existing-file and final-part verification, require prompt control return, and verify that retry safely resumes or re-verifies without incorrectly reporting completion. **Trace:** [Audit DC-12](docs/history/localmotive-comprehensive-audit.md#dc-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L742).

<a id="criterion-v06-dc-12-v3"></a>
- [ ] **V06-DC-12.V3** — Distinguish ordinary process-crash testing from target-environment OS-crash/power-loss validation; record throughput for 1/4/8 connections on available HDD, SATA SSD, and NVMe targets before optimizing. **Trace:** [Audit DC-12](docs/history/localmotive-comprehensive-audit.md#dc-12).
  **Current status: DEFERRED-OWNER.** D06-02: owner-deferred storage-class and OS-crash/power-loss facility. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L743).

### V06-MT-01

<a id="criterion-v06-mt-01-i1"></a>
- [x] **V06-MT-01.I1** — Define whether the warm workload measures a resident model with full prefill or actual KV-prompt reuse; represent that choice explicitly in the immutable workload and persisted protocol. **Trace:** [Audit MT-01](docs/history/localmotive-comprehensive-audit.md#mt-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L762).

<a id="criterion-v06-mt-01-i2"></a>
- [x] **V06-MT-01.I2** — Honor approved llama.cpp b10816 semantics: timings.prompt_n is newly processed prompt tokens, cache_n is cached prompt tokens, and total context includes prompt_n + cache_n + predicted_n. **Trace:** [Audit MT-01](docs/history/localmotive-comprehensive-audit.md#mt-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L763).

<a id="criterion-v06-mt-01-i3"></a>
- [x] **V06-MT-01.I3** — For prompt reuse, persist requested, processed, and cached counts separately and validate the relevant total; for full-prefill measurement, request cache_prompt=false while retaining process warmup. **Trace:** [Audit MT-01](docs/history/localmotive-comprehensive-audit.md#mt-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L764).

<a id="criterion-v06-mt-01-i4"></a>
- [x] **V06-MT-01.I4** — Keep exact generation-length validation and truthful prefill metrics, and version or migrate persisted contracts affected by the new count semantics rather than silently removing checks. **Trace:** [Audit MT-01](docs/history/localmotive-comprehensive-audit.md#mt-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L765).

<a id="criterion-v06-mt-01-v1"></a>
- [x] **V06-MT-01.V1** — Add a protocol regression returning prompt_n=512/cache_n=0 followed by prompt_n=1/cache_n=511, both generating 256 tokens; assert valid reuse or deliberate cache-off requests. **Trace:** [Audit MT-01](docs/history/localmotive-comprehensive-audit.md#mt-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L769).

<a id="criterion-v06-mt-01-v2"></a>
- [x] **V06-MT-01.V2** — Exercise no-warmup, partial-cache, inconsistent-total, and wrong-generation-count cases so cached successes are accepted without accepting genuinely incorrect workloads. **Trace:** [Audit MT-01](docs/history/localmotive-comprehensive-audit.md#mt-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L770).

<a id="criterion-v06-mt-01-v3"></a>
- [x] **V06-MT-01.V3** — Run one warmup and five default trials against packaged approved b10816 on Windows; record response counts, request cache policy, and the persisted manifest. **Trace:** [Audit MT-01](docs/history/localmotive-comprehensive-audit.md#mt-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L771).

### V06-MT-02

<a id="criterion-v06-mt-02-i1"></a>
- [x] **V06-MT-02.I1** — Replace the cloned internal benchmark summary in the public schema with a dedicated public summary containing numeric statistics, counts, and bounded failure categories instead of summary.failures raw strings. **Trace:** [Audit MT-02](docs/history/localmotive-comprehensive-audit.md#mt-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L788).

<a id="criterion-v06-mt-02-i2"></a>
- [x] **V06-MT-02.I2** — Construct public hardware, effective-context, and process-memory evidence through an allowlist; define whether source.detail and notes are omitted or transformed into approved public source identifiers. **Trace:** [Audit MT-02](docs/history/localmotive-comprehensive-audit.md#mt-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L789).

<a id="criterion-v06-mt-02-i3"></a>
- [x] **V06-MT-02.I3** — Make the privacy-review omissions describe the actual serialized policy, and validate public exports at both normal construction and direct persistence boundaries. **Trace:** [Audit MT-02](docs/history/localmotive-comprehensive-audit.md#mt-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L790).

<a id="criterion-v06-mt-02-i4"></a>
- [x] **V06-MT-02.I4** — Preserve complete diagnostics in authorized local evidence while retaining explicit user confirmation and create-new share-file publication. **Trace:** [Audit MT-02](docs/history/localmotive-comprehensive-audit.md#mt-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L791).

<a id="criterion-v06-mt-02-v1"></a>
- [x] **V06-MT-02.V1** — Extend the privacy regression with one successful and one failed observation and a correctly recomputed Some(summary), using distinct path, account-name, and secret canaries in the raw failure. **Trace:** [Audit MT-02](docs/history/localmotive-comprehensive-audit.md#mt-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L795).

<a id="criterion-v06-mt-02-v2"></a>
- [x] **V06-MT-02.V2** — Place separate canaries in nested evidence notes/details and inspect the entire serialized public file, not only observation-level fields. **Trace:** [Audit MT-02](docs/history/localmotive-comprehensive-audit.md#mt-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L796).

<a id="criterion-v06-mt-02-v3"></a>
- [x] **V06-MT-02.V3** — Verify useful counts and numeric statistics survive redaction and that existing targets are not overwritten without changing the confirmation requirement. **Trace:** [Audit MT-02](docs/history/localmotive-comprehensive-audit.md#mt-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L797).

### V06-MT-03

<a id="criterion-v06-mt-03-i1"></a>
- [x] **V06-MT-03.I1** — Introduce independent total advisor-call, elapsed-time, and measured-trial budgets so successful parsing and skipped measurements cannot leave a session unbounded. **Trace:** [Audit MT-03](docs/history/localmotive-comprehensive-audit.md#mt-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L814).

<a id="criterion-v06-mt-03-i2"></a>
- [x] **V06-MT-03.I2** — Canonicalize proposals after coercion and companion resolution, comparing effective configurations or normalized changes instead of raw JSON maps. **Trace:** [Audit MT-03](docs/history/localmotive-comprehensive-audit.md#mt-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L815).

<a id="criterion-v06-mt-03-i3"></a>
- [x] **V06-MT-03.I3** — Count no-op and duplicate proposals as bounded rejections, preserve their reasons in the session history, and return a specific terminal stopped reason when limits are reached. **Trace:** [Audit MT-03](docs/history/localmotive-comprehensive-audit.md#mt-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L816).

<a id="criterion-v06-mt-03-i4"></a>
- [x] **V06-MT-03.I4** — Decide whether the advertised three-fields-per-trial rule is a hard policy; if retained, enforce it in Rust and align advisor instructions with backend limits. **Trace:** [Audit MT-03](docs/history/localmotive-comprehensive-audit.md#mt-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L817).

<a id="criterion-v06-mt-03-v1"></a>
- [x] **V06-MT-03.V1** — Use a deterministic advisor repeatedly proposing baseline threads=-1 and a benchmark succeeding only for baseline; assert termination within the configured advisor-call limit. **Trace:** [Audit MT-03](docs/history/localmotive-comprehensive-audit.md#mt-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L821).

<a id="criterion-v06-mt-03-v2"></a>
- [x] **V06-MT-03.V2** — Repeat with numeric-string coercion, draftModel echoes, alternating no-op maps, and fields that leave emitted arguments unchanged. **Trace:** [Audit MT-03](docs/history/localmotive-comprehensive-audit.md#mt-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L822).

<a id="criterion-v06-mt-03-v3"></a>
- [x] **V06-MT-03.V3** — Test exact call/time/trial budget boundaries and verify cancellation during a no-op sequence prevents another cloud request when combined with the lifecycle cancellation fix. **Trace:** [Audit MT-03](docs/history/localmotive-comprehensive-audit.md#mt-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L823).

### V06-MT-04

<a id="criterion-v06-mt-04-i1"></a>
- [x] **V06-MT-04.I1** — Make cancellation a terminal orchestrator outcome with checks before and after advisor calls and before every attempt; stop proposing instead of recording cancellation as an ordinary candidate failure. **Trace:** [Audit MT-04](docs/history/localmotive-comprehensive-audit.md#mt-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L840).

<a id="criterion-v06-mt-04-i2"></a>
- [x] **V06-MT-04.I2** — Pass the cancellation signal into preparation, cancellable health startup, and completion requests, replacing the live tuner's uncancellable legacy benchmark path. **Trace:** [Audit MT-04](docs/history/localmotive-comprehensive-audit.md#mt-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L841).

<a id="criterion-v06-mt-04-i3"></a>
- [x] **V06-MT-04.I3** — Apply a documented overall deadline and bounded stop latency rather than relying on a 600-second health wait or separate per-read timeouts. **Trace:** [Audit MT-04](docs/history/localmotive-comprehensive-audit.md#mt-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L842).

<a id="criterion-v06-mt-04-i4"></a>
- [x] **V06-MT-04.I4** — Check process-tree termination results, surface cleanup failures, and release active tuning state only after lifecycle ownership has been resolved. **Trace:** [Audit MT-04](docs/history/localmotive-comprehensive-audit.md#mt-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L843).

<a id="criterion-v06-mt-04-v1"></a>
- [x] **V06-MT-04.V1** — Cancel during preparation, health wait, response wait, between repetitions, between advisor calls, and while the advisor emits only no-ops; assert the final cancellation reason and no subsequent advisor call. **Trace:** [Audit MT-04](docs/history/localmotive-comprehensive-audit.md#mt-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L847).

<a id="criterion-v06-mt-04-v2"></a>
- [x] **V06-MT-04.V2** — Inject process cleanup failure and confirm it is preserved rather than silently returning a successful tuning result. **Trace:** [Audit MT-04](docs/history/localmotive-comprehensive-audit.md#mt-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L848).

<a id="criterion-v06-mt-04-v3"></a>
- [x] **V06-MT-04.V3** — Run packaged Windows cancellation scenarios and record stop latency, surviving child processes, listener ownership, port release, and ability to start the next operation. **Trace:** [Audit MT-04](docs/history/localmotive-comprehensive-audit.md#mt-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L849).

### V06-MT-05

<a id="criterion-v06-mt-05-i1"></a>
- [x] **V06-MT-05.I1** — Introduce a single Rust operation coordinator with atomic reservations for ordinary startup, warm benchmarks, cold attempts, tuning, and quality suites. **Trace:** [Audit MT-05](docs/history/localmotive-comprehensive-audit.md#mt-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L866).

<a id="criterion-v06-mt-05-i2"></a>
- [x] **V06-MT-05.I2** — Retain a process-generation lease for each operation, including privately launched cold runtimes, and make Start reject or wait while another owner remains active. **Trace:** [Audit MT-05](docs/history/localmotive-comprehensive-audit.md#mt-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L867).

<a id="criterion-v06-mt-05-i3"></a>
- [x] **V06-MT-05.I3** — Make Stop cancel the actual operation owner and verify the expected process/listener generation for each request; reject attribution if replacement or ownership loss occurs. **Trace:** [Audit MT-05](docs/history/localmotive-comprehensive-audit.md#mt-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L868).

<a id="criterion-v06-mt-05-i4"></a>
- [x] **V06-MT-05.I4** — Capture model/runtime identity before quality requests and associate process-memory samples with the serving process; keep long I/O outside the existing server mutex. **Trace:** [Audit MT-05](docs/history/localmotive-comprehensive-audit.md#mt-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L869).

<a id="criterion-v06-mt-05-v1"></a>
- [x] **V06-MT-05.V1** — Use deterministic barriers to reproduce warm-benchmark/stop/restart, cold-benchmark/start/tune, quality/restart, and simultaneous start_server/start_tuning interleavings. **Trace:** [Audit MT-05](docs/history/localmotive-comprehensive-audit.md#mt-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L873).

<a id="criterion-v06-mt-05-v2"></a>
- [x] **V06-MT-05.V2** — Assert no second owner is granted, no old-PID memory evidence is joined to replacement responses, and replaced-server results cannot be finalized under the original identity. **Trace:** [Audit MT-05](docs/history/localmotive-comprehensive-audit.md#mt-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L874).

<a id="criterion-v06-mt-05-v3"></a>
- [x] **V06-MT-05.V3** — Exercise packaged Windows cancellation and restart flows, preserving records of process generations, listener ownership, and eventual reservation release. **Trace:** [Audit MT-05](docs/history/localmotive-comprehensive-audit.md#mt-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L875).

### V06-MT-06

<a id="criterion-v06-mt-06-i1"></a>
- [x] **V06-MT-06.I1** — Centralize a Rust-owned local HTTP client constructed from the validated launch profile, and use it consistently for health, tokenization, benchmark, and quality endpoints. **Trace:** [Audit MT-06](docs/history/localmotive-comprehensive-audit.md#mt-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L892).

<a id="criterion-v06-mt-06-i2"></a>
- [x] **V06-MT-06.I2** — Support configured TLS with an explicit certificate-trust policy and API-key authentication read only by Rust; keep key contents out of frontend state, events, logs, and manifest arguments. **Trace:** [Audit MT-06](docs/history/localmotive-comprehensive-audit.md#mt-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L893).

<a id="criterion-v06-mt-06-i3"></a>
- [x] **V06-MT-06.I3** — Add correct HTTP framing, bounded request/response sizes, and cancellation-aware whole-operation deadlines covering connection, writes, and reads. **Trace:** [Audit MT-06](docs/history/localmotive-comprehensive-audit.md#mt-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L894).

<a id="criterion-v06-mt-06-i4"></a>
- [x] **V06-MT-06.I4** — Until a profile's secured transport is supported, reject that combination before launch with a precise recovery message instead of allowing a ten-minute plaintext health timeout. **Trace:** [Audit MT-06](docs/history/localmotive-comprehensive-audit.md#mt-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L895).

<a id="criterion-v06-mt-06-v1"></a>
- [x] **V06-MT-06.V1** — Test trusted local TLS, rejected invalid certificates, API-key-protected completion, and missing/wrong keys; assert secret canaries never appear in observable diagnostics. **Trace:** [Audit MT-06](docs/history/localmotive-comprehensive-audit.md#mt-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L899).

<a id="criterion-v06-mt-06-v2"></a>
- [x] **V06-MT-06.V2** — Cover bracketed IPv6, chunked JSON, excessive response size, slow writes/reads, and cancellation during connection and response waits. **Trace:** [Audit MT-06](docs/history/localmotive-comprehensive-audit.md#mt-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L900).

<a id="criterion-v06-mt-06-v3"></a>
- [x] **V06-MT-06.V3** — Run accepted TLS/key profiles against the packaged target runtime and record successful health, benchmark, and quality behavior. **Trace:** [Audit MT-06](docs/history/localmotive-comprehensive-audit.md#mt-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L901).

### V06-MT-07

<a id="criterion-v06-mt-07-i1"></a>
- [x] **V06-MT-07.I1** — Replace the manually selected compatibility fields with a versioned canonical execution snapshot including effective performance/quality-relevant arguments and observed per-slot context. **Trace:** [Audit MT-07](docs/history/localmotive-comprehensive-audit.md#mt-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L918).

<a id="criterion-v06-mt-07-i2"></a>
- [x] **V06-MT-07.I2** — Cover thread counts, flash attention, KV/CPU offload, fit parameters, speculation/draft settings, device placement, overrides, extra options, and content identities for LoRA or other influences. **Trace:** [Audit MT-07](docs/history/localmotive-comprehensive-audit.md#mt-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L919).

<a id="criterion-v06-mt-07-i3"></a>
- [x] **V06-MT-07.I3** — Include CPU, RAM, platform, and metric/estimator identity where they affect applicability; define when missing driver or hardware facts make reuse insufficiently supported. **Trace:** [Audit MT-07](docs/history/localmotive-comprehensive-audit.md#mt-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L920).

<a id="criterion-v06-mt-07-i4"></a>
- [x] **V06-MT-07.I4** — Migrate or explicitly invalidate older calibration identities while retaining replay's separate command-argument comparison and preventing secret values from entering exposed identity records. **Trace:** [Audit MT-07](docs/history/localmotive-comprehensive-audit.md#mt-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L921).

<a id="criterion-v06-mt-07-v1"></a>
- [x] **V06-MT-07.V1** — Add table/property-driven checks changing each material field independently, including fields previously absent from CompatibilityIdentity, and assert compatibility changes appropriately. **Trace:** [Audit MT-07](docs/history/localmotive-comprehensive-audit.md#mt-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L925).

<a id="criterion-v06-mt-07-v2"></a>
- [x] **V06-MT-07.V2** — Test CPU-only machines, the same GPU with a changed CPU, fit-reduced effective context, changed draft settings with the same companion, and changed LoRA bytes under the same filename. All five scenarios are covered as canonical-snapshot identity tests in `calibration::tests::mt07_cpu_only_and_hardware_changes_are_distinguished` (CPU-only snapshot without adapters; changed host CPU; `context_requested` 32768 with `context_effective` 16384; same companion SHA with different `--spec-draft-n-max` arguments; same LoRA filename with different content digest) plus the every-field table test `mt07_snapshot_key_changes_for_every_material_field`. Focused run: `cargo test --lib mt07` 4 passed / 0 failed; mutation MUT-MT07 (key derivation ignores `lora_sha256`) FAILED the scenario test, restore returned green. Residual (hardware program, not this criterion): live throughput portability across physical CPU-only/changed-CPU hosts remains in the RT-06/S-25 environment register. **Trace:** [Audit MT-07](docs/history/localmotive-comprehensive-audit.md#mt-07).
  **Current status: VERIFIED.** Accepted implementation/fixture/available-host scope; unprovided physical hardware portions are DEFERRED-OWNER under D06-03/04. No broader hardware observation is claimed. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L926).

<a id="criterion-v06-mt-07-v3"></a>
- [x] **V06-MT-07.V3** — Verify old-schema calibration cannot silently masquerade as a current full identity, unknown identity has the documented outcome, and replay still rejects changed command arguments. **Trace:** [Audit MT-07](docs/history/localmotive-comprehensive-audit.md#mt-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L927).

### V06-MT-08

<a id="criterion-v06-mt-08-i1"></a>
- [x] **V06-MT-08.I1** — Create anchors in Rust from a persisted, validated benchmark run/manifest identity plus an explicit estimator identity and estimate value, instead of accepting a click-stamped copy of a mean. **Trace:** [Audit MT-08](docs/history/localmotive-comprehensive-audit.md#mt-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L944).

<a id="criterion-v06-mt-08-i2"></a>
- [x] **V06-MT-08.I2** — Derive observation time from the source run, enforce source-run uniqueness, and retain that identity across loading, deletion/reimport, and repeated requests. **Trace:** [Audit MT-08](docs/history/localmotive-comprehensive-audit.md#mt-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L945).

<a id="criterion-v06-mt-08-i3"></a>
- [x] **V06-MT-08.I3** — Define eligibility for partial, failed, or cancelled runs and enforce it independently of the frontend's presence-of-summary check. **Trace:** [Audit MT-08](docs/history/localmotive-comprehensive-audit.md#mt-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L946).

<a id="criterion-v06-mt-08-i4"></a>
- [x] **V06-MT-08.I4** — Display the actual eligible unique-run count, prevent repeated Add anchor actions from manufacturing samples, and document the existing interval formula using the modest Estimated interval label. **Trace:** [Audit MT-08](docs/history/localmotive-comprehensive-audit.md#mt-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L947).

<a id="criterion-v06-mt-08-v1"></a>
- [x] **V06-MT-08.V1** — Click Add anchor three times for one benchmark and assert only one eligible anchor exists and the three-run model gate remains closed; repeat with different manually entered estimates. **Trace:** [Audit MT-08](docs/history/localmotive-comprehensive-audit.md#mt-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L951).

<a id="criterion-v06-mt-08-v2"></a>
- [x] **V06-MT-08.V2** — Confirm three independent compatible run identities can build a model, original observation times survive import, and reimporting the same source cannot create another independent sample. **Trace:** [Audit MT-08](docs/history/localmotive-comprehensive-audit.md#mt-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L952).

<a id="criterion-v06-mt-08-v3"></a>
- [x] **V06-MT-08.V3** — Test the explicit failed/cancelled/partial-run policy and verify repeated source samples cannot create a misleading zero-width interval by satisfying the count gate. **Trace:** [Audit MT-08](docs/history/localmotive-comprehensive-audit.md#mt-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L953).

### V06-MT-09

<a id="criterion-v06-mt-09-i1"></a>
- [x] **V06-MT-09.I1** — Capture full model-content and effective launch identities before quality requests, including companion/LoRA influences, runtime identity, suite/version, harness, and observation time. **Trace:** [Audit MT-09](docs/history/localmotive-comprehensive-audit.md#mt-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L970).

<a id="criterion-v06-mt-09-i2"></a>
- [x] **V06-MT-09.I2** — Use the shared operation-generation ownership to ensure all quality cases were served by the same expected process and reject results after identity changes. **Trace:** [Audit MT-09](docs/history/localmotive-comprehensive-audit.md#mt-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L971).

<a id="criterion-v06-mt-09-i3"></a>
- [x] **V06-MT-09.I3** — Enforce compatibility in a Rust join boundary used by benchmark candidates and share exports; do not rely on logical header identity or unchecked frontend qualityPassRate attachment. **Trace:** [Audit MT-09](docs/history/localmotive-comprehensive-audit.md#mt-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L972).

<a id="criterion-v06-mt-09-i4"></a>
- [x] **V06-MT-09.I4** — Retain per-candidate quality history and consistently label the existing READY/JSON suite as two structural smoke cases rather than broad semantic quality. **Trace:** [Audit MT-09](docs/history/localmotive-comprehensive-audit.md#mt-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L973).

<a id="criterion-v06-mt-09-v1"></a>
- [x] **V06-MT-09.V1** — Reject quality attachment after changing KV precision, speculation, LoRA, companions, or model tensor bytes while preserving the filename, header, and file size. **Trace:** [Audit MT-09](docs/history/localmotive-comprehensive-audit.md#mt-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L977).

<a id="criterion-v06-mt-09-v2"></a>
- [x] **V06-MT-09.V2** — Test another model/runtime result at the Rust join boundary and stop/restart between quality cases; assert no stale pass rate is attached to the current candidate. **Trace:** [Audit MT-09](docs/history/localmotive-comprehensive-audit.md#mt-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L978).

<a id="criterion-v06-mt-09-v3"></a>
- [x] **V06-MT-09.V3** — Verify compatible historical results remain associated with their original candidate, and displays explain that a 1.0 rate means both structural cases passed. **Trace:** [Audit MT-09](docs/history/localmotive-comprehensive-audit.md#mt-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L979).

### V06-MT-10

<a id="criterion-v06-mt-10-i1"></a>
- [x] **V06-MT-10.I1** — Define one objective set and completeness policy before comparison; treat missing required evidence as ineligible or incomparable instead of dropping a different set of dimensions for each pair. **Trace:** [Audit MT-10](docs/history/localmotive-comprehensive-audit.md#mt-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L996).

<a id="criterion-v06-mt-10-i2"></a>
- [x] **V06-MT-10.I2** — Expose evidence coverage and prevent direct score comparison over candidate-specific weight denominators that reward omitted weak measurements. **Trace:** [Audit MT-10](docs/history/localmotive-comprehensive-audit.md#mt-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L997).

<a id="criterion-v06-mt-10-i3"></a>
- [x] **V06-MT-10.I3** — Precompute each objective range once, preserve explanations of constraint violations/dominators/components, and define duplicate-ID and deterministic-tie behavior. **Trace:** [Audit MT-10](docs/history/localmotive-comprehensive-audit.md#mt-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L998).

<a id="criterion-v06-mt-10-i4"></a>
- [x] **V06-MT-10.I4** — If 10,000-candidate support remains, bound the returned dominance detail and measure worst-case runtime rather than repeatedly allocating per-candidate range vectors. **Trace:** [Audit MT-10](docs/history/localmotive-comprehensive-audit.md#mt-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L999).

<a id="criterion-v06-mt-10-v1"></a>
- [x] **V06-MT-10.V1** — Use three Measured candidates all with decode=100: A prefill=100/latency=unknown/quality=0.5; B prefill=90/latency=10/quality=unknown; C prefill=unknown/latency=20/quality=0.9. Assert no A>B>C>A cycle under default null metric constraints. **Trace:** [Audit MT-10](docs/history/localmotive-comprehensive-audit.md#mt-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1003).

<a id="criterion-v06-mt-10-v2"></a>
- [x] **V06-MT-10.V2** — Verify missing required quality cannot raise a candidate above fully measured alternatives solely through a denominator change; cover all-missing, equal values, zero weights, constrained unknowns, duplicate IDs, and ties. **Trace:** [Audit MT-10](docs/history/localmotive-comprehensive-audit.md#mt-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1004).

<a id="criterion-v06-mt-10-v3"></a>
- [x] **V06-MT-10.V3** — Record worst-case latency and response size for the retained candidate limit and confirm explanations remain deterministic. **Trace:** [Audit MT-10](docs/history/localmotive-comprehensive-audit.md#mt-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1005).

### V06-MT-11

<a id="criterion-v06-mt-11-i1"></a>
- [x] **V06-MT-11.I1** — Separate allocated context capacity, occupied prompt length, per-slot capacity, concurrency, quality policy, latency, and decode throughput in the declared tuning objective. **Trace:** [Audit MT-11](docs/history/localmotive-comprehensive-audit.md#mt-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1022).

<a id="criterion-v06-mt-11-i2"></a>
- [x] **V06-MT-11.I2** — Use the corrected V2 harness with an immutable target-length workload and per-trial raw manifests, or explicitly label the retained short-prompt objective without claiming a full-context workload. **Trace:** [Audit MT-11](docs/history/localmotive-comprehensive-audit.md#mt-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1023).

<a id="criterion-v06-mt-11-i3"></a>
- [x] **V06-MT-11.I3** — Before scoring, require observed effective per-slot context to meet the selected requirement; reject or explicitly report candidates whose parallelism or automatic fit reduces it. **Trace:** [Audit MT-11](docs/history/localmotive-comprehensive-audit.md#mt-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1024).

<a id="criterion-v06-mt-11-i4"></a>
- [x] **V06-MT-11.I4** — Remeasure baseline and finalists, require a documented material improvement beyond observed variation, and apply an explicit quality policy for cache/speculation changes while keeping any three-field proposal limit enforced in Rust. **Trace:** [Audit MT-11](docs/history/localmotive-comprehensive-audit.md#mt-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1025).

<a id="criterion-v06-mt-11-v1"></a>
- [x] **V06-MT-11.V1** — Test candidates whose parallel setting divides context below target and whose fit policy reduces context; assert they cannot silently win the requested-capacity objective. **Trace:** [Audit MT-11](docs/history/localmotive-comprehensive-audit.md#mt-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1029).

<a id="criterion-v06-mt-11-v2"></a>
- [x] **V06-MT-11.V2** — Verify the long-prompt request/count contract and persisted workload for each candidate, including cancellation and raw failure retention through the V2 path. **Trace:** [Audit MT-11](docs/history/localmotive-comprehensive-audit.md#mt-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1030).

<a id="criterion-v06-mt-11-v3"></a>
- [x] **V06-MT-11.V3** — Confirm the selected winner with independent repeated measurements under the same workload and exercise the stated quality policy for precision/speculation changes. **Trace:** [Audit MT-11](docs/history/localmotive-comprehensive-audit.md#mt-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1031).

### V06-MT-12

<a id="criterion-v06-mt-12-i1"></a>
- [x] **V06-MT-12.I1** — Normalize startup and health failures into bounded structured observation errors at acquisition, preserving their outcome/category instead of storing arbitrary serialized launch evidence inline. **Trace:** [Audit MT-12](docs/history/localmotive-comprehensive-audit.md#mt-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1048).

<a id="criterion-v06-mt-12-i2"></a>
- [x] **V06-MT-12.I2** — Retain larger bounded runtime diagnostics separately and attach an explicit artifact reference/digest where available, distinguishing retained local logs from public share content. **Trace:** [Audit MT-12](docs/history/localmotive-comprehensive-audit.md#mt-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1049).

<a id="criterion-v06-mt-12-i3"></a>
- [x] **V06-MT-12.I3** — Apply Unicode-safe truncation with a visible truncation marker within the 4,096-byte observation limit; do not discard the whole run because one failure carries a longer log tail. **Trace:** [Audit MT-12](docs/history/localmotive-comprehensive-audit.md#mt-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1050).

<a id="criterion-v06-mt-12-i4"></a>
- [x] **V06-MT-12.I4** — Ensure finalization retains earlier successes, warmup failures, cancellation/timeouts, and the original diagnostic category before atomic manifest persistence. **Trace:** [Audit MT-12](docs/history/localmotive-comprehensive-audit.md#mt-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1051).

<a id="criterion-v06-mt-12-v1"></a>
- [x] **V06-MT-12.V1** — Inject a cold-start health failure whose structured error/log tail exceeds 4,096 bytes and assert a valid manifest is written with bounded summary and retained-diagnostic reference. **Trace:** [Audit MT-12](docs/history/localmotive-comprehensive-audit.md#mt-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1055).

<a id="criterion-v06-mt-12-v2"></a>
- [x] **V06-MT-12.V2** — Repeat for warmup failure and a later failed trial after successful observations; verify existing measurements and terminal outcome survive finalization. **Trace:** [Audit MT-12](docs/history/localmotive-comprehensive-audit.md#mt-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1056).

<a id="criterion-v06-mt-12-v3"></a>
- [x] **V06-MT-12.V3** — Exercise boundary-length and multibyte text, plus diagnostic-artifact failure, ensuring the user receives the original failure category rather than only a schema-limit error. **Trace:** [Audit MT-12](docs/history/localmotive-comprehensive-audit.md#mt-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1057).

### V06-MT-13

<a id="criterion-v06-mt-13-i1"></a>
- [x] **V06-MT-13.I1** — Create shared attempt/workload invariants covering positive unique sequential IDs, valid successful metrics/counts under the chosen cache protocol, requested attempt completion, and coherent terminal outcomes. **Trace:** [Audit MT-13](docs/history/localmotive-comprehensive-audit.md#mt-13).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1074).

<a id="criterion-v06-mt-13-i2"></a>
- [x] **V06-MT-13.I2** — Apply the same finalized-record contract to persistence, summary generation, replay, share construction, and direct share persistence; model draft/incomplete records separately if they are needed. **Trace:** [Audit MT-13](docs/history/localmotive-comprehensive-audit.md#mt-13).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1075).

<a id="criterion-v06-mt-13-i3"></a>
- [x] **V06-MT-13.I3** — Validate workload and launch shape plus nested hardware/effective-context/process-memory evidence, and recompute summary statistics/counts including median from the raw observations. **Trace:** [Audit MT-13](docs/history/localmotive-comprehensive-audit.md#mt-13).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1076).

<a id="criterion-v06-mt-13-i4"></a>
- [x] **V06-MT-13.I4** — Recompute quality aggregate status from cases and verify attached identities, replacing acceptance of independently supplied contradictory aggregate fields. **Trace:** [Audit MT-13](docs/history/localmotive-comprehensive-audit.md#mt-13).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1077).

<a id="criterion-v06-mt-13-v1"></a>
- [x] **V06-MT-13.V1** — Reject duplicate or zero trial IDs, zero-token successes, missing attempts without terminal failure, mismatched cache-aware counts, and contradictory terminal outcomes consistently at every exposed boundary. **Trace:** [Audit MT-13](docs/history/localmotive-comprehensive-audit.md#mt-13).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1081).

<a id="criterion-v06-mt-13-v2"></a>
- [x] **V06-MT-13.V2** — Reject altered summary counts/median, invalid workloads/launch shapes, contradictory quality cases/status, wrong quality identities, and invalid nested evidence through both builder and direct-export paths. **Trace:** [Audit MT-13](docs/history/localmotive-comprehensive-audit.md#mt-13).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1082).

<a id="criterion-v06-mt-13-v3"></a>
- [x] **V06-MT-13.V3** — Add bounded fuzz/property checks for deserialization and validation agreement, retaining valid finalized and intentionally incomplete fixtures as explicit separate contracts. **Trace:** [Audit MT-13](docs/history/localmotive-comprehensive-audit.md#mt-13).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1083).

### V06-MT-14

<a id="criterion-v06-mt-14-i1"></a>
- [x] **V06-MT-14.I1** — Reuse one complete calibration-model validator from build, persist, load, and apply, enforcing valid anchor count, creation/expiry ordering, finite metrics, and the supported TTL bound. **Trace:** [Audit MT-14](docs/history/localmotive-comprehensive-audit.md#mt-14).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1100).

<a id="criterion-v06-mt-14-i2"></a>
- [x] **V06-MT-14.I2** — Reject application before model creation, evaluate exact expiry consistently, and define freshness from original source-run timestamps rather than allowing rebuild time to refresh arbitrarily old evidence. **Trace:** [Audit MT-14](docs/history/localmotive-comprehensive-audit.md#mt-14).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1101).

<a id="criterion-v06-mt-14-i3"></a>
- [x] **V06-MT-14.I3** — Require compatible estimator/metric applicability and preserve source provenance supplied by unique-run anchors; expose unsupported or stale models as such instead of returning derived estimates. **Trace:** [Audit MT-14](docs/history/localmotive-comprehensive-audit.md#mt-14).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1102).

<a id="criterion-v06-mt-14-i4"></a>
- [x] **V06-MT-14.I4** — Align frontend/backend expiry semantics, preferably through a backend evaluation result, and check the final lower and upper interval bounds for finiteness after arithmetic. **Trace:** [Audit MT-14](docs/history/localmotive-comprehensive-audit.md#mt-14).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1103).

<a id="criterion-v06-mt-14-v1"></a>
- [x] **V06-MT-14.V1** — Test zero/insufficient anchor count, inverted timestamps, future creation, overlong TTL, key mismatch, and invalid finite arithmetic through every model entry point. **Trace:** [Audit MT-14](docs/history/localmotive-comprehensive-audit.md#mt-14).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1107).

<a id="criterion-v06-mt-14-v2"></a>
- [x] **V06-MT-14.V2** — Test one tick before, exactly at, and after expiry in both UI presentation and backend application; assert a single consistent decision. **Trace:** [Audit MT-14](docs/history/localmotive-comprehensive-audit.md#mt-14).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1108).

<a id="criterion-v06-mt-14-v3"></a>
- [x] **V06-MT-14.V3** — Rebuild from aged source anchors and confirm the documented freshness policy prevents silently renewing applicability solely through a new creation timestamp. **Trace:** [Audit MT-14](docs/history/localmotive-comprehensive-audit.md#mt-14).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1109).

### V06-MT-15

<a id="criterion-v06-mt-15-i1"></a>
- [x] **V06-MT-15.I1** — Route discovery's grouped shard names through the artifact module's analysis instead of deciding completeness from a HashSet that silently drops parse failures and duplicate file indices. **Trace:** [Audit MT-15](docs/history/localmotive-comprehensive-audit.md#mt-15).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1126).

<a id="criterion-v06-mt-15-i2"></a>
- [x] **V06-MT-15.I2** — Require every grouped file to parse consistently, agree on logical identity and expected count, and contribute exactly one unique required index. **Trace:** [Audit MT-15](docs/history/localmotive-comprehensive-audit.md#mt-15).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1127).

<a id="criterion-v06-mt-15-i3"></a>
- [x] **V06-MT-15.I3** — Return explicit duplicate, malformed, inconsistent-count, and missing-shard problems to discovery so the displayed complete state matches launch validation. **Trace:** [Audit MT-15](docs/history/localmotive-comprehensive-audit.md#mt-15).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1128).

<a id="criterion-v06-mt-15-i4"></a>
- [x] **V06-MT-15.I4** — Preserve deterministic first-shard selection, valid singleton behavior, split ordering, and existing symlink/reparse protections while replacing the divergent decision logic. **Trace:** [Audit MT-15](docs/history/localmotive-comprehensive-audit.md#mt-15).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1129).

<a id="criterion-v06-mt-15-v1"></a>
- [x] **V06-MT-15.V1** — Create a real temporary tree containing foo.gguf and foo-00001-of-00001.gguf; assert discovery and artifact validation agree that the duplicate index does not form a complete valid model. **Trace:** [Audit MT-15](docs/history/localmotive-comprehensive-audit.md#mt-15).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1133).

<a id="criterion-v06-mt-15-v2"></a>
- [x] **V06-MT-15.V2** — Cover duplicate indices, malformed index/count, conflicting expected counts, missing first shard, extension case variation, and valid complete singleton/split sets. **Trace:** [Audit MT-15](docs/history/localmotive-comprehensive-audit.md#mt-15).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1134).

<a id="criterion-v06-mt-15-v3"></a>
- [x] **V06-MT-15.V3** — Compare completeness and diagnostic outcomes across scanning and launch inspection for each fixture and verify only the intended first shard is selected for valid models. **Trace:** [Audit MT-15](docs/history/localmotive-comprehensive-audit.md#mt-15).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1135).

### V06-FE-01

<a id="criterion-v06-fe-01-i1"></a>
- [x] **V06-FE-01.I1** — Represent committed model selection and its editable launch profile as one coordinated state transition; keep uncommitted runtime-path input separate from the active runtime identity. **Trace:** [Audit FE-01](docs/history/localmotive-comprehensive-audit.md#fe-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1154).

<a id="criterion-v06-fe-01-i2"></a>
- [x] **V06-FE-01.I2** — Preserve the selected model during rescan when it still exists; otherwise load and normalize the replacement model's saved profile or clear selection/profile together, including failed scans. **Trace:** [Audit FE-01](docs/history/localmotive-comprehensive-audit.md#fe-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1155).

<a id="criterion-v06-fe-01-i3"></a>
- [x] **V06-FE-01.I3** — Route typed runtime inspection, native file selection, managed-build activation and installation completion through the same committed-runtime update so profile.runtime and inspected capabilities agree. **Trace:** [Audit FE-01](docs/history/localmotive-comprehensive-audit.md#fe-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1156).

<a id="criterion-v06-fe-01-i4"></a>
- [x] **V06-FE-01.I4** — Derive displayed model identity, readiness, completeness checks and persistence keys from the actual pending launch profile and its validated model ownership rather than unrelated selectedId state. **Trace:** [Audit FE-01](docs/history/localmotive-comprehensive-audit.md#fe-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1157).

<a id="criterion-v06-fe-01-i5"></a>
- [x] **V06-FE-01.I5** — Preserve the existing normalization rule that a deliberately selected current runtime supersedes an older runtime saved with a profile. **Trace:** [Audit FE-01](docs/history/localmotive-comprehensive-audit.md#fe-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1158).

<a id="criterion-v06-fe-01-v1"></a>
- [x] **V06-FE-01.V1** — Add component/IPC-boundary regressions selecting model B, rescanning inventory ordered A/B, deleting B, and failing the scan; assert visible identity and exact submitted/saved profile values. **Trace:** [Audit FE-01](docs/history/localmotive-comprehensive-audit.md#fe-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1162).

<a id="criterion-v06-fe-01-v2"></a>
- [x] **V06-FE-01.V2** — Exercise typed runtime B plus Inspect, file-picker B and managed activation B after configuring A; assert runtimePath, inspected runtime identity and profile.runtime converge. **Trace:** [Audit FE-01](docs/history/localmotive-comprehensive-audit.md#fe-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1163).

<a id="criterion-v06-fe-01-v3"></a>
- [x] **V06-FE-01.V3** — Run the packaged select, rescan, inspect, save and start scenario and capture the resulting server snapshot and command arguments. **Trace:** [Audit FE-01](docs/history/localmotive-comprehensive-audit.md#fe-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1164).

### V06-FE-02

<a id="criterion-v06-fe-02-i1"></a>
- [x] **V06-FE-02.I1** — Create a tuning-run record capturing the originating model ID, runtime identity, provider, advisor model and target context at dispatch, independent of the currently edited selection. **Trace:** [Audit FE-02](docs/history/localmotive-comprehensive-audit.md#fe-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1181).

<a id="criterion-v06-fe-02-i2"></a>
- [x] **V06-FE-02.I2** — Route progress, completion and stored reports through that run identity; retain the original run's provenance when navigation selects another model or a separate draft provider. **Trace:** [Audit FE-02](docs/history/localmotive-comprehensive-audit.md#fe-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1182).

<a id="criterion-v06-fe-02-i3"></a>
- [x] **V06-FE-02.I3** — Make Adopt best operate on the report's originating model instead of the current selectedId/name/context, and explicitly reconcile runtime selection through the established profile normalization policy. **Trace:** [Audit FE-02](docs/history/localmotive-comprehensive-audit.md#fe-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1183).

<a id="criterion-v06-fe-02-i4"></a>
- [x] **V06-FE-02.I4** — Prevent delayed progress from a finished run from updating a subsequent run; choose whether incompatible edits are disabled during tuning or clearly represented as separate drafts. **Trace:** [Audit FE-02](docs/history/localmotive-comprehensive-audit.md#fe-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1184).

<a id="criterion-v06-fe-02-i5"></a>
- [x] **V06-FE-02.I5** — Display the actual running advisor/model/context and completion state from the immutable record so changing provider tabs cannot relabel work already dispatched. **Trace:** [Audit FE-02](docs/history/localmotive-comprehensive-audit.md#fe-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1185).

<a id="criterion-v06-fe-02-v1"></a>
- [x] **V06-FE-02.V1** — Start tuning A, select B before deferred progress/completion arrives, then adopt; assert B's report, profile and storage key are not overwritten by A. **Trace:** [Audit FE-02](docs/history/localmotive-comprehensive-audit.md#fe-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1189).

<a id="criterion-v06-fe-02-v2"></a>
- [x] **V06-FE-02.V2** — Activate runtime B after measuring on runtime A and verify adoption follows the documented current-runtime policy without silently restoring A. **Trace:** [Audit FE-02](docs/history/localmotive-comprehensive-audit.md#fe-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1190).

<a id="criterion-v06-fe-02-v3"></a>
- [x] **V06-FE-02.V3** — Complete a run, start another and deliver late events from the first; verify the second run's trials and status remain unchanged. **Trace:** [Audit FE-02](docs/history/localmotive-comprehensive-audit.md#fe-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1191).

### V06-FE-03

<a id="criterion-v06-fe-03-i1"></a>
- [x] **V06-FE-03.I1** — Extend the existing request-sequence approach to cloud credential/model loads, connection probes, OAuth completion, key save/forget, GGUF reads, port suggestions and command preview. **Trace:** [Audit FE-03](docs/history/localmotive-comprehensive-audit.md#fe-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1208).

<a id="criterion-v06-fe-03-i2"></a>
- [x] **V06-FE-03.I2** — Capture the resource identity and form revision for each request and check them before committing success, failure or loading completion; obsolete responses must not replace current state. **Trace:** [Audit FE-03](docs/history/localmotive-comprehensive-audit.md#fe-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1209).

<a id="criterion-v06-fe-03-i3"></a>
- [x] **V06-FE-03.I3** — Clear provider-specific credential/model/probe presentation at provider-switch start, and clear GGUF metadata when selection changes or becomes absent rather than retaining another model's facts. **Trace:** [Audit FE-03](docs/history/localmotive-comprehensive-audit.md#fe-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1210).

<a id="criterion-v06-fe-03-i4"></a>
- [x] **V06-FE-03.I4** — Apply a suggested port only when both profile identity and the originally requested port remain unchanged; never overwrite a later manual edit or newly selected profile. **Trace:** [Audit FE-03](docs/history/localmotive-comprehensive-audit.md#fe-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1211).

<a id="criterion-v06-fe-03-i5"></a>
- [x] **V06-FE-03.I5** — Keep independent loading/error state for these resources and bind tuning readiness to the active provider's own credential and model result. **Trace:** [Audit FE-03](docs/history/localmotive-comprehensive-audit.md#fe-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1212).

<a id="criterion-v06-fe-03-v1"></a>
- [x] **V06-FE-03.V1** — Use deferred IPC-boundary promises to resolve provider A after B across credential/model/probe and credential-mutation operations; assert B's tab and actual readiness retain B's data. **Closed 2026-09-12:** `src/App.staleResponses.test.tsx` drives the production callers (deferred invokes) — provider-swap relabel, credential-save, probe reply and command-preview legs; mutants MB1-MB5 each failed their leg and passed after restore. **Trace:** [Audit FE-03](docs/history/localmotive-comprehensive-audit.md#fe-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1216).

<a id="criterion-v06-fe-03-v2"></a>
- [x] **V06-FE-03.V2** — Resolve model A metadata after selecting B or clearing selection; verify architecture and native-context choices are not overwritten or incorrectly clamped. **Trace:** [Audit FE-03](docs/history/localmotive-comprehensive-audit.md#fe-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1217).

<a id="criterion-v06-fe-03-v3"></a>
- [x] **V06-FE-03.V3** — Return an old port suggestion after a manual edit and an old command after a newer preview; assert both are discarded through actual component callers. **Closed 2026-09-12:** the manual-port-edit vs late-suggestion leg and the stale-preview discard leg in `src/App.staleResponses.test.tsx`; mutants MB1/MB2 caught. **Trace:** [Audit FE-03](docs/history/localmotive-comprehensive-audit.md#fe-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1218).

### V06-FE-04

<a id="criterion-v06-fe-04-i1"></a>
- [x] **V06-FE-04.I1** — Debounce or coalesce profile-edit previews so rapid character changes do not dispatch complete prepare_launch work for every intermediate form value. **Trace:** [Audit FE-04](docs/history/localmotive-comprehensive-audit.md#fe-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1235).

<a id="criterion-v06-fe-04-i2"></a>
- [x] **V06-FE-04.I2** — Separate cheap command composition from authoritative runtime trust, artifact, path and argument validation; describe provisional previews honestly and retain all required checks at actual launch. **Trace:** [Audit FE-04](docs/history/localmotive-comprehensive-audit.md#fe-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1236).

<a id="criterion-v06-fe-04-i3"></a>
- [x] **V06-FE-04.I3** — Move necessary expensive runtime probing and filesystem verification into a cancellable background operation, preventing synchronous preview work from blocking interface interaction. **Trace:** [Audit FE-04](docs/history/localmotive-comprehensive-audit.md#fe-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1237).

<a id="criterion-v06-fe-04-i4"></a>
- [x] **V06-FE-04.I4** — Cache runtime capability/artifact evidence only under explicit safe identity and freshness rules; invalidate it when executable/model identity changes rather than treating old evidence as current. **Trace:** [Audit FE-04](docs/history/localmotive-comprehensive-audit.md#fe-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1238).

<a id="criterion-v06-fe-04-i5"></a>
- [x] **V06-FE-04.I5** — Discard preview results for obsolete form revisions, coordinating with FE-03, and avoid native work for purely human-facing changes such as the profile display name when arguments are unchanged. **Trace:** [Audit FE-04](docs/history/localmotive-comprehensive-audit.md#fe-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1239).

<a id="criterion-v06-fe-04-v1"></a>
- [x] **V06-FE-04.V1** — Instrument probe/job counts while rapidly editing a profile; assert a bounded coalesced request count and an exact command corresponding only to the final accepted form. **Trace:** [Audit FE-04](docs/history/localmotive-comprehensive-audit.md#fe-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1243).

<a id="criterion-v06-fe-04-v2"></a>
- [x] **V06-FE-04.V2** — Simulate slow version/help probes and verify input, navigation and cancellation remain responsive in the packaged Windows binary; record actual observations rather than assumed timing. **Trace:** [Audit FE-04](docs/history/localmotive-comprehensive-audit.md#fe-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1244).

<a id="criterion-v06-fe-04-v3"></a>
- [x] **V06-FE-04.V3** — Tamper with or replace a runtime/artifact after cached preview evidence and assert actual launch still performs and enforces the authoritative trust checks. **Trace:** [Audit FE-04](docs/history/localmotive-comprehensive-audit.md#fe-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1245).

### V06-FE-05

<a id="criterion-v06-fe-05-i1"></a>
- [x] **V06-FE-05.I1** — Move benchmark, quality, preflight, ranking, imported-evidence and active-operation records out of the conditionally mounted Benchmark view into persistent application-level state or a dedicated store. **Trace:** [Audit FE-05](docs/history/localmotive-comprehensive-audit.md#fe-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1262).

<a id="criterion-v06-fe-05-i2"></a>
- [x] **V06-FE-05.I2** — Keep an application-wide run status and cancellation handle available while the user visits Control, Profile, Inventory or another screen; navigation must not orphan a dispatched measurement. **Trace:** [Audit FE-05](docs/history/localmotive-comprehensive-audit.md#fe-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1263).

<a id="criterion-v06-fe-05-i3"></a>
- [x] **V06-FE-05.I3** — Retain historical evidence with immutable model/runtime/workload/run identities instead of clearing the entire history when a profile fingerprint changes. **Trace:** [Audit FE-05](docs/history/localmotive-comprehensive-audit.md#fe-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1264).

<a id="criterion-v06-fe-05-i4"></a>
- [x] **V06-FE-05.I4** — Provide a saved-manifest recovery/loading route keyed by evidence identity and reconnect the view to completion received while it was unmounted. **Trace:** [Audit FE-05](docs/history/localmotive-comprehensive-audit.md#fe-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1265).

<a id="criterion-v06-fe-05-i5"></a>
- [x] **V06-FE-05.I5** — Separate invalidation of the current preflight or editable preview from retention of completed runs, and reset any export approval when its reviewed export content changes. **Trace:** [Audit FE-05](docs/history/localmotive-comprehensive-audit.md#fe-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1266).

<a id="criterion-v06-fe-05-v1"></a>
- [x] **V06-FE-05.V1** — Start a deferred benchmark, navigate away and back, then cancel; assert the same run remains active and the original cancellation handle is available. **Trace:** [Audit FE-05](docs/history/localmotive-comprehensive-audit.md#fe-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1270).

<a id="criterion-v06-fe-05-v2"></a>
- [x] **V06-FE-05.V2** — Complete measurement while another screen is open and verify returning to Benchmark displays that run and its saved evidence rather than a new empty session. **Trace:** [Audit FE-05](docs/history/localmotive-comprehensive-audit.md#fe-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1271).

<a id="criterion-v06-fe-05-v3"></a>
- [x] **V06-FE-05.V3** — Benchmark profile A, leave to edit/start B, benchmark B, then compare both records with distinct provenance and recover them through the supported saved-manifest route. **Closed 2026-09-12 (packaged, `scripts/g05_fe05v3.mjs`):** A (8192 context) and B (4096) each measured; manifests persisted under the app-data `benchmarks/` root with distinct paths, distinct launch provenance (`-c 8192` vs `-c 4096`) and distinct compatibility identities; both recovered through the supported route (`Add anchor` from each saved manifest, records loaded by key with `sourceRunId`s, `Replay manifest` restored the workload). **Trace:** [Audit FE-05](docs/history/localmotive-comprehensive-audit.md#fe-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1272).

### V06-FE-06

<a id="criterion-v06-fe-06-i1"></a>
- [x] **V06-FE-06.I1** — Store the exact selected adapters, hardware observation, manual capacity/note and consequential profile inputs with each preflight result, then derive whether the displayed result is current. **Trace:** [Audit FE-06](docs/history/localmotive-comprehensive-audit.md#fe-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1291).

<a id="criterion-v06-fe-06-i2"></a>
- [x] **V06-FE-06.I2** — Mark prior preflight/allocation evidence stale or clear its current-result presentation after adapter changes, manual-override edits, hardware refresh or any profile input used by preflight changes. **Trace:** [Audit FE-06](docs/history/localmotive-comprehensive-audit.md#fe-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1292).

<a id="criterion-v06-fe-06-i3"></a>
- [x] **V06-FE-06.I3** — Replace the incomplete profile fingerprint with a full preflight-input identity including draft/projector, speculation, KV offload, device, fitting and attention settings where relevant. **Trace:** [Audit FE-06](docs/history/localmotive-comprehensive-audit.md#fe-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1293).

<a id="criterion-v06-fe-06-i4"></a>
- [x] **V06-FE-06.I4** — Distinguish uninitialized adapter selection from the user's deliberate empty selection; initialize defaults once and allow unchecking the final adapter without immediately reselecting another. **Trace:** [Audit FE-06](docs/history/localmotive-comprehensive-audit.md#fe-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1294).

<a id="criterion-v06-fe-06-i5"></a>
- [x] **V06-FE-06.I5** — Align explicit adapter choice between Runtime and the evidence panel, and reject async inspection/preflight responses whose captured input revision is no longer current. **Trace:** [Audit FE-06](docs/history/localmotive-comprehensive-audit.md#fe-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1295).

<a id="criterion-v06-fe-06-v1"></a>
- [x] **V06-FE-06.V1** — Run preflight then change an adapter, capacity, note or refreshed hardware observation; assert old budget/allocation output is explicitly stale until recomputed. **Trace:** [Audit FE-06](docs/history/localmotive-comprehensive-audit.md#fe-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1299).

<a id="criterion-v06-fe-06-v2"></a>
- [x] **V06-FE-06.V2** — Uncheck the final adapter and verify the empty choice persists; refresh hardware with removed adapters and confirm selection reconciliation preserves deliberate intent. **Trace:** [Audit FE-06](docs/history/localmotive-comprehensive-audit.md#fe-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1300).

<a id="criterion-v06-fe-06-v3"></a>
- [x] **V06-FE-06.V3** — Resolve an old preflight after changing inputs, and after FE-05 makes state persistent edit each previously omitted profile field; verify stale evidence is never presented as current. **Trace:** [Audit FE-06](docs/history/localmotive-comprehensive-audit.md#fe-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1301).

### V06-FE-07

<a id="criterion-v06-fe-07-i1"></a>
- [x] **V06-FE-07.I1** — Represent benchmark lifecycle and cancellation-request progress separately instead of calling the generic busy-state wrapper with a replacement cancel operation name. **Trace:** [Audit FE-07](docs/history/localmotive-comprehensive-audit.md#fe-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1318).

<a id="criterion-v06-fe-07-i2"></a>
- [x] **V06-FE-07.I2** — Retain running or cancelling status until the original benchmark promise reaches its terminal outcome, rather than clearing active ownership when the cancel command acknowledges receipt. **Trace:** [Audit FE-07](docs/history/localmotive-comprehensive-audit.md#fe-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1319).

<a id="criterion-v06-fe-07-i3"></a>
- [x] **V06-FE-07.I3** — Replace null-as-failure action results with an explicit tagged success/error result so a successful Rust unit response can produce truthful cancellation acknowledgement. **Trace:** [Audit FE-07](docs/history/localmotive-comprehensive-audit.md#fe-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1320).

<a id="criterion-v06-fe-07-i4"></a>
- [x] **V06-FE-07.I4** — Prevent conflicting evidence actions from becoming enabled while the original run remains active, and give Add anchor the same compatible-operation guard used by other evidence actions. **Trace:** [Audit FE-07](docs/history/localmotive-comprehensive-audit.md#fe-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1321).

<a id="criterion-v06-fe-07-i5"></a>
- [x] **V06-FE-07.I5** — Handle failed or unavailable cancellation by retaining the run record and showing actionable feedback rather than implying termination or losing the remaining cancel/status affordance. **Trace:** [Audit FE-07](docs/history/localmotive-comprehensive-audit.md#fe-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1322).

<a id="criterion-v06-fe-07-v1"></a>
- [x] **V06-FE-07.V1** — Resolve cancel acknowledgement before the original benchmark ends; assert the interface stays cancelling and a new benchmark/quality action remains unavailable. **Trace:** [Audit FE-07](docs/history/localmotive-comprehensive-audit.md#fe-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1326).

<a id="criterion-v06-fe-07-v2"></a>
- [x] **V06-FE-07.V2** — Exercise successful null/unit responses, rejected cancellation and no-active-benchmark errors through the actual IPC action wrapper; verify distinct messages and lifecycle outcomes. **Closed 2026-09-12:** `src/V03EvidencePanel.cancel.test.tsx` drives the panel's own wrapper — a void acknowledgement keeps run ownership and says the run ends after the current attempt, a rejected cancel keeps the run and the next affordance, a late cancel retains the measured result, and the no-active-benchmark error does not lock the UI; mutants MF1/MF2 caught. **Trace:** [Audit FE-07](docs/history/localmotive-comprehensive-audit.md#fe-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1327).

<a id="criterion-v06-fe-07-v3"></a>
- [x] **V06-FE-07.V3** — Complete, fail and cancel the original run after acknowledgement; assert only its terminal result releases active ownership and conflicting actions. **Trace:** [Audit FE-07](docs/history/localmotive-comprehensive-audit.md#fe-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1328).

### V06-FE-08

<a id="criterion-v06-fe-08-i1"></a>
- [x] **V06-FE-08.I1** — Maintain an editable raw-argument string independently of the parsed profile.extraArgs array while the field is being edited so trailing spaces are not immediately normalized away. **Trace:** [Audit FE-08](docs/history/localmotive-comprehensive-audit.md#fe-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1345).

<a id="criterion-v06-fe-08-i2"></a>
- [x] **V06-FE-08.I2** — Parse and validate the draft at a deliberate boundary such as blur or explicit validation, preserving the existing documented self-contained token syntax. **Trace:** [Audit FE-08](docs/history/localmotive-comprehensive-audit.md#fe-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1346).

<a id="criterion-v06-fe-08-i3"></a>
- [x] **V06-FE-08.I3** — Keep visible draft text and committed parsed arguments distinguishable when validation fails; show the problematic token and recovery instead of silently concatenating or discarding input. **Trace:** [Audit FE-08](docs/history/localmotive-comprehensive-audit.md#fe-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1347).

<a id="criterion-v06-fe-08-i4"></a>
- [x] **V06-FE-08.I4** — Define behavior for paste, repeated whitespace, deletion, whitespace-only drafts and unsupported quoted/space-containing values without introducing shell execution or broadening privileged overrides. **Trace:** [Audit FE-08](docs/history/localmotive-comprehensive-audit.md#fe-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1348).

<a id="criterion-v06-fe-08-i5"></a>
- [x] **V06-FE-08.I5** — Preserve the backend restrictions on typed-field overrides and privileged capabilities; synchronize normalized committed text only after a successful parse/validation transition. **Trace:** [Audit FE-08](docs/history/localmotive-comprehensive-audit.md#fe-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1349).

<a id="criterion-v06-fe-08-v1"></a>
- [x] **V06-FE-08.V1** — Dispatch actual input events typing --flag-one, a space, and --flag-two=value; assert the visible separator remains and the committed IPC profile contains two argument tokens. **Trace:** [Audit FE-08](docs/history/localmotive-comprehensive-audit.md#fe-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1353).

<a id="criterion-v06-fe-08-v2"></a>
- [x] **V06-FE-08.V2** — Exercise paste, cursor edits, deletion, repeated whitespace, blank input and unsupported quoting, checking that errors preserve editable input and valid tokens remain deterministic. **Trace:** [Audit FE-08](docs/history/localmotive-comprehensive-audit.md#fe-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1354).

<a id="criterion-v06-fe-08-v3"></a>
- [x] **V06-FE-08.V3** — Submit disallowed privileged and typed-field override tokens through the corrected field and verify backend rejection remains intact. **Trace:** [Audit FE-08](docs/history/localmotive-comprehensive-audit.md#fe-08).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1355).

### V06-FE-09

<a id="criterion-v06-fe-09-i1"></a>
- [x] **V06-FE-09.I1** — Introduce safe versioned parsing, validation and migration for persisted profiles and tuning reports before normalization or render-time array/property access. **Trace:** [Audit FE-09](docs/history/localmotive-comprehensive-audit.md#fe-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1372).

<a id="criterion-v06-fe-09-i2"></a>
- [x] **V06-FE-09.I2** — Quarantine or isolate invalid records and provide a per-record recovery/reset path with a default profile fallback, preserving unrelated valid user records. **Trace:** [Audit FE-09](docs/history/localmotive-comprehensive-audit.md#fe-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1373).

<a id="criterion-v06-fe-09-i3"></a>
- [x] **V06-FE-09.I3** — Wrap every settings/profile/report write and separate persistence failure from a successfully completed native tuning or benchmark operation so valid results remain visible. **Trace:** [Audit FE-09](docs/history/localmotive-comprehensive-audit.md#fe-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1374).

<a id="criterion-v06-fe-09-i4"></a>
- [x] **V06-FE-09.I4** — Add a root error boundary with actionable recovery/diagnostic controls and ensure it does not replace targeted storage error handling. **Trace:** [Audit FE-09](docs/history/localmotive-comprehensive-audit.md#fe-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1375).

<a id="criterion-v06-fe-09-i5"></a>
- [x] **V06-FE-09.I5** — Define bounded report retention and explicit profile/report export or management controls so accumulated per-model records cannot exhaust browser storage without a recoverable explanation. **Trace:** [Audit FE-09](docs/history/localmotive-comprehensive-audit.md#fe-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1376).

<a id="criterion-v06-fe-09-v1"></a>
- [x] **V06-FE-09.V1** — Load malformed JSON, null, wrong extraArgs/trials types, older schemas and invalid enum values through normal selection/startup; assert the app remains usable and identifies the affected record. **Trace:** [Audit FE-09](docs/history/localmotive-comprehensive-audit.md#fe-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1380).

<a id="criterion-v06-fe-09-v2"></a>
- [x] **V06-FE-09.V2** — Inject quota/security write failures after successful native tuning/benchmark completion and in direct profile/settings handlers; verify completion remains visible and save failure is separately explained. **Trace:** [Audit FE-09](docs/history/localmotive-comprehensive-audit.md#fe-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1381).

<a id="criterion-v06-fe-09-v3"></a>
- [x] **V06-FE-09.V3** — Exercise blocked storage reads and recovery/reset of one corrupt record; assert other saved profiles survive and a valid migrated record still follows current-runtime normalization. **Trace:** [Audit FE-09](docs/history/localmotive-comprehensive-audit.md#fe-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1382).

### V06-FE-11

<a id="criterion-v06-fe-11-i1"></a>
- [x] **V06-FE-11.I1** — Capture source repository, filename, revision and destination in an immutable transfer record at dispatch, and use a stable job identity for progress and cancellation. **Trace:** [Audit FE-11](docs/history/localmotive-comprehensive-audit.md#fe-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1401).

<a id="criterion-v06-fe-11-i2"></a>
- [x] **V06-FE-11.I2** — Cancel the original job rather than rebuilding its target from the currently editable modelRoot; preserve the captured destination when the user chooses a new download folder. **Trace:** [Audit FE-11](docs/history/localmotive-comprehensive-audit.md#fe-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1402).

<a id="criterion-v06-fe-11-i3"></a>
- [x] **V06-FE-11.I3** — Expose active transfers independently of catalog filters and the selected build on a card so changing either cannot hide a running job's status or stop control. **Trace:** [Audit FE-11](docs/history/localmotive-comprehensive-audit.md#fe-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1403).

<a id="criterion-v06-fe-11-i4"></a>
- [x] **V06-FE-11.I4** — Handle rejected and false cancellation responses explicitly; show stopping only when accepted and retain useful diagnostics rather than leaving an unhandled promise rejection. **Trace:** [Audit FE-11](docs/history/localmotive-comprehensive-audit.md#fe-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1404).

<a id="criterion-v06-fe-11-i5"></a>
- [x] **V06-FE-11.I5** — Scope already-on-disk and completed-transfer presentation to the relevant destination/revision, retain authoritative backend verification, and provide inventory refresh or a clear completion action. **Trace:** [Audit FE-11](docs/history/localmotive-comprehensive-audit.md#fe-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1405).

<a id="criterion-v06-fe-11-v1"></a>
- [x] **V06-FE-11.V1** — Start in folder A, change destination to B, then stop; assert the cancellation request targets A's original job and false/failure responses are described truthfully. **Trace:** [Audit FE-11](docs/history/localmotive-comprehensive-audit.md#fe-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1409).

<a id="criterion-v06-fe-11-v2"></a>
- [x] **V06-FE-11.V2** — Change the selected build and catalog filters while transferring; verify all active jobs remain visible and individually cancellable. **Trace:** [Audit FE-11](docs/history/localmotive-comprehensive-audit.md#fe-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1410).

<a id="criterion-v06-fe-11-v3"></a>
- [x] **V06-FE-11.V3** — Exercise the same repository/file across different destinations or revisions, completion after selection changes, and verification of a prior done event against the newly chosen destination. **Trace:** [Audit FE-11](docs/history/localmotive-comprehensive-audit.md#fe-11).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1411).

### V06-FE-12

<a id="criterion-v06-fe-12-i1"></a>
- [x] **V06-FE-12.I1** — Remove the outline reset from .path-bar input or add an explicit .path-bar input:focus-visible rule applying the existing amber outline and offset. **Trace:** [Audit FE-12](docs/history/localmotive-comprehensive-audit.md#fe-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1428).

<a id="criterion-v06-fe-12-i2"></a>
- [x] **V06-FE-12.I2** — Scope the correction to the Model root and Download destination path-bar fields, both of which have border:0 and no alternative declared focus indicator. **Trace:** [Audit FE-12](docs/history/localmotive-comprehensive-audit.md#fe-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1429).

<a id="criterion-v06-fe-12-i3"></a>
- [x] **V06-FE-12.I3** — Retain the global :focus-visible treatment for ordinary labeled controls; do not rewrite unrelated input styles on the mistaken assumption that label input overrides the global pseudo-class. **Trace:** [Audit FE-12](docs/history/localmotive-comprehensive-audit.md#fe-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1430).

<a id="criterion-v06-fe-12-i4"></a>
- [x] **V06-FE-12.I4** — Ensure the restored outline is visible against the path-bar background and is not clipped by surrounding borders, overflow behavior or the narrow-screen layout. **Trace:** [Audit FE-12](docs/history/localmotive-comprehensive-audit.md#fe-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1431).

<a id="criterion-v06-fe-12-i5"></a>
- [x] **V06-FE-12.I5** — Keep pointer interaction and existing path input behavior intact while satisfying the normative design document's requirement to preserve the amber keyboard-focus ring. **Trace:** [Audit FE-12](docs/history/localmotive-comprehensive-audit.md#fe-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1432).

<a id="criterion-v06-fe-12-v1"></a>
- [x] **V06-FE-12.V1** — Tab into the model-folder and download-destination inputs and assert a nonzero amber outline in browser computed styles, including after editing and choosing a folder. **Trace:** [Audit FE-12](docs/history/localmotive-comprehensive-audit.md#fe-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1436).

<a id="criterion-v06-fe-12-v2"></a>
- [x] **V06-FE-12.V2** — Use an ordinary labeled runtime/profile input as a control; verify its existing global outline remains present and no unrelated focus regression is introduced. **Trace:** [Audit FE-12](docs/history/localmotive-comprehensive-audit.md#fe-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1437).

<a id="criterion-v06-fe-12-v3"></a>
- [x] **V06-FE-12.V3** — Perform packaged Windows keyboard traversal through both path bars at normal and narrow/zoomed layouts, recording whether focus is continuously identifiable. **Trace:** [Audit FE-12](docs/history/localmotive-comprehensive-audit.md#fe-12).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1438).

### V06-FE-13

<a id="criterion-v06-fe-13-i1"></a>
- [x] **V06-FE-13.I1** — Replace the incomplete inventory role=table structure with a native table containing column headers, cells and clearly named row actions, or explicitly choose a coherent accessible list pattern. **Trace:** [Audit FE-13](docs/history/localmotive-comprehensive-audit.md#fe-13).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1455).

<a id="criterion-v06-fe-13-i2"></a>
- [x] **V06-FE-13.I2** — Give every paired N-gram minimum/maximum and map lookup/draft-size input its own associated label identifying its distinct value instead of placing two inputs under one label. **Trace:** [Audit FE-13](docs/history/localmotive-comprehensive-audit.md#fe-13).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1456).

<a id="criterion-v06-fe-13-i3"></a>
- [x] **V06-FE-13.I3** — Implement provider tabs with the complete selected-tab, tabpanel, aria-controls and keyboard/roving-tabIndex relationships, or use ordinary buttons in a labeled group with suitable semantics. **Trace:** [Audit FE-13](docs/history/localmotive-comprehensive-audit.md#fe-13).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1457).

<a id="criterion-v06-fe-13-i4"></a>
- [x] **V06-FE-13.I4** — Expose the current primary navigation destination semantically and define focus placement or a skip-to-main route after changing screens. **Trace:** [Audit FE-13](docs/history/localmotive-comprehensive-audit.md#fe-13).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1458).

<a id="criterion-v06-fe-13-i5"></a>
- [x] **V06-FE-13.I5** — Repair the evidence heading hierarchy so its sections follow the containing heading level while retaining native controls and disclosure behavior. **Trace:** [Audit FE-13](docs/history/localmotive-comprehensive-audit.md#fe-13).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1459).

<a id="criterion-v06-fe-13-v1"></a>
- [x] **V06-FE-13.V1** — Add component accessible-role/name assertions for inventory columns/actions and every paired input; run an accessibility checker against representative loaded, empty and error states. **Trace:** [Audit FE-13](docs/history/localmotive-comprehensive-audit.md#fe-13).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1463).

<a id="criterion-v06-fe-13-v2"></a>
- [x] **V06-FE-13.V2** — Exercise provider navigation with Tab, Arrow keys and relevant Home/End behavior, and verify selection, focus and displayed panel remain synchronized. **Trace:** [Audit FE-13](docs/history/localmotive-comprehensive-audit.md#fe-13).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1464).

<a id="criterion-v06-fe-13-v3"></a>
- [x] **V06-FE-13.V3** — Use Windows Narrator or NVDA in the packaged app to traverse navigation, inventory, profile pairs and evidence headings, recording labels and focus after screen changes. **Trace:** [Audit FE-13](docs/history/localmotive-comprehensive-audit.md#fe-13).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1465).

### V06-FE-14

<a id="criterion-v06-fe-14-i1"></a>
- [x] **V06-FE-14.I1** — Replace the audited hardware-source, runtime-code, field-help and empty-log text colors with tokens meeting at least 4.5:1 against their actual regular-text backgrounds. **Trace:** [Audit FE-14](docs/history/localmotive-comprehensive-audit.md#fe-14).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1482).

<a id="criterion-v06-fe-14-i2"></a>
- [x] **V06-FE-14.I2** — Review extremely small help/evidence typography and increase practical sizes without losing complete values, treating computed contrast and legibility as related but distinct checks. **Trace:** [Audit FE-14](docs/history/localmotive-comprehensive-audit.md#fe-14).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1483).

<a id="criterion-v06-fe-14-i3"></a>
- [x] **V06-FE-14.I3** — Add evidence-specific one-column breakpoints, wrap action-heading rows and constrain 205px/240px grid minima so nested padding cannot force clipped controls in a 320px window. **Trace:** [Audit FE-14](docs/history/localmotive-comprehensive-audit.md#fe-14).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1484).

<a id="criterion-v06-fe-14-i4"></a>
- [x] **V06-FE-14.I4** — Redesign narrow bottom navigation and undersized path/plain-link/managed-entry targets to satisfy the repository's 44px mobile target requirement while retaining bottom clearance. **Trace:** [Audit FE-14](docs/history/localmotive-comprehensive-audit.md#fe-14).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1485).

<a id="criterion-v06-fe-14-i5"></a>
- [x] **V06-FE-14.I5** — Preserve intentional horizontal scrolling for the inventory table rather than replacing its information structure with unrelated cards, and verify zoomed desktop reflow. **Trace:** [Audit FE-14](docs/history/localmotive-comprehensive-audit.md#fe-14).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1486).

<a id="criterion-v06-fe-14-v1"></a>
- [x] **V06-FE-14.V1** — Calculate contrast from the final computed foreground/background pairs for all four affected regular-text cases; assert ratios meet the threshold without rounding an under-threshold value upward. **Trace:** [Audit FE-14](docs/history/localmotive-comprehensive-audit.md#fe-14).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1490).

<a id="criterion-v06-fe-14-v2"></a>
- [x] **V06-FE-14.V2** — Inspect 320, 375, 680 and 980px layouts and 200%/400% zoom for clipped evidence controls, horizontal overflow outside intentional inventory scrolling and reachable action controls. **Trace:** [Audit FE-14](docs/history/localmotive-comprehensive-audit.md#fe-14).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1491).

<a id="criterion-v06-fe-14-v3"></a>
- [x] **V06-FE-14.V3** — Measure target dimensions for bottom navigation/path actions and run packaged Windows visual/DPI or high-contrast checks, documenting actual remaining limitations. **Trace:** [Audit FE-14](docs/history/localmotive-comprehensive-audit.md#fe-14).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1492).

### V06-FE-15

<a id="criterion-v06-fe-15-i1"></a>
- [x] **V06-FE-15.I1** — Replace the loaded-profile VALID label when only shard completeness is known with precise states such as shards complete, path selected, inspection pending or validated. **Trace:** [Audit FE-15](docs/history/localmotive-comprehensive-audit.md#fe-15).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1509).

<a id="criterion-v06-fe-15-i2"></a>
- [x] **V06-FE-15.I2** — Stop marking first-run Runtime/Serve ready solely because path strings are nonempty; derive each readiness state from the inspected identity and relevant native validation evidence. **Trace:** [Audit FE-15](docs/history/localmotive-comprehensive-audit.md#fe-15).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1510).

<a id="criterion-v06-fe-15-i3"></a>
- [x] **V06-FE-15.I3** — Clear or explicitly mark stale runtime capabilities when committed runtime identity changes, and avoid describing a hard-coded speculation fallback as methods advertised by the executable. **Trace:** [Audit FE-15](docs/history/localmotive-comprehensive-audit.md#fe-15).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1511).

<a id="criterion-v06-fe-15-i4"></a>
- [x] **V06-FE-15.I4** — Offer supported speculation methods after inspection or label provisional selections clearly; expose unsupported/unknown capability status near affected settings without treating UI availability as support evidence. **Trace:** [Audit FE-15](docs/history/localmotive-comprehensive-audit.md#fe-15).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1512).

<a id="criterion-v06-fe-15-i5"></a>
- [x] **V06-FE-15.I5** — Replace unconditional green evidence headings with status-based tones so Unknown, Blocked, rejected candidates and non-measured classes use the defined neutral/amber/red semantics with words. **Trace:** [Audit FE-15](docs/history/localmotive-comprehensive-audit.md#fe-15).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1513).

<a id="criterion-v06-fe-15-v1"></a>
- [x] **V06-FE-15.V1** — Render nonexistent runtime paths, pending/failed inspection and a complete-shard model with an invalid profile; assert no unsupported VALID/ready claim is shown. **Trace:** [Audit FE-15](docs/history/localmotive-comprehensive-audit.md#fe-15).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1517).

<a id="criterion-v06-fe-15-v2"></a>
- [x] **V06-FE-15.V2** — Switch executable identity and delay inspection completion; verify old capabilities are not presented as current, and fallback methods are explicitly provisional or unavailable. **Trace:** [Audit FE-15](docs/history/localmotive-comprehensive-audit.md#fe-15).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1518).

<a id="criterion-v06-fe-15-v3"></a>
- [x] **V06-FE-15.V3** — Exercise unknown, blocked, rejected, measured and launch-validated evidence classes; assert each has accurate text and the intended semantic tone. **Trace:** [Audit FE-15](docs/history/localmotive-comprehensive-audit.md#fe-15).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1519).

### V06-FE-16

<a id="criterion-v06-fe-16-i1"></a>
- [x] **V06-FE-16.I1** — Replace the shared busy string and disconnected operation flags with explicit UI operation records and compatibility rules so one completion cannot clear another active operation. **Trace:** [Audit FE-16](docs/history/localmotive-comprehensive-audit.md#fe-16).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1536).

<a id="criterion-v06-fe-16-i2"></a>
- [x] **V06-FE-16.I2** — Coordinate legacy and v2 measurement controls, profile Start and other conflicting actions with the real active lifecycle, while preserving independent nonconflicting work where supported. **Trace:** [Audit FE-16](docs/history/localmotive-comprehensive-audit.md#fe-16).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1537).

<a id="criterion-v06-fe-16-i3"></a>
- [x] **V06-FE-16.I3** — Make server-status retrieval single-flight or event-driven, reject obsolete poll results after start/stop transitions, and avoid expected failing native polling in browser preview. **Trace:** [Audit FE-16](docs/history/localmotive-comprehensive-audit.md#fe-16).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1538).

<a id="criterion-v06-fe-16-i4"></a>
- [x] **V06-FE-16.I4** — Render running strategy/identity from the server snapshot rather than the current editable profile, and label legacy benchmark results with their actual model, runtime, workload and observation time. **Trace:** [Audit FE-16](docs/history/localmotive-comprehensive-audit.md#fe-16).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1539).

<a id="criterion-v06-fe-16-i5"></a>
- [x] **V06-FE-16.I5** — Preserve the last bounded server log and exit/failure evidence after a process stops; provide explicit semantics for legacy vs v2 measurement rather than unrelated results on one screen. **Trace:** [Audit FE-16](docs/history/localmotive-comprehensive-audit.md#fe-16).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1540).

<a id="criterion-v06-fe-16-v1"></a>
- [x] **V06-FE-16.V1** — Complete overlapping scan/cloud/start requests in different orders and verify active state persists; exercise legacy/v2 measurement and Start guards through actual UI interactions. **Closed 2026-09-12 (packaged, `scripts/g05_fe16.mjs`):** scan+cloud overlap left the live server on the same pid; a 10-trial v2 run survived navigation, a rescan and a cloud reload with its Cancel handle usable, the legacy Run button was disabled for the whole run and re-enabled after the cancel, and the server persisted (same pid) throughout. **Trace:** [Audit FE-16](docs/history/localmotive-comprehensive-audit.md#fe-16).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1544).

<a id="criterion-v06-fe-16-v2"></a>
- [x] **V06-FE-16.V2** — Resolve a delayed status poll after stop/start and assert it cannot replace the newer server snapshot; verify polling remains single-flight under slow native responses. **Trace:** [Audit FE-16](docs/history/localmotive-comprehensive-audit.md#fe-16).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1545).

<a id="criterion-v06-fe-16-v3"></a>
- [x] **V06-FE-16.V3** — Edit a draft while the server runs, then simulate unexpected exit; assert running identity was unchanged by edits and final logs/failure evidence remain visible. **Closed 2026-09-12 (packaged, `scripts/g05_fe16.mjs`):** the draft rename applied in the editor while the STRATEGY/PROCESS/ENDPOINT instruments and the snapshot strategy stayed identical; after a taskkill of the child the state went non-LIVE with zero processes, the persisted `*.log.failure.json` beside the run log carried `phase=runtime_exit`, `exitCode=1` and a 1313-character tail, and the Control log well kept the last bounded output under the stopped note; restart reached LIVE. **Also fixed:** the empty-state copy rendered a refactor placeholder (`Start this props.profile…`); the well now retains the tail after any stop (component test `src/screens/DashboardScreen.test.tsx`, mutants MW1/MW2 caught). **Trace:** [Audit FE-16](docs/history/localmotive-comprehensive-audit.md#fe-16).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1546).

### V06-FE-17

<a id="criterion-v06-fe-17-i1"></a>
- [x] **V06-FE-17.I1** — Use the tested defaultWorkload factory in the production evidence panel instead of maintaining a separate literal with the same intended values. **Trace:** [Audit FE-17](docs/history/localmotive-comprehensive-audit.md#fe-17).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1563).

<a id="criterion-v06-fe-17-i2"></a>
- [x] **V06-FE-17.I2** — Validate editable workload drafts through the production-used helper or a native validation endpoint before dispatch, surfacing field-level errors instead of relying only on input attributes. **Trace:** [Audit FE-17](docs/history/localmotive-comprehensive-audit.md#fe-17).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1564).

<a id="criterion-v06-fe-17-i3"></a>
- [x] **V06-FE-17.I3** — Require finite integer values where Rust uses integer types and expose complete applicable minima/maxima; preserve an editable blank or partial draft until it can be committed safely. **Trace:** [Audit FE-17](docs/history/localmotive-comprehensive-audit.md#fe-17).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1565).

<a id="criterion-v06-fe-17-i4"></a>
- [x] **V06-FE-17.I4** — Add representative Rust-to-TypeScript contract fixtures or generated schema checks for persisted/IPC workload and correctness-sensitive profile fields, avoiding unsupported parity assumptions. **Trace:** [Audit FE-17](docs/history/localmotive-comprehensive-audit.md#fe-17).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1566).

<a id="criterion-v06-fe-17-i5"></a>
- [x] **V06-FE-17.I5** — Review duplicated fit/default/companion decision paths against the architecture's native-authority rule and document the selected authoritative boundary before consolidating them. **Trace:** [Audit FE-17](docs/history/localmotive-comprehensive-audit.md#fe-17).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1567).

<a id="criterion-v06-fe-17-v1"></a>
- [x] **V06-FE-17.V1** — Exercise default factory values through the actual component and IPC payload, then test fractional, negative, blank, nonfinite and over-limit numeric drafts. **Trace:** [Audit FE-17](docs/history/localmotive-comprehensive-audit.md#fe-17).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1571).

<a id="criterion-v06-fe-17-v2"></a>
- [x] **V06-FE-17.V2** — Assert an invalid form dispatches no benchmark command, and separately invoke malformed native inputs to prove backend rejection remains in place. **Trace:** [Audit FE-17](docs/history/localmotive-comprehensive-audit.md#fe-17).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1572).

<a id="criterion-v06-fe-17-v3"></a>
- [x] **V06-FE-17.V3** — Run representative schema/default round trips and supported older-profile normalization fixtures; change a contract fixture deliberately to establish that the parity check catches drift. **Trace:** [Audit FE-17](docs/history/localmotive-comprehensive-audit.md#fe-17).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1573).

### V06-IPC-01

<a id="criterion-v06-ipc-01-i1"></a>
- [x] **V06-IPC-01.I1** — Define an operation ID and explicit starting, running, stopping, failed and cancelled states. Reserve ownership under a short lock and publish correlated progress before launching. **Trace:** [Audit IPC-01](docs/history/localmotive-comprehensive-audit.md#ipc-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1592).

<a id="criterion-v06-ipc-01-i2"></a>
- [x] **V06-IPC-01.I2** — Move blocking filesystem, hashing, subprocess and socket work to a blocking worker. Keep the child handle and cancellation signal reachable by Stop while readiness is pending; do not hold the server mutex across the 600-second health wait. **Trace:** [Audit IPC-01](docs/history/localmotive-comprehensive-audit.md#ipc-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1593).

<a id="criterion-v06-ipc-01-i3"></a>
- [x] **V06-IPC-01.I3** — Commit completion only if the operation ID still owns the slot. On cancellation, failure or window close, terminate and reap the contained process tree before reporting a terminal state or allowing a replacement start. **Trace:** [Audit IPC-01](docs/history/localmotive-comprehensive-audit.md#ipc-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1594).

<a id="criterion-v06-ipc-01-i4"></a>
- [x] **V06-IPC-01.I4** — Classify every synchronous command named in the audit by actual cost, including preview, inspection, health, scanning, GGUF, preflight, legacy benchmark and replay. Move expensive work off both the Tauri main thread and async executor without weakening backend validation. **Trace:** [Audit IPC-01](docs/history/localmotive-comprehensive-audit.md#ipc-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1595).

<a id="criterion-v06-ipc-01-v1"></a>
- [x] **V06-IPC-01.V1** — Use a benign fixture that binds the expected port but never becomes ready. In the packaged Windows application, verify responsive controls and starting status, then measure Stop-to-process-exit latency against a documented deadline. **Trace:** [Audit IPC-01](docs/history/localmotive-comprehensive-audit.md#ipc-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1599).

<a id="criterion-v06-ipc-01-v2"></a>
- [x] **V06-IPC-01.V2** — Exercise slow --help, unreadable GGUF, early child exit and window close during startup. Confirm child/listener cleanup, truthful terminal state and a successful subsequent start. **Trace:** [Audit IPC-01](docs/history/localmotive-comprehensive-audit.md#ipc-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1600).

<a id="criterion-v06-ipc-01-v3"></a>
- [x] **V06-IPC-01.V3** — Force a late completion from an older operation after a new request and prove it cannot publish or clear the new operation's state. **Trace:** [Audit IPC-01](docs/history/localmotive-comprehensive-audit.md#ipc-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1601).

### V06-IPC-02

<a id="criterion-v06-ipc-02-i1"></a>
- [x] **V06-IPC-02.I1** — Inventory resources and IPC origins used by a packaged build and derive an explicit production CSP for bundled scripts, styles, fonts and images. Keep development-only exceptions out of production. **Trace:** [Audit IPC-02](docs/history/localmotive-comprehensive-audit.md#ipc-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1618).

<a id="criterion-v06-ipc-02-i2"></a>
- [x] **V06-IPC-02.I2** — Review opener permission and custom-command reachability alongside the policy; retain only origins and operations required by actual application flows. **Trace:** [Audit IPC-02](docs/history/localmotive-comprehensive-audit.md#ipc-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1619).

<a id="criterion-v06-ipc-02-i3"></a>
- [x] **V06-IPC-02.I3** — Document the purpose of each necessary policy exception and keep catalog, model, log and provider text rendered as text rather than HTML. **Trace:** [Audit IPC-02](docs/history/localmotive-comprehensive-audit.md#ipc-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1620).

<a id="criterion-v06-ipc-02-v1"></a>
- [x] **V06-IPC-02.V1** — Exercise every screen, dialog, icon, external URL action and inference WebUI navigation in the packaged application with CSP enabled. **Trace:** [Audit IPC-02](docs/history/localmotive-comprehensive-audit.md#ipc-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1624).

<a id="criterion-v06-ipc-02-v2"></a>
- [x] **V06-IPC-02.V2** — Inject harmless hostile-text fixtures and deliberately disallowed inline/remote script requests; verify text remains inert and scripts are blocked without adding a broad unsafe-eval workaround. **Trace:** [Audit IPC-02](docs/history/localmotive-comprehensive-audit.md#ipc-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1625).

### V06-CLD-01

<a id="criterion-v06-cld-01-i1"></a>
- [x] **V06-CLD-01.I1** — Set request-line, code-length and concurrent-login limits. Enforce one monotonic end-to-end deadline through accept, read, parsing and exchange paths, including a steady byte trickle or successive stray connections. **Trace:** [Audit CLD-01](docs/history/localmotive-comprehensive-audit.md#cld-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1642).

<a id="criterion-v06-cld-01-i2"></a>
- [x] **V06-CLD-01.I2** — Accept only the intended method and callback path. Use a URL/query parser with percent decoding; reject duplicate/empty codes and malformed encodings, and handle harmless probes without prematurely terminating a valid login. **Trace:** [Audit CLD-01](docs/history/localmotive-comprehensive-audit.md#cld-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1643).

<a id="criterion-v06-cld-01-i3"></a>
- [x] **V06-CLD-01.I3** — Retain loopback binding, ephemeral ports and S256 PKCE. Add state binding only after confirming the provider's supported contract; do not replace PKCE with state. **Trace:** [Audit CLD-01](docs/history/localmotive-comprehensive-audit.md#cld-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1644).

<a id="criterion-v06-cld-01-i4"></a>
- [x] **V06-CLD-01.I4** — Show callback-received wording until exchange and Credential Manager storage actually succeed. Report exchange/storage failure in the application without leaving a false connected indication in the browser. **Trace:** [Audit CLD-01](docs/history/localmotive-comprehensive-audit.md#cld-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1645).

<a id="criterion-v06-cld-01-v1"></a>
- [x] **V06-CLD-01.V1** — Test valid and percent-encoded codes, favicon requests, wrong method/path, duplicate/empty code, malformed encoding and overlong lines against a local callback listener. **Trace:** [Audit CLD-01](docs/history/localmotive-comprehensive-audit.md#cld-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1649).

<a id="criterion-v06-cld-01-v2"></a>
- [x] **V06-CLD-01.V2** — Test half-open connections, slow byte trickles and repeated stray requests until the overall deadline. Assert bounded memory, bounded completion time and release of listener resources. **Trace:** [Audit CLD-01](docs/history/localmotive-comprehensive-audit.md#cld-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1650).

<a id="criterion-v06-cld-01-v3"></a>
- [x] **V06-CLD-01.V3** — Stub remote exchange and secure storage to fail independently; prove neither failure reports connected and that a subsequent login can start. Use synthetic credentials. **Trace:** [Audit CLD-01](docs/history/localmotive-comprehensive-audit.md#cld-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1651).

### V06-OPS-01

<a id="criterion-v06-ops-01-i1"></a>
- [x] **V06-OPS-01.I1** — Use a unique run ID for each normal and tuning log and safely create files in user-only storage, rejecting unexpected existing links or cross-instance filename collisions. **Trace:** [Audit OPS-01](docs/history/localmotive-comprehensive-audit.md#ops-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1668).

<a id="criterion-v06-ops-01-i2"></a>
- [x] **V06-OPS-01.I2** — Introduce a bounded sink or rotation with explicit per-run and total retention limits. Keep drain behavior from blocking the child, and preserve the current bounded UI tail. **Trace:** [Audit OPS-01](docs/history/localmotive-comprehensive-audit.md#ops-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1669).

<a id="criterion-v06-ops-01-i3"></a>
- [x] **V06-OPS-01.I3** — Capture the final failure tail and run identity in structured evidence before retention cleanup. Provide explicit diagnostic export and document location, lifetime and possible runtime-emitted sensitive text. **Trace:** [Audit OPS-01](docs/history/localmotive-comprehensive-audit.md#ops-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1670).

<a id="criterion-v06-ops-01-v1"></a>
- [x] **V06-OPS-01.V1** — Run a fixture that emits more than the configured quota; assert bounded retained disk use, continued child progress and availability of the newest diagnostic lines. **Trace:** [Audit OPS-01](docs/history/localmotive-comprehensive-audit.md#ops-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1674).

<a id="criterion-v06-ops-01-v2"></a>
- [x] **V06-OPS-01.V2** — Start a second run and simultaneous application instance; verify logs cannot overwrite another run's failure evidence. Exercise cleanup, failed log creation and unexpected link targets. **Trace:** [Audit OPS-01](docs/history/localmotive-comprehensive-audit.md#ops-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1675).

### V06-GH-01

<a id="criterion-v06-gh-01-i1"></a>
- [x] **V06-GH-01.I1** — Add pull-request checks for the main branch on GitHub-hosted Windows or an ephemeral isolated runner, with stable check names and the applicable type, test, build, dependency and release-policy checks; retain trusted main-push verification. **Trace:** [Audit GH-01](docs/history/localmotive-comprehensive-audit.md#gh-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1696).

<a id="criterion-v06-gh-01-i2"></a>
- [x] **V06-GH-01.I2** — Keep fork and other untrusted pull-request code off the owner's interactive self-hosted runner. Explicitly separate the PR execution environment from trusted packaging and publication permissions. **Trace:** [Audit GH-01](docs/history/localmotive-comprehensive-audit.md#gh-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1697).

<a id="criterion-v06-gh-01-i3"></a>
- [x] **V06-GH-01.I3** — Configure a main-branch ruleset requiring the newly available PR checks, restricting direct and force pushes, and documenting a workable solo-maintainer review and emergency-bypass policy; introduce the trigger before requiring its status. **Trace:** [Audit GH-01](docs/history/localmotive-comprehensive-audit.md#gh-01).
  **Current status: VERIFIED.** E04: effective main ruleset plus the solo-maintainer policy below; configuration complete. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1698).

<a id="criterion-v06-gh-01-i4"></a>
- [x] **V06-GH-01.I4** — Update AGENTS.md and workflow comments to describe actual push/PR behavior, runner eligibility and the difference between job dependency gates and repository merge enforcement. **Trace:** [Audit GH-01](docs/history/localmotive-comprehensive-audit.md#gh-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1699).

<a id="criterion-v06-gh-01-v1"></a>
- [x] **V06-GH-01.V1** — Use a benign PR to confirm the expected checks run before merge, report stable names and are executed only on the intended isolated runner. **Trace:** [Audit GH-01](docs/history/localmotive-comprehensive-audit.md#gh-01).
  **Current status: VERIFIED.** E03: hosted PR18 pr-check passed before merge; self-hosted jobs skipped. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1703).

<a id="criterion-v06-gh-01-v2"></a>
- [ ] **V06-GH-01.V2** — Introduce a controlled failing check in the test PR; verify merge is blocked until corrected, then confirm the corrected commit obtains the required successful statuses. **Trace:** [Audit GH-01](docs/history/localmotive-comprehensive-audit.md#gh-01).
  **Current status: OPEN.** U06-05: controlled failing PR, observed merge block and green recovery evidence still required. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1704).

<a id="criterion-v06-gh-01-v3"></a>
- [ ] **V06-GH-01.V3** — Read back effective branch/ruleset settings with appropriate access, record bypass actors and restrictions, and verify ordinary contributors cannot circumvent required checks through direct pushes. **Trace:** [Audit GH-01](docs/history/localmotive-comprehensive-audit.md#gh-01).
  **Current status: OPEN.** U06-05: effective rules readback exists; ordinary-contributor noncircumvention proof remains unconfirmed. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1705).

### V06-GH-02

<a id="criterion-v06-gh-02-i1"></a>
- [x] **V06-GH-02.I1** — Resolve the requested annotated or lightweight release tag to one full commit SHA once, validate its main ancestry and event relationship, and pass that immutable revision to audit, quality, package and publish checkouts. **Trace:** [Audit GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1722).

<a id="criterion-v06-gh-02-i2"></a>
- [x] **V06-GH-02.I2** — Record the resolved source revision with packaged behavior evidence and the producer's candidate inventory; verify these records and exact artifact hashes at publication instead of regenerating inventory from the publish job's current HEAD. **Trace:** [Audit GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1723).

<a id="criterion-v06-gh-02-i3"></a>
- [ ] **V06-GH-02.I3** — Protect released version tags against updates and deletion, document one-time tag creation, and require a new prerelease or patch version when source changes after an earlier candidate. **Trace:** [Audit GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02).
  **Current status: OPEN.** PARTIAL / U06-05 and U06-06: active immutable-tag settings accepted; tag-history/final-identity policy disposition remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1724).

<a id="criterion-v06-gh-02-i4"></a>
- [x] **V06-GH-02.I4** — Preserve historical release artifacts and corrective records. Document retries of unchanged source separately from releases containing new source, with an explicit identity mismatch failure before publication. **Trace:** [Audit GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1725).

<a id="criterion-v06-gh-02-v1"></a>
- [x] **V06-GH-02.V1** — Simulate moving the tag between quality, package and publish in isolated repository/workflow fixtures; every job must retain the resolved SHA or fail before promoting mismatched artifacts. **Trace:** [Audit GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1729).

<a id="criterion-v06-gh-02-v2"></a>
- [x] **V06-GH-02.V2** — Supply inventories or packaged records with a different source SHA, missing artifact or changed digest; verify publication validation rejects each case without overwriting the original evidence. **Trace:** [Audit GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1730).

<a id="criterion-v06-gh-02-v3"></a>
- [ ] **V06-GH-02.V3** — Run a complete candidate flow and compare checkout revisions, inventory, packaged evidence and publication metadata; read back the effective tag-update/deletion protection. **Trace:** [Audit GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02).
  **Current status: OPEN.** U06-02/U06-06/U06-08: full candidate identities and publication metadata must agree. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1731).

### V06-GH-03

<a id="criterion-v06-gh-03-i1"></a>
- [x] **V06-GH-03.I1** — Make clean-account lifecycle verification consume the already-built candidate installers through CandidateDir as a dependent release job, preserving the producer's source revision, inventory and expected installer hashes. **Trace:** [Audit GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1748).

<a id="criterion-v06-gh-03-i2"></a>
- [x] **V06-GH-03.I2** — Remove the independent prepublication wait for public release assets from the one-runner release path; do not occupy the only producer runner with a consumer waiting for artifacts it cannot yet produce. **Trace:** [Audit GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1749).

<a id="criterion-v06-gh-03-i3"></a>
- [x] **V06-GH-03.I3** — Define whether lifecycle verification is required before publication. If policy deliberately chooses a non-gating post-release check instead, trigger it only after completed publication and present its untested or failed status explicitly. **Trace:** [Audit GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1750).

<a id="criterion-v06-gh-03-i4"></a>
- [x] **V06-GH-03.I4** — Correct the historical v0.5 ledger explanation with a factual additive correction: Sandbox availability succeeded; the demonstrated failure was polling unpublished installers. Update workflow comments and documented scheduling to match the chosen sequence. **Trace:** [Audit GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1751).

<a id="criterion-v06-gh-03-v1"></a>
- [ ] **V06-GH-03.V1** — Exercise a fresh-version release in a single-runner configuration; the lifecycle job must start only after candidate artifacts exist, without release-not-found polling blocking the producer. **Trace:** [Audit GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: OPEN.** U06-02/U06-04/U06-06: complete final producer chain required. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1755).

<a id="criterion-v06-gh-03-v2"></a>
- [x] **V06-GH-03.V2** — Provide missing or mismatched candidate installers and confirm verification fails with an explicit identity error rather than attempting installation or reporting PASS. **Trace:** [Audit GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1756).

<a id="criterion-v06-gh-03-v3"></a>
- [ ] **V06-GH-03.V3** — Run the selected lifecycle path against the exact candidate bytes and confirm publication behavior matches the documented required/non-gating policy, including failure and cancellation outcomes. **Trace:** [Audit GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: OPEN.** U06-04/U06-06: native lifecycle and observed failure/cancellation gating required. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1757).

### V06-GH-04

<a id="criterion-v06-gh-04-i1"></a>
- [x] **V06-GH-04.I1** — Replace the MSI leftover-executable warning-and-continue branch with a failed uninstall verdict when the expected application remains; verify the intended installation scope and registry/product identity instead of relying on a generic executable search. **Trace:** [Audit GH-04](docs/history/localmotive-comprehensive-audit.md#gh-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1774).

<a id="criterion-v06-gh-04-i2"></a>
- [x] **V06-GH-04.I2** — After an upgrade, verify the installed executable's expected version and digest against the candidate inventory, and retain evidence identifying old and new installer/executable versions. **Trace:** [Audit GH-04](docs/history/localmotive-comprehensive-audit.md#gh-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1775).

<a id="criterion-v06-gh-04-i3"></a>
- [x] **V06-GH-04.I3** — Add separate migration scenarios for v0.4.1 profiles and settings and for v0.5.0 SQLite/user-override data. Populate realistic isolated persisted fixtures before upgrade and assert the documented preservation or migration behavior afterward. **Trace:** [Audit GH-04](docs/history/localmotive-comprehensive-audit.md#gh-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1776).

<a id="criterion-v06-gh-04-i4"></a>
- [x] **V06-GH-04.I4** — Cover appropriate NSIS and MSI fresh-install, upgrade and uninstall paths; replace the permanently hardcoded v0.4.0 baseline with an explicit supported-baseline matrix, and label eight-second process survival as startup smoke rather than full functional verification. **Trace:** [Audit GH-04](docs/history/localmotive-comprehensive-audit.md#gh-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1777).

<a id="criterion-v06-gh-04-v1"></a>
- [x] **V06-GH-04.V1** — Inject an MSI uninstall outcome that leaves the expected executable or product registration; prove the scenario cannot emit uninstall PASS. **Trace:** [Audit GH-04](docs/history/localmotive-comprehensive-audit.md#gh-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1781).

<a id="criterion-v06-gh-04-v2"></a>
- [x] **V06-GH-04.V2** — Simulate an upgrade that keeps the old executable despite a successful installer exit code; verify the version/digest assertion catches it. **Trace:** [Audit GH-04](docs/history/localmotive-comprehensive-audit.md#gh-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1782).

<a id="criterion-v06-gh-04-v3"></a>
- [ ] **V06-GH-04.V3** — Run clean-account installer scenarios and both version-specific migration fixtures on Windows; compare exact installed identity, retained data, uninstall outcomes and structured per-scenario verdicts. **Trace:** [Audit GH-04](docs/history/localmotive-comprehensive-audit.md#gh-04).
  **Current status: OPEN.** R06-01 / U06-04: corrected current native installer and preservation observations remain required. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1783).

### V06-GH-05

<a id="criterion-v06-gh-05-i1"></a>
- [x] **V06-GH-05.I1** — Add a version-aware packaged verifier for the 0.6 candidate that exercises the HF model catalog and SQLite workflow through actual WebView events and IPC, using an isolated application-data profile and controlled signed catalog fixtures. **Trace:** [Audit GH-05](docs/history/localmotive-comprehensive-audit.md#gh-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1800).

<a id="criterion-v06-gh-05-i2"></a>
- [x] **V06-GH-05.I2** — Cover first fill, fresh-session restart within cooldown, offline cache availability, valid signed refresh, invalid-signature fallback and corrupt-mirror recovery, while preserving a clear separation between local loading and network refresh. **Trace:** [Audit GH-05](docs/history/localmotive-comprehensive-audit.md#gh-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1801).

<a id="criterion-v06-gh-05-i3"></a>
- [x] **V06-GH-05.I3** — Exercise rich filter/facet and hardware-fit controls, catalog navigation or pagination where implemented, local user-row persistence/removal, refresh/cooldown feedback and visible bounds/error states through the shipped UI. **Trace:** [Audit GH-05](docs/history/localmotive-comprehensive-audit.md#gh-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1802).

<a id="criterion-v06-gh-05-i4"></a>
- [x] **V06-GH-05.I4** — Include the verifier in the release gate and bind its report to the immutable source and candidate binary digests. Retain the existing runtime verifier but identify injected presentation checks, real catalog interactions and CPU/runtime checks as distinct evidence. **Trace:** [Audit GH-05](docs/history/localmotive-comprehensive-audit.md#gh-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1803).

<a id="criterion-v06-gh-05-v1"></a>
- [x] **V06-GH-05.V1** — Demonstrate that the new packaged scenarios fail against the specific catalog restart, corruption and override defects they are intended to detect, then rerun them on their corrected implementation. **Trace:** [Audit GH-05](docs/history/localmotive-comprehensive-audit.md#gh-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1807).

<a id="criterion-v06-gh-05-v2"></a>
- [x] **V06-GH-05.V2** — Run the packaged catalog matrix from an empty profile and a persisted prior-version profile; verify both UI-visible results and actual backend/disk persistence across application restarts. **Trace:** [Audit GH-05](docs/history/localmotive-comprehensive-audit.md#gh-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1808).

<a id="criterion-v06-gh-05-v3"></a>
- [x] **V06-GH-05.V3** — Corrupt a signature and mirror, reject an input and delay a refresh; assert honest fallback/error/cooldown output without replacing real orchestration with private React-state injection. **Trace:** [Audit GH-05](docs/history/localmotive-comprehensive-audit.md#gh-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1809).

### V06-GH-06

<a id="criterion-v06-gh-06-i1"></a>
- [x] **V06-GH-06.I1** — Move collection of Sandbox result JSON, lifecycle logs and relevant host diagnostics into a finally-style path that runs before propagating a failure; preserve the original process or scenario error. **Trace:** [Audit GH-06](docs/history/localmotive-comprehensive-audit.md#gh-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1826).

<a id="criterion-v06-gh-06-i2"></a>
- [x] **V06-GH-06.I2** — Write a structured FAIL, TIMEOUT or cancellation/missing-result record when Sandbox never produces usable output, including release version, source revision, candidate digests, timing and the failing stage. **Trace:** [Audit GH-06](docs/history/localmotive-comprehensive-audit.md#gh-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1827).

<a id="criterion-v06-gh-06-i3"></a>
- [x] **V06-GH-06.I3** — Make the workflow upload the expected evidence on every terminal outcome and surface a missing required evidence file instead of treating an ignored empty upload as successful verification. **Trace:** [Audit GH-06](docs/history/localmotive-comprehensive-audit.md#gh-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1828).

<a id="criterion-v06-gh-06-i4"></a>
- [x] **V06-GH-06.I4** — Update job summaries to distinguish scenario status, artifact-upload status and actual artifact existence; report missing evidence as missing and never infer a lifecycle PASS from a green upload step. **Trace:** [Audit GH-06](docs/history/localmotive-comprehensive-audit.md#gh-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1829).

<a id="criterion-v06-gh-06-v1"></a>
- [x] **V06-GH-06.V1** — Inject a Sandbox FAIL result and verify the job remains failed while its exact result JSON and diagnostic log are retained in the uploaded evidence. **Trace:** [Audit GH-06](docs/history/localmotive-comprehensive-audit.md#gh-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1833).

<a id="criterion-v06-gh-06-v2"></a>
- [x] **V06-GH-06.V2** — Exercise timeout, missing or malformed result, early installer-download failure and cancellation paths; each must leave a bounded structured outcome with the correct stage and candidate identity. **Closed 2026-09-12:** the harness's default-off fault simulations (`-FaultSimulation timeout|malformed-result|missing-assets|stall`) are driven by `scripts/sandbox/test-fault-evidence.ps1`; the witness files bind source revision and candidate digests, statuses are TIMEOUT at `sandbox-timeout` / FAIL at `sandbox-run` / FAIL at `resolve-installers`, and the cancellation leg proves a killed run leaves no PASS artifact (mutant MG1 caught — the harness's own catch guard converts a relabelled PASS to FAIL). **Trace:** [Audit GH-06](docs/history/localmotive-comprehensive-audit.md#gh-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1834).

<a id="criterion-v06-gh-06-v3"></a>
- [ ] **V06-GH-06.V3** — Inspect the completed workflow's artifact collection, not just its upload-step conclusion, and confirm the summary accurately reports both verification outcome and evidence availability. **Trace:** [Audit GH-06](docs/history/localmotive-comprehensive-audit.md#gh-06).
  **Current status: OPEN.** U06-02/U06-06/U06-08: inspect actual retained/downloaded evidence and publication asset contents. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1835).

### V06-GH-07

<a id="criterion-v06-gh-07-i1"></a>
- [x] **V06-GH-07.I1** — Prepare the factual v0.4.0 public release correction from the existing corrective note: relabel the three L2/PARTIAL configurations as direct-runtime evidence and explain that packaged product qualification was not established; preserve its published binaries. **Trace:** [Audit GH-07](docs/history/localmotive-comprehensive-audit.md#gh-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1852).

<a id="criterion-v06-gh-07-i2"></a>
- [x] **V06-GH-07.I2** — Publish a compact version-to-host/backend/lifecycle evidence matrix with source/artifact references, distinguishing CPU packaged checks from CUDA/Vulkan support and from clean-account install, upgrade and uninstall results. **Trace:** [Audit GH-07](docs/history/localmotive-comprehensive-audit.md#gh-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1853).

<a id="criterion-v06-gh-07-i3"></a>
- [x] **V06-GH-07.I3** — Rewrite the README support section so each claimed status is conditional on evidence for that exact version and configuration; keep the actual v0.5 lifecycle result untested by that failed run rather than implying an installer failure. **Trace:** [Audit GH-07](docs/history/localmotive-comprehensive-audit.md#gh-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1854).

<a id="criterion-v06-gh-07-i4"></a>
- [x] **V06-GH-07.I4** — Correct stale runner schedule/hosting and push/PR CI descriptions to the currently verified configuration. Keep historical events attributable through dated corrections instead of rewriting old failure outcomes as later successes. **Trace:** [Audit GH-07](docs/history/localmotive-comprehensive-audit.md#gh-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1855).

<a id="criterion-v06-gh-07-v1"></a>
- [x] **V06-GH-07.V1** — Compare every matrix support or PASS label against the linked version, source, binary and scenario record; fail documentation review when host presence or L2 evidence substitutes for product qualification. **Trace:** [Audit GH-07](docs/history/localmotive-comprehensive-audit.md#gh-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1859).

<a id="criterion-v06-gh-07-v2"></a>
- [x] **V06-GH-07.V2** — Read back the affected public v0.4.0 release after an authorized correction and verify its former Supported wording no longer creates the false product-support claim. **Trace:** [Audit GH-07](docs/history/localmotive-comprehensive-audit.md#gh-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1860).

<a id="criterion-v06-gh-07-v3"></a>
- [x] **V06-GH-07.V3** — Check README, AGENTS and active workflow descriptions together for runner/trigger agreement and verify v0.5 CPU evidence remains distinct from missing accelerator and lifecycle qualification. **Trace:** [Audit GH-07](docs/history/localmotive-comprehensive-audit.md#gh-07).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1861).

### V06-GH-08

<a id="criterion-v06-gh-08-i1"></a>
- [x] **V06-GH-08.I1** — Pin or explicitly record the exact Rust and Node toolchains and relevant Windows build-image/tool versions so the immutable action commit is not confused with a reproducible compiler or runner environment. **Trace:** [Audit GH-08](docs/history/localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](docs/history/localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](docs/history/localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1878).

<a id="criterion-v06-gh-08-i2"></a>
- [x] **V06-GH-08.I2** — Produce a target-specific artifact-bound SBOM and third-party notices/license inventory from resolved package metadata and license texts, not Cargo.lock alone. Define the license-review policy and record unresolved metadata. Where feasible, add signed build provenance tied to the source revision and exact candidate hashes, stating what it attests and how to verify it. **Trace:** [Audit GH-08](docs/history/localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](docs/history/localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](docs/history/localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1879).

<a id="criterion-v06-gh-08-i3"></a>
- [x] **V06-GH-08.I3** — Separate trusted release build/publication credentials and environment from routine interactive owner state, preserving least-privilege workflow tokens and avoiding any untrusted PR execution on the personal runner. **Trace:** [Audit GH-08](docs/history/localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](docs/history/localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](docs/history/localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1880).

<a id="criterion-v06-gh-08-i4"></a>
- [x] **V06-GH-08.I4** — Define retention and recovery for candidate inventories, packaged/lifecycle evidence, readback artifacts and supporting logs beyond current ephemeral windows; document signing-key rotation/recovery without storing keys in the repository. **Trace:** [Audit GH-08](docs/history/localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](docs/history/localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](docs/history/localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1881).

<a id="criterion-v06-gh-08-i5"></a>
- [x] **V06-GH-08.I5** — Retain honest unsigned-installer disclosure and distinguish application Authenticode, Git tag identity, catalog Ed25519 signatures, build provenance and checksums as different controls. **Trace:** [Audit GH-08](docs/history/localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](docs/history/localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](docs/history/localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1882).

<a id="criterion-v06-gh-08-v1"></a>
- [x] **V06-GH-08.V1** — Reproduce the recorded toolchain selection from a clean eligible build environment and compare source, compiler, dependency lockfiles and environment metadata in the retained record. **Trace:** [Audit GH-08](docs/history/localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](docs/history/localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](docs/history/localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1886).

<a id="criterion-v06-gh-08-v2"></a>
- [x] **V06-GH-08.V2** — Verify the SBOM, license/notice inventory and any provenance against the candidate source, resolved Windows dependency graph and artifact digests; identify missing license information explicitly and confirm substituted artifacts or identities fail validation. **Trace:** [Audit GH-08](docs/history/localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](docs/history/localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](docs/history/localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1887).

<a id="criterion-v06-gh-08-v3"></a>
- [x] **V06-GH-08.V3** — Exercise evidence retrieval/recovery under the chosen retention policy and inspect effective job permissions and environment separation without exposing owner credentials. **Trace:** [Audit GH-08](docs/history/localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](docs/history/localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](docs/history/localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1888).

### V06-GH-09

<a id="criterion-v06-gh-09-i1"></a>
- [x] **V06-GH-09.I1** — Add a concise security reporting/contact policy and contributor setup/support guidance that explains supported scope, safe reproduction details, response ownership and how to avoid posting credentials or private diagnostics. **Trace:** [Audit GH-09](docs/history/localmotive-comprehensive-audit.md#gh-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1905).

<a id="criterion-v06-gh-09-i2"></a>
- [x] **V06-GH-09.I2** — Introduce lightweight issue/PR templates and ownership guidance suitable for a solo-maintainer project; record an accountable owner, priority, acceptance criteria and evidence location for each unresolved audit item. **Trace:** [Audit GH-09](docs/history/localmotive-comprehensive-audit.md#gh-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1906).

<a id="criterion-v06-gh-09-i3"></a>
- [x] **V06-GH-09.I3** — Configure automatic dependency-update proposals and scheduled dependency audits in a safe hosted or isolated environment; document how vulnerability, maintenance and unsoundness warnings are triaged separately and how target relevance is established. **Trace:** [Audit GH-09](docs/history/localmotive-comprehensive-audit.md#gh-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1907).

<a id="criterion-v06-gh-09-i4"></a>
- [x] **V06-GH-09.I4** — Inspect accessible current repository security/alert settings with appropriate authorization and record their actual status instead of inferring disabled services from absent files. Define any useful SAST or license-policy follow-up without duplicating the release SBOM work. **Trace:** [Audit GH-09](docs/history/localmotive-comprehensive-audit.md#gh-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1908).

<a id="criterion-v06-gh-09-i5"></a>
- [x] **V06-GH-09.I5** — Document backup and recovery responsibilities for the build runner, release evidence and signing assets; keep private recovery material outside public source and avoid process requirements that the maintainer cannot realistically sustain. **Trace:** [Audit GH-09](docs/history/localmotive-comprehensive-audit.md#gh-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1909).

<a id="criterion-v06-gh-09-v1"></a>
- [x] **V06-GH-09.V1** — Walk through a synthetic security report and contributor issue to verify that contact, ownership, required reproduction information and next-step expectations are clear without disclosing secrets. **Trace:** [Audit GH-09](docs/history/localmotive-comprehensive-audit.md#gh-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1913).

<a id="criterion-v06-gh-09-v2"></a>
- [x] **V06-GH-09.V2** — Confirm a controlled dependency-update proposal and scheduled audit run produce actionable results, preserve nonzero blocking failures and route informational warnings to an owned follow-up. **Trace:** [Audit GH-09](docs/history/localmotive-comprehensive-audit.md#gh-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1914).

<a id="criterion-v06-gh-09-v3"></a>
- [x] **V06-GH-09.V3** — Review effective settings and recovery instructions with the responsible maintainer; verify recorded task ownership and closure evidence can be located from the project's documentation. **Trace:** [Audit GH-09](docs/history/localmotive-comprehensive-audit.md#gh-09).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1915).

### V06-GH-10

<a id="criterion-v06-gh-10-i1"></a>
- [x] **V06-GH-10.I1** — Replace unconditional HOST_MATCH rows with per-row results derived from the detected CPU/GPU identity and the qualification job's stated expected host; represent MATCH, NO_MATCH and UNKNOWN explicitly. **Trace:** [Audit GH-10](docs/history/localmotive-comprehensive-audit.md#gh-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1932).

<a id="criterion-v06-gh-10-i2"></a>
- [x] **V06-GH-10.I2** — Treat a definite mismatch as a failed intended host-proof job and preserve UNKNOWN when identification is incomplete; do not promote a warning about mismatched hardware into a successful match record. **Trace:** [Audit GH-10](docs/history/localmotive-comprehensive-audit.md#gh-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1933).

<a id="criterion-v06-gh-10-i3"></a>
- [x] **V06-GH-10.I3** — Include actual source revision, runtime identity and driver/hardware observations needed to interpret the host evidence, keeping observed values separate from expected runner labels or a hardcoded runner name. **Trace:** [Audit GH-10](docs/history/localmotive-comprehensive-audit.md#gh-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1934).

<a id="criterion-v06-gh-10-i4"></a>
- [x] **V06-GH-10.I4** — Retain and strengthen the explicit policy that host presence does not establish packaged L4 product qualification. Report CPU, CUDA and Vulkan qualification only through the separate matching packaged scenario evidence, not through static host rows. **Trace:** [Audit GH-10](docs/history/localmotive-comprehensive-audit.md#gh-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1935).

<a id="criterion-v06-gh-10-v1"></a>
- [x] **V06-GH-10.V1** — Feed the host-record generator matching CPU/GPU observations and confirm only the corresponding expected host rows are marked matched, with the actual observations retained. **Trace:** [Audit GH-10](docs/history/localmotive-comprehensive-audit.md#gh-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1939).

<a id="criterion-v06-gh-10-v2"></a>
- [x] **V06-GH-10.V2** — Exercise mismatched CPU, mismatched GPU, missing/ambiguous GPU and stale-runner-label fixtures; verify none emits an unsupported HOST_MATCH and the intended host-proof outcome reflects the mismatch or uncertainty. **Trace:** [Audit GH-10](docs/history/localmotive-comprehensive-audit.md#gh-10).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1940).

<a id="criterion-v06-gh-10-v3"></a>
- [ ] **V06-GH-10.V3** — Run on the intended Windows host and compare emitted identity fields to directly observed hardware/driver/source values; verify no host-only record claims successful CUDA/Vulkan inference or complete L4 support. **Trace:** [Audit GH-10](docs/history/localmotive-comprehensive-audit.md#gh-10).
  **Current status: OPEN.** E07 / U06-03: current host-attestation workflow fails to parse; historical generator tests stay accepted. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1941).

### V06-QD-01

<a id="criterion-v06-qd-01-i1"></a>
- [x] **V06-QD-01.I1** — Replace the literal September 10, 2026 cutoff reference with the current UTC date by default, and decide whether to expose an explicit --as-of date for reproducible catalog snapshots. **Trace:** [Audit QD-01](docs/history/localmotive-comprehensive-audit.md#qd-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1960).

<a id="criterion-v06-qd-01-i2"></a>
- [x] **V06-QD-01.I2** — Compute the advertised update date, effective reference date and cutoff date from the same validated clock input; retain the effective reference and cutoff in publish evidence. **Trace:** [Audit QD-01](docs/history/localmotive-comprehensive-audit.md#qd-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1961).

<a id="criterion-v06-qd-01-i3"></a>
- [x] **V06-QD-01.I3** — Extract date selection and discovery/deduplication decisions into importable helpers so deterministic tests exercise production behavior without live Hub requests or replacing the signed catalog. **Trace:** [Audit QD-01](docs/history/localmotive-comprehensive-audit.md#qd-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1962).

<a id="criterion-v06-qd-01-i4"></a>
- [x] **V06-QD-01.I4** — Document the actual per-author cap, the meaning of --full and the first-page limit; state whether pagination is intentionally omitted instead of implying an exhaustive allowlist scan. **Trace:** [Audit QD-01](docs/history/localmotive-comprehensive-audit.md#qd-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1963).

<a id="criterion-v06-qd-01-v1"></a>
- [x] **V06-QD-01.V1** — Reproduce the audit fixture with a December 10, 2026 clock and a July 1 model; assert the model is excluded by the 90-day policy and the emitted dates agree. **Trace:** [Audit QD-01](docs/history/localmotive-comprehensive-audit.md#qd-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1967).

<a id="criterion-v06-qd-01-v2"></a>
- [x] **V06-QD-01.V2** — Cover exact cutoff boundaries, future timestamps, malformed lastModified, leap days and repeated builds with the same explicit reference date. **Trace:** [Audit QD-01](docs/history/localmotive-comprehensive-audit.md#qd-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1968).

<a id="criterion-v06-qd-01-v3"></a>
- [x] **V06-QD-01.V3** — Verify bounded discovery and duplicate selection with controlled metadata, then run the catalog validator on a generated fixture while confirming repository catalog/signature bytes remain unchanged. **Trace:** [Audit QD-01](docs/history/localmotive-comprehensive-audit.md#qd-01).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1969).

### V06-QD-02

<a id="criterion-v06-qd-02-i1"></a>
- [x] **V06-QD-02.I1** — Add a DOM/component test environment scoped to application tests, mocking IPC only at its boundary and preserving the exclusion of research and third-party tests. **Trace:** [Audit QD-02](docs/history/localmotive-comprehensive-audit.md#qd-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1986).

<a id="criterion-v06-qd-02-i2"></a>
- [x] **V06-QD-02.I2** — Add real temporary-database orchestration fixtures covering fetch, mirror publication, restart, cooldown, offline fallback, corrupt migration and download authorization, rather than calling only successful storage helpers. **Trace:** [Audit QD-02](docs/history/localmotive-comprehensive-audit.md#qd-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1987).

<a id="criterion-v06-qd-02-i3"></a>
- [x] **V06-QD-02.I3** — Exercise curated/local ID and filename collisions, atomic file replacement and preservation of local entries; write failing regressions before the corresponding catalog fixes are closed. **Trace:** [Audit QD-02](docs/history/localmotive-comprehensive-audit.md#qd-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1988).

<a id="criterion-v06-qd-02-i4"></a>
- [x] **V06-QD-02.I4** — Cover delayed/rejected IPC, filter reapplication, hardware updates, stale responses, malformed saved profiles, denied browser storage and accessible catalog/profile interactions. **Closed 2026-09-12:** `src/App.storage.test.tsx` (denied-origin matrix: runtime inspection, tuning report and benchmark saves keep the completed native result and surface bounded notices; first render survives a denied origin — this exposed and fixed a real crash in the disclosure initializer), catalog filter legs in `src/App.catalog.test.tsx` (slow-then-fast newest-wins, rejected filter keeps prior rows, null local-model snapshot tolerated), plus the stale-response suites above; mutants MC1/M-C/M-A caught. **Trace:** [Audit QD-02](docs/history/localmotive-comprehensive-audit.md#qd-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1989).

<a id="criterion-v06-qd-02-i5"></a>
- [x] **V06-QD-02.I5** — Extend version-aware packaged HF catalog acceptance to first run, restart, persistence, fallback, filters and actual serialization; distinguish these scenarios from the retained runtime-catalog checks. **Trace:** [Audit QD-02](docs/history/localmotive-comprehensive-audit.md#qd-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1990).

<a id="criterion-v06-qd-02-v1"></a>
- [x] **V06-QD-02.V1** — Demonstrate that regression cases fail for the audited boundary defects and pass only after their owning implementation tasks resolve them. **Trace:** [Audit QD-02](docs/history/localmotive-comprehensive-audit.md#qd-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1994).

<a id="criterion-v06-qd-02-v2"></a>
- [x] **V06-QD-02.V2** — Run the component suite and backend orchestration tests with delayed failures, corrupt databases and conflicting rows, retaining exact commands and results. **Trace:** [Audit QD-02](docs/history/localmotive-comprehensive-audit.md#qd-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1995).

<a id="criterion-v06-qd-02-v3"></a>
- [x] **V06-QD-02.V3** — Run the packaged Windows scenarios with an isolated profile and retain source/artifact-bound evidence; record unexecuted target-environment checks explicitly. **Trace:** [Audit QD-02](docs/history/localmotive-comprehensive-audit.md#qd-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1996).

<a id="criterion-v06-qd-02-v4"></a>
- [x] **V06-QD-02.V4** — Confirm keyboard navigation, labels, focus changes and status announcements on representative catalog/profile flows. **Trace:** [Audit QD-02](docs/history/localmotive-comprehensive-audit.md#qd-02).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L1997).

### V06-QD-03

<a id="criterion-v06-qd-03-i1"></a>
- [x] **V06-QD-03.I1** — Move synthetic loading, empty, error and rate-limit presentation scenarios into component tests using public React/DOM interfaces and a stable mocked backend boundary. **Trace:** [Audit QD-03](docs/history/localmotive-comprehensive-audit.md#qd-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2014).

<a id="criterion-v06-qd-03-i2"></a>
- [x] **V06-QD-03.I2** — Remove the packaged verifier's __reactFiber$, memoizedState, hook-shape discovery and queue.dispatch dependencies; replace setScenarioAndRefresh with actual user controls or explicitly labeled synthetic presentation setup. **Trace:** [Audit QD-03](docs/history/localmotive-comprehensive-audit.md#qd-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2015).

<a id="criterion-v06-qd-03-i3"></a>
- [x] **V06-QD-03.I3** — Keep genuine IPC, installation, tamper, health and restart checks intact, and describe separately what each check proves so synthetic rendering cannot be mistaken for a real request lifecycle. **Trace:** [Audit QD-03](docs/history/localmotive-comprehensive-audit.md#qd-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2016).

<a id="criterion-v06-qd-03-i4"></a>
- [x] **V06-QD-03.I4** — Replace the fixed 250-millisecond cancellation trigger with an observed active-operation/progress condition where available; retain bounded timeouts and useful diagnostics for missing transitions. **Trace:** [Audit QD-03](docs/history/localmotive-comprehensive-audit.md#qd-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2017).

<a id="criterion-v06-qd-03-i5"></a>
- [x] **V06-QD-03.I5** — Replace release-gate assertions that require private implementation strings with assertions about the verifier's observable outcomes and complete evidence records. **Trace:** [Audit QD-03](docs/history/localmotive-comprehensive-audit.md#qd-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2018).

<a id="criterion-v06-qd-03-v1"></a>
- [x] **V06-QD-03.V1** — Change hook ordering or introduce similarly shaped component state and verify the public-interface tests remain valid without adapting private-memory selectors. **Trace:** [Audit QD-03](docs/history/localmotive-comprehensive-audit.md#qd-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2022).

<a id="criterion-v06-qd-03-v2"></a>
- [x] **V06-QD-03.V2** — Inject delayed and failed backend responses through the test boundary; assert Refresh actually starts retrieval and reaches the expected terminal UI state. **Trace:** [Audit QD-03](docs/history/localmotive-comprehensive-audit.md#qd-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2023).

<a id="criterion-v06-qd-03-v3"></a>
- [x] **V06-QD-03.V3** — Exercise cancellation under fast and slow fixture progress, confirming it waits for an active operation and records completion or a bounded diagnostic failure. **Closed 2026-09-12:** one classifier (`scripts/lib/health_cancel.mjs`) owns the acceptance rules; `scripts/tests/health_cancel.test.mjs` drives fast (completed-before-cancel with the full passing contract), slow (accepted cancel with stage attribution), refused and bounded-timeout fixtures (mutant MQ1 caught); the packaged check (`scripts/verify_041.mjs`) now waits for an observed progress phase and records cancellation, completion or a bounded diagnostic through that classifier; `npm test` runs the fixture legs and the release gate pins the wiring. **Trace:** [Audit QD-03](docs/history/localmotive-comprehensive-audit.md#qd-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2024).

<a id="criterion-v06-qd-03-v4"></a>
- [x] **V06-QD-03.V4** — Run the preserved packaged IPC/install/tamper/health checks and inspect the evidence labels for a clear distinction between synthetic presentation and real execution. **Trace:** [Audit QD-03](docs/history/localmotive-comprehensive-audit.md#qd-03).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2025).

### V06-QD-04

<a id="criterion-v06-qd-04-i1"></a>
- [x] **V06-QD-04.I1** — Align active product/runtime documents with the actual desktop platform, local SQLite implementation, current version and standing unsigned-release policy; distinguish reusable runtime policy from version-specific qualification evidence. **Trace:** [Audit QD-04](docs/history/localmotive-comprehensive-audit.md#qd-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2042).

<a id="criterion-v06-qd-04-i2"></a>
- [x] **V06-QD-04.I2** — Add a storage matrix covering settings, signed JSON cache, SQLite/user rows, refresh stamps, managed runtimes and evidence stores, with accurate reset, export and backup behavior and any temp-directory fallback. **Trace:** [Audit QD-04](docs/history/localmotive-comprehensive-audit.md#qd-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2043).

<a id="criterion-v06-qd-04-i3"></a>
- [x] **V06-QD-04.I3** — Mark the September 6 qualification guide as a frozen snapshot or archive it with a current evidence index; correct current runnable version commands and the history path without rewriting past results. **Trace:** [Audit QD-04](docs/history/localmotive-comprehensive-audit.md#qd-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2044).

<a id="criterion-v06-qd-04-i4"></a>
- [x] **V06-QD-04.I4** — Reconcile contributor CI claims with actual triggers, document builder fail-the-whole-build behavior and --allow-empty, and record the schema-v1 compatibility decision plus old-client fallback expectations. **Trace:** [Audit QD-04](docs/history/localmotive-comprehensive-audit.md#qd-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2045).

<a id="criterion-v06-qd-04-i5"></a>
- [x] **V06-QD-04.I5** — Update module ownership/data-flow maps and present superseded v0.5 tracker decisions as dated history or beneath an authoritative current-status table. **Trace:** [Audit QD-04](docs/history/localmotive-comprehensive-audit.md#qd-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2046).

<a id="criterion-v06-qd-04-v1"></a>
- [x] **V06-QD-04.V1** — Review every discrepancy in the audit's QD-04 table against source, current workflow definitions and version-bound evidence; retain an explicit resolution for each row. **Trace:** [Audit QD-04](docs/history/localmotive-comprehensive-audit.md#qd-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2050).

<a id="criterion-v06-qd-04-v2"></a>
- [x] **V06-QD-04.V2** — Check documented commands and local paths, and verify storage/recovery descriptions against implemented behavior without presenting planned features as available. **Trace:** [Audit QD-04](docs/history/localmotive-comprehensive-audit.md#qd-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2051).

<a id="criterion-v06-qd-04-v3"></a>
- [x] **V06-QD-04.V3** — Confirm historical counts, blockers and closeout evidence remain intact and distinguishable from current status; rerun documentation/branding gates that apply. **Trace:** [Audit QD-04](docs/history/localmotive-comprehensive-audit.md#qd-04).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2052).

### V06-QD-05

<a id="criterion-v06-qd-05-i1"></a>
- [x] **V06-QD-05.I1** — Declare a Node engine range compatible with the actual locked bundler/test toolchain and correct the broad Node.js 20 setup instruction; the audited Vite version requires Node 20.19 or an allowed later line. **Trace:** [Audit QD-05](docs/history/localmotive-comprehensive-audit.md#qd-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2069).

<a id="criterion-v06-qd-05-i2"></a>
- [x] **V06-QD-05.I2** — Declare the selected package-manager version and project Node/Rust toolchain configuration, deciding explicitly which versions are exact pins and which are supported minimums. **Trace:** [Audit QD-05](docs/history/localmotive-comprehensive-audit.md#qd-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2070).

<a id="criterion-v06-qd-05-i3"></a>
- [x] **V06-QD-05.I3** — Align CI toolchain selection with those project declarations while retaining immutable action references, npm ci and Cargo --locked dependency resolution. **Trace:** [Audit QD-05](docs/history/localmotive-comprehensive-audit.md#qd-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2071).

<a id="criterion-v06-qd-05-i4"></a>
- [x] **V06-QD-05.I4** — Document how toolchain updates are proposed and verified, and record the actual Node, npm and Rust versions used in release evidence so floating environment changes are observable. **Trace:** [Audit QD-05](docs/history/localmotive-comprehensive-audit.md#qd-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2072).

<a id="criterion-v06-qd-05-v1"></a>
- [x] **V06-QD-05.V1** — Run documented setup and the required check suite in a clean environment with the declared supported tooling, retaining exact version output and command results. **Trace:** [Audit QD-05](docs/history/localmotive-comprehensive-audit.md#qd-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2076).

<a id="criterion-v06-qd-05-v2"></a>
- [x] **V06-QD-05.V2** — Check an older unsupported Node configuration receives a clear engine/setup constraint instead of being recommended as supported by the documentation. **Trace:** [Audit QD-05](docs/history/localmotive-comprehensive-audit.md#qd-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2077).

<a id="criterion-v06-qd-05-v3"></a>
- [x] **V06-QD-05.V3** — Verify CI, local configuration and release evidence agree on the chosen toolchain; confirm dependency installation preserves the committed lockfile selections. **Trace:** [Audit QD-05](docs/history/localmotive-comprehensive-audit.md#qd-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2078).

<a id="criterion-v06-qd-05-v4"></a>
- [x] **V06-QD-05.V4** — Review an example toolchain update against the documented procedure, including compatibility checks for the current Vite/Vitest and Rust dependencies. **Trace:** [Audit QD-05](docs/history/localmotive-comprehensive-audit.md#qd-05).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2079).

### V06-QD-06

<a id="criterion-v06-qd-06-i1"></a>
- [x] **V06-QD-06.I1** — Identify the imported document's exact upstream repository path and commit, record its content identity and import date, and disclose any provenance that cannot be established instead of guessing it. **Trace:** [Audit QD-06](docs/history/localmotive-comprehensive-audit.md#qd-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2096).

<a id="criterion-v06-qd-06-i2"></a>
- [x] **V06-QD-06.I2** — Resolve the ten audited missing relative targets to existing local resources or immutable URLs in the identified upstream tree, including multimodal, function-calling, examples, development notes and UI constants. **Trace:** [Audit QD-06](docs/history/localmotive-comprehensive-audit.md#qd-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2097).

<a id="criterion-v06-qd-06-i3"></a>
- [x] **V06-QD-06.I3** — Keep the application option-grouping document separate from the imported generated server reference; state that the selected executable's --help remains authoritative for actual supported flags. **Trace:** [Audit QD-06](docs/history/localmotive-comprehensive-audit.md#qd-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2098).

<a id="criterion-v06-qd-06-i4"></a>
- [x] **V06-QD-06.I4** — Add a small local-link check or an explicit, narrow policy for deliberately retained upstream-relative references, and describe how future reference imports preserve provenance and link integrity. **Trace:** [Audit QD-06](docs/history/localmotive-comprehensive-audit.md#qd-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2099).

<a id="criterion-v06-qd-06-v1"></a>
- [x] **V06-QD-06.V1** — Verify the recorded upstream path/commit and imported bytes or documented local changes against that exact source, preserving uncertainty if the historical origin remains unresolved. **Trace:** [Audit QD-06](docs/history/localmotive-comprehensive-audit.md#qd-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2103).

<a id="criterion-v06-qd-06-v2"></a>
- [x] **V06-QD-06.V2** — Recheck all ten broken targets identified in QD-06 and confirm corrected links resolve to the intended documents at the pinned revision. **Trace:** [Audit QD-06](docs/history/localmotive-comprehensive-audit.md#qd-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2104).

<a id="criterion-v06-qd-06-v3"></a>
- [x] **V06-QD-06.V3** — Run the local-link validator with a deliberately missing fixture target and a valid upstream-reference case to show it detects regressions without silently exempting ordinary broken links. **Trace:** [Audit QD-06](docs/history/localmotive-comprehensive-audit.md#qd-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2105).

<a id="criterion-v06-qd-06-v4"></a>
- [x] **V06-QD-06.V4** — Review the option map and imported header together to confirm readers can distinguish application grouping guidance, historical upstream documentation and live runtime capabilities. **Trace:** [Audit QD-06](docs/history/localmotive-comprehensive-audit.md#qd-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2106).

### V06-S-01

<a id="criterion-v06-s-01-i1"></a>
- [x] **V06-S-01.I1** — Specify how termination, process wait and stdout/stderr reader joins are supervised against an actual monotonic cleanup deadline; report an unresolved cleanup outcome honestly instead of checking elapsed time only after a blocking call returns. **Trace:** [Runtime additional observations](docs/history/localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2126).

<a id="criterion-v06-s-01-i2"></a>
- [x] **V06-S-01.I2** — Choose and document whether output overflow terminates a process immediately or only truncates retained output; if termination is promised, signal the supervisor as soon as the threshold is crossed. **Trace:** [Runtime additional observations](docs/history/localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2127).

<a id="criterion-v06-s-01-i3"></a>
- [x] **V06-S-01.I3** — Automatically discover production Rust modules for the hidden-command invariant and supplement source assertions with an immediate descendant-launch containment test. **Trace:** [Runtime additional observations](docs/history/localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2128).

<a id="criterion-v06-s-01-v1"></a>
- [x] **V06-S-01.V1** — Exercise child/grandchild fixtures that retain pipes, emit excessive output or exit during cleanup; measure Windows process/listener cleanup and inspect the outcome on timeout. **Trace:** [Runtime additional observations](docs/history/localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2132).

### V06-S-02

<a id="criterion-v06-s-02-i1"></a>
- [x] **V06-S-02.I1** — Document logical/header identity versus the full named artifact-set identity, including filename and companion ordering. Review cache and measurement consumers so metadata-only identity cannot stand in for a content check. **Trace:** [Runtime additional observations](docs/history/localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2148).

<a id="criterion-v06-s-02-i2"></a>
- [x] **V06-S-02.I2** — Validate available per-shard GGUF index/count and compatible header metadata consistently; represent unverifiable tensor-set completeness as unknown without reading tensor data or reimplementing inference. **Trace:** [Runtime additional observations](docs/history/localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2149).

<a id="criterion-v06-s-02-v1"></a>
- [x] **V06-S-02.V1** — Use same-size changes after the header, renamed identical bytes, reordered companions and inconsistent split metadata to prove each identity and completeness claim has the intended scope. **Trace:** [Runtime additional observations](docs/history/localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2153).

### V06-S-03

<a id="criterion-v06-s-03-i1"></a>
- [x] **V06-S-03.I1** — Reject zero or otherwise impossible block/head/key/value dimensions before computing dense KV estimates; preserve checked arithmetic and explicit reasons for unknown results. **Trace:** [Runtime additional observations](docs/history/localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2169).

<a id="criterion-v06-s-03-i2"></a>
- [x] **V06-S-03.I2** — Keep unsupported architectures, quantized caches, recurrent/hybrid layouts and uncertain multi-device allocations unknown; label file-size weight estimates as proxies. **Trace:** [Runtime additional observations](docs/history/localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2170).

<a id="criterion-v06-s-03-v1"></a>
- [x] **V06-S-03.V1** — Cover zero dimensions, overflow, valid supported dense shapes and unsupported layouts; assert no invalid metadata produces an authoritative zero-memory estimate. **Trace:** [Runtime additional observations](docs/history/localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2174).

### V06-S-04

<a id="criterion-v06-s-04-i1"></a>
- [x] **V06-S-04.I1** — Offer an explicit repair action that quarantines an invalid cached health model and downloads the pinned immutable revision through the existing size/hash authorization checks. **Trace:** [Runtime additional observations](docs/history/localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2190).

<a id="criterion-v06-s-04-i2"></a>
- [x] **V06-S-04.I2** — Preserve failure diagnostics and never execute the corrupt file; handle cancellation and a failed replacement without labeling the cache healthy. **Trace:** [Runtime additional observations](docs/history/localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2191).

<a id="criterion-v06-s-04-v1"></a>
- [x] **V06-S-04.V1** — Corrupt a benign cached fixture, request repair, interrupt a repair, and retry. Verify trust rejection before repair and successful verified replacement afterward. **Trace:** [Runtime additional observations](docs/history/localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2195).

### V06-S-05

<a id="criterion-v06-s-05-i1"></a>
- [x] **V06-S-05.I1** — Create shared positive/negative fixtures for JavaScript and Rust catalog validation, including required rich metadata, dates, integer bounds, revisions, filenames and quant labels. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2211).

<a id="criterion-v06-s-05-i2"></a>
- [x] **V06-S-05.I2** — Make dropped model counts and reasons visible. Enforce one row per repository or explicitly authorize files across all matching rows so a displayed second repository row is not unreachable. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2212).

<a id="criterion-v06-s-05-v1"></a>
- [x] **V06-S-05.V1** — Run the same fixtures through both validators, including duplicate repositories, duplicate IDs and malformed rows; assert displayed rows have predictable authorization and rejection diagnostics. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2216).

### V06-S-06

<a id="criterion-v06-s-06-i1"></a>
- [x] **V06-S-06.I1** — Bound nested file/tag arrays, field lengths and aggregate query payload work at the Rust boundary, including deserialization where feasible; prefer authoritative store IDs over caller-supplied full models. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2232).

<a id="criterion-v06-s-06-i2"></a>
- [x] **V06-S-06.I2** — Add explicit acceptance/rejection policy for Windows reserved device names, controls, trailing-dot/space and extended-path aliases without relaxing existing separator, ADS or traversal protections. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2233).

<a id="criterion-v06-s-06-v1"></a>
- [x] **V06-S-06.V1** — Exercise one oversized nested model, many bounded models, Unicode and Windows special-name fixtures; verify deterministic rejection before large cloning/formatting or file publication. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2237).

### V06-S-07

<a id="criterion-v06-s-07-i1"></a>
- [x] **V06-S-07.I1** — Define a bounded token input length before validation/storage and return an actionable rejection without logging secret contents. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2253).

<a id="criterion-v06-s-07-i2"></a>
- [x] **V06-S-07.I2** — Surface legacy Credential Manager deletion failures during migration/save and offer a retry/clear path; retain masked status and sensitive Authorization headers only. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2254).

<a id="criterion-v06-s-07-v1"></a>
- [x] **V06-S-07.V1** — Use a fake secret store for overlength inputs, failed legacy deletion, retry and explicit clear. Assert no token appears in sidecars, UI status, diagnostics or logs. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2258).

### V06-S-08

<a id="criterion-v06-s-08-i1"></a>
- [x] **V06-S-08.I1** — Define allowed redirect schemes/hosts and proxy expectations for fixed GitHub/HF endpoints; retain TLS verification and avoid accepting arbitrary frontend URLs. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2274).

<a id="criterion-v06-s-08-i2"></a>
- [x] **V06-S-08.I2** — Test sensitive-header handling across same-origin, cross-origin and scheme-changing redirects using synthetic credentials and a controlled transport fixture. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2275).

<a id="criterion-v06-s-08-v1"></a>
- [x] **V06-S-08.V1** — Prove Authorization reaches only intended endpoints and disallowed redirects fail with a safe diagnostic; repeat relevant resume and final-hash checks after legitimate redirects. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2279).

### V06-S-09

<a id="criterion-v06-s-09-i1"></a>
- [x] **V06-S-09.I1** — Keep the exclusively created temporary cache file handle through write/sync/publication instead of closing and reopening its name; preserve atomic body, ETag and signature publication. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2295).

<a id="criterion-v06-s-09-i2"></a>
- [x] **V06-S-09.I2** — Separate successful in-memory refresh from durable cache/stamp persistence. Surface write/sync/rename failures and retain a usable last-good snapshot. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2296).

<a id="criterion-v06-s-09-v1"></a>
- [x] **V06-S-09.V1** — Inject temp-file replacement attempts and write/sync/rename/stamp failures; verify no untrusted replacement is accepted and no failed persistence is reported as durable success. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2300).

### V06-S-10

<a id="criterion-v06-s-10-i1"></a>
- [x] **V06-S-10.I1** — Compare opaque ETags with strict equality, define safe handling of weak validators and only issue conditional requests compatible with their semantics. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2316).

<a id="criterion-v06-s-10-i2"></a>
- [x] **V06-S-10.I2** — Honor bounded Retry-After/backoff for 429 responses while respecting cancellation and an overall transfer budget; add explicit request and total deadlines to catalog-builder fetch/retry work. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2317).

<a id="criterion-v06-s-10-v1"></a>
- [x] **V06-S-10.V1** — Exercise case-distinct and weak ETags, changed validators, numeric/date Retry-After, malformed/extreme delays, cancellation during backoff and a stalled builder fetch. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2321).

### V06-S-11

<a id="criterion-v06-s-11-i1"></a>
- [x] **V06-S-11.I1** — Distinguish a model having some small variant from the selected quantized file meeting the size heuristic. Return or display the selected build and the inputs behind fit decisions. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2337).

<a id="criterion-v06-s-11-i2"></a>
- [x] **V06-S-11.I2** — Keep disk-size/weight heuristics separate from actual inference-memory qualification; show unknown metadata without fabricating capacity guarantees. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2338).

<a id="criterion-v06-s-11-v1"></a>
- [x] **V06-S-11.V1** — Use a model with small and large quants and a budget between them; switch quant filters and verify the label never claims the larger selected build fits solely because the smaller one does. **Trace:** [Catalog additional observations](docs/history/localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2342).

### V06-S-12

<a id="criterion-v06-s-12-i1"></a>
- [x] **V06-S-12.I1** — Label the repetitive greedy workload as a controlled microbenchmark and state unsupported workload classes; do not generalize speculative-decoding gains to every prompt. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2358).

<a id="criterion-v06-s-12-i2"></a>
- [x] **V06-S-12.I2** — Display sample count and explain that a five-trial nearest-rank p95 is the sample maximum. Keep derived TTFT distinct from directly observed first-token latency. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2359).

<a id="criterion-v06-s-12-i3"></a>
- [x] **V06-S-12.I3** — Label warm-server peak working set as process-lifetime CPU working-set evidence, excluding dedicated GPU memory; do not present it as isolated request allocation or combined CPU/GPU footprint. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2360).

<a id="criterion-v06-s-12-v1"></a>
- [x] **V06-S-12.V1** — Check UI and exported labels against five-trial, warm-process and missing-direct-TTFT fixtures; ensure sample count, units, provenance and caveats survive round trips. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2364).

### V06-S-13

<a id="criterion-v06-s-13-i1"></a>
- [x] **V06-S-13.I1** — Validate enum/numeric domains, relationships and sentinels for cache, attention, split, draft probabilities, threads and allocation counts using the selected runtime contract; argument existence alone is insufficient. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2380).

<a id="criterion-v06-s-13-i2"></a>
- [x] **V06-S-13.I2** — Describe filename/folder/quant-based companion matching as a heuristic and use available metadata/provenance for stronger matching. Clear or reject a retained draft path when the newly selected method has no compatible companion. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2381).

<a id="criterion-v06-s-13-v1"></a>
- [x] **V06-S-13.V1** — Reject invalid direct IPC profiles and advisor proposals before launch or another paid call. Test mixed-family folders, ambiguous companions and changing draft methods without a matching file. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2385).

### V06-S-14

<a id="criterion-v06-s-14-i1"></a>
- [x] **V06-S-14.I1** — Either provide correctly escaped commands for an explicitly named target shell or export structured argv; keep process execution on argument arrays. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2401).

<a id="criterion-v06-s-14-i2"></a>
- [x] **V06-S-14.I2** — Cover actual short/long path-bearing flags including -md and LoRA in manifest-safe argument handling, and document that raw local manifests still include explicit local paths and differ from public share exports. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2402).

<a id="criterion-v06-s-14-v1"></a>
- [x] **V06-S-14.V1** — Round-trip spaces, quotes, metacharacters and Unicode through the chosen command representation; use canary paths across every supported path-bearing flag and verify each stated redaction guarantee. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2406).

### V06-S-15

<a id="criterion-v06-s-15-i1"></a>
- [x] **V06-S-15.I1** — Set depth, visited-entry/work and diagnostic bounds for scans; check cancellation throughout traversal and offload scanning from the UI thread. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2422).

<a id="criterion-v06-s-15-i2"></a>
- [x] **V06-S-15.I2** — Return bounded per-path diagnostics for unreadable subdirectories while preserving valid discovered models, and retain symlink/reparse-point skipping. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2423).

<a id="criterion-v06-s-15-v1"></a>
- [x] **V06-S-15.V1** — Scan deep/wide fixture trees, inaccessible subdirectories and paths with Unicode/spaces; cancel mid-scan and verify responsive UI, bounded work and usable partial diagnostics. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2427).

### V06-S-16

<a id="criterion-v06-s-16-i1"></a>
- [x] **V06-S-16.I1** — Define benchmark/calibration retention, indexing and quota behavior, including explicit user cleanup and retention of records needed by calibration or exports. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2443).

<a id="criterion-v06-s-16-i2"></a>
- [x] **V06-S-16.I2** — Quarantine/report individual corrupt calibration records and continue loading valid compatible history; bound enumeration/parsing work rather than failing the complete load on the first bad file. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2444).

<a id="criterion-v06-s-16-v1"></a>
- [x] **V06-S-16.V1** — Load mixed valid/corrupt/oversized records and a large history; verify compatible records remain available and retention does not silently invalidate referenced evidence. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2448).

### V06-S-17

<a id="criterion-v06-s-17-i1"></a>
- [x] **V06-S-17.I1** — Explain that Verified import status records a user review, not local rerun, origin signature or cryptographic proof of measurement. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2464).

<a id="criterion-v06-s-17-i2"></a>
- [x] **V06-S-17.I2** — Retain Pending-on-import and explicit review transitions; keep imported evidence separately labeled and prevent future ranking/calibration integration from silently promoting its provenance. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2465).

<a id="criterion-v06-s-17-v1"></a>
- [x] **V06-S-17.V1** — Import, review, export and reload synthetic external evidence; assert provenance labels remain stable and it cannot impersonate a local run through state changes alone. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2469).

### V06-S-18

<a id="criterion-v06-s-18-i1"></a>
- [x] **V06-S-18.I1** — Generate types from the Rust authority or validate shared representative IPC fixtures across Rust and TypeScript, including snake/camel case, enum, nullability and error contracts. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities); [Maintainability assessment](docs/history/localmotive-comprehensive-audit.md#maintainability-assessment).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2485).

<a id="criterion-v06-s-18-i2"></a>
- [x] **V06-S-18.I2** — Version saved benchmark/calibration/profile/catalog formats explicitly and implement migrations or actionable rejection before changing token/cache semantics or compatibility identities. Keep serialization validation outside rendering. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities); [Maintainability assessment](docs/history/localmotive-comprehensive-audit.md#maintainability-assessment).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2486).

<a id="criterion-v06-s-18-v1"></a>
- [x] **V06-S-18.V1** — Run old/current/malformed fixtures through native serialization and production frontend consumers; verify migration retains valid identity and unknown fields/version failures are handled intentionally. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities); [Maintainability assessment](docs/history/localmotive-comprehensive-audit.md#maintainability-assessment).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2490).

### V06-S-19

<a id="criterion-v06-s-19-i1"></a>
- [x] **V06-S-19.I1** — Add bounded property/fuzz targets around malformed JSON, Unicode, nested proposal braces, duplicate IDs, shard sets, finite extremes and numeric coercion. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2506).

<a id="criterion-v06-s-19-i2"></a>
- [x] **V06-S-19.I2** — Assert common invariants across summary, persistence, import and export; retain deterministic regression seeds for every discovered issue and cap test resources. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2507).

<a id="criterion-v06-s-19-v1"></a>
- [x] **V06-S-19.V1** — Run a documented bounded campaign and confirm seeded mutations violate the intended invariant; archive commands, seed/corpus identity, time budget and observed result. **Trace:** [Measurement additional limits](docs/history/localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2511).

### V06-S-20

<a id="criterion-v06-s-20-i1"></a>
- [x] **V06-S-20.I1** — Add a concise first-use data-sent list/preview near AI Tune covering profile, hardware, runtime/model/companion paths, measurements, commands and errors; clearly separate cloud tuning from local inference and local share export. **Trace:** [Cloud remaining validation](docs/history/localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation); [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2527).

<a id="criterion-v06-s-20-i2"></a>
- [x] **V06-S-20.I2** — Design an optional minimal/redacted brief that removes unnecessary path/user identifiers while retaining facts required for useful advice; state what remains and preserve the proposal whitelist. **Trace:** [Cloud remaining validation](docs/history/localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation); [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2528).

<a id="criterion-v06-s-20-v1"></a>
- [x] **V06-S-20.V1** — Use canary paths and synthetic trial errors to inspect the exact full/minimal payloads. Verify disclosure is available before the action and does not expose stored secret values. **Trace:** [Cloud remaining validation](docs/history/localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation); [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2532).

### V06-S-21

<a id="criterion-v06-s-21-i1"></a>
- [x] **V06-S-21.I1** — Cap model-list, exchange, success and error response bodies and connect each request to the tuning attempt/time budget; handle bounded Retry-After/backoff without adding invisible unlimited retries. **Trace:** [Cloud remaining validation](docs/history/localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2548).

<a id="criterion-v06-s-21-i2"></a>
- [x] **V06-S-21.I2** — Create contract fixtures for all six fixed providers, including supported request parameters, response shape, auth failure, quota, model availability and malformed/oversized responses. Preserve the fixed HTTPS provider allowlist. **Trace:** [Cloud remaining validation](docs/history/localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2549).

<a id="criterion-v06-s-21-i3"></a>
- [x] **V06-S-21.I3** — Record which contracts were stubbed and which were checked live with authorized credentials; review the Anthropic compatibility layer on its actual documented behavior instead of assuming native-API differences prove failure. **Trace:** [Cloud remaining validation](docs/history/localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2550).

<a id="criterion-v06-s-21-v1"></a>
- [x] **V06-S-21.V1** — Prove oversized bodies, 429s, invalid JSON and cancellation terminate with bounded resources. For an explicitly authorized live smoke check, record provider/model/date and sanitized outcome without retaining credentials. **Trace:** [Cloud remaining validation](docs/history/localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2554).

### V06-S-22

<a id="criterion-v06-s-22-i1"></a>
- [x] **V06-S-22.I1** — Catch and present dialog, openUrl and cancellation rejections; use the existing structured error formatter consistently instead of displaying raw JSON through String(error). **Trace:** [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2570).

<a id="criterion-v06-s-22-i2"></a>
- [x] **V06-S-22.I2** — Keep concise recovery text with an explicit diagnostic disclosure/copy affordance, while preserving successful operation results when only a secondary UI or save action fails. **Trace:** [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2571).

<a id="criterion-v06-s-22-v1"></a>
- [x] **V06-S-22.V1** — Reject each dialog/opener/cancel promise and return structured/native string errors; verify actionable messages, no unhandled rejection and retained valid results. **Trace:** [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2575).

### V06-S-23

<a id="criterion-v06-s-23-i1"></a>
- [x] **V06-S-23.I1** — Render a Profile empty state before a model/profile exists, with the next action required to create one. **Trace:** [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2591).

<a id="criterion-v06-s-23-i2"></a>
- [x] **V06-S-23.I2** — Add a zero-model Inventory state and local recovery actions for choosing/rescanning a folder; distinguish an empty valid folder from a failed scan. **Trace:** [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2592).

<a id="criterion-v06-s-23-v1"></a>
- [x] **V06-S-23.V1** — Open the packaged app with no settings/models, visit Profile and Inventory, choose an empty folder and then a valid fixture folder; verify each state gives an accurate next action. **Trace:** [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2596).

### V06-S-24

<a id="criterion-v06-s-24-i1"></a>
- [x] **V06-S-24.I1** — Provide full selectable or explicitly expandable/copyable inventory directories, log paths and catalog filenames where truncation currently hides identity. **Trace:** [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2612).

<a id="criterion-v06-s-24-i2"></a>
- [x] **V06-S-24.I2** — Align signal styling and primary-action emphasis with the existing design policy or explicitly revise the normative policy with a reason; retain words alongside status colors. **Trace:** [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2613).

<a id="criterion-v06-s-24-i3"></a>
- [x] **V06-S-24.I3** — Respect reduced-motion preference in the JavaScript Jump to Advanced action and place keyboard focus coherently when expanding/navigating to that region. **Trace:** [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2614).

<a id="criterion-v06-s-24-v1"></a>
- [x] **V06-S-24.V1** — Inspect long paths, narrow/zoomed layouts and reduced-motion keyboard navigation; confirm paths remain retrievable and the intended focus target is visible. **Trace:** [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2618).

### V06-S-25

<a id="criterion-v06-s-25-i1"></a>
- [x] **V06-S-25.I1** — Record WebView2 input/frame stalls during preview, scan, hash and startup; measure hashed bytes, duplicate jobs and cancellation latency for cold/warm runtime operations. **Trace:** [Measurements after correctness](docs/history/localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored); [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations); [Rust dependency inventory](docs/history/localmotive-comprehensive-audit.md#rust).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2634).

<a id="criterion-v06-s-25-i2"></a>
- [x] **V06-S-25.I2** — Measure downloader throughput/CPU/disk queues at 1/4/8 connections, crash/resume loss, and catalog search/filter latency, IPC bytes and render time at the current catalog size. Optimize debounce, query ownership or presentation update frequency only where evidence supports it. **Trace:** [Measurements after correctness](docs/history/localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored); [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations); [Rust dependency inventory](docs/history/localmotive-comprehensive-audit.md#rust).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2635).

<a id="criterion-v06-s-25-i3"></a>
- [ ] **V06-S-25.I3** — Collect independent benchmark distributions/baseline drift, held-out calibration error and interval coverage, and release queue/failure/evidence-retention metrics. Avoid introducing virtualization or incompatible dependency unification purely from file size/counts. **Trace:** [Measurements after correctness](docs/history/localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored); [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations); [Rust dependency inventory](docs/history/localmotive-comprehensive-audit.md#rust).
  **Current status: OPEN.** PARTIAL / U06-07: hardware-only coverage deferred by D06-05; available release metrics remain executable and independent datasets are a separate input. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2636).

<a id="criterion-v06-s-25-i4"></a>
- [x] **V06-S-25.I4** — Run cargo tree --target x86_64-pc-windows-msvc -d in the resolved build environment, record which duplicate versions reach the shipped target, and measure release binary size before and after any proposed dependency change. Retain legitimate incompatible upstream/platform versions unless a tested change has a demonstrated benefit. **Trace:** [Measurements after correctness](docs/history/localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored); [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations); [Rust dependency inventory](docs/history/localmotive-comprehensive-audit.md#rust).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2637).

<a id="criterion-v06-s-25-v1"></a>
- [x] **V06-S-25.V1** — Run reproducible before/after scenarios with candidate SHA, hardware, workload, dataset size and tools recorded; verify an optimization preserves the relevant correctness and cancellation tests. **Trace:** [Measurements after correctness](docs/history/localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored); [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations); [Rust dependency inventory](docs/history/localmotive-comprehensive-audit.md#rust).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2641).

<a id="criterion-v06-s-25-v2"></a>
- [x] **V06-S-25.V2** — Attach the target dependency graph and comparable binary-size measurements to any deduplication proposal; rerun build, runtime and installer checks affected by a dependency change. **Trace:** [Measurements after correctness](docs/history/localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored); [Frontend additional observations](docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations); [Rust dependency inventory](docs/history/localmotive-comprehensive-audit.md#rust).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2642).

### V06-S-26

<a id="criterion-v06-s-26-i1"></a>
- [x] **V06-S-26.I1** — After the known regressions are covered, report module-level coverage and use targeted mutation checks for important invariants; do not substitute an arbitrary percentage for behavioral acceptance. **Trace:** [Testing strengths and measured scope](docs/history/localmotive-comprehensive-audit.md#testing-strengths-and-measured-scope).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2658).

<a id="criterion-v06-s-26-i2"></a>
- [x] **V06-S-26.I2** — Document or remove the duplicate release-gate invocation in CI while preserving all required checks and useful fail-fast behavior. **Trace:** [Testing strengths and measured scope](docs/history/localmotive-comprehensive-audit.md#testing-strengths-and-measured-scope).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2659).

<a id="criterion-v06-s-26-i3"></a>
- [x] **V06-S-26.I3** — Keep the ignored local-model-tree diagnostic separate from an assertion-based acceptance test and run the opt-in approved-runtime/hardware probe only with its actual prerequisites in an explicit qualification job. **Trace:** [Testing strengths and measured scope](docs/history/localmotive-comprehensive-audit.md#testing-strengths-and-measured-scope).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2660).

<a id="criterion-v06-s-26-v1"></a>
- [x] **V06-S-26.V1** — Verify one removed/disabled behavioral guard makes its regression fail; compare CI check inventory before/after deduplication and inspect the hardware probe result rather than counting ignored annotations as passes. **Trace:** [Testing strengths and measured scope](docs/history/localmotive-comprehensive-audit.md#testing-strengths-and-measured-scope).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2664).

### V06-S-27

<a id="criterion-v06-s-27-i1"></a>
- [x] **V06-S-27.I1** — Split catalog store/cache, supervised job lifecycle and runtime authorization/installer facades behind tested contracts, keeping Rust authoritative. **Trace:** [Maintainability assessment](docs/history/localmotive-comprehensive-audit.md#maintainability-assessment).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2680).

<a id="criterion-v06-s-27-i2"></a>
- [x] **V06-S-27.I2** — Extract independent screen/hooks and typed operation state from App and the evidence panel so result lifetime follows job ownership; separate presentational components from acquisition and persistence. **Trace:** [Maintainability assessment](docs/history/localmotive-comprehensive-audit.md#maintainability-assessment).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2681).

<a id="criterion-v06-s-27-i3"></a>
- [x] **V06-S-27.I3** — Move code incrementally with preserved boundary regressions; avoid a broad rewrite or using line count alone as the reason to change a module. **Trace:** [Maintainability assessment](docs/history/localmotive-comprehensive-audit.md#maintainability-assessment).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2682).

<a id="criterion-v06-s-27-v1"></a>
- [x] **V06-S-27.V1** — Run the existing and newly added observable start/cancel/finish/rerun, serialization and persistence scenarios before/after each extraction; verify no public route bypasses the shared authority. **Trace:** [Maintainability assessment](docs/history/localmotive-comprehensive-audit.md#maintainability-assessment).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2686).

### V06-S-28

<a id="criterion-v06-s-28-i1"></a>
- [x] **V06-S-28.I1** — Track a separately scoped full-history secret review, owner-provided repository security/settings review and runner isolation assessment, using access explicitly available for that review; retain unavailable items as unverified rather than inventing findings. **Trace:** [Scope, method, and limits](docs/history/localmotive-comprehensive-audit.md#scope-method-and-limits); [Remaining target verification](docs/history/localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2702).

<a id="criterion-v06-s-28-i2"></a>
- [x] **V06-S-28.I2** — If historical secrets are actually found, record a private remediation/rotation outcome without copying values into this tracker or public evidence. Link target-specific dependency/license work to GH-08 rather than duplicating it. **Trace:** [Scope, method, and limits](docs/history/localmotive-comprehensive-audit.md#scope-method-and-limits); [Remaining target verification](docs/history/localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2703).

<a id="criterion-v06-s-28-v1"></a>
- [x] **V06-S-28.V1** — Record inspected scope, tools/date, sanitized result and remaining access gaps for each review; do not turn an unavailable scan or absent alert into a clean bill of health. **Trace:** [Scope, method, and limits](docs/history/localmotive-comprehensive-audit.md#scope-method-and-limits); [Remaining target verification](docs/history/localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2707).

### V06-S-29

<a id="criterion-v06-s-29-i1"></a>
- [x] **V06-S-29.I1** — Document the exact handoff from the manually dispatched catalog candidate artifact through review, signature generation and committed promotion; make clear that the existing workflow does not itself publish to main. **Trace:** [CI history and reliability](docs/history/localmotive-comprehensive-audit.md#ci-history-and-reliability-context).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2730).

<a id="criterion-v06-s-29-i2"></a>
- [x] **V06-S-29.I2** — Define who retains/retrieves the candidate before the seven-day artifact expiry, how an expired/failed candidate is rebuilt, and how the promoted body/signature pair is checked together. **Trace:** [CI history and reliability](docs/history/localmotive-comprehensive-audit.md#ci-history-and-reliability-context).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2731).

<a id="criterion-v06-s-29-v1"></a>
- [x] **V06-S-29.V1** — Rehearse candidate generation, review and signature validation without treating candidate creation as publication. Exercise missing/expired artifacts and prove recovery cannot silently promote different unreviewed bytes. **Trace:** [CI history and reliability](docs/history/localmotive-comprehensive-audit.md#ci-history-and-reliability-context).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2735).

### V06-G-01

<a id="criterion-v06-g-01-i1"></a>
- [x] **V06-G-01.I1** — Record the actual v0.6 start commit and compare it with the audited e530371b056cd8e049c2246dbb151aa407bf359f snapshot. For each finding, establish whether the current branch still reproduces it or already has a fix with equivalent evidence. **Trace:** [Remediation and release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Audited scope](docs/history/localmotive-comprehensive-audit.md#scope-method-and-limits).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2762).

<a id="criterion-v06-g-01-i2"></a>
- [x] **V06-G-01.I2** — Assign an owner and current status to every finding package. Preserve all 72 original IDs, priorities and citations, including the consolidation of FE-10 into DC-04; record supplemental scope separately. **Trace:** [Remediation and release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Audited scope](docs/history/localmotive-comprehensive-audit.md#scope-method-and-limits).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2763).

<a id="criterion-v06-g-01-i3"></a>
- [x] **V06-G-01.I3** — If adopting this tracker in the repository, place the audit alongside it or adjust the audit-link prefix once, then verify every source anchor. Point the current contributor instructions to the new authoritative tracker while preserving historical closeout records. **Trace:** [Remediation and release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Audited scope](docs/history/localmotive-comprehensive-audit.md#scope-method-and-limits).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2764).

<a id="criterion-v06-g-01-v1"></a>
- [x] **V06-G-01.V1** — Reconcile the coverage register against the source report: 72 unique findings, 19 High, 43 Medium and 10 Low; confirm there is no unreferenced checkbox or broken audit anchor. **Trace:** [Remediation and release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Audited scope](docs/history/localmotive-comprehensive-audit.md#scope-method-and-limits).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2768).

### V06-G-02

<a id="criterion-v06-g-02-i1"></a>
- [x] **V06-G-02.I1** — For each behavioral fix, record a failing regression on the pre-fix implementation and its passing result on the fixed commit; exercise the actual boundary rather than only source substrings. **Trace:** [Remediation and release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2784).

<a id="criterion-v06-g-02-i2"></a>
- [x] **V06-G-02.I2** — Complete the verification ledger fields below with exact command/scenario, environment, fixture identity, result, test counts where meaningful, artifact/log path and residual limits. **Trace:** [Remediation and release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2785).

<a id="criterion-v06-g-02-i3"></a>
- [x] **V06-G-02.I3** — Mark a task complete only after its completion criteria and relevant integration layer are observed. A static-only change may use documented inspection, but a Windows-specific behavior requires Windows evidence. **Trace:** [Remediation and release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2786).

<a id="criterion-v06-g-02-v1"></a>
- [x] **V06-G-02.V1** — Review each proposed closure for a real observable regression and matching commit. Use a targeted mutation/negative control for critical guards so retained helper names cannot produce false confidence. **Trace:** [Remediation and release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2790).

### V06-G-03

<a id="criterion-v06-g-03-i1"></a>
- [x] **V06-G-03.I1** — Record Node/npm/Rust/OS/target versions, lockfile identities and the candidate source SHA. Use the declared toolchain and locked dependency installation; update version metadata consistently for 0.6.0, including generated lockfile root versions as applicable. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Verification ledger](docs/history/localmotive-comprehensive-audit.md#verification-ledger); [Actual verification status](docs/history/localmotive-comprehensive-audit.md#actual-verification-status); [Rust audit interpretation](docs/history/localmotive-comprehensive-audit.md#rust-audit-interpretation).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2806).

<a id="criterion-v06-g-03-i2"></a>
- [x] **V06-G-03.I2** — Run npm ci, npm run check, npm audit --json and the configured advisory policy. From src-tauri, run cargo fmt --check, cargo clippy --locked --all-targets -- -D warnings, cargo test --locked, and cargo test --locked --doc with RUSTDOCFLAGS set to -D warnings. Run the configured RustSec lockfile audit and retain its advisory-database revision, vulnerability results and informational warnings, with shipped-target relevance assessed separately. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Verification ledger](docs/history/localmotive-comprehensive-audit.md#verification-ledger); [Actual verification status](docs/history/localmotive-comprehensive-audit.md#actual-verification-status); [Rust audit interpretation](docs/history/localmotive-comprehensive-audit.md#rust-audit-interpretation).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2807).

<a id="criterion-v06-g-03-i3"></a>
- [x] **V06-G-03.I3** — Run all current catalog signature/schema, branding, approval-manifest, action-pin, research/qualification and release policy gates through the repository scripts/workflows; document explicit skips and their release impact. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Verification ledger](docs/history/localmotive-comprehensive-audit.md#verification-ledger); [Actual verification status](docs/history/localmotive-comprehensive-audit.md#actual-verification-status); [Rust audit interpretation](docs/history/localmotive-comprehensive-audit.md#rust-audit-interpretation).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2808).

<a id="criterion-v06-g-03-i4"></a>
- [x] **V06-G-03.I4** — Build the nonpublishing candidate exactly once at the chosen immutable source SHA using npm run tauri build or the equivalent retained release build job. Stage portable, MSI and NSIS outputs, compute SHA-256 values and create the original candidate inventory. Make the candidate workflow accept that explicit commit before the final release tag exists; later gates consume these retained bytes, and G-09 promotes them without rebuilding. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Verification ledger](docs/history/localmotive-comprehensive-audit.md#verification-ledger); [Actual verification status](docs/history/localmotive-comprehensive-audit.md#actual-verification-status); [Rust audit interpretation](docs/history/localmotive-comprehensive-audit.md#rust-audit-interpretation).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2809).

<a id="criterion-v06-g-03-v1"></a>
- [x] **V06-G-03.V1** — Attach successful output for the exact candidate SHA and inspect dependency warnings for the shipped target. Any code/build-input change invalidates affected evidence and requires the relevant gates again. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Verification ledger](docs/history/localmotive-comprehensive-audit.md#verification-ledger); [Actual verification status](docs/history/localmotive-comprehensive-audit.md#actual-verification-status); [Rust audit interpretation](docs/history/localmotive-comprehensive-audit.md#rust-audit-interpretation).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2813).

<a id="criterion-v06-g-03-v2"></a>
- [x] **V06-G-03.V2** — Verify all staged candidate assets exist and match their original inventory, version and source SHA before packaged/lifecycle jobs start. A missing, changed or rebuilt candidate invalidates the relevant downstream evidence. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Verification ledger](docs/history/localmotive-comprehensive-audit.md#verification-ledger); [Actual verification status](docs/history/localmotive-comprehensive-audit.md#actual-verification-status); [Rust audit interpretation](docs/history/localmotive-comprehensive-audit.md#rust-audit-interpretation).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2814).

### V06-G-04

<a id="criterion-v06-g-04-i1"></a>
- [x] **V06-G-04.I1** — Run every row in the minimum regression matrix below at its required layer, retaining failing-before and passing-after evidence for the associated fix. **Trace:** [Concrete minimum regression pack](docs/history/localmotive-comprehensive-audit.md#concrete-minimum-regression-pack).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2830).

<a id="criterion-v06-g-04-i2"></a>
- [x] **V06-G-04.I2** — Keep pure/SQL/protocol fixtures, component/IPC scenarios and packaged Windows runs labeled separately; a pass at one layer does not replace missing evidence at another. **Trace:** [Concrete minimum regression pack](docs/history/localmotive-comprehensive-audit.md#concrete-minimum-regression-pack).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2831).

<a id="criterion-v06-g-04-i3"></a>
- [x] **V06-G-04.I3** — Record cancellation timing, process/listener cleanup, actual identities, final file digests and preserved data where the scenario depends on those outcomes. **Trace:** [Concrete minimum regression pack](docs/history/localmotive-comprehensive-audit.md#concrete-minimum-regression-pack).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2832).

<a id="criterion-v06-g-04-v1"></a>
- [x] **V06-G-04.V1** — Review the matrix for 14 scenario groups with an observed result, evidence link and associated finding IDs; leave missing environment-dependent cases open. **Trace:** [Concrete minimum regression pack](docs/history/localmotive-comprehensive-audit.md#concrete-minimum-regression-pack).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2836).

### V06-G-05

<a id="criterion-v06-g-05-i1"></a>
- [x] **V06-G-05.I1** — Run packaged Windows tests for managed tamper rejection, NTFS rename/open-handle/hard-link behavior, legacy runtime migration, SQLite locking/recovery, active cancellation and Job Object cleanup. **Trace:** [Remaining target verification](docs/history/localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2852).

<a id="criterion-v06-g-05-i2"></a>
- [x] **V06-G-05.I2** — Verify accepted TLS/auth local profiles and default warm benchmarks against the actual approved runtime. Test stable mapping on identical GPUs when hardware is available; otherwise retain that row as untested. **Trace:** [Remaining target verification](docs/history/localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).
  **Current status: VERIFIED.** Accepted implementation/fixture/available-host scope; unprovided physical hardware portions are DEFERRED-OWNER under D06-03/04. No broader hardware observation is claimed. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2853).

<a id="criterion-v06-g-05-i3"></a>
- [ ] **V06-G-05.I3** — Perform keyboard/Narrator or NVDA, high-DPI/zoom/high-contrast and reduced-motion checks. Perform live cloud/HF credential scenarios only where an explicitly authorized test account and suitable environment are available. **Covered in the packaged probe:** keyboard traversal with visible focus, reduced motion, high-DPI/zoom, forced-colors high contrast (`verify_a11y.mjs` A11Y_PASS; evidence `release-evidence/0.6.0/attestations/g05-a11y-packaged-verification.log`). **Remaining (blocked):** a manual Narrator or NVDA pass on the packaged app, and live cloud/HF credential scenarios, which require an explicitly authorized test account and suitable environment. **Trace:** [Remaining target verification](docs/history/localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).
  **Current status: OPEN.** PARTIAL / U06-07: automated a11y accepted; manual listening and authorized live-account scenarios remain distinct from hardware deferral. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2854).

<a id="criterion-v06-g-05-v1"></a>
- [x] **V06-G-05.V1** — Bind every target result to source/candidate digest, OS build, driver/runtime/model identity and exact scenario. Distinguish CPU inference, accelerator inference, fixture presentation and external-service smoke checks. **Trace:** [Remaining target verification](docs/history/localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L2858).

### V06-G-06

<a id="criterion-v06-g-06-i1"></a>
- [x] **V06-G-06.I1** — Use candidate MSI/NSIS/portable bytes from the build, with checksums, in isolated clean-account lifecycle checks; do not occupy the publisher runner while waiting for not-yet-published assets. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-04](docs/history/localmotive-comprehensive-audit.md#gh-04); [GH-05](docs/history/localmotive-comprehensive-audit.md#gh-05); [GH-06](docs/history/localmotive-comprehensive-audit.md#gh-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3148).

<a id="criterion-v06-g-06-i2"></a>
- [ ] **V06-G-06.I2** — Test clean install/start and strict uninstall assertions for each supported installer. Test v0.4.1-to-v0.6 profile preservation separately from v0.5.0-to-v0.6 SQLite/user-override preservation. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-04](docs/history/localmotive-comprehensive-audit.md#gh-04); [GH-05](docs/history/localmotive-comprehensive-audit.md#gh-05); [GH-06](docs/history/localmotive-comprehensive-audit.md#gh-06).
  **Current status: OPEN.** R06-01 / U06-04: same native campaign; historical checkbox is superseded. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3149).

<a id="criterion-v06-g-06-i3"></a>
- [x] **V06-G-06.I3** — Verify target version and executable digest after upgrade and preserve expected user data. Collect machine-readable results and diagnostics on success, failure and timeout. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-04](docs/history/localmotive-comprehensive-audit.md#gh-04); [GH-05](docs/history/localmotive-comprehensive-audit.md#gh-05); [GH-06](docs/history/localmotive-comprehensive-audit.md#gh-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3150).

<a id="criterion-v06-g-06-v1"></a>
- [x] **V06-G-06.V1** — Inject a leftover installed executable, unchanged upgrade version, missing artifact and timeout; each must yield failure or explicit non-pass with retained evidence. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-04](docs/history/localmotive-comprehensive-audit.md#gh-04); [GH-05](docs/history/localmotive-comprehensive-audit.md#gh-05); [GH-06](docs/history/localmotive-comprehensive-audit.md#gh-06).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3154).

### V06-G-07

<a id="criterion-v06-g-07-i1"></a>
- [x] **V06-G-07.I1** — Revalidate compiled runtime approval roots, archive and per-file manifests, safe extraction/rollback, hidden contained processes and owned readiness listeners after refactoring. If the approved runtime changes, update every approval anchor and qualify the changed runtime explicitly. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Runtime controls](docs/history/localmotive-comprehensive-audit.md#controls-worth-preserving); [Catalog strengths](docs/history/localmotive-comprehensive-audit.md#strengths-worth-retaining); [Cloud controls](docs/history/localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation); [Supply-chain controls](docs/history/localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3170).

<a id="criterion-v06-g-07-i2"></a>
- [x] **V06-G-07.I2** — Keep Ed25519 catalog verification, mandatory model SHA-256, final-file verification and signed-snapshot authority intact. Never promote mutable SQLite/user metadata into curated authority by accident. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Runtime controls](docs/history/localmotive-comprehensive-audit.md#controls-worth-preserving); [Catalog strengths](docs/history/localmotive-comprehensive-audit.md#strengths-worth-retaining); [Cloud controls](docs/history/localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation); [Supply-chain controls](docs/history/localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3171).

<a id="criterion-v06-g-07-i3"></a>
- [x] **V06-G-07.I3** — Preserve Credential Manager storage, masked renderer status, fixed HTTPS providers, proposal whitelists and explicit export confirmation. Produce target-specific provenance/SBOM/notices and retain action-pin provenance per GH-08. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Runtime controls](docs/history/localmotive-comprehensive-audit.md#controls-worth-preserving); [Catalog strengths](docs/history/localmotive-comprehensive-audit.md#strengths-worth-retaining); [Cloud controls](docs/history/localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation); [Supply-chain controls](docs/history/localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3172).

<a id="criterion-v06-g-07-v1"></a>
- [x] **V06-G-07.V1** — Run negative fixtures for modified payloads, missing/extra manifest entries, invalid signature, redirected secret headers and mismatched evidence identities. Validate each support label against actual approved hardware evidence. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Runtime controls](docs/history/localmotive-comprehensive-audit.md#controls-worth-preserving); [Catalog strengths](docs/history/localmotive-comprehensive-audit.md#strengths-worth-retaining); [Cloud controls](docs/history/localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation); [Supply-chain controls](docs/history/localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3176).

### V06-G-08

<a id="criterion-v06-g-08-i1"></a>
- [x] **V06-G-08.I1** — Apply this plan's default release policy: all 19 High findings block the stabilization release until fixed and verified. Review every Medium/Low and supplemental item for completion or an explicit deferral with owner, reason, residual risk, workaround, follow-up milestone and evidence gap. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Audit priorities](docs/history/localmotive-comprehensive-audit.md#priority-scale).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3192).

<a id="criterion-v06-g-08-i2"></a>
- [x] **V06-G-08.I2** — Publish an accurate support matrix and known-limitations list. Keep unavailable GPU/OS/provider tests untested; distinguish reviewed imports, UI fixtures and actual inference. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Audit priorities](docs/history/localmotive-comprehensive-audit.md#priority-scale).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3193).

<a id="criterion-v06-g-08-i3"></a>
- [x] **V06-G-08.I3** — Retain the disclosed unsigned distribution policy if it continues; signing is not newly mandated by this tracker. Prepare user-facing changelog/version/support documentation from observed candidate behavior. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Audit priorities](docs/history/localmotive-comprehensive-audit.md#priority-scale).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3194).

<a id="criterion-v06-g-08-v1"></a>
- [ ] **V06-G-08.V1** — Reconcile task status, required checks, lifecycle results, immutable source/digests and public claims; a green aggregate summary must not hide an unresolved High finding or missing required evidence. **Kept open (2026-09-12, owner directive):** the reconciliation is maintained continuously in the third-pass record, but it closes only with the release decision - the unresolved High rows (GH-01, GH-02, GH-03, GH-06.V3) and the G-09/G-10 gates remain, and a written deferral is not treated as passing evidence. **Reconciled (2026-09-12 third pass):** the open boxes are individually categorised (owner-gated / environment-blocked) with completion criteria and unblock actions; unresolved High rows (GH-01, GH-02, GH-03, GH-06.V3) carry their criteria and are not marked verified; a written deferral is not treated as passing evidence anywhere in this record. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Audit priorities](docs/history/localmotive-comprehensive-audit.md#priority-scale).
  **Current status: OPEN.** U06-07: current evidence-based release acceptance, under standing authorization and explicit hardware deferrals. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3198).

### V06-G-09

<a id="criterion-v06-g-09-i1"></a>
- [ ] **V06-G-09.I1** — After the recorded ship decision, create/protect the v0.6.0 tag at the verified immutable SHA and promote the exact tested candidate assets; do not rebuild or retarget a used tag. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02); [GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: OPEN.** U06-06/U06-08: final source/tag disposition and exact-byte promotion; old tag is not the repaired candidate. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3214).

<a id="criterion-v06-g-09-i2"></a>
- [ ] **V06-G-09.I2** — Bind release inventory, checksums, packaged/lifecycle records and provenance to that SHA and each artifact digest. Refuse publication when source, version, candidate or required evidence differs. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02); [GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: OPEN.** U06-02/U06-06/U06-08: actual candidate inventory/records and promoted bytes agree. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3215).

<a id="criterion-v06-g-09-i3"></a>
- [ ] **V06-G-09.I3** — Read back the published tag/release metadata and complete asset set. Verify downloaded/public byte identity against the candidate where the release gate promises it, and verify Latest/prerelease/unsigned wording matches the intended release policy. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02); [GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: OPEN.** U06-08: actual public metadata and downloaded asset identity readback. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3216).

<a id="criterion-v06-g-09-v1"></a>
- [ ] **V06-G-09.V1** — Exercise wrong SHA, moved tag, modified bytes, missing assets and absent lifecycle evidence as negative publication controls before using the real release path. **Trace:** [Stabilization release exit criteria](docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](docs/history/localmotive-comprehensive-audit.md#gh-02); [GH-03](docs/history/localmotive-comprehensive-audit.md#gh-03).
  **Current status: OPEN.** U06-08: nonpublishing negative controls on the actual promotion contract. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3220).

### V06-G-10

<a id="criterion-v06-g-10-i1"></a>
- [ ] **V06-G-10.I1** — Record final source/release IDs, exact asset digests, successful and skipped verification, closure evidence for completed findings and the remaining risk register. **Trace:** [Remediation and release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [QD-04](docs/history/localmotive-comprehensive-audit.md#qd-04); [GH-07](docs/history/localmotive-comprehensive-audit.md#gh-07); [What to measure next](docs/history/localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored).
  **Current status: OPEN.** U06-09: actual final release/evidence/deferral closeout. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3236).

<a id="criterion-v06-g-10-i2"></a>
- [x] **V06-G-10.I2** — Preserve old v0.4.1/v0.5 closeout evidence as history. Add a current status/evidence index and link each deferred finding or supplemental item to its continuing owner and milestone. **Closed (2026-09-12):** the v0.4.1/v0.5 records stay frozen in `docs/history/TODO-0.4.1.md` / `TODO-0.5.md` plus git history; the third-pass open-box reconciliation indexes every deferred finding with its owner and milestone/trigger, and the review carries the matching deferred rows. **Trace:** [Remediation and release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [QD-04](docs/history/localmotive-comprehensive-audit.md#qd-04); [GH-07](docs/history/localmotive-comprehensive-audit.md#gh-07); [What to measure next](docs/history/localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3237).

<a id="criterion-v06-g-10-i3"></a>
- [x] **V06-G-10.I3** — Schedule the agreed dependency/advisory, evidence-retention and measured-performance follow-up through the project's normal process; keep follow-up metrics separate from historical audit results. **Closed (2026-09-12):** dependency/advisory updates are scheduled weekly via `.github/dependabot.yml` (npm, cargo, github-actions, grouped) with `npm audit --audit-level=moderate` and the `rust-audit` job on every run; evidence retention keeps `release-evidence/<version>/` and its attestations in-repo and in git history; measured-performance follow-ups stay in the audit's "What to measure after correctness is restored" and remain separate from audit results (third-pass schedule block below). **Trace:** [Remediation and release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [QD-04](docs/history/localmotive-comprehensive-audit.md#qd-04); [GH-07](docs/history/localmotive-comprehensive-audit.md#gh-07); [What to measure next](docs/history/localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3238).

<a id="criterion-v06-g-10-v1"></a>
- [x] **V06-G-10.V1** — Re-run traceability/link validation on the final tracker and check that no task marked complete lacks its closure record and no deferred item has disappeared. **Closed (2026-09-12):** `node scripts/verify_tracker_links.mjs` -> all links resolved and every open id keeps a review row; every register id exists in the tracker; no id is both checked and open (TRACEABILITY OK; final run numbers in the third-pass record). **Trace:** [Remediation and release acceptance](docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [QD-04](docs/history/localmotive-comprehensive-audit.md#qd-04); [GH-07](docs/history/localmotive-comprehensive-audit.md#gh-07); [What to measure next](docs/history/localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored).
  **Current status: VERIFIED.** Inherited evidence scope; current release qualification remains open. [Source/evidence](https://github.com/sato942/localmotive/blob/1881db93c54f1c4a181ed5070d7c1449c50a7d74/docs/history/TODO-0.6.md#L3242).
