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
  Owner decision 2026-09-26: DEFERRED until the first Write collaborator
  exists (V06-GH-01.V3 waiver already covers this trigger).
- [ ] **A2 — Remove the single-runner dependency (ADMIN).** Owner decision
  2026-09-26: register a self-hosted Windows runner that the team controls;
  the owner runner stays untouched (Admin-grant path rejected). See P1-12.
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
  - Owner decision 2026-09-26: DNS target is 9.9.9.9 first with DHCP
    fallback (not a restore of 1.1.1.1-first). Prefix precedence and IPv6
    binding restore to original unless the ledger notes otherwise.
  - BLOCKED-ELEVATION 2026-09-26: this shell is not elevated
    (`IsInRole(Administrator)` False), `Set-DnsClientServerAddress`
    refused with PermissionDenied, and Windows `sudo` is disabled on
    this host. Before-values recorded: Ethernet static DNS
    {1.1.1.1, 9.9.9.9}, DHCP enabled, 192.168.1.2/24, gateway
    192.168.1.1; `::ffff:0:0/96` precedence 46; `ms_tcpip6` on Ethernet
    Disabled. Fallback evidence: both 9.9.9.9 and 192.168.1.1 resolve
    google.com (nslookup, 2026-09-26). To finish, run these three lines
    in an elevated PowerShell, then record the after-values:
    `Set-DnsClientServerAddress -InterfaceAlias 'Ethernet' -ServerAddresses '9.9.9.9','192.168.1.1'`;
    `netsh interface ipv6 set prefixpolicy ::ffff:0:0/96 35 4`;
    `Enable-NetAdapterBinding -Name 'Ethernet' -ComponentID ms_tcpip6`.
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
- [ ] **P1-12 — Decouple the release jobs from the hardware host.** (In progress
  2026-09-25: `release.yml`/`release-promote.yml` jobs now need only
  `[self-hosted, Windows, X64, localmotive-release]`; `hardware-qualify.yml`
  keeps the hardware labels; runner `DESKTOP-HPTF57N-zen5-blackwell`
  re-registered (API dereg id 21 + fresh configure) with labels
  `self-hosted,Windows,X64,zen5,blackwell,localmotive-hw,localmotive-release`,
  online idle. Gate test updated+extended. Remaining: commit, pre-push gate,
  push the lane, one green `release.yml` dispatch as evidence.) Today
  `release.yml` and `release-promote.yml` need all six labels of the one
  runner on the owner's PC.
  - Behavior: the release jobs use `[self-hosted, Windows, X64,
    localmotive-release]`. `hardware-qualify.yml` keeps the hardware labels.
    At least one runner that the team controls carries `localmotive-release`
    (A2).
  - Evidence: one green `release.yml` run on that runner. Owner decision
    2026-09-26: the team-controlled runner path (A2); owner runner untouched.
- [x] **P1-13 — LAB-03: unify the three CDP clients.** Done 2026-09-25:
  `attach()` in `scripts/lib/cdp_client.mjs` gained `pageFilter` +
  `trustedInput` options; `verify_041.mjs` (deleted 100-line `CdpClient` +
  socket code, keeps its Localmotive page filter + trusted clicks),
  `drive_console_check.mjs`, and `verify_installer_payloads.mjs` (preserves its
  tolerated no-page/probe-error observations) attach through the lib. Only
  `new WebSocket(` site left is the lib. RED: single-owner test failed with 4
  sites; mutant re-add failed it again. `npm run check` 290/290, audit 0,
  verifiers pass.
- [x] **P1-14 — LAB-05: delete or wire the 15 dead g05 drivers.** Done 2026-09-25:
  deleted via `git rm` (`g05_cancellation`, `g05_churn_repro`, `g05_dc01`,
  `g05_hardlink_drive`, `g05_health`, `g05_launch_benchmark`, `g05_mt01d`,
  `g05_partial_resume`, `g05_partial_retention`, `g05_run_cancel`, `g05_state`,
  `g05_stop_supervision`, `g05_tamper_dll`, `g05_v2_cancel`, `g05_vitems_d`).
  Six referenced drivers stay (workflows + tests name them). RED: new
  external-reference test; mutant unreferenced file failed it. `npm run check`
 CHECK_EXIT:0, release-gates 171/0.
- [x] **P1-15 — RT-04: pin the directory inventory for the lease lifetime.** Done
  2026-09-25: `ManagedExecutionLease` carries `expected_files` +
  `revalidate_inventory()`; called after acquisition in `run_runtime_probe_with`,
  `spawn_server` (covers server-service + cold-bench launches), with the health
  context lease site also carrying the list. RED `E0599`, mutant always-Ok
  failed, source-order guards extended. fmt clean, clippy clean, cargo 643/0.
- [x] **P1-16 — RT-10: validate the runtime root before creating it.** Done
  2026-09-25: new `validate_install_root_before_create` (ancestors first, then
  existing-leaf `safe_directory`) runs before `create_dir_all` in
  `install_runtime`. RED `E0425`, mutant call-site removal failed the order
  guard, fmt/clippy clean, cargo 646/0.
- [x] **P1-17 — CORE-05: bound IPC vectors at the command boundary.** Done
  2026-09-25: `MAX_IPC_COMPANIONS` 64 / `MAX_IPC_ADAPTER_IDS` 16 /
  `MAX_IPC_OVERRIDES` 16 + `reject_oversized_ipc_vector`, enforced in
  `inspect_model_artifact` + `preflight_model` before allocation/work. RED
  `E0425`, cap-removal mutant failed, fmt/clippy clean, cargo 648/0.
