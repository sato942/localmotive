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
| V06-DC-04.V2 | deferred | Owner: agent. Reason: `save_user_catalog_override` has no frontend surface to drive from the UI (recorded in G-05 batch 3). Residual risk: low (authority behavior covered by Rust `override_authority_tests`). Workaround: none for end users (no override UI exists). Follow-up: a reviewed override UI or documented maintenance command, then the controlled-server run. Evidence gap: packaged correct/incorrect-SHA override exercise. |
| V06-MT-06.V3 | checked | G-05 batch 4: accepted TLS/key profile against the packaged target runtime - `https://127.0.0.1:8080` listener, `/props` 401 without key / 200 with key, app live, app benchmark over TLS mean 1004.76 tok/s (median 1006.10). |
| V06-MT-01.V3 | checked | Packaged DEFAULT workload on the fixed candidate `0cebbba8…`: 1 warmup + 5 trials against approved CUDA b10816 - decode mean 1002.60 tok/s, p50 1001.75, p95 1007.57, "5/5 sampled" (G-05 batch 7). An earlier three-trial warm run recorded 999.00/995.11 tok/s. |
| V06-MT-04.V2 | deferred | Owner: agent. Reason: no cleanup-failure injection harness exists at the packaged layer; the failure path surfaces as the visible "did not stop" error in `stop_server`. Residual risk: low. Follow-up: fault-injection test in `proc.rs`. Evidence gap: injected cleanup-failure result. |
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
| V06-FE-03.V1 | release manager | The exact deferred-IPC interleavings (A resolves after B across credential/model/probe) were not scripted at the component boundary | Low; single-flight state machine has unit coverage | Script the interleaving against the panel adapter seam | Next frontend hardening pass | Interleaving fixtures |
| V06-FE-03.V3 | release manager | Old port suggestion/old command discard scenarios were not scripted through component callers; the FE-04 argv gate covers the command path | Low | Extend the panel tests with the two stale-order cases | Next frontend hardening pass | Component-level stale-order fixtures |
| V06-FE-04.V2 | release manager | Slow-probe responsiveness was not re-measured on the packaged binary after the async hardening; S-25 measured catalog responsiveness only | Low; probes run off the UI thread by IPC-01 design | Re-run the S-25 measurement script with an injected slow probe | Final-candidate re-bind window | Packaged slow-probe timing |
| V06-FE-05.V1 | release manager | The navigate-away-and-back cancel scenario was not scripted; the same run and cancel handle are covered by unit state tests | Low | Extend the packaged cancel driver with a navigation leg | Final-candidate re-bind window | Navigation-leg probe |
| V06-FE-05.V2 | release manager | Measurement completion while another screen is open was not exercised on the packaged binary | Low; state is held at App scope with tests | Add the leg to the packaged benchmark driver | Final-candidate re-bind window | Probe |
| V06-FE-05.V3 | release manager | Distinct-provenance comparison across two profiles and the saved-manifest recovery route were not walked in the packaged app | Low; provenance fields and record persistence have unit/integration coverage | Walk both records through the calibration panel next window | Final-candidate re-bind window | Packaged record walk |
| V06-FE-16.V1 | release manager | Overlapping scan/cloud/start orderings were unit-covered but not scripted through the packaged UI | Low | Script two orderings against the packaged app | Final-candidate re-bind window | Ordering probes |
| V06-FE-16.V3 | release manager | Edit-while-running plus unexpected-exit was partially observed (kill tests, identity tests) but not captured as one scenario | Low | Extend the stop-supervision driver with a draft edit leg | Final-candidate re-bind window | Combined scenario |
| V06-IPC-01.V2 | release manager | Slow `--help`, unreadable GGUF, and early child exit during startup were not all scripted; cleanup is covered by unit tests and the kill tests | Low; the startup path is cancellable and cleanup is tested | Add the three legs to the startup probe family | Final-candidate re-bind window | Startup-leg probes |
| V06-QD-02.I4 | release manager | Denied browser storage and the delayed/rejected IPC matrix were not fully scripted; corrupt-JSON quarantine and filter/hardware updates have tests | Low | Extend the catalog component suite with denied-storage cases | Next frontend hardening pass | Storage-denial fixtures |
| V06-QD-03.V3 | release manager | Cancellation under fast/slow fixture progress waits for the active operation; only the real-run cancel was exercised packaged | Low | Add fast/slow fixture legs to the QD-03 driver | Final-candidate re-bind window | Fixture-progress probe |

These deferrals do not close their items; the checkboxes stay open until the follow-up evidence exists.
