# Localmotive 0.6 — remaining work after 076a3eeecbdf

Baseline commit: `076a3eeecbdf8fa149a4c5f675526ad4ae24b58e`  
Repository: https://github.com/sato942/localmotive  
Prepared: 2026-09-13  
Status: **OPEN — reviewed correction complete; release acceptance and publication incomplete.**  
Companion goal: `GOAL-0.6-post-076a3eeecbdf.md`, supplied separately for this residual tracker (not the earlier closed correction goal). That file is not committed in this repository; the tracker does not depend on its presence.

## Purpose and authority

This is the active residual-work tracker for the state after the baseline commit. It preserves the original audit criteria and accepted corrections. It does not restart the 72-finding audit, replace historical evidence, or authorize a release. The user requested this new TODO and one related goal; creating these documents is not approval for repository settings, tag creation, publication, paid hosted campaigns, physical fault testing, or live credentials.

The source tracker is [the original TODO at the baseline](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md); the attached `TODO-0.6(1).md` is byte-identical. Its SHA-256 is `7fc68e59e27d32fdd447f9cdfd9402b2cf3691ef32e9a90f9b1a5ba030a25d3d`. Its evidence source is [the original audit](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md). Each action below links to its original criterion or an existing acceptance/recordkeeping obligation. `P06-*` and `R06-*` are local work labels, not new audit findings.

The original file contains **645 checked lines and 19 unchecked lines**. `V06-RT-06.V3` appears twice because the exercised four-backend portion and unexercised seven-installed-backend portion are separate. The older “18 open” count is superseded. A checked line alone is not verification evidence.

This file has **25 action checkboxes**: four preparation/recordkeeping actions, all 19 original open criteria, and two acceptance packages hidden by historical checked rows. Several actions can share one valid evidence record. The new count is not 25 newly discovered defects, and it is not a release-readiness percentage.

Read `AGENTS.md`, this file and the relevant source criterion before work. Use the latest explicit owner instruction for authorization. Earlier main-push approvals covered their stated work; do not assume they authorize arbitrary new commits, tags or settings changes. If new owner authorization is already explicit and applicable, record it and proceed without asking again.

## Accepted baseline — preserve it