- [x] **P1-18 — PROC-02: join pipe readers with a deadline after failed cleanup.**
  Done 2026-09-25: `join_reader_with_deadline` (5 s `READER_JOIN_GRACE`,
  detach past the deadline) replaces all six reader joins in
  `output_with_timeout_and_cancel`. RED `E0425`, bare-join mutant failed the
  source guard, fmt/clippy clean, cargo 651/0.
- [x] **P1-19 — PROC-03: surface unresolved termination distinctly.** Done
  2026-09-25: new `ProcessFailureKind::Unresolved` + `cleanup_outcome`
  (timeout/cancel branches route `terminate_and_wait` through it; the
  post-loop branch changed `Io` → `Unresolved`); mapped to new
  `HealthFailureReason::Unresolved` + TS mirror `"unresolved"`. RED
  `E0425/E0599`, ignore-flag mutant failed, `npm run check` EXIT 0, cargo
  653/0, clippy clean.
- [x] **P1-20 — PROC-04: clear the starting slot when `local_client` fails.** Done
  2026-09-25: `local_client(&profile)?` hoisted above `spawn_server` in
  `start_server_worker`; the wait reuses `&client`. A client failure now
  returns while no child/slot exists. RED guard failed pre-fix, move-back
  mutant failed, fmt/clippy clean, cargo 654/0. Note: one full-suite run
  showed an unrelated parallel-load flake
  (`health_from_an_unrelated_process...`, isolated green, rerun green);
  not hidden, not caused by this diff.
- [x] **P1-21 — PROC-05: stop following redirects on the health readiness client.**
  Done 2026-09-25: extracted `loopback_readiness_client` (`Policy::none` +
  `no_proxy`, same timeouts) used by the loopback stage; ported R15 redirect
  test fails without the policy. RED `E0425/E0433`, policy-removal mutant
  failed, fmt/clippy clean, cargo 655/0.
- [x] **P1-22 — PROC-06: track and join the nested completion worker.** Done
  2026-09-25: `completion_request_supervised` keeps the worker `JoinHandle`
  and routes cancel/deadline exits through `join_completion_worker` (5 s
  grace via `proc::join_reader_with_deadline`, now `pub(crate)`); a survivor
  reports `HealthFailureReason::Unresolved`. RT-03 fixture updated so
  `terminate` models a real kill (releases the socket). RED failed pre-fix,
  abandon-mutant failed, RT-03 5/5 in 0.58 s, cargo 656/0, clippy clean.
- [x] **P1-23 — PROC-07: refuse links in transport-file reads.** Done 2026-09-25:
  `read_bounded_file` runs `artifact::validate_regular_non_reparse_file`
  before the open. RED via removal mutant (pre-fix error was `could not be
  read... Access is denied`, proving the link was followed); file symlinks
  need privilege this host lacks, so the test uses one when creatable and a
  privilege-free junction otherwise. fmt/clippy clean, cargo 657/0.
- [x] **P1-24 — PROC-08: stop following replaced log paths.** Done 2026-09-25:
  `second_writer` runs new `download::ensure_no_link_or_reparse` (no-open
  variant: link-counting opens fail on the sink-held file with os error 32,
  and extra hard-link names cannot divert a pinned handle), `write_failure_evidence`
  runs full `ensure_safe_write_entry` (truncating write). Reused the RT-01
  check instead of a new helper. RED `E0425`, removal mutant failed the
  guard, fmt/clippy clean, cargo 660/0.
- [x] **P1-25 — DL-01: revalidate override rows at read time.** Done 2026-09-25:
  new `row_mac` column (schema v2, in-place `ALTER` for v1, no backfill —
  backfilling would launder a swapped digest) + HMAC-SHA256 tag over every
  authorization field, key in Credential Manager (`Localmotive /
  catalog-override-mac`, get-or-create; `hmac 0.12` dep justified as the
  audit fix). `user_override_file` rechecks the tag each read (constant-time
  `verify_slice`); legacy rows refused until re-saved. RED tamper test,
  bypass mutant failed (2 tests), round-trip + legacy + migration tests,
  fmt/clippy clean, cargo 664/0.
- [x] **P1-26 — DL-03: verify the handle, not the path, on reads.** Done 2026-09-25:
  new `artifact::open_verified_read_file` (pre-check, then open, then
  handle verify: link count 1, regular bit, final-path equals the requested
  path; unix `nlink` variant). Migrated `sha256_path`, `sha256_prefix_path`,
  `gguf read_summary`, `read_bounded_file` (reads the handle, no re-open),
  health model hash. Execution sites (probes/spawns) stay under the
  managed-execution lease, not this helper — pinned by the `proc10_readers`
  source guard. RED `E0425`, skip-verify mutant failed the hard-link test,
  fmt/clippy clean, cargo 668/0.
- [x] **P1-27 — DL-04: bound the OAuth key-exchange response.** Done 2026-09-25:
  new `MAX_KEY_EXCHANGE_BYTES` (16 KiB; the answer is one short JSON object)
  + `exchange_code_for_key` reads through `read_bounded_body` instead of
  unbounded `response.text()`. `read_bounded_body` now takes `impl Read` so
  the cap is unit-testable without network (gate forbids live calls).
  Behavioral oversized test + call-site source guard (self-comment-safe),
  cap-removal mutant failed, fmt/clippy clean, cargo 670/0.
