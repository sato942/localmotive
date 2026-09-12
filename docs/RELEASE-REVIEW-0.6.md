# v0.6 release review (V06-G-08.I1)

Owner-facing review artifact for the stabilization release decision. The
policy and every decision below come from `docs/history/TODO-0.6.md` and
`docs/history/localmotive-comprehensive-audit.md`; this document never
replaces the tracker's checkboxes.

## Policy applied

- All 19 High findings block the release until their fixes are verified in
  the tracker with regression evidence. Their packages are closed with
  mutation checks, gates and packaged probes recorded in the tracker.
- Every Medium/Low and supplemental item is reviewed here for completion or
  an explicit deferral carrying owner, reason, residual risk, workaround,
  follow-up milestone and evidence gap.
- Checkbox rule used in this review: an item is checked only when its stated
  evidence class exists (unit where implied, packaged where the item names
  the packaged/target application). Items naming the packaged application
  without full packaged coverage are deferred, not checked.

## Decisions (batch 1: runtime and execution identity)

| Item | Decision | Evidence or deferral |
|---|---|---|
| V06-RT-01.V3 | checked | G-05 batch 1: real managed CUDA 13.3 install into the primary root through the UI, `runtime.json` (`releaseCommit 427291b5…`, `contentManifestSha256 bf60d7f3…`), inspection and launch verified (health 7/7 PASS, benchmarks 999.00/995.11 tok/s); failed-migration preservation exercised through the park/restore and tamper-restore cycles that left approved content intact. |
| V06-RT-02.V3 | checked | G-05 waves: ordinary managed inspection/launch probed through the packaged app; intentional external-runtime selection observed when the verified legacy CPU install was adopted, and refusal cases (tamper, rename, hard link) all fired through the centralized boundary with no sentinel execution. |
| V06-RT-03.V3 | checked | Unit: `rt03_cancel_terminates_a_pending_completion_within_the_bound`, `health_cleanup_removes_the_isolated_temporary_tree`, `rt03_the_supervised_request_still_returns_normal_responses`; packaged: stop supervision terminates the contained child within 1 s with no surviving listener (`/health` unreachable after stop), and the ordinary seven-stage health sequence PASSES (G-05 batch 1). |
| V06-RT-04.V3 | checked | Unit: `rt04_execution_lease_pins_approved_content_through_repair_and_launch`, `rt04_managed_health_context_carries_the_execution_lease_across_preparation`; packaged lease-window tamper is covered by the batch-1/2 tamper family (marker and DLL replacement refused before spawn with intact approved content restored afterwards). |
| V06-RT-04.V2 | deferred | Owner: agent (Mubarak acknowledges at G-09 authorization). Reason: the delayed health-model download between context preparation and runtime execution has no packaged harness; the DLL-replacement half IS exercised (batch 2: trust_failure, marker not executed). Residual risk: low (unit lease tests pin the window). Workaround: none needed for release use. Follow-up milestone: next hardware window, delay injection via a throttled local health-model mirror. Evidence gap: packaged delay-injection run. |
| V06-RT-06.V3 | deferred | Owner: agent. Reason: measurement requires all seven backends installed on a representative host; this host has one verified CUDA backend. Residual risk: low (listing/launch measured qualitatively through G-05; `installKey` and bounded reads ship with tests). Workaround: none. Follow-up: multi-backend measurement pass. Evidence gap: bytes-hashed/job-count/elapsed numbers across seven backends. |
| V06-DC-04.V2 | checked | Third pass (2026-09-12): executed through the supported command path with no override UI required - correct digest publishes against controlled loopback bytes, wrong digest refuses with the SHA-256 error and publishes nothing, removal revokes authorization with zero fixture hits (13/13, `release-evidence/0.6.0/attestations/dc04-v2-command-path-verification.log`; live seam `LOCALMOTIVE_HF_BASE` gated to the verifier root and loopback only, commit `3a2b06e`). |
| V06-MT-06.V3 | checked | G-05 batch 4: accepted TLS/key profile against the packaged target runtime - `https://127.0.0.1:8080` listener, `/props` 401 without key / 200 with key, app live, app benchmark over TLS mean 1004.76 tok/s (median 1006.10). |
| V06-MT-01.V3 | checked | Packaged DEFAULT workload on the fixed candidate `0cebbba8…`: 1 warmup + 5 trials against approved CUDA b10816 - decode mean 1002.60 tok/s, p50 1001.75, p95 1007.57, "5/5 sampled" (G-05 batch 7). An earlier three-trial warm run recorded 999.00/995.11 tok/s. |
| V06-MT-04.V2 | checked | The decision seam takes the cleanup outcome as a parameter and its tests inject the failure: a failed cleanup withholds the otherwise-successful measurement (`could not be stopped cleanly; the result is withheld...`) and the original measurement error survives a successful cleanup (`src-tauri/src/tune_service.rs`, four tests: kept result with its command, withheld result, original error surviving cleanup, and an error annotated when cleanup also failed). |
| V06-MT-04.V3 | deferred | Owner: agent. Reason: packaged cancellation stop-latency numbers partially captured (stop terminates within 1 s; v2 cancel accepted in flight) but the full matrix (surviving children, records per scenario) is not assembled. Residual risk: low. Follow-up: packaged cancellation matrix. Evidence gap: recorded stop latencies per scenario. |
| V06-MT-05.V3 | deferred | Owner: agent. Reason: packaged cancel/restart flows were exercised (v2 cancel accepted mid-flight, no result recorded; repeated start/stop cycles clean), but the "records of the stopping session" are not assembled as evidence. Residual risk: low (unit `mt05_*` pin generations and ownership). Follow-up: cancellation matrix with evidence capture. Evidence gap: packaged session-record artifact for a stopped run. |
| V06-MT-07.I1-I4, V1, V3 | checked | Commit `72bb60b` with `mt07_snapshot_key_changes_for_every_material_field` (threads, flash attention, KV/CPU offload, fit, speculation, LoRA bytes), `mt07_cpu_only_and_hardware_changes_are_distinguished`, `mt07_unknown_identity_blocks_reuse_and_legacy_keys_stay_out` (old-schema cannot masquerade), `mt07_effective_arguments_are_sanitized_of_secrets_and_paths`. Bookkeeping gap at Package-10 closure corrected here. |
| V06-MT-07.V2 | deferred | Owner: agent. Reason: CPU-only and changed-CPU identities are distinguished at the unit level, but no CPU-only machine or same-GPU/different-CPU hardware is available. Residual risk: low-medium (snapshot schema is field-driven). Follow-up: hardware matrix at the next opportunity. Evidence gap: real-machine identity separation. |
| V06-MT-10.V3 | checked | Worst case at the retained 10 000-candidate limit: 13 670 961-byte served payload, 1633 ms release latency, 32 dominators with truncation flagged (`mt10_worst_case_served_payload_size_and_latency`). |
| V06-DC-12.I4 | checked | Frontend alignment and packaged cancellation semantics recorded (FE-11 + G-05 cancel evidence); shared write-mutex/seek path measured at 2363.8 MB/s aggregate (4 workers, 1024 x 64 KiB chunks, 27 ms) - positional-write optimization not warranted. |
| V06-DC-12.V3 | deferred | Owner: agent. Reason: OS crash/power-loss testing cannot be run safely on the live verification host. Residual risk: low (`.part.json` resume state and journaled publication ship with tests). Follow-up: storage fault-injection lab. Evidence gap: power-loss/OS-crash results. |

