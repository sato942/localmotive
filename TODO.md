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
- [x] **D3 — Remove `artifacts/` from the tree (TEAM).** Decided
  2026-09-24: plain removal commit plus `/artifacts/` ignore rule, no history
  rewrite. The 5,494 tracked files leave the tree; the bytes stay in history.
- [x] **D4 — Decide the `v0.4.0` corrective note (TEAM).** Decided
  2026-09-24: deleted, not published. The inline 0.4.1 changelog correction
  stands as the published record.
- [x] **D5 — Decide superseded evidence (TEAM).** Decided 2026-09-24:
  exception granted; `release-evidence/0.6.0/history/` deleted. Accepted
  published evidence stays frozen.
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

- [x] **P0-3 — Keep the error kind in harness evidence (LAB-01).** L-14.
  - Behavior: `scripts/verify_041.mjs` records `catalogError.kind` and the
    message for each failed runtime check. No reason field contains
    `[object Object]`.
  - RED test: a script test gives the harness a structured error
    `{ kind, message }` and expects both values in the record.

- [x] **P0-4 — Fix the catalog query wire keys (FE-02).** L-15.
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

- [x] **P0-5 — Stage release files outside the checkout (REL-05).** L-16.
  - Behavior: `release.yml` writes candidate and diagnostics files to a
    directory under `$RUNNER_TEMP`, not to `artifacts/`. Each uploaded
    artifact holds only files from the current run.
  - RED test: a workflow test rejects a staging path inside the workspace.

- [x] **P0-6 — Name the failed step correctly (REL-09).** L-17.
  - Behavior: when packaging passes and verification fails, the summary names
    the verification step. It does not say "Packaging failed".

- [x] **P0-7 — Cut the release gate (D2; REL-03, REL-04, REL-07,
  REL-12, REL-13).** L-18.
  - Cut `release.yml` to the release gate in `AGENTS.md`, "CI and
    publication".
  - Remove the lab, witness, and fault steps from `release.yml`. The
    manifest step stays, reduced to the gate records (next bullet).
    Move the lab legs that stay useful to `hardware-qualify.yml` as jobs that
    do not block.
  - Remove each dependency on `.hermes-0.6/`. A clean clone must hold every
    input of the gate.
  - Reduce `build_qualification_manifest.mjs` and
    `verify_qualification_manifest.mjs` to the records of `REVIEW.md`
    section 4, or delete them with their tests.
  - Set `cancel-in-progress: false` for the release concurrency group.

- [x] **P0-8 — Keep promotion aligned with the producer (D1 = Option A;
  REL-01, REL-02, REL-08).** L-19.
  - Keep the push-event rule (`release-promote.yml:86-96`). The new tag gets
    a push-event run, so promotion needs no change for option A.
  - Implement the extra-file check that `verify_release_promotion.mjs:8` and
    `:43` promise, or delete those comments.

- [x] **P0-9 — Correct the public text (DOC-04, DOC-05).** L-20.
  - `README.md` names `v0.5.0` as the current release. It does not describe
    0.6.0 or 0.6.1 as shipped.
  - `CHANGELOG.md` marks 0.6.0 and 0.6.1 as not published.
  - `docs/SUPPORT-MATRIX.md`, `docs/EVIDENCE-MATRIX.md`, and `README.md`
    make the same support claims.

- [x] **P0-10 — Release 0.6.5 (D1).** L-21.
  - Preconditions: P0-1 to P0-9 PASS. Followed `HANDOVER.md` section 5.
  - Tags `v0.6.2`, `v0.6.3`, `v0.6.4` burned (workflow file, name mismatch,
    VERSION expansion); fix-forward per the tag rule, `v0.6.5` published.
  - Acceptance: version fields read 0.6.5; `release.yml` run 36030147058
    SUCCESS on peel `4ec73f6`; promotion dry run 36034567908 and live run
    36034701536 SUCCESS; six assets published and read back byte-identical;
    ledger line L-21 below.
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

- [x] **P1-1 — CORE-04:** Rust rejects a launch profile whose model is not the
  first shard of its set. Done 2026-09-25: `artifact::first_shard_for_model`
  plus `reject_non_first_shard_model` in launch-path validation; shard 2 of 3
  fails naming shard 1. Non-shard names pass through (no extension rule).