- [x] **P1-28 — MT-01: stop presenting a same-prompt repeat as confirmation.** Done
  2026-09-25: relabel (no held-out prompt; the fixed harness prompt is a design
  constant). UI now says "reproduced / did not reproduce on the fixed harness
  prompt (same-prompt re-measurement[, not independent confirmation])". Wire
  field `confirmed` kept so old reports load (documented on the struct);
  stopped_reason "stays unconfirmed" → "records no verdict". RED render test
  + label mutant failed, `npm run check` EXIT 0 (291/291), cargo 670/0.
- [x] **P1-29 — MT-03: persist warmup-only failures as Failed manifests.** Done
  2026-09-25: `validate_attempt_consistency` accepts empty observations iff the
  terminal outcome is Failed or TimedOut (the run carries warmup errors +
  outcome, so it persists as Failed via the existing `(Err, Some)` summary
  arm). Cancelled/None/Succeeded + empty still rejected (cancel leaves no
  record, as before). RED contract test + reject-mutant failed, fmt/clippy
  clean, cargo 671/0.
- [x] **P1-30 — MT-05: restore the server after a cold v2 benchmark.** Done
  2026-09-25: relaunch chosen (refusing would make cold unreachable — cold
  needs a live snapshot at start). `benchmark_v2` keeps the cold profile,
  finalizes the record, drops the reservation, then relaunches through the
  normal `start_server` path. New `serverRestored`/`serverRestoreError` on
  `BenchmarkRunResult` (mirrored in `model.ts`, shown in the evidence panel);
  relaunch failure keeps the record and names the error. RED merge test +
  discard-mutant failed; R04 gate updated for `let mut result` (drain-before-
  read invariant unchanged); `npm run check` EXIT 0 (291/291), cargo 672/0.
- [x] **P1-31 — MT-06: bound the worker drain.** Done 2026-09-25:
  `drain_owned_workers` returns `DrainOutcome::{Drained, Unresolved}` after one
  bounded ceiling wait; the infinite 1 s loop is gone. Snapshot folds
  Unresolved into Failed + a drain note (observations stay in the manifest);
  the v2 command marks Failed, appends the note, and skips the relaunch (R04
  overlap) with a recorded skip reason; legacy discards with a drain error
  after clearing its slot. Rule change per AGENTS.md (R04-as-tested blocked
  every bounded path): infinite-hold pins replaced in release-gates + the old
  r16 wait test (scenario now in proc14). RED proc14 + always-Drained mutant
  failed; `npm run check` EXIT 0 (291/291), cargo 673/0, clippy clean.
- [x] **P1-32 — MT-08: make replay respect unknown execution identity.** Done
  2026-09-25: `validate_replay_compatibility` takes the current machine's
  unknowns and refuses while either side is non-empty, naming the unobserved
  identities (`Replay refused: unobserved execution identity (...);
  re-measure ...`). Worker passes the rebuilt snapshot's unknowns instead of
  discarding them. RED proc15 (both sides) + key-only mutant failed; cargo
  674/0, fmt/clippy clean.
