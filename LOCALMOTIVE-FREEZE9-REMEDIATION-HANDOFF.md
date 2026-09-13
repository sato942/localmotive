# Localmotive freeze-9: fixed remediation handoff

**Reviewed checkpoint:** `c4526c1d453d7b23272786aa53b6eb0592a05bf2`  
**Candidate producer:** `eb01bc9f97417e74ce7de7da85c86582f42ceb67`  
**Handoff date:** 2026-09-13  
**Purpose:** Clarify the six outstanding workstreams from the last review without restarting the audit or changing the existing universal goal.

## Instructions to the implementing agent

Continue the existing universal goal. This document replaces the compressed follow-up message about this checkpoint; it does not create another `/goal` or `/subgoal`.

Read the entire handoff before changing code. Record the current HEAD and compare it with the reviewed checkpoint. If a listed issue has already been fixed by a later commit, demonstrate that fact and close the corresponding handoff row without repeating the fix. Do not reset to the reviewed commit or overwrite user work.

Use the six fixed IDs `F9-01` through `F9-06` below in the repository tracker alongside the existing audit/V06 IDs. These are review labels, not replacement audit findings. The mappings below identify existing acceptance requirements; verify them in the authoritative tracker when recording the work. Keep the original task wording unless the owner approves a scope change.

Do not launch another comprehensive audit for this handoff. A new observation belongs in a separate note unless it demonstrates a regression introduced by these changes or directly prevents an existing acceptance criterion from being satisfied. Explain that relationship before adding work. Do not add optional cleanup, new features, extra hardware matrices, or arbitrary repetition counts to the completion gate.

## What is already credited

The review confirmed the following at the pinned checkpoint. Preserve these fixes and their evidence rather than reopening the entire finding family:

