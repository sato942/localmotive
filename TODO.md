# Localmotive tracker

This file is the one tracker for Localmotive. It lists the open work in
priority order.

Basis: `REVIEW.md` (2026-09-24, source `9b098577095ab80ead9bff6e5475456166b340c1`).
Finding IDs such as RT-05 refer to that review.

The closed trackers, the earlier audit, and the 0.6 reports are in git history
only. The last tree that holds them is `9b098577095ab80ead9bff6e5475456166b340c1`.
Do not restore them to the tree.

## How to use this tracker

- Each item states one outcome, its acceptance criteria, and its evidence.
- Before you fix a defect, write a test that fails for the stated reason.
  Record that failure in the ledger.
- Record evidence in section 11 in this format:
  `L-nn | date | revision | command or run ID | artifact | PASS, FAIL, or UNKNOWN`.
- A checkbox is not proof. Check a box only when a ledger line says PASS.
- The dev team decides every item in this tracker. No item waits for owner
  approval (release authority in `AGENTS.md`, 2026-09-24). An item marked
  **ADMIN** needs a one-time change in GitHub settings that only the
  repository admin can make.
- When an item is complete, check it and write its ledger ID after it.
- Do not add another tracker, report, review snapshot, or campaign document
  to the tree.

---

## 1. State on 2026-09-24