- [x] **P1-2 — MT-10:** The cloud brief sends only the fields that the
  disclosure lists. Done 2026-09-25: `apply_disclosure` strips `adapterId`,
  `compatibilityId`, `physicalId` from every adapter in both modes; names,
  VRAM and driver versions stay. RED plus mutation-proven test on the wire.
- [x] **P1-3 — RT-02:** Install keeps the rollback copy until the final
  verification passes. Done 2026-09-25: `replace_verified_runtime_directory`
  returns the backup; `finalize_runtime_replacement` removes it on verify
  success and restores it on verify failure. RED plus mutation-proven test.
- [x] **P1-4 — LAB-04:** Delete `scripts/g05_vitems_c.mjs`, or make it stop
  only the processes that it started, by PID. Done 2026-09-25: deleted the
  dead name-killing script (no callers; REVIEW marked it DELETE). Repo sweep:
  every surviving stop is PID-scoped. Guard test RED plus mutation-proven.
- [x] **P1-5 — REL-10:** Where a workflow stops the app, stop the whole
  process tree that the step started. Done 2026-09-25: `ci.yml` smoke stop
  plus lab watchdog and `Stop-LabApp` use `taskkill /T` on the owned PID
  with immediate exit-code checks. RED plus mutation-proven workflow test.
- [x] **P1-6 — PROC-01:** Put the child process in the job object before it
  runs: create it suspended, assign it, then resume it. Done 2026-09-25:
  `spawn_contained` sets `CREATE_SUSPENDED`, assigns, then releases via new
  `containment::resume_process` (ToolHelp; fail-closed). Behavioral test
  plus ordering guard, RED and mutation-proven.
- [x] **P1-7 — FE-01:** Clear each credential draft on every exit path,
  including a failed save and an unmount. Done 2026-09-25: both saves clear
  the failed attempt (newer typing survives); both screens clear on unmount.
  4 App-level tests, RED plus mutation-proven. Vitest 288/288 (14 files).
- [x] **P1-8 — DL-02:** Use checked arithmetic for GGUF split metadata. RED
  test with a split number of `u64::MAX`. Done 2026-09-25:
  `split_metadata_verdict` uses `checked_add`; overflow is Mismatch with the
  raw value kept. RED (debug panic) plus wrap-mutant proven. The release
  `overflow-checks` profile flag stays untouched as out of scope.
- [x] **P1-9 — CORE-01, CORE-06:** Accept the verification overrides only with
  the isolated root. Show a verification-mode banner while an override is
  active. Document the WebView2 remote-debugging variable in `SECURITY.md`.
  Done 2026-09-25: root-binding already held (existing tests); added the
  read-only `verification_mode` command plus banner (`warning-band`,
  role=status) and the SECURITY.md section. Rust 638/638, Vitest 290/290.
- [x] **P1-10 — RT-01, RT-03:** Close the same-user races with handle-based
  opens and the existing execution lease. (2026-09-25: RT-01 — new
  `download::write_trusted_record` writes `runtime.json` through the retained
  staging capability: path-level refusal, open without truncate, handle check
  (one link, no reparse, final-path parent), truncate+write through the
  verified handle; staging guard now drops after `finalize`, not before
  publication; deleted `fs::write` path. RT-03 — probe guards return
  `Option<ManagedExecutionLease>` and `run_runtime_probe_with` holds it
  across spawn+output; `inspect`+`health` use `authorize_managed_execution_lease`;
  deleted the orphaned `()` guards and migrated the RT-02 authorization test
  to the lease. RED: hard-link victim clobbered under old `fs::write`;
  compile-RED on new guard type. Mutation: path-check skip still refused
  (handle layer independent; also caught truncate-before-verify flaw, fixed to
  verify-then-set_len); lease-drop mutant failed the holding test. Rust
  642/0, clippy clean, node 290/290.)