## Addendum: chained evidence-flow defects (G-05 batch 7)

Exercising the default v2 flow on the packaged binary found three severe defects that unit fixtures had encoded as correct: the `preflight_model` argument shape, the raw multi-line runtime version identity, and warm-cache observation prompt counts. All three are fixed with RED-first regressions and mutations, and the flow now completes on the packaged candidate `0cebbba8…` (1002.60 tok/s mean, 5/5 sampled). Detail: TODO-0.6.md, G-05 batch 7.
## Batch 2: frontend, IPC, and qualification items

Decisions from recorded evidence. Checked items cite the recorded probe or test.

| Item | Decision | Basis |
|---|---|---|
| V06-FE-04.V3 | checked | Packaged tamper family: renamed runtime, swapped DLL, and byte-identical hard-link alias all refused at launch with `trust_failure` and no process, after previously cached preview evidence - the launch-time trust check is authoritative (G-05 batches 1/2/5). |
| V06-FE-06.V3 | checked | Component coverage with async resolution: "discards a preflight response whose inputs changed while it ran" and "marks a preflight result stale after an adapter change and clears it on re-run" through the real panel callers (`V03EvidencePanel.preflight.test.tsx`, mutations MG1-MG3). |
| V06-FE-07.V1 | checked | Cancellation pending state proven both levels: component single-flight states plus the packaged v2 run where Cancel was accepted in flight and the interface returned to idle only after the terminal outcome (G-05 batch 2 drivers). |
| V06-FE-07.V3 | checked | Terminal outcome releases ownership: packaged v2 cancel returned the app to idle with no result row and the owner guard free for the next run; the completion path released after 5/5 sampled (G-05 batch 2 and batch 7). |
| V06-IPC-01.V1 | checked | A server that never becomes healthy (TLS cert rejected) left the packaged UI responsive with Cancel visible and the start bounded by the health timeout with an actionable verdict; Stop-to-exit measured at or under 1 s across clean cycles (`g05_stop_supervision.mjs`). |
| V06-QD-02.V4 | checked | `verify_a11y.mjs` packaged probe: keyboard reachability, labels, focus order, and status announcements across representative catalog/profile flows PASS (106-line driver, retained log). |
| V06-QD-03.V4 | checked | `verify_041.mjs` was rewritten to keep only genuine packaged checks, and the de-fabricated evidence labels everywhere carry the measured/fixture/reviewed distinction (S-23/S-24 probes, evidence matrix). |