| Item | Accepted state and limit |
|---|---|
| Reviewed source | `076a3eeecbdf8fa149a4c5f675526ad4ae24b58e`. Do not silently replace it with a moving branch tip. |
| Exact-revision CI | [Run 34774614565](https://github.com/sato942/localmotive/actions/runs/34774614565) completed successfully: check, rust-audit, Security audit and package-smoke. PR-only check was correctly skipped on a push. |
| F9-01 | Classification correction accepted; independent classifier/release-gate tests 150/150; old-library regression 7 pass / 2 expected failures; committed UI/API record replay retains PASS. Zero/unavailable samples are NOT-EXERCISED, observed overlap fails. |
| F9-02 / F9-03 / F9-06 | Closed per owner review. Keep client-publication ordering, actual producer-schema validation and truthful backend scope intact. |
| F9-04 | Staged packaged-verification transfer and promotion integration accepted. A fresh staged producer record is bound before manifest validation; missing/mismatched records fail. |
| F9-05 | Independent implementation accepted. Corrected-fixture native verification remains unexecuted/blocked and is tracked in R06-01. |
| Historical packaged campaign | Freeze-10 `mt06-cycles-result.json`: 75/75, source `e9a36b3aa075d72713d88af4cc74d64d561f12f8`. Classifier replay is separate from a new packaged run. |
| Historical manifest | 18 records validate against source `e9a36b3...`; generation revision `362843c`, manifest rebind `9e8582f`, recorded release-workflow digest `091a0d160d2295cdd5d9a5661cc3821fe95a44eb93bd83a2590986d5509850c0`. |
| Lifecycle carry-forward | Current manifest carries four historical lifecycle records from two source groups. Carry-forward provenance is not a claim that corrected native scenarios ran on current candidate bytes. Reuse legitimate evidence only within its actual scope. |
| Release state at baseline | No published v0.6.0 release. Release revision selection, real-path qualification, authorization and publication remain open. |

Sources: [current F9 dispositions](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/RELEASE-REVIEW-0.6.md#L308); [qualification manifest](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/release-evidence/0.6.0/qualification-manifest-0.6.0.json); [corrected classifier](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/scripts/lib/mt06_verdicts.mjs#L39); [driver wiring](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/scripts/g05_mt06_cycles.mjs#L586).

Historical freeze-10 artifact identities, retained solely to identify those existing bytes:

| Artifact | SHA-256 | Bytes |
|---|---|---:|
| Portable | `d6a6ef663c925f9605f06ea2957ef95409dace00d798ebc28e5666249fd59427` | 20818432 |
| Setup | `b935033d2ca2dbaf595c0a20e629053f0b4097ebe4b1059c351d140530ab8652` | 5290112 |
| MSI | `bc5c6aae332487430c1d7a238b668e3919f1514f713dd0c9d1ab80e248313b9b` | 9035776 |

A future verification run must record its own source and measured digests. Unchanged product inputs do not establish reproducible bytes. Current CI packaging is not automatically the release-qualified bundle. Never relabel an older manifest or assign these historical digests to new outputs.

## Status, evidence and authorization rules

Use these explicit states beside each action: `READY`, `IN-PROGRESS`, `BLOCKED-OWNER`, `BLOCKED-ENVIRONMENT`, `BLOCKED-DEPENDENCY`, `VERIFIED`, or `DEFERRED-OWNER`. A checkbox is checked only for `VERIFIED`. An approved deferral remains unchecked and retains the original unmet criterion. Unknown, skipped, fixture-only, carried-forward and NOT-EXERCISED observations are not PASS for a wider claim.

At baseline the 19 original open rows comprise 15 owner/release-dependent criteria, three hardware/performance criteria and one mixed human/credential criterion. Availability and authorization are separate: a missing environment is not automatically resolved by permission, and an available account is not permission to use it.

| Action | Authority for this goal |
|---|---|
| Read repository/CI/settings, compare evidence, inspect available capabilities without running a new campaign | Available preparation work; use existing access and do not expose credentials. |
| Maintain this tracker and current status/handoff documentation; make isolated local documentation commits | Authorized local preparation when the owner starts this goal. Keep new commits local unless applicable push authorization exists. Preserve user changes. |
| Reuse existing fixtures and inspect prepared commands/payloads | Allowed within preparation; do not rebuild already accepted evidence without a concrete reason. |
| Apply branch/tag rulesets; open the hosted PR campaign; consume its three Windows-hosted runs | Requires explicit owner approval covering the particular action and budget. Prepared payloads are not applied settings. |
| Create/push a version tag; dispatch verification using that identity | Requires explicit owner approval of the exact revision and tag/run. This is separate from permission to publish. |
| Promotion dry run, actual publication, changing a published release | Requires the corresponding explicit approval. Keep promotion separate from verification. |
| Manual accessibility work, live cloud/HF account scenarios, additional hardware or destructive fault campaigns | Require the actual operator/environment and applicable approval; use isolated test data. Never test power loss against real user data. |

When a required tool refuses an operation, do not bypass its access controls. Record the actual blocker. Do all useful independent preparation before requesting a decision. Do not turn one blocked action into a reason to stop other READY actions.

## Preparation actions — start here

### P06-01 — Establish and reconcile the current state

- [x] **P06-01 — Reconcile the residual tracker and current summaries.**

Completed 2026-09-13 (commit `09c3f59`; exact-revision CI [run 34779617259](https://github.com/sato942/localmotive/actions/runs/34779617259), success: check, rust-audit, Security audit, package-smoke; pr-check correctly skipped on push).

Starting status: `READY`. Trace: [V06-G-02.I2/I3](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L2785); [V06-G-08.V1](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L3198); [V06-G-10.I2/V1](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L3237).

Record local HEAD, origin/main, working-tree state and the exact baseline CI URL/conclusions. If HEAD has advanced, inspect the actual difference before carrying evidence forward. Do not reset, rebase or overwrite user work. Documentation-only successors may retain relevant source evidence; product/build-input changes require a stated affected-evidence analysis, not an automatic full audit.

Make the current summary explicitly say that the F9 correction is accepted but release acceptance is incomplete. Replace current uses of the stale 18-open count and overbroad “all 72 verified” statement; retain historical text with clear dates/superseded labels. Preserve the original audit and frozen v0.4.1/v0.5 closeout evidence. Make the checked GH-04/G-06 native limitations and FE-05 sign-off hold visible through the R06 rows below. Do not mark old incomplete criteria passed to make totals look consistent.

Acceptance: the current index points to this TODO; all 19 original unchecked criteria and both R06 packages are present; accepted F9 closures are unchanged; counts are generated from actual task lines; relevant original V06 links resolve. Existing original-tracker validators may not count this new file, so inspect its crosswalk explicitly rather than treating an old script's success as coverage of a new file.

Completion record: HEAD and origin/main are `09c3f59` (documentation-only residual tracker on reviewed baseline `076a3ee`); tree clean; baseline CI [run 34774614565](https://github.com/sato942/localmotive/actions/runs/34774614565) success for `076a3ee`; source tracker verified 645 checked / 19 unchecked lines; this file verified 25 boxes (completion of P06-01..P06-04 moves it to 4 checked / 21 unchecked); `check_tracker.mjs` validates only the source file (72 findings, 662 boxes) and is not coverage of this file. The current-status pointer lives in `docs/RELEASE-REVIEW-0.6.md` (top notice linking this root tracker); no AGENTS.md change was made for it. Source tracker left byte-identical; audit and frozen v0.4.1/v0.5 closeout evidence preserved; GH-04/G-06 native limits and the FE-05 sign-off hold stay visible through R06-01/R06-02 below.

### P06-02 — Prepare one concrete release and authorization proposal

- [x] **P06-02 — Prepare the owner decision packet with corrected revision and artifact identities.**

Completed 2026-09-13: the operative proposal below is the packet (preparation only; no approval consumed, nothing applied, tagged, or published).

Starting status: `READY`. Trace: [V06-G-03.I4/V1/V2](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L2809); [V06-G-08.V1](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L3198); [V06-G-09.I1/I2](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L3214); [existing executable handoff](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/RELEASE-REVIEW-0.6.md#L237).

Supersede the executable `REVIEWED_SHA=16a8317...` recipe: it predates the accepted correction. Propose reviewed code revision `076a3eeecbdf8fa149a4c5f675526ad4ae24b58e` unless the owner selects and reviews a later source. Keep the document revision, proposed tag revision, binary producer revision and verification run ID distinct. A later documentation-only commit need not cause another candidate build merely to place its own SHA inside itself.

Prepare, without applying, the exact main/tag ruleset payloads, readback commands, bypass policy, and three-run hosted PR campaign. Reuse and review the existing payloads instead of creating another policy system. State the separate decisions needed for settings, verification tag/run, promotion dry run, publication and external testing. Specify the candidate qualification/lifecycle matrix and evidence-consumption path.

Resolve sequence explicitly: the current `release.yml` builds and verifies on a tag and never publishes; `release-promote.yml` publishes from a selected successful verification run. Permission to create that verification identity does not complete G-08's final ship decision. The original G-03/G-09 wording describes pre-tag candidate qualification and tag creation after a ship decision. Record this ordering difference for owner approval in the packet; do not silently redefine either criterion or add a new workflow. With an approved two-stage sequence, authorization to verify comes first, final acceptance follows the evidence, and promotion follows explicit publication approval. Until that sequence is approved, the dependent real-path actions remain blocked. Do not require publication as a prerequisite for the prepublication ship decision. The proposal must explicitly distinguish required prepublication High-finding evidence from observations possible only after publication, notably the publication-metadata portion of V06-GH-02.V3. Obtain the owner's approved interpretation of that ordering; leave the publication-only portion open and unverified until actual promotion/readback. This is not an implicit waiver or a premature PASS. If the owner requires the literal ordering to remain unchanged, record the conflict as a blocker rather than inventing a workflow or cycling between impossible prerequisites.

Acceptance: one reviewable proposal names exact commands/payloads, target revision, what each approval permits, validation/readback steps, dependencies, and all remaining evidence gaps. The final ship decision and completed publication are never claimed by this preparation action.

## Operative proposal — pending owner approval (P06-02 packet, 2026-09-13)

This section is the decision packet. It authorizes nothing; every numbered approval below is a separate explicit owner decision. No step is executed merely because it is written here.

Proposed verification revision: `076a3eeecbdf8fa149a4c5f675526ad4ae24b58e` (reviewed correction; documentation commit `09c3f59` needs no new candidate build to place its own SHA inside itself). Historical binary producer `e9a36b3aa075d72713d88af4cc74d64d561f12f8` (portable `d6a6ef66…`, setup `b935033d…`, MSI `bc5c6aae…`) identifies existing bytes only; a future verification run records its own measured digests. The `REVIEWED_SHA=16a8317…` recipe in `docs/RELEASE-REVIEW-0.6.md` §2 predates the correction and is superseded for any release including it. Keep the document revision, proposed tag revision, binary producer revision, and verification run ID distinct.

Approval A — repository settings (GH-01.I3, GH-02.I3). Payloads: the exact `main-pr-check` branch ruleset (`pull_request` rule + required `pr-check` status + non-fast-forward + deletion) and the exact `immutable-release-tags` tag ruleset (`refs/tags/v*`, update + deletion) in `docs/RELEASE-REVIEW-0.6.md` §1. Observed 2026-09-13: `gh api repos/sato942/localmotive/rulesets` returns `[]`; `branches/main.protected` is `false`; remote tags end at `v0.5.0`. Apply only the approved payloads, then read back effective rules, required status names, PR/direct-push restrictions, and bypass actors (`gh api repos/sato942/localmotive/rulesets --jq '.[] | {id, name, target, enforcement}'`). A submitted payload alone is not effective-setting evidence. Bypass policy: no bypass actors for ordinary contributions; owner documents the solo-maintainer emergency path before requiring the status.

Approval B — three-run hosted PR campaign (GH-01.V1/V2/V3). (1) Benign PR on an isolated `windows-latest` runner proving `pr-check` runs green under its stable name (record PR head SHA, event, job name, runner labels, permissions). (2) Deliberately failing commit on the same PR proving the red state and blocked merge (do not merge the failing revision; record green/red SHAs and run URLs). (3) Revert proving green again. Push CI and self-hosted success do not substitute for these PR observations. Ordinary-contributor proof needs the authorized contributor role; an owner bypass is not that proof. No hosted run starts without this approval and its budget.

Approval C — verification identity (GH-03.V1, G-09.I1). Owner authorizes the exact revision and tag (proposed: revision `076a3ee…`, tag `v0.6.0`; never retarget a used tag). Tag push runs `Release verify` (`.github/workflows/release.yml`: resolve → rust-audit → quality → package → clean-account-lifecycle → qualification), which builds, verifies, runs the four-leg lifecycle matrix, assembles and validates the qualified bundle, and stops — it never publishes. Inputs: `git tag -a v0.6.0 <sha> && git push origin v0.6.0`. Qualification consumes that run's lifecycle records and stages that run's fresh packaged record (`--stage-packaged-verification`); evidence-consumption path and the candidate qualification/lifecycle matrix are the workflow's qualification job. Watch the run to success; it publishes nothing.

Approval D — prepublication reconciliation and ship decision (G-08.V1, R06-02, GH-02.V3 prepublication portion). After the verification run, reconcile required prepublication High-finding evidence (GH-01/GH-02/GH-03/GH-06.V3 rows), FE-05 sign-off against its four stated criteria, required settings/candidate verification, and every environment limitation in P06-03. A conditional permission to run verification does not close this criterion. Publication/readback records (G-09/G-10) are not a prerequisite for this decision; do not make them circular.

Approval E — promotion dry run, then publication and readback (G-09.I1/I2/I3/V1, G-10.I1). Dispatch `release-promote.yml` with `tag=v0.6.0`, `verify_run_id=<successful Approval-C run>`, `confirm=PUBLISH v0.6.0` (exact phrase, required even for a dry run), `dry_run=true` first; repeat with `dry_run=false` only on owner publication approval. The gate refuses non-owner dispatchers, wrong confirm phrases, unsuccessful or mismatched verify runs, off-main or check-red commits, bundle-identity contradictions, and tag drift (re-resolved peel immediately before publication). Then read back tag/release metadata, the complete asset set, downloaded byte identity, and Latest/prerelease/unsigned wording. Negative controls (wrong SHA, moved-tag resolution, modified bytes, missing assets, absent lifecycle evidence) fail through the real validation path without moving the protected tag or uploading tampered assets.

Sequencing clarification for owner approval: the original G-03/G-09 wording describes pre-tag candidate qualification with tag creation after a ship decision, while the current two-file path creates the verification identity first (tag push → verify run), reconciles prepublication evidence, records the publication decision, then promotes. Requested interpretation: verification authorization precedes final acceptance; final acceptance follows the verification evidence; promotion follows explicit publication approval. Publication-only observations — notably the publication-metadata portion of V06-GH-02.V3 — stay open and unverified until actual promotion/readback; this is not a waiver or a premature PASS. If the owner requires the literal pre-tag ordering unchanged, record that conflict as a blocker rather than redefining either criterion or adding a workflow.

Remaining evidence gaps at packet date: GH-01/GH-02/GH-03/GH-06.V3 owner-gated rows; G-09/G-10 release rows; RT-06.V3 seven-installed portion; DC-12.V3; S-25.I3; G-05.I3 manual/account scenarios; R06-01 native four-leg matrix; R06-02 FE-05 sign-off (see P06-03 matrix and the ledger).

### P06-03 — Prepare the remaining environment evidence without inventing it

- [x] **P06-03 — Consolidate the pending test matrix and unblock inputs.**

Completed 2026-09-13: the environment matrix below is the consolidated record (read-only capability inspection; no campaign executed, no run output invented).

Starting status: `READY` for planning/read-only capability inspection. Trace: [V06-RT-06.V3](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L351); [V06-DC-12.V3](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L743); [V06-S-25.I3](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L2636); [V06-G-05.I3](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L2854); [V06-GH-04.V3](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L1783); [V06-G-06.I2](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L3149).

Reuse the existing backend, a11y, preservation and lifecycle scripts. Record the host/OS/device, disk and fault-harness, manual operator, authorized test account and dataset inputs each pending scenario needs. Inspect actual executable arguments before presenting commands; do not invent ports, candidate paths, adapter IDs, credentials, dataset access or successful run outputs.

Give every lifecycle leg its own baseline version, candidate source/digests, harness revision, fixture scope and execution status. The manifest has four carried-forward records; do not summarize this as only “two reruns” without checking which current assertions each record really supports. One authorized native campaign may satisfy both GH-03 workflow and GH-04/G-06 preservation criteria if its assertions and identities cover them.

Acceptance: one matrix names every unavailable input and the precise scenario/evidence required when it becomes available. Preparing that matrix does not close the underlying verification boxes.

## Environment matrix — consolidated 2026-09-13 (P06-03 record)

Observed host capabilities (inspection only, not execution): `C:\Windows\System32\WindowsSandbox.exe` is present on this host, but Sandbox boot here is historically intermittent, so presence is not a boot guarantee. Single GPU vendor: one NVIDIA RTX 5090. Disks observed: one AMD-RAID SSD array plus one USB flash drive; no HDD / SATA-SSD / NVMe split is confirmed. All five harnesses exist with the interfaces stated per row; no campaign was executed and no run output is recorded here.

| Pending evidence | Harness and exact inputs | Scenario and required record | Unavailable input / prerequisite |
|---|---|---|---|
| RT-06.V3 seven-installed-backend portion | `scripts/g05_rt06_all_backends.mjs <cdpPort> <adapterId> [installBatch]`, candidate bound from the committed inventory | Exercise every backend the seven-installed criterion demands on identified hardware and candidate bytes, distinguishing installed / selected / launched / refused / measured per backend | A representative Windows host with the required backend combination (the recorded requirement concerns seven backends and a three-vendor host, not seven GPUs); campaign authorization. The measured four-backend scope plus three vendor refusals is preserved separately and does not satisfy installation. No duration is promised. |
| DC-12.V3 | 1/4/8-connection matrix against identified targets; approved OS-crash/power-loss facility | Disk identity, workload, connection count, throughput, actual fault mechanism, post-recovery assertions per target class | Identified HDD, SATA SSD, and NVMe targets; isolated disposable data; approved fault facility. A terminated application process is not an OS crash or power loss. No destructive experiment is run to clear this box. |
| S-25.I3 | Independent benchmark distributions, baseline drift, held-out calibration error, interval coverage, release queue/failure/evidence-retention observations | Observed data with stated sample scope and provenance for each claimed metric | Independent benchmark/held-out data, identified host/runtime/model conditions, measurement-session approval. Completed MT-07 identity tests stay closed; physical-hardware portability remains a limitation. |
| G-05.I3 manual/account scenarios | Packaged app + `scripts/verify_a11y.mjs` (automated portion already A11Y_PASS) | Actual screen-reader navigation/announcements (Narrator or NVDA); specified cloud/HF credential behaviors on identified candidate bytes, values kept out of files/logs | Human operator for the listening/navigation pass; explicitly authorized cloud/HF test accounts and suitable environment. Automated tree checks do not replace listening; missing accounts stay untested, not passed. |
| R06-01 native four-leg matrix (GH-04.V3, G-06.I2) | `scripts/sandbox/host-run-lifecycle.ps1 -Tag <tag> -Version <version> -PreviousTag <baseline> -CandidateDir <artifacts> -EvidenceName <leg> -PreservationFlavor <mirror\|cache> [-FaultSimulation none]` with `run-lifecycle-in-sandbox.ps1`; fixtures from `build_preservation_fixture.py`, verdicts from `verify_preservation.py` | Four legs on the selected candidate in a suitable isolated Windows environment: upgrades from v0.4.0 and v0.5.0, preservation from v0.4.1 (real profile/settings/cache assertions, `schemaVersion` 1, `models` array) and v0.5.0 (SQLite/user-override assertions); clean-install/start, strict uninstall, version/digest checks, seeded-data preservation; deliberate loss/corruption of seeded required data fails | Owner authorization for the campaign; Sandbox boot on the execution host; published baselines (current installers arrive via the verify run's `-CandidateDir`; previous v0.4.0/v0.5.0/v0.4.1/v0.5.0 assets download from their published releases inside the harness). One authorized campaign may also cover the GH-03 workflow and GH-06 preservation assertions it actually exercises. |

### P06-04 — Maintain evidence and produce a finite handoff

- [x] **P06-04 — Produce the completed available-work handoff and next-decision list.**

Completed 2026-09-13: preparation handoff recorded below and in the ledger; remaining actions each carry a concrete prerequisite. This completes preparation only — not G-10.I1 or the TODO.

Starting status: `READY` once available preparation is complete; remains maintainable after later approvals. Trace: [V06-G-02.I2/I3](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L2785); [V06-G-08.V1](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L3198); [V06-G-10.I1/I2/V1](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L3236).

For each action record status, original task/audit link, relevant commit, actual command/scenario, source and artifact identity, result/count/exit status, evidence path, limitations, and exact blocker or approval reference. Distinguish observation from inspection, deterministic fixture, record replay, historical carry-forward, actual native run and actual release run. Resolve only processes created by this work; preserve their output and never kill unrelated user processes.

Acceptance: no READY task is abandoned, no running owned process is silently forgotten, every remaining open action has a concrete prerequisite and next step, and the final message states either full completion or BLOCKED with exact remaining IDs. A blocked handoff may complete P06-04; it does not complete G-10.I1 or the entire TODO.

## Preparation handoff — available authorized work complete (2026-09-13)

Baseline and HEAD: reviewed correction `076a3eeecbdf8fa149a4c5f675526ad4ae24b58e`; documentation commit `09c3f59` (this tracker). Exact-revision CI: [run 34774614565](https://github.com/sato942/localmotive/actions/runs/34774614565) success for `076a3ee`; [run 34779617259](https://github.com/sato942/localmotive/actions/runs/34779617259) success for `09c3f59` (each: check, rust-audit, Security audit, package-smoke success; pr-check correctly skipped on push). Prior CI belongs to its recorded SHA; neither run authorizes a release.

Task states: P06-01, P06-02, P06-03, P06-04 VERIFIED (preparation only). Remaining open: all 19 original criteria (RT-06.V3, DC-12.V3, GH-01.I3/V1/V2/V3, GH-02.I3/V3, GH-03.V1/V3, GH-06.V3, S-25.I3, G-05.I3, G-08.V1, G-09.I1/I2/I3/V1, G-10.I1) plus R06-01 and R06-02 — 21 boxes, each BLOCKED-OWNER, BLOCKED-ENVIRONMENT, or BLOCKED-DEPENDENCY with the prerequisite stated in its row. No owned background processes remain.

Proposed release sequence (pending owner approval; executes nothing): (1) owner authorizes the exact verification revision/tag and the verification run; (2) the verification run produces and qualifies its actual artifacts; (3) required prepublication evidence is reconciled and the owner makes the final publication decision; (4) separately authorized promotion, publication, and readback follow. The original pre-tag/ship-order wording needs the owner's explicit sequencing clarification (see the packet's sequencing paragraph); the final publication decision is not stated as preceding the verification that supports it. Publication-only portions, including GH-02.V3's publication metadata, stay open until actually observed.

Next decision inputs: approvals A–E in the operative proposal above; seven-backend host, storage-class targets, measurement program, human operator, test accounts, and Sandbox-booted native campaign (see the environment matrix). No settings changes, hosted PR campaign, tag, verification dispatch, promotion, publication, hardware campaign, or live-account use is authorized by this reconciliation.

## Original unchecked criteria — preserve every requirement

The following 19 checkbox texts are carried forward verbatim apart from making audit links absolute. The additional execution notes narrow ambiguity; they do not replace the original acceptance wording. Group ordering below follows the source document, not permission to execute out of dependency order.

### V06-RT-06.V3

- [ ] **V06-RT-06.V3 (seven-installed-backend acceptance portion) — environment-blocked:** the acceptance wording asks for all seven backends installed on a representative host. This host runs one GPU vendor (NVIDIA, adapter `luid:0000000000014f4f`), so the maximum eligible set is four installed/selected/launched backends plus the three vendor refusals; a seven-simultaneous-backend host needs three GPU vendors. Unblock action: run `scripts/g05_rt06_all_backends.mjs` (candidate bound from the committed inventory) on a three-GPU-vendor Windows host. This box must not be closed from fixture evidence or relabelled from the historical 12/12. **Trace:** [Audit RT-06](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#rt-06).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L351).  
Starting status: `BLOCKED-ENVIRONMENT`.  
Unblock prerequisite: Suitable representative Windows hardware and authorization for the additional campaign.

Execution: Preserve the measured four installed/selected/launched backends and three vendor refusals. Use the existing g05_rt06_all_backends.mjs driver and actual candidate inventory; distinguish installed, selected, launched, refused and measured per backend.

Required closure evidence: Every backend demanded by the seven-installed criterion must actually be exercised with identified hardware and candidate bytes. Refusals remain useful observations but cannot satisfy installation. If the requested host combination is unavailable or infeasible, record that fact and obtain an explicit scope/deferral decision; do not repeat the single-vendor run or redefine it as seven installed.

### V06-DC-12.V3

- [ ] **V06-DC-12.V3** — Distinguish ordinary process-crash testing from target-environment OS-crash/power-loss validation; record throughput for 1/4/8 connections on available HDD, SATA SSD, and NVMe targets before optimizing. **Trace:** [Audit DC-12](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#dc-12).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L743).  
Starting status: `BLOCKED-ENVIRONMENT`.  
Unblock prerequisite: Identified HDD, SATA SSD and NVMe targets; isolated disposable data; an approved OS-crash/power-loss facility.

Execution: Prepare the 1/4/8-connection matrix. Preserve ordinary process-crash results separately. Gather the specified measurements before proposing tuning.

Required closure evidence: Record disk identity, workload, connection count, throughput, actual fault mechanism and post-recovery assertions. A terminated application process is not an OS crash or power loss. Missing target classes stay untested; perform no destructive experiment merely to eliminate this box.

### V06-GH-01.I3

- [ ] **V06-GH-01.I3** — Configure a main-branch ruleset requiring the newly available PR checks, restricting direct and force pushes, and documenting a workable solo-maintainer review and emergency-bypass policy; introduce the trigger before requiring its status. **Trace:** [Audit GH-01](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-01).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L1698).  
Starting status: `BLOCKED-OWNER`.  
Unblock prerequisite: Approval to apply the prepared main-branch ruleset and its solo-maintainer/bypass policy.

Execution: Inspect existing settings first, then apply only the approved payload. Confirm pr-check exists before requiring it. Preserve isolated PR execution and trusted push runner separation.

Required closure evidence: Read back active rules, required status names, PR/direct-push restrictions and bypass actors. A JSON payload or successful request alone is not effective-setting evidence.

### V06-GH-01.V1

- [ ] **V06-GH-01.V1** — Use a benign PR to confirm the expected checks run before merge, report stable names and are executed only on the intended isolated runner. **Trace:** [Audit GH-01](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-01).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L1703).  
Starting status: `BLOCKED-OWNER`.  
Unblock prerequisite: Approval for the benign PR and the three-run hosted campaign; suitable account access.

Execution: Use one test PR on the intended isolated Windows-hosted runner. Bind result to that PR head SHA and retain event, job name, runner labels and permission evidence.

Required closure evidence: The intended check actually runs and reports its stable name on the benign PR. Push CI and self-hosted success do not substitute for this PR observation.

### V06-GH-01.V2

- [ ] **V06-GH-01.V2** — Introduce a controlled failing check in the test PR; verify merge is blocked until corrected, then confirm the corrected commit obtains the required successful statuses. **Trace:** [Audit GH-01](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-01).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L1704).  
Starting status: `BLOCKED-OWNER`.  
Unblock prerequisite: Approved PR campaign from GH-01.V1 and effective required-check protection.

Execution: Within the same campaign, introduce one deliberate bounded failure, observe merge blocking, then correct/revert it and observe green. Total expected campaign: three hosted runs, including V1.

Required closure evidence: Retain distinct green/red/green SHAs and run URLs plus actual mergeability/protection evidence. Run additional hosted attempts only when covered by the approval or a subsequent budget decision; do not merge the deliberately failing revision.

### V06-GH-01.V3

- [ ] **V06-GH-01.V3** — Read back effective branch/ruleset settings with appropriate access, record bypass actors and restrictions, and verify ordinary contributors cannot circumvent required checks through direct pushes. **Trace:** [Audit GH-01](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-01).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L1705).  
Starting status: `BLOCKED-OWNER`.  
Unblock prerequisite: Applied protection and permission to read effective settings; an approved way to exercise an ordinary contributor identity.

Execution: Read effective enforcement and bypass restrictions. Use the actual authorized contributor role or report the missing role; an owner bypass is not ordinary-contributor proof.

Required closure evidence: Effective settings and observed contributor restrictions demonstrate required checks cannot be bypassed through ordinary direct pushes. If access hides the effective settings, leave the criterion open.

### V06-GH-02.I3

- [ ] **V06-GH-02.I3** — Protect released version tags against updates and deletion, document one-time tag creation, and require a new prerelease or patch version when source changes after an earlier candidate. **Trace:** [Audit GH-02](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-02).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L1724).  
Starting status: `BLOCKED-OWNER`.  
Unblock prerequisite: Approval for immutable version-tag protection and actor/bypass policy.

Execution: Apply only the approved tag ruleset. Check the target pattern and effective no-update/no-delete enforcement. Prepare a safe testing method without mutating a released tag.

Required closure evidence: Record effective tag protection and the documented new-version policy. Never move or delete a used release tag as a negative control.

### V06-GH-02.V3

- [ ] **V06-GH-02.V3** — Run a complete candidate flow and compare checkout revisions, inventory, packaged evidence and publication metadata; read back the effective tag-update/deletion protection. **Trace:** [Audit GH-02](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-02).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L1731).  
Starting status: `BLOCKED-DEPENDENCY`.  
Unblock prerequisite: Tag protection plus an authorized real candidate flow; publication metadata only becomes available after authorized publication.

Execution: Compare resolved checkout SHAs, producer inventory, packaged/lifecycle evidence, selected verification run and eventual publication metadata. Share evidence with GH-03 and G-09 instead of creating a second candidate campaign.

Required closure evidence: Every boundary agrees on source/version/artifact identity, effective tag protection is read back, and publication-side evidence is present when applicable. Before publication, record completed prepublication portions without closing the full criterion.

### V06-GH-03.V1

- [ ] **V06-GH-03.V1** — Exercise a fresh-version release in a single-runner configuration; the lifecycle job must start only after candidate artifacts exist, without release-not-found polling blocking the producer. **Trace:** [Audit GH-03](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-03).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L1755).  
Starting status: `BLOCKED-DEPENDENCY`.  
Unblock prerequisite: Approved verification sequence, exact revision/tag identity, runner and lifecycle environment.

Execution: Execute the existing Release verify workflow on the approved identity. Observe artifact production before downstream lifecycle consumption on the single-runner arrangement.

Required closure evidence: Record run/job ordering, same-run artifact identity and completion. No baseline/release-not-found polling may block the job that produces the required candidate. A scratch rehearsal is not this real workflow execution.

### V06-GH-03.V3

- [ ] **V06-GH-03.V3** — Run the selected lifecycle path against the exact candidate bytes and confirm publication behavior matches the documented required/non-gating policy, including failure and cancellation outcomes. **Trace:** [Audit GH-03](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-03).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L1757).  
Starting status: `BLOCKED-DEPENDENCY`.  
Unblock prerequisite: Authorized real verification/lifecycle execution and the approved required/non-gating policy.

Execution: Exercise the selected lifecycle path against that run's exact bytes; collect relevant success, failure and cancellation behavior using isolated controls. Preserve the verification/promotion split.

Required closure evidence: The real path produces per-leg verdicts and evidence, and failures/cancellation enforce the stated publication gate. Do not trigger a real public release merely to test a negative outcome; keep controls nonpublishing.

### V06-GH-06.V3

- [ ] **V06-GH-06.V3** — Inspect the completed workflow's artifact collection, not just its upload-step conclusion, and confirm the summary accurately reports both verification outcome and evidence availability. **Trace:** [Audit GH-06](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-06).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L1835).  
Starting status: `BLOCKED-DEPENDENCY`.  
Unblock prerequisite: Completed authorized verification run with its artifact collection accessible.

Execution: Inspect the actual retained collection and download/open relevant inventory, packaged, lifecycle and diagnostic records. Compare contents to the summary and expected evidence contract.

Required closure evidence: Required records exist, identities and contents are correct, and missing/failed evidence is reported accurately. An upload step's green conclusion alone cannot close this row.

### V06-S-25.I3

- [ ] **V06-S-25.I3** — Collect independent benchmark distributions/baseline drift, held-out calibration error and interval coverage, and release queue/failure/evidence-retention metrics. Avoid introducing virtualization or incompatible dependency unification purely from file size/counts. **Trace:** [Measurements after correctness](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored); [Frontend additional observations](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations); [Rust dependency inventory](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#rust).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L2636).  
Starting status: `BLOCKED-ENVIRONMENT`.  
Unblock prerequisite: Independent benchmark/held-out data, identified host/runtime/model conditions and approval for the measurement session.

Execution: Collect the specified distributions, drift, calibration error/interval coverage and release queue/failure/retention observations. State available sample scope and provenance; do not invent sample size or manufacture independence through repeated imports.

Required closure evidence: Observed data supports each claimed metric, with omissions explicit. Preserve the physical-hardware portability program as a related residual, while leaving completed MT-07 identity tests closed. Do not add UI virtualization, dependency consolidation or product optimization as a substitute.

### V06-G-05.I3

- [ ] **V06-G-05.I3** — Perform keyboard/Narrator or NVDA, high-DPI/zoom/high-contrast and reduced-motion checks. Perform live cloud/HF credential scenarios only where an explicitly authorized test account and suitable environment are available. **Covered in the packaged probe:** keyboard traversal with visible focus, reduced motion, high-DPI/zoom, forced-colors high contrast (`verify_a11y.mjs` A11Y_PASS; evidence `release-evidence/0.6.0/attestations/g05-a11y-packaged-verification.log`). **Remaining (blocked):** a manual Narrator or NVDA pass on the packaged app, and live cloud/HF credential scenarios, which require an explicitly authorized test account and suitable environment. **Trace:** [Remaining target verification](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L2854).  
Starting status: `BLOCKED-ENVIRONMENT`.  
Unblock prerequisite: A human operator for Narrator/NVDA and explicitly authorized test accounts for live cloud/HF scenarios.

Execution: Reuse the packaged keyboard/focus, zoom/high-DPI, reduced-motion and forced-color proof where its binding remains applicable. Execute only the missing manual listening/navigation and authorized live-account scenarios.

Required closure evidence: Record actual screen-reader navigation/announcements and the specified credential behaviors on identified candidate bytes. Keep credential values out of files/logs. Automated accessibility-tree checks do not replace listening; lack of an authorized account remains untested, not passed.

### V06-G-08.V1

- [ ] **V06-G-08.V1** — Reconcile task status, required checks, lifecycle results, immutable source/digests and public claims; a green aggregate summary must not hide an unresolved High finding or missing required evidence. **Kept open (2026-09-12, owner directive):** the reconciliation is maintained continuously in the third-pass record, but it closes only with the release decision - the unresolved High rows (GH-01, GH-02, GH-03, GH-06.V3) and the G-09/G-10 gates remain, and a written deferral is not treated as passing evidence. **Reconciled (2026-09-12 third pass):** the open boxes are individually categorised (owner-gated / environment-blocked) with completion criteria and unblock actions; unresolved High rows (GH-01, GH-02, GH-03, GH-06.V3) carry their criteria and are not marked verified; a written deferral is not treated as passing evidence anywhere in this record. **Trace:** [Stabilization release exit criteria](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Audit priorities](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#priority-scale).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L3198).  
Starting status: `BLOCKED-DEPENDENCY`.  
Unblock prerequisite: Current evidence reconciliation, required prepublication High-finding acceptance under the owner-approved sequence in P06-02, FE-05 sign-off, required settings/candidate verification and an explicit owner ship decision.

Execution: Present the proposal from P06-02 with every unresolved criterion and evidence limitation. Apply the existing G-08.I1 policy: required High findings must be fixed and verified; Medium/Low/supplemental deferrals need the full owner/risk/workaround/milestone/evidence-gap record.

Required closure evidence: Record an explicit prepublication decision supported by current evidence, not a green aggregate. A conditional permission to run verification does not close this final ship criterion. Publication/readback follows the decision and is completed in G-09/G-10; do not make those later records a circular prerequisite.

### V06-G-09.I1

- [ ] **V06-G-09.I1** — After the recorded ship decision, create/protect the v0.6.0 tag at the verified immutable SHA and promote the exact tested candidate assets; do not rebuild or retarget a used tag. **Trace:** [Stabilization release exit criteria](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-02); [GH-03](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-03).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L3214).  
Starting status: `BLOCKED-OWNER`.  
Unblock prerequisite: Approved exact source and immutable tag plan, completed required acceptance and explicit authorization for promotion.

Execution: Follow the owner-approved sequence recorded in P06-02. If a tag was authorized earlier solely to run verification, preserve its identity and record that separate authorization. Never retarget it. Promote only the qualified selected run's actual tested assets.

Required closure evidence: The approved tag/revision and promoted bytes are verified and linked to their decisions. The original ordering requirement is not silently considered satisfied by a different sequence: record the owner's approved clarification. Tag existence alone is not full completion of this row.

### V06-G-09.I2

- [ ] **V06-G-09.I2** — Bind release inventory, checksums, packaged/lifecycle records and provenance to that SHA and each artifact digest. Refuse publication when source, version, candidate or required evidence differs. **Trace:** [Stabilization release exit criteria](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-02); [GH-03](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-03).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L3215).  
Starting status: `BLOCKED-DEPENDENCY`.  
Unblock prerequisite: Selected successful qualified run and applicable promotion authorization.

Execution: Use the existing manifest, inventory and promotion validators on the actual selected bundle. Bind source/version, each asset's size/digest, producer packaged record and lifecycle records before publication.

Required closure evidence: Valid actual bundle passes and mismatched source/version/bytes/evidence fails. Preserve historical inventories unchanged. Qualification preparation can be recorded early; the complete criterion follows the selected real release path.

### V06-G-09.I3

- [ ] **V06-G-09.I3** — Read back the published tag/release metadata and complete asset set. Verify downloaded/public byte identity against the candidate where the release gate promises it, and verify Latest/prerelease/unsigned wording matches the intended release policy. **Trace:** [Stabilization release exit criteria](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-02); [GH-03](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-03).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L3216).  
Starting status: `BLOCKED-OWNER`.  
Unblock prerequisite: Actual authorized publication.