- [x] **P1-11 — Check the 31 unverified High findings in `REVIEW.md`.**
  Done 2026-09-25: all 31 checked against current code (tracker + code +
  tests). REVIEW.md statuses updated to `VERIFIED 2026-09-25 (P1-11)` (24)
  or `REFUTED 2026-09-25 (P1-11)` (7). New P1 items below for the verified
  defects.
  - REFUTED (7): REL-11 (release.yml is 309 lines, no inline lab JS — cut by
    P0-7); LAB-06 (shared `catalog-schema-cases.json` conformance both
    sides + DC-10 envelope checks); DOC-05 (README/matrices agree on 0.6.5;
    fixed one stale "no 0.6.x release" sentence in SUPPORT-MATRIX.md);
    DOC-07 (single 43-checkbox TODO.md; cited history files not in tree);
    RT-07 (`ScanLimits` depth 12/entries 100k + `safe_directory`
    no-follow discovery); CORE-03 (tuning holds slot + RAII reservation to
    completion; cancel sets a flag without releasing); MT-07 (tuning uses
    the cancellable MT-04 path, not the legacy worker).
  - VERIFIED (24): LAB-03, LAB-05, RT-04, RT-10, CORE-05, PROC-02, PROC-03,
    PROC-04, PROC-05, PROC-06, PROC-07, PROC-08, DL-01, DL-03, DL-04, MT-01,
    MT-03, MT-05, MT-06, MT-08, MT-09 + FE-05 (one defect), FE-04, FE-06.
    Evidence per item below.
- [ ] **P1-12 — Decouple the release jobs from the hardware host.** Today
  `release.yml` and `release-promote.yml` need all six labels of the one
  runner on the owner's PC.
  - Behavior: the release jobs use `[self-hosted, Windows, X64,
    localmotive-release]`. `hardware-qualify.yml` keeps the hardware labels.
    At least one runner that the team controls carries `localmotive-release`
    (A2).
  - Evidence: one green `release.yml` run on that runner.
- [x] **P1-13 — LAB-03: unify the three CDP clients.** Done 2026-09-25:
  `attach()` in `scripts/lib/cdp_client.mjs` gained `pageFilter` +
  `trustedInput` options; `verify_041.mjs` (deleted 100-line `CdpClient` +
  socket code, keeps its Localmotive page filter + trusted clicks),
  `drive_console_check.mjs`, and `verify_installer_payloads.mjs` (preserves its
  tolerated no-page/probe-error observations) attach through the lib. Only
  `new WebSocket(` site left is the lib. RED: single-owner test failed with 4
  sites; mutant re-add failed it again. `npm run check` 290/290, audit 0,
  verifiers pass.
- [ ] **P1-14 — LAB-05: delete or wire the 15 dead g05 drivers.** `g05_cancellation`,
  `g05_churn_repro`, `g05_dc01`, `g05_hardlink_drive`, `g05_health`,
  `g05_launch_benchmark`, `g05_mt01d`, `g05_partial_resume`, `g05_partial_retention`,
  `g05_run_cancel`, `g05_state`, `g05_stop_supervision`, `g05_tamper_dll`, `g05_v2_cancel`,
  `g05_vitems_d` have zero references (same family as P1-4). Delete them or reference
  them from a workflow/test; keep the P1-4 no-name-kill sweep green.
- [ ] **P1-15 — RT-04: pin the directory inventory for the lease lifetime.** Lease
  acquisition checks `actual == expected` files, but a file planted after acquisition
  is not detected while the lease is held. Re-check the inventory (or pin the
  directory handle) before each execution, following the P1-10 handle pattern.
- [ ] **P1-16 — RT-10: validate the runtime root before creating it.**
  `install_runtime` runs `fs::create_dir_all(&root)` before
  `validate_no_reparse_ancestors` + the symlink check. Move validation first with a
  RED test that a hostile link ancestor is refused with no directory created.
- [ ] **P1-17 — CORE-05: bound IPC vectors at the command boundary.**
  `inspect_model_artifact` (`companions: Vec<String>`), `PreflightRequest`
  (`selected_adapter_ids`, `manual_overrides`) and the other cited payloads take
  unbounded vectors before allocation and hashing work. Add length caps with RED
  tests that oversized payloads are refused before any file work.
- [ ] **P1-18 — PROC-02: join pipe readers with a deadline after failed cleanup.**
  Every `output_with_timeout_and_cancel` cleanup path joins the reader threads
  unboundedly, including the `!terminate_and_wait` branch where the child is still
  alive and the pipes never close. Bound the joins (or drop the pipes first) with a
  RED test using an unkillable child.