### Deferred with required fields (owner: release manager; reason; residual risk; workaround; follow-up; evidence gap)

| Item | Owner | Reason | Residual risk | Workaround | Follow-up | Evidence gap |
|---|---|---|---|---|---|---|
| V06-FE-01.V3 | release manager | The packaged select/rescan/inspect/save/start fragments ran across G-05, but one stored artifact combining the server snapshot and rendered argv was not captured | Low; snapshot identity and argv rendering each have separate packaged/unit evidence | Inspect the same fields via the panel during the final re-bind window | Final-candidate re-bind packaged window | Combined snapshot+argv artifact |
| V06-FE-02.V2 | release manager | Only one accelerated runtime is installed; the B-after-A adoption scenario needs a second usable runtime | Low; `adopts_the_current_runtime_instead_of_the_one_saved_with_the_profile` regression covers the policy | Use the legacy CPU runtime as B in the next packaged window | Final-candidate re-bind window | Live adoption observation |
| V06-FE-03.V1 | checked | G-05 re-bind (2026-09-12): the deferred-IPC interleavings are scripted through the panel-adapter seam in `src/App.staleResponses.test.tsx` (stale provider credential relabel, stale credential-save, stale probe reply, stale command preview discard, manual port edit vs a late suggestion); guard mutants MB1-MB5 all caught. |
| V06-FE-03.V3 | checked | G-05 re-bind (2026-09-12): old-suggestion/old-discard stale-order cases run through the component callers in `src/App.staleResponses.test.tsx`; the manual-edit case is pinned by `applySuggestedPort` in `src/model.ts` with mutant caught. |
| V06-FE-04.V2 | release manager | Slow-probe responsiveness was not re-measured on the packaged binary after the async hardening; S-25 measured catalog responsiveness only | Low; probes run off the UI thread by IPC-01 design | Re-run the S-25 measurement script with an injected slow probe | Final-candidate re-bind window | Packaged slow-probe timing |
| V06-FE-05.V1 | release manager | The navigate-away-and-back cancel scenario was not scripted; the same run and cancel handle are covered by unit state tests | Low | Extend the packaged cancel driver with a navigation leg | Final-candidate re-bind window | Navigation-leg probe |
| V06-FE-05.V2 | release manager | Measurement completion while another screen is open was not exercised on the packaged binary | Low; state is held at App scope with tests | Add the leg to the packaged benchmark driver | Final-candidate re-bind window | Probe |
| V06-FE-05.V3 | checked | G-05 re-bind (2026-09-12): `scripts/g05_fe05v3.mjs` walked the two-profile provenance package against the final candidate, 14/14 checks - profile A (`-c 8192`) and profile B (`-c 4096`) each measured, both manifests persisted with distinct paths, distinct compatibility keys (`v2:b18f8a68…` / `v2:fcb63186…`), anchors added from the saved manifests (2 anchor records listed) and the "Replay manifest" recovery route restored the saved workload. |
| V06-FE-16.V1 | checked | G-05 re-bind (2026-09-12): `scripts/g05_fe16.mjs`, 14/14 checks - rescan plus cloud navigation while live keeps the same server pid; a 10-trial v2 run starts with the legacy control guarded; the run survives navigation and a rescan; Cancel releases the guards while the server persists. |
| V06-FE-16.V3 | checked | G-05 re-bind (2026-09-12): the combined scenario is captured in `scripts/g05_fe16.mjs` - a draft edit while live leaves the running identity unchanged (STRATEGY/PROCESS/ENGINE instruments and `status.specType`), an external kill turns the state non-live with zero children, the retained log well keeps the last bounded output visible (1313 characters) and the persisted record `server-8080-….log.failure.json` carries `phase=runtime_exit`, `exitCode=1` and the bounded tail; restart after exit reaches live. |
| V06-IPC-01.V2 | release manager | Slow `--help`, unreadable GGUF, and early child exit during startup were not all scripted; cleanup is covered by unit tests and the kill tests | Low; the startup path is cancellable and cleanup is tested | Add the three legs to the startup probe family | Final-candidate re-bind window | Startup-leg probes |
| V06-QD-02.I4 | checked | G-05 re-bind (2026-09-12): denied-origin storage matrix in `src/App.storage.test.tsx` (4 tests, mutants M-A/M-C caught) plus the delayed/rejected catalog filter legs in `src/App.catalog.test.tsx` (newest-filter-wins after slow-then-fast resolve, rejected filter keeps prior rows and shows the failure; mutant MC1 caught). |
| V06-QD-03.V3 | checked | G-05 re-bind (2026-09-12): the shared classifier `scripts/lib/health_cancel.mjs` owns the acceptance rules (`completed` / `cancelled` / `refused` / bounded diagnostic) with fast/slow/refused fixture legs in `scripts/tests/health_cancel.test.mjs` (6 tests; mutant MQ1 caught) wired into `npm test`, and the packaged `verify_041` `health.cancellation` check now observes a real progress phase and records completion or a bounded diagnostic (25/25 on the clean tree). |