| Area | Accepted progress and its limit |
| --- | --- |
| CI | [Run 34730128073](https://github.com/sato942/localmotive/actions/runs/34730128073) passed. Its logs report 613 Rust tests passed, 7 ignored; 141 Vitest tests passed; 208 Node tests passed. This establishes the configured CI result, not every release acceptance criterion. |
| Candidate identity | The current inventory and manifest agree on the freeze-9 producer and the three reported artifact digests. The previous stale-inventory finding is resolved. |
| Manifest baseline | The current manifest validates 18 records. The three dependency-free manifest/payload/promotion suites passed 32/32 in the review environment. F9-03 concerns semantic rejection of contradictory records, not an allegation that the current artifact identities are wrong. |
| Worker draining | The fixed 300-second ceiling is gone. Preparation and measured work share a client; draining continues past the expected bound while workers remain active. F9-02 is a separate error path introduced around client construction. |
| SEC1 | Complete raw SEC1 structures are passed to the parser, with regressions. Do not repeat the earlier scalar-versus-DER defect. |
| Lifecycle ownership | The harness now holds an OS file handle that excludes competing writers, and witnesses exercise a real second process and takeover after owner termination. Do not repeat the old check-then-write lock-acquisition finding. |
| Staged installers | Copied installer bytes are rechecked against the inventory. Final manifest validation has gained lifecycle schema/scenario/identity checks. |
| Installed payloads | The portable/NSIS/MSI differences are explained and checked as the three-byte bundle marker, with other bytes compared and separate shell probes. Do not repeat the unexplained-digest finding or describe shell probes as broader measurements than they are. |
| Release architecture | Verification and explicitly authorized promotion are separate. Promotion consumes a qualified bundle without rebuilding it, and public packaged JSON is compared with producer bytes. F9-04 concerns the remaining producer/consumer integration defects. |
| RT-06 | The final evidence measures four installed backends, selects and launches all four, and records three expected vendor refusals. All four final health runs passed on their first attempt. Preserve that scoped 22/22 result. |

The native v0.5.0 upgrade and v0.4.1 preservation records exist for freeze-9. The v0.4.0 upgrade and v0.5.0 preservation records are explicitly carried from the prior candidate with a ledger. The review did not rerun Windows Sandbox. Its intermittent startup and retained timeout are a real environment limitation; do not relabel carried records as native freeze-9 runs.

## F9-01 — Make cancellation/restart evidence identify the operation being tested

**Type:** Confirmed verification gap; the proposed production root cause is unproven.  
**Trace:** Audit MT-05/MT-06; `V06-MT-05.V1`, `V06-MT-05.V2`, `V06-MT-05.V3`, `V06-MT-06.V3`, `V06-G-04.I3`; follow-up R11.

**Observed facts at the reviewed checkpoint:**

- The committed result records `processingAtAttempt = "0"`, an enabled button, and `uiAccepted = true`.
- The driver clicks the UI Run button before issuing its direct API attempt. A refusal of that second call can therefore be caused by the replacement benchmark just started through the UI.
- The “previous active” predicate accepts observations made earlier, at cancellation or helper entry, rather than requiring activity at the actual replacement invocation.
- Its refusal predicate can also pass for an accepted API attempt when the UI state is disabled.
- The driver selects the newest benchmark record. On a null outcome it runs another benchmark and substitutes that retry's record; the original selected record identity is not retained in the final cycle result.

Sources: [restart action order](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/scripts/g05_mt06_cycles.mjs#L218), [retry substitution](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/scripts/g05_mt06_cycles.mjs#L389), [recorded attempt](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/release-evidence/0.6.0/attestations/mt06-cycles-result.json#L130).

**What this proves:** The final 59/59 does not establish refusal while the preceding cancelled request was still active. It does not, by itself, prove that production code loses cancellation between trials. The workload implementation contains between-trial cancellation handling; investigate with correlated records before changing that behavior.

**Required work:**

1. Associate each original run, cancellation, restart attempt, saved record, and retry with an unambiguous run identity. Use an existing identifier or narrowly add the instrumentation needed; do not use “newest file” alone when multiple runs can exist.
2. Preserve the original selected record and its outcome separately from every retry. A successful retry is a separate observation and must not overwrite the original assertion.
3. Exercise UI and direct API replacement attempts in separate scenarios. The API refusal test must not start a UI replacement first.
4. Hold the original request active at the actual attempted replacement using a controlled delay/barrier or an equivalent deterministic setup. Observe ownership/request activity immediately around that invocation and observe overlap from that point, not after a later settling delay.
5. If the old request has already ended, classify the timing attempt as not exercised. That is neither proof of a product failure nor a passing test of the active-request boundary.

**Close when:** A correlated record shows the old operation active at the invocation, the expected conflict/refusal or correct serialization, its subsequent termination, and an eventual successful replacement without overlap. Negative controls must reject prohibited backend overlap and substitution of an unrelated saved record. The active-request coverage assertion must classify an already-inactive attempt as not exercised; that classification is not a product failure. Diagnose a production terminal-outcome defect only if the correlated evidence reproduces one. No arbitrary count of lucky timing reruns is required.

## F9-02 — Release active benchmark state when client construction fails

**Type:** Source-confirmed product error path; not reproduced on Windows during review.  
**Trace:** Audit MT-05/MT-06; `V06-MT-05.I1`, `V06-MT-05.V3`, `V06-MT-06.I1`, `V06-MT-06.V1`; related ownership requirements from R04.

**Observed path:** `benchmark_v2` publishes the active benchmark slot and then calls fallible `local_client(&server.profile)?` before entering the result/cleanup boundary. An error there bypasses the later slot clear. Client construction rereads certificate/API-key files, while the server snapshot primarily checks the running validated server and clones its profile. A relevant file becoming invalid after server launch can therefore leave the benchmark slot occupied even though no benchmark started.

Sources: [slot publication and client construction](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/src-tauri/src/measurement_service.rs#L518), [client file reads](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/src-tauri/src/local_client.rs#L465).

**Required work:** Construct the fallible client before publishing the active slot, or use an ownership-safe cleanup guard covering every exit. Choose the smallest correct design. Do not remove the new worker-drain protections, and never clear a replacement operation's state from an older operation's cleanup.

**Close when:** A behavior regression induces client-construction failure, observes a truthful failure with no permanently occupied benchmark state, repairs the input, and starts a subsequent benchmark successfully. Use temporary local test credentials/certificates; no live cloud/HF account is needed. If Windows execution is unavailable, complete the implementation and meaningful local regression, and identify the remaining packaged check separately rather than claiming it ran.

## F9-03 — Validate the actual packaged-verification producer schema

**Type:** Confirmed validator defect, demonstrated with isolated semantic mutations.  
**Trace:** Audit GH-02; `V06-GH-02.I2`, `V06-GH-02.V2`, `V06-G-03.V2`, `V06-G-09.I2`, `V06-G-09.V1`; follow-up R08/R12.

**Observed mismatch:** The real packaged record uses `source_revision`, `artifact.name`, `artifact.size_bytes`, `artifact.sha256`, and per-check `status`. The validator checks optional `sourceRevision` and `candidatePortableSha256` instead. Its generic failed-check test only checks `ok === false`. The test fixture models the alternate camelCase shape rather than the actual producer shape.

Sources: [actual producer record](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/release-evidence/0.6.0/attestations/packaged-verification-0.6.0.json#L1), [validator branch](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/scripts/verify_qualification_manifest.mjs#L498), [test fixture](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/scripts/tests/lib/manifest_fixture.mjs#L154).

**Review reproduction:** A coherent fixture using the actual producer schema passed. Changing only its source revision passed. Changing only its artifact digest passed. Marking a required check `FAIL` while retaining an overall PASS also passed. Record hashes/count metadata were refreshed where needed so these tested semantic contradictions rather than incidental file-hash mismatches.

**Required work:** Make the producer schema explicit and required. Compare its source and portable artifact identity with the current producer inventory/manifest, including the expected name and size. Enforce per-check outcomes and their consistency with the aggregate result. If supporting more than one schema is necessary, identify and validate each explicitly; absence of required fields must not silently skip validation.

**Close when:** Tests derived from a genuine producer record show: valid unchanged record PASS; wrong source FAIL; wrong digest FAIL; wrong size/name FAIL; missing required identity FAIL; failed required check with an aggregate PASS FAIL. Refresh record hashes in semantic mutations so rejection demonstrates the relevant relationship check. Keep existing inventory, lifecycle, carry-forward, newline, and path protections working. This task does not allege that the presently committed candidate bytes are wrong.

## F9-04 — Connect the release producers and consumers and retain failure evidence

**Type:** Source-confirmed integration gaps; no authorized live tag/promotion run was performed.  
**Trace:** Audit GH-02/GH-03/GH-06; `V06-GH-02.I1`, `V06-GH-02.I2`, `V06-GH-03.I1`, `V06-GH-06.I3`, `V06-G-03.I4`, `V06-G-09.I1`, `V06-G-09.I2`; follow-up R13/R14.

**Observed gaps:**

1. Packaging writes `artifacts/packaged-verification-0.4.1.json`, while the 0.6 promotion contract requires `packaged-verification-0.6.0.json`. There is no connecting rename/copy in the reviewed workflow.
2. Qualification performs a fresh checkout and downloads the verified package artifact, but does not download its dependency's lifecycle artifact. Its generator reads the committed attestations directory instead of the just-produced lifecycle records. Other mandatory attestations must also correspond to the candidate being assembled, not merely exist in that checkout.
3. The proposed tag is the producer commit `eb01bc9`, while later commits contain harness, verifier, and evidence corrections. The executable handoff must specify which revisions actually run; it cannot assume HEAD's tools exist at the older tag.
4. Packaging still lacks failure/always diagnostic retention for build, packaged-verification, catalog, or staging failures. Existing lifecycle uploads do not cover the package job.

Sources: [produced filename](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/.github/workflows/release.yml#L335), [qualification checkout and downloads](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/.github/workflows/release.yml#L514), [generator inputs](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/scripts/build_qualification_manifest.mjs#L143), [promised packaged asset](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/scripts/verify_release_promotion.mjs#L180).

**Required work:**

- Make the actual producer output names and downstream contract agree; retain the historical verifier script name if desired, but do not let it determine an incorrect release asset name.
- Connect the produced lifecycle evidence to qualification and provide correctly identified mandatory evidence for the retained candidate set. If a record is unavailable, fail or disclose the permitted gap according to the existing contract; never fill the hole with unrelated committed PASS records.
- Specify producer source, workflow revision, harness revisions, run identity, inventory, and artifact identities deliberately. Preserve separate verification and explicit promotion without rebuilding at promotion. A deliberate revision split is acceptable when recorded and validated; no particular implementation architecture is mandated here.
- Add package-job diagnostic collection that executes on failure, retains the original error, and reports missing evidence accurately.

**Close when:** A local or otherwise already-authorized nonpublishing integration check assembles the exact workflow-produced directory layout and runs the real qualification/promotion validators. A valid assembled set passes. Missing required lifecycle records, records from an unrelated run without an explicitly permitted and validated carry-forward entry, mismatched evidence filenames, and source/artifact contradictions fail. The two already disclosed carried lifecycle legs do not become mandatory native reruns merely to close this integration task; preserve their external-verification status. A controlled package-stage failure exercises diagnostic collection and preserves the failing outcome. Merely matching workflow text is insufficient proof of file plumbing. Report any unexecuted remote upload observation separately; do not require a public release to finish the independent integration work.

## F9-05 — Implement the actual v0.4.1 profile/settings preservation fixture

**Type:** Confirmed implementation/coverage gap; native end-to-end execution may remain environment-blocked.  
**Trace:** Audit GH-04; `V06-GH-04.I3`, `V06-GH-04.V3`, `V06-G-06.I2`; follow-up R07.

**Observed gap:** The reviewed sandbox scenario creates canary text and catalog cache/mirror data. Those checks do not exercise the separate v0.4.1 profiles/settings requirement, which remains checked in the tracker. The cache verifier also accepts a body containing only `schemaVersion: null`; the review executed that case and observed a preservation PASS.

Sources: [fixture scope](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/scripts/sandbox/run-lifecycle-in-sandbox.ps1#L301), [cache validation](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/scripts/sandbox/verify_preservation.py#L73), [explicit original requirement](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/docs/history/TODO-0.6.md#L1775).

**Required work:** Derive representative profile/settings fixtures from the actual released v0.4.1 persistence schema and storage location. Exercise the actual load/migration behavior and assert recovered values, not just file presence. Keep the genuine v0.5.0 SQLite/user-override checks. Tighten cache validation against its actual released contract rather than accepting any object containing a schema-version key.

**Close the independent implementation portion when:** Valid released-version fixtures load/migrate to the expected values. Loss or corruption of the required values deliberately seeded by the preservation fixture fails its preservation assertion, and malformed cache structures fail their verifier. A legitimate fresh profile or documented recovery/default behavior is not automatically a production error; the assertion must follow the documented migration contract for the seeded user data. The sandbox driver is wired to seed and inspect the real fixture. If a native sandbox run remains unavailable, leave `GH-04.V3` and the unexecuted native portion of `G-06.I2` open with that exact prerequisite. Do not claim that fixture tests alone prove an installer upgrade on Windows.

This task does not authorize modifying real user profiles. Use isolated test data. It does not require repeatedly rebooting an intermittently available sandbox until a run happens to pass.

## F9-06 — Reconcile the existing tracker with the scoped evidence

**Type:** Documentation/traceability correction, not a request for more hardware.  
**Trace:** Audit RT-06 and stabilization exit criteria; `V06-RT-06.V3`, `V06-G-08.V1`.

**Observed contradiction:** The freeze-9 RT-06 JSON states that the seven-installed-backend criterion is unsatisfied, and the release review identifies the remaining hardware scope. However, the authoritative task checkbox and its later disposition table still describe the old 12/12 result as closure.

Sources: [checked task](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/docs/history/TODO-0.6.md#L350), [current scoped review](https://github.com/sato942/localmotive/blob/c4526c1d453d7b23272786aa53b6eb0592a05bf2/docs/RELEASE-REVIEW-0.6.md#L290).

**Required work:** Preserve the 22/22 result for the four installed/selected/launched backends and three expected refusals. Mark the original seven-installed-backend acceptance portion incomplete/environment-blocked consistently in the task checkbox, current disposition table, and report. Identify historical closure statements as superseded rather than rewriting historical measurements. Apply the same accurate current-versus-historical labeling to the six handoff rows when recording their results.

**Close when:** A reader can identify the current coverage and remaining prerequisite without reconciling contradictory “closed” and “blocked” statements. Do not acquire hardware, reopen correct vendor refusals, or claim that MT-07's verified deterministic identity cases require new physical machines. Keep any separate physical-hardware program distinct.

## Execution order and verification limits

Work on F9-02 before the final F9-01 packaged proof. F9-03 should be complete before the final F9-04 integration proof. F9-05 can proceed independently. Reconcile F9-06 and all status rows after the evidence settles. Parallel work is acceptable when file ownership and dependencies are controlled; no required change should be lost during consolidation.

For each fix, use the existing repository testing standards: demonstrate failure for the relevant reason, implement the correction, and verify the actual behavior or relationship. Test counts alone do not close a row. Record exact commands and results actually obtained; do not invent test names or claim execution from source inspection.

After product changes, select a candidate and requalify affected behavior on its actual bytes. Reuse unaffected evidence only with the existing explicit source-delta/carry-forward rules. Do not rebuild solely to make a documentation commit equal the binary producer commit. Conversely, do not relabel an older binary as containing a product fix.

No extra soak counts are added by this handoff. A deterministic regression plus the appropriate affected packaged proof is preferable to repeatedly missing a timing window. Preserve failed, invalid, and retried attempts with their own identities.

## Authorization and external prerequisites

Continue previously authorized repository remediation, local verification, commits, and pushes within their existing boundaries. This document grants no new external permissions.

Leave explicit owner decisions outside this repair pass: tag creation/push, publication/promotion, rulesets, the three hosted PR-campaign runs, and any SignPath/signing-policy decision. Do not reopen an already approved unsigned-distribution policy merely because signing is available.

Keep unavailable hardware, independent benchmark data, live credentials, manual accessibility testing, and the two unavailable native sandbox reruns separately blocked. Their existence must not stop the independent implementation portions above; their absence must not be disguised as PASS. Do not request authorization for code repair that is already authorized.

## Required final handoff and stopping condition

Return this table with one row per fixed handoff ID:

| ID | Original audit/V06 trace | Disposition | Implementation commit | Regression/integration evidence | Packaged/native evidence or remaining prerequisite |
| --- | --- | --- | --- | --- | --- |
| F9-01 | MT-05/MT-06; R11 | | | | |
| F9-02 | MT-05/MT-06; related R04 ownership | | | | |
| F9-03 | GH-02; R08/R12 | | | | |
| F9-04 | GH-02/GH-03/GH-06; R13/R14 | | | | |
| F9-05 | GH-04/G-06; R07 | | | | |
| F9-06 | RT-06/G-08 | | | | |

Allowed dispositions are: **fixed and verified**; **disproved against current source with concrete evidence**; or **independent work complete, named external verification still blocked**. If implementation or an executable regression is unfinished, say so explicitly rather than using the third disposition.

Include final HEAD, binary producer SHA, artifact digests, workflow/harness revisions, relevant CI links, evidence paths, and the remaining external-action list. State the source of each claim: source inspection, executed local test, recorded packaged result, or executed native result.

**Stop this repair pass when all six rows have a supported disposition, the required changed-code checks pass, affected evidence is correctly bound, and the current documentation agrees.** Return the result for owner review. Do not reset the universal goal, repeat resolved findings, start a fresh audit, or continue optional testing to manufacture more work. Completion of this bounded repair pass does not authorize publication or erase external acceptance gaps.