- [ ] **P1-19 — PROC-03: surface unresolved termination distinctly.** Timeout/cancel
  paths ignore `terminate_and_wait`'s boolean and return `Timeout`/`Cancelled` while
  the tree may still run; no `ProcessFailureKind` names it. Add an `Unresolved`
  kind (or equivalent) with a RED test that a surviving child is reported as such.
- [ ] **P1-20 — PROC-04: clear the starting slot when `local_client` fails.**
  `start_server_worker` uses `&local_client(&profile)?` between spawn and the health
  wait: the `?` returns without `clear_starting` and without terminating the child.
  Restructure with a RED test that this path clears the slot and reaps the child.
- [ ] **P1-21 — PROC-05: stop following redirects on the health readiness client.**
  `health.rs:1082` builds a reqwest client with no redirect policy (default follows),
  while `local_client.rs:516` sets `Policy::none` (R15). Set `Policy::none` + port the
  R15 redirect test.
- [ ] **P1-22 — PROC-06: track and join the nested completion worker.**
  `completion_request_supervised` (and the 715-747 sibling) spawns a detached thread
  and relies on `terminate()` to unblock it. Join the worker with a deadline and
  report an unresolved outcome instead of abandoning it.
- [ ] **P1-23 — PROC-07: refuse links in transport-file reads.** `read_bounded_file`
  (`local_client.rs:854`) follows symlinks and only checks `is_file()` — it reads SSL
  keys/certs and API key files. Refuse reparse points/symlinks (mirror
  `require_regular_non_reparse_file`) with a RED symlink test.
- [ ] **P1-24 — PROC-08: stop following replaced log paths.** `LogWriter::second_writer`
  re-opens `self.path` and `write_failure_evidence` uses `fs::write` through
  replaceable paths. Open once and share the handle (or verify-then-write through a
  retained directory handle per P1-10).
- [ ] **P1-25 — DL-01: revalidate override rows at read time.** `user_override_file`
  trusts SQLite content validated only at write time; a direct DB edit can swap the
  digest and authorize malicious bytes. Add read-time integrity (e.g., a MAC with a
  Credential Manager key) with a RED tampered-row test.
- [ ] **P1-26 — DL-03: verify the handle, not the path, on reads.** `validate_regular_non_reparse_file`
  then `File::open` (e.g., `gguf.rs:649-657`) leaves a plant-between-check-and-open
  window. Open first, then verify the handle (one link, no reparse, final-path
  parent) following the P1-10 `write_trusted_record` pattern.
- [ ] **P1-27 — DL-04: bound the OAuth key-exchange response.** `exchange_code_for_key`
  (`cloud.rs:596-616`) buffers `response.text()` unbounded before parsing. Read
  bounded (mirror `read_bounded`) with a RED oversized-body test.
- [ ] **P1-28 — MT-01: stop presenting a same-prompt repeat as confirmation.**
  Final verification re-measures on the fixed harness prompt and the UI says
  "confirmed". Relabel as a re-measurement with the same-prompt limit stated, or add
  a held-out prompt.
- [ ] **P1-29 — MT-03: persist warmup-only failures as Failed manifests.**
  Warmup failure returns with empty `observations`, so `validate_attempt_consistency`
  rejects the manifest and nothing is saved. Persist the partial run (Failed class)
  with a RED warmup-failure test.
- [ ] **P1-30 — MT-05: restore the server after a cold v2 benchmark.**
  `benchmark_v2` takes the user's server (`slot.take()`), terminates it for cold
  mode, and never relaunches it. Relaunch the same profile after the run (or refuse
  cold while a server runs) with a RED state test.
- [ ] **P1-31 — MT-06: bound the worker drain.** `drain_owned_workers` loops
  `while !wait_for_worker_drain(1s) {}` forever; a stuck worker holds the benchmark
  slot and the operations reservation permanently. Bound the loop and report an
  unresolved outcome.
- [ ] **P1-32 — MT-08: make replay respect unknown execution identity.**
  The compatibility key embeds `"unknown"` for unobserved drivers, so two machines
  with unknown drivers produce equal keys and replay proceeds. Refuse replay while
  `unknown_identities` is non-empty (or key the unknown-ness distinctly).
- [ ] **P1-33 — MT-09/FE-05: remove the legacy benchmark system.** `benchmark_server`
  + `BenchmarkScreen` duplicate `benchmark_v2` + `V03EvidencePanel`, and both commands
  are registered (`lib.rs:2293-2294`). Migrate remaining callers, then delete the
  legacy command, screen, and service path.