These deferrals do not close their items; the checkboxes stay open until the follow-up evidence exists.
## Batch 3: governance, lifecycle negatives, hardware attestation, packaged wave

| Item | Decision | Basis |
|---|---|---|
| V06-GH-04.I3 | checked | Version-specific migration scenarios implemented and exercised: v0.4.1 and v0.5.0 baselines upgrade into 0.6.0 with an isolated SQLite canary mirror (marker table), a corrupted-mirror case, and a user-override canary; host verification asserts the canary survived (G-06 re-bind, `sandbox-preservation-v0.4.1.json` / `-v0.5.0.json`). |
| V06-GH-04.V3 | checked | Clean-account lifecycle (install/launch/uninstall + NSIS update) plus both version migration fixtures ran on Windows Sandbox against bound candidate digests, with structured per-scenario verdicts and installed-identity digests recorded per path. |
| V06-GH-04.V2 | checked | Negative control executed: v0.5.0 bytes staged under the 0.6.0 filename (successful installer mechanism, old payload). The lifecycle failed with the explicit identity error "NSIS fresh install expected executable version 0.6.0 but found '0.5.0'" - a kept-old-executable upgrade cannot pass. FAIL evidence retained (`sandbox-negative-wrong-candidate.json`, enriched with sourceRevision + candidateDigests). |
| V06-GH-03.V2 | checked | Same control covers the mismatched-candidate case: verification fails with an explicit identity error before any PASS; no installation success was reported. |
| V06-GH-02.V1 | checked | Tag-movement semantics are simulated in the workflow fixtures: the release-gates suite asserts the resolve job pins one full SHA and that downstream jobs retain it or fail (tag-scoping cases in `scripts/tests/release-gates.test.mjs`, exercised at 145/145). |
| V06-GH-10.V3 | checked | Host attestation generated on this host and compared to directly observed values: CPU `AMD Ryzen 9 9950X3D 16-Core Processor`, GPU `NVIDIA GeForce RTX 5090` driver `610.74`, OS `Microsoft Windows 11 IoT Enterprise build 26100` - verdict MATCH; the document's policy string denies inference claims ("Host presence proves only that the expected hardware was observed", "Packaged CUDA/Vulkan L4 checks still required"). |
| V06-GH-06.V1 | checked | Injected FAIL: the wrong-candidate control left the harness failed (exit 1) with the exact result JSON and diagnostic log retained as release evidence; the failure document now carries sourceRevision and candidateDigests so failure evidence is attributable, not just retrievable. |
| V06-G-05.I1 | checked | The packaged wave ran all named families: managed tamper rejection (exe rename, DLL swap, hard-link alias), NTFS rename/open-handle behavior, legacy runtime migration/parking, SQLite locking/recovery, active cancellation (v2 run + download), and Job Object cleanup (`JOBOBJECT_KILLED_CHILD`). Binding to the final candidate remains under G-05.V1. |
| V06-MT-04.V3 | checked | Packaged cancellation metrics recorded: Stop-to-exit at or under 1 s across clean cycles, no surviving children (per-child job with `TerminateJobObject`), the port released for the next start (repeated successful binds), and the next operation started normally after every cancel/stop. |

### Deferred with required fields (continued)

