**Localmotive 0.6 — follow-up review of committed remediation**

Reviewed 12 September 2026. Repository revision: **db548c820cbdf2d8b289f797ce27d31f1ff338f3**. Scope: the new release review, the previous release/client findings, relevant callers, qualification helpers, retained evidence, and live GitHub CI/governance state. This is a targeted follow-up, not a fresh audit of every project component.

**Decision: release approval should remain blocked.** The commits and successful CI are independently verifiable. Several earlier defects remain unchanged, and the additional evidence does not establish the claimed cancellation and migration guarantees. New helper-level gaps can also allow incomplete lifecycle qualification to be reported as PASS.

The uploaded RELEASE-REVIEW-0.6(1).md is byte-identical to [the committed review](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/docs/RELEASE-REVIEW-0.6.md). Its SHA-256 is 6cc4972a67c4ca86b370dbfc5619120e62c4e52919d7d0ea0ab7bf2e2e37aa47.

**What was independently verified**

| Check | Observed result | Scope |
|---|---|---|
| Current main | db548c820cbdf2d8b289f797ce27d31f1ff338f3 | GitHub branch API and local checkout |
| Exact-commit CI | [Run 34674588345](https://github.com/sato942/localmotive/actions/runs/34674588345) succeeded | check, rust-audit, and package-smoke succeeded; pr-check was skipped for this push |
| Rust suite | 600 passed, 0 failed, 7 ignored | Read from [the CI check log](https://github.com/sato942/localmotive/actions/runs/34674588345/job/103502045191), not rerun locally |
| Frontend tests | 141 Vitest tests passed | Same CI log |
| Node tests | 162 passed in the combined npm-test run | The earlier 127-test release-gates pass is repeated within those 162; do not add it again |
| Formatting/lint/build checks | Successful CI steps | Includes Rust formatting, Clippy, documentation-test command, dependency checks and packaged build/startup smoke |
| Tracker-link validation | 1,397 links; 643 checked tasks; 20 open; TRACEABILITY OK | Executed locally; validates references/accounting, not behavioral correctness |
| Release dependency analysis | Four missing direct dependencies | Executed locally against committed YAML |
| Publication condition | Tag-based manual dispatch with publish=false evaluates true | Evaluated the actual simple Boolean predicate; no workflow dispatched |
| Preservation verifier | Both retained baseline samples pass | Executed locally; each SQLite database contains only canary_marker, one row |
| Governance | main.protected=false; repository rulesets=[] | Live GitHub readback |
| Release tag | No v0.6.0 tag present | Read-only git ls-remote --tags |

The exact-commit runs API returned one run, CI. Therefore this successful run is not execution evidence for release.yml. The older green run quoted in the review, 34674036480, belongs to **946ac1c912f2a9dcf0c43b2181eafa49a5220cc0**.

No Windows Sandbox session, local Rust build, release workflow, publication, PR, or repository-setting mutation was initiated during this review. Recorded Windows results are identified below as records, rather than independent reruns.

**Improvements that deserve credit**

The producer builds once, verifies/stages artifacts, and publication retrieves those artifacts without rebuilding. The publication inventory CLI now resolves artifact paths from the inventory's directory. It verifies producer evidence without rewriting it. Genuine lifecycle job failure blocks publication.

The new cancellation tests did run successfully in CI. They demonstrate eventual worker exit, response-body connection cleanup by the request deadline, and preservation of single-request behavior. They do not prove cancellation abort or immediate-restart safety.

Fresh v0.4.1 and v0.5.0 lifecycle records now exist, including separate candidate and harness source fields. Earlier wrong-version negative evidence remains available. The RT-04 delayed-download/tamper/restore campaign has a retained 17/17 console record.

Call-site inspection also narrows two earlier concerns: normal launch profiles reject host-delimiter injection and require a complete TLS certificate/key configuration. Health and cold-benchmark supervisors terminate owned children; ordinary tuning trials preserve cleanup failures. Those paths should not be described as universally abandoning processes.

**Finding register**

P1 means a release-blocking correctness or qualification defect. P2 means a material validation, evidence, or handoff defect. These are follow-up IDs, not additions silently attributed to the original audit.

| ID | Priority | Finding | Existing audit/TODO trace |
|---|---|---|---|
| R01 | P1 | Release outputs read through missing direct dependencies | GH-02; G-09 |
| R02 | P1 | Manual publish=false bypass on a tag reference | GH-02; G-09 |
| R03 | P1 | Cancellation can become successful benchmark/final-winner evidence | MT-04; MT-06; MT-13 |
| R04 | P1 | Warm-benchmark ownership released while cancelled request continues | MT-05; MT-06; related MT-04 |
| R05 | P1 | Missing/unverified preservation permits lifecycle success | GH-04; GH-06; G-06; G-09 |
| R06 | P1 | Lifecycle does not compare candidate bytes with producer inventory | GH-03; GH-04; G-06; G-09 |
| R07 | P1 | Canary preservation does not test the required application migrations | GH-04; G-06.I2 |
| R08 | P1 qualification | Current evidence set mixes candidate identities | GH-08; G-05.V1; G-06; G-09 |
| R09 | P2 / qualification | Installed payload identities differ without equivalent qualification | GH-04.I2; G-06.I3; G-09 |
| R10 | P2 | TLS pair validator accepts invalid PEM/mismatched pairs | MT-06.I4 |
| R11 | P2 | Packaged cancellation driver overstates what its assertions prove | MT-06.V3; related RT-03/MT-04 |
| R12 | P2 | Consumer inventory validation lacks schema/version/exact-set checks | GH-02; G-09 |
| R13 | P2 | Release evidence retention/readback/tag checks remain incomplete | GH-02; GH-06; GH-08; G-09 |
| R14 | P2 / owner gate | Governance remains unapplied and owner handoff is not ready | GH-01; GH-02; G-08; G-09 |
| R15 | P2 hardening | Transport policy and allocation/file-read bounds remain incomplete | MT-06 |

All references above use the V06- prefix in [the tracker](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/docs/history/TODO-0.6.md), which links to [the original audit](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/docs/history/localmotive-comprehensive-audit.md).

**R01 — Release dependency errors are still present**

[release.yml](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/.github/workflows/release.yml#L108) still contains all four defects:

| Consumer | Current declaration | Missing dependency |
|---|---|---|
| quality, line 109 | rust-audit | resolve |
| package, line 198 | rust-audit, quality | resolve |
| clean-account-lifecycle, line 402 | resolve, package | quality |
| publish, line 476 | rust-audit, quality, package, clean-account-lifecycle | resolve |

The first source-identity assertion fails at line 137: the checkout has a SHA, while needs.resolve.outputs.sha is unavailable. Fixing only quality leaves package, lifecycle artifact naming, and publication source binding broken. GitHub exposes direct dependencies in needs; transitive graph reachability does not make their outputs accessible. [GitHub context reference](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#needs-context).

The existing [gate checker](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/scripts/verify_workflow_gates.mjs#L14) checks transitive reachability and returns ok=true for this workflow. Some [tests](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/scripts/tests/release-gates.test.mjs#L2050) explicitly require the incomplete declarations. This explains why green gate tests coexist with the defect.

Required closure: add resolve directly to quality/package/publish and quality directly to lifecycle. Replace tests that preserve the wrong lists with checks that each referenced output belongs to an actual direct dependency and an existing producer output. Then verify the nonpublishing path.

**R02 — The publication input can still be ignored**

The [publication predicate](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/.github/workflows/release.yml#L473) accepts a version-tag reference independently of event type. A workflow_dispatch at refs/tags/v0.6.0, requesting v0.6.0, satisfies the first branch with publish=false. GitHub permits manual dispatch references to be tags. [Dispatch API](https://docs.github.com/en/rest/actions/workflows#create-a-workflow-dispatch-event).

This is currently masked by R01. It becomes reachable once dependency wiring is fixed.

Required closure: require github.event_name == 'push' in the automatic tag branch. Exercise push/manual events, branch/tag references, matching/mismatching tags, and both publication-input values. Every manual false case must skip publishing even when all other gates pass. Substring assertions on the predicate are insufficient.

**R03 — A cancelled last response can be recorded as success**

[local_client.rs line 389](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/src-tauri/src/local_client.rs#L387) still returns a received result without checking cancellation. A flag set after dispatch but before response acceptance can be missed when the response arrives before the next 500 ms polling timeout.

The application consequences are now visible:

- [measurement.rs lines 403–425](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/src-tauri/src/measurement.rs#L403) converts the successful response into Succeeded without a post-request flag check. A final trial has no next iteration to observe cancellation.
- [measurement_service.rs](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/src-tauri/src/measurement_service.rs#L411) can classify the resulting run as Measured.
- [tune.rs lines 1050–1074](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/src-tauri/src/tune.rs#L1050) checks cancellation before finalist measurement, but can confirm the winner after that measurement without checking cancellation again.

Required closure: define cancellation at result acceptance and protect both the shared request path and final evidence/winner acceptance. Use coordinated request-accepted → cancellation-set → response-released fixtures, including the last benchmark trial and finalist verification. This was established by source review, not a newly executed timing reproduction.

**R04 — Warm cancellation releases ownership before cleanup**

The new [worker counter](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/src-tauri/src/local_client.rs#L368) increments/decrements around the same detached worker. It supplies no abort signal, join, global cap, or admission control; the observer is test-only and each warm request constructs another client.

The warm [benchmark path](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/src-tauri/src/measurement_service.rs#L369) uses that request while retaining the running server. cancel_benchmark only sets a flag. benchmark_v2 clears its active slot at line 539 and returns, releasing the operation reservation, while the worker may still be doing inference.

The new [repeat-cycle test](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/src-tauri/src/local_client.rs#L1326) waits for active_workers==0 before the next cycle. It therefore proves serialized eventual drain, not safe immediate restart.

Required closure: keep the operation unavailable until cleanup completes, or implement cancellable request ownership with bounded cleanup. Immediately attempt replacement work as soon as the application allows it while the old request deliberately remains slow. Assert no overlapping conflicting inference or accumulated workers.

Preserve the positive behavior: healthy responses longer than 500 ms must still complete once. Health's supervisor and cold/trial-child cleanup already provide stronger ownership; reuse their guarantees where appropriate without assuming a shared live server can always be killed.

**R05 — Lifecycle can report PASS with missing verification**

[host-run-lifecycle.ps1 lines 272–305](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/scripts/sandbox/host-run-lifecycle.ps1#L272) assigns preservation=missing-files when collected files are absent, or UNVERIFIED when Python is unavailable. Only literal FAIL throws. The other states retain the sandbox's overall PASS, print PASS, and exit zero.

The release job depends on process success, so a required preservation check can be absent while its gate succeeds. The committed fresh runs happen to contain preservation PASS; this finding concerns untested failure behavior.

Required closure: validate the required scenario set, fields, artifacts, and exact PASS outcomes before success. Missing files, missing verifier, malformed results, omitted scenarios and non-PASS preservation must retain diagnostics and exit unsuccessfully.

**R06 — Lifecycle source labeling is not artifact verification**

The [new inventory binding](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/scripts/sandbox/host-run-lifecycle.ps1#L36) correctly rejects a source-SHA/environment mismatch. But it reads only sourceRevision. It does not compare actual installer sizes or hashes with the inventory.

At [lines 130–170](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/scripts/sandbox/host-run-lifecycle.ps1#L130), it copies expected filenames, computes new hashes, and later labels those hashes with the inventory's source. If either local installer is missing, it falls back to public release downloads even when CandidateDir was explicitly supplied. That can reintroduce the prepublication wait and mix candidate origins.

The publication job's later hash check cannot establish which bytes an earlier lifecycle job actually exercised.

Required closure: validate the complete producer inventory against copied candidate bytes before Sandbox starts. Explicit candidate mode must fail on missing/mismatched files, with no public-asset fallback. Bind lifecycle results to those verified bytes and require the publisher to verify that binding.

**R07 — Both baseline runs preserve generic canaries, not real application data**

Fresh v0.4.1 and v0.5.0 upgrade records exist. However, the [fixture setup](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/scripts/sandbox/run-lifecycle-in-sandbox.ps1#L181) plants a text canary and a static SQLite file after launching the older app. The [verifier](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/scripts/sandbox/verify_preservation.py#L17) checks only canary_marker and the text content.

I ran the verifier on both collected baseline samples and inspected their schemas: both contain exactly one table, canary_marker, and one row. They contain no application catalog schema or user overrides. No real profile is created and recovered through the application.

Thus the records support installer upgrades and generic file retention, not the specific v0.4.1 profile/settings and v0.5.0 SQLite/override migrations required by G-06.I2. GH-04.I3/V3 and G-06.I2 should be reopened to that extent.

Required closure: create real persisted data with each released application, or use fixtures independently proven to match its schema, then assert recovered values and override authority through 0.6 after upgrade. Run both baselines in candidate qualification; release.yml still explicitly requests only v0.4.0.

**R08 — The committed current-candidate evidence is inconsistent**

| Record | Candidate/source field | Artifact identity | Assessment |
|---|---|---|---|
| [Candidate inventory](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/release-evidence/0.6.0/candidate-inventory-0.6.0.json) | 3a2b06e7… | Portable fbbd2a1a…, setup b83afa2c…, MSI 1d217350… | Older candidate |
| [Packaged verification](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/release-evidence/0.6.0/attestations/packaged-verification-0.6.0.json) | c64e031c… | Same older portable fbbd2a1a… | Candidate and verifier-source identity conflated |
| [Fresh lifecycle](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/release-evidence/0.6.0/attestations/sandbox-clean-account-lifecycle.json) | 7c32fa90… | Setup e405cd1e…, MSI c18eecc2… | Newer candidate; harness source recorded separately |
| [Fresh v0.4.1 preservation](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/release-evidence/0.6.0/attestations/sandbox-preservation-v0.4.1.json) | 7c32fa90… | Same newer installers | Newer generic-canary record |
| [Fresh v0.5.0 preservation](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/release-evidence/0.6.0/attestations/sandbox-preservation-v0.5.0.json) | 7c32fa90… | Same newer installers | Newer generic-canary record |

The tracker identifies 7c32fa9 as the freeze; the review and committed inventory still identify the older set as current. Product code changed between 3a2b06e and 7c32fa9 in download.rs, local_client.rs, and runtime.rs. This is not merely a later documentation commit.

The [packaged verifier](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/scripts/verify_041.mjs#L48) also defaults source_revision to repository HEAD. Checking that same checkout against a requested revision does not establish the executable's build source.

Required closure: retain a canonical candidate manifest linking the actual producer source, all three artifacts, and matching qualification records. Record verifier/harness revision separately. Label older records superseded without rewriting their historical claims. A later documentation-only commit does not itself require rebuilding a valid frozen candidate.

**R09 — Installed executable identities still need qualification**

The fresh lifecycle records report different installed executable hashes:

| Installation family | Executable SHA-256 |
|---|---|
| NSIS fresh/update/preservation | 97f54fd75714a10a07a76282717fabd43d283ecf9f4e9f34309563b65bf8e118 |
| MSI fresh | 458db471ba0f8b12ed0dac61e7d82decdf581431c5cc39e68200bbed8c73548a |

The [sandbox helper](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/scripts/sandbox/run-lifecycle-in-sandbox.ps1#L213) logs the MSI difference and still reports PASS. It does not compare installed payloads to trusted expected digests for each distribution. This difference is disclosed; it is not proof of malicious substitution or necessarily a packaging defect. It does prevent silently extending portable-only behavior qualification to an unexplained different payload.

Version validation at lines 102–104 uses StartsWith(expected), which also accepts unintended suffixes such as 0.6.00 for 0.6.0.

Required closure: establish expected installed-payload identities, explain differences and qualify each distinct payload as needed. Normalize and compare versions under an exact documented rule.

**R10 — The TLS validator and its test still accept invalid PEM**

[local_client.rs](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/src-tauri/src/local_client.rs#L131) checks marker strings rather than parsing certificate/private-key data and matching the pair.

More concretely, [core.rs lines 3742–3759](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/src-tauri/src/core.rs#L3742) calls marker-wrapped MIIB text a real PEM pair and expects validation success. MIIB decodes to only three bytes; it is not a complete certificate or key. A later client certificate parse does not satisfy prelaunch pair validation.

Required closure: use valid fixtures and real parsing/matching. Malformed certificate/key data and unrelated pairs must fail before launch.

Correction to the earlier review: upstream core validation already rejects ordinary one-sided TLS configuration. The remaining demonstrated defect is content/pair validation, not universal acceptance of one-sided profiles.

**R11 — The packaged cancellation driver accepts weaker outcomes than claimed**

[scripts/g05_mt06_cycles.mjs](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/scripts/g05_mt06_cycles.mjs#L153) infers active inference from an enabled Cancel button. It does not require server processing to be positive before cancellation. It accepts “Benchmark finished” as settlement and later waits for processing to drain before beginning another cycle.

It does not assert an actual Cancelled terminal record, absence of successful persisted evidence, or a bounded measured cancellation latency. Its process helper counts every llama-server.exe on the machine and returns zero when tasklist fails, rather than proving ownership.

The 37/37 statement appears in the report/tracker, but no corresponding run log was found in the committed 0.6 evidence directory. That is an availability/verification gap, not a claim that the run was fabricated.

Required closure: coordinate cancellation with real request acceptance, check the terminal outcome and persisted records, measure cleanup timing, track owned PIDs, fail on process-enumeration errors, and retain a source/digest-bound result. Add an immediate-restart scenario without a test-imposed drain interval.

**R12 — Consumer inventory validation trusts the supplied entry list**

[verifyPublishedInventory](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/scripts/verify_candidate_inventory.mjs#L80) checks source equality and then loops over supplied artifacts. It does not validate supported schema, expected release, exact unique asset set, or canonical names.

A direct local call accepted a source-matching inventory with release=wrong-version and artifacts=[] without executable files. This is a function-level validation gap: the workflow's subsequent sha256sum command rejects the missing-file fixture, so it is not a demonstrated complete publication bypass.

Required closure: enforce the producer's schema/version/exact-set invariants at the consumer boundary too. Cover empty, duplicate, missing, extra and wrong-version entries, plus real size/digest mutations.

**R13 — Remaining release-evidence gaps**

| Gap | Source | Required closure |
|---|---|---|
| Failed package/readback diagnostics are success-only uploads | release.yml lines 383–389 and 577–583 | Add separate failure-tolerant diagnostic uploads; keep “verified candidate” success-only |
| Some lifecycle preflight failures bypass structured failure capture | host-run-lifecycle.ps1 validation before the main try | Retain an attributable fresh failure result without leaving a stale PASS as apparent current evidence |
| Downloaded public JSON is only checked nonempty | release.yml lines 565–575 | Compare producer JSON bytes or trusted digests as well as binaries/checksum text |
| No final remote peeled-tag/source recheck | Publication and readback steps | Reject tag drift immediately before publication and enforce immutable-tag governance |
| Promised SBOM/catalog/lifecycle evidence is not in the public six-file set | release.yml lines 541–547; review line 235 | Reconcile the promised public contract with upload/readback assertions |
| Lifecycle summary still says non-gating | release.yml line 453 | Correct the stale summary to match the actual dependency |

[Workflow source](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/.github/workflows/release.yml). The SBOM is generated and uploaded separately as an Actions artifact; the gap is release attachment, not absence of generation.

**R14 — Owner approval is not the only remaining work**

Live readback still shows no repository rulesets and unprotected main. That external gate remains open, as the review acknowledges. Source fixes and evidence repairs above can be completed before requesting owner configuration or publication.

The [owner package](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/docs/RELEASE-REVIEW-0.6.md#L153) also needs correction:

- It simultaneously says origin/main remains e530371 and records the completed bootstrap push.
- Its branch ruleset requires status checks but lacks the separate pull_request rule needed for the stated PR-only policy. [GitHub rules reference](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets#require-a-pull-request-before-merging).
- Tag creation omits an explicit reviewed SHA; the command uses whichever HEAD happens to be checked out.
- Workflow watching selects the latest run, rather than a run bound to the intended tag, commit and event.
- gh release view requests isLatest, which is not a supported JSON field. Use the supported release fields and a separate latest-release lookup. [CLI reference](https://cli.github.com/manual/gh_release_view).
- A green → controlled failure → corrected green PR campaign entails multiple check executions; describe and authorize the concrete campaign rather than promising one run.
- The claim that rust-audit runs on every PR conflicts with ci.yml: that job is push-only and pr-check does not independently run cargo-audit.

Required closure: prepare a coherent, executable owner handoff after the agent-completable blockers are fixed. Keep settings and publication gated by the existing owner decision; do not treat unavailable authorization as evidence of successful enforcement.

**R15 — Transport hardening still needs explicit scope**

Cargo resolves direct reqwest 0.12.28 with default features disabled and rustls-tls enabled. The [builder](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/src-tauri/src/local_client.rs#L224) does not explicitly express redirect, proxy or exclusive-root policy. Adding a trusted certificate is not by itself evidence of exclusive pinning. Confirm intended behavior with the pinned build instead of extrapolating from a different reqwest version.

The client's own host validator remains permissive, but [LaunchProfile validation](https://github.com/sato942/localmotive/blob/db548c820cbdf2d8b289f797ce27d31f1ff338f3/src-tauri/src/core.rs#L940) restricts normal production profiles to IP addresses or localhost. No reachable profile-based destination injection was demonstrated in this review.

Request serialization allocates before its 4 MiB check at local_client.rs lines 329–334. File reading checks metadata then separately calls unbounded fs::read at lines 516–525. Use bounded serialization and one opened handle with a limit-plus-one read. Normal caller constraints reduce reachability but do not make those helpers intrinsically bounded.

The response path already uses a limit-plus-one reader. API-key Debug output is redacted. Neither should be reported as absent.

**Recommended completion order and evidence standard**

1. Fix R01/R02 and their semantic tests before attempting any release run.
2. Fix R03/R04 and strengthen R11 so cancellation correctness and immediate restart are actually exercised.
3. Fix R05/R06/R07/R09/R12; qualify real application migrations and exact candidate/installed identities.
4. Reconcile R08 and R13 against one chosen candidate. Preserve historical records under explicit superseded identities.
5. Complete R10/R15 and regenerate only affected qualification evidence when shipped behavior changes.
6. Correct R14, then present the concrete remaining owner actions.

Keep each task linked to its existing V06 audit/TODO references. The successful 643/20 link count is an accounting result; it does not override the reopened behavioral findings. Record command, exit status, source/candidate identity, environment and evidence location before marking a requirement verified. Missing execution stays unverified.

**Reproduction and review limits**

Local checks used the committed tree, Python/SQLite, Node, and Git. The workflow repro fed PyYAML-parsed objects into the unchanged original JavaScript validator function body because this checkout did not have npm's yaml dependency installed. The publication test evaluated the actual simple predicate with equivalent Boolean/string operators. These are targeted semantic checks, not a simulated successful Windows/GitHub release run.

The preservation checks used the committed fresh collected SQLite/text artifacts in read-only inspection and the repository's actual Python verifier. No real old/new application upgrade was rerun locally.

The reported Rust/frontend/package successes come from the inspected CI logs at the pinned commit. They support what those tests and smoke steps assert, not the unexecuted release pipeline or every release-qualification claim.

Full candidate hashes for reconciliation:

| Artifact set | Artifact | SHA-256 |
|---|---|---|
| Inventory at 3a2b06e7… | Portable | fbbd2a1ab8c6c2698a85ef8535146b316c5944da2362192a7aafa2399b217b4b |
| Inventory at 3a2b06e7… | NSIS installer | b83afa2c509405b2cb5074f275a5b7ad7fe6cb710641c23a8f2833bee6b6dee0 |
| Inventory at 3a2b06e7… | MSI installer | 1d217350ba0cd47a022e4741fa7b1ac295cbb52521c63d23d8bc6d5da82c9f65 |
| Lifecycle at 7c32fa90… | NSIS installer | e405cd1e7b8cc3f2dca82dbd768d19f0c0dd55ff62891864b7e9a2c7fbea2887 |
| Lifecycle at 7c32fa90… | MSI installer | c18eecc23ed121b4cfbe8f3b125b0b4e13ee46a99fcee722adc252c4b9641899 |