Execution: Read back the published tag and release metadata, actual complete asset list and downloaded byte identities. Check Latest/prerelease and unsigned wording against the approved policy.

Required closure evidence: The public assets match the selected producer inventory and promised evidence bytes; results and release IDs are recorded. Draft preparation or workflow artifacts alone are not public readback.

### V06-G-09.V1

- [ ] **V06-G-09.V1** — Exercise wrong SHA, moved tag, modified bytes, missing assets and absent lifecycle evidence as negative publication controls before using the real release path. **Trace:** [Stabilization release exit criteria](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-02); [GH-03](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-03).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L3220).  
Starting status: `BLOCKED-DEPENDENCY`.  
Unblock prerequisite: An approved nonpublishing rehearsal of the actual promotion entry point and candidate layout.

Execution: Exercise wrong SHA, moved-tag resolution, modified bytes, missing assets and missing lifecycle evidence through the real validation path. Use isolated copies or controlled resolution fixtures; never move the actual protected release tag or upload tampered public assets.

Required closure evidence: Each control fails for its intended identity/evidence reason before a publication side effect. Record positive control and each negative separately. Existing unit tests support the result but cannot substitute for required real-entry-point evidence.

### V06-G-10.I1

- [ ] **V06-G-10.I1** — Record final source/release IDs, exact asset digests, successful and skipped verification, closure evidence for completed findings and the remaining risk register. **Trace:** [Remediation and release acceptance](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [QD-04](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#qd-04); [GH-07](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#gh-07); [What to measure next](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored).

Source: [Original criterion](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L3236).  
Starting status: `BLOCKED-DEPENDENCY`.  
Unblock prerequisite: Completed authorized release/readback, or an explicit final release disposition with truthful unexecuted actions.

Execution: Write the final record with exact source/tag/release/run IDs, per-asset measured digests, verification classes, accepted findings, approved deferrals and risk register. Preserve original audit and earlier release history.

Required closure evidence: A completed-release closeout is backed by actual publication/readback. If the owner decides not to release, document that outcome but keep unperformed publication criteria open; a blocked preparation handoff is not a completed v0.6 release.

## Acceptance obligations obscured by historical checked rows

These are existing requirements. Their implementation is not being reopened. A single real campaign may satisfy related V06 rows once its assertions and evidence actually cover them.

### R06-01 — Native preservation and lifecycle acceptance

- [ ] **R06-01 — Obtain the still-missing native evidence for V06-GH-04.V3 and V06-G-06.I2.**

Starting status: `BLOCKED-ENVIRONMENT`, with candidate/run authorization also required when executing a new campaign. Trace: [V06-GH-04.V3](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L1783); [V06-G-06.I2](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L3149); [current F9-05 limit](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/RELEASE-REVIEW-0.6.md#L318); original audit GH-04 and stabilization exit criteria, linked by those rows.

The historical tracker checks these rows, while current F9-05 explicitly leaves corrected native verification blocked. Use the current scope and actual manifest, not the older checkmarks or an ambiguous “two reruns” statement. Prepare the supported clean-install/start/uninstall and four named upgrade/preservation legs, showing exactly which require new native execution: upgrades from v0.4.0 and v0.5.0, preservation from v0.4.1 and v0.5.0. Keep the corrected v0.4.1 real profile/settings/cache assertions separate from v0.5.0 SQLite/user-override assertions.

Reuse `scripts/sandbox/run-lifecycle-in-sandbox.ps1`, `host-run-lifecycle.ps1`, `build_preservation_fixture.py` and `verify_preservation.py`; inspect their actual interfaces before using them. Bind candidate and installed-payload identities with the established bundle-marker rules, source and harness revision, fixture seed/readback, terminal result and original logs. Do not revive the already resolved assumption that every installed payload must have the portable's exact unmodified digest.

Acceptance: each required pending native assertion actually runs in a suitable isolated Windows environment on the selected candidate, with successful preservation and strict lifecycle verdicts. Deliberate loss/corruption of seeded required data fails the assertion. Existing fixture/component tests and provenance-valid historical records remain valuable but do not stand in for an unexecuted current native assertion. Share the resulting records with GH-03/GH-06; do not duplicate the campaign simply because two task IDs refer to it. If the native facility remains unavailable, preserve BLOCKED and the exact missing capability without repeated boot attempts until one happens to pass.

### R06-02 — Explicit FE-05 finding acceptance

- [ ] **R06-02 — Obtain the existing owner sign-off for FE-05.**

Starting status: `BLOCKED-OWNER`. Trace: [FE-05 finding-level hold](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L1278); [V06-G-08.V1](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L3198); original audit FE-05.

Present the already accepted evidence that active/completed run state survives navigation and teardown, completion elsewhere stays visible, both profiles' saved manifests/anchors recover provenance by identity, and I1–I5 map to implementation commits and regressions. Use existing evidence and its actual source bindings; no new A/B walkthrough is required solely to request sign-off.

Acceptance: the owner explicitly accepts the finding's stated completion criteria in the release decision. Record the decision reference. Until then, the finding-level hold remains visible even though its implementation/test cells are checked.

## Scope clarifications that prevent new work from being invented

- **Signing:** [V06-G-08.I3](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L3194) permits the disclosed unsigned policy. SignPath is an optional owner decision, not a new mandatory blocker. Continue truthful unsigned/SmartScreen/checksum disclosure unless the owner changes that policy. If signing is later chosen, record its effect on actual artifact bytes and affected qualification before promotion; do not add signing work now merely to clear an old owner-items list.
- **MT-07:** [V06-MT-07.V2](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md#L926) is accepted for the five canonical-snapshot identity scenarios. Physical CPU-only/changed-CPU throughput portability remains a measurement-program limitation associated with RT-06/S-25. It does not reopen the completed identity tests.
- **RT-06:** retain the measured four-backend scope and three correct vendor refusals. The separately open seven-installed portion is not satisfied by a larger harness check count.
- **Current CI:** its exact-SHA success is accepted. Do not repeatedly run full suites, recut freezes or rebuild binaries to rewrite status prose. Run meaningful affected checks after a concrete change and satisfy applicable repository gates; reuse evidence whose scope remains valid.
- **New defects:** report only concrete evidence encountered on an in-scope required path. Map it to the affected original criterion and describe the minimal correction and regression before changing behavior. Do not reopen an accepted finding based on a hypothesis or expand into general refactoring. Product/workflow behavior changes beyond this residual acceptance plan require an explicit scope decision.

## Execution order and approval checkpoints

1. Complete P06-01 through P06-03 using available source, evidence and prepared tooling. Keep progress in this file. Prepare missing decisions before declaring the entire goal blocked.
2. Reuse applicable owner approvals. Execute approved protection/PR actions and available authorized external scenarios, recording their results. Different independent actions may proceed without waiting for unrelated hardware.
3. Obtain explicit approval for the exact source/tag/verification sequence in P06-02. Execute the existing verification workflow once for the selected candidate; preserve each run's actual outputs and failure history. Qualification and native evidence can jointly satisfy several criteria without duplicate runs.
4. Resolve the required evidence and FE-05 hold, then record the G-08 final ship decision. Approved deferral of eligible work does not turn unexecuted tests into PASS and does not silently waive the default High-finding rule.
5. With explicit promotion/publication approval, run the actual nonpublishing controls/dry run, promote the selected qualified bytes, read back the public release and finish G-09/G-10.
6. At any point where every remaining action depends on absent permission, access, a human or unavailable equipment, complete P06-04 and pause the single goal as BLOCKED. Resume the same tracker when the missing input arrives. Do not generate more work to keep the loop alive.

## Evidence and decision ledger

Append actual observations below. Empty templates are not results. Point to existing records where sufficient; do not copy large logs or credentials into the tracker.

| Action / original criterion | State | Authorization or prerequisite | Source / artifact / harness identities | Command or observed scenario | Result and evidence location | Remaining limit / next step |
|---|---|---|---|---|---|---|
| Baseline checkpoint | VERIFIED | Read-only review | source `076a3eeecbdf8fa149a4c5f675526ad4ae24b58e`; historical producer separately `e9a36b3...` | Independent scoped review; exact-SHA CI | CI34774614565 success; 150/150 focused checks; 18-record manifest validates | Remaining acceptance/publication actions above are still open |
| P06-01 state reconciliation (2026-09-13) | VERIFIED | Available preparation work | HEAD=origin/main=`076a3eeecbdf8fa149a4c5f675526ad4ae24b58e`; tree clean except untracked residual tracker | `git rev-parse`, `git status`, `gh run view 34774614565`, box-line counts, link-spot checks | Baseline CI run 34774614565 success (check, rust-audit, Security audit, package-smoke; pr-check correctly skipped on push); source tracker 645 checked / 19 unchecked; residual tracker 25 unchecked / 0 checked; `check_tracker.mjs` still reports the source file only (72 findings, 662 boxes) and was not misread as covering the new file | Doc-index pointer still needs the owner's preferred location (AGENTS.md edit blocked: approval prompt timed out); source tracker left byte-identical |
| P06-02 decision packet (2026-09-13) | VERIFIED | Available preparation work; no approval consumed | Reviewed code revision `076a3eeecbdf8fa149a4c5f675526ad4ae24b58e`; historical producer `e9a36b3...`; docs `RELEASE-REVIEW-0.6.md` sections 1-2 | Read `release.yml`, `release-promote.yml`, `ci.yml`; read back rulesets (`[]`) and branch protection (`false`); local `mt06_verdicts` 9/9 and manifest re-verify (18 records, 27 PASS lines, exit 0) as packet inputs | Packet persisted in the operative proposal section of this tracker: superseded 16a8317 recipe, exact ruleset payloads (already in review doc), three-run PR campaign with no promised duration, tag/verify/promote/dry-run/publish/readback sequence with the confirm-phrase and drift gates; GH-02.V3 two-stage approval need stated | All approvals still missing; nothing applied, tagged, or published |
| P06-03 capability inspection (2026-09-13) | VERIFIED | Read-only inspection; no campaign executed | This host: `WindowsSandbox.exe` present, single NVIDIA RTX 5090, AMD-RAID SSD array + USB flash (no HDD/NVMe/SATA-SSD split confirmed) | `ls`/`Get-CimInstance`/`Get-PhysicalDisk`; harness interface reads (`g05_rt06_all_backends.mjs` usage, `host-run-lifecycle.ps1` params, `verify_preservation.py` usage) | All five harnesses exist; exact interfaces recorded (CDP port + adapter ID; Tag/Version/PreviousTag/CandidateDir/EvidenceName/Flavor/FaultSimulation; mirror/cache canary files). No run output invented | Native execution still needs owner authorization + Sandbox boot + published baselines (v0.4.0/v0.5.0/v0.4.1/v0.5.0 assets); a representative host with the required seven-backend combination (three-vendor host, not seven GPUs), storage-class targets, measurement program, human operator, and test accounts remain unavailable. No campaign duration is promised. |
| Reconciliation (2026-09-13) | VERIFIED | Documentation-only reconciliation; no new approval consumed | Reviewed docs commit `09c3f59`; CI [run 34779617259](https://github.com/sato942/localmotive/actions/runs/34779617259) success for that SHA | Checked P06-01..P06-04 against their preparation-only acceptance; persisted the operative proposal, environment matrix, and proposed release sequence in the tracker; added the review-doc pointer; no code/build/soak/freeze change | Tracker now 4 checked / 21 unchecked (generated counts); source tracker still 645/19; crosswalk (19 criteria + R06-01/R06-02) intact | Commit kept local; CI for any new documentation commit honestly reported per-SHA; goal stays paused until a real prerequisite changes |
| Bounded repair, paste-7 order (2026-09-14) | VERIFIED | Standing release authorization; off-tag repair only, tag untouched | Harness HEAD `353491e70f29d593cd183cc7757f0815ff1ba167` (PR #26 merge `b00ac08` on PR #25 `0e72fc0`); tag `v0.6.0` still object `92385b2da503d761fa2ba1eaae62911f2c0e98d2` peeling to `27349f93d7d17eac9b15a6895cd94e850a122a8b`; rulesets 23218751/23218749 active, no bypass actors; producer/tagged bytes unchanged | `scripts/verify_packaged_matrix.ps1` revision 2 (owned-handle cleanup: `Start-OwnedFixture`/`Stop-OwnedFixture`/`Stop-OwnedProcess` via .NET handles, `__serve` adoption validation, `Invoke-Verifier` exits from handles, `$AttemptId`-scoped paths, outer `finally`; zero `taskkill`/port/name scans); `scripts/tests/verify_cleanup_matrix.ps1` 8-scenario acceptance; PR #26 via protected path (hosted pr-check green) | `verify_cleanup_matrix.ps1` 8/8 GREEN on Windows PowerShell 5.1; 6 fail on prior implementation for expected reasons (int-pid binding, missing owned-fixture stop, bare-pid kill, stale `$PID`); release-gates 144/144, workflow-gates ok, integration 10/10, `npm test` 248/248; `Start-OwnedFixture` rehearsed live (init PASS, sig HTTP 200, `__serve` cmdline confirmed, prep-kill shape observed); stale worktree removed; zero owned processes left | Rehearsal is orchestration-only, not a tag verification run; the `v0.6.0` tag/version disposition (exact proposed SHA, checks, immutable-tag exception) is presented in the report, not executed here; lifecycle/qualification still require the final verification chain |

Use the following fields for each owner decision: decision date, actor, action IDs, exact allowed revision/tag/run/assets or settings payload, cost/test-account/environment limits where applicable, decision text/reference, and whether it permits verification, promotion dry run or publication. Never fill an approval field by inference from “goal complete,” a green test or a prepared command.

## Completion and blocked-stop contract

**Full TODO completion** requires the original acceptance criteria to be met with appropriate evidence, native obligations and FE-05 hold resolved, required owner decisions recorded, and authorized release/publication/readback/closeout performed. If an eligible requirement is explicitly deferred, report **release accepted with named deferrals**, not “every TODO verified.” Keep its original unchecked criterion and follow-up record.

**Blocked stop** is required when all available authorized work is exhausted and every remaining action has a concrete external prerequisite. It is a valid stopping point for autonomy, not full TODO completion. Do not ask repeatedly for the same missing approval, automatically resume a blocked loop, create subgoals or change the acceptance wording to satisfy a judge.

The final handoff must state the baseline and current HEAD/remote state; exact CI/run evidence applicable to each revision; verified, blocked and deferred action IDs; source/asset/harness distinctions; owner decisions consumed and still required; external inputs; no outstanding unreconciled owned processes; and the precise next action for each blocker. Use `BLOCKED — available authorized work complete` when appropriate. Use a completed-release claim only after real release evidence exists.

## Source index

- [Original TODO at 076a3ee](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/TODO-0.6.md)
- [Original audit and exit policy](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/history/localmotive-comprehensive-audit.md)
- [Release review and historical handoff](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/docs/RELEASE-REVIEW-0.6.md)
- [Remediation report](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/REPORT-0.6.md)
- [Release verify workflow](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/.github/workflows/release.yml)
- [Separate promotion workflow](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/.github/workflows/release-promote.yml)
- [Historical qualification manifest](https://github.com/sato942/localmotive/blob/076a3eeecbdf8fa149a4c5f675526ad4ae24b58e/release-evidence/0.6.0/qualification-manifest-0.6.0.json)
- [Exact-baseline CI](https://github.com/sato942/localmotive/actions/runs/34774614565)