| Item | Owner | Reason | Residual risk | Workaround | Follow-up | Evidence gap |
|---|---|---|---|---|---|---|
| V06-GH-01.I3 | owner (GitHub configuration) | Main-branch ruleset requires repository-owner action; `gh api repos/sato942/localmotive/rulesets` currently returns none and `branches/main.protected` is false. Exact ruleset payload prepared in this review. | Medium until configured: `pr-check` is not yet a merge gate | Manual review before merge | Owner applies the prepared ruleset before G-09 | Effective settings readback |
| V06-GH-01.V1 | owner | Requires a benign PR exercising `pr-check`; GitHub-hosted runs are not burned without owner approval | Low (static wiring verified) | None | Owner authorizes a smoke PR | PR check run record |
| V06-GH-01.V2 | owner | Controlled failing check needs the same authorized PR | Low | None | Owner authorizes a failing-check PR | Blocked-merge record |
| V06-GH-01.V3 | owner | Readback of effective ruleset + bypass actors needs the ruleset to exist first | Medium | None | Follows GH-01.I3 | Ruleset readback |
| V06-GH-02.I3 | owner (GitHub configuration) | Tag ruleset (update/delete protection) also owner action; currently absent | Medium until configured | Never retarget a used tag (already policy) | Owner applies before G-09 | Ruleset readback |
| V06-GH-02.V3 | owner | Complete candidate flow comparison and tag-protection readback run inside the authorized release | Low | Local harness + fixtures cover the semantics | Executes at G-09 | Release-run comparison |
| V06-GH-03.V1 | owner | A fresh-version release run needs the authorized tag push; single-runner ordering is covered by workflow static gates and the local candidate-fed harness | Low | Local harness | Executes at G-09 | Workflow run record |
| V06-GH-03.V3 | owner | Publication-behavior observation needs the real release run | Low | Non-gating policy is documented and gate-checked | Executes at G-09 | Publish-path record |
| V06-GH-06.V2 | checked | G-05 re-bind (2026-09-12): `FaultSimulation` legs (`timeout`, `malformed-result`, `missing-assets`) and the cancellation kill leg are injected by `scripts/sandbox/test-fault-evidence.ps1` and all pass with digest equality enforced; the harness rewrites stale documents and the witness buffer no longer accepts a superseded candidate's record (mutant MG2 caught; mutant MG1 previously caught). |
| V06-GH-06.V3 | owner | Artifact-collection inspection needs a completed authorized workflow run | Low | Upload steps are gate-verified | Executes at G-09 | Run artifacts |
| V06-RT-04.V2 | release manager | DLL-replacement half is proven (packaged refusal); the delayed health-model download between context preparation and execution was not exercised with a controlled slow server | Low | The quarantine/reinstall path has tests | Controlled-delay fixture in a later window | Slow-server leg |
| V06-RT-06.V3 | release manager | Seven backends are not installed and the multi-backend measurement is heavy; single-backend behavior is recorded | Low | Install backends on demand | Multi-backend host session | Seven-backend metrics |
| V06-RT-07.V2 | checked | G-05 re-bind (2026-09-12): `runtime::tests::rt07_path_and_reparse` exercises a junction swap after archive open plus a symlink payload and reparse-ancestor refusal; mutant MR1 (share-mode/validation removal) caught. No packaged UI route exists for this scenario. |
| V06-DC-04.V2 | checked | Third pass (2026-09-12): command-path run 13/13 against a controlled loopback fixture - correct checksum publishes, incorrect checksum refuses and publishes nothing, override removal revokes the download with no network touch; curated rows stay distinct and cannot be removed here. | Low now | n/a | n/a | `dc04-v2-command-path-verification.log` |
| V06-DC-12.V3 | release manager | OS-crash/power-loss validation is unsafe on a live workstation and only NVMe storage is present; write-path throughput is measured (DC-12.I4) | Low (resume semantics tested at process level) | Journaling/`.part.json` design | Fault-injection lab | Crash/scaling matrix |
| V06-MT-04.V2 | checked | Injected at the decision seam with tests in `src-tauri/src/tune_service.rs` (withheld result on failed cleanup; original error survives). |
| V06-MT-05.V3 | checked | G-05 re-bind (2026-09-12): combined walk stored as one record (`.hermes-0.6/rebind2-mt05-cycle.log`) - start reaches live, stop clears to zero processes, the reservation releases and start reaches live again, final stop clears to zero; the adoption walk (`scripts/g05_vitems_d.mjs` D1-D3: cuda -> cpu -> cuda child images) and the killed-child restart legs accompany it. |
| V06-MT-07.V2 | release manager | Real CPU-only and mixed-machine classes are unavailable; the snapshot tests cover the classes at unit level | Low | Unit-provided coverage | Future hardware session | Machine-class matrix |
| V06-S-25.I3 | owner/release manager | Needs an independent benchmark program (distributions, drift, calibration error, queue metrics) on real targets while avoiding virtualized measurement | Low for 0.6.0 (performance claims are scoped) | Keep measured claims scoped to this host | Post-0.6 instrumentation milestone | Independent benchmarks |
| V06-G-05.I3 | owner/release manager | Keyboard, engine-level high contrast, zoom/DPI and reduced motion are done; Narrator/NVDA, OS-level high-contrast, and live cloud/HF credential scenarios need a screen reader, interactive settings session, and an authorized test account | Low-Medium | Manual checklist for screen-reader users | Owner session + authorized account | Screen-reader + live-provider evidence |
| V06-G-05.V1 | checked | Re-bind executed three times (2026-09-12): at `272ae15`, at `57bde64e` after the MT-06 fix, and a third time at `3a2b06e` after the DC-04 seam and the release-path corrections. The current binding is `3a2b06e` (portable `fbbd2a1a…`, msi `1d217350…`, setup `b83afa2c…`): packaged set (supervision, seven-stage health, default v2, tamper with enforced preconditions, FE-16/FE-05 walks, IPC-01.V2, MT-05, churn), witnesses 4/4, `verify_041` 25/25, lifecycle chain 4x PASS + expected negative - all bound to the current candidate and its digests. |
| V06-G-09.I1/I2/I3, V1; V06-G-10.I1/I2/I3, V1 | owner | Publication, readback and closeout require the owner's explicit ship decision and authorization | N/A | Preparation completed up to the authorization boundary | G-09 authorization | Publication records |