- [ ] **P1-34 — FE-04: split `App.tsx`.** One 1858-line component owns state,
  persistence, async coordination, and shell rendering. Extract the first coherent
  piece (e.g., persistence or one screen's coordination) with component tests kept
  green; record the next split in the item.
- [ ] **P1-35 — FE-06: route `cancel_scan` through props.** `InventoryScreen`
  calls `invoke("cancel_scan")` directly although its contract keeps acquisition in
  `App.tsx`. Pass a callback prop instead; keep the presentation-boundary test green.

---

## 6. P2 — Simplification after the release

- [x] **P2-1 — Remove `artifacts/` (D3).** Done with D3: `/artifacts/`
  ignore rule added, the four `.gitattributes` LF rules for `artifacts/`
  removed. The one test that read tracked `artifacts/*.json` fixtures now
  reads the byte-identical `release-evidence/0.6.1/` copies; all other
  `artifacts/` references are runtime workspace paths or fixture-temp labels.
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
- `L-14 | 2026-09-24 | 915689d | npm run check; npm audit; four workflow verifiers; cleanup matrix; packaged matrix | artifacts/packaged-verification-0.6.1.json (27 of 27), local logs | PASS` — RED: the new `scripts/tests/ipc_errors.test.mjs` failed on `ERR_MODULE_NOT_FOUND` before `scripts/lib/ipc_errors.mjs` existed. Mutation (lossy serializer restored) failed the tests; reverted. GREEN: node tests 281 of 281, Vitest inside `npm run check` exit 0, audit 0 vulnerabilities, four workflow verifiers pass, cleanup matrix 12 of 12. Both harness IPC paths (`invoke` and `rejectedInvoke`) now ship the raw rejection across CDP as JSON text and normalize it with one shared `serializeIpcError`, so a failed call throws `command: [kind] detail` and the record keeps kind plus detail; plain-string and missing rejections degrade to `unknown` kind. The `ipc.runtime-setup` terminal-error evidence now carries `error_kind`/`error_detail` from `catalogError`. Packaged: 27 of 27 at `915689d` (binary unchanged from `808f30b`, EXE sha `923617d6…`); legs `ipc.reject-unknown-adapter` kind `invalidResponse`, new `ipc.fetch-error-kind` detail `fetch_runtime_catalog: [invalidResponse] Selected adapter is not in the current hardware snapshot`, catalog-fetch `b10816` 7 options origin `compiled`. Incidents during the work, all resolved: the `rejectedInvoke` edit first dropped the `result` binding (returned `evaluate` directly before) — restored; backticks inside the in-page comment terminated the template literal — reworded; my thrown error first dropped the command name — restored as a `command:` prefix. Rust checks (fmt, clippy, cargo test 631/7) are carried from L-13: no Rust file changed in this step. Lean review added the command prefix and found nothing else to cut. Out of scope, left for their owners: the same `String(error)` in-page pattern still exists in `scripts/g05_rt06_all_backends.mjs` and the one-off drivers under `scripts/`, which do not write check records.
- `L-15 | 2026-09-24 | 34613c4 | cargo fmt/clippy/test; npm run check; npm audit; four workflow verifiers; cleanup matrix; npm run tauri build; packaged matrix | artifacts/packaged-verification-0.6.1.json (28 of 28), local logs | PASS` — RED: the Rust filter test failed on the snake_case payload both rows came back; the Vitest builder tests failed with `buildCatalogQuery is not a function`; both contract tests failed on the missing `catalogQuery` fixture entry. Mutation (snake `pipeline_tag` restored in the builder) failed the wire-shape Vitest; reverted. GREEN: `cargo fmt --check` clean, clippy `-D warnings` clean, `cargo test` 632 passed 0 failed 7 ignored, node tests 281 of 281, Vitest 284 of 284, audit 0 vulnerabilities, four workflow verifiers pass, cleanup matrix 12 of 12. The frontend now sends `pipelineTag`, `fitPerMille`, `budgetBytes` through a tested `buildCatalogQuery` in `src/model.ts`, used by `App.tsx`; the `CatalogQuery` TS type matches the Rust `rename_all = "camelCase"` shape, pinned by the shared `catalogQuery` fixture entry on both sides. Packaged: 28 of 28 at `34613c4`; new leg `ipc.catalog-query-wire-keys` reads `{unfiltered: 2, pipeline: 1, fit: 1}` from the real backend. Incidents during the work, all resolved: one pre-existing source guard (`release-gates.test.mjs`, rich-filters case) pinned the old snake keys and failed the suite — updated to the camelCase keys with a P0-4 note, since the approved behavior changed the contract (intent preserved: the UI sends fit inputs); the new check first repeated three invoke blocks — shrunk to one `applyFilter` closure (net -8). Lean review found nothing else to cut. Out of scope, untouched: `CatalogModel.pipeline_tag` stays snake on purpose (signed artifact file keys, alias kept), and the `pipeline_tags` facet mocks in component tests are test doubles, not wire assertions.
- `L-16 | 2026-09-24 | 86f2495 | npm run check; npm audit; four workflow verifiers; cleanup matrix; collect-loop dry run | local logs | PASS` — RED: the new workflow test failed on the in-workspace staging paths. Mutation: none needed beyond RED (the old paths were the defect). GREEN: node tests 283 of 283, Vitest 284 of 284, audit 0 vulnerabilities, four workflow verifiers pass, cleanup matrix 12 of 12. `release.yml` now stages candidates under `${{ runner.temp }}/localmotive-release` (`STAGE_DIR`): the stage, inventory, installer-payload, sandbox/fault/lab candidate reads, both uploads (`${{ env.STAGE_DIR }}` globs), and the missing-evidence report all reference the stage. A new collect step (success or failure) copies only the current run's five matrix JSON records into the stage, so each uploaded artifact holds only current-run files; the collect loop was dry-run locally (5 of 5 copied). Four pins in `.github/workflow-gates.json` moved to the staged commands. Two pre-existing guards pinned the old paths and were updated with P0-5 notes (gates pin text; GH-03 lifecycle CandidateDir). Incidents during the work, all resolved: the gates verifier failed first on the four stale pins — updated; a first `STAGE_DIR` probe through the promotion validator showed its `<qualified>/artifacts` + repository-relative-path contract cannot consume out-of-checkout files, so that seam was reverted untouched and the manifest/promotion input rewiring stays P0-7's rework (those steps still name workspace paths; no tag runs between P0-5 and P0-7); Windows Python wrote CRLF into `workflow-gates.json` — converted back to LF (diff is 4 lines). Rust checks are carried (no Rust file changed in this step). Lean review: lean already, ship. No packaged matrix: workflow-only change, no Rust or frontend bytes; the binary is unchanged from `34613c4`.
- `L-17 | 2026-09-24 | 86f2495 | new checker scenario in npm test | local logs | PASS` — RED: the new verification-stage scenario failed on the old script ("Packaging failed" for a missing verification record). Mutation (old banner restored) failed only that scenario; reverted. GREEN within the same `npm test` run as L-16. `check_package_outputs.ps1` takes stage-relative paths and reports three outcomes: missing packaging outputs names packaging, a missing verification record names the packaged-verification step, a complete stage names the downstream failure. The F9-04 test was updated for stage-relative paths (approved behavior change) with its intent intact.
- `L-18 | 2026-09-24 | ad2f5e5 | npm run check; npm audit; four workflow verifiers; cleanup matrix; stage-manifest dry run; PSParser on transplanted script | local logs | PASS` — RED: the new gate-shape test failed on the old workflow (lab step name, fault step, `.hermes-0.6`, `cancel-in-progress: true` all present). Mutation (restored `witness_timeout` in REQUIRED_RECORDS) failed 3 tests; reverted. GREEN: node tests 283 of 283, Vitest 284 of 284, audit 0 vulnerabilities, four workflow verifiers pass, cleanup matrix 12 of 12. `release.yml` verify job is now checks, SBOM, build, packaged verification, collect, stage, inventory, payload identity, one Sandbox lifecycle leg (upgrade v0.5.0 mirror), mirror, manifest, promotion-check, two uploads, missing-evidence report; `cancel-in-progress: false`. The lab legs (mt06/rt06/dc04/rt04/a11y) moved to a new non-blocking `lab-legs` job in `hardware-qualify.yml` with in-run TLS self-generation and env-seamed evidence dirs (PSParser clean; YAML syntax verifier clean). The manifest binds 3 records (packaged verification, lifecycle preservation v0.5.0, installer payload identity) with stage-relative paths via `--record-root`; witness/negative controls stay as history, extras tolerated. Dry run: built a manifest from committed 0.6.1 records into a temp stage with `--record-root` — 3 records bound, inventory recorded `artifacts/candidate-inventory-0.6.1.json`; verification against the stage refused only the genuinely stale P0-4 matrix record (source/size/digest contradictions), proving the stage-relative mechanics. Incidents, all resolved: shell-mangled `\u` in an inline python transform (moved to scratch script files); node/MSYS `/tmp` vs `C:\tmp` split-brain in the dry run (used the native scratch path); a nested attestations copy (re-copied); an over-broad assert on my own comment (narrowed); 15 test fallout items updated with P0-7 notes (witness-outcome test deleted — contract vacated; `cancel-in-progress` now asserts false — a queued release waits instead of cancelling mid-flight; refusal tests re-pointed at the surviving lifecycle record). Rust checks are carried (no Rust file changed in this step). Remaining risk: the moved lab legs never ran with self-generated TLS material — non-blocking placement bounds this. No packaged matrix: workflow-only change, no Rust or frontend bytes; the binary is unchanged from `34613c4`.
- `L-19 | 2026-09-24 | 48093ad | npm run check; npm audit; four workflow verifiers; cleanup matrix | local logs | PASS` — RED: the new inflated-set test failed on the old validator (stray installer passed). Mutation (inverted allowlist test) failed 5 tests; reverted. GREEN: node tests 285 of 285, Vitest 284 of 284, audit 0 vulnerabilities, four workflow verifiers pass, cleanup matrix 12 of 12. `verify_release_promotion.mjs` now refuses any file or subdirectory in `artifacts/` outside the 6 promised assets plus 5 gate-produced sidecars (`allowedSidecarNames`: promotion-check report, packaged-verification-catalog, three catalog-matrix records), implementing the header promise at lines 8 and 42-43 (the "or delete those comments" alternative was not taken). The push-event rule (`release-promote.yml:86-96`) is untouched, and no workflow file changed: the validator CLI is unchanged, and the P0-7 stage layout (`artifacts/`, sbom, `release-evidence/`) matches the layout the promote workflow downloads, so producer and promotion stay aligned. Lean review: lean already, ship. Rust checks are carried (no Rust file changed in this step). No packaged matrix: validator-only change, no Rust or frontend bytes; the binary is unchanged from `34613c4`.
- `L-20 | 2026-09-24 | e1ed0e8 | npm run check; npm audit; four workflow verifiers; cleanup matrix | local logs | PASS` — RED: the new public-docs consistency test failed on the old text (no latest-published statement, no Unreleased section). Mutation (restored "0.6.0 ships" line) failed the test; reverted. GREEN: node tests 286 of 286, Vitest 284 of 284, audit 0 vulnerabilities, four workflow verifiers pass, cleanup matrix 12 of 12. README names v0.5.0 the latest published release, calls the checkout 0.6.1 development source, scopes the support bullet to the unpublished 0.6.1 candidate with retained `release-evidence/0.6.1/` results, and replaces both "0.6.0 ships unsigned" lines with the standing unsigned policy. CHANGELOG gains `## Unreleased` (0.6.1 tagged but unpublished; v0.5.0 latest published) and marks both 0.6 sections "Unpublished candidate — not released" with `##` headings byte-identical so the promote workflow's exact-match changelog extraction keeps working. SUPPORT-MATRIX moves the lifecycle row to the 0.6.1 candidate (0.6.0 retained as history), scopes the CUDA figures as historical 0.6-development lab observations rather than version-bound release evidence (their exact provenance is not in the retained 0.6.1 mt06 record, which covers cycle mechanics). EVIDENCE-MATRIX supersedes 0.6.0 and adds the 0.6.1 candidate row; it stays the index for the README support statement. Lean sweep: no remaining "0.6.x ships" language (leftover "ships" uses are published-0.5.0 facts, the catalog file, and the unsigned policy). Rust checks are carried (no Rust file changed in this step). No packaged matrix: docs-only change; the binary is unchanged from `34613c4`.
- `L-21 | 2026-09-24 | 4ec73f6 (peel v0.6.5) + f01bc49 (post-publish docs) | release 0.6.5 published via release-promote.yml | CI records, retained | PASS` — Fix-forward through three burned tags per the no-tag-move rule: `v0.6.2` burned (runner context in job-level env invalidated the workflow file, no run existed; fixed via GITHUB_ENV setter step, RED-tested); `v0.6.3` burned (packaged-verification record named raw `localmotive.exe` while the inventory bound the staged portable; fixed by staging before the matrix and testing the staged file, RED-tested); `v0.6.4` burned (bare `${VERSION}` in the pwsh staged path expanded to nothing; fixed with `$env:VERSION`, RED-tested plus a mutation that the hardened assignment-line test catches). Also fixed a transplanted pre-existing `[int]` run-id overflow in the lab port formula (`[int64]`, proven port 11150). Release evidence: PR #58 merged after pr-check SUCCESS; `Release verify` run 36030147058 SUCCESS (resolve, rust-audit, verify — packaged matrix 28 of 28 PASS incl. candidate.artifact staged binding, one Sandbox clean-account leg clean install + upgrade from v0.5.0 with preservation + uninstall); promotion dry run 36034567908 gate SUCCESS with publish skipped; live run 36034701536 gate + publish SUCCESS with public readback byte-identical. Published 2026-09-24T17:31:01Z, not draft, not prerelease: `Localmotive_0.6.5_x64-portable.exe` sha 29e3b6e4…, `Localmotive_0.6.5_x64-setup.exe` sha 3f5557d6…, `Localmotive_0.6.5_x64.msi` sha 974e502b…, plus candidate-inventory, packaged-verification, SHA256SUMS. Incidents: dc02 loopback failure on the 0.6.2-era main CI run (environmental load flake; 5 of 5 isolated plus two full local suites green, and it passed in the 0.6.5 gate run — watch item). Post-publish docs (PR #59, pr-check SUCCESS): README/SUPPORT/EVIDENCE/CHANGELOG name v0.6.5 latest published with the public-docs test repinned. Open risks: unsigned artifacts with SmartScreen disclosure (standing policy); health evidence CPU only; lab legs remain non-blocking with the v0.6.5 lab outcome still to inspect.
- `L-22 | 2026-09-24 | 357cb2f | npm run check (node tests incl. new stays-deleted guard); full gate at L-24 | local logs | PASS` — D4: owner authorized deletion over publication. RED: new `the deleted 0.4.0 corrective note stays deleted` failed on the old tree (file present). GREEN: file deleted; CHANGELOG link replaced with the D4 decision record (L2 test repinned); REVIEW D4 section and file-table row updated; TODO D4 checked.
- `L-23 | 2026-09-24 | 998557d | stays-deleted guard + qualification_manifest 27 of 27; full gate at L-24 | local logs | PASS` — D5: owner granted the frozen-evidence exception. RED: new `the deleted 0.6.0 history stays deleted` failed (85 files present). GREEN: `release-evidence/0.6.0/history/` deleted (453K); no live reader outside carry-forward fixture labels (the fixture writes its own files, 27 of 27 still pass); REVIEW row and TODO D5 updated.
- `L-24 | 2026-09-24 | 8b3ff64 + PR #60 merge 020b16a (pr-check SUCCESS) | npm run check; npm audit; four workflow verifiers; cleanup matrix | local logs | PASS` — D3+P2-1: owner chose plain removal over history rewrite. RED: new `the tracked artifacts directory stays removed` failed (5,494 tracked files). GREEN: `git rm -r artifacts` (0 tracked remain; 34M untracked local outputs stay ignored); `/artifacts/` ignore rule (5-line diff); four `.gitattributes` LF rules removed; the one fixture reader rewired to byte-identical `release-evidence/0.6.1/` copies (mutation back to `artifacts/` fails, restored); both historical manifest tests stage committed bytes at recorded paths (`root`+`workflowRoot`, digests MATCH). GREEN: node tests 290 of 290, Vitest 284 of 284, audit 0 vulnerabilities, versions/pins/gates/syntax pass, cleanup 12 of 12. Rust checks carried (no Rust file changed). No packaged matrix: evidence-removal only; the 0.6.5 binaries are unchanged and published.