- [x] **P1-33 — MT-09/FE-05: remove the legacy benchmark system.** Done 2026-09-25:
  deleted `benchmark_server` + `run_legacy_benchmark` + `finalize_legacy_benchmark`
  + registration + guard tuples; stripped the screen's legacy half (Benchmark view
  is now the v2 panel only); removed `runBenchmark`/legacy state/tokens/repeats,
  the Dashboard LAST TEST tile, `legacyBenchmarkInputError` + `BenchmarkSummary`
  (frontend; Rust keeps both for tune's `LiveBench`), and 26 obsolete tests
  (storage save-failure, staleResponses legacy block, model input). Kept
  `cancel_benchmark` (v2 adapter) + the slot helper test (renamed). RED absence
  test + registration-restore mutant failed; `npm run check` EXIT 0 (265/265),
  cargo 670/0, clippy clean. Post-commit fix: python text-mode writes had
  flipped LF to CRLF file-wide on the 5 script-edited files; converted back to
  LF and amended (40+/807-).
- [x] **P1-34 — FE-04: split `App.tsx`.** Done (first piece) 2026-09-25:
  extracted the persistence cluster (`readRecord`, `quarantineRecord`,
  `persistRecord`, `persistenceFailureNote`, `readSetting`) verbatim to
  `src/persistence.ts`; `App.tsx` 1822→1781 lines. Branding allowlist moved
  with the migrated lines. `npm run check` EXIT 0 (265/265). Next split
  recorded: the `useDebouncedValue` hook + `inTauri`/`idleStatus` module
  preamble, then one screen's coordination (tune or catalog).
- [x] **P1-35 — FE-06: route `cancel_scan` through props.** Done 2026-09-25:
  `InventoryScreen` takes `cancelScan: () => void` (no `invoke` import left);
  `App.tsx` owns `cancelScan()` beside `scan()`. RED boundary test fails on the
  old file (`invoke(` present, no `cancelScan`); `npm run check` EXIT 0 (265/265).

---

## 6. P2 — Simplification after the release

- [x] **P2-1 — Remove `artifacts/` (D3).** Done with D3: `/artifacts/`
  ignore rule added, the four `.gitattributes` LF rules for `artifacts/`
  removed. The one test that read tracked `artifacts/*.json` fixtures now
  reads the byte-identical `release-evidence/0.6.1/` copies; all other
  `artifacts/` references are runtime workspace paths or fixture-temp labels.
- [x] **P2-2 — Delete dead scripts (LAB-05).** Done 2026-09-25: new generic
  sweep test (`every automation script is referenced or deleted`) was RED with
  11 dead files, then caught a 12th (`watch_console_windows.py`, REVIEW-only
  mention) the manual pass missed; deleted all 12 via `git rm`. `npm run check`
  EXIT 0 (265/265).
- [x] **P2-3 — Keep one CDP client (LAB-03):** `scripts/lib/cdp_client.mjs`.
  Triple-checked 2026-09-25: exactly one `new WebSocket(` in `scripts/`
  (`lib/cdp_client.mjs:44`); consolidated by P1-13; enforced by the
  `the CDP WebSocket is constructed in exactly one place` gate (green in
  `npm run check` EXIT 0 runs). No change needed.
- [x] **P2-4 — Replace source-text tests with behavior tests.** Done (safe
  subset) 2026-09-25: deleted `cold_benchmark_uses_a_fresh_runtime...` (twinned
  by `proc13` + `cold_harness_*` behaviorals) and `r16_..._releases_ownership...`
  (twinned by `proc14` + `r16_*` + `f9_02_*` behaviorals, all mutant-proven in
  P1-30/P1-31). KEPT the other 7 `ALL_SOURCES` guards (managed-trust, lease
  ordering RT-04, contained-spawn, launch evidence, snapshot gate, memory
  wiring, IPC-01 placement: ordering/absence/wiring invariants with no
  unit-observable equivalent; deleting them loses trust-boundary coverage) so
  `ALL_SOURCES` stays; KEPT the release-gates doc/decision pins (D3/D4/D5, L2
  ceiling, unsigned disclosure: owner-ordered locks, not incidental prose).
  Rust 668/0, fmt/clippy clean.
- [x] **P2-5 — Remove the audit-ticket labels.** Done 2026-09-25: stripped
  286 ticket-label comment lines (period-preserving redo after a first pass
  ate `XS-00001` inside a string literal and 20 sentence periods; lesson:
  ticket regexes need `\b` anchors and must never eat trailing periods, and
  the ticket must sit at paren-content start) and renamed 192 ticket-prefixed
  test fns to behavior names across 19 files (no new duplicates; compiler is
  the verifier). Gate test extended (comments + `fn` prefixes; mutant-probed:
  flags `r16_/proc14_/s05_`, passes `process_/sha256_`). Release-gates name
  pins updated to the new names (R10/R15/R04/R16 pins; the deleted P2-4 guard
  pin now points at the surviving cancellation twin in measurement_service).
  Rust 668/0, fmt/clippy clean, `npm run check` EXIT 0 15 files / 265 tests
  (`/tmp/check41.log`), audit 0, versions/pins/gates PASS. Scope: Rust source
  only; scripts/ evidence keys (`mt06_verdicts`, driver names) untouched.
- [x] **P2-6 — Keep one benchmark system (MT-09, FE-05).** Done 2026-09-25:
  tuner migrated to the v2 contract via new `measure_trial_summary` probe
  (`measurement_service.rs`: per-trial `evidence::Workload`, v2 warm workload
  runner + `completion_request_cancellable`, `summarize_observations` →
  `BenchmarkSummaryV2`); `TrialMeasurement.summary` + consumers (evidence
  mapping, winner `decode_tps.mean`, fixtures via new `summarize_fixture_tps`)
  on V2; V1 core block deleted (`BenchmarkSummary`, `summarize_benchmark`,
  `parse_tps`, V1 completion path, `validate_benchmark_options`,
  `LEGACY_BENCH_TIMEOUT`, `benchmark_server_cancellable`, 8 tests); lib.rs
  live-bench guard repointed to the probe. One cut-script range swallowed an
  unrelated test (`raw_file_arguments...`); restored byte-identical to HEAD.
  Semantic changes: trial spread is population std (0.5 for [130,131], was
  sqrt(0.5)); trials keep partial successes with `failed_trials` noted (V1
  aborted on first error). Mutant `.skip(1)` in fixture helper failed 6 tune
  tests, green after restore. Rust FMT_CLEAN, clippy 0, 660/0 exact;
  `npm run check` EXIT 0 15 files / 265 tests (`/tmp/check42.log`).
- [x] **P2-7 — Keep one catalog schema source (LAB-06).** Closed 2026-09-25
  as corroborated-refuted (no code change): the two-validator copy is
  deliberate and pinned by `shared_fixture_cases_agree_with_the_javascript_
  validator` (catalog.rs:1940, green in the 660/0 suite); the alleged shape
  gap is benign by contract — `sequence`/`expires` are `Option<u64>`
  (catalog.rs:255/259), enforced only when present (DC-10 replay protection
  for served updates), so the checked-in catalog without them and
  `validate_catalog.mjs` pinning only `schemaVersion: 2` are consistent, not
  contradictory. P1-11 REFUTED stands.
- [x] **P2-8 — Split `src/App.tsx` (FE-04)** and keep IPC out of the screens
  (FE-06). First increment done 2026-09-25: cloud-credential cluster
  (~160 lines: providers, credential, key draft, model list, probe state +
  all 8 acquisition fns) extracted to `src/useCloudCredentials.ts`;
  `App.tsx` 1792→1638 lines, screens unchanged (same props). RED gate
  `cloud credentials live in a hook` failed before, passes now. `frontend
  Sources()` registers the hook so FE-03/FE-06 pins keep seeing moved code.
  Mutant (drop FE-01 failure-clear) failed exactly
  `clears the cloud key draft when the save fails`, green after restore.
  `npm run check` EXIT 0, 15 files / 265 tests (`/tmp/check46.log`),
  audit 0 vulns. Remaining App.tsx clusters (runtime, catalog, tuning) split
  in later increments.
- [x] **P2-9 — Merge `docs/EVIDENCE-MATRIX.md` into
  `docs/SUPPORT-MATRIX.md`.** Done 2026-09-25: version table (hosts,
  versions, column meanings, reading rules) plus the S-14/S-16/S-18/S-19
  audit notes transplanted to a `Version evidence` section (78→193 lines);
  `git rm` the old file; backlinks in README/CHANGELOG repointed; the two
  gates that read the old file now read SUPPORT-MATRIX. RED gate
  `EVIDENCE-MATRIX is merged` failed before, passes now; resurrecting the
  old file fails it again (mutant). `npm run check` EXIT 0
  (`/tmp/check47.log`).
- [x] **P2-10 — Prune superseded evidence (D5).** Closed 2026-09-25 as
  already-done (no change): D5 executed under L-23 (`998557d`,
  merged via PR #60) — `release-evidence/0.6.0/history/` (85 files, 453K)
  deleted, tree now holds only attestations + inventory/manifest/SHA256SUMS;
  the stays-deleted guard rides in the full gate (green in `/tmp/check47.log`).
- [x] **P2-11 — Remove stale references to deleted files.** Done 2026-09-25:
  pruned 10 dead entries from `HISTORICAL_FILES` (`TODO-0.6.md`,
  `Future_branding.md`, `docs/history/*` ×5, `TODO-0.4.md`, `TODO-0.4.1.md` —
  none exist in the tree; closed trackers live in git history `9b09857`);
  kept `CHANGELOG.md`, `TODO.md`, `research-freeze-manifest.json`. Replaced
  the obsolete `covers the archived docs layout` gate with
  `names only files present in the tree` (RED failed, GREEN passes; re-adding
  one dead entry fails it again). The `catalog/providers.json` note stays
  until the next catalog change, per the row. `npm run check` EXIT 0
  (`/tmp/check48.log`).
- [ ] **P2-12 — Triage the 72 Medium and 9 Low findings** in `REVIEW.md`, one
  part at a time. Part 1 done 2026-09-25 (REL-14..16, LAB-07..13, dispositions
  + REL-16/LAB-12 fixes below; remaining parts: RT, CORE, PROC, DL, MT, FE):
  - REL-14 (duplicate CI/release gates): ACCEPT — release re-runs source
    checks on the tag peel by governance rule, not by accident.
  - REL-15 (gate/workflow coupling): ACCEPT under the P2-4 policy (prune
    twinned guards, keep trust-boundary pins); no blanket deletion.
  - REL-16 (hardware summary names hosted runner): VERIFIED + fixed — summary
    now names `localmotive-release`, gated by `hardware summary names...`.
  - LAB-07 (fixed sleeps)/LAB-08 (text selectors): bulk SUPERSEDED by P1-14 +
    P2-2 deletions; residual lab-only drivers run in the non-gating night
    campaign → ACCEPT.
  - LAB-09 (hard-coded ports): SUPERSEDED — cited files deleted, survivors
    take `$labPort`/`$portable` args.
  - LAB-10 (campaign names): SUPERSEDED — P1-14/P2-2 executed its delete list;
    `verify_041`/`verify_060_catalog` names are frozen contract identities.
  - LAB-11 (`dryrun_catalog`): SUPERSEDED — file deleted in P2-2.
  - LAB-12 (catalog tie-ordering): VERIFIED + fixed — secondary keys
    (`filename`, `repo`) on all three sorts, gated by `deterministic
    tie-breakers` (RED→GREEN→mutant). Freshness bytes still change per build
    by DC-10 design.
  - LAB-13 (Low, error coercion): ACCEPT — cosmetic, lab-only survivors.
  Part 2 done 2026-09-25 (RT-08/09/11..15 + RT-01/RT-03 adjudication):
  - RT-08 (GPU preflight Unknown): ACCEPT — Unknown is the honest contract
    without a GPU (`Show unknown when evidence is unavailable`); night
    hardware campaign covers real GPUs, never the release gate.
  - RT-09 (lossy help discovery): REFUTED — discovery is lossy by nature of
    `--help` prose, but authority is `filter_supported_args` against
    `capabilities.supported_flags` (core.rs:1645), which enforces the rule.
  - RT-11 (poisoned state locks): VERIFIED + fixed — new `lock_recover`
    helper in `runtime_service.rs`, all 6 sites converted, RED
    (`E0432` pre-helper) → GREEN (`poisoned_state_lock_recovers...`) →
    mutant (unwrap restore fails it). The lib.rs slot-protocol pin follows
    the new spelling. The wider 20-site idiom stays: same rationale.
  - RT-12 (source-spelling tests): ACCEPT under the P2-4 policy — the RT-11
    pin update above is the policy working, not a violation.
  - RT-14 (version-pinned data): ACCEPT — pinning IS the trust mechanism
    (P0-1 compiled catalog); bumps update pins by design.
  - RT-13 (Low, dead scaffolding): VERIFIED + pruned — zero-caller
    `GithubAsset::sample` test helper deleted (661/0 after).
  - RT-15 (Low, ticket narration): SUPERSEDED — P2-5 stripped it.
  - RT-01/RT-03 (Medium, same-user races): ACCEPT per the Medium threat
    model; P1-10/P1-15 closed the user-data half. Rust 661/0, fmt/clippy
    clean.
  Part 3 done 2026-09-25 (CORE-07..12 + CORE-01/02/06 adjudication):
  - CORE-01 (catalog override + banner): CLOSED — done in P1-9
    (`verification_mode` + banner + SECURITY.md).
  - CORE-02 (Low, StrictMode double effects): ACCEPT — dev-only; the packaged
    collision half is RT-06's scope.
  - CORE-06 (WEBVIEW2 env args): ACCEPT — standard platform behavior,
    same-user trigger per the Medium model.
  - CORE-07 (SKIP_HARDWARE_PROBE): ACCEPT — set only in `ci.yml` Rust steps;
    release/hardware never set it, and the skip writes a visible
    `detection_status` string in-product (runtime.rs:1352).
  - CORE-08 (help parser weaker than contract): REFUTED — same evidence as
    RT-09 (`filter_supported_args`, core.rs:1645).
  - CORE-09 (capability not minimal): PART FIXED — removed the two dead
    opener origins (`tauri.app`, `react.dev`; no in-app URL targets them),
    gated by `opener allowlist carries no unused origins` (RED→GREEN).
    `core:default` bundle + loopback wildcard ACCEPTED: bundle replacement
    risks packaged-only breakage beyond a Medium; loopback serves
    user-selected server ports.
  - CORE-10 (dead commands/duplicates): VERIFIED + fixed — legacy
    `scan_models` command deleted (no frontend/script caller; tests mock
    `scan_models_report`); registration + expensive-command span re-anchored;
    `core::scan_models` kept as `#[cfg(test)]` shorthand. Cancellation
    commands are live (P1-35); runtime-setup/scan pairs are distinct commands.
  - CORE-11 (mixed domains): ACCEPT as managed-incrementally — narration
    stripped (P2-5), splits continue per-increment (P1-34/P2-8); no big-bang
    rewrite.
  - CORE-12 (error conversion + lock unwraps): SPLIT — plain-string errors
    are contractual (S-18 wire contract) → REFUTED half; all 14 command-slot
    `.lock().unwrap()` (6 runtime_service + 8 lib.rs) converted to the shared
    `lock_recover` → FIXED half (661/0, fmt/clippy clean). Lesson: new Rust
    comments must not carry ticket labels (P2-5 gate caught three).
  Part 5 done 2026-09-25 (DL-05..16, all mutant-proven unless noted):
  - DL-05 FIX: GGUF pair budget 1,000,000 to 65,536; relevant facts capped
    at 1,024 via new `max_facts` parse limit (file order kept, sorted
    after). Duplicates stay first-wins (deterministic `find`), not
    rejected: rejecting risks breaking legitimate files.
  - DL-06 FIX: row bound moved into SQL (`ORDER BY id LIMIT 5001`);
    per-model file cap 4,096 added. Boundary inserts run in transactions
    (0.43 s for the set, was 23 s of fsyncs).
  - DL-07 FIX: override revisions pass the shared signed-catalog
    `is_safe_revision` rule (now `pub(crate)`); traversal, empty
    segments, blank, padded values rejected, `refs/pr/27` accepted.
    Single-dot segments stay accepted (normalize away in URLs, same as
    the signed path).
  - DL-08 REFUTED: CRLF normalization is semantics-preserving for JSON
    (both forms parse identically); exact-bytes would break CRLF
    checkouts, pinned by `crlf_catalog_still_verifies_after_normalization`.
  - DL-10 FIX: cloud API client carries explicit `Policy::none`; a 302
    surfaces as an error, credentials never follow.
  - DL-11 FIX: request-side bounds (key 4 KiB, model id 256 B, prompts
    256 KiB) enforced in `save_credential` and `chat_with_deadline`
    before keyring writes and request construction.
  - DL-12 FIX: cloud retry-after delegates to the one shared
    download-side parser (numeric, date, 86,400 s horizon); orphaned
    `MAX_RETRY_AFTER_SECS` removed. Absurd waits fall back to backoff
    instead of sleeping 30 s.
  - DL-13 FIX: `httpdate_secs` allowlists weekday names; 14-case
    boundary test pins shape, ranges, junk rejection, leap-:60
    documentation.
  - DL-15 FIX: `open_catalog_db` validates reparse ancestors before
    creation; squatted roots fail loudly. Rejection arm is
    privilege-gated (this host cannot create symlinks); control
    (real dir opens) proves everywhere.
  - DL-16 REFUTED: resume identity is same-source string equality;
    mismatch direction is safe (full re-download, never a wrong resume).
  - DL-09 FIXED 2026-09-26: `catalog/README.md` gains a `Key rotation`
    section — sequenced single-key rollover (new-key release with
    new-signed bundle while the old-signed catalog is still served, then
    switch the publish signing + `CATALOG_SIGNING_KEY_PEM`, then destroy
    the old key); schema v2 has no dual-signature support. Owner decision
    2026-09-26: single-key rollover STANDS; no dual-sign schema change.
  Part 6 done 2026-09-26 (MT-02/04/11..15):
  - MT-02 (present-but-invalid `first_token_ms` silently dropped):
    VERIFIED + fixed — strict parse now errors on a present invalid
    value (`completion_timing_rejects_a_present_but_invalid_first_token`
    RED→GREEN); finite-range guard on derived TTFT (mutant-proven).
  - MT-04 (evidence persistence weaker than acquisition): VERIFIED +
    fixed — `validate_complete` rejects zero observation metrics and
    enforces requested-context identity + exact warmup counts
    (`manifest_rejects_a_zero_observation_metric` etc. RED→GREEN→mutant).
  - MT-11 (redaction misses non-path secrets): VERIFIED + fixed —
    case-insensitive bare-account-name redaction everywhere, not just
    home-path prefix (`minimal_disclosure_removes_a_bare_account_name_outside_paths`
    RED→GREEN). Residual: arbitrary non-path names stay in evidence text
    (accepted: bounded to the operator's own machine).
  - MT-12 (cloud timeouts): ACCEPT — bounded (180 s clamp, one bounded
    429 retry, tuning deadline); Stop latency documented.
  - MT-13 (unknown dead fields): SPLIT — unknowns REFUTED (consumed by
    the P1-32 replay gate); dead `launch_compatibility_key` and
    `last_raw_reply` plumbing deleted (cargo check clean); narration
    CLOSED by P2-5.
  - MT-14 (log drains detached from Stop): VERIFIED + fixed — bounded
    `join_log_drains` (2 s deadline, lingering drains reported) called
    in managed stop, cold-bench cleanup, and trial cleanup
    (`log_drains_join_before_the_deadline...` RED→GREEN→mutant).
  - MT-15 (two std-dev conventions): VERIFIED + fixed — population SD
    documented on both live fields; shared [130, 131] → 0.5 vector pins
    it on both sides; dead `sample_std_dev` + its 3 tests deleted
    (overflow-robustness retained via `metric_summary_remains_finite` +
    extremes tests).
  Part 7 done 2026-09-26 (FE-01/07..13):
  - FE-01 (credential drafts linger): CLOSED — P1-7 stands: both screens
    clear drafts on save success, guarded failed save, and unmount
    (verified in source).
  - FE-07 (stale owner callback): VERIFIED + fixed — owner callback
    through a ref; mid-run swap test (`owner.test.tsx`) RED→GREEN→mutant
    (old effect fails it); eslint-disable removed.
  - FE-08 (reset throws on denied storage): VERIFIED + fixed — exported
    `resetSavedState` (try/catch-report/finally-reload, injectable
    reload); 2 tests RED→GREEN→mutant. Caught a vacuous-test trap:
    jsdom storage does not route through `Storage.prototype`, so the
    denied-storage test uses `vi.stubGlobal` (the prototype-spy version
    passed the mutant).
  - FE-09 (lamp shadows vs flat rule): CLOSED — only the two lamp
    shadows exist; DESIGN.md now names the lamp-only exception.
  - FE-10 (production casts): FIXED — `Window.__TAURI_INTERNALS__`
    augmentation in `vite-env.d.ts` (App.tsx + 6 test sites simplified);
    fingerprint is a typed projection (no cast); explicit
    `invoke<ResultType>` generics in the adapter; `tsc` clean. Remaining
    casts are test-fixture partial shapes (accepted residual).
  - FE-11 (brittle UI tests): ACCEPT — exact-copy IPC assertions are
    release-gate contract, class assertions pin status semantics,
    real-timer waits are bounded and green.
  - FE-12 (ts-expect-error in vite.config): FIXED — `@types/node`
    devDependency + `types: ["node"]` in tsconfig.node.json; node tsc
    clean. Lockfile churn 19 lines.
  - FE-13 (token parallel sources): FIXED — DESIGN.md gains an Evidence
    Panel section (bars/actions/status grammar from the real code) and a
    source-of-truth note (`src/App.css` wins; theme.css/tokens.json are
    reference mirrors); design lint 0 errors 0 warnings.
  Gate-test fallout from the above, both intent-preserving: the
  `release-gates` FE-07 adapter regex now tolerates the explicit generic
  (command pin unchanged); the lib.rs tuning-lifecycle pin follows the
  drain-join cleanup spelling.
  Shipped 2026-09-26 as `f80108f` on `fix/p1-defects` (pushed); PR #66
  lane→main opened, pr-check pending (run 36238338443). Merge only after
  pr-check passes.
  Part 4 done 2026-09-25 (PROC-09..22):
  - PROC-09 (cleanup port-closed): REFUTED with measured evidence — this
    host reports closed loopback ports (even never-bound ones) as `TimedOut`,
    not `ConnectionRefused` (probe2). A refused-only predicate would fail
    cleanup everywhere here. Kept `.is_err()` but extracted
    `loopback_port_is_closed` with the platform note + a pin test; the port
    leg stays corroborating (conjoins with `child_stopped`/`tree_stopped`).
  - PROC-10 (health contract debug-only): FIXED the release half —
    `finish_run` now fails closed on an invalid stage list in all builds
    (was `debug_assert!` only), pinned by RED-then-GREEN test. No external
    producers exist (all construction inside health.rs); typed observations
    per stage would change the IPC wire — deferred.
  - PROC-11 (CPU enumeration): FIXED — CPU branch requires the
    `Available devices:` header (from the real fixture) and reports
    CPU-specific detail (no more "exact adapter" claim). Mutant-proven.
  - PROC-12 (completion mislabeled): FIXED — `completion_failure_reason`
    maps cancelled/timeout/output-limit separately, preserves the client
    string in the detail (was: always Timeout, string erased).
    Mutant-proven. Fallback is MalformedOutput (matches the Io precedent);
    no new IPC variant.
  - PROC-13 (failed stages skip cleanup): FIXED — 10 failing exits now route
    through `fail_health_stage`, reporting Unresolved when the tree survives
    termination (was: bool discarded). Pinned by a wiring test (exactly one
    bare call survives: the supervised-cleanup closure, owned downstream)
    + a pure-outcome unit test. One repair detour: a mutant edit ate a
    call's closer; repaired by hand, 670/0 after.
  - PROC-14 (readiness bound): REFUTED — client carries 2s per-attempt
    timeout + 250ms connect timeout (P1-21), loop bounded by the 120s budget
    with cancel checks; 100ms sleep is a bounded poll.
  - PROC-15 (DER walker): residual PINNED — pair/mismatch coverage existed;
    mutant proved the DER-frame fail-closed paths untested (indefinite
    length accepted silently). Added `der_frame_rejects...` unit test
    (fails on mutant, passes restored).
  - PROC-16 (detached drains): CLOSED — superseded by P1-31 (bounded drains
    called in benchmark/command paths).
  - PROC-17 (tuning port sleep): FIXED — `wait_for_port_release` polls
    connect-until-refused with a 5s bound (was blind 600ms sleep).
    Mutant-proven (sleep-600 fails the <500ms closed-port assert).
  - PROC-18 (stop holds mutex): FIXED — take-out-terminate-commit; slot
    restored on termination failure; reservation held across. Existing stop
    tests (incl. real-process reap) green.
  - PROC-19 (log retention): FIXED — NotFound → Ok(0), other read failures
    → Err (was: all Ok(0)). RED-then-GREEN. Call site stays non-fatal by
    decision (launch must not brick on diagnostic retention).
  - PROC-20 (source-text tests): ACCEPT per P2-4 (trust-boundary policy
    pins stay; behavior tests added everywhere else this part).
  - PROC-21 (test flake): FIXED the PID-only dirs — new
    `test_support::unique_temp_dir` (pid + nanos + counter) + migrated 4
    local_client sites. `settle_until` already bounded; `temp_path` file
    names stay per-test-distinct (residual noted).
  - PROC-22 (ticket narration): CLOSED — superseded by P2-5.
  Rust 671/0, fmt/clippy clean; `npm run check` EXIT 0 (`/tmp/check52.log`).
  Watch: one unidentified 669/1 transient during the PROC-13 repair; three
  subsequent full runs green (670/0 ×2, 671/0 ×1) + 12/12 new-test repeats.

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

Owner decision 2026-09-26: schedule Bounded UI input next (was unscheduled).

- [x] **Bounded UI input (SCHEDULED 2026-09-26, done on
  `fix/bounded-ui-input`):** no value outside its valid range (see
  CORE-05). Backend bounds done in P1-11; all three surfaces already gate
  on validators pre-action. This lane fixed the residual defect: numeric
  fields ate incomplete typing (`valueAsNumber` → NaN → `""`), against the
  DESIGN rule. New `parseNumericText` (model.ts, 3 RED→GREEN tests) +
  `useNumericText` hook (RED→GREEN→mutant; resyncs only on external value
  change via ref), applied to Profile (41 fields via module-level
  `ProfileNumberInput`), Tune (3), and the benchmark workload (4); fields
  are text + numeric keyboard with error-as-text gating unchanged. Two
  mechanism-pinned tests repinned to the new contract with intent intact
  (V03 `checkValidity` → errors-list/inputMode; Profile native bounds →
  visibility + validator bounds). Caught live: the first resync design
  fought a noop parent (Profile test RED) — fixed with the change-ref.
  `npm run check` EXIT 0 (node 303/0, Vitest 273/273, build OK), audit 0
  vulns, tsc clean, design lint 0/0. No Rust changes; Rust results carried
  from L-25.
- UI speed and responsiveness. Measure first.
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
- `L-25 | 2026-09-26 | ef18931 plus uncommitted lane changes (P2-12 MT/FE/DL-09) | npm run check; npm audit --audit-level=moderate; four workflow verifiers; verify_cleanup_matrix.ps1; cargo fmt --check; cargo clippy --locked --all-targets -- -D warnings; RUSTDOCFLAGS='-D warnings' cargo test --locked; tsc app+node; designmd lint | local logs (/tmp/npmcheck2.log, /tmp/cargotest2.log) | PASS` — MT-02/04/11/14/15 + MT-13 deletions, FE-07/08/10/12/13 + FE-09 doc, DL-09 rotation doc, two intent-preserving gate-test repins (release-gates adapter regex, lib.rs lifecycle pin). RED→GREEN→mutant for every behavioral fix (incl. a caught vacuous jsdom storage-spy test). GREEN: node tests 303/0, Vitest 268/268, audit 0 vulnerabilities, four workflow verifiers pass, cleanup 12/12, fmt clean, clippy 0 warnings, Rust 686 passed 0 failed 6 ignored (default parallelism), tsc clean both projects, design lint 0 errors 0 warnings. Watch: one `dropping_a_contained_process_terminates_descendants` failure in an earlier full-suite run under parallel load; passes solo and 14/0 ×3 in isolation plus this full green run — recorded, not a blocker. EOL hygiene: three tool-touched files normalized back to HEAD LF convention. New dep `@types/node ^22` (lockfile +19 lines).
- `L-26 | 2026-09-26 | a81a382 (merge of 46d15a0) | PR #66, CI run 36238443887 job pr-check | — | PASS` — lane `fix/p1-defects` (56 commits: P1 fixes + P2-12 REL/LAB/RT/CORE/PROC/DL/MT/FE triage) merged to `main` after pr-check SUCCESS (9m32s on `windows-latest`). Lane branch deleted locally and remotely; checkout is `main` at `a81a382`, clean. Remaining open: P0-11 (waits for owner DNS answer), P1-12/A2 (team-controlled runner or Admin grant), A1 (permission grants + push-refusal test), backlog, section 7 risks.
- `L-27 | 2026-09-26 | 397da7b (merge of a52a29a) | PR #70, CI run 36249866371 job pr-check | — | PASS` — backlog Bounded UI input merged to `main` after pr-check SUCCESS (10m22s on `windows-latest`). Lane branch deleted locally and remotely; checkout is `main` at `397da7b`, clean.