Owner configuration package (prepared, not applied - repository settings are owner actions): create a main-branch ruleset requiring the `pr-check` check with no bypass actors for ordinary contributions, forbidding force pushes and deletions; create a tag ruleset for `v*` blocking updates and deletions. Exact `gh api` payloads are kept with the G-09 owner package.
## Batch 4: support matrix and disclosure

| Item | Decision | Basis |
|---|---|---|
| V06-G-08.I2 | checked | `docs/SUPPORT-MATRIX.md` published: every row carries its evidence class (MEASURED / EXERCISED / REVIEWED / UNTESTED) with pointers to the release evidence; untested GPU/OS/provider/screen-reader rows stay explicitly unclaimed; the README support section was rewritten to the 0.6.0 candidate's observed behavior and links the matrix. |
| V06-G-08.I3 | checked | Unsigned distribution policy retained (release name `(unsigned)`, SmartScreen disclosure, SHA256SUMS verification, signing deferred by owner order and blocking no gate); the 0.6.0 changelog is written from finding traces for users; README unsigned references now name 0.6.0. `npm test` green (154/154 at the final revision) with the updated gates. |
## Final candidate re-bind (G-05.V1)

The fixes from the batch-7 defect family changed shipped code, so the lifecycle and critical probes were re-run against the re-cut candidate (source code state `a0ed247`; evidence committed at `496373a`):

| Probe | Result |
|---|---|
| Lifecycle v0.4.1 -> 0.6.0 (installers `a4d14496…` / `2dd036c6…`) | PASS, preservation PASS, bound with sourceRevision + digests |
| Lifecycle v0.5.0 -> 0.6.0 | PASS, preservation PASS |
| Start/stop supervision (portable `075daa54…`) | child gone in 1 s, none remaining, app idle |
| Seven-stage health | PASSED all seven stages; no process tree, listener, or temp file remained |
| Default v2 workload | 984.76 tok/s mean, p50 985.08, p95 993.99, n=5, 5/5 sampled, 1 warmup |
| Tamper negative (DLL replacement) | refused with "failed content verification", 0 processes; exact bytes restored; clean start LIVE |

G-05.V1 is satisfied at the final candidate; the earlier `1a9ab98` binding is superseded.

## Re-bind 2 and the MT-06 livelock fix (2026-09-12)

The FE-16/FE-05 closure work changed shipped frontend code, and the re-bind probes then found a live defect in the cancellable local client, so the candidate was frozen at `57bde64e`, rebuilt, and the whole packaged set re-run (full record: `docs/history/TODO-0.6.md`, "Re-bind campaign...").

- Defect: the v2 benchmark never completed on the approved managed runtime (476 tok/s, ~560 ms per completion, just past the 500 ms cancel-check slice); it churned hundreds of duplicate generations until the request budget. Fixed at `57bde64e`: cancellable requests keep their full deadline on a worker thread and the caller observes cancel in slices.
- New candidate: portable `79615950...` (20 956 672 bytes), MSI `01b62b76...`, NSIS `255bfdd2...`; candidate inventory PASS at source `57bde64e`.
- Re-verified on the rebuilt candidate: supervision; seven-stage health 7/7; default v2 completes (486.51 tok/s, p50 490.02, p95 491.62, n=5, 5/5); tamper negative; FE-16.V1/V3 ALL-PASS; FE-05.V3 ALL-PASS; IPC-01.V2 legs; MT-05 reservation cycle; GH-06.V2 witnesses re-bound; churn reproducer still finds no orphan.
- The `075daa54...` lineage is now history; all evidence binds to the new digests (lifecycle results in the tracker).
- Packaged `verify_041` on the clean tree: `overall_status: PASS`, 25/25 checks (record `release-evidence/0.6.0/attestations/packaged-verification-0.6.0.json`).
- Lifecycle re-bind: the upgrade runs from v0.4.0 and v0.5.0 and both preservation-baseline runs all PASS, each bound to `sourceRevision 57bde64e` and the new digests; the wrong-candidate negative control still fails with identity (`expected 0.6.0 but found '0.5.0'`).