| Fact | Value | Evidence |
|---|---|---|
| Last published release | `v0.5.0` (`a4b7127`), published 2026-09-10T18:56:46Z | L-02 |
| `v0.6.0` | Tag at `27349f9`. Never published. The tag is immutable. | L-04 |
| `v0.6.1` | Tag at `4b31431`. Never published. The tag is immutable. | L-03, L-04 |
| `main` | `0bd39fb` | L-03 |
| Lane branch | `fix/u06-stabilization` at `9b09857`, merged into `main` | — |
| Version in `package.json` | 0.6.1 | — |
| Rulesets | `main-pr-check` (23218749) and `immutable-release-tags` (23218751), both active. The owner is the only bypass actor on `main-pr-check` (readback 2026-09-20). | old tracker, U06-05 |
| Pull-request rule | strict `pr-check`, 0 required approvals. Tag rule: no update, no deletion; creation is open to Write. | `gh api` readback 2026-09-24 |
| Collaborators | `sato942` (admin) only | `gh api` readback 2026-09-24 |
| Self-hosted runners | 1: `DESKTOP-HPTF57N-zen5-blackwell` (owner's PC), labels `self-hosted, Windows, X64, zen5, blackwell, localmotive-hw` | `gh api` readback 2026-09-24 |
| Environments | 0. `release-promote.yml` needs only the phrase `PUBLISH <tag>`. | `gh api` readback 2026-09-24 |
| Local gates at `9b09857` | PASS | L-01 |
| Release runs after `v0.5.0` | 14 started, 0 passed | L-02 |
| Promotion runs, all time | 0 | L-02 |

Why nothing ships: `REVIEW.md` sections 1 and 2. In short:

1. The gate needs a live GitHub API call that a fresh profile cannot make
   reliably, and the harness discards the error kind (RT-05, RT-06, LAB-01).
2. The gate needs developer-only files that a clean runner does not have
   (REL-04).
3. Promotion rejects the dispatch run that was chosen as the final producer
   (REL-01).
4. Owner stops forbade the other exits (REL-02, DOC-06). The owner lifted
   them on 2026-09-24 and gave the team release authority.

---

## 2. Standing rules

Release authority: the dev team has standing authority to bump versions,
create tags, run release qualification, and publish releases. The owner
granted it on 2026-09-24 (`AGENTS.md`, "CI and publication"). The owner stops
of 2026-09-23 are lifted: promotion, changes to `release-promote.yml`, new
tags, new versions, and `workflow_dispatch` runs are allowed. A used tag
still never moves.

Standing release policy:

- Releases ship unsigned. No signing step exists, and none is pursued. Each
  release discloses the unsigned files and the SmartScreen limits.
- Claim support only for the scope that the evidence for that version covers.
- The self-hosted runner is the validation gate for trusted Windows runs.
  `windows-latest` runs only the untrusted `pr-check` job. It is not a
  release gate.
- Before a push, run the default-parallel local suite and get a green result.
- Land each gate or verifier change RED before GREEN, with a mutation proof.
- Fix flaky tests at the cause (unique temporary directories, robust
  fixtures, load-aware assertions). A thread-count pin is a temporary
  mitigation only.
- Merges: `pr-check` must pass (strict). The rule requires 0 approvals, so a
  Write member can merge a PR after `pr-check` passes.

---

## 3. Team decisions

- [x] **D1 — Exit the release deadlock.** Decision: Option A, 2026-09-24,
  under the delegated authority. Fix P0-1 to P0-4, bump the version once, tag
  a commit on `main`, and let the push-event run qualify that tag. `v0.6.1`
  stays unpublished. The team can change this decision before the tag.
  Options B and C are in `REVIEW.md` section 3.
- [x] **D2 — Adopt the reduced release gate.** Adopted in `AGENTS.md`, "CI
  and publication", on 2026-09-24. P0-7 implements it. The lab campaign moves
  to `hardware-qualify.yml` and does not block a release.
- [ ] **D3 — Remove `artifacts/` from the tree (TEAM).** 5,494 tracked files,
  including 33 WebView2 profiles (DOC-01, DOC-02). A rewrite of public history
  needs a force push, and only the owner can bypass `main-pr-check`.
- [ ] **D4 — Decide the `v0.4.0` corrective note (TEAM).** Publish
  `release-evidence/0.4.1/v0.4.0-corrective-note.md`, or delete it.
- [ ] **D5 — Decide superseded evidence (TEAM).** `AGENTS.md` keeps accepted
  evidence frozen. Allow the deletion of `release-evidence/0.6.0/history/`,
  or keep it.
- [ ] **A1 — Give the team access (ADMIN).** Give each developer the Write
  role. Give the release lead Maintain or Admin. Before the first Write grant
  takes effect, run the direct-push refusal test (section 8, V06-GH-01.V3).
- [ ] **A2 — Remove the single-runner dependency (ADMIN).** Register a
  self-hosted Windows runner that the team controls, or give the release
  lead Admin so that the team can register one. See P1-12.
- [x] **D6 — Approve the `AGENTS.md` pointer change (OWNER).** The owner
  approved it on 2026-09-24 ("edit AGENTS.md"). L-07.
  - `AGENTS.md:185-187` now reads: "Read `REVIEW.md` for the current
    findings. Closed trackers and the earlier audit are in git history; the
    last tree that holds them is `9b09857`."
  - `AGENTS.md:190` now reads "Keep accepted evidence frozen." The words
    "historical trackers" were removed, because the closed trackers are no
    longer in the tree.
  - `docs/history/localmotive-comprehensive-audit.md` was deleted.

---

## 4. P0 — Make the next release possible

Do these items in order. P0-1 to P0-6 are product or harness defects. They do
not wait for D1.

- [x] **P0-1 — Build the runtime catalog from the compiled approval (RT-05,
  REL-06, LAB-02).** L-12.
  - Behavior: `load_runtime_setup` and `fetch_runtime_catalog` return the
    approved `b10816` catalog from `src-tauri/approved_runtimes.json` without
    a network call. A download still checks the byte count and the SHA-256
    from that file.
  - A check for a newer upstream release stays a separate, manual action. It
    must not block setup.
  - RED test: a Rust test with no network and an empty cache expects a
    catalog with the 13 approved assets. Today the call returns an error.
  - Packaged check: the packaged matrix passes `ipc.runtime-setup`,
    `ipc.runtime-catalog-fetch`, and `ipc.runtime-recommendation` while the
    app cannot reach `api.github.com`.

- [x] **P0-2 — Share one catalog load between concurrent callers (RT-06).** L-13.
  - Behavior: two concurrent calls to `load_runtime_setup` or
    `fetch_runtime_catalog` get the same result. Neither call gets
    `kind: Busy`.
  - RED test: start two calls at the same time against a slow fetch double.
    Today the second call returns `Busy`.
  - Mutation proof: put back the fail-fast guard and confirm that the test
    fails.

- [ ] **P0-3 — Keep the error kind in harness evidence (LAB-01).**
  - Behavior: `scripts/verify_041.mjs` records `catalogError.kind` and the
    message for each failed runtime check. No reason field contains
    `[object Object]`.
  - RED test: a script test gives the harness a structured error
    `{ kind, message }` and expects both values in the record.

- [ ] **P0-4 — Fix the catalog query wire keys (FE-02).**
  - Behavior: the frontend sends `pipelineTag`, `fitPerMille`, and
    `budgetBytes`. The pipeline filter and the hardware-fit filter change the
    result list.
  - RED tests:
    - a Rust test that reads the frontend-shaped payload and expects the
      filters to apply;
    - a Vitest test for the query builder in `src/model.ts`;
    - a `CatalogQuery` entry in `scripts/tests/fixtures/ipc-contract.json`.
  - Packaged check: in the packaged app, select a pipeline filter and assert
    that the result count changes.

- [ ] **P0-5 — Stage release files outside the checkout (REL-05).**
  - Behavior: `release.yml` writes candidate and diagnostics files to a
    directory under `$RUNNER_TEMP`, not to `artifacts/`. Each uploaded
    artifact holds only files from the current run.
  - RED test: a workflow test rejects a staging path inside the workspace.

- [ ] **P0-6 — Name the failed step correctly (REL-09).**
  - Behavior: when packaging passes and verification fails, the summary names
    the verification step. It does not say "Packaging failed".

- [ ] **P0-7 — Cut the release gate (D2; REL-03, REL-04, REL-07,
  REL-12, REL-13).**
  - Cut `release.yml` to the release gate in `AGENTS.md`, "CI and
    publication".
  - Remove the lab, witness, fault, and manifest steps from `release.yml`.
    Move the lab legs that stay useful to `hardware-qualify.yml` as jobs that
    do not block.
  - Remove each dependency on `.hermes-0.6/`. A clean clone must hold every
    input of the gate.
  - Reduce `build_qualification_manifest.mjs` and
    `verify_qualification_manifest.mjs` to the records of `REVIEW.md`
    section 4, or delete them with their tests.
  - Set `cancel-in-progress: false` for the release concurrency group.

- [ ] **P0-8 — Keep promotion aligned with the producer (D1 = Option A;
  REL-01, REL-02, REL-08).**
  - Keep the push-event rule (`release-promote.yml:86-96`). The new tag gets
    a push-event run, so promotion needs no change for option A.
  - Implement the extra-file check that `verify_release_promotion.mjs:8` and
    `:43` promise, or delete those comments.

- [ ] **P0-9 — Correct the public text (DOC-04, DOC-05).**
  - `README.md` names `v0.5.0` as the current release. It does not describe
    0.6.0 or 0.6.1 as shipped.
  - `CHANGELOG.md` marks 0.6.0 and 0.6.1 as not published.
  - `docs/SUPPORT-MATRIX.md`, `docs/EVIDENCE-MATRIX.md`, and `README.md`
    make the same support claims.

- [ ] **P0-10 — Release 0.6.2 (D1).**
  - Preconditions: P0-1 to P0-9 PASS. Follow `HANDOVER.md` section 5.
  - Steps for option A:
    1. Bump the version once in each version field.
       `node scripts/verify_versions.mjs` passes.
    2. Add the user-facing changes to `CHANGELOG.md`.
    3. Merge to `main`. Tag one commit on `main`.
    4. Let the push-event `release.yml` run qualify the tag.
  - Acceptance:
    - The verify run passes. The ledger records the run ID, the peel SHA, and
      the SHA-256 of the portable EXE, the MSI, and the NSIS installer.
    - On the exact candidate bytes, the lifecycle leg passes: clean install,
      upgrade from `v0.5.0` with user data kept, and uninstall with no
      leftovers.
    - The candidate bundle holds the inventory, the checksums, the SBOM, and
      the packaged record. Each record names the peel SHA.
    - Negative publication controls fail as expected: wrong SHA, changed
      bytes, and a missing asset.
    - The release lead runs `release-promote.yml`, first with
      `dry_run: true`, then with `dry_run: false`. No owner approval is
      needed. The workflow publishes the same bytes. Nothing is rebuilt.
    - Promotion occurs before the qualified bundle expires. `release.yml`
      keeps the bundle and the diagnostics for 90 days.
    - Readback: the public tag, the release metadata, the asset set, and the
      downloaded digests match the candidate. The release is marked Latest.
      The notes disclose the unsigned files and the SmartScreen limits.
    - One ledger line records the final source, the run IDs, the asset
      digests, and the open risks.
  - This item replaces U06-06, U06-07, U06-08, U06-09, V06-GH-02.V3,
    V06-GH-03.V1, V06-GH-03.V3, V06-GH-04.V3, V06-GH-06.V3, V06-G-06.I2,
    V06-G-08.V1, V06-G-09.I1, V06-G-09.I2, V06-G-09.I3, V06-G-09.V1, and
    V06-G-10.I1 from the old tracker.

- [ ] **P0-11 — Revert the host network change after the release.**
  - On 2026-09-22, the release host got a reversible network change: IPv4
    prefix precedence from 35 to 46, DNS order with 1.1.1.1 first, and IPv6
    unbound from Ethernet. The old tracker says: "IPv4-prefer stays until
    green."
  - After P0-10 passes, restore the original settings. Record the before and
    after values in the ledger.
  - P0-1 removes the reason for this change.

---

## 5. P1 — Verified defects

Fix these after P0 and before the next feature.

- [ ] **P1-1 — CORE-04:** Rust rejects a launch profile whose model is not the
  first shard of its set. RED test: a profile that points at shard 2 of a
  3-shard set gets a validation error that names the first shard.
- [ ] **P1-2 — MT-10:** The cloud brief sends only the fields that the
  disclosure lists. Remove `adapterId`, `compatibilityId`, and `physicalId`
  from the payload, or add them to the disclosure. RED test on the serialized
  brief in minimal mode and in full mode.
- [ ] **P1-3 — RT-02:** Install keeps the rollback copy until the final
  verification passes. RED test: make the final verification fail and expect
  the previous runtime back in place.
- [ ] **P1-4 — LAB-04:** Delete `scripts/g05_vitems_c.mjs`, or make it stop
  only the processes that it started, by PID. Search all scripts and
  workflows for stops by name or by port, and remove each one.
- [ ] **P1-5 — REL-10:** Where a workflow stops the app, stop the whole
  process tree that the step started.
- [ ] **P1-6 — PROC-01:** Put the child process in the job object before it
  runs: create it suspended, assign it, then resume it.
- [ ] **P1-7 — FE-01:** Clear each credential draft on every exit path,
  including a failed save and an unmount.
- [ ] **P1-8 — DL-02:** Use checked arithmetic for GGUF split metadata. RED
  test with a split number of `u64::MAX`.
- [ ] **P1-9 — CORE-01, CORE-06:** Accept the verification overrides only with
  the isolated root. Show a verification-mode banner while an override is
  active. Document the WebView2 remote-debugging variable in `SECURITY.md`.
- [ ] **P1-10 — RT-01, RT-03:** Close the same-user races with handle-based
  opens and the existing execution lease.
- [ ] **P1-11 — Check the 31 unverified High findings in `REVIEW.md`.**
  For each finding, reproduce or refute it. Record VERIFIED or REFUTED in the
  ledger. Add a P1 item for each verified defect.
  - Release and lab: REL-11, LAB-03, LAB-05, LAB-06.
  - Documentation: DOC-05, DOC-07.
  - Runtime: RT-04, RT-07, RT-10.
  - Core: CORE-03, CORE-05.
  - Process and health: PROC-02, PROC-03, PROC-04, PROC-05, PROC-06,
    PROC-07, PROC-08.
  - Downloads and cloud: DL-01, DL-03, DL-04.
  - Measurement: MT-01, MT-03, MT-05, MT-06, MT-07, MT-08, MT-09.
  - Frontend: FE-04, FE-05, FE-06.
- [ ] **P1-12 — Decouple the release jobs from the hardware host.** Today
  `release.yml` and `release-promote.yml` need all six labels of the one
  runner on the owner's PC.
  - Behavior: the release jobs use `[self-hosted, Windows, X64,
    localmotive-release]`. `hardware-qualify.yml` keeps the hardware labels.
    At least one runner that the team controls carries `localmotive-release`
    (A2).
  - Evidence: one green `release.yml` run on that runner.

---

## 6. P2 — Simplification after the release

- [ ] **P2-1 — Remove `artifacts/` (D3).** Add a `.gitignore` rule.
  Remove the `.gitattributes` LF rules for `artifacts/`. Confirm that no test
  or workflow reads the directory.
- [ ] **P2-2 — Delete dead scripts (LAB-05).** List each script in `scripts/`
  with its callers. Delete each script that no `package.json` script,
  workflow, or kept test calls. `scripts/g05_dc01.mjs` is one (FE-03).
- [ ] **P2-3 — Keep one CDP client (LAB-03):** `scripts/lib/cdp_client.mjs`.
- [ ] **P2-4 — Replace source-text tests with behavior tests.** Delete the
  test-only `ALL_SOURCES` (`src-tauri/src/lib.rs:17-37`). Delete the prose
  assertions in `scripts/tests/release-gates.test.mjs`.
- [ ] **P2-5 — Remove the audit-ticket labels** from Rust source (399) and
  rename ticket-named test modules by behavior.
- [ ] **P2-6 — Keep one benchmark system (MT-09, FE-05).**
- [ ] **P2-7 — Keep one catalog schema source (LAB-06).**
- [ ] **P2-8 — Split `src/App.tsx` (FE-04)** and keep IPC out of the screens
  (FE-06).
- [ ] **P2-9 — Merge `docs/EVIDENCE-MATRIX.md` into
  `docs/SUPPORT-MATRIX.md`.**
- [ ] **P2-10 — Prune superseded evidence (D5).**
- [ ] **P2-11 — Remove stale references to deleted files:** the
  `HISTORICAL_FILES` entries in `scripts/verify_branding.mjs`, and the note in
  `catalog/providers.json` that names `docs/history/TODO-0.5.md` (change it
  at the next catalog change).
- [ ] **P2-12 — Triage the 72 Medium and 9 Low findings** in `REVIEW.md`, one
  part at a time.

---

## 7. Open risks

- **Startup slot.** The post-spawn local-client constructor can return before
  `clear_starting` runs. This matches PROC-04. It needs a controlled
  reproduction.
- **LoRA paths.** The pinned runtime splits scaled LoRA entries on every
  colon. A Windows drive path is therefore unproven. Reproduce with the
  qualified runtime bytes before you propose a correction.
- **Legacy ownership corrections** need an independent review.
- **Shared checkout.** A file once disappeared from this checkout without an
  owning command. Look for another worker that uses the same checkout.
- **Harness quoting.** On 2026-09-23, PowerShell-generated JavaScript put the
  literal text `+ JSON.stringify(...) +` into the page, because a
  single-quoted `'' + expression + ''` boundary was escaped. Do not build
  JavaScript by string concatenation in PowerShell. Pass values with a
  placeholder replace, and run `node --check` on generated scripts.
- **Measurement science (S-25.I3).** Both items are BLOCKED-INPUT:
  - Independent benchmark distributions and baseline drift: no external
    dataset and no multi-day drift program. Do not turn one tune session into
    a distribution. Reopen only if the owner funds a measurement protocol.
  - Held-out calibration error and interval coverage: the tuner table is not
    a held-out study. Report no coverage percentages from in-sample trials.
    Keep the modest wording "Estimated interval".

---

## 8. Recorded deferrals

These items are not executed and not PASS. Support claims must exclude them.

- **D06-01 to D06-05** — CLOSED-DEFERRED-FAR-FUTURE 2026-09-21: a
  seven-vendor GPU host, a storage-class and OS-crash facility, physical
  configurations that were not provided, a same-model multi-GPU rig, and
  hardware-dependent metrics.
- **V06-RT-06.V3** (all seven backends installed) — CLOSED-DEFERRED-FAR-FUTURE
  2026-09-21. The evidence covers one NVIDIA host (RTX 5090), NVMe only.
- **V06-DC-12.V3** (OS-crash and power-loss validation) —
  CLOSED-DEFERRED-FAR-FUTURE 2026-09-21. Only process-crash tests exist.
- **V06-MT-07.V2** (live portability matrix) — CLOSED-DEFERRED-FAR-FUTURE
  2026-09-21. No CPU-only machine. Make no claim that calibration works on
  CPU-only hosts.
- **V06-G-05.I3** (manual Narrator and NVDA listening, live cloud and Hugging
  Face accounts) — CLOSED-DEFERRED-USER-REPORTS 2026-09-21. The automated
  accessibility evidence stands. Real user reports are the feedback path.
- **V06-GH-01.V3** (a normal contributor cannot push past the required
  checks) — WAIVED 2026-09-20, because no Write collaborator exists. Trigger:
  before the first Write collaborator gets access, run a direct-push refusal
  test with that identity.

Reopen a deferral when the missing environment exists. The team decides.

---

## 9. Backlog

These items are not scheduled.

- UI speed and responsiveness. Measure first.
- Bounded UI input: no value outside its valid range (see CORE-05).
- Smarter tuning search without AI. Research the algorithms first.
- In-app version check and automatic update. This needs a design decision on
  update signing.
- Modern inference methods (DSpark2, n-gram, MTP). Research first.
- Model quality measurement with lighteval.

---

## 10. Documentation changes on 2026-09-24

`REVIEW.md` section 11 gives the reason for each file. These files were
deleted from the tree. Git history keeps them. The last tree that holds them
is `9b09857`.

- Second tracker and reports: `TODO-0.6-post-076a3eeecbdf.md`,
  `REPORT-0.6.md`, `agents_feedback.md`, `ideas.md`.
- Superseded reviews: `docs/RELEASE-REVIEW-0.6.md`,
  `docs/localmotive-0.6-followup-review-db548c8.md`.
- Campaign and snapshot documents: `docs/QUALIFICATION-MAP-0.6.md`,
  `docs/qualification-tests.md`, `docs/OPTION_MAP.md`,
  `docs/Future_branding.md`.
- Unpinned upstream copy: `docs/LLAMA-SERVER-README.md`.
- Closed trackers and the earlier audit: `docs/history/TODO-0.4.1.md`,
  `docs/history/TODO-0.4.md`, `docs/history/TODO-0.5.md`,
  `docs/history/TODO-0.6.md`, `docs/history/TODO.md`, and
  `docs/history/localmotive-comprehensive-audit.md` (deleted after D6).
- Dead scripts that read the deleted trackers: `scripts/check_tracker.mjs`,
  `scripts/verify_tracker_links.mjs`.

The live items of `agents_feedback.md` and `ideas.md` are in sections 7
and 9.

To restore one file: `git show 9b09857:<path> > <path>`.

---

## 11. Evidence ledger

- `L-01 | 2026-09-24 | 9b09857 | npm run check; npm audit --audit-level=moderate; verify_versions, verify_workflow_pins, verify_workflow_gates, verify_workflow_syntax (node.exe); verify_cleanup_matrix.ps1; cargo fmt --check; cargo clippy --locked --all-targets -- -D warnings; cargo test --locked | local logs | PASS` — node tests 276 of 276, Vitest 282 of 282, audit 0 vulnerabilities, cleanup matrix 12 of 12, Rust 628 passed and 7 ignored. The first run of the four workflow verifiers failed with `stdin is not a tty`. That was a tooling failure. The `node.exe` rerun passed.
- `L-02 | 2026-09-24 | — | gh run list --workflow release.yml; gh run list --workflow release-promote.yml; gh release view v0.5.0 | — | FAIL` — after the `v0.5.0` release: 14 `release.yml` runs, 0 passed. `release-promote.yml`: 0 runs. Last published: `v0.5.0` at 2026-09-10T18:56:46Z.
- `L-03 | 2026-09-23 | 0bd39fb | runs 35897526108, 35901604360 (workflow_dispatch, tag v0.6.1) | artifact localmotive-0.6.1-verify-diagnostics | FAIL` — step #19 failed on `ipc.runtime-recommendation` ("[object Object]"), `ipc.reject-unknown-adapter`, and `ipc.runtime-catalog-fetch` ("[object Object]"). Step #18 passed. Steps #24 to #27 did not run.
- `L-04 | 2026-09-14 to 2026-09-22 | 27349f9, 4b31431 | v0.6.0 runs 34803387581 to 34819219725 (6 runs, 6 SHAs); v0.6.1 run 35731266321 (6 attempts) | — | FAIL` — `v0.6.1`: attempt 1 failed the main-ancestry gate, attempts 2 to 5 failed the packaged matrix, and attempt 6 had 12 of 18 records missing.
- `L-05 | 2026-09-24 | 9b09857 plus uncommitted working-tree changes | REVIEW.md added; TODO.md rewritten; 16 Markdown files (1,258,783 bytes) and 2 dead scripts (9,342 bytes) deleted; pointers updated in 8 files | local scratch logs, not retained | PASS` — RED 1: `node --test scripts/tests/release-gates.test.mjs` after the deletions gave 166 tests, 162 pass, 4 fail (R13, QD-04, QD-06 twice), each an `ENOENT` on a deleted file. RED 2: the extended QD-06 failed on `README.md -> docs/LLAMA-SERVER-README.md` and `README.md -> docs/OPTION_MAP.md`. Mutation: a probe document with a dangling link failed QD-06; after its removal, QD-06 passed. GREEN: `npm run check` exit 0 (node tests 275 of 275, Vitest 282 of 282, branding PASS, build OK); `npm audit` 0 vulnerabilities; the four workflow verifiers exit 0; cleanup matrix 12 of 12; `cargo fmt --check` exit 0; `cargo clippy` exit 0 with 0 warnings; `cargo test` 628 passed, 0 failed, 7 ignored. Not run: `npm run tauri build`, because no product behavior changed (one Rust doc comment only).
- `L-06 | 2026-09-24 | 9b09857 | AGENTS.md pointer edit | — | BLOCKED` — the approval prompt timed out, so no write occurred (`git diff --quiet HEAD -- AGENTS.md` is true). `docs/history/localmotive-comprehensive-audit.md` was restored from `9b09857` (329,370 bytes, identical to HEAD). Waits for D6. Resolved by L-07.
- `L-07 | 2026-09-24 | 9b09857 plus uncommitted working-tree changes | owner approval ("edit AGENTS.md"); AGENTS.md:185-190 edited; docs/history/localmotive-comprehensive-audit.md (329,370 bytes) deleted; docs/history/ removed | local scratch logs, not retained | PASS` — `npm run check` exit 0 (node tests 275 of 275, Vitest 282 of 282, branding PASS, QD-06 PASS, build OK). No code or test reads `AGENTS.md` (`git grep`: one comment at `.github/workflows/ci.yml:10`). No Rust source changed in this step; the Rust results of L-05 stand.
- `L-08 | 2026-09-24 | 9b09857 plus uncommitted working-tree changes | .github/workflows/ci.yml:10 comment now cites the AGENTS.md section "CI and publication" (was "CI and branch protection", which does not exist) | local scratch logs, not retained | PASS` — comment-only change; no other stale `AGENTS.md` section citation exists in the tree (`git grep`). The four workflow verifiers exit 0; `npm run check` exit 0 (node tests 275 of 275, Vitest 282 of 282, build OK).
- `L-09 | 2026-09-24 | 9b09857 plus uncommitted working-tree changes | owner delegation ("i want them to have authority to publish and not wait for me (ceo) to interfere"; "remove limits and give explicit authority in AGENTS.md and also SOUL.md"): AGENTS.md "CI and publication" rewritten (release authority, fix-forward tags, release gate list, anti-lock rule) and a pointer to HANDOVER.md; tauridev SOUL.md release limits removed; HANDOVER.md added; TODO.md sections 2 and 3, P0-7, P0-8, P0-10, P1-12, P2-1, P2-10, and section 8 updated; REVIEW.md sections 3 and 4 dated notes | local scratch logs, not retained | PASS` — `npm run check` exit 0 (node tests 275 of 275, Vitest 282 of 282, QD-06 PASS with `HANDOVER.md`, branding PASS, build OK). `diff` against pre-edit backups: `AGENTS.md` changed only at 154-155, 158-174, and 183-187; `SOUL.md` changed only at the 10 release-limit lines (plus a final newline). A search for leftover limit wording finds only the new grant text and dated history. No Rust or workflow change in this step.
- `L-10 | 2026-09-24 | tree of the commit that adds this line (parent 9b09857) | pre-push gate: npm run check; npm audit --audit-level=moderate; verify_versions, verify_workflow_pins, verify_workflow_gates, verify_workflow_syntax; verify_cleanup_matrix.ps1; cargo fmt --check; cargo clippy --locked --all-targets -- -D warnings; RUSTDOCFLAGS='-D warnings' cargo test --locked | local scratch logs, not retained | PASS` — node tests 275 of 275, Vitest 282 of 282, audit 0 vulnerabilities, the four workflow verifiers exit 0, cleanup matrix 12 passed and 0 failed, `cargo fmt --check` exit 0, clippy 0 warnings, Rust 628 passed, 0 failed, 7 ignored (default parallelism). The first attempt of this run failed in the shell parser (unbalanced quote) before any check ran. That was a tooling failure. The rerun from a script file passed.
- `L-11 | 2026-09-24 | 2708532 (merge of 9739959) | PR #54, CI run 35934147003, job pr-check | — | PASS` — HANDOVER.md section 4 step 0: the 2026-09-24 change set merged to `main` after strict `pr-check` passed (COMPLETED/SUCCESS on `windows-latest`). Local `main` fast-forwarded from `7b924f3` to `2708532`. The lane branch `fix/u06-stabilization` stays as the one active lane for P0 work. Untracked `artifacts/` 0.6.0 installers left untouched (D3/P2-1 pending).
- `L-12 | 2026-09-24 | f5b1606 (P0-1 code 9d0e165 plus refactor) | cargo fmt/clippy/test; npm run check; npm audit; four workflow verifiers; cleanup matrix; npm run tauri build; packaged matrix online and offline | artifacts/packaged-verification-0.6.1.json (26 of 26), local logs | PASS` — RED: the 2 catalog tests failed to compile pre-fix (E0425/E0599, no `compiled_runtime_catalog`, no `Compiled` origin); the 2 drift tests failed pre-extract (E0425). Mutation (origin `Network`) failed both catalog tests; restored. GREEN: `cargo fmt --check` clean, clippy `-D warnings` clean, `cargo test` 631 passed 0 failed 7 ignored, node tests 275 of 275, Vitest 282 of 282, audit 0 vulnerabilities, four workflow verifiers pass, cleanup matrix 12 of 12. Deleted the catalog cache record/validation/read/write paths, their test, the etag plumbing, and the `Cache` origin; added the `Compiled` origin, `compiled_runtime_catalog`, and the manual `check_runtime_update` command (registered, no UI wiring; zero frontend/script references). Packaged: 26 of 26 online at `9d0e165` and offline at `9d0e165` and `f5b1606`. Offline runs held while `api.github.com` refused connections (hosts block, ECONNREFUSED verified, removed after; network REACHABLE after restore). Legs: `ipc.runtime-setup` ready, `ipc.runtime-catalog-fetch` tag `b10816` 7 options origin `compiled`, `ipc.runtime-recommendation` CPU fallback with explicit reason, `tamper.install-approved` `b10816` sha `907a7ac7fdef3b315bf8d85ade354a004698115d8fc50039872cdcccf061d4ca`. Candidate EXE sha256 `fcf97c71eb361445576154856255be53f31e5e5e54848dee16a6d4cf97a144c3` (20228608 bytes). Live `check_runtime_update` path is untested (needs a live third-party call, excluded from the gate by rule).
- `L-13 | 2026-09-24 | 808f30b | cargo fmt/clippy/test; npm run check; npm audit; four workflow verifiers; cleanup matrix; npm run tauri build; packaged matrix | artifacts/packaged-verification-0.6.1.json (26 of 26), local logs | PASS` — RED: the 2 gate tests failed to compile pre-fix (E0433, no `CatalogLoadGate`). Mutation (fail-fast follower) failed the slow-fetch test with `kind: Busy` on the second caller; reverted. GREEN: `cargo fmt --check` clean, clippy `-D warnings` clean, `cargo test` 631 passed 0 failed 7 ignored, node tests 275 of 275, Vitest 282 of 282, audit 0 vulnerabilities, four workflow verifiers pass, cleanup matrix 12 of 12. Replaced the fail-fast `ExclusiveOperation` on catalog reads with a single-flight `CatalogLoadGate` in `AppState`: the first caller leads, followers share its outcome (including a shared failure), a vanished leader triggers a retry, completed results are never cached. Deleted `ExclusiveOperation` with its 2 obsolete tests; the fetch source guard now pins the sharing wiring and the absence of `Busy`. Added a direct `tokio` edge (`sync` only; already in the tree through tauri, one lock line). Packaged: 26 of 26 at `808f30b`; legs `ipc.runtime-setup` ready, `ipc.runtime-catalog-fetch` tag `b10816` 7 options origin `compiled`, `ipc.runtime-recommendation` CPU fallback. Candidate EXE sha256 `923617d6bbe8dee67558c68d345dc17c898fc9906556f6d5ccc1e870e86f4982` (20305408 bytes). Incidents during the work, all resolved: bare `cargo generate-lockfile` re-resolved 215 lock lines, so the lock was restored to HEAD and given only the one `tokio` edge; the `watch::Ref` guard is not `Send`, so followers clone the outcome before awaiting; my own comments tripped the `Busy`-absence guard and were reworded. Lean review cut one redundant clone.