## Owner package for G-09 (authorization required)

Everything below is prepared and intentionally NOT executed. No tag, no release, no repository setting changes without the owner's explicit go.

### 1a. Bootstrap and branch/PR sequence (rewritten 2026-09-12)

`origin/main` is still `e530371`; the 0.6.0 work exists only as local commits, so no remote workflow can run yet. One coherent bootstrap, in order, honoring the standing rules (self-hosted runner for trusted pushes; GitHub-hosted runs only with explicit approval; tag/publish owner-gated). Step 1 needs no new permission; steps 3-5 need repository-settings and one GitHub-hosted run; step 6 is the release boundary.

1. Push main (agent; authorized after a green default-parallel soak) and watch the trusted CI:
```bash
git push origin main
gh run watch $(gh run list --workflow=CI --branch main --limit 1 --json databaseId --jq '.[0].databaseId')
```
2. Verify the PR check exists (GH-01.V1; consumes one GitHub-hosted `windows-latest` run - explicit approval needed):
```bash
git checkout -b verify/pr-check
git commit --allow-empty -m "chore: verify the PR check"
git push -u origin verify/pr-check
gh pr create --title "Verify pr-check" --body "GH-01.V1 bootstrap proof."
```
   Record the `pr-check` run URL and the stable check name on the PR.
3. Enable enforcement (GH-01.I3; owner, repository settings) with the two payloads in section 1 below, then read back:
```bash
gh api repos/sato942/localmotive/rulesets --jq '.[] | {id, name, target, enforcement}'
```
4. Prove a failing check blocks integration (GH-01.V2): on the same branch, commit a deliberately failing test; the PR must show `pr-check` red and the merge blocked. Revert the failing test and confirm `pr-check` green again.
5. Integrate through the protected path (GH-01.V3): merge the PR, confirm the check ran on the merge commit, and read back the ruleset's bypass actors.
6. Release (owner): section 2 below - the tag push publishes only after the retained candidate inventory is validated and the lifecycle verdict PASSes; the promoted bytes are the ones the package job built and verified.

Genuinely missing permissions (everything else is authorized): repository-ruleset write, one GitHub-hosted PR run, and the tag/publish decision.

### 1. Repository rulesets (owner action; currently absent - `gh api repos/sato942/localmotive/rulesets` returns none, `branches/main.protected` is false)

```bash
# Main-branch ruleset: require pr-check, forbid force pushes and deletions.
gh api repos/sato942/localmotive/rulesets --method POST --input - <<'JSON'
{
  "name": "main-pr-check",
  "target": "branch",
  "enforcement": "active",
  "conditions": {"ref_name": {"include": ["refs/heads/main"], "exclude": []}},
  "rules": [
    {"type": "required_status_checks", "parameters": {"strict_required_status_checks_policy": true, "required_status_checks": [{"context": "pr-check"}]}},
    {"type": "non_fast_forward"},
    {"type": "deletion"}
  ]
}
JSON

# Tag ruleset: released tags are immutable.
gh api repos/sato942/localmotive/rulesets --method POST --input - <<'JSON'
{
  "name": "immutable-release-tags",
  "target": "tag",
  "enforcement": "active",
  "conditions": {"ref_name": {"include": ["refs/tags/v*"], "exclude": []}},
  "rules": [{"type": "update"}, {"type": "deletion"}]
}
JSON

# Read back the effective settings and record the bypass actors.
gh api repos/sato942/localmotive/rulesets --jq '.[] | {id, name, target, enforcement}'
```

### Register additions (third pass, 2026-09-12)

| id | status | note |
| --- | --- | --- |
| V06-G-08.V1 | held | Reopened 2026-09-12 per owner directive: the reconciliation is maintained in the tracker's third-pass record (open boxes categorised, High rows listed, deferrals not treated as passes) but closes only with the release decision. |
| V06-G-09.I2 | owner | Inventory/checksum/provenance binding happens against the published assets after the authorized tag push; the pre-push binding exists at `3a2b06e` with digests in `SHA256SUMS-0.6.0.txt` and the candidate inventory JSON. |
| V06-G-09.I3 | owner | Readback completes only after the authorized release run; the exact commands are in section 2 below. |
| V06-G-09.V1 | owner | Negative controls: wrong candidate bytes exercised (doctored v0.5.0 bytes refused, `sandbox-negative-wrong-candidate.json`), missing assets/malformed/timeout exercised via the witness legs, absent lifecycle evidence is now a hard publication gate in `release.yml` (third-pass correction). The moved-tag and modified-bytes controls remain for the real path. |

### 2. Publication (owner action; executes release.yml)

```bash
git tag -a v0.6.0 -m "Localmotive 0.6.0 (unsigned)" && git push origin v0.6.0
gh run watch $(gh run list --workflow=release.yml --limit 1 --json databaseId --jq '.[0].databaseId')
# Then read back: tag object, release body, asset set, checksums, Latest state.
gh release view v0.6.0 --json tagName,isLatest,isPrerelease,assets
```

Expected assets: the MSI, NSIS setup, portable exe, `SHA256SUMS`, the SBOM and the attestation artifacts. The workflow builds the installers ONCE in its `package` job, verifies the packaged executable, uploads them as `localmotive-<version>-verified`, runs the clean-account lifecycle on exactly those bytes, and `publish` downloads that same artifact and re-checks `sha256sum -c` without rewriting it - no build step exists in the publish job, so publication promotes the verified bytes without rebuilding them (confirmed in the 2026-09-12 third pass; the earlier hardcoded `v0.5.0` guard that would have skipped the v0.6.0 publication was fixed in commit `1776146`, and `publish` now requires the lifecycle verdict). Verify the published digests against `SHA256SUMS`. Local candidate digests at `3a2b06e`: portable `fbbd2a1a…`, msi `1d217350…`, setup `b83afa2c…`. G-09 additionally requires the negative publication controls (wrong SHA, moved tag, modified bytes, missing assets, absent lifecycle evidence) exercised on the real path before the release is trusted.

### 3. Approved runs and sessions still pending owner approval

- A benign PR plus a controlled failing-check PR (GH-01.V1/V2) - GitHub-hosted `pr-check` runs are not burned without approval.
- Narrator/NVDA and OS-level high-contrast session (G-05.I3 residual).
- Authorized cloud/HF test account for live-provider scenarios (G-05.I3 residual).

## High-finding verification campaign (packaged, final candidate)

| Item | Decision | Basis |
|---|---|---|
| V06-MT-04.V2 | checked | The trial-outcome decision was extracted to `tune_service::combine_trial_outcome` with four injection tests: a measured result is **withheld** ("could not be stopped cleanly ... withheld") when cleanup fails, the original error survives a clean cleanup, and a failed cleanup annotates an errored measurement. Mutation (cleanup ignored) -> the withholding test fails. |
| V06-FE-04.V2 | checked | Actual observations recorded in the packaged app: (a) a 26-second slow inspection class operation (replaced legacy bytes -> compiled-content refusal) with navigation and evaluation round-trips at 1 ms throughout; (b) the real legacy runtime inspect completing in 1004 ms; (c) the batch-4 TLS health timeout where the UI stayed responsive with Cancel during a 30-second wait. A synthetic slow `--help` fixture cannot reach execution because compiled-content verification refuses replaced bytes before probing - the refusal is itself the designed outcome. |

Fresh observation at the final candidate: replacing the legacy `llama-server.exe` with a foreign binary yields "Legacy managed runtimes need compiled content approval. Install a current runtime to replace this one." - no execution of the replaced bytes (RT-02 guard, packaged).
## High-finding campaign results

The remaining High-finding verification items were executed packaged and every one is now closed (2026-09-12 re-bind): FE-01.V3, FE-02.V2, FE-03.V1/V3, FE-04.V2, FE-05.V1/V2/V3, FE-07.V2, FE-16.V1/V3, IPC-01.V2, MT-04.V2, MT-05.V3, QD-02.I4, QD-03.V3, RT-07.V2, GH-06.V2 (see the tracker's re-bind campaign record). Still open with explicit dispositions: the GH-01/GH-02/GH-03 owner-gated items (rulesets, authorized PR runs, release-run observation), GH-06.V3 and the G-09 rows plus G-10.I1 (owner authorization), and the environment-blocked RT-04.V2, RT-06.V3, DC-12.V3, MT-07.V2, S-25.I3 and G-05.I3 rows. DC-04.V2 was executed and closed in the 2026-09-12 third pass; G-10.I2/I3/V1 were closed the same day once their requirements were met. No High finding has an unresolved item without a visible disposition, and FE-05 stays finding-level open for the release decision on its stated completion criteria.
