# Localmotive repository review

Review date: 2026-09-24 (Asia/Dubai).

Reviewed source: branch `fix/u06-stabilization` at
`9b098577095ab80ead9bff6e5475456166b340c1`. `origin/main` at
`0bd39fbfa0c9c19f08f6c4d3c185bb0440169fe0` contains this commit.

Product code at this commit equals the `v0.6.1` peel `4b31431`. In
`scripts/`, `.github/`, `src/`, and `src-tauri/`, only
`.github/workflows/release.yml` (+242 lines) and
`scripts/tests/release-gates.test.mjs` (+39 lines) differ.

Review status: **PARTIALLY VERIFIED**.

- The lead reviewer ran every local gate and read the release run records.
- Nine scoped reviewers read the files in their scope. Each reviewer reported
  findings with `file:line` evidence.
- The lead reviewer checked 35 of the 137 findings against source, run
  records, or artifacts. 23 held as stated. 12 held with a changed severity.
- The other 102 findings are reviewer reports. The finding index marks each
  of them `UNVERIFIED`.
- This review did not start a packaged build, a CI run, a dispatch, or a
  promotion.

---

## 1. Verdict

The product does not stop a release. The release system stops it.

1. **The code is healthy enough to ship.** At `9b09857`, every local gate
   passes (section 8).
2. **Nothing has shipped for 13 days.** The last published release is
   `v0.5.0` (2026-09-10T18:56:46Z). Since then, 14 release runs started, and
   0 passed. `release-promote.yml` has never run, not even once.
3. **The team works on process, not product.** 191 of the 308 non-merge
   commits since `v0.5.0` change no file in `src/` or `src-tauri/src/`. The
   `v0.5.0..HEAD` diff changes 5,928 files. 5,728 of them (96.6%) are
   captures in `artifacts/` and records in `release-evidence/`. Only 200 are
   other files.
4. **The repository is mostly capture debris.** 5,494 of the 6,005 tracked
   files (91.5%) are raw captures in `artifacts/`. They include 33 WebView2
   browser profiles.
5. **The release gate is a lab campaign, not a release gate.** It demands 18
   records. The records need fault injection, repeated cancellation cycles, a
   model download, TLS key material, Windows Sandbox runs, and the live GitHub
   API. A clean runner cannot produce them.
6. **The rules lock each other.** Tags are immutable. A tag must be on `main`.
   Promotion accepts only a push-event run at the tag commit. The owner
   forbids a new version and a tag move. With all of these rules in force,
   `v0.6.1` cannot ship.
7. **The team diagnosed the wrong thing.** `TODO.md:131` says "Root cause is
   environmental, proven". The same line records a product defect that the
   team reproduced: a second concurrent call returns `kind: busy`. The
   harness also discards the error kind that names the cause.
8. **The tracker contradicts the code.** `TODO.md:133` says that promotion
   "needs no change". `release-promote.yml:90`, `:92`, and `:100` reject the
   run that the owner chose as the final producer.

The repair is not another verifier. The repair is deletion. Cut the release
gate to the checks that protect a user. Fix three product defects. Ship from
`main`.

---

## 2. Why no update has shipped

Each step below is verified against run records or source.

1. **`v0.6.0` is dead.** On 2026-09-14, the tag `v0.6.0` was pushed at 6
   commits in about 4 hours: `938b995`, `dbffa64`, `c4760d5`, `383fecb`, `1cfa029`,
   and `27349f9`. All 6 runs failed. The tag is now immutable at `27349f9`.
   The fixes landed after it.
2. **The push path for `v0.6.1` is dead.** Run `35731266321` (tag `v0.6.1`
   at `4b31431`) failed 6 times:
   - Attempt 1 failed the main-ancestry gate: `4b31431` was not yet on
     `main`.
   - Attempts 2 to 5 failed the packaged matrix on 3 runtime-catalog IPC
     checks.
   - Attempt 6 failed manifest assembly: 12 of the 18 records were missing
     from the tag tree. The records were committed after the tag. A re-run
     uses the workflow file of the tag, so it fails again.
3. **The dispatch path stops at step #19.** Runs `35897526108` and
   `35901604360` (`workflow_dispatch` from `main` at `0bd39fb`) failed at
   step #19, "Verify the packaged executable", after about 6.5 minutes. Step
   #18 (the build) passed. Steps #24 to #27 (Sandbox lifecycle, lab legs, and
   witnesses) have never run in CI. The team merged 242 lines of release
   workflow that no runner has executed.
4. **The lab step fails next.** `release.yml:386` and
   `scripts/g05_mt06_cycles.mjs:34` read `.hermes-0.6/g05-tls/*`. This
   directory exists only on the developer's disk. Only that developer's
   `.git/info/exclude` names it. The runner workspace
   `C:/actions-runner-localmotive/_work/localmotive/localmotive` does not
   contain it. `release-gates.test.mjs:3158` already notes that the path
   "cannot validate in a clone".
5. **Promotion refuses the result.** Assume that every verify step passes.
   - `release-promote.yml:90` requires `event == push`.
   - `release-promote.yml:92` requires `head_sha` to equal the tag peel
     `4b31431`. The dispatch head is `0bd39fb`.
   - `release-promote.yml:100` requires a green `pr-check` or `check` on the
     peel. `4b31431` has neither.
6. **The three matrix failures have one cause: no runtime catalog.**
   - `runtime.rs:2808-2887` (`fetch_catalog`) always calls `api.github.com`
     without a token. GitHub limits unauthenticated calls to 60 requests per
     hour for each IP address.
   - A fresh profile has no cache. The matrix always uses a fresh profile.
   - `src-tauri/approved_runtimes.json` already pins release `b10816`. For
     each of its 13 assets, it pins the name, the URL, the byte count, and the
     SHA-256. The live call adds no trust.
   - `runtime_service.rs:38-53` and `:84-90` return `kind: Busy` to a second
     concurrent caller. The UI calls `loadRuntimeSetup()` at startup
     (`App.tsx:1418`). The harness calls the same command.
   - `verify_041.mjs:463` replaces every catalog error with the label
     `"terminal-error"`. It discards `catalogError.kind`. The other two
     failing checks record `"[object Object]"` as the reason.
7. **The failure report is wrong.** In run `35901604360`,
   `check_package_outputs.ps1` printed "Packaging failed before producing".
   Packaging passed at step #18.
8. **The staging directory is the capture dump.** `release.yml:182` creates
   `artifacts/` in a checkout that already tracks 5,494 files there.
   `release.yml:494` and `:507` upload `artifacts/*` and `artifacts/**`. The
   diagnostics artifact of run `35901604360` holds 5,495 files (26,370,566
   bytes). Only 1 of those files is new.

Result: under the current rules, no retry can publish `v0.6.1`.

---

## 3. Owner decisions

Update, 2026-09-24: the owner gave the dev team release authority
(`AGENTS.md`, "CI and publication"; `HANDOVER.md`). These decisions now belong
to the team. D1 is Option A, and D2 is adopted (`TODO.md` section 3).

At review time, the team could not make these decisions, and each decision
blocked the next release.

**D1 — Exit the deadlock.** Select one option.

- **Option A (recommended): ship the next version from `main`.** First, fix
  RT-05, RT-06, and FE-02. Users see these defects. Then bump the version
  once, tag a commit on `main`, and let the push-event run qualify that tag.
  This obeys the four-identity rule in `AGENTS.md`: the bump comes with
  product changes, not with paperwork. `v0.6.1` stays unpublished, and
  `CHANGELOG.md` records it as not published.
- **Option B: keep `v0.6.1`.** Change `release-promote.yml` so that it
  accepts a dispatch run that is bound by run ID and by recorded source SHA.
  Remove the `.hermes-0.6` dependency. Make the 18-record campaign pass on the
  runner. This option costs the most. It still needs the product fixes to
  pass step #19.
- **Option C: no decision.** Then 0.6.x does not ship.

**D2 — Reduce the release gate** to the checks in section 4. Move the lab
campaign to `hardware-qualify.yml` as a check that does not block a release.

**D3 — Remove `artifacts/` from the tree.** A rewrite of public history is a
separate decision.

**D4 — Decide the `v0.4.0` corrective note**
(`release-evidence/0.4.1/v0.4.0-corrective-note.md`). Decided 2026-09-24:
deleted, not published.

---

## 4. The release gate to keep

Update, 2026-09-24: `AGENTS.md`, "CI and publication", adopts this list as
the release gate.

Keep these checks. Remove all other checks from the release gate.

1. `npm run check`, `npm audit --audit-level=moderate`, the four workflow
   verifiers, and `scripts/tests/verify_cleanup_matrix.ps1`.
2. `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`,
   and `cargo test --locked`.
3. `npm run tauri build` from the tag peel.
4. The packaged matrix, with fixtures only and no live network.
5. Installer payload identity for the NSIS and MSI files.
6. One lifecycle leg in Windows Sandbox: clean install, upgrade from the last
   published version with user-data preservation, and uninstall.
7. Checksums, the SBOM, and a retained candidate bundle.

Promotion then publishes the same bytes, as it does now.

---

## 5. Cut list

Delete each item. Do not improve it.

| Item | Evidence | Action |
|---|---|---|
| 18-record qualification manifest | `build_qualification_manifest.mjs`, `verify_qualification_manifest.mjs`, `release.yml:228-479` (REL-12) | Replace with section 4. |
| Lab, witness, and fault steps in `release.yml` | Never executed in CI; depend on `.hermes-0.6` (REL-04) | Move to `hardware-qualify.yml`, non-blocking. |
| Campaign scripts (`g05_*`, `verify_s2x_*`, `verify_060_catalog.mjs`, and similar) | LAB-05; `g05_dc01.mjs` has no caller (FE-03) | Keep one packaged-matrix driver. Delete the others. |
| Three CDP clients | LAB-03 | Keep `scripts/lib/cdp_client.mjs` only. |
| Source-text tests | `lib.rs:17-37` test-only `ALL_SOURCES` with 13 uses; prose pins in `release-gates.test.mjs` (3,230 lines, 166 tests) | Replace with behavior tests, or delete. |
| Audit-ticket labels in source | 399 labels in `src-tauri/src/*.rs`; modules such as `ipc01_startup_tests` | Delete the labels. Name each test by its behavior. |
| Live GitHub API call for the pinned runtime | `runtime.rs:2808-2887` (RT-05) | Build the catalog from `approved_runtimes.json`. |
| Legacy benchmark path | MT-09, FE-05 | Keep one benchmark system. |
| Second catalog validator | LAB-06 | Keep one schema source. |
| `artifacts/` | 5,494 tracked files, 33 browser profiles (DOC-01, DOC-02, REL-05) | Remove (D3). |
| Second tracker, reports, closed trackers, copied upstream docs | DOC-03; section 11 | Delete (section 11). |

---

## 6. Product defects to fix first

All rows are lead-verified.

| ID | Severity | Defect | Location |
|---|---|---|---|
| RT-05 | Critical | The runtime catalog needs a live, unauthenticated `api.github.com` call. A fresh or offline profile gets no catalog. | `runtime.rs:2808-2887` |
| RT-06 | High | A second concurrent catalog reader gets `kind: Busy` instead of the shared result. | `runtime_service.rs:38-53`, `:84-90` |
| FE-02 | High | The catalog query sends `pipeline_tag`, `fit_per_mille`, and `budget_bytes`. Rust expects camelCase and ignores the three keys. The pipeline filter and the hardware-fit filter have no effect. | `model.ts:1771-1774`; `catalog.rs:771`; `App.tsx:1360`, `:1365-1366` |
| MT-10 | High | The cloud brief sends adapter IDs, compatibility IDs, and the NVIDIA `physicalId` UUID, also in minimal mode. The disclosure does not list them. | `tune.rs:236-240`, `:326-345`; `runtime.rs:360-382` |
| CORE-04 | High | Rust does not make sure that `-m` gets the first shard. Only the frontend default does. | `core.rs:987`; `lib.rs:443-449`; `model.ts:1479` |
| RT-02 | High | Install deletes the rollback copy before the final verification. | `runtime.rs:4406-4459`, `:4569-4588` |
| LAB-04 | High | Harness scripts stop every `llama-server` and `localmotive` process by name. | `g05_vitems_c.mjs:109`, `:166` |
| LAB-01 | High | The harness discards the structured error kind and records `"[object Object]"`. | `verify_041.mjs:463` |
| DOC-04 | High | `README.md` describes 0.6.0 as shipped. The latest published release is `v0.5.0`. | `README.md:17`, `:331`, `:369`, `:376` |

---

## 7. Severity rubric and lead corrections

Rubric:

- **Critical:** blocks a release now, exposes a user secret outside the
  user's account, or crosses a privilege boundary.
- **High:** breaks behavior that a user sees, or a product guarantee in normal
  use. Also: breaks a non-negotiable rule in `AGENTS.md` with a realistic
  trigger.
- **Medium:** needs an unusual trigger, such as a same-user race, a hostile
  local file, or a debug build. Also: a defect that multiplies maintenance
  cost.
- **Low:** dead code, naming, or cosmetic defects.

Same-user threat model: Localmotive runs as the signed-in user. The per-user
install and the managed runtime are in that user's `%LOCALAPPDATA%`. A
process that runs as the same user can already replace those binaries. A race
that needs such a process is Medium, not Critical.

| ID | Reviewer | Lead | Reason |
|---|---|---|---|
| RT-01 | Critical | Medium | Same-user race in the user's own `%LOCALAPPDATA%`. Fix it, but it crosses no privilege boundary. |
| RT-02 | Critical | High | The early deletion of the rollback copy is real and breaks the atomic-publication rule. The race part needs a same-user writer. |
| RT-03 | Critical | Medium | Same-user race between the verification and the execution. |
| CORE-01 | High | Medium | The catalog override needs same-user control of the environment. Keep it for the packaged matrix, bind it to the isolated root, and show a verification-mode banner. |
| CORE-02 | High | Low | React StrictMode runs effects twice only in development builds. In the packaged app, the UI startup call (`App.tsx:1418`) collides with the harness call. That collision is RT-06. |
| CORE-06 | High | Medium | `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` is standard WebView2 behavior. It needs same-user control of the environment. |
| FE-01 | Critical | Medium | `App.tsx:957` and `:762` clear the drafts after a successful save. A draft stays only while the user types and after a failed save. |
| FE-03 | High | Low | `g05_dc01.mjs` has no caller in `package.json`, the workflows, or the scripts. It is dead code, not a gate. |
| DL-02 | High | Medium | The release profile does not set `overflow-checks`. Release builds wrap and report a false mismatch. Only debug builds panic. |
| PROC-01 | High | Medium | The window between spawn and job assignment exists (`proc.rs:293-306`). `llama-server` does not start child processes at launch. |
| REL-08 | High | Medium | The comments at `verify_release_promotion.mjs:8` and `:43` promise an extra-file check that does not exist. Publication uploads only the promised names. |
| DOC-01 | Critical | High | Count-only scan: 33 profiles. `Cookies` has 0 rows. `Login Data` has 0 rows. `Web Data` has 0 autofill rows. `History` has 33 rows, all local. Each `Local State` holds a DPAPI-wrapped key that only the owner's Windows account can open. No usable secret was found. Remove the profiles for hygiene and size. |

---

## 8. Local gate results at `9b09857`

The lead reviewer ran these commands on 2026-09-24 on the development host.

| Gate | Result |
|---|---|
| `npm run check` | PASS. Exit 0 in 30 s. `node --test`: 276 of 276. Vitest: 282 of 282 in 13 files. Catalog, branding, research, qualification, and icon verifiers pass. The Vite build passes. |
| `npm audit --audit-level=moderate` | PASS. 0 vulnerabilities. |
| `verify_versions`, `verify_workflow_pins`, `verify_workflow_gates`, `verify_workflow_syntax` | PASS. The first run failed with `stdin is not a tty`, a tooling failure of the `node` wrapper under Git Bash. The rerun with `node.exe` exited 0 for all four. |
| `pwsh -NoProfile -File scripts/tests/verify_cleanup_matrix.ps1` | PASS. 12 passed, 0 failed. |
| `cargo fmt --check` | PASS. |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS. |
| `RUSTDOCFLAGS='-D warnings' cargo test --locked` | PASS. 628 passed, 0 failed, 7 ignored. |
| `npm run tauri build` | UNKNOWN locally. Not run for this review. CI step #18 of run `35901604360` built `0bd39fb` successfully. |

The green gates prove less than they suggest. They did not catch FE-02, a
broken IPC contract in a shipped feature. Many tests assert source text and
document wording instead of behavior (section 5).

---

## 9. Measurements

| Measure | Value |
|---|---|
| Tracked files | 6,005 |
| Tracked files in `artifacts/` | 5,494 (91.5%) |
| Tracked files in `release-evidence/` | 247 |
| Tracked files in `scripts/` | 96 |
| Rust source | 48,302 lines in 26 files; 63 `#[tauri::command]` functions |
| Rust tests | 635 (628 pass, 7 ignored) |
| Frontend source | 12,468 lines in 31 files; `App.css` 690 lines |
| Script source (`.mjs`, `.ps1`, `.py`, `.sh`) | 19,726 lines |
| `scripts/tests/release-gates.test.mjs` | 3,230 lines, 166 tests |
| Workflows | 1,268 lines; `release.yml` 517 lines |
| Tracked Markdown outside `artifacts/` | 1,795,987 bytes in 35 files |
| Commits since `v0.5.0` | 338 (308 non-merge) |
| Non-merge commits since `v0.5.0` with no product change | 191 of 308 |
| Files changed `v0.5.0..HEAD` | 5,928 (+120,931 / -10,082 lines): `artifacts/` 5,494, `release-evidence/` 234, other 200 |
| `release.yml` runs since 2026-09-10 | 18: 3 success (all on 2026-09-10, `v0.4.1` and `v0.5.0`), 15 failure |
| `release.yml` runs after the `v0.5.0` release | 14: 0 success, 14 failure |
| `release-promote.yml` runs, all time | 0 |
| `ci.yml` runs since 2026-09-10 | 173: 87 success, 74 failure |

---

## 10. Method and limits

- Nine reviewers each received one scope, the project rules, and explicit
  criteria. Each reviewer read the files in its scope and wrote findings with
  `file:line` evidence and short quotes.
- The lead reviewer ran the local gates. The lead reviewer read the release
  run records, the job steps, and the downloaded diagnostics artifact. The
  lead reviewer checked 35 findings against source and applied the rubric in
  section 7.
- The finding index gives each finding one state:
  - `VERIFIED`: the lead reviewer confirmed the claim and its severity.
  - `CORRECTED`: the lead reviewer confirmed the defect and changed its
    severity.
  - `UNVERIFIED`: a reviewer report that the lead reviewer did not check.
- Limits:
  - The documentation reviewer did not read the largest historical files in
    full (the closed trackers, the audit, the copied upstream README, and the
    0.6 reports). Section 11 decides their disposition by role and by
    references, not by full content.
  - The measurement reviewer could not read `research/measuring/`, because
    that directory is absent from this checkout.
  - The lab reviewer did not read all of `catalog/catalog.json`.
  - No packaged build, installer test, or Sandbox run was executed for this
    review.
- The browser-profile scan counted rows only. It opened each SQLite file
  read-only and immutable. It printed no cookie value, credential, URL, or key
  material.

---

## 11. Markdown disposition

This table decides each tracked Markdown file outside `artifacts/`. Git history
keeps each deleted file. The last tree that holds all of them is
`9b098577095ab80ead9bff6e5475456166b340c1`.

| File | Decision | Reason |
|---|---|---|
| `README.md` | Keep | Product front page. Fix DOC-04. |
| `CHANGELOG.md` | Keep | Release notes. `release-promote.yml` reads it. |
| `AGENTS.md` | Keep | Project rules. Its pointer to the earlier audit now points to this review (owner approval on 2026-09-24, `TODO.md` D6). |
| `CONTRIBUTING.md` | Keep | GitHub community file. |
| `SECURITY.md` | Keep | GitHub security policy. The issue templates link it. |
| `TODO.md` | Keep | The one tracker. Rewrite it from this review. |
| `REVIEW.md` | Keep | This review. |
| `catalog/README.md` | Keep | Catalog schema, signing, and curation rules. |
| `docs/DESIGN.md` | Keep | Normative design system. |
| `docs/PRODUCT.md` | Keep | Product schema for the design tools. |
| `docs/RUNTIME_MANAGER.md` | Keep | Runtime contract. |
| `docs/SUPPLY-CHAIN.md` | Keep | Supply-chain posture. |
| `docs/SUPPORT-MATRIX.md` | Keep | Support scope for users. |
| `docs/EVIDENCE-MATRIX.md` | Keep | Evidence by version for users. Merge it into `SUPPORT-MATRIX.md` later. |
| `docs/ARTIFACT-IDENTITY.md` | Keep | Design of model artifact identity. |
| `docs/CATALOG-PROMOTION.md` | Keep | Catalog promotion procedure. |
| `docs/SECURITY-REVIEW-LIMITS.md` | Keep | `SECURITY.md` links it. |
| `release-evidence/0.4.1/v0.4.0-corrective-note.md` | Deleted | Owner decision D4 (2026-09-24): deleted, not published. |
| `release-evidence/0.6.0/history/freeze8-da091a4/README.md` | Deleted | Owner decision D5 (2026-09-24): exception to the frozen-evidence rule for superseded 0.6.0 history. |
| `TODO-0.6-post-076a3eeecbdf.md` | Delete | Second tracker. It breaks the one-tracker rule. |
| `REPORT-0.6.md` | Delete | Snapshot report. Superseded. |
| `agents_feedback.md` | Delete | Agent log. Its open risks move to `TODO.md`. |
| `ideas.md` | Delete | Idea list. Its items move to the `TODO.md` backlog. |
| `docs/RELEASE-REVIEW-0.6.md` | Delete | 0.6 review. Superseded by this review. |
| `docs/localmotive-0.6-followup-review-db548c8.md` | Delete | Follow-up review. Superseded by this review. |
| `docs/QUALIFICATION-MAP-0.6.md` | Delete | Map of the 18-record campaign that section 5 cuts. Its source commit `c0ae538` is stale. |
| `docs/qualification-tests.md` | Delete | Frozen 0.4.1 snapshot. |
| `docs/OPTION_MAP.md` | Delete | Historical index. The selected runtime's `--help` output is authoritative. |
| `docs/LLAMA-SERVER-README.md` | Delete | 107 KB copy of upstream documentation without a pinned commit. Cite upstream instead. |
| `docs/Future_branding.md` | Delete | Record of a completed rename. |
| `docs/history/TODO-0.4.1.md`, `TODO-0.4.md`, `TODO-0.5.md`, `TODO-0.6.md`, `TODO.md` | Delete | Closed trackers. Git history keeps them. |
| `docs/history/localmotive-comprehensive-audit.md` | Delete | Earlier audit. This review supersedes it for current work. Deleted after the owner approved the `AGENTS.md` pointer change (`TODO.md` D6). Git history keeps it. |
| `artifacts/stop-ownership-audit-20260920-0624/HANDOFF.md` | Remove with D3 | Part of the capture dump. Do not remove single files from an evidence dump. |

The deletion also makes two scripts dead: `scripts/check_tracker.mjs` and
`scripts/verify_tracker_links.mjs`. They validate only the deleted trackers.
No gate calls them. Delete them in the same change.

---

## 12. Finding summary by part

| Part | Scope | Prefix | Critical | High | Medium | Low | Total |
|---|---|---|---|---|---|---|---|
| 1 | Release pipeline, CI and qualification tooling | REL | 5 | 7 | 4 | 0 | 16 |
| 2 | Lab harness, verifier and catalog scripts | LAB | 0 | 6 | 6 | 1 | 13 |
| 3 | Repository hygiene, documentation and process | DOC | 0 | 7 | 7 | 1 | 15 |
| 4 | Rust runtime management | RT | 1 | 5 | 7 | 2 | 15 |
| 5 | Rust command surface and core model | CORE | 0 | 3 | 8 | 1 | 12 |
| 6 | Rust process, transport and health | PROC | 0 | 7 | 14 | 1 | 22 |
| 7 | Rust catalog, downloads, file formats and cloud | DL | 0 | 3 | 13 | 0 | 16 |
| 8 | Rust measurement, evidence and tuning | MT | 0 | 8 | 7 | 0 | 15 |
| 9 | Frontend | FE | 0 | 4 | 6 | 3 | 13 |
| | **All** | | 6 | 50 | 72 | 9 | 137 |

Counts use the lead severity. Reviewer severities before correction: Critical 11, High 55, Medium 64, Low 7.
Lead checks: 23 VERIFIED, 12 CORRECTED, 102 UNVERIFIED.

## 13. Finding index

Sorted by lead severity, then by part. Each detailed finding follows in its part.

| ID | Severity | Lead check | Finding | Location |
|---|---|---|---|---|
| REL-01 | Critical | VERIFIED | Promotion rejects the owner-mandated dispatch producer | `.github/workflows/release.yml:5-14`; `.github/workflows/release-promote.yml:85-94`; runs `35897526108`, `35901604360`. |
| REL-02 | Critical | VERIFIED | Immutable tag plus main ancestry prevents repair of a used tag | `.github/workflows/release.yml:43-48`; `.github/workflows/release-promote.yml:75-83`; `AGENTS.md:153-166`; runs `34803387581`–`3481921972... |
| REL-03 | Critical | VERIFIED | Evidence provenance rules contradict the tag-tree gate | `AGENTS.md:160-171`; `.github/workflows/release.yml:196-208`, `481-497`; `scripts/tests/release-gates.test.mjs:3155-3225`. |
| REL-04 | Critical | VERIFIED | Clean checkout lacks the lab TLS/API-key material | `.github/workflows/release.yml:383-445`; `scripts/g05_mt06_cycles.mjs:30-34`; read-only `git ls-files` result: zero `.hermes-0.6` entries. |
| REL-05 | Critical | VERIFIED | Tracked `artifacts/` is uploaded as release evidence and diagnostics | `.github/workflows/release.yml:179-190`, `489-510`; read-only `git ls-files artifacts/**` result: 5,494 tracked files; runs `35897526108`... |
| RT-05 | Critical | VERIFIED | Catalog refresh is a network hard dependency for the app and release gate | `src-tauri/src/runtime.rs:15`, `2500-2527`, `2808-2886`; `src-tauri/src/runtime_service.rs:38-65`, `80-114`; `scripts/verify_041.mjs:447-... |
| REL-06 | High | VERIFIED | Live unauthenticated GitHub API is a release gate | `scripts/verify_041.mjs:469-519`; `src-tauri/src/runtime.rs:15`; runs `35897526108`, `35901604360`. |
| REL-07 | High | VERIFIED | Manifest binds the tag workflow, not the dispatch workflow | `.github/workflows/release.yml:35`, `107-110`; `scripts/build_qualification_manifest.mjs:140-149`; `scripts/verify_release_promotion.mjs:... |
| REL-09 | High | VERIFIED | Failure report mislabels packaged verification as packaging failure | `.github/workflows/release.yml:171-190`, `514-517`; `scripts/check_package_outputs.ps1:19-25`; runs `35897526108`, `35901604360`. |
| REL-10 | High | VERIFIED | `Stop-LabApp` kills only the parent process | `.github/workflows/release.yml:271-295`. |
| REL-11 | High | UNVERIFIED | Generated lab JavaScript is hardcoded and DOM-text fragile | `.github/workflows/release.yml:271-289`, `329-382`, `421-449`. |
| REL-12 | High | VERIFIED | Eighteen mandatory records turn a release into a fault-injection campaign | `scripts/build_qualification_manifest.mjs:29-48`; `scripts/verify_qualification_manifest.mjs:58-83`, `643-655`; `.github/workflows/releas... |
| REL-13 | High | VERIFIED | A 360-minute release can be cancelled by its own retry | `.github/workflows/release.yml:18-20`, `91-97`. |
| LAB-01 | High | VERIFIED | Structured runtime-catalog errors are destroyed as `[object Object]` | `scripts/verify_041.mjs:96,269-273,471-519`. |
| LAB-02 | High | VERIFIED | A live GitHub outage is reported as product failure | `scripts/verify_041.mjs:447-519`; `src-tauri/src/runtime.rs:15,1949-1952,2822-2830`. |
| LAB-03 | High | UNVERIFIED | Three independent CDP clients guarantee behavior drift | `scripts/lib/cdp_client.mjs:32-149`; `scripts/verify_041.mjs:139-237`; `scripts/drive_console_check.mjs:20-60`. |
| LAB-04 | High | VERIFIED | Unscoped process-name kills violate the harness safety rule | `scripts/g05_vitems_c.mjs:42,94,109,166`; `scripts/g05_fe16.mjs:41-45,152-157`; `scripts/g05_stop_supervision.mjs:12-25`; `scripts/g05_ta... |
| LAB-05 | High | UNVERIFIED | Operational coverage is scattered across dead one-off drivers and source-only tests | reference table; source-only assertions `scripts/tests/release-gates.test.mjs:2021-2031`; active direct lab calls `.github/workflows/rele... |
| LAB-06 | High | UNVERIFIED | Catalog validation is a two-language copy with an unchecked output-shape gap | `scripts/lib/catalog_schema.mjs:1-8,63-160`; `src-tauri/src/catalog.rs:362-428`; `scripts/build_catalog.mjs:230-248`; `catalog/catalog.js... |
| DOC-01 | High (was Critical) | CORRECTED | Tracked browser-profile databases and credential-store filenames are a privacy exposure | `artifacts/**` name-only inventory; `.gitignore:25-29`; `AGENTS.md:29-35`. |
| DOC-02 | High | VERIFIED | `artifacts/` is repository-scale raw capture, not release evidence | `artifacts/**`; `.gitignore:1-16`; commit `6099c6f` (all named audit trees). |
| DOC-03 | High | VERIFIED | The one-tracker rule is contradicted by active and “authoritative” pointers | `AGENTS.md:183-189`; `TODO.md:3-6`; `TODO-0.6-post-076a3eeecbdf.md:9-19`; `docs/RELEASE-REVIEW-0.6.md:3-8`; `docs/EVIDENCE-MATRIX.md:5-8`... |
| DOC-04 | High | VERIFIED | Public documentation presents unpublished 0.6.x as a release | `CHANGELOG.md:3-16`; `README.md:15-18,352-369`; `TODO.md:115-139`; `package.json:4`. |
| DOC-05 | High | UNVERIFIED | Three support authorities disagree about 0.6 evidence | `README.md:327-341`; `docs/EVIDENCE-MATRIX.md:3-8,21-28`; `docs/SUPPORT-MATRIX.md:18-33`. |
| DOC-06 | High | VERIFIED | Immutable tags plus post-tag digest-bound evidence created the release deadlock | `AGENTS.md:153-174`; `TODO.md:129-139`; `docs/QUALIFICATION-MAP-0.6.md:51-60`. |
| DOC-07 | High | UNVERIFIED | The release ledger has expanded into a 111-package, 663-checkbox control system | `REPORT-0.6.md:12-20`; `docs/history/TODO-0.6.md:13-25`; `TODO.md:30-169`. |
| RT-02 | High (was Critical) | CORRECTED | Staging verification is not bound to publication and rollback is destroyed too early | `src-tauri/src/runtime.rs:4569-4588` and `4406-4459`. |
| RT-04 | High | UNVERIFIED | The execution lease pins expected files but not the directory inventory | `src-tauri/src/runtime.rs:4139-4160`, `4090-4126`, and managed health `4783-4791`. |
| RT-06 | High | VERIFIED | Concurrent catalog requests fail fast instead of joining or reusing work | `src-tauri/src/lib.rs:103-120`; `src-tauri/src/runtime_service.rs:38-53` and `84-90`. |
| RT-07 | High | UNVERIFIED | Managed-runtime discovery crosses reparse ancestors and has no scan bound | `src-tauri/src/runtime.rs:992-996`, `1102-1164`; caller `src-tauri/src/runtime_service.rs:17-27`. |
| RT-10 | High | UNVERIFIED | Path validation is inconsistent and sometimes happens after the first write | `src-tauri/src/runtime.rs:2679-2696`, `4489-4498`, `4630-4636`, `4680-4689`. |
| CORE-03 | High | UNVERIFIED | An abandoned tuning command can release ownership while its blocking worker keeps running | `src-tauri/src/tune_service.rs:244-266,361-371`; `src-tauri/src/server_service.rs:26-32`; `src-tauri/src/lib.rs:250-269,4217-4234`. |
| CORE-04 | High | VERIFIED | The launch path discovers `first_shard` but does not enforce it for `-m` | `src-tauri/src/core.rs:430-463,528-539,982-987`; `src-tauri/src/artifact.rs:488-504`; `src-tauri/src/lib.rs:621-626`. |
| CORE-05 | High | UNVERIFIED | Several IPC payloads are typed but not bounded before allocation or expensive work | `src-tauri/src/lib.rs:1479-1502,1557-1569,1574-1583,1639-1669,1854-1882`; `src-tauri/src/core.rs:642,1272-1274,1432-1456`; `src-tauri/src... |
| PROC-02 | High | UNVERIFIED | Failed process cleanup can turn a bounded operation into an unbounded pipe-reader join | `src-tauri/src/proc.rs:35–50, 423–480` |
| PROC-03 | High | UNVERIFIED | Timeout and cancellation return success-shaped errors even when termination failed | `src-tauri/src/proc.rs:357–391, 427–471` |
| PROC-04 | High | UNVERIFIED | Startup can leave the `starting` slot permanently occupied | `src-tauri/src/server_service.rs:104–122` |
| PROC-05 | High | UNVERIFIED | Health readiness uses a redirect-following transport path | `src-tauri/src/health.rs:1082–1155`; comparison: `src-tauri/src/local_client.rs:509–517` |
| PROC-06 | High | UNVERIFIED | Health cancellation abandons an untracked nested request worker | `src-tauri/src/health.rs:659–713, 715–747` |
| PROC-07 | High | UNVERIFIED | Transport-file reads do not enforce reparse/no-follow semantics at the read site | `src-tauri/src/local_client.rs:854–876`; launch validation comparison: `src-tauri/src/lib.rs:722–739` |
| PROC-08 | High | UNVERIFIED | Secondary log handles and failure evidence follow a replaced path | `src-tauri/src/log_sink.rs:100–111, 203–217` |
| DL-01 | High | UNVERIFIED | Mutable SQLite override rows are accepted as download authority without read-time revalidation | `src-tauri/src/lib.rs:2010-2044`; `src-tauri/src/catalog_db.rs:727-777` |
| DL-03 | High | UNVERIFIED | Reparse/symlink validation is separated from the file open | `src-tauri/src/artifact.rs:277-302`; `src-tauri/src/gguf.rs:649-657` |
| DL-04 | High | UNVERIFIED | OAuth key exchange buffers an unbounded response | `src-tauri/src/cloud.rs:596-616` |
| MT-01 | High | UNVERIFIED | Heuristic in-sample repeat is presented as confirmation | `src-tauri/src/tune.rs:976-982, 1551-1649`; `src/screens/TuneScreen.tsx:344-349`; `src-tauri/src/measurement.rs:14, 19-28`. |
| MT-03 | High | UNVERIFIED | Warmup-only failure cannot produce the promised saved manifest | `src-tauri/src/measurement.rs:342-390`; `src-tauri/src/evidence.rs:756-823`; `src-tauri/src/measurement_service.rs:537-550`; `src/V03Evid... |
| MT-05 | High | UNVERIFIED | Cold benchmark removes the managed server and does not restore it | `src-tauri/src/measurement_service.rs:613-688, 689-733`; `src/V03EvidencePanel.tsx:591-598`. |
| MT-06 | High | UNVERIFIED | Worker-drain failure can reserve the benchmark slot forever | `src-tauri/src/measurement_service.rs:28-42, 709-723`; `src-tauri/src/local_client.rs:677-737`. |
| MT-07 | High | UNVERIFIED | Tuner cancellation releases its operation while its legacy HTTP worker may continue | `src-tauri/src/tune_service.rs:25-101, 228-365`; `src-tauri/src/core.rs:2325-2345`; `src-tauri/src/local_client.rs:677-737`. |
| MT-08 | High | UNVERIFIED | Unknown execution identity is recorded but ignored by replay | `src-tauri/src/measurement_service.rs:242-268, 286-317, 792-807`; `src-tauri/src/calibration.rs:81-84`; `src-tauri/src/evidence.rs:637-644`. |
| MT-09 | High | UNVERIFIED | Shipped legacy benchmark is weaker and duplicates v2 | `src-tauri/src/measurement_service.rs:52-129, 603-734`; `src-tauri/src/core.rs:2240-2346`; `src/screens/BenchmarkScreen.tsx:45-76`; `src/... |
| MT-10 | High | VERIFIED | Cloud wire payload is broader than the displayed disclosure | `src-tauri/src/tune.rs:218-256, 326-356`; `src-tauri/src/runtime.rs:161-177, 363-380`; `src-tauri/src/gguf.rs:102-132`; `src/screens/Tune... |
| FE-02 | High | VERIFIED | Catalog query payload uses the wrong wire keys | `src/model.ts:1762-1775`; `src/App.tsx:1354-1367`; `src-tauri/src/catalog.rs:770-798`. |
| FE-04 | High | UNVERIFIED | `App` is a controller, persistence service, async coordinator, and shell renderer in one component | `src/App.tsx:87-164,170-359,362-755,757-1028,1030-1239,1243-1501,1503-1858`. |
| FE-05 | High | UNVERIFIED | Legacy benchmark and “v2” evidence are two active product systems, not one migrated feature | `src/App.tsx:173-177,259,1301-1333`; `src/screens/BenchmarkScreen.tsx:43-86`; `src/V03EvidencePanel.tsx:126,572-580`; `src/App.staleRespo... |
| FE-06 | High | UNVERIFIED | The presentation boundary is contradicted by a direct screen IPC call and untyped adapter calls | `src/screens/InventoryScreen.tsx:1-13,51-58`; `src/evidence-adapter.ts:18-43`. |
| REL-08 | Medium (was High) | CORRECTED | Promotion silently accepts extra files | `scripts/verify_release_promotion.mjs:42-52`, `83-106`; `.github/workflows/release-promote.yml:226-233`. |
| REL-14 | Medium | UNVERIFIED | CI repeats heavy release gates and a build | `.github/workflows/ci.yml:53-88`, `110-141`; `.github/workflows/release.yml:133-178`; `.github/workflow-gates.json:20-88`. |
| REL-15 | Medium | UNVERIFIED | `release-gates.test.mjs` is heavily coupled to workflow spelling | `scripts/tests/release-gates.test.mjs:210-236`, `1452-1660`, `3155-3225`. |
| REL-16 | Medium | UNVERIFIED | Hardware workflow summary contradicts actual release runner policy | `.github/workflows/hardware-qualify.yml:100-108`; `.github/workflows/release.yml:22-24`, `67-69`, `91-94`. |
| LAB-07 | Medium | UNVERIFIED | Fixed sleeps are synchronization protocol | inventory below; examples `g05_cancellation.mjs:41,45`, `g05_mt06_cycles.mjs:302,307`, `verify_a11y.mjs:39`, `verify_csp.mjs:39`. |
| LAB-08 | Medium | UNVERIFIED | Visible button text is the selector API | selector inventory below, especially `verify_041.mjs:252,427,531`, `g05_vitems_c.mjs:19-29`, `verify_060_catalog.mjs:226-240`. |
| LAB-09 | Medium | UNVERIFIED | Hard-coded ports and path defaults make runs collide or test the wrong machine | `drive_console_check.mjs:4,104,116`; `g05_mt06_cycles.mjs:104,132`; `g05_tamper_dll.mjs:17`; `g05_vitems_c.mjs:94`; `verify_a11y.mjs:17`;... |
| LAB-10 | Medium | UNVERIFIED | Campaign and version names have become architecture | headers/output paths in `scripts/g05_*.mjs`; `verify_041.mjs:19`; `verify_060_catalog.mjs:1-17`; `measure_s25_packaged.mjs:1-7`; S-23/S-2... |
| LAB-11 | Medium | UNVERIFIED | `dryrun_catalog.mjs` duplicates a stale builder path | `dryrun_catalog.mjs:1-18,37-91`; `build_catalog.mjs:4-7,17,174-206`; `catalog/README.md:123-131`. |
| LAB-12 | Medium | UNVERIFIED | Catalog publication rewrites 630 KB with tie-dependent ordering | `build_catalog.mjs:164,215-248,259-264`; `catalog/catalog.json`. |
| DOC-08 | Medium | UNVERIFIED | Stale and missing path references make the documentation graph non-reproducible | `CHANGELOG.md:16`; `docs/EVIDENCE-MATRIX.md:7`; `REPORT-0.6.md:6,23`; `docs/qualification-tests.md:3,201`; `.gitattributes:25`; `TODO.md:... |
| DOC-09 | Medium | UNVERIFIED | Contributor and policy check lists duplicate the same work | `AGENTS.md:73-93`; `CONTRIBUTING.md:13-32`; `package.json:10,18`. |
| DOC-10 | Medium | UNVERIFIED | README feature availability exceeds the evidence matrix for providers and backends | `README.md:121-125,185-214`; `docs/PRODUCT.md:15-35`; `docs/SUPPORT-MATRIX.md:26-35,48-56`. |
| DOC-11 | Medium | UNVERIFIED | Frozen qualification and runtime documents are still on the engineer’s current path | `docs/qualification-tests.md:1-3,18-26`; `docs/RUNTIME_MANAGER.md:1-3,54-66`; `docs/SUPPORT-MATRIX.md:8-10`. |
| DOC-12 | Medium | UNVERIFIED | `TODO.md` is not a usable next-action tracker | `TODO.md:30-41,115-169,227`; `TODO-0.6-post-076a3eeecbdf.md:48-50`. |
| DOC-13 | Medium | UNVERIFIED | Ignored research is cited as public release evidence | `.gitignore:15-16`; `CHANGELOG.md:266-271,279`; `docs/history/TODO-0.4.1.md:100-117`. |
| DOC-14 | Medium | UNVERIFIED | Deleted signing paths remain in repository metadata | `.gitattributes:21-25`; `TODO.md:18-21`; `docs/RUNTIME_MANAGER.md:104-112`. |
| RT-01 | Medium (was Critical) | CORRECTED | The install record is written through an attacker-replaceable path | `src-tauri/src/runtime.rs:3677-3703`, called by `src-tauri/src/runtime.rs:4561-4574`. |
| RT-03 | Medium (was Critical) | CORRECTED | Managed probes verify, release the check, and then execute without an identity lease | `src-tauri/src/core.rs:1735-1748`, `1780-1787`, `2202-2211`; `src-tauri/src/runtime.rs:4217-4228`; public callers `src-tauri/src/lib.rs:1... |
| RT-08 | Medium | UNVERIFIED | Production GPU preflight is structurally forced to Unknown | `src-tauri/src/lib.rs:1735-1764`; `src-tauri/src/preflight.rs:31-33` and `544-553`. |
| RT-09 | Medium | UNVERIFIED | Help capability parsing is exact only after a lossy, substring-based discovery step | `src-tauri/src/core.rs:1475-1495`, `1577-1601`, `1672-1718`; generated flags `982-1005` and `1093-1097`. |
| RT-11 | Medium | UNVERIFIED | Runtime service commands panic on poisoned state locks | `src-tauri/src/runtime_service.rs:130-146`, `150-157`, `170-208`, `263-271`. |
| RT-12 | Medium | UNVERIFIED | Security tests assert source spelling instead of exercising the trust boundary | `src-tauri/src/runtime.rs:5130-5166`; `src-tauri/src/lib.rs:2352-2387` and `3017-3041`. |
| RT-14 | Medium | UNVERIFIED | Version-pinned data and tests are coupled to every upstream bump | `src-tauri/approved_runtimes.json:3-10`; `src-tauri/runtime-content-manifests/cpu.json:3-11` and corresponding tops of all seven manifest... |
| CORE-01 | Medium (was High) | CORRECTED | Production environment variables can replace the signed catalog authority | `src-tauri/src/lib.rs:2253-2257`; `src-tauri/src/catalog.rs:28-68,82-101`; `src-tauri/src/download.rs:345-370`. |
| CORE-06 | Medium (was High) | CORRECTED | Release WebView remote debugging is enabled by inherited environment | `src-tauri/tauri.conf.json:22-24`; `src-tauri/capabilities/default.json:8-48`; `src-tauri/src/main.rs:1-6`; `scripts/verify_packaged_impl... |
| CORE-07 | Medium | UNVERIFIED | `LOCALMOTIVE_SKIP_HARDWARE_PROBE` is a release behavior switch, not a test-only hook | `src-tauri/src/runtime.rs:1336-1355` (called by `runtime_service.rs:17-24,91-97`); release environment hook context also appears in `src-... |
| CORE-08 | Medium | UNVERIFIED | Capability derivation is real-help-based at launch but the parser and preview are weaker than the contract | `src-tauri/src/core.rs:1460-1495,1577-1670`; `src-tauri/src/lib.rs:1789-1804`. |
| CORE-09 | Medium | UNVERIFIED | The capability file is not minimal by inspection | `src-tauri/capabilities/default.json:8-48`; `src-tauri/tauri.conf.json:22-24`; `src-tauri/.gitignore:5-7`. |
| CORE-10 | Medium | UNVERIFIED | Registered dead commands and duplicated read paths enlarge the IPC surface | registration `src-tauri/src/lib.rs:2262-2324`; legacy commands `lib.rs:1416-1502`; cancellation `lib.rs:1518-1555`; duplicate runtime set... |
| CORE-11 | Medium | UNVERIFIED | The two central files mix unrelated production domains with source-text tests and audit-ticket narration | `src-tauri/src/lib.rs:17-36,2352-2369,3638-3644,4315-4352`; `src-tauri/src/core.rs:2348-2360,2952-2971,4170-4352`. |
| CORE-12 | Medium | UNVERIFIED | Raw error conversion and poisoned-lock unwraps lose recovery information or panic the app | `src-tauri/src/lib.rs:1871-1882,1901-1903,2078-2099`; `src-tauri/src/core.rs:2281-2283,441-443,506-513`. |
| PROC-01 | Medium (was High) | CORRECTED | Job membership is assigned after the child is already running | `src-tauri/src/proc.rs:283–320` |
| PROC-09 | Medium | UNVERIFIED | Cleanup PASS treats every failed TCP connect as proof that the port is closed | `src-tauri/src/health.rs:1323–1349` |
| PROC-10 | Medium | UNVERIFIED | The health contract allows PASS without stage-specific evidence and is enforced only in debug builds | `src-tauri/src/health.rs:82–91, 189–213, 230–245` |
| PROC-11 | Medium | UNVERIFIED | CPU device enumeration passes without examining enumeration output | `src-tauri/src/health.rs:450–460, 818–864` |
| PROC-12 | Medium | UNVERIFIED | Completion failures are all mislabeled as timeouts | `src-tauri/src/health.rs:715–747` |
| PROC-13 | Medium | UNVERIFIED | Failed health stages skip cleanup evidence and ignore termination outcomes | `src-tauri/src/health.rs:1104–1123, 1156–1186, 1192–1205, 1236–1247` |
| PROC-14 | Medium | UNVERIFIED | Readiness timing is not a strict 120-second/cancellation bound | `src-tauri/src/health.rs:1082–1105, 1137–1190` |
| PROC-15 | Medium | UNVERIFIED | Security-sensitive certificate/key pairing uses a handwritten 233-line DER walker | `src-tauri/src/local_client.rs:132–364` |
| PROC-16 | Medium | UNVERIFIED | Benchmark and tuning callers detach log-drain handles | `src-tauri/src/measurement_service.rs:458–459`; `src-tauri/src/tune_service.rs:40–41`; server join path `src-tauri/src/server_service.rs:... |
| PROC-17 | Medium | UNVERIFIED | Tuning uses a fixed sleep instead of waiting for port release | `src-tauri/src/tune_service.rs:94–100` |
| PROC-18 | Medium | UNVERIFIED | Stop holds the server mutex during process termination and drain waiting | `src-tauri/src/server_service.rs:249–272` |
| PROC-19 | Medium | UNVERIFIED | Log-retention failures are swallowed, so quotas are advisory | `src-tauri/src/log_sink.rs:228–235`; caller `src-tauri/src/lib.rs:780–782` |
| PROC-20 | Medium | UNVERIFIED | Source-text tests self-match and do not test the safety property | `src-tauri/src/proc.rs:606–613`; `src-tauri/src/health.rs:1551–1556, 1357–1399, 1481–1549` |
| PROC-21 | Medium | UNVERIFIED | Test synchronization and PID-only temp names are still flake-prone | `src-tauri/src/local_client.rs:1125–1260, 1331–1337, 1664–1968, 1882–1928, 2008–2203`; `src-tauri/src/proc.rs:646–816`; `src-tauri/src/he... |
| DL-02 | Medium (was High) | CORRECTED | Hostile GGUF split metadata can overflow and panic | `src-tauri/src/artifact.rs:54-74`; `src-tauri/src/gguf.rs:591-595` |
| DL-05 | Medium | UNVERIFIED | GGUF parser bounds individual values but not container/output overhead tightly enough | `src-tauri/src/gguf.rs:16-34`, `535-605` |
| DL-06 | Medium | UNVERIFIED | SQLite row bound is enforced after materializing rows and all file children | `src-tauri/src/catalog_db.rs:307-373`, `396-405` |
| DL-07 | Medium | UNVERIFIED | User override revisions are length-checked but not path-safe | `src-tauri/src/catalog_db.rs:534-564`; `src-tauri/src/download.rs:394-400` |
| DL-08 | Medium | UNVERIFIED | Signature verification is not over the exact fetched bytes | `src-tauri/src/catalog.rs:603-616`; `catalog/README.md:79-82` |
| DL-09 | Medium | UNVERIFIED | There is no production key-rotation protocol | `src-tauri/src/catalog.rs:20-23`, `603-616` |
| DL-10 | Medium | UNVERIFIED | Cloud clients do not install an explicit redirect policy | `src-tauri/src/cloud.rs:623-645`, `901-916` |
| DL-11 | Medium | UNVERIFIED | Cloud credential and request inputs have no byte bounds | `src-tauri/src/cloud.rs:223-237`, `848-879` |
| DL-12 | Medium | UNVERIFIED | Retry-After policy is duplicated and inconsistent | `src-tauri/src/runtime.rs:2545-2556`; `src-tauri/src/download.rs:925-945`; `src-tauri/src/cloud.rs:716-730` |
| DL-13 | Medium | UNVERIFIED | Handwritten HTTP-date parsing should be replaced | `src-tauri/src/download.rs:948-1005`; `src-tauri/src/cloud.rs:716-729`; `src-tauri/Cargo.toml:36` |
| DL-14 | Medium | UNVERIFIED | Security-critical workflows are oversized and audit narration is mixed into implementation | `src-tauri/src/download.rs:1-4064`; `src-tauri/src/catalog.rs:1-3902` |
| DL-15 | Medium | UNVERIFIED | Cache-root fallback and SQLite open do not enforce the same reparse boundary as downloads | `src-tauri/src/catalog_service.rs:10-21`; `src-tauri/src/catalog_db.rs:48-55` |
| DL-16 | Medium | UNVERIFIED | Weak ETags are treated as resumable identity | `src-tauri/src/download.rs:431-435`, `1520-1535`, `1591-1652` |
| MT-02 | Medium | UNVERIFIED | Invalid optional timing is silently downgraded to missing and derived overflow is unchecked | `src-tauri/src/measurement.rs:272-317`. |
| MT-04 | Medium | UNVERIFIED | Manifest completeness checks are weaker than acquisition checks | `src-tauri/src/evidence.rs:697-753, 756-807, 813-946`; `src-tauri/src/measurement.rs:43-65`. |
| MT-11 | Medium | UNVERIFIED | Minimal disclosure does not remove arbitrary user names | `src-tauri/src/tune.rs:259-323`; `src/screens/TuneScreen.tsx:203-213`. |
| MT-12 | Medium | UNVERIFIED | Cloud advisor request ignores Stop until the blocking call returns | `src-tauri/src/tune.rs:837-840, 1388-1407`; `src-tauri/src/cloud.rs:858-947`; `src-tauri/src/tune_service.rs:374-393`. |
| MT-13 | Medium | UNVERIFIED | Runtime manifest identity has dead fields and audit-ticket narration that contradicts code | `src-tauri/src/evidence.rs:623-652`; `src-tauri/src/measurement_service.rs:405-416, 632-640`; `src-tauri/src/calibration.rs:28-190`; `src... |
| MT-14 | Medium | UNVERIFIED | Tuning and cold benchmark detach log-drain threads | `src-tauri/src/tune_service.rs:40-100`; `src-tauri/src/measurement_service.rs:456-488`; `src-tauri/src/lib.rs:73-75, 745-753`; `src-tauri... |
| MT-15 | Medium | UNVERIFIED | The same product exposes two incompatible standard-deviation conventions | `src-tauri/src/measurement.rs:139-160`; `src-tauri/src/tune.rs:985-997`. |
| FE-01 | Medium (was Critical) | CORRECTED | Credential drafts are retained in React state | `src/App.tsx:273-274,307-308,757-764,949-958`; `src/screens/TuneScreen.tsx:79,270-273`; `src/screens/CatalogScreen.tsx:43,271-276`. |
| FE-07 | Medium | UNVERIFIED | A hooks lint suppression hides a callback dependency in the run-ownership effect | `src/V03EvidencePanel.tsx:227-238`. |
| FE-08 | Medium | UNVERIFIED | Error-boundary reset can fail in the same storage failure mode it is meant to recover | `src/ErrorBoundary.tsx:8-21,49-52`. |
| FE-09 | Medium | UNVERIFIED | The flat design rule still has two box shadows | `src/App.css:43-44`; `docs/DESIGN.md:557-581`. |
| FE-10 | Medium | UNVERIFIED | Type assertions bypass real IPC/domain shapes in production and fixtures | `src/App.tsx:135`; `src/V03EvidencePanel.tsx:73-78`; test fixtures including `src/V03EvidencePanel.calibration.test.tsx:112-124`, `src/V0... |
| FE-11 | Medium | UNVERIFIED | Tests couple to DOM classes/text and use real timers for debounce/async flushing | `src/App.catalog.test.tsx:177-192,386-450,1420-1463`; `src/V03EvidencePanel.calibration.test.tsx:75-95,148-165`; `src/V03EvidencePanel.pr... |
| LAB-13 | Low | UNVERIFIED | Error coercion is repeated outside the known `verify_041` path | `drive_console_check.mjs:111,124,132`; `dryrun_catalog.mjs:44`; `g05_mt06_cycles.mjs:214`; `g05_rt04v2_delay.mjs:165`; `g05_rt06_all_back... |
| DOC-15 | Low | UNVERIFIED | The 2,143-line vendored upstream README is a reference, not a local contract | `docs/LLAMA-SERVER-README.md:1-3,27-32`; `docs/OPTION_MAP.md:1-5`. |
| RT-13 | Low | UNVERIFIED | Dead scaffolding and an orphan service section remain in the runtime surface | `src-tauri/src/runtime.rs:148-158`, `3990-3997`; `src-tauri/src/runtime_service.rs:273-275`. |
| RT-15 | Low | UNVERIFIED | Audit-ticket and phase narration has become production maintenance noise | `src-tauri/src/runtime.rs:1552-1582`, `2355-2633`, `3814-3816`, `3989-3997`, `4503-4507`, `4638-4642`, plus the test block `4808-9147`; `... |
| CORE-02 | Low (was High) | CORRECTED | StrictMode turns the read-only runtime catalog guard into a startup race | `src/main.tsx:6-11`; `src/App.tsx:467-541,543-563,1417-1427`; `src-tauri/src/runtime_service.rs:38-64,79-114`. |
| PROC-22 | Low | UNVERIFIED | Audit-ticket narration is embedded throughout production code | examples include `src-tauri/src/proc.rs:126–132, 357–359`, `src-tauri/src/log_sink.rs:1–13`, `src-tauri/src/local_client.rs:419–423`, `sr... |
| FE-03 | Low (was High) | CORRECTED | A maintained CDP gate searches for a button the current UI no longer renders | `scripts/g05_dc01.mjs:25-31`; `src/screens/CatalogScreen.tsx:139-152`. |
| FE-12 | Low | UNVERIFIED | Vite config suppresses a Node type error instead of fixing the config contract | `vite.config.ts:1-7`; `tsconfig.node.json:1-10`. |
| FE-13 | Low | UNVERIFIED | Design tokens and component documentation are parallel, unenforced sources of truth | `src/main.tsx:1-5`; `src/App.tsx:7`; `src/App.css:1-19`; `docs/theme.css:1-8`; `docs/tokens.json:1-14`; `docs/DESIGN.md:153-340`. |

---

# File-by-file reviews

Each part below lists a verdict for each file in its scope and the detailed findings.
The lead check under each finding heading gives its verification state.

## Part 1 — Release pipeline, CI and qualification tooling (finding prefix REL)

### Summary

This is a read-only review of the 62 files in the requested release/CI/qualification scope. All 62 were read completely in line-numbered chunks. Four additional producer/dependency files were read because the workflows invoke them: `scripts/verify_041.mjs`, `scripts/verify_060_catalog.mjs`, `scripts/g05_mt06_cycles.mjs`, `scripts/g05_rt06_all_backends.mjs`, `scripts/g05_rt04v2_delay.mjs`, `scripts/g05_dc04_override.mjs`, `scripts/verify_a11y.mjs`, and `src-tauri/src/runtime.rs`. No repository file was modified and no build, npm, cargo, Tauri, app, workflow, or dispatch command was run.

The release has not shipped because the path is not one release gate. It is a long, stateful campaign with incompatible identity rules and several prerequisites that do not exist in a clean checkout.

- `gh run view` confirms runs `35897526108` and `35901604360` were `workflow_dispatch` `Release verify` runs, both failed in `verify` at `Verify the packaged executable` and `Report missing packaged evidence`.
- Run `35731266321` was a `push` run at product SHA `4b31431d28d1503efc7b80a46c77ad2c3d54f082`; it failed at `Assemble the canonical qualification manifest`.
- The two dispatch runs had `head_sha=0bd39fbfa0c9c19f08f6c4d3c185bb0440169fe0`, while their product checkout was the tag peel. `release-promote.yml` requires the verify run to be a `push` and requires its `head_sha` to equal the product tag SHA. The owner-selected dispatch producer therefore cannot pass promotion even after the packaged checks are fixed.
- A clean runner has no tracked `.hermes-0.6/g05-tls` material, but the release's mt06 driver reads `api-key.txt` from that directory unconditionally. The current lab is not clean-checkout runnable.
- `artifacts/` is a tracked dump containing 5,494 files, including WebView2/Crashpad profile data. The release stages into that directory and uploads globs over it. The failure diagnostics path explicitly uploads the whole tree.
- The manifest requires 18 records, including six deliberately negative witness outcomes and four lifecycle variants. This is release-campaign evidence, not the minimum evidence needed to establish build, tests, packaged smoke, installer lifecycle, and checksums.

Finding counts: 5 Critical, 8 High, 3 Medium, 0 Low. The most important cuts are: stop staging in tracked `artifacts/`; remove the local TLS dependency; remove the witness/lab campaign from the release gate; make promotion accept and identify a workflow-dispatch producer; and retain one clean, digest-bound candidate bundle.

### Release critical path

The following is the path to one public publish, not merely the `Release verify` job. Each numbered item is a required stage or an external dependency. The count is approximate: I count 41 named stages below, or roughly 50 failure opportunities when the four lifecycle legs and individual lab legs are expanded. These are not statistically independent; many share the same self-hosted machine, network, GPU, WebView2 state, and GitHub services.

1. GitHub must start the `Release verify` trigger (`push` tag or `workflow_dispatch` with `tag`) and not cancel it. The workflow uses a single concurrency group with `cancel-in-progress: true` (`.github/workflows/release.yml:5-20`).
2. A self-hosted Windows runner matching `Windows`, `X64`, `zen5`, `blackwell`, and `localmotive-hw` must be available for `resolve` (`release.yml:22-35`). The same runner class is required by `rust-audit` and `verify` (`release.yml:67-93`).
3. `actions/checkout`, with full history and tags, must resolve the requested tag ref (`release.yml:30-37`). GitHub action availability, the remote, credentials, tag visibility, and a clean enough workspace are prerequisites.
4. `resolve` must fetch `origin/main` and prove the tag commit is an ancestor of main (`release.yml:43-48`).
5. The tag must match `vMAJOR.MINOR.PATCH`, and `verify_versions.mjs` must accept every product and lockfile version (`release.yml:50-65`).
6. The independent `rust-audit` job must checkout the same source, install the pinned Rust toolchain, and complete `rustsec/audit-check` with the GitHub token (`release.yml:67-89`). This depends on Rust/toolchain availability, crates/advisory network access, and GitHub checks permission.
7. `verify` must checkout the exact resolved SHA and prove `git rev-parse HEAD` equals `RESOLVED_SHA` (`release.yml:91-116`).
8. Node 20, the pinned Rust toolchain, the Rust cache, and `npm ci` must work on the self-hosted runner (`release.yml:118-131`). This uses npm registry/network and a usable Cargo/Rust environment.
9. Version, action-pin, and workflow-gate checks must pass (`release.yml:133-140`).
10. `npm run check` must pass (`release.yml:142-143`). `package.json:10-18` expands this into all listed Node tests, Vitest, catalog validation, branding, research-anchor, qualification, icon verification, TypeScript, Vite, and frontend build.
11. `npm audit --audit-level=moderate` must reach the npm advisory service and pass (`release.yml:145-146`).
12. `cargo fmt --check`, Clippy with `-D warnings`, and locked Rust tests/docs must pass (`release.yml:148-160`). The trusted runner's hardware-sensitive Rust paths are not run with the PR skip flag.
13. `npm sbom --sbom-format cyclonedx` must run and produce a non-empty SBOM (`release.yml:162-166`).
14. `npm run tauri build` must produce the portable executable, MSI, and NSIS setup package (`release.yml:168-170`). This requires the Tauri CLI, Node, Cargo/Rust, Windows bundler tooling, and the Windows build environment.
15. The packaged matrix must find the portable executable, choose a run-specific CDP port, launch WebView2, and expose a page within 90 seconds (`release.yml:171-178`; `scripts/verify_packaged_impl.ps1:80-99`). Port availability, WebView2, process startup, and the packaged bytes are all gates.
16. The packaged matrix must complete `verify_041` and its real IPC/UI checks (`scripts/verify_packaged_impl.ps1:292-307`). The runtime recommendation/catalog checks call `fetch_runtime_catalog` (`scripts/verify_041.mjs:469-519`) and therefore depend on the product's unauthenticated GitHub API path (`src-tauri/src/runtime.rs:15`) as well as the GPU snapshot and catalog response.
17. The local catalog fixture must initialize, be adopted by verified process identity, survive first-fill/prep/restart, and merge into a source-bound record (`scripts/verify_packaged_impl.ps1:148-195`, `215-347`). This requires Node child processes, CIM process inspection, CDP, local ports, and cleanup.
18. The workflow must copy exactly three built outputs, calculate `SHA256SUMS`, and pass non-empty-file checks (`release.yml:179-190`).
19. `verify_candidate_inventory.mjs` must bind those three bytes, their sizes, checksums, release, and source revision (`release.yml:192-194`; `scripts/verify_candidate_inventory.mjs:38-73`).
20. Candidate-generated evidence is deleted, then installer payload identity must be extracted and functionally probed (`release.yml:196-212`; `scripts/verify_installer_payloads.mjs:15-30`, `51-117`). This requires the local 7-Zip installation at `C:\Program Files\7-Zip\7z.exe` unless `LOCALMOTIVE_7ZIP` is set, plus WebView2/CDP.
21. `WindowsSandbox.exe` and `wsb.exe --version` must be present (`release.yml:214-226`). Windows Sandbox must be enabled and usable by this self-hosted account.
22. The v0.4.0 upgrade lifecycle leg must download the prior NSIS asset through `gh`, run Windows Sandbox, install/uninstall, launch, and preserve data (`release.yml:228-243`; `host-run-lifecycle.ps1:160-180`, `361-505`).
23. The v0.5.0 upgrade lifecycle leg repeats the same full sandbox path against another GitHub Release asset (`release.yml:234-238`).
24. The v0.4.1 preservation leg repeats the sandbox path with the cache/settings fixture and Python preservation verification (`release.yml:237-243`; `host-run-lifecycle.ps1:413-470`).
25. The v0.5.0 preservation leg repeats it with the SQLite mirror fixture. Every lifecycle leg depends on `GH_TOKEN`, `gh` CLI, prior public assets/tags, Windows Sandbox networking, WebView2 bootstrapper availability, NSIS, MSI/msiexec, registry state, and isolated ports (`host-run-lifecycle.ps1:160-180`, `274-317`, `338-389`; `run-lifecycle-in-sandbox.ps1:283-359`).
26. The host preservation verifier must find Python and validate the collected SQLite/cache/settings data (`host-run-lifecycle.ps1:413-470`; `verify_preservation.py:1-224`).
27. The in-run lab must create isolated app/profile/scratch roots, start the candidate with a dynamic CDP port, detect a real GPU adapter, and keep all ports free (`release.yml:245-321`).
28. The pinned SmolLM2 model must download from Hugging Face with `curl.exe` (`release.yml:311-318`). Hugging Face availability, TLS, disk, and the pinned URL are external gates.
29. RT06 must install/inspect all selected runtime backends and write source/digest-bound evidence (`release.yml:323-328`; `g05_rt06_all_backends.mjs:23-30`, `256-340`). It needs the GPU adapter, runtime catalog/assets, network, and sufficient disk/time.
30. The generated activation and model-selection drivers must navigate the packaged UI, find exact button/placeholder text, inspect the managed runtime, and select the pinned model (`release.yml:329-382`).
31. The TLS profile driver must read `.hermes-0.6/g05-tls/key.pem`, `cert.pem`, and `api-key.txt` through the Profile UI, save them, start llama-server, and prove HTTPS metrics (`release.yml:383-445`; `g05_mt06_cycles.mjs:30-34`).
32. MT06 must run 75 cancellation/restart cycles, inspect app-owned children and port 8080, and write evidence (`release.yml:446-449`; `g05_mt06_cycles.mjs:67-90`, `290-333`, `699-722`).
33. DC04 must start a loopback fixture on its own port and exercise override/checksum/revocation behavior (`release.yml:451-455`; `g05_dc04_override.mjs:42-73`, `121-212`).
34. RT04 must serve the pinned model slowly, run three health attempts, and prove no process is spawned from tampered runtime bytes (`release.yml:457-461`; `g05_rt04v2_delay.mjs:52-86`, `119-262`).
35. The a11y driver must launch the portable executable with its own random CDP port and complete its DOM/accessibility assertions (`release.yml:463-466`; `verify_a11y.mjs:13-46`, `215-223`).
36. The witness harness must run the six fault/lock legs and produce every expected negative record (`release.yml:472-479`; `test-fault-evidence.ps1:46-234`). It requires `pwsh`, temporary process/lock state, and the same lifecycle harness.
37. The canonical manifest must find all 18 records, the candidate inventory, and fresh packaged record (`release.yml:481-483`; `build_qualification_manifest.mjs:29-48`, `166-200`).
38. `verify_release_promotion.mjs` must validate the complete manifest, exact three binaries, checksums, inventory, and producer packaged record without publishing (`release.yml:485-487`; `verify_release_promotion.mjs:83-205`).
39. The qualified artifact must upload successfully from the broad globs in `release.yml:489-499`; GitHub artifact storage, retention, and action availability are required.
40. The owner-only promotion gate must checkout the immutable tag, require the exact confirm phrase, fetch the remote tag, call two GitHub APIs for the verify run/check-runs, and accept the verify run's event/head SHA (`release-promote.yml:32-112`). This is where the current dispatch producer is rejected.
41. The promotion job must download the exact artifact, validate it again, extract a non-empty changelog section, publish six explicitly listed assets unsigned, and read the public release back through `gh release view/download`, exact asset-count, `cmp`, inventory, and checksum checks (`release-promote.yml:114-140`, `175-267`). It also depends on `contents: write`, GitHub release API limits, and the action service.

`hardware-qualify.yml` and `catalog.yml` are sibling workflows, not demonstrated publish prerequisites. `release-promote.yml:96-108` requires only a green `pr-check` or `check` run by name; whether external branch rules also require hardware qualification is UNKNOWN from this repository.

### Structural deadlocks

#### 1. Immutable tag plus main-ancestry gate makes a bad used tag unrecoverable

`release.yml:43-48` refuses any tag SHA that is not already an ancestor of `origin/main`. Promotion repeats the ancestry requirement at `release-promote.yml:75-83`, and the authoritative project policy forbids retargeting a used tag (`AGENTS.md:153-155`, `160-166`). Once a used tag points at a non-main or incomplete candidate, merging a repair cannot change the tag peel. A release can only escape by creating another version/tag, which the owner explicitly ruled out for this campaign. The six failed v0.6.0 attempts and the failed v0.6.1 tag attempt are the operational manifestation of this deadlock.

#### 2. "Records committed in the tag tree" conflicts with artifact-only evidence and no paperwork version bump

The historical gate comments/tests require every referenced record to be committed and visible from a fresh checkout (`release-gates.test.mjs:3155-3187`), and the 0.6.1 test pins the producer SHA and 18-record manifest (`release-gates.test.mjs:3193-3203`). But the authoritative policy says evidence is an attestation commit or CI artifact, never source, and says not to accept a record merely because it sits in the tag tree (`AGENTS.md:160-171`). The current release workflow deliberately deletes stale evidence and creates records in-run after checkout (`release.yml:196-208`, `245-479`, `481-497`). The same test file then explicitly proves that assembly accepts records outside git containment (`release-gates.test.mjs:3206-3225`).

This is three incompatible contracts: committed evidence, artifact-only evidence, and an implementation that accepts untracked evidence. If the first contract is enforced, evidence committed after the tag is invisible and no version bump/paperwork commit or tag move is allowed. If the second is intended, the committed-record assertions are obsolete. The minimal fix is to delete the tag-tree requirement and validate only the run-produced, digest-bound qualified bundle.

#### 3. Main workflow plus tag-tree scripts splits the producer identity

A dispatch run executes the workflow YAML from the selected main branch, but the workflow checks out the tag for repository code (`release.yml:33-36`, `107-110`). The manifest builder then hashes `.github/workflows/release.yml` from that tag checkout (`build_qualification_manifest.mjs:140-149`). Promotion verifies that digest against `workflowRoot: root`, where `root` is the tag checkout (`verify_release_promotion.mjs:70-90`). Therefore the manifest can prove the tag-tree workflow, not the main workflow text that GitHub actually interpreted. A main-only workflow fix can be executed without being represented by the manifest, while a tag-tree script can be paired with a newer workflow that expects different behavior.

The owner’s split can be made coherent only by recording both identities: the main workflow commit/digest as the producer workflow and the tag peel as the product source, then validating the uploaded producer manifest against both. Do not pretend the tag-tree workflow hash is the dispatch workflow hash.

#### 4. The owner-selected `workflow_dispatch` producer cannot satisfy promotion's `push`/product-SHA assertion

This is the direct publish deadlock. The producer workflow allows `workflow_dispatch` (`release.yml:5-14`). Promotion insists that the verify run event is `push` and that `run.head_sha` equals the tag's product SHA (`release-promote.yml:85-94`). Read-only GitHub data showed runs `35897526108` and `35901604360` were `workflow_dispatch` with main workflow head `0bd39fbfa0c9c19f08f6c4d3c185bb0440169fe0`, not the tag product SHA. They would fail promotion even if the verify job were green. A dispatch run's head SHA identifies the workflow/tooling revision; `RESOLVED_SHA` identifies the product tag peel. The gate currently conflates them.

The fix is not a tag move or a version bump. Promotion must accept `workflow_dispatch`, validate the run's workflow source separately, and validate the qualified manifest's `sourceRevision` against the peeled tag SHA.

### Per-file verdicts

Verdicts are for this release/qualification scope. `SHRINK` means retain the control but remove campaign-only breadth; `FIX` means the current contract is unsafe or contradictory; `DELETE` means remove it from the release path (some product/unit coverage may be retained elsewhere).

#### Workflows

| File | LOC | Verdict |
|---|---:|---|
| `.github/workflows/catalog.yml` | 92 | KEEP — independent signed catalog candidate flow is coherent and does not publish the desktop release. |
| `.github/workflows/ci.yml` | 220 | SHRINK — retain isolated PR and one trusted source check; remove duplicate release build/smoke work. |
| `.github/workflows/hardware-qualify.yml` | 152 | FIX — keep as optional host proof but correct the stale runner claim and do not imply it gates publication. |
| `.github/workflows/release-promote.yml` | 287 | FIX — repair dispatch identity, reject bundle extras, and keep exact-byte publication/readback. |
| `.github/workflows/release.yml` | 517 | SHRINK — keep one candidate build and minimal qualification; delete the lab/witness campaign and tracked staging. |

#### GitHub policy and issue metadata

| File | LOC | Verdict |
|---|---:|---|
| `.github/workflow-actions.json` | 51 | KEEP — retain immutable action inventory. |
| `.github/workflow-gates.json` | 127 | FIX — update the reduced command contract and dispatch producer identity. |
| `.github/dependabot.yml` | 25 | KEEP — unrelated dependency-update policy is small and valid. |
| `.github/ISSUE_TEMPLATE/bug_report.yml` | 43 | KEEP — not a release blocker. |
| `.github/ISSUE_TEMPLATE/config.yml` | 5 | KEEP — not a release blocker. |
| `.github/ISSUE_TEMPLATE/feature_request.yml` | 28 | KEEP — not a release blocker. |

#### Maintained release scripts

| File | LOC | Verdict |
|---|---:|---|
| `scripts/build_qualification_manifest.mjs` | 260 | SHRINK — replace the 18-record campaign map with a minimal run-produced manifest. |
| `scripts/verify_qualification_manifest.mjs` | 690 | SHRINK — retain source/digest/schema checks but remove tag-tree assumptions and campaign-only contracts. |
| `scripts/verify_release_promotion.mjs` | 283 | FIX — enumerate the qualified tree and verify the producer workflow identity separately. |
| `scripts/verify_candidate_inventory.mjs` | 158 | KEEP — exact three-artifact inventory/checksum binding is core. |
| `scripts/verify_installer_payloads.mjs` | 251 | KEEP — 7-Zip extraction and payload identity are credible installer evidence. |
| `scripts/verify_packaged_matrix.ps1` | 64 | KEEP — wrapper has a clear exit-code contract. |
| `scripts/verify_packaged_impl.ps1` | 380 | KEEP — owned-process cleanup and catalog fixture identity are valuable; use only the reduced matrix. |
| `scripts/check_package_outputs.ps1` | 26 | FIX — report downstream verification failure accurately instead of calling it packaging failure. |
| `scripts/verify_versions.mjs` | 77 | KEEP — version/lockfile identity gate is required before freezing a tag. |
| `scripts/verify_workflow_pins.mjs` | 77 | KEEP — immutable action pin gate is required. |
| `scripts/verify_workflow_gates.mjs` | 170 | FIX — retain parsed policy validation but remove obsolete/duplicated release assumptions. |
| `scripts/verify_workflow_syntax.mjs` | 119 | KEEP — syntax validation remains cheap and useful. |
| `scripts/verify_qualification.mjs` | 216 | KEEP — retain repository qualification checks if they remain in the one source check. |
| `scripts/verify_research_anchor.mjs` | 268 | KEEP — retain the source/research tracking gate outside the release campaign. |
| `scripts/check_tracker.mjs` | 114 | KEEP — one tracker check is preferable to another release policy layer. |
| `scripts/verify_tracker_links.mjs` | 133 | KEEP — retain link integrity in the normal source check. |
| `scripts/sign_catalog_candidate.mjs` | 16 | KEEP — belongs to the independent catalog workflow. |
| `scripts/qualification/build_host_attestation.mjs` | 128 | KEEP — retain optional host attestation, but do not make it a hidden release prerequisite. |

#### Sandbox harness and fixtures

| File | LOC | Verdict |
|---|---:|---|
| `scripts/sandbox/host-run-lifecycle.ps1` | 549 | SHRINK — retain one candidate-bound installer lifecycle; remove four-baseline/preservation campaign paths. |
| `scripts/sandbox/run-lifecycle-in-sandbox.ps1` | 551 | SHRINK — retain install/uninstall/upgrade smoke and owned cleanup; remove preservation/settings branches. |
| `scripts/sandbox/test-fault-evidence.ps1` | 237 | DELETE — negative fault witnesses belong in tests, not every release. |
| `scripts/sandbox/build_preservation_fixture.py` | 281 | DELETE — preservation campaign is not in the minimal release acceptance. |
| `scripts/sandbox/verify_preservation.py` | 224 | DELETE — remove with preservation campaign; retain only if separately required by product policy. |
| `scripts/sandbox/canary-catalog-cache.json` | 4 | DELETE — preservation fixture. |
| `scripts/sandbox/canary-mirror.sqlite.b64` | 1 | DELETE — preservation fixture. |
| `scripts/sandbox/canary-settings.json` | 33 | DELETE — preservation fixture. |
| `scripts/sandbox/canary-userdata.txt` | 1 | DELETE — preservation fixture. |

#### Tests and fixtures

| File | LOC | Verdict |
|---|---:|---|
| `scripts/tests/catalog_schema.test.mjs` | 78 | KEEP — catalog schema behavior. |
| `scripts/tests/fixtures/catalog-schema-cases.json` | 433 | KEEP — catalog schema fixtures. |
| `scripts/tests/fixtures/cloud-contracts.json` | 355 | KEEP — cloud contract fixtures used by source tests. |
| `scripts/tests/fixtures/ipc-contract.json` | 21 | KEEP — IPC contract fixture. |
| `scripts/tests/health_cancel.test.mjs` | 137 | KEEP — bounded health cancellation behavior. |
| `scripts/tests/http_retry.test.mjs` | 99 | KEEP — retry behavior. |
| `scripts/tests/installer_payloads.test.mjs` | 54 | KEEP — payload verifier regression coverage. |
| `scripts/tests/ipc_contract.test.mjs` | 49 | KEEP — IPC contract checks. |
| `scripts/tests/lib/manifest_fixture.mjs` | 478 | SHRINK — fixture only the reduced manifest and identity relationships. |
| `scripts/tests/mt06_verdicts.test.mjs` | 268 | DELETE from release gating — MT06 is not minimal acceptance. |
| `scripts/tests/preservation_verifier.test.mjs` | 225 | DELETE from release gating — preservation campaign is cut. |
| `scripts/tests/qualification_manifest.test.mjs` | 414 | SHRINK — retain candidate/source/digest negative controls, remove 18-record/tree coupling. |
| `scripts/tests/release-gates.test.mjs` | 3230 | SHRINK — replace brittle workflow-text assertions with parsed contract tests and one real command-path test. |
| `scripts/tests/release_promotion.test.mjs` | 215 | KEEP — exact-byte promotion and no-checkout-fallback behavior is core. |
| `scripts/tests/verify_cleanup_matrix.ps1` | 378 | KEEP — real packaged-orchestration cleanup path is valuable. |
| `scripts/tests/verify_lifecycle_ownership.ps1` | 40 | DELETE from release gating — covered by the reduced lifecycle harness. |
| `scripts/tests/verify_preservation_collection.ps1` | 49 | DELETE — preservation campaign is cut. |
| `scripts/tests/verify_settings_retention.ps1` | 42 | DELETE — preservation/settings campaign is cut. |
| `scripts/tests/verify_settings_session.ps1` | 123 | DELETE — preservation/settings campaign is cut. |
| `scripts/tests/workflow_integration.test.mjs` | 327 | SHRINK — test the reduced parsed workflow and artifact contract, not duplicated text. |

#### Schemas and package metadata

| File | LOC | Verdict |
|---|---:|---|
| `release-evidence/packaged-verification.schema.json` | 111 | KEEP — packaged record schema remains needed. |
| `release-evidence/qualification-attestation.schema.json` | 53 | KEEP — retain if host attestation remains an optional artifact. |
| `release-evidence/qualification-matrix.schema.json` | 37 | SHRINK — reduce to the minimal candidate/installer/package matrix. |
| `package.json` | 46 | SHRINK — retain one check script and remove release-only suite duplication. |

### Findings

#### Critical

##### REL-01 — Critical — Promotion rejects the owner-mandated dispatch producer

> **Lead check: VERIFIED.** release-promote.yml:90 requires event push; :92 requires head_sha equal to the tag peel; :100 requires pr-check or check on the peel. Dispatch runs 35897526108 and 35901604360 ran at 0bd39fb.

**Location:** `.github/workflows/release.yml:5-14`; `.github/workflows/release-promote.yml:85-94`; runs `35897526108`, `35901604360`.

**Title:** `workflow_dispatch` verify runs can never satisfy the promotion event/head-SHA gate.

**Evidence:**
> `assert.equal(run.event, 'push', 'verify run event');` (`release-promote.yml:89-94`)
>
> `workflow_dispatch:` with required `tag` input (`release.yml:9-14`)

**Why it matters:** The two owner-selected producer runs are confirmed `workflow_dispatch` runs with main's workflow head SHA, not the product tag SHA. Even a green packaged run would be rejected before publication. This alone explains why the selected producer path cannot ship.

**Concrete fix:** Accept `workflow_dispatch` in the promotion gate. Validate the dispatch workflow revision separately, then require the downloaded manifest/inventory `sourceRevision` and the remote peeled tag to equal the product SHA. Do not compare a dispatch run's `head_sha` to the product SHA.

##### REL-02 — Critical — Immutable tag plus main ancestry prevents repair of a used tag

> **Lead check: VERIFIED.** Run 35731266321 attempt 1 failed the main-ancestry gate. The release tag ruleset is immutable.

**Location:** `.github/workflows/release.yml:43-48`; `.github/workflows/release-promote.yml:75-83`; `AGENTS.md:153-166`; runs `34803387581`–`34819219725`, `35731266321`.

**Title:** A bad tag cannot be made eligible by merging the repair.

**Evidence:**
> `git merge-base --is-ancestor "$(git rev-parse HEAD)" origin/main` (`release.yml:46-48`)
>
> `test "$sha" = "$peeled"` and `git merge-base --is-ancestor "$sha" origin/main` (`release-promote.yml:75-83`)

**Why it matters:** Once an immutable `v0.6.0`/`v0.6.1` tag points at the wrong or incomplete commit, a later main merge does not change the peeled tag. The policy forbids retargeting and forbids creating a paperwork version solely to repair evidence. Re-running the same tag only repeats the failure.

**Concrete fix:** Freeze source, tooling identity, and product version before creating a new tag; for an already-used tag, stop attempting to qualify it under a different tree. For the current owner decision, keep the tag fixed and fix the artifact-only dispatch/promotion contract instead of adding another tag or version.

##### REL-03 — Critical — Evidence provenance rules contradict the tag-tree gate

> **Lead check: VERIFIED.** Run 35731266321 attempt 6 failed because 12 records were missing from the tag tree.

**Location:** `AGENTS.md:160-171`; `.github/workflows/release.yml:196-208`, `481-497`; `scripts/tests/release-gates.test.mjs:3155-3225`.

**Title:** The release simultaneously treats evidence as source, as CI artifact, and as untracked in-run output.

**Evidence:**
> `every referenced record committed` (`release-gates.test.mjs:3155-3158`)
>
> `manifest assembly accepts records outside git containment` and `missing ... outside git` (`release-gates.test.mjs:3206-3225`)
>
> The workflow deletes stale records, produces them in-run, and uploads them (`release.yml:196-208`, `481-497`).

**Why it matters:** A record committed after the immutable tag is absent from the tag checkout, producing the observed missing-record failure. Requiring that record in the tag tree conflicts with the authoritative rule that evidence is an artifact and never source. The no-paperwork-version rule removes the only way to place later records in the immutable tree.

**Concrete fix:** Make the in-run qualified bundle authoritative. Remove the committed-record assertions and the historical-manifest test from the release gate. Require every record to carry source/candidate identity where applicable and validate the uploaded bundle, not tag-tree containment.

##### REL-04 — Critical — Clean checkout lacks the lab TLS/API-key material

> **Lead check: VERIFIED.** The runner workspace has no .hermes-0.6 directory. g05_mt06_cycles.mjs:34 reads it at module load.

**Location:** `.github/workflows/release.yml:383-445`; `scripts/g05_mt06_cycles.mjs:30-34`; read-only `git ls-files` result: zero `.hermes-0.6` entries.

**Title:** The release depends on developer-local ignored files that are absent on a clean runner.

**Evidence:**
> `$tlsDir = Join-Path $repoRoot ".hermes-0.6/g05-tls"` (`release.yml:386`)
>
> `readFileSync(join(process.cwd(), ".hermes-0.6", "g05-tls", "api-key.txt"))` (`g05_mt06_cycles.mjs:34`)

**Why it matters:** The application is given paths to files that the workflow never creates, and the mt06 driver reads `api-key.txt` directly. A clean checkout cannot pass the TLS/mt06 lab. The path being locally excluded through `.git/info/exclude` does not put it in the runner checkout and must not be used for credentials or test material.

**Concrete fix:** Remove the TLS lab from the release gate, or generate disposable test key/cert/API-key files under a run-scoped `$RUNNER_TEMP` directory and verify their existence before use. Never read `.hermes-0.6` from the repository checkout.

##### REL-05 — Critical — Tracked `artifacts/` is uploaded as release evidence and diagnostics

> **Lead check: VERIFIED.** The diagnostics artifact of run 35901604360 holds 5,495 files (26,370,566 bytes). 5,494 are tracked history.

**Location:** `.github/workflows/release.yml:179-190`, `489-510`; read-only `git ls-files artifacts/**` result: 5,494 tracked files; runs `35897526108`, `35901604360`.

**Title:** The staging directory is also a historical audit/data dump.

**Evidence:**
> `mkdir -p artifacts` followed by candidate copies (`release.yml:181-189`)
>
> Qualified upload includes `artifacts/*`; failure diagnostics includes `artifacts/**` (`release.yml:493-510`)
>
> Tracked sample includes `artifacts/.../webview/EBWebView/Crashpad/...` (read-only `git ls-files`).

**Why it matters:** The workflow does not clear or isolate the tracked tree before globbing it. The diagnostic upload is unambiguously broad; the qualified upload's directory glob is recursively collected by the upload-artifact action. The parent observed 5,495 files in the diagnostics artifact for the two failed runs. This bloats the bundle, creates stale-evidence ambiguity, and can expose browser profile databases or other historical data.

**Concrete fix:** Stage into a newly created run-scoped `$RUNNER_TEMP/localmotive-qualified-$GITHUB_RUN_ID` directory, copy only the explicit candidate/inventory/checksum/packaged files, and upload exact paths. Remove historical audit data from the repository's artifact staging path or stop using that path entirely. The qualified verifier must reject unexpected files.

#### High

##### REL-06 — High — Live unauthenticated GitHub API is a release gate

> **Lead check: VERIFIED.** runtime.rs:2808-2887 always calls api.github.com.

**Location:** `scripts/verify_041.mjs:469-519`; `src-tauri/src/runtime.rs:15`; runs `35897526108`, `35901604360`.

**Title:** Packaged smoke depends on a shared 60-request/hour tokenless API budget.

**Evidence:**
> `fetch_runtime_catalog` is called twice by the packaged checks (`verify_041.mjs:475`, `507`)
>
> `RELEASE_BY_TAG_URL = "https://api.github.com/repos/ggml-org/llama.cpp/releases/tags"` (`runtime.rs:15`)

**Why it matters:** Every local app launch and every packaged check shares the unauthenticated GitHub API quota for the runner/IP. The recorded failures were the three IPC checks `ipc.runtime-recommendation`, `ipc.reject-unknown-adapter`, and `ipc.runtime-catalog-fetch`, with terminal-error/catalog-result evidence. This is not a transient qualification detail; it is a gate designed around an external quota.

**Concrete fix:** Ship/use a pinned runtime catalog for the qualification path with all required tag/commit/asset/size/digest fields, and test the fresh-profile/no-network path. If a live refresh remains a product feature, make it non-blocking for release smoke and preserve structured error kinds rather than `[object Object]`.

##### REL-07 — High — Manifest binds the tag workflow, not the dispatch workflow

> **Lead check: VERIFIED.** build_qualification_manifest.mjs:146-149 hashes the checked-out release.yml.

**Location:** `.github/workflows/release.yml:35`, `107-110`; `scripts/build_qualification_manifest.mjs:140-149`; `scripts/verify_release_promotion.mjs:86-90`.

**Title:** Main workflow and tag-tree scripts have no single recorded workflow identity.

**Evidence:**
> `ref: ${{ needs.resolve.outputs.sha }}` checks out the tag product tree (`release.yml:107-110`)
>
> Manifest hash is read from `resolve(root, ".github/workflows/release.yml")` (`build_qualification_manifest.mjs:146-149`)
>
> Promotion validates with `workflowRoot: root` (`verify_release_promotion.mjs:86-90`).

**Why it matters:** GitHub interpreted main's dispatch workflow, but the manifest records the tag checkout's workflow. A main-only workflow repair can run without being represented in the evidence. This invalidates the four-identity audit trail even if all product bytes are correct.

**Concrete fix:** Record the main workflow commit/digest as a separate producer identity supplied by the dispatch run, and validate it from an uploaded workflow-source record. Keep the product `sourceRevision` bound to the tag peel. Alternatively, run the workflow from the tag; do not mix the two models silently.

##### REL-08 — Medium (lead; reviewer said High) — Promotion silently accepts extra files

> **Lead check: CORRECTED.** The comments at verify_release_promotion.mjs:8 and :43 promise an extra-file check that does not exist. Publication uploads only the promised names.

**Location:** `scripts/verify_release_promotion.mjs:42-52`, `83-106`; `.github/workflows/release-promote.yml:226-233`.

**Title:** The verifier checks six promised names but never enumerates the qualified directory.

**Evidence:**
> `promisedAssetNames()` returns the six expected names (`verify_release_promotion.mjs:42-52`)
>
> The implementation loops over those names but has no `readdir`/extra-file check (`verify_release_promotion.mjs:95-106`)
>
> Publish lists six exact paths and ignores the rest (`release-promote.yml:226-233`).

**Why it matters:** A polluted qualified bundle can pass promotion. Publication ignores extras, so the problem is silent rather than fail-closed; diagnostics and retained artifacts remain polluted. The comment claims an inflated set is refused, but the implementation does not enforce that claim.

**Concrete fix:** Enumerate every file in the downloaded bundle, compare against an explicit allowlist for the six public assets plus the manifest/records, and fail on any extra or misplaced file before publication.

##### REL-09 — High — Failure report mislabels packaged verification as packaging failure

> **Lead check: VERIFIED.** Run 35901604360: step #18 passed, and the log says 'Packaging failed before producing'.

**Location:** `.github/workflows/release.yml:171-190`, `514-517`; `scripts/check_package_outputs.ps1:19-25`; runs `35897526108`, `35901604360`.

**Title:** A downstream failure invokes a script that claims the package was never produced.

**Evidence:**
> `if: failure()` invokes `check_package_outputs.ps1` (`release.yml:514-517`)
>
> The script prints `Packaging failed before producing:` for missing staged files (`check_package_outputs.ps1:19-23`)
>
> Both named runs failed at `Verify the packaged executable` and then `Report missing packaged evidence`.

**Why it matters:** `npm run tauri build` can succeed while the packaged matrix fails before staging. The reporter then sees no staged files and calls the successful build a packaging failure. This hides the actual IPC/API/root-cause failure and sends repair work toward the wrong stage.

**Concrete fix:** Make the report step say `candidate outputs missing at diagnostic checkpoint` and report the first failed step. Invoke a packaging-specific report only when the build step itself failed; otherwise list which downstream artifacts had not yet been staged.

##### REL-10 — High — `Stop-LabApp` kills only the parent process

> **Lead check: VERIFIED.** Stop-LabApp in release.yml calls $proc.Kill() without the process tree.

**Location:** `.github/workflows/release.yml:271-295`.

**Title:** Lab cleanup proves only that the Tauri parent exited, not that its server child exited.

**Evidence:**
> `Start-Process $portable -PassThru` returns the parent handle (`release.yml:275`)
>
> `Stop-LabApp` calls `$proc.Kill()` and `WaitForExit`, with no descendant/process-tree cleanup (`release.yml:291-295`).

**Why it matters:** The app owns a llama-server child. A killed parent can leave the child listening and contaminate the next lab leg, especially because four legs reuse the same lab port family and profile. It also contradicts the stronger owned-child logic elsewhere in `verify_packaged_impl.ps1`.

**Concrete fix:** Track the app's owned process tree and stop descendants through handles/job objects, or use the existing app-owned stop path before parent cleanup and verify no child remains. Never replace this with a global process-name or port kill.

##### REL-11 — High — Generated lab JavaScript is hardcoded and DOM-text fragile

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `.github/workflows/release.yml:271-289`, `329-382`, `421-449`.

**Title:** The inline lab driver hardcodes server port 8080, sleeps, and scrapes visible button text.

**Evidence:**
> Generated HTTPS probe uses `port: 8080` (`release.yml:421-429`)
>
> Generated scripts use repeated `setTimeout`/fixed sleeps and `querySelectorAll('button')` with exact text such as `Runtime`, `Inspect`, and `Inventory` (`release.yml:329-382`, `438-449`).

**Why it matters:** The CDP port is dynamically computed, while the server port is assumed. UI copy, localization, disabled state, timing, or a profile with another configured port makes the test fail or, worse, probe the wrong service. The lab becomes a second UI automation framework embedded in YAML.

**Concrete fix:** Delete the generated drivers and reuse one maintained CDP/IPC driver with semantic selectors/ARIA or direct typed commands. Read the configured server endpoint from the app state; poll for a condition with a deadline instead of fixed sleeps.

##### REL-12 — High — Eighteen mandatory records turn a release into a fault-injection campaign

> **Lead check: VERIFIED.** The manifest requires 18 records. Attempt 6 of run 35731266321 missed 12 of them.

**Location:** `scripts/build_qualification_manifest.mjs:29-48`; `scripts/verify_qualification_manifest.mjs:58-83`, `643-655`; `.github/workflows/release.yml:228-479`.

**Title:** Negative witnesses and four lifecycle variants are mandatory for every publish.

**Evidence:**
> The manifest map contains 18 records (`build_qualification_manifest.mjs:29-48`)
>
> Witness records deliberately require `FAIL`/`TIMEOUT` outcomes (`verify_qualification_manifest.mjs:58-67`)
>
> The release invokes four lifecycle legs, six witness legs, and five lab legs before manifest assembly (`release.yml:228-479`).

**Why it matters:** A release can be functionally ready and still fail because a fault witness, old-release download, GPU backend, Hugging Face request, or local lab setup is unavailable. The v0.6.1 attempt that reached manifest assembly reported 12 missing records; the campaign is the multiplier, not evidence that a three-binary release is unsafe.

**Concrete fix:** Keep fault-injection and preservation tests in CI/nightly qualification. For publication retain one packaged smoke, one candidate-bound NSIS/MSI install/uninstall/upgrade record, candidate inventory, checksums, and public readback. Do not require negative witness files in the public release manifest.

##### REL-13 — High — A 360-minute release can be cancelled by its own retry

> **Lead check: VERIFIED.** release.yml uses concurrency group localmotive-release with cancel-in-progress: true and timeout-minutes 360.

**Location:** `.github/workflows/release.yml:18-20`, `91-97`.

**Title:** `cancel-in-progress: true` discards long-running candidate qualification.

**Evidence:**
> `group: localmotive-release` and `cancel-in-progress: true` (`release.yml:18-20`)
>
> `verify` has `timeout-minutes: 360` (`release.yml:91-97`).

**Why it matters:** A re-dispatch, tag retry, or duplicate trigger cancels an active run that may already have consumed build, GPU, Sandbox, and GitHub quota. The cancellation leaves no qualified bundle and can race with retained diagnostics on the self-hosted runner.

**Concrete fix:** Set cancellation false for a frozen product run, or use a tag/source-specific group and explicitly reject a second producer for the same tag. Keep cleanup and artifact retention on cancellation.

#### Medium

##### REL-14 — Medium — CI repeats heavy release gates and a build

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `.github/workflows/ci.yml:53-88`, `110-141`; `.github/workflows/release.yml:133-178`; `.github/workflow-gates.json:20-88`.

**Title:** The same source checks and package work are executed in main CI and release verification.

**Evidence:**
> CI runs versions/pins/gates, `npm run check`, audit, fmt, Clippy, and tests (`ci.yml:53-88`)
>
> Release repeats the same commands and then builds/launches the package (`release.yml:133-178`).

**Why it matters:** A release qualification takes the cost and failure surface of the main check again, then adds another build. The build must be done on the exact candidate bytes, but static checks and source tests do not need to run twice for the same SHA. `workflow-gates.json` codifies both copies.

**Concrete fix:** Require a green check run for the exact candidate SHA and retain one candidate-only build/package/lifecycle path. If the release workflow must re-run source tests for governance, delete the corresponding main package-smoke/build duplication rather than keeping both.

##### REL-15 — Medium — `release-gates.test.mjs` is heavily coupled to workflow spelling

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `scripts/tests/release-gates.test.mjs:210-236`, `1452-1660`, `3155-3225`.

**Title:** 38 of 166 tests directly read workflow YAML/policy files and assert text, regex, or ordering.

**Evidence:**
> `release.yml` is read and matched with regexes for identity strings (`release-gates.test.mjs:210-222`)
>
> The historical manifest test explicitly filters out workflow digest drift (`release-gates.test.mjs:3173-3175`, `3197-3199`).

**Why it matters:** A harmless step rename, shell normalization, or reordering can require test edits even when behavior is unchanged. Conversely, the manifest workflow digest is not a real protection in this test because drift is deliberately exempted, while the production verifier still rejects drift (`verify_qualification_manifest.mjs:256-273`). The suite is broad but not a reliable end-to-end workflow test; it never runs GitHub Actions.

**Concrete fix:** Keep parsed workflow contract tests for jobs, needs, permissions, and forbidden publish actions. Delete comment/text/ordering assertions that duplicate YAML parsing. Rebuild and validate the manifest when the workflow changes; do not filter the digest drift as a permanent historical exception.

##### REL-16 — Medium — Hardware workflow summary contradicts actual release runner policy

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `.github/workflows/hardware-qualify.yml:100-108`; `.github/workflows/release.yml:22-24`, `67-69`, `91-94`.

**Title:** The hardware job says the main release is hosted while the release is self-hosted.

**Evidence:**
> Summary says `Main Release build/publish stays on GitHub-hosted windows-latest` (`hardware-qualify.yml:105-107`)
>
> Release jobs use `[self-hosted, Windows, X64, zen5, blackwell, localmotive-hw]` (`release.yml:22-24`, `67-69`, `91-94`).

**Why it matters:** Operators will provision and troubleshoot the wrong runner class. The same summary also says full packaged L4 PASS needs another verifier, while the release itself runs GPU-dependent packaged/lab checks. This stale statement makes ownership and prerequisite triage harder.

**Concrete fix:** Correct the summary to state the actual runner and explicitly distinguish optional host attestation from the release candidate verifier.

### Answers to specific questions

#### 1. Critical path and failure-point estimate

The 41-step list above is the numbered critical path. Expanded by four independent lifecycle legs, the RT06/MT06/DC04/RT04/a11y legs, and publication readback, it is approximately 50 distinct failure opportunities. The largest external clusters are:

- **GitHub:** Actions queue/concurrency, tag/ref visibility, `origin/main`, `gh` CLI/token, GitHub Release assets for v0.4.0/v0.4.1/v0.5.0, Actions artifact storage, GitHub API quota, release write/readback, and action availability.
- **Build toolchain:** Node 20/npm registry, Cargo/Rustup/crates/advisory service, Tauri CLI, Windows bundlers, MSI/NSIS, 7-Zip, PowerShell/pwsh, Python, and `msiexec`.
- **Windows host:** self-hosted labels, WebView2, Windows Sandbox/`wsb.exe`, registry state, process/CIM visibility, clean temp/profile state, GPU adapter and runtime availability, disk, and exclusive ports.
- **Network/data:** unauthenticated `api.github.com`, Hugging Face pinned model download, Sandbox networking for WebView2 bootstrapper, local fixture ports, app server port 8080, CDP ports 10041+run-derived, 10093 in Sandbox, and random a11y ports.
- **Local-only files:** `.hermes-0.6/g05-tls/{key.pem,cert.pem,api-key.txt}` are required but absent from a clean checkout; 7-Zip is assumed at a machine-specific path unless overridden.

Duplicated with main CI: `verify_versions`, workflow pins, workflow gates, `npm run check`, `npm audit`, Cargo fmt, Clippy, and Rust tests run in `ci.yml:53-88` and again in `release.yml:133-160`. Package-smoke also builds the app, checks all three outputs, starts the portable app, sleeps eight seconds, and kills the parent (`ci.yml:110-141`); release rebuilds and runs the full packaged matrix (`release.yml:168-178`). The exact candidate still needs one build and one packaged qualification, but the static source gates and the basic package-smoke are duplicated.

#### 2. Structural deadlocks

The four concrete deadlocks are documented above:

1. Immutable used tag + main-ancestry check means a failed tag cannot be repaired in place (`release.yml:43-48`, `release-promote.yml:75-83`, `AGENTS.md:153-166`).
2. Requiring records in the tag tree conflicts with artifact-only evidence, in-run generation, and the prohibition on version bumps for paperwork (`AGENTS.md:160-171`, `release.yml:196-208`, `release-gates.test.mjs:3155-3225`).
3. Workflow-from-main plus scripts/product-from-tag records the wrong workflow digest (`release.yml:35`, `107-110`; `build_qualification_manifest.mjs:146-149`; `verify_release_promotion.mjs:86-90`).
4. The dispatch producer is rejected as a push run and its main workflow head is rejected as the product SHA (`release.yml:9-14`; `release-promote.yml:85-94`; runs `35897526108`, `35901604360`).

The "no version bump for paperwork" rule is the constraint that prevents the tag-tree deadlock from being papered over. It is correct; the tag-tree evidence requirement is the part that must be removed.

#### 3. Verification/refutation

**(a) `.hermes-0.6/g05-tls`: confirmed.** The path is not tracked (`git ls-files` returned no `.hermes-0.6` entries). The release only passes paths through the UI (`release.yml:383-401`), while mt06 reads `api-key.txt` directly (`g05_mt06_cycles.mjs:30-34`). A clean checkout has no file to read and no workflow step creates it. This is a release blocker.

**(b) `artifacts/` pollution: confirmed for diagnostics; confirmed by action glob semantics for qualified upload, with the action's recursion not locally replayed.** The repository has 5,494 tracked files under `artifacts/`, including browser-profile databases. `release.yml:493-510` explicitly uploads `artifacts/*` and, on failure, `artifacts/**`; the latter definitely walks the historical tree. The parent-observed 5,495-file diagnostics artifacts in runs `35897526108` and `35901604360` match that design. `verify_release_promotion.mjs` checks only the six promised names and never enumerates extras (`83-106`), while `release-promote.yml:226-233` publishes only six exact files. Extra files therefore pass validation and are silently ignored by publication rather than rejected.

**(c) `check_package_outputs.ps1`: confirmed.** The workflow invokes it on any failure (`release.yml:514-517`), and the script prints `Packaging failed before producing` whenever staged files are absent (`check_package_outputs.ps1:19-25`). A failure in packaged verification happens before the stage step (`release.yml:171-190`), so a successful build can generate the exact misleading message. Runs `35897526108` and `35901604360` prove this sequence.

**(d) Generated lab JavaScript: confirmed.** The lab block writes JavaScript with fixed sleeps and exact button/placeholder text (`release.yml:329-382`, `438-449`). The HTTPS probe hardcodes port 8080 (`release.yml:421-429`) while the CDP/lab ports are dynamically computed (`release.yml:260-263`). This is brittle and should be deleted or replaced with a maintained driver.

**(e) `Stop-LabApp`: confirmed.** It calls `Kill()`/`WaitForExit()` only on the `Start-Process` parent object (`release.yml:291-295`). It does not enumerate or stop the llama-server child. The packaged matrix has stronger owned-handle logic (`verify_packaged_impl.ps1:46-77`), but the lab does not reuse it.

#### 4. `release-gates.test.mjs` static coupling

There are **166** `test(...)` declarations in `scripts/tests/release-gates.test.mjs`. Using a conservative source-based classification, **38/166 (22.9%)** directly read a real `.github` workflow/policy file via `readFile(join(process.cwd(), ".github", ...))` or `loadWorkflows(process.cwd())` and assert YAML/policy text, regex, job shape, or ordering. The other **128/166** exercise pure validators, temporary fixtures, subprocess exit behavior, manifests, inventories, or source checks; none executes a GitHub Actions run or the packaged application.

Examples of the 38 static tests:

- `release evidence uses one resolved immutable revision` (`release-gates.test.mjs:210-222`) reads both release workflows and matches exact shell strings.
- `release workflow serializes runs...` and the packaged orchestration tests (`release-gates.test.mjs:1554-1610`) assert exact workflow YAML snippets.
- `a tag push verifies without publishing...` (`release-gates.test.mjs:1660-1781`) checks trigger/publish text.
- `the qualification manifest validates...` (`release-gates.test.mjs:3169-3204`) reads the historical manifest and exempts workflow digest drift.

The tests are therefore coupled to workflow spelling for those 38 cases, but not by a literal expected workflow digest in the test. Any workflow edit that changes an asserted string/order forces a test edit. The production manifest does record and verify the workflow digest (`build_qualification_manifest.mjs:146-149`; `verify_qualification_manifest.mjs:256-273`), but the historical gate deliberately filters that failure (`release-gates.test.mjs:3173-3175`, `3197-3199`).

#### 5. The 18-record manifest

"Digest-bound" below means bound to the candidate bytes/source, not merely hashed as a file inside the manifest. Every manifest entry gets a file hash; only the rows marked yes have a producer-to-candidate relationship that prevents cross-build reuse.

| Record | Producer | Candidate/digest binding | Minimal credible release |
|---|---|---|---|
| `packaged_verification` | `verify_041.mjs` through `verify_packaged_matrix.ps1` (`release.yml:171-178`; `verify_packaged_impl.ps1:292-301`) | **Yes** — source, portable name/size/SHA checked (`verify_qualification_manifest.mjs:498-553`) | **Keep**, reduced to packaged smoke if the full IPC matrix is removed. |
| `lifecycle_upgrade_v0.4.0` | `host-run-lifecycle.ps1` with `PreviousTag=v0.4.0` (`release.yml:234-243`) | **Yes** — setup/MSI digests, inventory SHA, source | Drop; v0.5.0 is the relevant previous public baseline. |
| `lifecycle_upgrade_v0.5.0` | `host-run-lifecycle.ps1` with `PreviousTag=v0.5.0` | **Yes** | **Keep one** compact NSIS/MSI install/uninstall/upgrade record. |
| `lifecycle_preservation_v0.4.1` | `host-run-lifecycle.ps1`, cache flavor | **Yes** | Drop; preservation is outside the stated minimum. |
| `lifecycle_preservation_v0.5.0` | `host-run-lifecycle.ps1`, mirror flavor | **Yes** | Drop; preservation is outside the stated minimum. |
| `witness_missing_assets` | `test-fault-evidence.ps1` → host harness with missing assets | **No** — negative control has no candidate bytes | Drop; keep as a CI fault test. |
| `witness_timeout` | fault harness → host lifecycle with candidate | **Yes in producer fields**, but it is a negative control | Drop from release; keep in CI. |
| `witness_malformed_result` | fault harness → host lifecycle with candidate | **Yes in producer fields**, but it is a negative control | Drop from release; keep in CI. |
| `witness_preservation_missing` | fault harness → host lifecycle with candidate | **Yes in producer fields**, but it is a negative control | Drop from release; keep in CI. |
| `witness_stale_lock` | fault harness takeover with missing-assets path | **No** — no candidate-byte binding | Drop from release; keep in CI. |
| `witness_live_lock` | fault harness live-lock refusal/takeover | **No** for the final missing-assets refusal record | Drop from release; keep in CI. |
| `mt06_cancellation` | `g05_mt06_cycles.mjs` (`release.yml:446-448`) | **Yes** — `sourceRevision` and `portableDigest` (`verify_qualification_manifest.mjs:585-604`) | Drop from minimal release; retain as optional/nightly runtime qualification. |
| `installer_payload_identity` | `verify_installer_payloads.mjs` (`release.yml:210-212`) | **Yes** — inventory and extracted payload digests | Keep, or fold its fields into the one lifecycle record. |
| `dc04_command_path` | `g05_dc04_override.mjs` (`release.yml:451-455`) | **No explicit candidate digest/source in the log** | Drop; this is a catalog-edge regression, not minimum release evidence. |
| `rt04_delayed_download` | `g05_rt04v2_delay.mjs` (`release.yml:457-461`) | **No explicit candidate digest/source in the log** | Drop; retain as product/integration regression coverage. |
| `a11y_packaged_verification` | `verify_a11y.mjs` (`release.yml:463-466`) | **No explicit candidate digest/source in the log** | Drop from release; run accessibility checks in CI or a separate packaged job. |
| `rt06_all_backends` | `g05_rt06_all_backends.mjs` (`release.yml:323-328`) | **Yes** — source and portable digest | Drop from minimal release; optional GPU qualification. |
| `rt06_full_run_log` | Same RT06 invocation (`release.yml:323-328`) | **Partial** — manifest file hash only; duplicate log companion, not an independently checked candidate record | Drop; retain structured RT06 JSON if the optional campaign remains. |

A minimal retained manifest has two records if payload identity is folded into lifecycle: packaged smoke and one candidate-bound installer lifecycle. If the existing separate payload verifier is retained, it has three records. The candidate inventory, `SHA256SUMS`, and three candidate binaries remain public artifacts, not extra qualification records. Fifteen of the current 18 records can be removed from the publication gate.

#### 6. Minimal replacement release pipeline

The replacement is described in the final section. It preserves the non-negotiables: explicit owner approval, same bytes promoted, unsigned disclosure, immutable product identity, and isolated untrusted PRs.

### Minimal replacement pipeline

#### Keep

1. **PR isolation:** Keep `ci.yml:143-220` as the only untrusted PR job on ephemeral `windows-latest`, read-only, no secrets. Keep the stable `pr-check` name and branch protection.
2. **One trusted source check:** On main, run versions, action pins, parsed workflow policy, `npm run check`, audit, Rust fmt/Clippy/tests, and the Rust advisory audit once. Remove the duplicate main `package-smoke` build if the release build is the only shipped-byte build.
3. **Owner-directed producer:** Keep one manual `Release verify` dispatch input `tag`. Resolve the remote peeled tag to one full SHA, require main ancestry, validate product version, and checkout exactly that SHA for all product/scripts.
4. **One build:** Run `npm ci`, source checks for the candidate SHA, `npm run tauri build`, and one packaged smoke/CDP matrix. The build output is the only candidate byte set.
5. **Clean staging:** Create a run-scoped empty directory under `$RUNNER_TEMP`. Copy the three binaries, inventory, checksums, packaged-smoke record, one installer lifecycle record, and optional payload identity into explicit paths. Never use repository `artifacts/`.
6. **One installer lifecycle:** In one Windows Sandbox run, test NSIS fresh install/launch/uninstall, MSI fresh install/launch/uninstall, and upgrade from v0.5.0. Bind the record to the candidate inventory and installer digests. Do not run four historical baselines or preservation fixtures.
7. **Manifest and verify:** Build a small manifest from those in-run records, record product SHA, producer workflow SHA/digest, inventory digest, candidate digests, and unsigned status. Validate the exact qualified tree and reject every extra file.
8. **Retain exact bytes:** Upload the qualified bundle with explicit file paths, not `artifacts/*`. Retain the successful verify run ID and manifest.
9. **Explicit promotion:** Keep `release-promote.yml` as a separate owner-only `workflow_dispatch` with exact `PUBLISH <tag>` confirmation and `dry_run` default true. Accept a verify run whose event is `workflow_dispatch`, validate its workflow head separately, and require the downloaded manifest's product SHA to equal the immutable tag peel.
10. **No rebuild:** Download the exact qualified artifact from the verify run, validate it once, and pass those exact paths to `action-gh-release`. Publish six exact assets, `prerelease: false`, with the unsigned/SmartScreen disclosure.
11. **Public readback:** Read the release back with `gh`, assert exact tag/name/draft/prerelease/asset count, download the six assets, compare inventory and packaged record byte-for-byte, and run checksum verification from the download directory.

#### Delete from the release path

- The `push.tags` automatic release producer if the owner has selected one manual dispatch producer; this prevents a tag push from creating a second, incompatible candidate run.
- The separate release `rust-audit`/main package-smoke duplication if the exact source check is a required green check reused by promotion; keep one advisory audit execution.
- `.hermes-0.6` TLS/API-key dependency and the generated TLS/UI lab block.
- RT06 all-backends, MT06 75-cycle campaign, DC04, RT04, and a11y as publication gates; keep them optional/nightly or product regression tests.
- Six witness records and their per-release fault harness; retain the harness tests in CI if the failure behavior matters.
- Four preservation/old-baseline lifecycle legs; retain one v0.5.0 installer upgrade lifecycle.
- The tag-tree/committed-record requirement and the historical manifest tests that exempt workflow drift.
- Broad `artifacts/*`/`artifacts/**` uploads and all stale tracked audit data in the staging path.

#### Unknowns that must remain visible

- Whether repository branch rules outside these files require `hardware-qualify` is UNKNOWN; the promotion YAML itself requires only `pr-check` or `check`.
- The exact qualified-upload recursion is an external `actions/upload-artifact` implementation detail; diagnostics pollution is certain because `artifacts/**` is explicit, and the observed run artifact count confirms the failure mode.
- No local build/test result is claimed here because running cargo/npm/Tauri/app/workflow was prohibited by the review scope. The named GitHub run conclusions and job steps above are the only execution evidence used.

---

## Part 2 — Lab harness, verifier and catalog scripts (finding prefix LAB)

### Summary

Read-only static review of the requested `scripts/` and `catalog/` scope. All 49 scoped scripts were read completely; `catalog/README.md`, `catalog/providers.json`, and `catalog/catalog.json.sig` were read completely. `catalog/catalog.json` was characterized from its first 300 and last 100 lines, as required. No Node, Cargo, Tauri, or network call was run.

The caller scan is limited to `package.json`, `.github/workflows/*.yml`, `scripts/verify_packaged_matrix.ps1`, `scripts/verify_packaged_impl.ps1`, `scripts/sandbox/*.ps1`, and `scripts/tests/**`. `UNREFERENCED` means no match in that required reference set; it does not claim manual invocation is impossible.

- **KEEP:** 11 scripts.
- **MERGE INTO MATRIX:** 7 scripts. They are live lab/version entry points, but packaged checks belong in the existing CDP matrix.
- **DELETE:** 31 scripts. They have no operational caller in the required set or are superseded by an active verifier/shared helper.
- `catalog/catalog.json`: **17,299 lines / 630,182 bytes**, **158 models / 1,417 files**. The pretty-printed file is rewritten wholesale by the builder.
- The most damaging release-gate defect is `verify_041.mjs`: two runtime-catalog checks turn structured Rust IPC errors into `[object Object]` and record `{}` evidence. A live GitHub failure becomes product `FAIL`, not a tooling/network outcome.
- Current source does **not** prove `ipc.reject-unknown-adapter` produces `[object Object]`: its current `rejectedInvoke` path serializes the raw object. If a captured record says all three did so, it is stale, from another artifact, or an unexpected harness path; do not accept it as product evidence.

### Script reference table

`Referenced by` lists exact `file:line` matches in the required caller set. Source-reading tests are not operational execution paths.

| Script | Lines | Referenced by | Verdict |
|---|---:|---|---|
| `scripts/build_catalog.mjs` | 266 | `.github/workflows/catalog.yml:38`; `package.json:12`; `scripts/tests/release-gates.test.mjs:177,2194,2206,2230` | KEEP |
| `scripts/drive_console_check.mjs` | 134 | UNREFERENCED in the required reference set | DELETE |
| `scripts/dryrun_catalog.mjs` | 109 | UNREFERENCED in the required reference set | DELETE |
| `scripts/g05_cancellation.mjs` | 153 | UNREFERENCED in the required reference set | DELETE |
| `scripts/g05_churn_repro.mjs` | 87 | UNREFERENCED in the required reference set | DELETE |
| `scripts/g05_dc01.mjs` | 77 | UNREFERENCED in the required reference set | DELETE |
| `scripts/g05_dc04_override.mjs` | 212 | `.github/workflows/release.yml:453`; `scripts/tests/release-gates.test.mjs:3043` | MERGE INTO MATRIX |
| `scripts/g05_fe05v3.mjs` | 166 | `scripts/tests/release-gates.test.mjs:2027` | DELETE |
| `scripts/g05_fe16.mjs` | 224 | `scripts/tests/release-gates.test.mjs:2022` | DELETE |
| `scripts/g05_hardlink_drive.mjs` | 78 | UNREFERENCED in the required reference set | DELETE |
| `scripts/g05_health.mjs` | 71 | UNREFERENCED in the required reference set | DELETE |
| `scripts/g05_launch_benchmark.mjs` | 114 | UNREFERENCED in the required reference set | DELETE |
| `scripts/g05_mt01d.mjs` | 123 | UNREFERENCED in the required reference set | DELETE |
| `scripts/g05_mt06_cycles.mjs` | 722 | `.github/workflows/release.yml:447`; `scripts/tests/release-gates.test.mjs:1191` | MERGE INTO MATRIX |
| `scripts/g05_partial_resume.mjs` | 68 | UNREFERENCED in the required reference set | DELETE |
| `scripts/g05_partial_retention.mjs` | 129 | UNREFERENCED in the required reference set | DELETE |
| `scripts/g05_rt04v2_delay.mjs` | 262 | `.github/workflows/release.yml:459` | MERGE INTO MATRIX |
| `scripts/g05_rt06_all_backends.mjs` | 340 | `.github/workflows/release.yml:327` | MERGE INTO MATRIX |
| `scripts/g05_run_cancel.mjs` | 94 | UNREFERENCED in the required reference set | DELETE |
| `scripts/g05_state.mjs` | 56 | UNREFERENCED in the required reference set | DELETE |
| `scripts/g05_stop_supervision.mjs` | 81 | UNREFERENCED in the required reference set | DELETE |
| `scripts/g05_tamper_dll.mjs` | 137 | UNREFERENCED in the required reference set | DELETE |
| `scripts/g05_v2_cancel.mjs` | 55 | UNREFERENCED in the required reference set | DELETE |
| `scripts/g05_vitems_c.mjs` | 184 | UNREFERENCED in the required reference set | DELETE |
| `scripts/g05_vitems_d.mjs` | 144 | UNREFERENCED in the required reference set | DELETE |
| `scripts/lib/catalog_schema.mjs` | 160 | `scripts/tests/catalog_schema.test.mjs:5`; `scripts/tests/fixtures/catalog-schema-cases.json:2` | KEEP |
| `scripts/lib/catalog_window.mjs` | 15 | `scripts/tests/release-gates.test.mjs:2219` | KEEP |
| `scripts/lib/cdp_client.mjs` | 150 | `.github/workflows/release.yml:299` | KEEP |
| `scripts/lib/health_cancel.mjs` | 79 | `scripts/tests/health_cancel.test.mjs:10`; `scripts/tests/release-gates.test.mjs:947` | KEEP |
| `scripts/lib/http_retry.mjs` | 82 | `scripts/tests/http_retry.test.mjs:13`; `scripts/tests/release-gates.test.mjs:2939` | KEEP |
| `scripts/lib/mt06_verdicts.mjs` | 175 | `scripts/tests/mt06_verdicts.test.mjs:12`; `scripts/tests/release-gates.test.mjs:1230` | KEEP |
| `scripts/lib/quant_label.mjs` | 120 | `scripts/tests/release-gates.test.mjs:2170` | KEEP |
| `scripts/measure_s25_packaged.mjs` | 188 | UNREFERENCED in the required reference set | DELETE |
| `scripts/negative_control_console.py` | 55 | UNREFERENCED in the required reference set | DELETE |
| `scripts/validate_catalog.mjs` | 87 | `.github/workflows/catalog.yml:41,78`; `package.json:11`; `scripts/tests/release-gates.test.mjs:178` | KEEP |
| `scripts/verify_041.mjs` | 903 | `scripts/tests/lib/manifest_fixture.mjs:105,117`; `scripts/tests/qualification_manifest.test.mjs:224`; `scripts/tests/release-gates.test.mjs:163,911,916,933,1042,1056,1074,1501,1511,1516`; `scripts/verify_packaged_impl.ps1:297` | MERGE INTO MATRIX |
| `scripts/verify_060_catalog.mjs` | 674 | `scripts/tests/release-gates.test.mjs:987,988,989,990,1024,1527`; `scripts/verify_packaged_impl.ps1:161,169,305,317,326,340` | MERGE INTO MATRIX |
| `scripts/verify_a11y.mjs` | 224 | `.github/workflows/release.yml:465` | MERGE INTO MATRIX |
| `scripts/verify_branding.mjs` | 128 | `package.json:15`; `scripts/tests/release-gates.test.mjs:12,1685` | KEEP |
| `scripts/verify_csp.mjs` | 186 | UNREFERENCED in the required reference set | DELETE |
| `scripts/verify_g05_managed_install.mjs` | 93 | UNREFERENCED in the required reference set | DELETE |
| `scripts/verify_high_contrast.mjs` | 79 | UNREFERENCED in the required reference set | DELETE |
| `scripts/verify_icons.mjs` | 59 | `package.json:14`; `scripts/tests/release-gates.test.mjs:11` | KEEP |
| `scripts/verify_responsive.mjs` | 123 | UNREFERENCED in the required reference set | DELETE |
| `scripts/verify_s23_firstrun.mjs` | 158 | UNREFERENCED in the required reference set | DELETE |
| `scripts/verify_s24_paths.mjs` | 159 | UNREFERENCED in the required reference set | DELETE |
| `scripts/verify_s27_runtime.mjs` | 86 | UNREFERENCED in the required reference set | DELETE |
| `scripts/verify_s27_tune.mjs` | 103 | UNREFERENCED in the required reference set | DELETE |
| `scripts/watch_console_windows.py` | 77 | UNREFERENCED in the required reference set | DELETE |

#### Table notes

- `scripts/g05_fe05v3.mjs`, `scripts/g05_fe16.mjs`, and other drivers appear in `scripts/tests/release-gates.test.mjs:2021-2031` because tests read their source and assert strings; that does not launch them.
- Internal imports outside the caller set: `catalog_schema.mjs` <- `validate_catalog.mjs:4`; `catalog_window.mjs` <- `build_catalog.mjs:11`; `health_cancel.mjs` <- `verify_041.mjs:14`; `http_retry.mjs`, `quant_label.mjs` <- `build_catalog.mjs:9-10`; `mt06_verdicts.mjs` <- `g05_mt06_cycles.mjs:16`; and `cdp_client.mjs` <- 33 scoped drivers, including `g05_cancellation.mjs:4`, `verify_060_catalog.mjs:25`, and `verify_a11y.mjs:14`. These are active support imports.
- The seven MERGE rows are `g05_dc04_override`, `g05_mt06_cycles`, `g05_rt04v2_delay`, `g05_rt06_all_backends`, `verify_041`, `verify_060_catalog`, and `verify_a11y`. Preserve unique assertions; move them behind the matrix and delete standalone campaign entry points.

#### Catalog assets (not scripts)

| File | Lines/bytes | Evidence | Verdict |
|---|---:|---|---|
| `catalog/README.md` | 145 / 6,791 | `release-gates.test.mjs:281,2266,2271,2279,2282,2306` | KEEP as contract; remove stale dry-run instruction |
| `catalog/providers.json` | 1 / 630 | `build_catalog.mjs:13`; `release-gates.test.mjs:175` | KEEP |
| `catalog/catalog.json` | 17,299 / 630,182 | `catalog.yml:41,47,78,81,88,89`; `package.json:11`; `release-gates.test.mjs:174`; cleanup test reads `verify_cleanup_matrix.ps1:125,150,174,198,223,260,284` | KEEP; sign exact reproducible bytes |
| `catalog/catalog.json.sig` | 1 / 89 | `catalog.yml:81,89` | KEEP |

### Findings

#### High

##### LAB-01 — High — Structured runtime-catalog errors are destroyed as `[object Object]`

> **Lead check: VERIFIED.** verify_041.mjs:463 maps each catalog error to 'terminal-error'. The diagnostics artifact records '[object Object]'.

- **Location:** `scripts/verify_041.mjs:96,269-273,471-519`.
- **Evidence:**
  > 269|      return { ok: false, error: String(error) };
  > 273|  if (!result?.ok) throw new Error(result?.error ?? `Tauri command ${command} failed`);
  > 96|    addCheck(id, criterion, "FAIL", {}, error instanceof Error ? error.message : String(error));
- **Why it matters:** Rust returns a serializable `RuntimeCatalogError` object (`src-tauri/src/runtime.rs:2459-2475`). `ipc.runtime-recommendation` (`:471-485`) and `ipc.runtime-catalog-fetch` (`:502-519`) call `invoke`; the page-side catch stringifies the object. The outer check records `[object Object]` with literal empty evidence, losing kind, retry data, and whether the cause was HTTP, timeout, rate limit, or trust failure.
- **Fix:** Centralize a tested CDP error normalizer preserving `{kind,message,retryAfterSeconds}` plus bounded raw JSON. Never use `String(error)` for IPC rejection; require structured evidence for failed IPC checks.

##### LAB-02 — High — A live GitHub outage is reported as product failure

> **Lead check: VERIFIED.** Same evidence as RT-05 and LAB-01.

- **Location:** `scripts/verify_041.mjs:447-519`; `src-tauri/src/runtime.rs:15,1949-1952,2822-2830`.
- **Evidence:**
  > `verify_041.mjs:475` — `invoke("fetch_runtime_catalog", { adapterId }, 120_000)`
  > `runtime.rs:2822-2830` — `approved_release_url()` goes through `fetch_catalog_http(...)`.
  > `verify_041.mjs:80-97` — `runCheck` has only `PASS`/`FAIL` and catches every exception into a check.
- **Why it matters:** The runtime catalog uses `api.github.com` (`runtime.rs:15`, URL at `1949-1952`). Timeout, quota, 5xx, or outage becomes product `FAIL`; there is no tooling/blocked/not-run disposition. `load_runtime_setup` can leave `catalog_result: "terminal-error"` (`verify_041.mjs:447-465`) while the later checks still perform live fetches. This violates the rule that harness failures are tooling failures.
- **Fix:** Use a signed fixture for deterministic qualification, or classify typed network failures as `TOOLING`/`NOT RUN` without setting product overall status to FAIL. Keep a separately named live-smoke job only if required.

##### LAB-03 — High — Three independent CDP clients guarantee behavior drift

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `scripts/lib/cdp_client.mjs:32-149`; `scripts/verify_041.mjs:139-237`; `scripts/drive_console_check.mjs:20-60`.
- **Evidence:**
  > `cdp_client.mjs:42` — `new WebSocket(page.webSocketDebuggerUrl)`
  > `verify_041.mjs:217` — `new WebSocket(page.webSocketDebuggerUrl)`
  > `drive_console_check.mjs:22` — `new WebSocket(url)`
- **Why it matters:** Grep finds **three implementations**, not three call sites: shared client, inline 0.4.1 client, and inline console client. All do target discovery/message routing/`Runtime.evaluate`; only the shared client has rebind/exception collection. The release verifier has a private fork.
- **Fix:** Delete both inline clients and import `scripts/lib/cdp_client.mjs` everywhere. Move target filtering and timeout policy into that client and test it once.

##### LAB-04 — High — Unscoped process-name kills violate the harness safety rule

> **Lead check: VERIFIED.** g05_vitems_c.mjs:109 and :166 stop processes by name.

- **Location:** `scripts/g05_vitems_c.mjs:42,94,109,166`; `scripts/g05_fe16.mjs:41-45,152-157`; `scripts/g05_stop_supervision.mjs:12-25`; `scripts/g05_tamper_dll.mjs:26`; `scripts/g05_rt04v2_delay.mjs:97`.
- **Evidence:**
  > 109|  execSync('powershell -NoProfile -Command "Get-Process llama-server -ErrorAction SilentlyContinue | Stop-Process -Force"');
  > 166|execSync('powershell -NoProfile -Command "Get-Process localmotive-portable,localmotive -ErrorAction SilentlyContinue | Stop-Process -Force; exit 0"');
  > 155|  execSync(`taskkill /F /PID ${killPid}`, { stdio: "pipe" });
- **Why it matters:** These drivers enumerate or kill by image name, not an owned process handle and verified command line. They can stop an operator's unrelated `llama-server`, another run, or another Localmotive instance. `tasklist /FI "IMAGENAME eq llama-server.exe"` appears in seven drivers, so non-kill observations can also attribute another run to the candidate. The project explicitly forbids stopping unrelated processes by name or port.
- **Fix:** Delete unreferenced drivers. Retained coverage must use existing matrix ownership and parent-scoped children/handles; never add a process-name fallback.

##### LAB-05 — High — Operational coverage is scattered across dead one-off drivers and source-only tests

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** reference table; source-only assertions `scripts/tests/release-gates.test.mjs:2021-2031`; active direct lab calls `.github/workflows/release.yml:327,447,453,459,465`.
- **Evidence:**
  > `release-gates.test.mjs:2021-2027` — reads `g05_fe16.mjs`/`g05_fe05v3.mjs` with `readFile`.
  > `verify_packaged_impl.ps1:292-306` — maintained matrix runs `verify_041` and catalog first-fill.
  > `release.yml:447-465` — separate steps invoke MT06, DC04, RT04, and A11Y drivers.
- **Why it matters:** The table has 31 DELETE recommendations. Many only have audit comments or source-string tests; seven campaign drivers bypass the maintained packaged matrix. A green source assertion is not evidence that a packaged candidate was exercised.
- **Fix:** Keep one matrix-owned manifest and runner. Port the seven unique live legs into matrix phases, retain pure helper tests, delete source-only gates for removed scripts, then delete unreferenced drivers.

##### LAB-06 — High — Catalog validation is a two-language copy with an unchecked output-shape gap

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `scripts/lib/catalog_schema.mjs:1-8,63-160`; `src-tauri/src/catalog.rs:362-428`; `scripts/build_catalog.mjs:230-248`; `catalog/catalog.json:1-36`.
- **Evidence:**
  > `catalog_schema.mjs:1-4` — JS says Rust implements the same contract.
  > `catalog.rs:371-372` — Rust says it mirrors the shared JavaScript contract.
  > `build_catalog.mjs:237-238` vs `catalog.json:1-36` — builder emits `sequence`/`expires`, checked-in signed document has neither.
- **Why it matters:** JS and Rust each implement filename, revision, digest, tag, date, duplicate, and empty-catalog rules. That is deliberate and fixture-backed, but still two maintenance surfaces. More concretely, the builder writes signed freshness fields while the checked-in catalog's top-level shape has no such keys. `validate_catalog.mjs` does not require them, so a publish gate can pass a document that does not match the current builder freshness contract. Its other provider/signature/slug/`.gguf` rules also do not equal Rust's row parser.
- **Fix:** Declare one versioned schema source and generate/compare JS and Rust contracts. Require every builder field in `validate_catalog.mjs`, or remove freshness fields until they are signed-schema fields. Add a checked-in builder-shape test without live HF data.

#### Medium

##### LAB-07 — Medium — Fixed sleeps are synchronization protocol

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** inventory below; examples `g05_cancellation.mjs:41,45`, `g05_mt06_cycles.mjs:302,307`, `verify_a11y.mjs:39`, `verify_csp.mjs:39`.
- **Evidence:**
  > `g05_cancellation.mjs:41` — `await settle(1200);`
  > `g05_mt06_cycles.mjs:307` — `await settle(1500);`
  > `verify_a11y.mjs:39` — `await sleep(4_000);`
- **Why it matters:** A constant delay is neither readiness assertion nor proof of state. Slow machines wait too little; fast machines waste time. Timing changes create false red/green results.
- **Fix:** Replace state synchronization with bounded semantic waits: DOM predicates, IPC events, process state, HTTP readiness, or publication. Keep only deliberate pacing delays and name them.

**Fixed-sleep call inventory (helper definitions excluded):**

`scripts/build_catalog.mjs: 169, 171`; `scripts/drive_console_check.mjs: 15, 113`; `scripts/dryrun_catalog.mjs: 82, 91`; `scripts/g05_cancellation.mjs: 41, 45, 56, 66, 68, 76, 87, 99, 109, 119, 124, 146`; `scripts/g05_churn_repro.mjs: 45, 52, 57, 63, 68, 77`; `scripts/g05_dc01.mjs: 23, 35, 68`; `scripts/g05_fe05v3.mjs: 67, 76, 81, 95, 98, 102, 112, 116, 155, 158, 161`; `scripts/g05_fe16.mjs: 59, 64, 74, 77, 79, 85, 87, 97, 100, 116, 120, 123, 126, 147, 149, 204, 216`; `scripts/g05_hardlink_drive.mjs: 20, 30, 44, 72`; `scripts/g05_health.mjs: 19, 34`; `scripts/g05_launch_benchmark.mjs: 37, 41, 50, 52, 67, 91, 96`; `scripts/g05_mt01d.mjs: 30, 35, 49, 53, 64, 81, 119, 121`; `scripts/g05_mt06_cycles.mjs: 82, 87, 139, 215, 302, 307, 321, 337, 339, 341, 348, 360, 383, 403, 422, 478, 480, 484, 497, 513, 523, 545, 568, 582, 640, 644, 657, 670`; `scripts/g05_partial_resume.mjs: 40, 65`; `scripts/g05_partial_retention.mjs: 44, 55, 77, 95, 97, 114`; `scripts/g05_rt04v2_delay.mjs: 106, 110, 152, 175, 215, 248`; `scripts/g05_rt06_all_backends.mjs: 68, 217`; `scripts/g05_run_cancel.mjs: 18, 31, 46, 75`; `scripts/g05_state.mjs: 21, 34`; `scripts/g05_stop_supervision.mjs: 48, 56, 67`; `scripts/g05_tamper_dll.mjs: 49, 60, 82, 89, 104, 110, 126`; `scripts/g05_v2_cancel.mjs: 22, 43`; `scripts/g05_vitems_c.mjs: 32, 52, 54, 58, 60, 64, 66, 68, 80, 82, 88, 93, 104, 113, 123, 138, 142, 154, 163, 165, 167, 172`; `scripts/g05_vitems_d.mjs: 22, 48, 50, 52, 54, 57, 63, 65, 69, 71, 74, 80, 82, 84, 86, 89, 95, 97, 99, 101, 108, 116, 118, 120, 125, 130`; `scripts/lib/cdp_client.mjs: 148`; `scripts/measure_s25_packaged.mjs: 33, 102, 110, 126, 143, 166, 182`; `scripts/negative_control_console.py: 37`; `scripts/verify_041.mjs: 219, 234, 244, 796`; `scripts/verify_060_catalog.mjs: 148, 217, 388, 407`; `scripts/verify_a11y.mjs: 32, 39, 46, 82, 107, 149, 153`; `scripts/verify_csp.mjs: 33, 39, 63, 74, 83, 116`; `scripts/verify_g05_managed_install.mjs: 43, 67`; `scripts/verify_high_contrast.mjs: 23`; `scripts/verify_responsive.mjs: 34, 41, 58, 93`; `scripts/verify_s23_firstrun.mjs: 58, 64, 74, 96, 98, 110`; `scripts/verify_s24_paths.mjs: 44, 52, 54, 56, 86, 101, 114, 142`; `scripts/verify_s27_runtime.mjs: 36`; `scripts/verify_s27_tune.mjs: 36, 46`; `scripts/watch_console_windows.py: 61`

##### LAB-08 — Medium — Visible button text is the selector API

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** selector inventory below, especially `verify_041.mjs:252,427,531`, `g05_vitems_c.mjs:19-29`, `verify_060_catalog.mjs:226-240`.
- **Evidence:**
  > `verify_041.mjs:252` — `node.textContent.trim() === ...`
  > `g05_vitems_c.mjs:29` — broad containers are searched by `textContent` before a descendant button is clicked.
  > `verify_060_catalog.mjs:230` — navigation buttons are found by visible-text `.includes(...)`.
- **Why it matters:** Copy edits, localization, duplicate labels, nesting, or a hidden button changes the target. Visible strings are presentation, not stable contract identifiers.
- **Fix:** Add stable `data-testid`/ARIA identifiers, assert uniqueness, and use role/name only where the accessible name is contractual. Delete selectors for removed one-offs.

**Visible-text selector inventory (file:line):**

`scripts/g05_cancellation.mjs: 13, 14, 24, 25, 31, 32, 59, 126, 130, 135`; `scripts/g05_churn_repro.mjs: 28, 39, 40`; `scripts/g05_dc01.mjs: 14, 15, 28, 29`; `scripts/g05_fe05v3.mjs: 26, 32, 77`; `scripts/g05_fe16.mjs: 28, 30, 36, 38`; `scripts/g05_hardlink_drive.mjs: 14, 15, 24, 25, 33, 34, 66, 67`; `scripts/g05_health.mjs: 13, 14, 22, 23, 43`; `scripts/g05_launch_benchmark.mjs: 15, 16, 25, 26, 44, 57, 58`; `scripts/g05_mt01d.mjs: 13, 14, 87, 88`; `scripts/g05_mt06_cycles.mjs: 42, 43, 52, 53`; `scripts/g05_partial_resume.mjs: 28, 29, 59`; `scripts/g05_partial_retention.mjs: 28, 38, 39, 61, 65, 66, 69`; `scripts/g05_rt04v2_delay.mjs: 105, 107`; `scripts/g05_run_cancel.mjs: 12, 13, 34, 35, 48, 49, 51, 52, 58, 59, 84, 85`; `scripts/g05_state.mjs: 15, 16, 26, 39`; `scripts/g05_stop_supervision.mjs: 31, 41, 42`; `scripts/g05_tamper_dll.mjs: 45, 59, 81, 83, 103, 105, 108, 122`; `scripts/g05_v2_cancel.mjs: 11, 12, 24, 25, 31, 32`; `scripts/g05_vitems_c.mjs: 19, 23, 25, 29`; `scripts/g05_vitems_d.mjs: 14, 16`; `scripts/measure_s25_packaged.mjs: 21, 22, 114, 115, 121, 137, 138, 146, 147, 155`; `scripts/verify_041.mjs: 252, 427, 436, 531, 579, 580`; `scripts/verify_060_catalog.mjs: 226, 230, 240`; `scripts/verify_a11y.mjs: 110, 143, 144, 155, 162`; `scripts/verify_csp.mjs: 54, 59, 76, 77`; `scripts/verify_g05_managed_install.mjs: 19, 34, 35, 55`; `scripts/verify_high_contrast.mjs: 29`; `scripts/verify_s23_firstrun.mjs: 30, 31, 41, 42`; `scripts/verify_s24_paths.mjs: 25, 26, 34, 35, 85, 88`; `scripts/verify_s27_runtime.mjs: 26, 27, 52, 53, 55, 64`; `scripts/verify_s27_tune.mjs: 26, 27, 86, 87, 89`

##### LAB-09 — Medium — Hard-coded ports and path defaults make runs collide or test the wrong machine

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `drive_console_check.mjs:4,104,116`; `g05_mt06_cycles.mjs:104,132`; `g05_tamper_dll.mjs:17`; `g05_vitems_c.mjs:94`; `verify_a11y.mjs:17`; `verify_csp.mjs:12`; `verify_responsive.mjs:13`.
- **Evidence:**
  > `drive_console_check.mjs:4` — `const PORT = process.argv[2] || '10011';`
  > `g05_mt06_cycles.mjs:104,132` — metrics/liveness fixed to `127.0.0.1:8080`.
  > `g05_tamper_dll.mjs:17` — `process.argv[2] ?? 10070` selects a stale CDP port.
- **Why it matters:** Busy ports attach to the wrong process or fail late. MT06 assumes metrics on 8080 despite the app's selected server port. Drive-console defaults to `C:\llama`, `C:\models`, and port 8137 (`:5,104,116`), not matrix-owned inputs. Standalone UI probes choose an unreserved 57000-57899 range.
- **Fix:** Pass endpoints/paths from attempt-scoped matrix state, reserve ports through one owner, and derive metrics from the returned process contract. Delete stale defaults.

**Port inventory in scoped scripts:**

- `scripts/drive_console_check.mjs` — 4: const PORT = process.argv[2] || '10011';; 104: port: 8137,; 116: `fetch('http://127.0.0.1:8137/health').then(r => r.text(), e => 'ERR ' + e)`,
- `scripts/g05_mt06_cycles.mjs` — 104: port: 8080,; 132: const socket = connect({ host: "127.0.0.1", port: 8080 });
- `scripts/g05_tamper_dll.mjs` — 17: const PORT = Number(process.argv[2] ?? 10070);
- `scripts/g05_vitems_c.mjs` — 94: const log = execSync(`powershell -NoProfile -Command "Get-ChildItem $env:TEMP\\localmotive\\server-8080-*.log | Sort-Object LastWriteTime | Select-Object -Last 1 | ForEach-Object FullName"`, { encoding: "utf8" }).trim();
- `scripts/verify_a11y.mjs` — 17: const port = 57000 + Math.floor(Math.random() * 900);
- `scripts/verify_csp.mjs` — 12: const port = 57000 + Math.floor(Math.random() * 900);
- `scripts/verify_responsive.mjs` — 13: const port = 57000 + Math.floor(Math.random() * 900);

There is no `10013` literal in scoped scripts. It appears in the prescribed command `AGENTS.md:112` and historical documentation `docs/history/TODO-0.4.md:479-480`; it is an invocation choice, not script-owned.

##### LAB-10 — Medium — Campaign and version names have become architecture

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** headers/output paths in `scripts/g05_*.mjs`; `verify_041.mjs:19`; `verify_060_catalog.mjs:1-17`; `measure_s25_packaged.mjs:1-7`; S-23/S-24/S-27 verifiers at their headers.
- **Evidence:**
  > `verify_041.mjs:19` — output is `release-evidence/0.4.1/packaged-verification.json`.
  > `verify_060_catalog.mjs:1-17` — explicitly a version-aware 0.6 verifier.
  > `g05_vitems_c.mjs:1-4` — “High-finding campaign C” is in the filename/header.
- **Why it matters:** `g05`, `dc04`, `fe16`, `mt06`, `rt04v2`, `rt06`, `vitems_c/d`, `s23/24/25/27`, `041`, and `060` are audit/work-item labels, not stable capabilities. They force each release to rediscover which old campaign still matters and create the one-verifier-per-release failure mode.
- **Fix:** Move durable assertions to capability-named matrix phases (`catalog`, `runtime-install`, `health-cancel`, `accessibility`, `lifecycle`) and bind evidence to version/source/digest in data, not filenames.

**Campaign disposition:**

- **Delete after retaining replacement coverage:** unreferenced cancellation variants (`g05_cancellation`, `g05_run_cancel`, `g05_v2_cancel`), health/state/launch/stop/tamper/hardlink drivers, DC-01, VITEMS C/D, partial-download drivers, FE-05/FE-16 drivers, `measure_s25_packaged`, `verify_csp`, `verify_g05_managed_install`, `verify_high_contrast`, `verify_responsive`, `verify_s23_firstrun`, `verify_s24_paths`, `verify_s27_runtime`, `verify_s27_tune`, `drive_console_check`, `watch_console_windows`, and `negative_control_console`.
- **Merge then delete campaign filename:** `g05_dc04_override`, `g05_mt06_cycles`, `g05_rt04v2_delay`, `g05_rt06_all_backends`, `verify_041`, `verify_060_catalog`, and `verify_a11y`.
- **Keep stable support:** catalog build/validation/shared helpers, branding, icons, and one shared CDP client.

##### LAB-11 — Medium — `dryrun_catalog.mjs` duplicates a stale builder path

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `dryrun_catalog.mjs:1-18,37-91`; `build_catalog.mjs:4-7,17,174-206`; `catalog/README.md:123-131`.
- **Evidence:**
  > `dryrun_catalog.mjs:16-17` — cutoff is literal `2026-09-10T00:00:00Z`.
  > `build_catalog.mjs:4-7` — builder already documents `--dry-run`.
  > `build_catalog.mjs:19-21` — maintained builder uses rolling cutoff.
- **Why it matters:** The standalone file is unreferenced, has separate failure behavior, and freezes a date that `catalog_window.mjs` was created to remove. README still advertises the dead path, so operators get different cutoff semantics from CI.
- **Fix:** Delete it; make `node scripts/build_catalog.mjs --dry-run` the only command; update README/tests.

##### LAB-12 — Medium — Catalog publication rewrites 630 KB with tie-dependent ordering

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `build_catalog.mjs:164,215-248,259-264`; `catalog/catalog.json`.
- **Evidence:**
  > `build_catalog.mjs:246,248` — sort then whole-object `JSON.stringify(..., null, 2)`.
  > `build_catalog.mjs:262-264` — temporary file replaces `catalog/catalog.json` wholesale.
  > current catalog — 50 tied file sizes and 2 tied model-download counts.
- **Why it matters:** `sequence: Date.now()`/`expires` change every build (`build_catalog.mjs:230-238`), so unchanged content changes signed bytes. Files sort only by size and models only by downloads; equal keys inherit upstream order. This creates large review diffs and possible semantic reordering.
- **Fix:** Add deterministic secondary keys (`repo/id`, filename/revision), separate freshness metadata if protocol permits, and have validation assert ordering/diff policy.

#### Low

##### LAB-13 — Low — Error coercion is repeated outside the known `verify_041` path

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `drive_console_check.mjs:111,124,132`; `dryrun_catalog.mjs:44`; `g05_mt06_cycles.mjs:214`; `g05_rt04v2_delay.mjs:165`; `g05_rt06_all_backends.mjs:42,108`; `verify_a11y.mjs:221`; `verify_csp.mjs:178`; `verify_responsive.mjs:120`.
- **Evidence:**
  > `g05_rt06_all_backends.mjs:42` — `error: String(error)` crosses page boundary.
  > `verify_a11y.mjs:221` — `A11Y_ERROR ${String(error)}` is the diagnostic.
  > `dryrun_catalog.mjs:44` — per-author record stores `String(error)`.
- **Why it matters:** Structured exceptions become opaque strings and reader failures become indistinguishable from product assertions. These paths are not all matrix-active, but future promotion would repeat LAB-01.
- **Fix:** Reuse one bounded formatter preserving error name/kind/message/cause and bounded stack/JSON. Add a rejected-object negative test and assert output is not `[object Object]`.

### Answers to specific questions

#### 1. Reference status and verdicts

The complete file-by-file answer is the table above: 49 scripts, exact line counts, exact caller file/line matches, and KEEP/MERGE INTO MATRIX/DELETE. The operational/textual distinction matters: `release-gates.test.mjs:2021-2031` reads FE scripts and asserts source strings; it does not launch them. Maintained packaged orchestration is `scripts/verify_packaged_matrix.ps1:41-64` and `scripts/verify_packaged_impl.ps1:292-344`; direct G-05 legs remain in `.github/workflows/release.yml:327,447-466`.

#### 2. Independent CDP client count

There are **three independent implementations**:

1. `scripts/lib/cdp_client.mjs:32-148` — shared; `new WebSocket` at line 42 and `Runtime.evaluate` at lines 13-29.
2. `scripts/verify_041.mjs:139-237` — inline; `new WebSocket` at line 217 and `Runtime.evaluate` at line 178.
3. `scripts/drive_console_check.mjs:20-60` — inline; `new WebSocket` at line 22 and `Runtime.evaluate` at line 51.

`webSocketDebuggerUrl` appears in `drive_console_check.mjs:12`, `lib/cdp_client.mjs:38,42`, and `verify_041.mjs:213,217`. Every driver should use `scripts/lib/cdp_client.mjs`; it already provides retry/rebind and exception collection (`:95-148`).

#### 3. Fragility inventory

##### Fixed sleeps

The exhaustive file/line inventory is under LAB-07. Release-relevant fixed waits include `verify_041.mjs:234,244`, `verify_060_catalog.mjs:148,217,388,407`, `verify_a11y.mjs:32,39,46,82,107,149,153`, and the active G-05 calls listed there. Fixed numeric calls include catalog pacing, every G-05 `settle(...)`, MT06 probes, UI verifiers, and Python watcher/negative-control sleeps.

##### Hard-coded ports

The literal inventory is under LAB-09. Fixed ports are 10011/8137 (`drive_console_check`), 8080 (`g05_mt06_cycles` metrics), 10070 (`g05_tamper_dll` default), and `server-8080` (`g05_vitems_c`). Standalone UI probes choose an unreserved 57000-57899 range. Prescribed 10013 is not hardcoded by a scoped script.

##### Selectors by visible button text

The full file/line inventory is under LAB-08. It spans G-05 drivers, `verify_041`, `verify_060_catalog`, accessibility/CSP/responsive probes, S-23/S-24/S-27 probes, and S-25 measurement. The recurring pattern is `querySelectorAll("button")` plus `textContent.trim() === ...` or `.includes(...)`.

##### `String(error)` / object-to-string rendering

Exact scoped locations are:

`scripts/drive_console_check.mjs: 111, 124, 132`; `scripts/dryrun_catalog.mjs: 44`; `scripts/g05_mt06_cycles.mjs: 214`; `scripts/g05_rt04v2_delay.mjs: 165`; `scripts/g05_rt06_all_backends.mjs: 42, 108`; `scripts/verify_041.mjs: 86, 96, 233, 270, 300, 301, 305, 820, 824, 882, 894`; `scripts/verify_a11y.mjs: 221`; `scripts/verify_csp.mjs: 178`; `scripts/verify_responsive.mjs: 120`

The highest-impact path is `verify_041.mjs:269-273`, inside the page before the error reaches `runCheck`. `error.message` is not sufficient for Tauri's structured object rejection; use explicit serialization.

##### Process-name/PID kills and global scans

Exact kill locations are:

- `scripts/g05_fe16.mjs` — 155: execSync(`taskkill /F /PID ${killPid}`, { stdio: "pipe" });; 157: check("fe16.v3.unexpected-exit-kill", false, `taskkill failed: ${error.message}`);
- `scripts/g05_vitems_c.mjs` — 109: execSync('powershell -NoProfile -Command "Get-Process llama-server -ErrorAction SilentlyContinue | Stop-Process -Force"');; 166: execSync('powershell -NoProfile -Command "Get-Process localmotive-portable,localmotive -ErrorAction SilentlyContinue | Stop-Process -Force; exit 0"');

Global image-name scans also occur at `g05_churn_repro.mjs:14`, `g05_fe16.mjs:43`, `g05_rt04v2_delay.mjs:97`, `g05_stop_supervision.mjs:15,50`, `g05_tamper_dll.mjs:26`, `g05_vitems_c.mjs:42`, and `g05_vitems_d.mjs:37`. These are observations rather than kills, but can attribute another process to the candidate. `verify_060_catalog.mjs:491-500` also calls `process.kill(state.serverPid)` from a state-file PID without validating ownership.

#### 4. `verify_041.mjs` trace for the three runtime checks

- `ipc.runtime-recommendation`: `verify_041.mjs:471-485`, calling `invoke("fetch_runtime_catalog", ...)` at line 475.
- `ipc.reject-unknown-adapter`: `:488-500`, calling `rejectedInvoke(...)` at line 492.
- `ipc.runtime-catalog-fetch`: `:502-519`, calling `invoke("fetch_runtime_catalog", ...)` at line 507.

For the two `invoke` checks: `verify_041.mjs:261-272` evaluates a page expression and catches the Tauri rejection; `:269-270` returns `error: String(error)`. Rust's `RuntimeCatalogError` is an object with `kind`, `message`, and `retry_after_seconds` (`src-tauri/src/runtime.rs:2470-2475`), so coercion is `[object Object]`. `:273` throws `new Error(result.error)`, and `runCheck` at `:90-98` records `FAIL`, `{}` evidence, and that message.

The unknown-adapter path is different in the checked-in file: `verify_041.mjs:285-305` JSON-serializes the caught object and extracts `kind/message/retryAfterSeconds`; Rust's rejection is explicit at `src-tauri/src/runtime_service.rs:103-108` with `InvalidResponse`. Therefore the all-three claim is **not proven** by current source. A record saying all three produced `[object Object]` came from an older verifier, another artifact, or an unexpected exception outside the guarded catch. Preserve that discrepancy as a harness failure and rerun after fixing the reader.

The checks depend on the live GitHub API: `runtime.rs:15` names the GitHub tag endpoint, `approved_release_url` is built at `:1949-1952`, and `fetch_catalog` performs the HTTP request at `:2822-2830`. A network failure is misreported as product `FAIL` because `runCheck` has no tooling disposition and overall status is based on every check being `PASS` (`verify_041.mjs:322-348`).

#### 5. Campaign/version naming that should be removed

Headers explicitly name G-05, DC-01/DC-04, FE-05/FE-16, MT-01/MT-06, RT-04/RT-06, VITEMS C/D, S-23/S-24/S-25/S-27, and versioned `041`/`060`. The disposition is:

- **Delete after retaining replacement coverage:** all unreferenced `g05_*` drivers listed DELETE in the table, `measure_s25_packaged`, `verify_csp`, `verify_g05_managed_install`, `verify_high_contrast`, `verify_responsive`, `verify_s23_firstrun`, `verify_s24_paths`, `verify_s27_runtime`, `verify_s27_tune`, `drive_console_check`, `watch_console_windows`, and `negative_control_console`.
- **Merge then delete the campaign filename:** `g05_dc04_override`, `g05_mt06_cycles`, `g05_rt04v2_delay`, `g05_rt06_all_backends`, `verify_041`, `verify_060_catalog`, and `verify_a11y`, because workflow/matrix lines prove they still carry live coverage.
- **Keep:** `scripts/lib/*`, catalog build/validation, branding, and icons because they are capability/tooling names with active imports or package commands.

#### 6. Catalog tooling duplication, size, and churn

- **`catalog_schema` vs Rust:** yes, row validation is implemented twice. JS says Rust mirrors it at `catalog_schema.mjs:1-8`; Rust says the same at `catalog.rs:371-372`; fixture `scripts/tests/fixtures/catalog-schema-cases.json:2` backs the shared cases. This is not proof of disagreement, but it is a two-language drift surface.
- **`validate_catalog.mjs` vs Rust:** not a full duplicate. `validate_catalog.mjs:26-48,50-85` adds publish rules (schema version, provider allowlist, detached signature, stable slugs, `.gguf`, full ISO dates, zero shared-contract drops). Rust `parse_catalog` at `catalog.rs:362-428` performs application row parsing, dropping invalid rows and rejecting duplicates. Boundaries need to be explicit.
- **`build_catalog.mjs` vs Rust:** not duplicates: builder curates/pins HF data (`build_catalog.mjs:60-167,230-248`), Rust consumes/verifies the signed artifact. The freshness-field mismatch is a contract gap.
- **`dryrun_catalog.mjs`:** obsolete duplicate. Builder documents `--dry-run` (`build_catalog.mjs:4-7`) and uses rolling cutoff (`:19-21`); standalone is unreferenced and freezes `2026-09-10` (`dryrun_catalog.mjs:16-17`).
- **Size/format:** 630,182 bytes and 17,299 lines; 158 models and 1,417 files. Two-space pretty JSON, whole-file atomic replacement (`build_catalog.mjs:248,262-264`). Models sort only by downloads and files only by size; current data has 2 tied model-download pairs and 50 tied file-size pairs. Equal-key ordering can inherit upstream order and cause churn.

### Cross-cutting observations

1. The repository has the beginnings of the right architecture: shared CDP, pure MT06/health/catalog helpers, and owned-handle PowerShell cleanup. Old campaign drivers and direct workflow legs remain beside it.
2. “Referenced” is confused with “executed.” Source-reading tests protect historical text but do not qualify a packaged binary.
3. Matrix ownership safeguards (`verify_packaged_impl.ps1:148-213,353-367`) are stronger than the PID kill in `verify_060_catalog.mjs:491-500`; the latter must not independently kill a PID.
4. Two orchestration styles remain: maintained matrix and direct lab steps in `release.yml:327,447-466`. That split explains campaign-name sprawl and divergent error handling.
5. Minimum safe consolidation is deletion-first: normalize errors/CDP transport, port seven live legs into the matrix, delete the 31 dead/duplicate scripts, remove source-only gates for removed scripts, and make catalog output deterministic.

### Review scope and read status

Fully read: every requested `scripts/g05_*.mjs`, verifier/catalog/console script, and current `scripts/lib/*.mjs`; `catalog/README.md`, `catalog/providers.json`, and `catalog/catalog.json.sig` were fully read. `catalog/catalog.json` was intentionally **not fully read** under the requested protocol: first 300 and last 100 lines were read and its complete line/byte/JSON structure characterized (`wc -l -c`: 17,299 / 630,182; JSON parse: 158 models / 1,417 files).

---

## Part 3 — Repository hygiene, documentation and process (finding prefix DOC)

### Summary

This is a read-only review of the Localmotive checkout at `9b098577095ab80ead9bff6e5475456166b340c1` on `fix/u06-stabilization`. No repository file, Git ref, workflow, application, package manager, Rust toolchain, or network service was run or changed. The working tree had three untracked 0.6.0 installer candidates under `artifacts/`; their names and sizes were listed only.

The central failure is process identity, not a lack of written policy. The repository has a current tracker, a second file declaring itself an active residual tracker, multiple evidence matrices that point at different trackers, a qualification map whose title and main records are 0.6.0 while its identity section discusses 0.6.1, and public documentation that calls an unpublished line a release. The release workflow then checks a fixed tag tree while digest-bound evidence was committed after that tag. `TODO.md` records the resulting 12 missing records and says the tag cannot move. The policy is internally coherent only after several owner amendments; the checked-in documentation is not.

Measured anchors:

- The version manifests all say `0.6.1`: `package.json:4`, `src-tauri/Cargo.toml:4`, and `src-tauri/tauri.conf.json:4`.
- Local tags exist for `v0.5.0`, `v0.6.0`, and `v0.6.1`; their peeled commits are respectively `a4b7127f739f7420232d9b6f63da693d39128d0b`, `27349f93d7d17eac9b15a6895cd94e850a122a8b`, and `4b31431d28d1503efc7b80a46c77ad2c3d54f082`.
- The task context identifies v0.5.0 as the last published release. The repository itself says `Nothing published` for the 0.6.1 campaign (`TODO.md:127`) and keeps U06-06/U06-07/U06-08 open (`TODO.md:115-161`).
- There are 16 strict tracker/ledger/report files under the root and `docs/`: 7 trackers, 5 matrices/ledgers, and 4 audit/review reports. Broadly counting `CHANGELOG.md`, `agents_feedback.md`, and `ideas.md` as logs/backlogs gives 19 planning/status files. Historical files are valid frozen records, but several current documents still point at them as operational authority.
- The repository tree has 6,005 tracked files and 329,944,028 tracked bytes. `artifacts/` is 5,494 files and 322,015,644 bytes: 91.49% of files and 97.60% of bytes. `release-evidence/` is 247 files and 1,593,304 bytes: 4.11% of files and 0.48% of bytes. Together they are 95.60% of tracked files and 98.08% of tracked bytes.
- `artifacts/` has 4,464 files with no extension, 533 `.log`, 136 `.json`, 66 `.db`, 22 `.gguf`, 13 `.zip`, and 66 additional database sidecars. It contains 12 `a11y-*` and 30 `native-*` WebView/profile directories. The name-only inventory contains 33 copies each of `Cookies`, `Login Data`, `History`, `Web Data`, `Local State`, `Secure Preferences`, `Network Persistent State`, `Trust Tokens`, `Trust Tokens-journal`, `Vpn Tokens`, and `Vpn Tokens-journal` (363 browser-store paths), plus `token-bound-mutant.log` (364 token-named paths total). No credential, cookie, history, token, or browser-store contents were opened.
- The same raw-output problem appears in Git history: since the published v0.5.0 base, `git log --format='%ad %s' --date=short a4b7127f739f7420232d9b6f63da693d39128d0b..HEAD` returns 338 commits. Under the classification described below, 105 are docs/evidence-only, 123 touch product code, 76 are automation/tooling-containing without product code, 33 are merge/no-path commits, and 1 is other. Documentation/evidence-only work is 31.1% of the range; docs-only commits alone are 86.

The release process is therefore spending a large share of its change budget on ledgers, captured environments, evidence rebinding, and policy reconciliation while the one user-visible result remains unpublished. That is the process overhead this review treats as a release blocker.

### Per-file verdicts

Sizes are tracked tree bytes from `git ls-tree -r -l HEAD`. `KEEP` means retain as reference or accepted evidence, not that it is current operational authority. `ARCHIVE` means move behind an explicit historical boundary or make the historical status unmistakable. `MERGE -> DELETE` means extract any unique current obligations into `TODO.md`, then remove the competing tracker. The verdicts do not authorize deletion of published history; owner approval is required for history rewrites.

#### Root files

| File | Size | Verdict | One-line reason |
|---|---:|---|---|
| `README.md` | 22,003 B | FIX | Current support and unsigned-policy disclosures are useful, but 0.6.0/0.6.1 release status and version scope are wrong or ambiguous. |
| `CHANGELOG.md` | 35,006 B | FIX | Published 0.4.x/0.5.0 history is useful; 0.6.0/0.6.1 are presented as release sections without an Unreleased boundary and one pointer targets a missing root tracker. |
| `CONTRIBUTING.md` | 2,558 B | FIX | Duplicates the gate suite in `AGENTS.md` and presents a separate command list that encourages repeated checks. |
| `SECURITY.md` | 2,148 B | KEEP | Concise reporting, secret, checksum, unsigned-release, and review-limit boundaries are consistent with current policy. |
| `AGENTS.md` | 10,742 B | FIX | It correctly states one-tracker and identity rules, but the repository violates its own rules and the checked-in workflow cannot satisfy the tag/evidence contract. |
| `TODO.md` | 48,192 B | FIX | It is the only intended current tracker, but it embeds stale 0.6.0 instructions, 0.6.1 amendments, huge evidence prose, and no short next-action block. |
| `agents_feedback.md` | 2,327 B | ARCHIVE | This is a dated log and risk snapshot, not the current tracker; its useful lessons are already said to live in `AGENTS.md`. |
| `ideas.md` | 455 B | MERGE -> DELETE | An unowned eight-line backlog is a second planning surface and conflicts with the one-tracker rule. |
| `.gitignore` | 412 B | FIX | It ignores `research/`, but not `artifacts/`, WebView profiles, raw audit captures, or local release candidates. |
| `.gitattributes` | 1,450 B | FIX | It preserves digest-sensitive files, but still names deleted `src-tauri/tauri.signing.conf.json` and does not prevent raw artifact capture. |
| `TODO-0.6-post-076a3eeecbdf.md` | 74,398 B | MERGE -> DELETE | It explicitly declares itself an active residual-work tracker, directly violating `AGENTS.md:183-189`. |
| `REPORT-0.6.md` | 68,223 B | ARCHIVE | A freeze-era report with a missing handoff reference and obsolete candidate snapshots; it should not sit beside the current tracker as if current. |

#### `docs/` files

| File | Size | Verdict | One-line reason |
|---|---:|---|---|
| `docs/ARTIFACT-IDENTITY.md` | 5,491 B | KEEP | Clear identity-level contract; it distinguishes filename/header/content identity without acting as a release tracker. |
| `docs/CATALOG-PROMOTION.md` | 4,650 B | KEEP | The manual catalog-signing handoff is explicit and distinct from application-release promotion. |
| `docs/EVIDENCE-MATRIX.md` | 6,886 B | FIX | It calls a missing root `TODO-0.6.md` authoritative and labels 0.6.0 in development/NOT RUN while other current documents report measured 0.6 evidence. |
| `docs/Future_branding.md` | 5,230 B | KEEP | Completed branding/migration record; not a current release tracker. |
| `docs/OPTION_MAP.md` | 3,651 B | KEEP | Correctly calls the imported option map historical and the selected runtime `--help` authoritative. |
| `docs/PRODUCT.md` | 4,517 B | FIX | Product target language lists CPU/NVIDIA/AMD/Intel and several backend archives without consistently foregrounding the support matrix's untested status. |
| `docs/QUALIFICATION-MAP-0.6.md` | 7,993 B | FIX | Title, main map, candidate, and tag are 0.6.0 while the four-identity section discusses 0.6.1; this is a second release map, not a generated current view. |
| `docs/RUNTIME_MANAGER.md` | 5,129 B | FIX | It mixes explicitly historical 0.4.1 counts and support contract language into a current runtime-policy document. |
| `docs/SECURITY-REVIEW-LIMITS.md` | 5,710 B | KEEP | Its scope limits are unusually explicit; retain it as a dated security record and do not treat it as a clean-history certificate. |
| `docs/SUPPLY-CHAIN.md` | 4,827 B | KEEP | Current unsigned/checksum/attestation limitations are stated without claiming publisher authenticity. |
| `docs/SUPPORT-MATRIX.md` | 6,383 B | FIX | It is the right support authority, but its per-finding pointer still names frozen `docs/history/TODO-0.6.md` and its 0.6.0 status is not reconciled with the current 0.6.1 line. |
| `docs/qualification-tests.md` | 16,076 B | ARCHIVE | It declares itself a frozen 0.4.1 snapshot while current navigation still points engineers to it as a qualification reference. |
| `docs/RELEASE-REVIEW-0.6.md` | 52,025 B | ARCHIVE | Historical owner-facing review, with a superseded recipe and a false “active residual tracker” pointer; it is unsafe as current release guidance. |
| `docs/localmotive-0.6-followup-review-db548c8.md` | 30,369 B | ARCHIVE | Useful independent review history, but it is a second release review and records an older repository/governance snapshot. |
| `docs/LLAMA-SERVER-README.md` | 107,107 B | KEEP | Vendored upstream reference; retain the provenance warning and never use it as the runtime capability authority. |
| `docs/history/TODO-0.4.1.md` | 109,207 B | KEEP | Frozen 0.4.1 audit/remediation record; current documents must stop treating it as active. |
| `docs/history/TODO-0.4.md` | 44,835 B | KEEP | Frozen 0.4.0 tracker and evidence ledger; preserve as history only. |
| `docs/history/TODO-0.5.md` | 39,383 B | KEEP | Frozen shipped v0.5.0 ship ledger; useful for published-release provenance. |
| `docs/history/TODO-0.6.md` | 681,765 B | KEEP | Frozen 4,246-line 0.6.0 audit tracker; do not delete accepted history, but remove all current-authority pointers to it. |
| `docs/history/TODO.md` | 14,747 B | KEEP | Frozen v0.3 implementation tracker; its “authoritative” language is historical and must not be read as current. |
| `docs/history/localmotive-comprehensive-audit.md` | 329,370 B | KEEP | Frozen 2,264-line source audit; retain for traceability, not for current task status. |

#### Other reviewed identity/reference files

| File | Size | Verdict | One-line reason |
|---|---:|---|---|
| `assets/app-icon.svg` | 495 B | KEEP | Small source asset; no hygiene or process defect found. |
| `package.json` | 1,963 B | KEEP | Current version source and command surface; its `check` composition is evidence for consolidating contributor instructions. |
| `src-tauri/Cargo.toml` | 2,703 B | KEEP | Current Rust version identity and dependency manifest. |
| `src-tauri/tauri.conf.json` | 1,594 B | KEEP | Current Tauri version/product identity manifest. |

#### Generated/raw evidence buckets

| Path | Size | Verdict | One-line reason |
|---|---:|---|---|
| `artifacts/**` | 5,494 tracked files / 322,015,644 B; 5,497 working files / 356,094,918 B | DELETE from index/history after owner approval | Raw audit dumps, logs, zips, diffs, model fixtures, SQLite/WebView profiles, and unneeded capture trees dominate the repository; retain only a small allowlist of current candidate identity records if required. |
| `release-evidence/**` | 247 tracked files / 1,593,304 B | KEEP with allowlist review | This is the intended accepted-evidence store; keep manifest, checksum, lifecycle, preservation, and public-readback records, but review raw logs/SQLite files for privacy before retention. |

Long-file characterization, without claiming full reads: `TODO-0.6-post-076a3eeecbdf.md` is 475 lines/74,398 B; `REPORT-0.6.md` 792/68,223; `docs/RELEASE-REVIEW-0.6.md` 323/52,025; `docs/localmotive-0.6-followup-review-db548c8.md` 267/30,369; `docs/LLAMA-SERVER-README.md` 2,143/107,107; `docs/history/TODO-0.4.1.md` 2,421/109,207; `docs/history/TODO-0.4.md` 657/44,835; `docs/history/TODO-0.5.md` 489/39,383; `docs/history/TODO-0.6.md` 4,246/681,765; `docs/history/TODO.md` 335/14,747; and `docs/history/localmotive-comprehensive-audit.md` 2,264/329,370. The first 150 lines plus heading indexes were read for these files, as requested; they were not fully read.

### Findings

#### Critical

##### DOC-01 — High (lead; reviewer said Critical) — Tracked browser-profile databases and credential-store filenames are a privacy exposure

> **Lead check: CORRECTED.** Count-only scan: 0 cookie, login, and autofill rows. The History rows are local. The Local State keys are DPAPI-wrapped for the owner's account.

- **Location:** `artifacts/**` name-only inventory; `.gitignore:25-29`; `AGENTS.md:29-35`.
- **Evidence:**
  > `AGENTS.md:31-35` — “Never put secrets in files, logs, browser storage, command lines, or frontend state.”
  > `.gitignore:25-29` ignores `.env`, key, and PEM patterns, but has no `artifacts/` or WebView-profile rule.
- **Measured fact:** 33 copies of each of 11 browser-store names are tracked: `Cookies`, `Login Data`, `History`, `Web Data`, `Local State`, `Secure Preferences`, `Network Persistent State`, `Trust Tokens`, both `Trust Tokens-journal` and `Vpn Tokens`, plus `Vpn Tokens-journal`; 363 names total. One additional token-named log is tracked. The contents were deliberately not opened.
- **Why it matters:** These are credential, session, history, and browser state stores by filename. The inventory proves privacy exposure risk, not that a usable credential is present; content is **UNKNOWN by scope**. A public repository should not carry raw browser profiles regardless of whether the stores happen to be empty.
- **Concrete fix:** Stop committing raw profile directories. Add ignore rules for raw capture trees, remove them from the index and then request owner approval before any history rewrite. Preserve only sanitized, purpose-built JSON/log attestations after a privacy review. Rotate any credential that a separate authorized review finds in the history.

#### High

##### DOC-02 — `artifacts/` is repository-scale raw capture, not release evidence

> **Lead check: VERIFIED.** 5,494 of 6,005 tracked files are in artifacts/.

- **Location:** `artifacts/**`; `.gitignore:1-16`; commit `6099c6f` (all named audit trees).
- **Evidence:**
  > `artifacts/` is 5,494/6,005 tracked files and 322,015,644/329,944,028 tracked bytes.
  > The dominant trees are `ideas-audit-*` (1,301 files/79,997,768 B), `tune-input-audit-*` (823/49,210,889 B), and `lora-audit-*` (806/47,918,319 B).
- **Why it matters:** 97.60% of repository bytes are test-session output. It slows clone, indexing, review, secret scanning, and CI checkout while making current release artifacts indistinguishable from old audit captures. The three untracked 0.6.0 installers add 34,079,274 working-tree bytes and are not ignored.
- **Concrete fix:** Keep `release-evidence/` as the accepted-evidence boundary. Delete raw `artifacts/` capture trees from the index after owner approval; retain only candidate inventory, checksums, packaged-verification records, and any explicitly required public-readback record. Do not rewrite Git history without owner authorization.

##### DOC-03 — The one-tracker rule is contradicted by active and “authoritative” pointers

> **Lead check: VERIFIED.** TODO-0.6-post-076a3eeecbdf.md and other trackers exist beside TODO.md.

- **Location:** `AGENTS.md:183-189`; `TODO.md:3-6`; `TODO-0.6-post-076a3eeecbdf.md:9-19`; `docs/RELEASE-REVIEW-0.6.md:3-8`; `docs/EVIDENCE-MATRIX.md:5-8`; `docs/SUPPORT-MATRIX.md:8-10`.
- **Evidence:**
  > `AGENTS.md:183-189` — “Use `TODO.md` as the current tracker… Use one tracker only… Do not create another release tracker.”
  > `TODO-0.6-post-076a3eeecbdf.md:9-11` — “This is the active residual-work tracker.”
  > `docs/RELEASE-REVIEW-0.6.md:3` calls that second file the “active residual-work tracker.”
- **Why it matters:** A new engineer cannot know whether `TODO.md`, the root post-baseline file, the old root `TODO-0.6.md` named by other docs, or `docs/history/TODO-0.6.md` controls the release. The historical records are not the problem; current documents presenting them as current authority are.
- **Concrete fix:** Make `TODO.md` the only operational status file. Merge only still-open obligations from the root post-baseline tracker and current maps into it, then delete or explicitly archive the competing file. Replace every current pointer to root `TODO-0.6.md` or historical TODO authority with a `TODO.md` anchor. Keep frozen history unchanged.

##### DOC-04 — Public documentation presents unpublished 0.6.x as a release

> **Lead check: VERIFIED.** README.md:17, :331, :369, and :376 describe 0.6.0 as shipped. The latest published release is v0.5.0.

- **Location:** `CHANGELOG.md:3-16`; `README.md:15-18,352-369`; `TODO.md:115-139`; `package.json:4`.
- **Evidence:**
  > `CHANGELOG.md:3-6` — `## 0.6.1` and “Patch release on the 0.6 stabilization line.”
  > `README.md:17-18` — “In this 0.6.0 source build” while line 369 says “Localmotive 0.6.0 ships unsigned.”
  > `TODO.md:127` — “Tag `v0.6.1` pushed… Nothing published.”
- **Why it matters:** The current manifests say 0.6.1, the README repeatedly says 0.6.0, the changelog has no `Unreleased` heading, and the tracker says no 0.6 release exists. A user can be sent to GitHub Latest and still infer that a 0.6 artifact is the supported product.
- **Concrete fix:** Add an explicit `## Unreleased` section naming 0.6.1 as a tagged but unpublished candidate. Mark 0.6.0/0.6.1 notes as unreleased until public asset readback succeeds. Make README identify v0.5.0 as the latest published release and call the checkout a 0.6.1 development source. Remove “ships” language until publication.

##### DOC-05 — Three support authorities disagree about 0.6 evidence

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `README.md:327-341`; `docs/EVIDENCE-MATRIX.md:3-8,21-28`; `docs/SUPPORT-MATRIX.md:18-33`.
- **Evidence:**
  > `docs/EVIDENCE-MATRIX.md:25-28` calls 0.6.0 “In development,” “In progress,” and `NOT RUN` for accelerator and Sandbox columns.
  > `docs/SUPPORT-MATRIX.md:22-30` calls the 0.6.0 Windows 11 lifecycle and RTX 5090 CUDA inference `MEASURED`.
  > `README.md:331-335` repeats the measured lifecycle/CUDA claim and says the evidence is version-bound.
- **Why it matters:** The reader cannot distinguish a stale matrix from an unsupported public claim. Release evidence is supposed to be exact-version and digest-bound; a matrix saying NOT RUN beside a matrix saying MEASURED defeats that control.
- **Concrete fix:** Generate both matrices from one versioned evidence manifest, or designate one as the only current authority. Update the row to 0.6.1 if that is the qualified candidate, retain 0.6.0 as historical, and make every README support sentence name the exact published or candidate version.

##### DOC-06 — Immutable tags plus post-tag digest-bound evidence created the release deadlock

> **Lead check: VERIFIED.** Same evidence as REL-01, REL-02, and REL-03.

- **Location:** `AGENTS.md:153-174`; `TODO.md:129-139`; `docs/QUALIFICATION-MAP-0.6.md:51-60`.
- **Evidence:**
  > `AGENTS.md:160-169` requires a fixed tag, producer SHA, evidence outside source, and says digest-bound records cannot transfer between builds.
  > `TODO.md:131` records 12 missing records because they live in evidence commit `d3085ff` after tag target `4b31431`; “The tag cannot move.”
  > `TODO.md:135-139` requires a workflow-dispatch producer that creates all digest-bound records in the same run.
- **Why it matters:** The rules reject the two obvious repairs: moving the immutable tag or staging records from a different build. The tag-triggered workflow checks out the tag tree, so it cannot see post-tag committed records. This is a structural release blocker, not a missing checkbox.
- **Concrete fix:** Use one producer run that checks out the fixed product SHA, builds candidate bytes, produces every digest-bound record against those bytes, assembles the manifest from the retained run workspace, and promotes only that retained bundle. Remove the requirement that current qualification records be committed into the tag tree. Keep SHA-bound evidence and run artifacts as evidence, not source.

##### DOC-07 — The release ledger has expanded into a 111-package, 663-checkbox control system

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `REPORT-0.6.md:12-20`; `docs/history/TODO-0.6.md:13-25`; `TODO.md:30-169`.
- **Evidence:**
  > `REPORT-0.6.md:14-20` — 72 findings, 29 supplemental packages, 10 gates, 111 packages, 663 checkboxes, and 1,051 audit links.
  > `docs/history/TODO-0.6.md:13-15` repeats 72/29/10 and makes all 19 High findings default blockers.
- **Why it matters:** The repository has converted an audit into a permanent release operating system. Historical traceability is valuable; forcing every historical package, gate, carry-forward, mutation proof, owner disposition, and digest note into the current path makes the next ship decision hard to see and easy to invalidate with a documentation commit.
- **Concrete fix:** Freeze the audit and old ledger. Reduce `TODO.md` to a short current release gate list with one status, one owner, one evidence pointer, and one next action per open gate. Generate detailed manifests from scripts or retained CI artifacts instead of copying their contents into the tracker.

#### Medium

##### DOC-08 — Stale and missing path references make the documentation graph non-reproducible

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `CHANGELOG.md:16`; `docs/EVIDENCE-MATRIX.md:7`; `REPORT-0.6.md:6,23`; `docs/qualification-tests.md:3,201`; `.gitattributes:25`; `TODO.md:18-20`.
- **Evidence:**
  > `CHANGELOG.md:16` says current verification is in `TODO-0.6.md`; no tracked root `TODO-0.6.md` exists.
  > `REPORT-0.6.md:6` names `LOCALMOTIVE-FREEZE9-REMEDIATION-HANDOFF.md`; no tracked file exists.
  > `.gitattributes:25` names `src-tauri/tauri.signing.conf.json`; `TODO.md:18-20` says that file was deleted.
- **Why it matters:** Broken links are not cosmetic in this repository: path identity is part of evidence identity. A release reviewer following the documented path can land on a missing file or an old frozen file and reach the wrong release decision.
- **Concrete fix:** Run one repository-local link/path check over all current docs. Replace missing current paths with `TODO.md` or a retained `release-evidence/` path. Mark historical links as historical and remove deleted signing paths from `.gitattributes`. Keep old external commit links only where they are explicitly historical.

##### DOC-09 — Contributor and policy check lists duplicate the same work

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `AGENTS.md:73-93`; `CONTRIBUTING.md:13-32`; `package.json:10,18`.
- **Evidence:**
  > `AGENTS.md:76-87` lists `npm run check`, catalog/workflow gates, then Rust format, Clippy, and tests.
  > `CONTRIBUTING.md:18-25` separately lists TypeScript, `npm test`, build, and the same Rust checks.
  > `package.json:18` says `npm run check` already runs `npm test`, catalog/branding/research/qualification/icon checks, and the build.
- **Why it matters:** Engineers following both files can repeat TypeScript, frontend tests, build, and Rust checks. The policy says not to run the same suite twice, but the onboarding docs make duplication the default. This consumes release time and makes reported verification counts incomparable.
- **Concrete fix:** Make `npm run check` the one frontend/repository command. Keep one Rust command block. Put release-only audit, packaging, and native lifecycle steps in a separate release section with clear prerequisites; delete duplicated commands from `CONTRIBUTING.md`.

##### DOC-10 — README feature availability exceeds the evidence matrix for providers and backends

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `README.md:121-125,185-214`; `docs/PRODUCT.md:15-35`; `docs/SUPPORT-MATRIX.md:26-35,48-56`.
- **Evidence:**
  > `README.md:192-200` lists six provider configurations and `README.md:201-214` describes sign-in, credentials, and measured trials.
  > `docs/SUPPORT-MATRIX.md:54-56` says provider and Hugging Face flows are fixture-only and no live provider is claimed as supported.
  > `docs/SUPPORT-MATRIX.md:31-33` marks other GPUs and managed Vulkan/OpenVINO/SYCL/CPU runtimes `UNTESTED`.
- **Why it matters:** “Configured,” “available in the catalog,” and “supported” are different states. README feature prose is readable as a support promise even though the matrix deliberately limits the claim to fixtures or one measured host.
- **Concrete fix:** Label provider rows “configuration/fixture coverage; live calls untested.” Label backend assets “catalog option; product support untested” unless an exact packaged attestation exists. Keep the feature description, but attach its evidence class to every public support claim.

##### DOC-11 — Frozen qualification and runtime documents are still on the engineer’s current path

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `docs/qualification-tests.md:1-3,18-26`; `docs/RUNTIME_MANAGER.md:1-3,54-66`; `docs/SUPPORT-MATRIX.md:8-10`.
- **Evidence:**
  > `docs/qualification-tests.md:1-3` says it is a frozen 0.4.1 snapshot and “not the current status,” then points at `docs/history/TODO-0.6.md`.
  > `docs/RUNTIME_MANAGER.md:1-3` says its release-specific counts are historical 0.4.1 evidence.
  > `docs/SUPPORT-MATRIX.md:8-10` still points per-finding records at frozen history.
- **Why it matters:** A document can be frozen and still be harmful if current navigation sends engineers to it. This is how old PASS/PENDING/BLOCKED vocabulary and old 0.4.1 scope enter a 0.6.1 release decision.
- **Concrete fix:** Move or clearly prefix frozen qualification documents as historical. Put a short current-status banner at every historical entry point that links only to `TODO.md`, the current manifest, and the current support matrix.

##### DOC-12 — `TODO.md` is not a usable next-action tracker

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `TODO.md:30-41,115-169,227`; `TODO-0.6-post-076a3eeecbdf.md:48-50`.
- **Evidence:**
  > `TODO.md:115` starts U06-06, then lines 127-139 add a long 0.6.1 workflow amendment; U06-07, U06-08, and U06-09 remain later.
  > The longest current line is line 227 at 2,971 UTF-8 bytes and contains three stapled asks.
  > The post-baseline file defines `READY`, `IN-PROGRESS`, `BLOCKED-*`, `VERIFIED`, and `DEFERRED-OWNER`, but current `TODO.md` has zero `READY`, `IN-PROGRESS`, or `VERIFIED` labels.
- **Measured vocabulary:** `TODO.md` contains 82 all-caps state-token occurrences across 7 labels when `PASS` is included: 32 `PASS`, 26 `OPEN`, 11 `CLOSED`, 6 `CLOSED-DEFERRED-FAR-FUTURE`, 4 `BLOCKED-INPUT`, 2 `CLOSED-DEFERRED-USER-REPORTS`, and 1 `DEFERRED-OWNER`. Its ID scan finds 9 distinct `U06-*`, 26 `V06-*`, 19 `GH-*`, 13 `G-*`, 2 `S-*`, 3 `D06-*`, and 2 `R06-*` tokens: 74 normalized IDs across 7 used prefix families; `P06-*` and `E0x` are unused in the current file.
- **Why it matters:** An engineer can search for `U06-06`, but cannot reliably establish whether the next action is a 0.6.0 tag repair, a 0.6.1 dispatch, acceptance, publication, or closeout without reading several kilometer-long lines. There is no short “next action / blocked by / exact command” section.
- **Concrete fix:** Put a five-line `NEXT` block at the top of `TODO.md`: exact version, exact producer workflow, prerequisite, blocking decision, and success condition. Use one current state vocabulary. Keep historical IDs only as links; do not reproduce every evidence paragraph in the current tracker.

##### DOC-13 — Ignored research is cited as public release evidence

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `.gitignore:15-16`; `CHANGELOG.md:266-271,279`; `docs/history/TODO-0.4.1.md:100-117`.
- **Evidence:**
  > `.gitignore:15-16` ignores `/research`.
  > `CHANGELOG.md:266-271` cites `research/0.4/evidence/...` and admits research evidence is gitignored and not carried in the tag.
  > `CHANGELOG.md:279` claims a research smoke command passed while also saying the packaged verifier was not run.
- **Why it matters:** A public reader cannot reproduce or audit the cited evidence from a normal checkout. The release notes mix L2 research observations, packaged claims, and a missing ignored tree. This is especially dangerous because the same section historically called L2 rows “Supported” before correcting that language in 0.4.1.
- **Concrete fix:** Keep `research/` ignored if that is required, but copy only sanitized, immutable summary attestations into `release-evidence/<version>/` and link to those. If the source record is unavailable, say `UNKNOWN` rather than citing a local ignored path as release proof.

##### DOC-14 — Deleted signing paths remain in repository metadata

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `.gitattributes:21-25`; `TODO.md:18-21`; `docs/RUNTIME_MANAGER.md:104-112`.
- **Evidence:**
  > `.gitattributes:25` applies LF pinning to `src-tauri/tauri.signing.conf.json`.
  > `TODO.md:18-21` says the signing config and signing script are deleted and releases are unsigned by standing policy.
  > `docs/RUNTIME_MANAGER.md:108-112` says no application code-signing step exists.
- **Why it matters:** This stale path makes a release reviewer wonder whether signing is disabled, deleted, or merely omitted. It also creates unnecessary line-ending rules in a workflow that already has enough digest-sensitive files.
- **Concrete fix:** Remove the deleted signing path from `.gitattributes`. Keep one current unsigned-policy statement in `AGENTS.md`, `README.md`, `CHANGELOG.md`, and `SUPPLY-CHAIN.md`; do not retain a dead signing configuration path outside historical audit links.

#### Low

##### DOC-15 — The 2,143-line vendored upstream README is a reference, not a local contract

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `docs/LLAMA-SERVER-README.md:1-3,27-32`; `docs/OPTION_MAP.md:1-5`.
- **Evidence:**
  > `docs/LLAMA-SERVER-README.md:1-3` says it is an imported upstream copy and its relative links intentionally do not resolve locally.
  > `docs/LLAMA-SERVER-README.md:29-32` says the option list is generated and must not be edited manually.
- **Why it matters:** The file is useful, but its 2,143-line option table adds review surface and broken-link noise while the policy correctly says runtime `--help` is authoritative. It should not be mistaken for evidence that a backend or flag works in Localmotive.
- **Concrete fix:** Keep provenance and the upstream revision, but link to the upstream document or generate a small pinned option summary. Keep the local option map explicitly historical and runtime-help-derived.

### Answers to specific questions

#### 1. Trackers, ledgers, reports, authority, and dangling references

The strict count is 16 files under the repository root and `docs/` that function as trackers, evidence ledgers/matrices, or audit/release reports:

- **Seven trackers:** `TODO.md`, `TODO-0.6-post-076a3eeecbdf.md`, `docs/history/TODO.md`, `docs/history/TODO-0.4.1.md`, `docs/history/TODO-0.4.md`, `docs/history/TODO-0.5.md`, `docs/history/TODO-0.6.md`.
- **Five matrices/ledgers:** `docs/EVIDENCE-MATRIX.md`, `docs/SUPPORT-MATRIX.md`, `docs/QUALIFICATION-MAP-0.6.md`, `docs/qualification-tests.md`, `docs/SECURITY-REVIEW-LIMITS.md`.
- **Four reports/reviews:** `REPORT-0.6.md`, `docs/RELEASE-REVIEW-0.6.md`, `docs/localmotive-0.6-followup-review-db548c8.md`, `docs/history/localmotive-comprehensive-audit.md`.

Broad counting adds `CHANGELOG.md` (release ledger), `agents_feedback.md` (dated risk log), and `ideas.md` (backlog), giving 19 planning/status surfaces.

Current or current-sounding authority claims are contradictory:

- `AGENTS.md:183-189` and `TODO.md:3-6` correctly designate `TODO.md` as the single current tracker.
- `TODO-0.6-post-076a3eeecbdf.md:9-11` declares itself the active residual tracker. `docs/RELEASE-REVIEW-0.6.md:3` repeats that designation.
- `docs/EVIDENCE-MATRIX.md:5-8` calls `TODO-0.6.md` the authoritative per-item evidence source, but that root file is absent. `docs/SUPPORT-MATRIX.md:8-10` calls frozen `docs/history/TODO-0.6.md` the per-finding record source. `docs/qualification-tests.md:3` also points current evidence to the frozen history file.
- `docs/history/TODO.md:5` calls itself the authoritative v0.3 tracker, and `docs/history/TODO-0.5.md:3` calls its final ship ledger authoritative. Those are valid historical claims only; current docs fail to consistently label them as non-operational.

Verified missing tracked paths:

- `TODO-0.6.md`: absent; referenced by `CHANGELOG.md:16`, `docs/EVIDENCE-MATRIX.md:7`, `REPORT-0.6.md:23`, and many historical/current prose references.
- `TODO-0.6(1).md`: absent; named as an attached byte-identical file in `TODO-0.6-post-076a3eeecbdf.md:13`.
- `LOCALMOTIVE-FREEZE9-REMEDIATION-HANDOFF.md`: absent; referenced by `REPORT-0.6.md:6`.
- `REPORT-0.4.1.md`: absent; referenced by `docs/qualification-tests.md:201`.
- `src-tauri/tauri.signing.conf.json` and `scripts/sign-windows.ps1`: absent by design according to `TODO.md:18-21`, but `.gitattributes:25` still names the former and historical audit tables still name both.
- `research/0.4/`: absent/ignored; historical notes and changelog entries cite it as evidence.

The one-tracker rule is violated by current operational pointers, not by preserving frozen history. The repair is to make the distinction mechanical: one current tracker, one generated current evidence index, and an explicit `docs/history/` boundary.

#### 2. Contradictions between docs

| Topic | Side A | Side B | Assessment |
|---|---|---|---|
| Version | `package.json:4`, `Cargo.toml:4`, `tauri.conf.json:4` all say `0.6.1`. | `README.md:17,369` repeatedly says `0.6.0`; `docs/QUALIFICATION-MAP-0.6.md:1,8-12` is titled/maps 0.6.0. | Current version identity is 0.6.1; current docs are mixed. |
| Release status | `TODO.md:127` says v0.6.1 tag exists and “Nothing published”; `TODO.md:129-139` keeps the final producer open. | `CHANGELOG.md:3-16` calls 0.6.1 a patch release and 0.6.0 “This release”; `README.md:369` says 0.6.0 “ships.” | Public status is misleading; 0.6.x should be marked Unreleased until readback. |
| 0.6 evidence | `docs/EVIDENCE-MATRIX.md:28` says 0.6.0 is “In development,” in progress, and NOT RUN. | `docs/SUPPORT-MATRIX.md:22,30` and `README.md:331-335` call 0.6.0 lifecycle/CUDA evidence MEASURED/PASS. | The matrices cannot both be current. |
| OS support | `docs/SUPPORT-MATRIX.md:22-24` measures Windows 11 and marks Windows 10 UNTESTED. `README.md:336-338` says Windows 10 remains untested. | `CHANGELOG.md:156-161`, `CHANGELOG.md:183-187`, and `docs/PRODUCT.md:29-35` state the target scope is Windows 10 and Windows 11 x64. | “Target” is not support, but the public text does not consistently keep that distinction in headings and release sections. |
| Hardware/backends | `docs/SUPPORT-MATRIX.md:30-33` measures one RTX 5090 CUDA path and marks other NVIDIA, AMD, Intel, Vulkan, OpenVINO, SYCL, and managed CPU paths UNTESTED. | `docs/PRODUCT.md:15-35` lists CPU/NVIDIA/AMD/Intel users and reviewed CPU/CUDA/ROCm/SYCL/Vulkan/OpenVINO archives; `README.md:121-125` lists all approved assets. | Catalog/feature availability is being read as support unless the evidence class is carried into the claim. |
| Historical 0.4 support | `CHANGELOG.md:266-269` calls three Windows 11/RTX/CPU-CUDA-Vulkan rows “Supported.” | `CHANGELOG.md:204-212` later says those were L2 direct-runtime evidence and did not establish product support. | The correction exists, but the old public wording remains in the same changelog and needs an explicit corrective banner. |
| Signing | `AGENTS.md:172-174`, `README.md:352-354`, and `docs/SUPPLY-CHAIN.md:27-34` consistently say application installers are unsigned and no signing step is pursued. | `.gitattributes:25` still points at deleted signing configuration; `TODO.md:18-21` says it is deleted. | Policy is not substantively contradictory; the stale path creates avoidable uncertainty. |

#### 3. README accuracy against the evidence and support matrices

| README claim group | Evidence comparison | Verdict |
|---|---|---|
| Product role, Windows desktop control plane, llama-server ownership (`README.md:3-11`) | Consistent with `docs/PRODUCT.md:5-25` and `docs/RUNTIME_MANAGER.md:5`. | Accurate as product description; not a packaged compatibility claim. |
| Runtime management and exact help gating (`README.md:103-125`) | Consistent with `AGENTS.md:18-28`, `docs/RUNTIME_MANAGER.md:19-42`, and `docs/OPTION_MAP.md:1-5`. The approved asset list is not support proof; `SUPPORT-MATRIX.md:31-33` marks most backend paths untested. | Feature/policy description is plausible; asset list must be labeled availability, not support. |
| GGUF inventory, shards, companions, profiles (`README.md:127-138`) | Consistent with `docs/ARTIFACT-IDENTITY.md:7-33` and `docs/PRODUCT.md:43-57`. No release matrix row proves every UI path. | Implementation description, not evidence of every packaged flow. |
| Server control and benchmark (`README.md:140-150`) | CPU packaged lifecycle is present in `SUPPORT-MATRIX.md:22` and `EVIDENCE-MATRIX.md:25-27` for older published versions; the exact 0.6 version is disputed by `EVIDENCE-MATRIX.md:28`. | Keep, but bind to an exact candidate/release manifest. |
| Catalog size and signed cache/download behavior (`README.md:152-183`) | The 158-model/1,417-file count matches `CHANGELOG.md:168-174`; signature/cache behavior aligns with `SUPPORT-MATRIX.md:52-53` and `README.md:168-170`. | Accurate for catalog state, subject to date/version identity. |
| Hugging Face/cloud credentials (`README.md:176-214`) | Storage policy matches `AGENTS.md:29-35` and `SUPPORT-MATRIX.md:55-56`; live authenticated flows are not claimed by the matrix. | Mark token/provider behavior fixture-only or untested where applicable. |
| AI Tune providers (`README.md:185-214`) | UI/configuration is described, but `SUPPORT-MATRIX.md:54` says six-provider coverage is contract fixtures only and no live provider is supported. | Overbroad unless “configured providers” is explicitly separated from “supported providers.” |
| Network/data disclosure (`README.md:220-249,264-305`) | The README honestly says local runtime/model/companion paths and trial evidence are sent to the selected cloud provider (`README.md:232-249`); this is consistent with the security posture. | Accurate disclosure; no live-provider support should be implied. |
| Platform support (`README.md:327-350`) | Matches `SUPPORT-MATRIX.md:22-35` for the one Windows 11/RTX 5090 measured scope and untested Windows 10/other hardware. It conflicts with `EVIDENCE-MATRIX.md:28` and the 0.6.1 manifest identity. | Content is conditionally accurate, but version-bound evidence must be repaired first. |
| Unsigned artifacts and SmartScreen (`README.md:352-376`) | Consistent with `AGENTS.md:172-174`, `docs/SUPPLY-CHAIN.md:25-34`, and `SUPPORT-MATRIX.md:72`. | Policy disclosure accurate; “0.6.0 ships” is not accurate for an unpublished line. |
| Release/check workflow (`README.md:451-460`) | `AGENTS.md:157-171` and `TODO.md:131-139` show the workflow is not yet at a successful final producer/publication state. | Describe as required workflow, not completed release verification. |

The README is **not honest enough that 0.6.x is unreleased**. It warns users not to use the unreleased source tip (`README.md:15-18`), but then says 0.6.0 “ships,” has no explicit 0.6.1 candidate/unreleased banner, and does not match the package version.

#### 4. CHANGELOG versus published releases

- The v0.4.0, v0.4.1, and v0.5.0 sections exist. `docs/EVIDENCE-MATRIX.md:25-27` marks them as public releases, and `docs/history/TODO-0.5.md:3-5` explicitly says v0.5.0 shipped on 2026-09-10. These sections are broadly consistent with the stated published-release history.
- The changelog also preserves the 0.4.0 support-language correction: the original “Supported” rows are visible at `CHANGELOG.md:266-269`, and the correction is at `CHANGELOG.md:204-212`. That is honest historical context but still easy to misread.
- The 0.6.0 and 0.6.1 sections are not marked `[Unreleased]`. The 0.6.1 text says “Patch release” (`CHANGELOG.md:3-10`), while the tracker says the tag is pending publication and “Nothing published” (`TODO.md:127`).
- The 0.6.0 section points at missing root `TODO-0.6.md` (`CHANGELOG.md:14-16`) instead of the current `TODO.md`.
- The changelog’s `Unreleased` section is therefore **incorrect: it does not exist, and the 0.6.x entries are positioned as releases**. Fix the release boundary before publication; do not rewrite the published 0.4.x/0.5.0 history.

#### 5. TODO.md usability

Measured from the tracked file:

- 294 lines, 48,192 bytes.
- Longest line: line 227, 2,971 UTF-8 bytes. The next long lines are line 131 at 1,430 bytes, line 94 at 1,404 bytes, and line 135 at 1,381 bytes.
- 82 state-token occurrences across 7 distinct labels when `PASS` is included. The operational Open/Closed/Deferred/Blocked family contributes 50 occurrences; `PASS` contributes 32. `FAIL`, `READY`, `IN-PROGRESS`, `VERIFIED`, `NOT-EXERCISED`, `UNTESTED`, and `UNKNOWN` do not occur as exact labels in the current file.
- ID families in the current file: `U06-*` 9 distinct, `V06-*` 26, `GH-*` 19, `G-*` 13, `S-*` 2, `D06-*` 3, `R06-*` 2. That is 74 normalized IDs across 7 used families. `P06-*` and `E0x` are absent despite the post-baseline tracker discussing `P06-*`/`R06-*` local labels.
- The immediate work is technically present: U06-06 starts at `TODO.md:115`, the 0.6.1 final producer amendment is at `TODO.md:135-139`, U06-07 is at `TODO.md:143`, and publication is U06-08 at `TODO.md:155-161`. It is not reliably discoverable by a new engineer in under five minutes because the first open item is surrounded by obsolete 0.6.0 tag instructions, multiple later amendments, mixed ID families, and very long evidence lines. There is no top-level exact `NEXT` block.

#### 6. AGENTS.md rules that contradict, cannot be enforced, or create the deadlock

| Rule | Evidence | Failure mode | Repair |
|---|---|---|---|
| One current tracker only | `AGENTS.md:183-189` | Current docs still name the deleted root tracker, active post-baseline tracker, and frozen history as authority. | Make `TODO.md` the only current path; mechanically check current docs for tracker references. |
| Immutable tag; no late version bump | `AGENTS.md:153-166` | Correctly protects provenance, but the tag-triggered workflow checks a tree that lacks post-tag evidence. | Produce digest-bound records in the same retained run as the candidate, not by committing them after the tag. |
| Evidence never source and digest-bound records never transfer | `AGENTS.md:160-171` | Combined with the current tag-tree manifest assembly, it rejects staging and leaves 12 records missing (`TODO.md:131`). | Use CI artifacts/run workspace as the evidence input; retain sourceRevision and candidate digests. |
| Full test/mutation discipline for every behavioral change | `AGENTS.md:51-71` | “Write failing first” and controlled mutation proof are not uniformly machine-enforced and are expensive for low-risk changes. | Keep mutation proof for safety/release contracts; make the scope explicit instead of requiring an undocumented human ritual for every change. |
| Full preflight plus packaged matrix | `AGENTS.md:73-118` | The policy is a long checklist while `CONTRIBUTING.md` repeats parts of it; successful build is correctly not release approval, but the next gate is not concise. | Split PR checks, candidate qualification, and publication readback into three short contracts. |
| Main plus one lane | `AGENTS.md:141-147` | This is an owner/process policy, not a local verifier; branch drift can still occur while docs say the rule is satisfied. | Verify branch count and open-PR/worktree state in one maintainer check, not in prose alone. |
| Research remains ignored | `AGENTS.md:191-194` and `.gitignore:15-16` | Historical changelog and TODO records cite ignored research as release evidence. | Keep research ignored if required, but publish sanitized evidence summaries or state UNKNOWN. |
| Unsigned policy is terminal | `AGENTS.md:172-174` | Current policy is clear, but `.gitattributes` retains a deleted signing config path. | Remove stale signing metadata and keep one unsigned policy source. |

The major process deadlock is measured, not inferred: after the 0.6.1 tag, the evidence commit was later than the tag; the tag cannot move; cross-build staging failed digest coherence; and the final producer therefore had to be amended into a workflow-dispatch design (`TODO.md:131-139`).

#### 7. `artifacts/` fraction, unsafe contents, ignore rules, and cleanup

Tracked tree measurements:

| Area | Files | Bytes | Share of tracked repository |
|---|---:|---:|---:|
| Entire tree | 6,005 | 329,944,028 | 100% |
| `artifacts/` | 5,494 | 322,015,644 | 91.49% files / 97.60% bytes |
| `release-evidence/` | 247 | 1,593,304 | 4.11% files / 0.48% bytes |
| Both | 5,741 | 323,608,948 | 95.60% files / 98.08% bytes |

Top tracked `artifacts/` directories by bytes are `ideas-audit-20260919-2202` (79,997,768 B), `tune-input-audit-20260920-0141` (49,210,889 B), `lora-audit-20260919-2345` (47,918,319 B), `benchmark-input-audit-20260920-0214` (29,402,744 B), `legacy-ownership-audit-20260920-0435` (28,790,154 B), `legacy-benchmark-input-audit-20260920-0310` (28,575,664 B), `manifest-audit-20260920-0049` (19,652,486 B), `benchmark-statistics-audit-20260920-0349` (18,948,875 B), and `stop-ownership-audit-20260920-0624` (18,937,095 B). Git history attributes these trees to commit `6099c6f`, “fix(u06): close legacy benchmark ownership review gaps.”

Should never have been committed in raw form:

- WebView2 profile stores and databases, including the 364 name-matched browser/token paths listed above. Contents are **UNKNOWN** because they were not opened.
- Raw WebView cache/Local Storage/Session Storage trees, SQLite files, WAL/journal files, model fixtures, baseline zips, diagnostic diffs, and screenshots/log captures when a sanitized assertion would suffice.
- Repeated audit-session directories with no current release-consumer path.
- Local candidate installers if they are only working-tree outputs. The current three untracked 0.6.0 installers are not tracked, but they are also not ignored.

Suggested ignore policy for future raw output, subject to preserving already-tracked accepted evidence:

```gitignore
## Raw local qualification captures; keep the curated release-evidence tree.
/artifacts/*
/.hermes-0.6/

## Allow only small candidate identity records when they are intentionally staged.
!/artifacts/SHA256SUMS-*.txt
!/artifacts/candidate-inventory-*.json
!/artifacts/packaged-verification-*.json
!/artifacts/catalog-*.json

## Never allow browser/runtime profile captures into evidence by accident.
artifacts/**/a11y-*/
artifacts/**/native-*/
artifacts/**/webview/
artifacts/**/appdata/
```

Git ignore alone does not remove already-tracked files. The cleanup recommendation is: first inventory accepted evidence and retain only the release-evidence allowlist; then remove raw `artifacts/` paths from the index in a reviewed commit; only after owner approval consider history rewriting. Do not delete or rewrite `release-evidence/` blindly: 0.4.1/0.6.0/0.6.1 records are part of the stated release audit trail.

#### 8. Commit-history pattern since v0.5.0

Method:

1. Enumerate the range with the requested `git log --format='%ad %s' --date=short a4b7127f739f7420232d9b6f63da693d39128d0b..HEAD`; it returns 338 commit subjects from 2026-09-10 through 2026-09-23.
2. Independently enumerate each commit’s paths with `git diff-tree --root --no-commit-id --name-only -r <sha>`.
3. Classify each commit exclusively: all documentation paths = docs-only; all `artifacts/`/`release-evidence/` paths = evidence-only; a union of those = docs+evidence-only; any `src/`, `src-tauri/`, `catalog/`, `assets/`, or source-code path = product-code-touching; remaining automation/workflow/config commits without product code = automation/tooling-containing; no-path merges = merge/no-paths; the remaining `.gitattributes` change = other.

Results:

| Class | Commits | Share of 338 |
|---|---:|---:|
| Docs-only | 86 | 25.4% |
| Evidence-only | 6 | 1.8% |
| Docs + evidence only | 13 | 3.8% |
| **Docs/evidence-only total** | **105** | **31.1%** |
| Product-code-touching | 123 | 36.4% |
| Automation/tooling-containing, no product code | 76 | 22.5% |
| Merge/no paths | 33 | 9.8% |
| Other (`.gitattributes`) | 1 | 0.3% |
| **Total** | **338** | **100%** |

The comparison is not “documentation is bad.” It shows that the release work has a large evidence/process stream: 105 commits alter only status/evidence surfaces, while the final release is still blocked by U06-06/U06-07/U06-08. Several product-code commits also carry documentation/evidence changes; the exclusive categories above intentionally do not call those docs-only.

### Cross-cutting observations

1. **The repository confuses preservation with authority.** Keeping a frozen ledger is correct. Linking every current decision to frozen ledgers, while also keeping an active post-baseline tracker, turns history into a competing control plane.
2. **The current tracker is a data dump rather than an execution index.** The evidence details are valuable, but the next command, blocking owner decision, and exact success condition should be at the top. Long embedded records make a documentation commit capable of changing what a reviewer believes is current.
3. **Evidence identity is stronger than the process that assembles it.** The four-identity rule is correct in principle, but the workflow still assembles from a tag checkout that cannot contain later evidence. The fix is to simplify the producer/consumer boundary, not to create a new version or move a tag.
4. **The public product story is mostly cautious where the matrix is current.** The README correctly discloses Windows 10, other hardware, screen readers, live cloud providers, and OS-crash limits. The failure is version status and the stale/contradictory evidence matrix, not a blanket unsupported-hardware claim.
5. **Unsigned application distribution is not the current contradiction.** `AGENTS.md`, `README.md`, `SECURITY.md`, `SUPPLY-CHAIN.md`, and `SUPPORT-MATRIX.md` consistently disclose unsigned files and SmartScreen. The dead signing path in `.gitattributes` is cleanup debt, not evidence of a signing step.
6. **The raw artifact tree is a privacy and operations problem at once.** It is large enough to dominate the repository and contains names of browser credential/history stores. The contents were not inspected; any assertion that credentials are actually recoverable is UNKNOWN until an authorized sensitive review.
7. **The report set should be frozen, not extended.** The next release should not create another `REPORT-*`, `REVIEW-*`, `TODO-*`, or versioned map. Keep historical reports immutable and put only current open obligations in `TODO.md`.
8. **Unverified items remain unverified.** This review did not inspect browser-store contents, run release workflows, query GitHub, run tests, build the app, or verify public release assets. Publication status beyond the local records and task context is not independently re-fetched here.

---

## Part 4 — Rust runtime management (finding prefix RT)

### Summary

- `runtime.rs` is 9,147 lines and mixes Windows hardware probing, catalog policy, HTTP/cache transport, archive extraction, content authority, install publication, health-model repair, execution leases, runtime identity, and roughly 4,340 lines of tests. It is not a module; it is a trust boundary and a test suite welded together.
- The GitHub catalog client is unauthenticated, has one 15-second request deadline, no retry loop, and only uses disk cache after attempting GitHub. A clean packaged verifier therefore still depends on `api.github.com`.
- A transient catalog error or rate limit with no validated cache becomes `catalog: None`; the release verifier labels that state `terminal-error`, even though the Rust error is typed and retryable.
- Concurrent catalog calls do not join. `ExclusiveOperation` is a fail-fast `AtomicBool`; the second call gets `Busy`. Setup maps that to a successful IPC response with no catalog, while the explicit fetch command returns an error.
- Archive extraction is substantially defended: bounded entries, traversal rejection, alternate-stream rejection, duplicate canonical paths, symlink rejection, capability-directory writes, same-handle archive hashing, exact inventory, and per-file SHA-256 checks are present.
- Install publication still has a real filesystem race: `runtime.json` is written with path-based `fs::write` without the hard-link/reparse protection already used by `download.rs`. The staging verification is also not bound atomically to publication, and the old destination can be discarded before the final post-publish verification succeeds.
- The protected execution lease is used for server launch and managed health, but public `--version`, `--help`, and legacy `--list-devices` probes use a check-only guard and spawn after it returns. The source-pattern test claims the boundary is safe without testing this missing lease.
- Flag filtering is based on actual help output and rejects required generated flags that are absent, which is the correct ownership boundary. The parser is still a token heuristic: feature booleans use substring matching and supported flags are not normalized for `--flag=value` forms.
- GPU preflight is forced to `Unknown` in the production caller because `runtime_topology_known` is true only for CPU. KV estimation is verified only for the `llama` architecture. This is conservative, but it makes accelerator preflight non-decisive rather than useful.
- The approval data is deliberately pinned, but the code and tests retain stale `b10796` prose plus exact `b10816` fixture assertions. Every upstream bump requires editing the approval JSON, seven content manifests, two release fixtures, and several tests instead of regenerating one authoritative bundle.

### Per-file verdicts

| File | Lines | Verdict | Largest functions / notes |
|---|---:|---|---|
| `src-tauri/src/runtime.rs` | 9,147 | **SPLIT** | `parse_approved_manifest` 1718-1935 (218); `detect_dxgi_adapters` 414-577 (164); `build_approved_catalog` 2022-2180 (159). Also contains all runtime tests from 4808-9147. |
| `src-tauri/src/runtime_service.rs` | 275 | **SHRINK** | `load_runtime_setup` 13-66 (54); `check_managed_runtime_health` 161-210 (50); `fetch_runtime_catalog` 80-116 (37). Delete the orphan cloud-credentials section and replace production lock `unwrap()` calls. |
| `src-tauri/src/preflight.rs` | 1,261 | **SHRINK** | `build_preflight_report` 450-615 (166); `estimate_kv_cache_bytes` 35-171 (137); `build_device_plan` 328-425 (98). Move the 802-1261 test block out of the production file. |
| `src-tauri/approved_runtimes.json` | 350 | **KEEP** | Authoritative compiled approval pin; no functions. It currently pins release `b10816` at lines 3-10. |
| `src-tauri/runtime-content-manifests/cpu.json` | 271 | **KEEP** | Authoritative per-file content pin; no functions. |
| `src-tauri/runtime-content-manifests/cuda-12.4.json` | 296 | **KEEP** | Authoritative per-file content pin; no functions. |
| `src-tauri/runtime-content-manifests/cuda-13.3.json` | 296 | **KEEP** | Authoritative per-file content pin; no functions. |
| `src-tauri/runtime-content-manifests/openvino.json` | 411 | **KEEP** | Authoritative per-file content pin; no functions. |
| `src-tauri/runtime-content-manifests/rocm.json` | 291 | **KEEP** | Authoritative per-file content pin; no functions. |
| `src-tauri/runtime-content-manifests/sycl.json` | 376 | **KEEP** | Authoritative per-file content pin; no functions. |
| `src-tauri/runtime-content-manifests/vulkan.json` | 276 | **KEEP** | Authoritative per-file content pin; no functions. |
| `src-tauri/tests/fixtures/runtime/b10816-release.json` | 174 | **SHRINK** | Full version-pinned GitHub response fixture; no functions. Generate from the approved bundle or keep it outside the production source tree. |
| `src-tauri/tests/fixtures/runtime/b10816-required-jobs.json` | 255 | **SHRINK** | Full version-pinned required-job fixture; no functions. Same churn problem as the release fixture. |
| `src-tauri/tests/fixtures/runtime/blocked-jobs.json` | 24 | **KEEP** | Small behavior fixture; no functions. |

### Findings

#### RT-01 — Medium (lead; reviewer said Critical) — The install record is written through an attacker-replaceable path

> **Lead check: CORRECTED.** Same-user race in the user's own %LOCALAPPDATA%. It crosses no privilege boundary.

- **Location:** `src-tauri/src/runtime.rs:3677-3703`, called by `src-tauri/src/runtime.rs:4561-4574`.
- **Evidence:** `write_runtime_install_record` ends with `fs::write(root.join("runtime.json"), bytes)`; it does not use `CapDir::open_with`, `create_new`, a hard-link count check, or a reparse-point-safe handle.
- **Why it matters:** A same-user process can race the staging directory and replace `runtime.json` with a symlink, reparse point, or hard link. `fs::write` follows the link or truncates the linked file. A hard link can evade the later reparse check and overwrite another file before the installer reports success or failure. `download.rs` already has the missing write-entry checks at `672-704`; runtime installation simply does not reuse them.
- **Fix:** Reject archive entries named `runtime.json`, then create the record with a capability-directory `create_new` open and retain the directory capability through verification/publication. Reject links and `nNumberOfLinks != 1` before writing. Add a Windows hard-link and reparse-race regression test that asserts the victim remains unchanged.

#### RT-02 — High (lead; reviewer said Critical) — Staging verification is not bound to publication and rollback is destroyed too early

> **Lead check: CORRECTED.** The early deletion of the rollback copy is real and breaks the atomic-publication rule. The race part needs a same-user writer.

- **Location:** `src-tauri/src/runtime.rs:4569-4588` and `4406-4459`.
- **Evidence:** The installer verifies staging at `4575-4580`, explicitly drops `staging_guard` at `4581`, renames at `4582`, and verifies the final directory only afterward at `4583-4588`. The replacement helper removes the old backup at `4456-4459` before that final verification.
- **Why it matters:** A writer can change staging after the first verification and before rename. If final verification then fails, the new destination is already published and the old destination has already been deleted; the cleanup at `4597-4599` only removes the old staging path, which no longer exists. Even a non-malicious post-publish failure can leave a failed install in place with no rollback copy.
- **Fix:** Keep an execution-identity/capability proof through publication, reverify using retained handles before deleting the backup, and restore the old destination on every post-publish failure. If backup deletion fails after a successful publish, return success with deferred cleanup rather than reporting a failed install with an inconsistent tree. Add a mutation-in-the-publication-window test.

#### RT-03 — Medium (lead; reviewer said Critical) — Managed probes verify, release the check, and then execute without an identity lease

> **Lead check: CORRECTED.** Same-user race between the verification and the execution.

- **Location:** `src-tauri/src/core.rs:1735-1748`, `1780-1787`, `2202-2211`; `src-tauri/src/runtime.rs:4217-4228`; public callers `src-tauri/src/lib.rs:1468-1477` and `1489-1502`.
- **Evidence:** `run_runtime_probe_with` calls `guard(path)?` and then constructs `hidden_command(path)`. `authorize_managed_execution_with` returns `Result<(), String>` after verification; the separate `authorize_managed_execution_lease` API is not used by the probe path.
- **Why it matters:** A managed `llama-server.exe` or `llama-cli.exe` can be replaced after the last hash check and before process creation. The server launch path correctly holds `ManagedExecutionLease` at `lib.rs:761-768`, and managed health retains one at `runtime.rs:4783-4805`; the public inspection and legacy health commands do not. The documented claim at `core.rs:1730-1734` that the guard prevents bypass is therefore only a check-before-use claim, not an execution-identity guarantee.
- **Fix:** Make the probe guard return `Option<ManagedExecutionLease>` and hold it across `output_with_timeout_and_cancel`, or expose one probe helper that acquires the lease and owns the child lifetime. Test a synchronized replacement between verification and spawn. Do not treat source-order assertions as proof.

#### RT-05 — Critical — Catalog refresh is a network hard dependency for the app and release gate

> **Lead check: VERIFIED.** runtime.rs:2808-2887 always calls api.github.com. approved_runtimes.json pins name, URL, bytes, and SHA-256 for each b10816 asset.

- **Location:** `src-tauri/src/runtime.rs:15`, `2500-2527`, `2808-2886`; `src-tauri/src/runtime_service.rs:38-65`, `80-114`; `scripts/verify_041.mjs:447-465`, `471-518`.
- **Evidence:** `fetch_catalog` reads the disk cache at `2815-2821`, then unconditionally builds a client and calls `fetch_catalog_http` at `2822-2830`. The cache is used only inside the error branch at `2874-2884`. The verifier maps `baselineSetup.catalog` being absent directly to `catalog_result: "terminal-error"` at `verify_041.mjs:460-464`.
- **Why it matters:** There is no retry loop, no in-memory result reuse, no cache-first/offline mode, and no bundled runtime-catalog fallback. A clean verifier profile has no cache. A timeout, DNS/connection error, HTTP error, or rate limit therefore leaves setup with `catalog: None`; the setup IPC technically succeeds but the gate records a terminal error and later explicit catalog checks fail. The setup command and explicit fetch command are separate GitHub requests, worsening unauthenticated quota pressure.
- **Fix:** Ship a signed/approved catalog snapshot or repository-hosted mirror as the release-gate authority. Make the cache-first policy explicit with bounded freshness and stale-cache behavior; add a user-visible offline mode. Coalesce setup and explicit fetch into one in-memory future/result. Retry only bounded transient failures with backoff, and make the gate classify `Busy`, `Timeout`, and `RateLimited` as retryable rather than terminal.

#### RT-04 — High — The execution lease pins expected files but not the directory inventory

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/runtime.rs:4139-4160`, `4090-4126`, and managed health `4783-4791`.
- **Evidence:** `authorize_managed_execution_lease_with` calls `collect_install_files` and compares the returned list at `4150-4157`, then calls `lease_verified_installation` at `4159`; the lease opens and retains only manifest-listed files.
- **Why it matters:** An extra file can be created after the inventory scan and before or during lease acquisition. The expected EXE/DLL bytes are pinned, but the code never proves that the directory still contains exactly the approved inventory when the child starts. A loader that resolves an unexpected DLL by name can turn that gap into code execution. The existing test only tries to overwrite already-listed files while leased (`runtime.rs:7970-7997`); it does not add an extra file in the inventory window.
- **Fix:** Open and retain a directory handle with deletion/create sharing restrictions, acquire the file handles through that capability, and re-enumerate the directory after all handles are acquired. Reject any extra or changed entry. Add an extra-DLL race test for both server launch and managed health.

#### RT-06 — High — Concurrent catalog requests fail fast instead of joining or reusing work

> **Lead check: VERIFIED.** runtime_service.rs:38-53 and :84-90 return kind Busy to a second concurrent caller.

- **Location:** `src-tauri/src/lib.rs:103-120`; `src-tauri/src/runtime_service.rs:38-53` and `84-90`.
- **Evidence:** `ExclusiveOperation::acquire` uses `compare_exchange(false, true, ...)` and maps failure to `A {operation} request is already active`. `load_runtime_setup` converts that to `RuntimeCatalogErrorKind::Busy`; `fetch_runtime_catalog` returns the Busy error.
- **Why it matters:** The second caller does not receive the in-flight catalog, validated cache, or a shared result. A setup call can therefore return hardware and managed runtimes with `catalog: None` even while an identical request is about to succeed. The release gate and frontend then see a missing catalog, not a joined request. This is a correctness failure, not coalescing; the separate verification coalescer at `runtime.rs:3897-3959` proves the codebase knows how to join work but does not apply that design to catalog fetches.
- **Fix:** Store one in-flight catalog future/result keyed by the approved identity and adapter selection. Return the same result to concurrent callers, while retaining a separate short-lived Busy response only for operations that cannot safely join.

#### RT-07 — High — Managed-runtime discovery crosses reparse ancestors and has no scan bound

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/runtime.rs:992-996`, `1102-1164`; caller `src-tauri/src/runtime_service.rs:17-27`.
- **Evidence:** `list_managed_runtimes_in_with` calls `fs::read_dir(root)` at `1114-1117` without `validate_no_reparse_ancestors`; `safe_directory` checks only the candidate directory metadata. The two nested `read_dir` loops have no entry, record-count, cancellation, or output-size cap.
- **Why it matters:** A junction in a root ancestor can redirect discovery outside the intended runtime tree, and a user- or attacker-populated root with a large number of tag/install directories can make startup enumerate and serialize an unbounded vector. The later content verification does not prevent the initial traversal or IPC denial of service.
- **Fix:** Validate the complete root path before the first `read_dir`, open it through the existing capability-directory helper, bound tags/install records and path lengths, and return a bounded structured diagnostic when the cap is reached. Add an ancestor-junction and large-directory test.

#### RT-10 — High — Path validation is inconsistent and sometimes happens after the first write

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/runtime.rs:2679-2696`, `4489-4498`, `4630-4636`, `4680-4689`.
- **Evidence:** `read_catalog_cache` and `read_install_record` inspect `symlink_metadata` and then reopen by path with `fs::read`. `install_runtime` calls `fs::create_dir_all(&root)` at `4490` before validating ancestors at `4491`; health-model repair does the same for its parent at `4683-4685` before validation.
- **Why it matters:** A path can change between the metadata check and the reopen, and `create_dir_all` can follow an attacker-installed junction before the reparse check rejects it. This violates the project rule that lexical normalization is not containment proof and duplicates the exact check/use gap the capability-directory code was meant to remove.
- **Fix:** Validate ancestors before any create/open operation, then operate through one opened directory capability. Read records/cache bodies through the already-validated handle. Reuse `download.rs:672-704` for file-write identity checks instead of maintaining another weaker path-based variant.

#### RT-08 — Medium — Production GPU preflight is structurally forced to Unknown

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/lib.rs:1735-1764`; `src-tauri/src/preflight.rs:31-33` and `544-553`.
- **Evidence:** The production caller sets `runtime_topology_known: matches!(execution_path, ExecutionPath::Cpu)` at `1737`. `build_preflight_report` then unconditionally sets `memory.class = FitClass::Unknown` for every non-CPU path when that flag is false. KV layout support is also `matches!(..., "llama")` only.
- **Why it matters:** Single-GPU, multi-GPU, CUDA, ROCm, SYCL, Vulkan, and OpenVINO paths never receive verified topology from this caller, so the report cannot become a useful memory-fit class even when adapter capacities and a device plan are present. Non-Llama GGUF architectures are always unknown for dense KV estimation. This avoids false confidence but leaves the main accelerator use case without a decision.
- **Fix:** Feed a verified runtime-device-to-adapter mapping from the bounded runtime probe into `PreflightFacts`, or state explicitly that GPU preflight is unavailable and remove the misleading derived device plan. Replace the one-architecture KV gate with a tested architecture/layout table; preserve `Unknown` for layouts without evidence.

#### RT-09 — Medium — Help capability parsing is exact only after a lossy, substring-based discovery step

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/core.rs:1475-1495`, `1577-1601`, `1672-1718`; generated flags `982-1005` and `1093-1097`.
- **Evidence:** `parse_supported_flags` splits on whitespace and retains cleaned tokens beginning with `-`; `parse_capabilities` sets `metrics`, `multimodal`, and `fit` with `help.contains("--...")`. `argument_flag` at `1359-1365` strips `=value`, but `parse_supported_flags` does not perform the same normalization.
- **Why it matters:** The launch boundary does use the selected runtime's real help output and exact flag set, and required generated flags are rejected at `1645-1655`; there is no backend-name inference in that path. However, a help line containing `--fit-target` can make `fit` true, and a help token shaped `--flag=value` is stored differently from generated `--flag`. Whether current upstream help uses the latter form is **UNKNOWN** without the prohibited network fetch, but the parser is not robust against it.
- **Fix:** Parse option declarations into canonical flag names, normalize `--flag=value`, reject prose/prefix collisions, and test exact-vs-prefix cases (`--fit` vs `--fit-target`) against captured help fixtures. Keep required/optional filtering as the final authority.

#### RT-11 — Medium — Runtime service commands panic on poisoned state locks

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/runtime_service.rs:130-146`, `150-157`, `170-208`, `263-271`.
- **Evidence:** Install, health, and cancellation commands use `state.runtime_install.lock().unwrap()`, `state.runtime_health.lock().unwrap()`, or equivalent production lock unwraps; only the health-repair path maps lock poisoning to a `Result`.
- **Why it matters:** A panic while a command owns one of these locks poisons the mutex. The next runtime install/health/cancel IPC call panics again instead of returning a bounded error, leaving the control plane degraded and potentially leaving an active cancellation slot uncleared.
- **Fix:** Map every lock failure to `Result<T, String>` and clear operation state with a guard that survives task errors. Add a poisoned-lock behavior test where the command boundary is directly testable.

#### RT-12 — Medium — Security tests assert source spelling instead of exercising the trust boundary

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/runtime.rs:5130-5166`; `src-tauri/src/lib.rs:2352-2387` and `3017-3041`.
- **Evidence:** `rt08_last_error_is_captured_before_cleanup_calls` reads `include_str!("runtime.rs")` and compares substring positions. The inspect and launch tests read `ALL_SOURCES` and assert that function names occur in a desired order.
- **Why it matters:** These tests pass when the named text remains while behavior is wrong. That is exactly what happened to the probe lease boundary: the source test proves `authorize_managed_execution` is called before probing, but cannot prove that the verified bytes remain pinned through process creation. They are implementation-spelling tests, not observable-result tests, and they make refactoring risky.
- **Fix:** Replace source scans with real fixture executables, synchronized replacement attempts, bounded process calls, and observable rejection/lease results. Keep only narrowly justified API seam tests for Windows last-error behavior.

#### RT-14 — Medium — Version-pinned data and tests are coupled to every upstream bump

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/approved_runtimes.json:3-10`; `src-tauri/runtime-content-manifests/cpu.json:3-11` and corresponding tops of all seven manifests; `src-tauri/tests/fixtures/runtime/b10816-release.json:2-8`; `src-tauri/src/runtime.rs:1552-1582`, `5354-5403`, `6542-6551`.
- **Evidence:** The approval bundle and every content manifest carry release `b10816`; the fixtures repeat that release identity; runtime code comments and binding tests still name `b10796`, while a path assertion hard-codes `b10816`.
- **Why it matters:** The production parser is mostly generic, but the release process must edit a large set of files and test literals for each upstream release. Stale `b10796` prose makes current approval history ambiguous, and exact path assertions fail even when the new approval data is valid. This is maintenance churn in the path the owner already cannot publish reliably.
- **Fix:** Generate approval/content manifests and offline fixtures from one versioned approval input, derive expected tags in tests from `approved_runtime_identity()`, and move historical release snapshots out of production modules. Keep the release pin as data; do not spread it through code comments and tests.

#### RT-13 — Low — Dead scaffolding and an orphan service section remain in the runtime surface

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/runtime.rs:148-158`, `3990-3997`; `src-tauri/src/runtime_service.rs:273-275`.
- **Evidence:** `GithubAsset::sample` is `#[cfg(test)]` and `#[allow(dead_code)]` with no call site. `VerifiedInstallation.install_dir` and `.install` are both explicitly allowed dead fields; the only observed consumer is `server_path`. `runtime_service.rs` ends with a `Cloud credentials` heading and no implementation.
- **Why it matters:** `allow(dead_code)` is masking abandoned design rather than documenting a required interface. The orphan heading makes ownership unclear, and dead lease-related fields imply work that is not actually connected to the probe path.
- **Fix:** Delete the unused fixture helper and dead fields, simplify `VerifiedInstallation`, and remove the orphan section. If a field is required for a lease, pass it into the lease-producing API and add a behavioral consumer first.

#### RT-15 — Low — Audit-ticket and phase narration has become production maintenance noise

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/runtime.rs:1552-1582`, `2355-2633`, `3814-3816`, `3989-3997`, `4503-4507`, `4638-4642`, plus the test block `4808-9147`; `src-tauri/src/runtime_service.rs:1-5,72-73`; `src-tauri/src/preflight.rs:117-120`.
- **Evidence:** The runtime file contains dozens of `audit RT-*`, `S-*`, `A-*`, `Phase 1 RED`, `Phase 2`, and `V1/V2` labels; a static scan found 57 audit-like label occurrences across 33 labels. Comments such as “This test failed before...” narrate an audit ticket rather than the invariant.
- **Why it matters:** Ticket IDs, stale release names, and phase history are not the contract. They make code review harder, encourage preserving obsolete scaffolding, and are already out of sync with the actual lease behavior. The 4,340-line test module amplifies the problem.
- **Fix:** Keep short invariant/rationale comments at security boundaries, move historical audit evidence to the project ledger, delete phase narration, and split tests into `runtime_tests.rs`/fixture modules. Do not use comments as verification evidence.

### Answers to specific questions

#### 1. Catalog fetch: auth, cache, retries, timeouts, errors, and offline behavior

- **URL and authentication:** `runtime.rs:15` defines `https://api.github.com/repos/ggml-org/llama.cpp/releases/tags`; `approved_release_url` constructs the approved tag URL at `1949-1951`. `runtime_catalog_client` installs only GitHub `Accept` and a Localmotive `User-Agent` at `2502-2515`. There is no `Authorization` header, token lookup, credential-manager access, or authenticated GitHub client. The unit test at `6191-6244` explicitly asserts that no authorization, cookie, bearer, or API-key header is sent.
- **Timeouts and retries:** The client has a 15-second total timeout and a five-second connect timeout at `2500`, `2514-2515`. `fetch_catalog_http` calls `request.send().await` once at `2530-2541`; there is no retry/backoff loop. Request timeout maps to `Timeout`; other transport failures map to `Http`.
- **HTTP/error kinds:** `429` maps to `RateLimited` and preserves `Retry-After` at `2545-2556`. A `403` body containing “rate limit” also maps to `RateLimited` at `2558-2576`; other `403` and non-success statuses map to `Http`. Oversized bodies map to `BodyTooLarge` at `2594-2601` or during bounded streaming at `2630-2657`; invalid UTF-8 or non-release JSON maps to `InvalidResponse` at `2611-2627`. Approved-manifest identity failures become `TrustFailure` at `2854-2864`.
- **Cache behavior:** The cache is `cache/runtime-catalog-v1.json` at `2671-2676`. Its schema, URL, approved tag/commit/manifest digest, body digest, and release identity are checked at `2425-2454`; there is no maximum age check, only a nonzero timestamp. `fetch_catalog` loads it, still performs the network request, and uses it only when the request fails at `2874-2884`. A validated cache therefore provides a stale-on-error fallback, not an offline-first path.
- **Can a transient/rate-limit error become terminal?** Yes. With no validated cache, `fetch_catalog` returns the typed timeout/HTTP/rate-limit error. `load_runtime_setup` converts any such error into `catalog: None` plus `catalog_error` at `runtime_service.rs:54-64`. The current release gate then records `catalog_result: "terminal-error"` whenever `baselineSetup.catalog` is false at `scripts/verify_041.mjs:460-464`. A valid cache prevents terminal state by allowing `cache_catalog`; an invalid/missing cache does not.
- **Offline path:** Local managed-runtime listing is independent of the catalog: `runtime_service.rs:17-27` calls hardware detection and `list_managed_runtimes` before the catalog request, and `runtime.rs:1106-1109` documents that listing is cheap and unverified. The catalog itself has no cache-first/offline command, no bundled fallback, and no release-gate fixture injection. **UNKNOWN:** the current external GitHub quota was not rechecked because network calls were prohibited; the source proves the client is unauthenticated.

#### 2. Busy/exclusive-operation behavior

`ExclusiveOperation::acquire` is a single `AtomicBool` compare-and-exchange at `lib.rs:103-118`. The second concurrent catalog request fails immediately; it does not wait and does not join. `load_runtime_setup` catches that failure and returns a response with `catalog: None` and `catalog_error.kind = Busy` at `runtime_service.rs:38-64`. `fetch_runtime_catalog` returns the Busy error directly at `80-90`. The verification coalescer at `runtime.rs:3897-3959` is unrelated and does not coalesce catalog fetches.

#### 3. Archive extraction, install, reparse points, digests, publication, rollback, and DLL tamper checks

**Controls that are present:**

- `extract_zip_from_reader` opens a verified capability directory and bounds entry count/size/total size at `2989-3003` and `3054-3062`.
- ZIP traversal is rejected with `enclosed_name`; alternate data-stream names containing `:` are rejected at `3011-3020`; canonical duplicate output paths are rejected at `3021-3041`; Unix symlink entries are rejected at `3045-3053`.
- Files are opened with capability-relative `create_new` at `3074-3078`, not written through arbitrary extracted paths.
- The archive is opened once, sized and SHA-256 hashed through that handle, rewound, and extracted from that same handle at `3123-3168`. Reopening the archive by path is avoided.
- Post-extraction verification checks the compiled content-manifest anchor and install record, exact file inventory, every file's byte count and SHA-256, and the approved `llama-server.exe` record at `3853-3895`. `open_and_digest_managed_file` rejects reparse/symlink files, multiple hard links, alternate streams, and changes observed during hashing at `3567-3623`.
- The seven content manifests are explicit per-file inventories. `cpu.json:14-...` is representative; each manifest carries release identity and every approved DLL/EXE payload. An existing known DLL tamper is detected by the inventory/digest walk.

**Holes:** RT-01 is the unguarded install-record write; RT-02 is the verify/publish/rollback race; RT-04 is the extra-file window after inventory; RT-10 is inconsistent path validation. The archive extractor itself is the stronger part of this subsystem. The installer is not yet an atomic, crash-safe trust transaction.

#### 4. Flag derivation from `--help`

The intended authority is correct: `parse_supported_flags` derives a list from runtime help at `core.rs:1475-1495`; `filter_supported_args` compares generated tokens against that list at `1577-1601`; `validate_launch_arguments` rejects missing required generated flags and reports omitted optional flags at `1619-1669`. `LaunchProfile::build_args` generates the managed flags at `core.rs:982-1005` and `1093-1097`. No backend-name rule was found that declares a flag supported merely because the runtime is CUDA, Vulkan, or another named backend.

The implementation is not fully robust. `parse_capabilities` uses literal substring checks for `--metrics`, `--mmproj`, and `--fit` at `1672-1718`, and `parse_supported_flags` does not normalize `--flag=value` while `argument_flag` does. The exact filtering boundary is good; the help parser feeding it needs an option-aware parser and collision tests. Whether upstream currently emits `--flag=value` is **UNKNOWN** under the no-network restriction.

#### 5. Natural split for `runtime.rs`

The current file boundaries are the source slices that should move together; the ranges are intentionally non-contiguous where related types and implementations are interleaved.

| Proposed module | Current line ranges | Responsibility |
|---|---|---|
| `runtime/hardware.rs` | 1-581, 1185-1545 | Evidence helpers, Windows memory/DXGI/NVIDIA detection, adapter identity, hardware recommendation inputs. |
| `runtime/catalog_model.rs` | 125-146, 583-741, 1552-2349 | GitHub DTOs, runtime options, approval schema, compatibility records, manifest parsing, catalog construction/recommendation. |
| `runtime/discovery.rs` | 742-1183 | Runtime identity, DLL evidence, managed-runtime listing and record discovery. |
| `runtime/catalog_transport.rs` | 2390-2887 | Cache record validation, HTTP client, response/error mapping, ETag handling, network/cache fallback. |
| `runtime/archive.rs` | 2903-3373 | Approved asset resolution, adapter validation, download targets, ZIP extraction and runtime discovery. |
| `runtime/verification.rs` | 3374-4321 | Compiled content manifests, install-record identity, path/file digests, coalesced verification, execution leases. |
| `runtime/install.rs` | 4322-4601 | Staging, downloads, extraction orchestration, replacement publication, cleanup. |
| `runtime/health_model.rs` | 4603-4807 | Pinned health-model cache/repair and managed-health context lease creation. |
| `runtime/tests.rs` plus fixtures | 4808-9147 | All unit/security fixtures and tests; currently more than half of the file after the production code. |

Longest production functions in the current monolith are `parse_approved_manifest` (`1718-1935`, 218 lines), `detect_dxgi_adapters` (`414-577`, 164 lines), `build_approved_catalog` (`2022-2180`, 159 lines), `detect_hardware` (`1336-1477`, 142 lines), and `install_runtime` (`4462-4602`, 141 lines). The longest preflight function is `build_preflight_report` (`450-615`, 166 lines). Splitting by trust responsibility is more important than merely making files smaller.

#### 6. Dead code, duplication, test-only production code, audit narration, and version pins

- **Dead code:** `GithubAsset::sample` is an unused test-only helper at `runtime.rs:148-158`. `VerifiedInstallation.install_dir` and `.install` are explicitly `allow(dead_code)` at `3990-3997`; the current caller only needs `server_path`. `runtime_service.rs:273-275` is an empty cloud-credentials section.
- **Duplicated safety logic:** `runtime.rs` has `safe_directory` (`992-996`), capability-directory opening (`2972-2981`), install inventory validation (`3631-3674`), and path-based record/cache reads (`2679-2696`, `3776-3792`). `artifact.rs:249-287` owns ancestor/file validation and `download.rs:672-704` owns safe write-entry checks. The runtime installer should reuse those authorities or expose one shared capability helper; it should not keep a weaker `fs::write` variant.
- **Test-only code in the production module:** `runtime.rs:4808-9147` is a `#[cfg(test)]` module. Additional `#[cfg(test)]` helpers are interspersed at `148`, `812`, `818`, `826`, `1546`, `1980`, `1989`, `2185`, `2301`, `2889`, `2925`, and `3113`. `preflight.rs:802-1261` is also tests. Move these into dedicated test modules so production review does not require reading 4,000 lines of fixtures and audit narration.
- **Audit-ticket narration:** A static scan found 57 audit-like label occurrences across 33 labels in `runtime.rs`, plus phase/RED history throughout the test block. Examples are `runtime.rs:2355-2633`, `3814-3816`, `4503-4507`, and `4638-4642`. Keep invariant comments; move ticket history and “this failed before” narratives to the evidence ledger.
- **Version-pinned constants:** `approved_runtimes.json:3-10`, all seven content manifests, and both `b10816` fixtures intentionally pin one approved release. The avoidable coupling is in tests and prose: `runtime.rs:1552` and `1581` still describe `b10796`, while `5354-5403` and `6549` encode historical/current build names. Derive the active tag from `approved_runtime_identity()` and generate fixtures from the approved bundle.

### Cross-cutting observations

- The approval chain is stronger than the catalog transport. Compiled approval checks release identity, asset URL/size/digest, required jobs, compatibility records, and per-file content manifests (`runtime.rs:1718-1934`, `3705-3775`), but the app still makes the release gate depend on live unauthenticated GitHub metadata.
- The code has two different meanings of “verified”: discovery explicitly returns `content_verified: false` (`runtime.rs:1106-1109`, `1147-1154`), while server launch and managed health retain file handles. Public inspection and legacy health use neither meaning consistently; they verify once and then execute without a lease.
- `download.rs` demonstrates the safer pattern already available: link/reparse checks, hard-link count checks, target reservation, partial recovery, and verified publication (`download.rs:672-704`, `1179-1207`, `1402-1407`). Runtime install should use that authority instead of growing parallel path-based checks.
- The release gate currently proves the catalog twice per normal packaged path: `load_runtime_setup` calls `fetch_catalog`, then the verifier calls `fetch_runtime_catalog` again. That is an avoidable API dependency and a direct rate-limit multiplier.
- No cargo, npm, Tauri, app, or network command was run. This review is static and evidence-based under the read-only constraint. The race findings require regression tests before they can be considered closed.
- Repository read-only status was preserved. Existing untracked packaged artifacts were present in the repository working tree; they were not modified. The review artifact was written outside the repository at the requested path.

---

## Part 5 — Rust command surface and core model (finding prefix CORE)

### Summary

This is a static, read-only review of the ten explicitly scoped files. `lib.rs` and `core.rs` were read in full in line-bounded chunks. Caller and boundary evidence was read from the relevant service modules, `src/App.tsx`, `src/main.tsx`, `src/model.ts`, `catalog.rs`, `download.rs`, `artifact.rs`, `runtime.rs`, `cloud.rs`, `evidence.rs`, `tune_service.rs`, `server_service.rs`, and the packaged verifier scripts. No build, test, application launch, network request, or repository write was performed.

- There are **61 registered `#[tauri::command]` functions**. The handler list is in `src-tauri/src/lib.rs:2262-2324`.
- There are **50 unique literal frontend `invoke(...)` names** in `src/`. **11 registered commands have no such frontend call**; they are dead, legacy, verifier-only, or stale IPC surface unless another non-frontend caller exists.
- The final server-launch path is mostly correct about capability filtering: it runs the selected runtime's probes, parses its `--help`, filters generated arguments, and rejects missing managed flags. The preview path intentionally does not do that validation and is only provisional.
- Split-model discovery is deterministic and returns `firstShard`, but the authoritative launch path still sends `LaunchProfile.model` to `-m` without checking it equals the inspected artifact's `first_shard`. The frontend normally supplies `firstShard`; Rust does not enforce the invariant.
- The exclusive catalog guard rejects rather than joins. React StrictMode invokes the startup effect twice in development, so the second read-only setup request can win the sequence race with a `Busy` result and replace the usable catalog with an error state.
- Several direct IPC payloads remain unbounded at deserialization or before expensive work: `selected_adapter_ids`, companion arrays, tuning proposal maps, `extra_args` count, cloud secret/model strings, legacy health request fields, and `Workload.id` are examples. Some other payloads are correctly bounded downstream; this is not a claim that every `String` is unsafe.
- The production binary contains verifier environment seams outside `cfg(test)`. A user-controlled environment can replace the catalog URL, catalog verifying key, catalog cache root, and loopback Hugging Face base. `LOCALMOTIVE_SKIP_HARDWARE_PROBE` also changes release hardware evidence. The packaged matrix deliberately injects `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=...`, and the application has no production-side stripping of it.
- `lib.rs` is 4,428 lines, with 2,077 test-module lines (2352-4428, 46.91%) and 2,351 preceding lines. `core.rs` is 4,562 lines, with 2,215 test-module lines (2348-4562, 48.55%). Both files are past the point where source-pattern tests and cross-domain edits are safe.

No Critical finding is assigned. High findings are still release-relevant: they concern trust-source replacement, a startup race in development, possible abandoned-worker overlap, launch identity correctness, IPC resource exhaustion, and local CDP exposure.

### Per-file verdicts

| File | Verdict | Lines | Largest functions over 150 lines |
|---|---|---:|---|
| `src-tauri/src/lib.rs` | **SPLIT** | 4,428 | `preflight_model` 1609-1787 (179); `wait_until_healthy_inner` 1249-1414 (166) |
| `src-tauri/src/core.rs` | **SPLIT** | 4,562 | `build_args` 894-1275 (382); test `s13_impossible_profile_domains_fail_before_launch_and_name_the_value` 4170-4352 (183) |
| `src-tauri/src/main.rs` | **KEEP** | 6 | None |
| `src-tauri/build.rs` | **KEEP** | 3 | None |
| `src-tauri/Cargo.toml` | **KEEP** | 57 | None |
| `src-tauri/.cargo/config.toml` | **KEEP** | 2 | None |
| `src-tauri/tauri.conf.json` | **FIX** | 48 | None |
| `src-tauri/capabilities/default.json` | **FIX** | 50 | None |
| `src-tauri/.gitignore` | **KEEP** | 7 | None |
| `rust-toolchain.toml` | **KEEP** | 7 | None |

`lib.rs` should retain the shared `AppState`, operation coordinator, and the narrow Tauri bootstrap, but move the existing cohesive blocks without adding another policy layer:

- `operation_state.rs`: `AppState`, `OperationOwner`, reservation/stop ownership helpers, approximately `lib.rs:103-282`.
- `launch_service.rs`: artifact validation, launch preparation, process creation, health wait, log/failure evidence, approximately `lib.rs:421-1414`.
- `model_service.rs`: scan, runtime/health/description, GGUF and artifact IPC, approximately `lib.rs:1416-1580`.
- `preflight_service.rs`: `PreflightRequest`, preflight planning, approximately `lib.rs:1574-1787`.
- `cloud_service.rs`: cloud credential/provider commands, approximately `lib.rs:1844-1904`.
- `download_service.rs`: download authorization, target validation, progress, cancellation, approximately `lib.rs:1915-2205`.
- Keep `run`/handler registration near `lib.rs:2218-2350`; move the tests beside the moved behavior instead of keeping one 2,000-line test module.

`core.rs` should be split by existing responsibility, not by new wrapper layers:

- `model_scan.rs`: shard naming, companion roles/ranking, bounded recursive scan, approximately `core.rs:14-550`.
- `profile.rs`: `LaunchProfile`, domain/input validation, argument generation and raw-extra-argument validation, approximately `core.rs:552-1458`.
- `runtime_probe.rs`: help parsing, capability derivation, argument filtering, probes and health decisions, approximately `core.rs:1460-2200`.
- `benchmark_math.rs`: benchmark parsing and summaries, approximately `core.rs:2200-2347`.
- Put the current tests beside those modules. Do not preserve source-text tests as a substitute for behavior tests.

### Findings

#### Critical

No Critical findings. The high-severity trust and local-debugging problems below are serious, but the evidence reviewed does not show remote network exposure, secret exfiltration, or unconditional data loss.

#### High

##### CORE-01 — Medium (lead; reviewer said High) — Production environment variables can replace the signed catalog authority

> **Lead check: CORRECTED.** The override needs same-user control of the environment.

- **Location:** `src-tauri/src/lib.rs:2253-2257`; `src-tauri/src/catalog.rs:28-68,82-101`; `src-tauri/src/download.rs:345-370`.
- **Evidence:**
  > `lib.rs:2254-2257` — `pub fn run() {` / `// Record verifier-only catalog source overrides...` / `catalog::apply_env_verify_source();`
- **Why it matters:** `apply_env_verify_source` is called on every run, not behind `cfg(test)`. If `LOCALMOTIVE_VERIFY_ISOLATED_ROOT` exists, the release process accepts a loopback `LOCALMOTIVE_CATALOG_URL`, an arbitrary 32-byte `LOCALMOTIVE_CATALOG_PUBKEY`, and an environment-selected cache root under the supplied isolated path. The download module has a matching release hook: with the same environment gate, `LOCALMOTIVE_HF_BASE` can rebase canonical Hugging Face downloads to a loopback server. There is no cryptographic proof that the environment was set by the verifier. A user, shortcut, wrapper, or attacker controlling process launch can make the production binary trust a catalog signed by a key supplied in the environment. The comment at `catalog.rs:32-34` says normal users keep the shipped key, but the implementation only tests environment presence.
- **Fix:** Remove these seams from the publishable binary. Put the fixture authority in a verifier-only build/launcher that is not the release artifact, or require an authenticated, compiled verifier capability rather than an environment-presence switch. Apply the same removal to `download::resolve_download_base`. Add a packaged test proving a normal release process ignores all `LOCALMOTIVE_VERIFY_*`, `LOCALMOTIVE_CATALOG_*`, and `LOCALMOTIVE_HF_BASE` values.

##### CORE-02 — Low (lead; reviewer said High) — StrictMode turns the read-only runtime catalog guard into a startup race

> **Lead check: CORRECTED.** React StrictMode runs effects twice only in development builds. The packaged collision is App.tsx:1418 against the harness call (RT-06).

- **Location:** `src/main.tsx:6-11`; `src/App.tsx:467-541,543-563,1417-1427`; `src-tauri/src/runtime_service.rs:38-64,79-114`.
- **Evidence:**
  > `runtime_service.rs:38-53` — `ExclusiveOperation::acquire(...)` / `Err(message) => Err(RuntimeCatalogError {` / `kind: RuntimeCatalogErrorKind::Busy,`
- **Why it matters:** `load_runtime_setup` and `fetch_runtime_catalog` use the same atomic guard. They do not join an in-flight request. `load_runtime_setup` converts a second request into a successful setup response with `catalog: None` and `catalog_error: Busy`; the frontend accepts that response. The app is rendered under `<React.StrictMode>` and its mount effect calls `loadRuntimeSetup()` without an idempotent request/coalescing layer. In development StrictMode, the effect is mounted, cleaned up, and mounted again while the first request is still in flight. The second request can acquire the frontend sequence, receive `Busy`, and make the runtime screen show a catalog error; the first result is then discarded as stale. The exact packaged production behavior of this StrictMode double-mount was not executed here, but the development race is directly implied by the source.
- **Fix:** Make `load_runtime_setup` join or share the existing catalog future/cache instead of returning `Busy` for an equivalent read. Alternatively make the frontend startup effect idempotent with one shared promise and retain the latest successful catalog rather than overwriting it with a busy response. Add a double-mount regression test.

##### CORE-03 — High — An abandoned tuning command can release ownership while its blocking worker keeps running

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/tune_service.rs:244-266,361-371`; `src-tauri/src/server_service.rs:26-32`; `src-tauri/src/lib.rs:250-269,4217-4234`.
- **Evidence:**
  > `tune_service.rs:361-365` — `})` / `.await` / `.map_err(|error| error.to_string());` / `clear_tuning(&state);`
- **Why it matters:** `start_tuning` stores the operation reservation in a local RAII variable and starts `spawn_blocking`. If the command future is dropped, the reservation drops, but a started Tokio blocking task is not stopped merely because its `JoinHandle` is dropped. The cleanup call and tuning-slot clear occur only after the await returns. `start_server` checks the abandoned benchmark slot but does not check the tuning slot; after the reservation is released it can acquire the coordinator and launch while the abandoned tuning worker is still launching/measuring. The existing regression test covers this exact abandoned shape for benchmarks, not tuning. Whether a particular Tauri caller disconnect causes this command future to be dropped is **UNKNOWN** from the static review; the worker/lease behavior if it happens is clear.
- **Fix:** Bind ownership to the worker lifetime, not only to the command future. Store a join/drain handle and cancellation flag in `AppState`, have every replacement command refuse all live worker slots, and drain the worker before releasing the operation reservation. Add the tuning equivalent of `abandoned_benchmark_slot_still_refuses_replacement_work` and exercise a dropped command future.

##### CORE-04 — High — The launch path discovers `first_shard` but does not enforce it for `-m`

> **Lead check: VERIFIED.** core.rs:987 passes profile.model to -m. lib.rs:443-449 does not compare it with first_shard. The frontend default at model.ts:1479 is correct.

- **Location:** `src-tauri/src/core.rs:430-463,528-539,982-987`; `src-tauri/src/artifact.rs:488-504`; `src-tauri/src/lib.rs:621-626`.
- **Evidence:**
  > `core.rs:982-987` — `let mut args = Vec::new();` / `let push = ...;` / `push(&mut args, "-m", self.model.clone());`
- **Why it matters:** Discovery sorts shard indices and exposes the first path. Artifact inspection also computes `first_shard`. But `prepare_launch` validates the artifact and then calls `validate_launch_arguments(profile, ...)`; it never rewrites or rejects `profile.model` based on `artifacts[0].first_shard`. A caller can therefore submit a valid-looking `LaunchProfile` whose model is shard 2, and the generated argv passes shard 2 to `llama-server`. The normal React model builder uses `model.firstShard` (`src/model.ts:1476-1481`), but the Rust IPC boundary must enforce the invariant independently.
- **Fix:** Normalize a validated launch profile to the inspected `first_shard` before `build_args`, or reject any `profile.model` that is not exactly the inspected first shard. Test a split model with a second-shard profile and assert the actual effective argv contains only shard 1.

##### CORE-05 — High — Several IPC payloads are typed but not bounded before allocation or expensive work

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/lib.rs:1479-1502,1557-1569,1574-1583,1639-1669,1854-1882`; `src-tauri/src/core.rs:642,1272-1274,1432-1456`; `src-tauri/src/tune_service.rs:173-209,400-418`; `src-tauri/src/evidence.rs:257-313`.
- **Evidence:**
  > `lib.rs:1574-1582` — `selected_adapter_ids: Vec<String>` / `manual_overrides: Vec<runtime::HardwareOverride>` / `reserve_bytes: Option<u64>`
- **Why it matters:** `selected_adapter_ids` has no count or per-item bound before the preflight worker iterates it, builds a result for every entry, and repeatedly searches hardware/manual overrides. `LaunchProfile.extra_args` has a per-token 1,024-byte check but no count or total-byte limit before cloning and extending the argv. Artifact inspection accepts an unbounded companion vector and opens every supplied path. Tuning accepts an unbounded companion vector, `spec_types`, and `BTreeMap<String, Value>`; `Workload.id` is only checked for non-empty, not length. `cloud_save_credential` accepts a secret with no maximum length in `lib.rs:1855-1859` and `cloud.rs:223-236`; `cloud_probe` accepts an unbounded model string. Legacy health accepts unbounded path/backend/model strings and an adapter vector. Some adjacent inputs are correctly bounded—catalog payloads, download repo/filename, connections, and manual overrides—so the issue is the missing boundary policy on these specific paths.
- **Fix:** Reject input at each Tauri command boundary with shared constants for total JSON/string bytes, vector counts, map counts, and per-item text. Bound `selected_adapter_ids`, `extra_args` count and total bytes, artifact companions, tuning companions/spec types/changes, `Workload.id`, cloud model/secret, and legacy health fields before `spawn_blocking`, filesystem access, keyring writes, or network calls.

##### CORE-06 — Medium (lead; reviewer said High) — Release WebView remote debugging is enabled by inherited environment

> **Lead check: CORRECTED.** Standard WebView2 behavior. It needs same-user control of the environment.

- **Location:** `src-tauri/tauri.conf.json:22-24`; `src-tauri/capabilities/default.json:8-48`; `src-tauri/src/main.rs:1-6`; `scripts/verify_packaged_impl.ps1:274-277`; `scripts/verify_csp.mjs:18-25`.
- **Evidence:**
  > `verify_packaged_impl.ps1:274-277` — `$env:LOCALAPPDATA = $IsolatedRoot` / `$env:LOCALMOTIVE_VERIFY_ISOLATED_ROOT = $IsolatedRoot` / `$env:WEBVIEW2_USER_DATA_FOLDER = $Profile` / `$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$CdpPort --user-data-dir=$Profile"`
- **Why it matters:** The release build has no Rust or Tauri configuration that strips `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`; `windows_subsystem` only hides the console. The packaged matrix intentionally starts the candidate with a WebView2 remote-debugging port and then drives it over CDP. CSP and Tauri opener permissions do not disable Chromium/WebView2 CDP. A local process able to reach that port can inspect and drive the webview, including invoking the frontend's permitted IPC surface. The current executable was not launched under the read-only constraint, so live port reachability is **UNKNOWN**; the release-verifier code path and intended exposure are explicit.
- **Fix:** Do not inherit arbitrary browser arguments in the publishable binary. Run CDP checks against a verifier-only build/launcher or an explicit non-publishable verification mode, and strip/deny `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` and related profile variables in normal release startup. Add a packaged negative check that a normal launch has no debugger endpoint.

#### Medium

##### CORE-07 — Medium — `LOCALMOTIVE_SKIP_HARDWARE_PROBE` is a release behavior switch, not a test-only hook

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/runtime.rs:1336-1355` (called by `runtime_service.rs:17-24,91-97`); release environment hook context also appears in `src-tauri/src/catalog.rs:82-101`.
- **Evidence:**
  > `runtime.rs:1341-1348` — `if std::env::var_os("LOCALMOTIVE_SKIP_HARDWARE_PROBE").is_some() {` / `... vendor: "cpu" ...` / `detection_status: "Hardware probing skipped by LOCALMOTIVE_SKIP_HARDWARE_PROBE"`
- **Why it matters:** This branch is not under `cfg(test)`. Any release process launched with the variable reports no adapters and CPU hardware, changing runtime recommendations, preflight decisions, and evidence. It is useful for CI but it is an integrity-affecting production switch. The code does not distinguish an authorized verifier from an ordinary user.
- **Fix:** Move the skip to a verifier-only binary or feature. In the normal artifact, fail/report unknown on probe failure rather than accepting an environment opt-out that changes product facts. If a release matrix must skip probes, record that mode outside the product's evidence and ensure it cannot be set by a normal launch.

##### CORE-08 — Medium — Capability derivation is real-help-based at launch but the parser and preview are weaker than the contract

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/core.rs:1460-1495,1577-1670`; `src-tauri/src/lib.rs:1789-1804`.
- **Evidence:**
  > `core.rs:1475-1494` — `help.split_whitespace()` / `clean = token.trim_matches(...)` / `if clean.starts_with('-') ... Some(clean.to_string())`
- **Why it matters:** The authoritative path does call the selected runtime's probes, uses `supported_flags`, omits unsupported arguments, and rejects unsupported managed flags (`core.rs:1619-1665`). That satisfies the core safety requirement for actual launch. However, `parse_supported_flags` treats every option-looking token anywhere in help text as support; whether the current llama help output contains misleading option mentions was not executed and is **UNKNOWN**. More concretely, `preview_command` builds raw profile args and returns PowerShell/CMD strings without runtime inspection or capability filtering (`lib.rs:1795-1802`). The UI can therefore display/copy a command that final launch will reject. The command string is not executed by Rust.
- **Fix:** Parse the option sections of `--help` with fixtures covering aliases, prose, values, and wrapped descriptions, or require a capability fixture format from the runtime. Make preview use the same validation result when a runtime is available and label unvalidated previews as such, including rejected flags.

##### CORE-09 — Medium — The capability file is not minimal by inspection

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/capabilities/default.json:8-48`; `src-tauri/tauri.conf.json:22-24`; `src-tauri/.gitignore:5-7`.
- **Evidence:**
  > `default.json:8-12` — `"core:default"` / `"dialog:allow-open"` / `{ "identifier": "opener:allow-open-url",`
- **Why it matters:** `core:default` is a permission bundle rather than an explicit least-privilege list. The generated schema expansion is ignored (`.gitignore:5-7`), so the exact extra core operations are not available in this repository. `dialog:allow-open` is broad but plausibly required for file/folder selection. The opener allowlist is materially broad: wildcard paths on Hugging Face, GitHub, all listed provider consoles, and `http://127.0.0.1/*` permit opening arbitrary paths/ports on those origins. The opener does not itself grant filesystem access or shell execution, and the loopback wildcard is likely needed for user-selected llama-server ports, but the ACL is still broader than the observed exact application links.
- **Fix:** Generate and review the expanded ACL, replace `core:default` with only the core permissions actually used, and document each plugin permission. Narrow opener patterns to exact product/provider origins and the smallest loopback scope supported by the server-port contract. Keep `dialog:allow-open` only if all current file/folder pickers require it.

##### CORE-10 — Medium — Registered dead commands and duplicated read paths enlarge the IPC surface

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** registration `src-tauri/src/lib.rs:2262-2324`; legacy commands `lib.rs:1416-1502`; cancellation `lib.rs:1518-1555`; duplicate runtime setup `runtime_service.rs:12-114`; duplicate scan wrappers `lib.rs:1416-1451`.
- **Evidence:**
  > `lib.rs:1423-1425` — `/// ... for the application path ...` / `/// ... the legacy scan_models command stays for verifier scripts.`
- **Why it matters:** The literal frontend grep found no `invoke` for `cancel_gguf_read`, `catalog_fit_budget`, `check_runtime_health`, `download_eta`, `format_bytes`, `managed_runtime_root`, `remove_user_catalog_override`, `runtime_verification_stats`, `save_user_catalog_override`, `scan_models`, or `validate_launch_profile`. They remain registered and callable by a compromised or stale webview. In particular, `read_gguf_summary` publishes a cancellation flag but the UI never invokes `cancel_gguf_read`, so the intended cancellation path is unreachable from the current frontend. `load_runtime_setup` and `fetch_runtime_catalog` duplicate hardware validation and catalog recommendation logic while sharing a non-joining guard; `scan_models` and `scan_models_report` duplicate wrappers with different state/cancellation behavior.
- **Fix:** Remove commands after verifier/tests migrate, or add real frontend callers and contract tests. Make one runtime-catalog read path that returns the shared result, and one scan path with explicit legacy compatibility only where an external verifier demonstrably requires it. Do not retain an IPC command solely because a source-pattern test mentions it.

##### CORE-11 — Medium — The two central files mix unrelated production domains with source-text tests and audit-ticket narration

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/lib.rs:17-36,2352-2369,3638-3644,4315-4352`; `src-tauri/src/core.rs:2348-2360,2952-2971,4170-4352`.
- **Evidence:**
  > `lib.rs:17-22` — `/// Test-only concatenation of every source file...` / `#[cfg(test)]` / `pub(crate) const ALL_SOURCES...`
- **Why it matters:** The source-pattern guards concatenate whole source files and split on function spelling. Other tests use `include_str!("core.rs")` and assert that a substring contains a runner or guard. These tests can pass while behavior is wrong and fail on harmless extraction/renaming; they are a direct tax on the requested module split. Production comments also carry review-ticket narration such as `review deleg_16c0e72`, `audit MT-05`, `audit IPC-01`, `audit S-27`, and `audit GH-05` throughout both files. `U06`, `GH-01`, and `S-25` were not found in these two files; their presence in other scripts is not evidence for this scope.
- **Fix:** Replace source scans with behavior tests using real temporary files, contained processes, and observable IPC results. Move historical audit mapping to the project ledger/docs. Keep only short invariants beside the code. Split production blocks and move tests with them; do not preserve `ALL_SOURCES` as a cross-file policy mechanism.

##### CORE-12 — Medium — Raw error conversion and poisoned-lock unwraps lose recovery information or panic the app

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/lib.rs:1871-1882,1901-1903,2078-2099`; `src-tauri/src/core.rs:2281-2283,441-443,506-513`.
- **Evidence:**
  > `core.rs:2281-2283` — `let value: serde_json::Value = serde_json::from_str(body).map_err(|e| e.to_string())?;`
- **Why it matters:** Several Tauri paths surface raw `JoinError`, JSON, or filesystem strings without naming the operation or recovery. `cloud_list_models`, `cloud_probe`, and the OAuth worker use `map_err(|error| error.to_string())`; the benchmark JSON parser does the same. Download/catalog command paths use `state.catalog.lock().unwrap()` and `state.downloads.lock().unwrap()`, so a poisoned lock becomes a process panic rather than a structured `Result<String>`. Other nearby code already wraps errors with phase and path, so the inconsistency is avoidable.
- **Fix:** Use an internal error type carrying operation, kind, and recovery, map it to the stable Tauri string contract at the boundary, and replace production state-lock unwraps with fail-closed `map_err` results. Preserve the underlying error text as bounded diagnostic detail.

#### Low

No separate Low finding is needed. The remaining issues are either covered by the Medium capability/ACL and error findings or are cleanup items after the dead commands are removed.

### Answers to specific questions

#### 1. Every registered Tauri command, unused frontend commands, and unbounded inputs

The exact command attribute lines are listed below. They are all included in the handler at `src-tauri/src/lib.rs:2262-2324`.

**`src-tauri/src/catalog_service.rs`**

- `load_model_catalog:28`
- `fetch_model_catalog:58`
- `catalog_local_models:156`
- `save_user_catalog_override:198`
- `remove_user_catalog_override:211`
- `filter_catalog:223`
- `catalog_facets:234`
- `catalog_rich_facets:244`
- `catalog_fit_budget:254`
- `hf_token_status:269`
- `save_hf_token:274`
- `clear_hf_token:279`

**`src-tauri/src/lib.rs`**

- `scan_models:1416`
- `scan_models_report:1426`
- `cancel_scan:1454`
- `inspect_runtime:1468`
- `check_runtime_health:1489`
- `describe_runtime:1505`
- `list_managed_runtimes:1513`
- `read_gguf_summary:1518`
- `cancel_gguf_read:1544`
- `inspect_model_artifact:1557`
- `preflight_model:1608`
- `preview_command:1789`
- `validate_launch_profile:1823`
- `cloud_providers:1844`
- `cloud_credential_status:1849`
- `cloud_save_credential:1854`
- `cloud_clear_credential:1862`
- `cloud_list_models:1867`
- `cloud_probe:1876`
- `cloud_openrouter_login:1888`
- `suggest_port:1910`
- `download_catalog_file:2062`
- `cancel_download:2189`
- `format_bytes:2207`
- `download_eta:2213`
- `about_info:2218`

**`src-tauri/src/measurement_service.rs`**

- `benchmark_server:52`
- `benchmark_v2:603`
- `cancel_benchmark:736`
- `replay_benchmark_manifest:751`

**`src-tauri/src/runtime_service.rs`**

- `load_runtime_setup:12`
- `detect_hardware:67`
- `runtime_verification_stats:74`
- `fetch_runtime_catalog:79`
- `managed_runtime_root:117`
- `install_managed_runtime:124`
- `cancel_managed_runtime_install:149`
- `check_managed_runtime_health:160`
- `repair_health_model:211`
- `cancel_health_model_repair:248`
- `cancel_managed_runtime_health:262`

**`src-tauri/src/server_service.rs`**

- `start_server:20`
- `stop_server:169`
- `server_status:275`
- `read_server_log:326`

**`src-tauri/src/tune_service.rs`**

- `tune_disclosure_list:223`
- `start_tuning:228`
- `cancel_tuning:374`
- `apply_tuning_trial_changes:409`

The frontend grep found 50 unique literal `invoke` names. These 11 registered commands had no `invoke("name", ...)` or `invoke('name', ...)` call in `src/`:

- `cancel_gguf_read`
- `catalog_fit_budget`
- `check_runtime_health`
- `download_eta`
- `format_bytes`
- `managed_runtime_root`
- `remove_user_catalog_override`
- `runtime_verification_stats`
- `save_user_catalog_override`
- `scan_models`
- `validate_launch_profile`

**Unbounded or incompletely bounded command inputs observed:**

| Command or request | Boundary input | Evidence and downstream status |
|---|---|---|
| `scan_models`, `scan_models_report` | `root: String` | `lib.rs:1416-1445`; scan work is bounded by `ScanLimits`, but the IPC string has no explicit size cap. |
| `inspect_runtime`, `describe_runtime`, `read_gguf_summary` | `path: String` | `lib.rs:1468-1476,1505-1510,1518-1541`; no command-boundary length check. |
| `check_runtime_health` | `RuntimeHealthRequest.path`, `expected_adapters: Vec<String>`, backend/model strings | `lib.rs:1479-1502`; the legacy wrapper forwards all fields directly. |
| `inspect_model_artifact` | `first_shard: String`, `companions: Vec<String>` | `lib.rs:1557-1569`; `artifact.rs:481-484` reads every supplied companion; no count/total path budget. |
| `preflight_model` | `selected_adapter_ids: Vec<String>`; `LaunchProfile.extra_args` count | `lib.rs:1574-1582,1639-1669`; manual overrides are bounded later, but selected IDs are iterated without a count cap. `core.rs:1272-1274` clones all extra args after only per-item checks. |
| `preview_command`, `validate_launch_profile`, `start_server`, tuning profile requests | `LaunchProfile.extra_args: Vec<String>` | `core.rs:642,1432-1456`; each token is capped at 1,024 bytes, but the vector and aggregate bytes are not. Most scalar profile fields do pass `validate_profile_input_bounds`. |
| `cloud_save_credential`, `cloud_probe` | `secret: String`, `model: String` | `lib.rs:1854-1882`; provider IDs are validated, but `cloud::save_credential` has no secret maximum at `cloud.rs:223-236`, and probe model text has no explicit maximum. |
| `suggest_port` | `host: String` | `lib.rs:1910-1912`; `core::pick_free_port` restricts host syntax, not input length. |
| `download_catalog_file` and cancellation | `revision: Option<String>`, `destination: String` | `lib.rs:2062-2075,2189-2205`; repo/filename and connection count are defended, but revision/destination have no explicit byte budget. |
| `benchmark_v2`, replay | `Workload.id: String` | `evidence.rs:257-313`; validation requires non-empty but never calls `validate_text` or caps the ID. Other numeric workload fields are bounded. |
| `start_tuning`, `apply_tuning_trial_changes` | companions/spec types vectors and proposal `BTreeMap<String, Value>` | `tune_service.rs:175-209,400-418`; numeric workload fields are bounded, but collection count and proposal JSON size are not. |

#### 2. Capabilities, CSP, devtools/CDP, and environment backdoors

**CSP rating: mostly minimal, Low residual risk.** The release CSP at `tauri.conf.json:22-24` has `script-src 'self'`, no `unsafe-eval`, `object-src 'none'`, `base-uri 'self'`, `form-action 'none'`, and `connect-src` limited to the Tauri IPC origins and self. `style-src 'unsafe-inline'` and `img-src data:` are broader than strict CSP but may be required by the current React/icon implementation. The `devCsp` allows the Vite localhost origin and websocket only for development; it is not a release policy. I did not run the CSP verifier.

**ACL rating: Medium.** `core:default` is a bundle and is not demonstrably least privilege. `dialog:allow-open` is broader than a single picker but likely needed for the current file/folder selection UI. `opener:allow-open-url` permits wildcard paths on eleven origins and every `http://127.0.0.1/*` path/port (`capabilities/default.json:12-46`). It opens an external browser; it does not grant arbitrary shell execution. The exact `core:default` expansion is **UNKNOWN** because the generated schema is ignored and was not available in the reviewed tree.

**Devtools/CDP rating: High local exposure.** The packaged verifier sets `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` to a remote-debugging port at `scripts/verify_packaged_impl.ps1:274-277`, and the other packaged scripts use the same mechanism. No release startup code strips it. This makes a release executable remotely inspectable over a local CDP endpoint when launched with the inherited environment. The current binary's live endpoint was not tested under the read-only rule, so runtime reachability is **UNKNOWN**, but the intended release path is explicit and the matrix relies on it.

**Environment table:**

| Variable/path | Release behavior | Rate |
|---|---|---|
| `LOCALMOTIVE_VERIFY_ISOLATED_ROOT`, `LOCALMOTIVE_CATALOG_URL`, `LOCALMOTIVE_CATALOG_PUBKEY`, `LOCALMOTIVE_CATALOG_ROOT` | Release `run()` calls the hook; environment presence enables a loopback endpoint, caller-selected key, and caller-selected cache root. | **High**, CORE-01 |
| `LOCALMOTIVE_HF_BASE` | Release download URL rebasing is enabled when the verifier-root variable exists and accepts loopback HTTP. | **High**, CORE-01 |
| `LOCALMOTIVE_SKIP_HARDWARE_PROBE` | Release hardware detection returns synthetic CPU/no-adapter evidence. | **Medium**, CORE-07 |
| `LOCALMOTIVE_DIAG_RUNTIME` in `lib.rs:2752-2759` | Inside `#[test]` only; not compiled into a normal release build. | **None in release** |
| `LOCALAPPDATA`, `USERPROFILE`, temporary-directory APIs | Normal Windows/user profile location behavior, not a verifier trust override by itself. | **Not a backdoor observed** |
| `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`, `WEBVIEW2_USER_DATA_FOLDER` | Consumed by WebView2 before/around initialization; the packaged test harness uses them for CDP. | **High local exposure**, CORE-06 |

There is no `env::var` use in `core.rs` production code. The direct `lib.rs` environment diagnostic is test-only, but `lib.rs:2255-2257` calls the release `catalog.rs` environment hook. That distinction is the reason the catalog hook cannot be dismissed as a test-only seam.

#### 3. Busy guard and startup race

`reserve_operation` rejects any existing managed-inference owner rather than joining:

- `src-tauri/src/lib.rs:225-230`: `if let Some((current, _)) = state.active { return Err(format!("{} is already active; finish or cancel it first.", current.label())); }`
- `OperationOwner` labels server, warm benchmark, cold benchmark, and tuning at `lib.rs:153-174`.

The runtime catalog read uses a separate `ExclusiveOperation` atomic:

- `runtime_service.rs:38-53` catches a second request in `load_runtime_setup` and embeds `RuntimeCatalogErrorKind::Busy`.
- `runtime_service.rs:84-90` returns `Err` with `kind: Busy` from `fetch_runtime_catalog`.

So the answer is **yes: concurrent read-only catalog invokes are rejected, not joined**. The exact startup defect is **yes in development**: `main.tsx` uses `React.StrictMode`, while the empty-dependency effect in `App.tsx:1417-1427` starts `loadRuntimeSetup()` and only invalidates the sequence during cleanup. The first request continues, the second sees Busy, and the second response can win the frontend sequence. In a normal production React build the StrictMode double-invocation path is generally absent, but equivalent concurrent refresh/adapter invokes still do not join. The backend should coalesce read-only catalog work regardless of frontend lifecycle.

#### 4. Profile/flag generation, shard selection, companions, and shell strings

- **Help-derived at final launch: yes, with caveats.** `core::inspect_runtime` probes `--version` and `--help` through `hidden_command` (`core.rs:1728-1755`), `parse_supported_flags` derives tokens from the selected help output (`core.rs:1475-1495`), and `validate_launch_arguments` filters the generated argv and rejects missing managed flags (`core.rs:1619-1665`). This is the authoritative `start_server` path through `lib.rs:621-626` and `lib.rs:796-801`. The parser is a heuristic token scan, not a structured option parser; see CORE-08.
- **Preview is not help-derived.** `preview_command` calls `profile.build_args()` and produces shell/argv representations without loading runtime capabilities (`lib.rs:1795-1804`). It is suitable only as a provisional display.
- **First-shard discovery is deterministic but launch enforcement is incomplete.** `assemble_models` sorts shards by parsed index and path (`core.rs:450-463`) and emits `first_shard` (`core.rs:534-539`). `artifact::inspect_artifact` returns the path for index 1 (`artifact.rs:488-504`). `LaunchProfile::build_args` nevertheless uses the submitted `self.model` (`core.rs:986-987`), causing CORE-04.
- **Companion selection is deterministic.** `rank_companions` sorts by role, quantization distance, then name (`core.rs:175-192`). Tuning companion lookup uses the role-prefixed list and takes the first matching path (`tune.rs:517-540`); the discovery list is already ranked.
- **No shell command is executed from a shell string.** `escaped_command_with_args` exists for display/copy and explicitly documents that the app launches with an argument array (`core.rs:1277-1286`). Actual launch uses `proc::hidden_command(&profile.runtime).args(&validation.arguments.effective_args)` (`lib.rs:796-801`). The PowerShell/CMD strings are still an output surface that can be misleading or overlarge, but they are not the process-launch mechanism.

#### 5. Splitting `lib.rs` and `core.rs`

The concrete split is in the per-file verdicts above. The current largest production functions are:

- `lib.rs:1609-1787`, `preflight_model`, 179 lines: hardware observation, artifact facts, capacity selection, report construction, and device planning.
- `lib.rs:1249-1414`, `wait_until_healthy_inner`, 166 lines: transport retries, process exit, timeout evidence, and sleeps.
- `core.rs:894-1275`, `LaunchProfile::build_args`, 382 lines: domain validation, transport file validation, and every generated flag.

The largest core function after that is a 183-line test (`core.rs:4170-4352`). The core production `validate_domains` function is 146 lines (`747-892`), just below the requested 150-line threshold, and should move with `profile.rs` anyway.

`lib.rs` test split: `#[cfg(test)] mod release_security_tests` begins at line 2352. Lines 2352-4428 are 2,077 lines, 46.91% of the file; lines 1-2351 are the remaining 53.09% and include a small test-only `ALL_SOURCES` declaration. `core.rs` tests begin at line 2348; lines 2348-4562 are 2,215 lines, 48.55% of the file.

#### 6. Dead code, duplication, test hooks, audit narration, and weak errors

- **Dead registered commands:** the 11-name list in Q1 has no literal frontend invoke. `scan_models` is explicitly retained as a legacy verifier command (`lib.rs:1423-1425`). `cancel_gguf_read` has a real backend flag but no current frontend caller (`lib.rs:1518-1555`).
- **Duplicated logic:** `load_runtime_setup` and `fetch_runtime_catalog` both detect/validate adapter state and call catalog fetch/recommendation (`runtime_service.rs:17-64,91-114`) but differ in Busy handling. `scan_models` and `scan_models_report` wrap the same core scan with different cancellation/state behavior. `preview_command`, `validate_launch_profile`, and authoritative launch all call `build_args` but expose different levels of validation. `check_runtime_health` is an old public path beside the newer managed-health family.
- **Test-only hooks that are correctly excluded:** `ALL_SOURCES` is `#[cfg(test)]` at `lib.rs:21-36`; `LOCALMOTIVE_DIAG_RUNTIME` is inside a `#[test]` at `lib.rs:2752-2759`; the core test module begins at `core.rs:2348`.
- **Hooks incorrectly present in release:** catalog source overrides, HF-base rebasing, and hardware-probe skipping are in normal compiled functions, not `cfg(test)`; see CORE-01 and CORE-07.
- **Audit-ticket narration:** actual labels read in these files include `audit MT-05`, `audit IPC-01`, `audit S-15`, `audit RT-04`, `audit GH-05`, `audit S-27`, and `review deleg_16c0e72`. `U06`, `GH-01`, and `S-25` were not found in `lib.rs` or `core.rs` by the direct search. The comments are not harmless when source-pattern tests depend on them or when they describe a historical review rather than a current invariant.
- **Errors that lose kind/context:** raw `error.to_string()` appears at `lib.rs:1871-1882,1901-1903`, and `core.rs:2281-2283`. Filesystem errors in `core.rs:442-443,506-513` also lose operation/path context at the conversion site. The `Result<T,String>` boundary itself is required by the project rule, but these messages should still name the operation and recovery. `state.catalog.lock().unwrap()` and `state.downloads.lock().unwrap()` at `lib.rs:2078-2099,2151,2198` panic on poisoning instead of returning the required boundary error.

### Cross-cutting observations

1. **The command boundary is not the authority everywhere.** Profile validation is deliberately backend-owned and fairly broad, catalog/query validation is defensive, and download target validation is meaningful. But shard identity, several collection sizes, cloud secret size, tuning proposal size, and legacy command inputs still rely on the frontend or downstream incidental limits.
2. **The safest launch mechanism is present.** `hidden_command` plus `.args(...)` is used for actual runtime execution. No user text is interpolated into a shell command for launch. The display command is a separate, explicitly named shell string.
3. **The signed-catalog design is undermined by its release verifier seam.** The loopback restriction prevents a remote URL override, but it does not prevent a user-controlled local fixture from choosing the verifying key. That is a test harness authorization mechanism compiled into production, not merely a harmless diagnostic.
4. **Read-only work should be coalesced, not globally rejected.** The runtime catalog guard is a mutex-like single-flight flag but has no shared result. The current frontend lifecycle exposes that design immediately under StrictMode.
5. **Operation ownership is asymmetrical.** Benchmarks received an explicit abandoned-worker slot/drain treatment (`measurement_service.rs:709-723` and `lib.rs:250-269`); tuning does not. The code and tests should use one worker-lifetime ownership pattern for all inference jobs.
6. **The release/debug boundary is not represented in Tauri config.** Release CSP is reasonably tight, but a parent process environment can add WebView2 CDP. Packaged verification and production launch are currently the same binary path, so the matrix's useful CDP seam is also a release attack surface.
7. **No runtime or build claim was executed.** Cargo, npm, Tauri, the packaged executable, CDP, and network paths remain unverified in this review because the delegated rules explicitly prohibited them. The file evidence supports the findings; live release behavior should be confirmed after the source fixes with the existing gates.

---

## Part 6 — Rust process, transport and health (finding prefix PROC)

### Summary

- Review basis: revision `9b098577095ab80ead9bff6e5475456166b340c1`. The seven requested Rust source files and all thirteen health fixtures were read completely. The scoped files have no diff at this revision; the working tree has three unrelated untracked `artifacts/Localmotive_0.6.0_*` files.
- Scope size: `proc.rs` 817 lines, `log_sink.rs` 449, `health.rs` 1,944, `local_client.rs` 2,220, `server_service.rs` 561, `test_support.rs` 65, `property_tests.rs` 135; 6,191 Rust lines total. Fixtures are 77 lines total, including three empty stderr fixtures.
- Verdict: **not release-ready for process/health safety**. The process job is created before spawn but assigned after spawn, so kill-on-close is not atomic with child execution. Timeout/cancellation paths ignore failed cleanup and can then join pipe readers forever. The readiness probe has a separate reqwest client with no explicit redirect rejection. Several health and cleanup claims are weaker than their status text says.
- Finding counts: 0 Critical, 8 High, 13 Medium, 1 Low.
- Verification limitation: this was read-only as directed. No Cargo, app, network, or test execution was performed. Runtime behavior of the Windows job-assignment race and redirect path is therefore unexecuted; the source-level defects and missing guards are directly evidenced below.

### Per-file verdicts

- **FIX — `src-tauri/src/proc.rs` — 817 lines.** Largest production function: `output_with_timeout_and_cancel` (`395–492`). The post-spawn job-assignment race, discarded cleanup result, and unbounded reader joins are process-lifecycle defects.
- **FIX — `src-tauri/src/log_sink.rs` — 449 lines.** Largest function: `prune_log_directory` (`231–283`). Primary creation is collision-safe, but the second writer and failure-evidence write reopen/follow paths without equivalent no-link protection; prune errors are hidden.
- **SPLIT — `src-tauri/src/health.rs` — 1,944 lines.** Largest function: `run_managed_health` (`775–1351`). Seven stages, process lifecycle, temp-tree deletion, HTTP supervision, and evidence construction are one 577-line control function; several failure paths bypass cleanup evidence.
- **SHRINK — `src-tauri/src/local_client.rs` — 2,220 lines.** No function exceeds 150 lines; the largest production functions are `execute` (`630–740`) and `validate_transport_files` (`396–460`). The HTTP transport is correctly delegated to reqwest, but 233 lines (`132–364`) hand-walk certificate/key DER and the tests/fixture material dominate the file.
- **FIX — `src-tauri/src/server_service.rs` — 561 lines.** Largest production function: `start_server_worker` (`87–167`). One `?` after spawn leaves the published startup state uncleared; Stop also holds the server mutex during potentially 12 seconds of process/drain waiting.
- **KEEP — `src-tauri/src/test_support.rs` — 65 lines.** Small deterministic generator and campaign helper; no function exceeds 150 lines. Its callers still need unique filesystem ownership.
- **FIX — `src-tauri/src/property_tests.rs` — 135 lines.** No function exceeds 150 lines. The GGUF campaign uses a PID-only global-temp directory and has no cleanup guard for panic/interruption.
- **KEEP — `src-tauri/tests/fixtures/health/backend-ops.txt` — 1 line.** Static Vulkan success record; no functions.
- **KEEP — `src-tauri/tests/fixtures/health/benchmark.jsonl` — 1 line.** Static benchmark record; no functions.
- **KEEP — `src-tauri/tests/fixtures/health/completion.json` — 9 lines.** Static completion response; no functions.
- **KEEP — `src-tauri/tests/fixtures/health/cpu.list-devices.stderr.txt` — 0 lines.** Empty stderr fixture; no functions.
- **KEEP — `src-tauri/tests/fixtures/health/cpu.list-devices.stdout.txt` — 2 lines.** Canned `(none)` device output; no functions.
- **KEEP — `src-tauri/tests/fixtures/health/cuda.list-devices.stderr.txt` — 0 lines.** Empty stderr fixture; no functions.
- **KEEP — `src-tauri/tests/fixtures/health/cuda.list-devices.stdout.txt` — 2 lines.** Canned CUDA0/RTX 5090 output; no functions.
- **KEEP — `src-tauri/tests/fixtures/health/decision-records.json` — 46 lines.** Static CPU/CUDA/Vulkan decision records; no functions.
- **KEEP — `src-tauri/tests/fixtures/health/devices.txt` — 3 lines.** Static CPU/Vulkan device output; no functions.
- **KEEP — `src-tauri/tests/fixtures/health/ready.json` — 1 line.** Static readiness body; no functions.
- **KEEP — `src-tauri/tests/fixtures/health/smoke-model.json` — 10 lines.** Static model identity record; no functions.
- **KEEP — `src-tauri/tests/fixtures/health/vulkan.list-devices.stderr.txt` — 0 lines.** Empty stderr fixture; no functions.
- **KEEP — `src-tauri/tests/fixtures/health/vulkan.list-devices.stdout.txt` — 2 lines.** Canned Vulkan0/RTX 5090 output; no functions.

### Findings

#### Critical

_No Critical findings._

#### High

##### PROC-01 — Medium (lead; reviewer said High) — Job membership is assigned after the child is already running

> **Lead check: CORRECTED.** The window exists at proc.rs:293-306. llama-server does not start child processes at launch.

**Location:** `src-tauri/src/proc.rs:283–320`

**Evidence:**

> `proc.rs:288–293`: the job is created, then `wrapped.spawn()` is called.
>
> `proc.rs:299–305`: the code says “Assign immediately after spawn” and only then calls `job.assign(&handle)`.

**Why it matters:** `CREATE_SUSPENDED` is not requested. The child can execute and create a grandchild in the interval between `spawn()` and `AssignProcessToJobObject`. Kill-on-close is configured on the job, but the source does not establish that descendants created during this interval are in that job. The comment is an assertion of intent, not proof of atomic containment. A leaked `llama-server` descendant can keep serving after Stop or app exit.

**Concrete fix:** create the process suspended, assign its process handle to the already-configured job, verify assignment, then resume it. If the wrapper cannot provide that sequence, use a native suspended `CreateProcess` path or a maintained wrapper with an atomic/verified job-ownership contract. Add a real Windows integration test in which the child immediately creates a descendant and the test verifies descendant termination; do not use a source-text assertion.

##### PROC-02 — High — Failed process cleanup can turn a bounded operation into an unbounded pipe-reader join

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/proc.rs:35–50, 423–480`

**Evidence:**

> `proc.rs:39–50`: `read_bounded` keeps reading until EOF, even after the retained byte limit is exceeded.
>
> `proc.rs:443–445`: after timeout termination, the code joins `stdout_reader` without a join deadline.
>
> `proc.rs:456–457`: the status-error path does the same for both reader threads.

**Why it matters:** the retained buffer is bounded, but the reader thread's lifetime is not. If a surviving child or grandchild holds an inherited pipe handle open, `read_bounded` never sees EOF and `JoinHandle::join()` never returns. The documented 10-second termination deadline is then not a real upper bound for timeout, cancellation, or status-error paths. This defeats the health/runtime probe deadline and can hang a blocking worker indefinitely.

**Concrete fix:** make pipe ownership explicitly cancellable and closeable, or use bounded asynchronous/overlapped pipe reads whose handles can be closed after the cleanup deadline. Never unconditionally join a reader after reporting failed tree termination. Return an unresolved-cleanup error while retaining a tracked owner, and verify that no descendant still owns the pipe before declaring the operation finished.

##### PROC-03 — High — Timeout and cancellation return success-shaped errors even when termination failed

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/proc.rs:357–391, 427–471`

**Evidence:**

> `proc.rs:363–365`: the contract says `false` means callers must report unresolved cleanup.
>
> `proc.rs:430–437`: the cancellation branch calls `terminate_and_wait` and ignores its boolean result.
>
> `proc.rs:443–452`: the timeout branch does the same and returns `Timeout` regardless of cleanup outcome.

**Why it matters:** callers receive `Cancelled` or `Timeout` even if the process tree is still alive. `Drop` makes another best-effort kill, but its result is also discarded (`proc.rs:116–119`), so there is no durable ownership or evidence path for an unresolved process. The result can therefore release higher-level operation ownership while a runtime remains alive.

**Concrete fix:** capture cleanup status in every error branch and return a structured primary-plus-cleanup failure. Keep the process/job owner attached to an unresolved child until it is reaped, or fail closed at the operation coordinator. Make `Drop` a last resort, not the only cleanup report.

##### PROC-04 — High — Startup can leave the `starting` slot permanently occupied

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/server_service.rs:104–122`

**Evidence:**

> `server_service.rs:104–111`: the child is spawned and the startup state is already published by the caller.
>
> `server_service.rs:112–118`: `&local_client(&profile)?` is evaluated directly inside the health call.
>
> `server_service.rs:119–122`: only a health `Err` reaches `clear_starting`; a client-construction error exits through `?` first.

**Why it matters:** a transport-file race, TLS-client construction failure, or API-key read failure after spawn returns directly from `start_server_worker`. The child is dropped and gets only best-effort cleanup, but `starting` is not cleared. `server_status` can remain in `starting`, and later Start/Stop requests can be rejected as if an old startup still owns the slot.

**Concrete fix:** construct the client before spawning, or handle the client result in an explicit cleanup branch that terminates/reaps the child and calls `clear_starting`. Add a regression test that invalidates the transport file between launch validation and client construction and asserts the startup slot is cleared.

##### PROC-05 — High — Health readiness uses a redirect-following transport path

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/health.rs:1082–1155`; comparison: `src-tauri/src/local_client.rs:509–517`

**Evidence:**

> `health.rs:1082–1086`: the readiness client sets proxy and timeout options but no `redirect(Policy::none())`.
>
> `health.rs:1137–1153`: it accepts the resulting response body when the status is 200 and `status` is `ok`.
>
> `local_client.rs:512–517`: the centralized client explicitly disables redirects, but readiness does not use that policy.

**Why it matters:** a managed server can return a 3xx `Location`; the readiness client has no explicit redirect rejection. A redirected 200 body can satisfy the readiness JSON test while the subsequent listener check only proves that the original local port belongs to the child. This path can make health verification contact a non-loopback target and diverges from the safer centralized client.

**Concrete fix:** route readiness through `LocalHttpClient::plain`, or set `reqwest::redirect::Policy::none()` on the readiness builder and reject every 3xx. Add a test where the local server redirects to a second listener and assert that the second listener is never reached.

##### PROC-06 — High — Health cancellation abandons an untracked nested request worker

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/health.rs:659–713, 715–747`

**Evidence:**

> `health.rs:671–674`: `completion_request_supervised` spawns an outer thread and never retains or joins its handle.
>
> `health.rs:728–741`: `completion_request` creates a new client and passes `&AtomicBool::new(false)` to its cancellable request.
>
> `health.rs:676–681`: cancellation only invokes the supplied process-termination callback; it does not set the inner request's flag or drain its client worker.

**Why it matters:** the supervisor can return `Cancelled` while the outer thread and the reqwest worker continue. The inner cancellation flag is unreachable after dispatch, so the only interruption mechanism is whatever closing the server socket happens to do. A stalled connection can consume the full 120-second request budget, and repeated health runs have no shared worker counter to prevent overlap.

**Concrete fix:** construct one `LocalHttpClient` outside the supervisor, pass a live cancellation flag into the completion call, retain the worker handle, and wait for worker ownership to drain before releasing the health operation. If termination cannot close the request by the deadline, report unresolved ownership instead of returning a clean cancellation result.

##### PROC-07 — High — Transport-file reads do not enforce reparse/no-follow semantics at the read site

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/local_client.rs:854–876`; launch validation comparison: `src-tauri/src/lib.rs:722–739`

**Evidence:**

> `local_client.rs:861–867`: the code uses `File::open(path)` and only checks `metadata.is_file()`.
>
> `local_client.rs:870–875`: the same opened path is read with a bounded handle, but no symlink/reparse or opened-handle identity is checked.
>
> `lib.rs:722–739`: profile validation checks non-reparse files earlier, but the client later reopens the path.

**Why it matters:** validation and use are separate path resolutions. A reparse point or replacement between them can redirect the API-key, certificate, or private-key read. An unexpected API-key target can then be sent as a bearer header to the local server; an unexpected certificate can change the trust root. The bounded read limits size, not target identity.

**Concrete fix:** use one safe opened handle for validation and reading, or open with Windows no-follow/reparse-point protections and validate the handle's final identity. Re-run the same protection inside `LocalHttpClient::from_profile`; do not rely on an earlier pathname check.

##### PROC-08 — High — Secondary log handles and failure evidence follow a replaced path

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/log_sink.rs:100–111, 203–217`

**Evidence:**

> `log_sink.rs:103–111`: `second_writer` reopens `self.path` with `OpenOptions::append().open(...)` instead of cloning the already-validated file handle.
>
> `log_sink.rs:212–216`: `write_failure_evidence` derives a sibling path and calls `fs::write`, which follows an existing link and overwrites it.

**Why it matters:** `LogSink::create` protects only the initial log path with `create_new`. A local actor can replace the log or evidence pathname after creation. The second stream can append to an unexpected file, and failure evidence can overwrite up to 64 KiB of a planted target. This is exactly the path/reparse class the project rules prohibit.

**Concrete fix:** make `second_writer` use `self.file.try_clone()` so it never resolves the pathname again. Create failure evidence with a safe `create_new`/no-follow handle under a verified parent, refuse pre-existing links, and write through that handle. Add a planted-link test for the evidence path, not only the primary log path.

#### Medium

##### PROC-09 — Medium — Cleanup PASS treats every failed TCP connect as proof that the port is closed

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/health.rs:1323–1349`

**Evidence:**

> `health.rs:1325–1329`: `connect_timeout(...).is_err()` is assigned to `port_closed`.
>
> `health.rs:1331–1348`: any `port_closed == true` contributes to the cleanup PASS.

**Why it matters:** connection refusal, timeout, access denial, and other network errors all become “closed.” A timeout or local filtering error is not evidence that no listener remains. The final cleanup stage can therefore pass without proving the stated invariant.

**Concrete fix:** accept only a positively identified closed result such as `ConnectionRefused`, combine it with a listener-owner query showing no owner, and retry within a bounded cleanup deadline. Preserve an indeterminate/error result instead of converting it to PASS.

##### PROC-10 — Medium — The health contract allows PASS without stage-specific evidence and is enforced only in debug builds

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/health.rs:82–91, 189–213, 230–245`

**Evidence:**

> `health.rs:83–91`: `HealthStageResult::passed` creates PASS for any non-completion stage with only a caller-supplied string.
>
> `health.rs:200–213`: `validate_stage_contract` checks status/ordering and only requires structured evidence for deterministic completion.
>
> `health.rs:242–245`: `finish_run` calls the validator through `debug_assert!` only.

**Why it matters:** a future caller can manufacture PASS for device, backend, server, cancellation, or cleanup with no observed facts, and release builds will not reject an invalid result. Current `run_managed_health` generally performs checks before calling the constructor, but the public result/validator contract does not preserve that guarantee.

**Concrete fix:** make stage constructors private or stage-specific and carry typed observations for every PASS. Validate the complete result in release builds and fail closed if the evidence/order contract is invalid.

##### PROC-11 — Medium — CPU device enumeration passes without examining enumeration output

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/health.rs:450–460, 818–864`

**Evidence:**

> `health.rs:455–460`: the CPU branch returns `Ok(None)` based only on `adapter_name.is_none()` and never inspects `output`.
>
> `health.rs:818–824`: the command is run, but `health.rs:860–864` records PASS after the branch returns.

**Why it matters:** a CPU `--list-devices` process that exits zero with empty or unrelated output still receives the detail “identified the approved backend.” The canned CPU fixture is `(none)`, but the production parser does not require even that observation.

**Concrete fix:** require the approved CPU marker/format or an explicit runtime fact that CPU enumeration succeeded, and use CPU-specific detail that does not claim an exact device was identified.

##### PROC-12 — Medium — Completion failures are all mislabeled as timeouts

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/health.rs:715–747`

**Evidence:**

> `health.rs:728–740`: the request is issued through the bounded client with a 120-second budget.
>
> `health.rs:742–747`: every client error is discarded and converted to `HealthFailureReason::Timeout` with “request failed.”

**Why it matters:** output-limit, malformed transport, connection, TLS, and other failures become timeout evidence. That makes the health record materially less truthful and can send operators to the wrong recovery path.

**Concrete fix:** preserve the typed `LocalHttpClient`/`ProcessFailure` cause and map timeout, cancellation, output limit, malformed response, and transport failure separately. Do not erase the error string before the stage result is built.

##### PROC-13 — Medium — Failed health stages skip cleanup evidence and ignore termination outcomes

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/health.rs:1104–1123, 1156–1186, 1192–1205, 1236–1247`

**Evidence:**

> `health.rs:1106–1118`: cancellation calls `child.terminate_and_wait()` and immediately returns a failed readiness stage.
>
> `health.rs:1192–1205`: readiness timeout does the same before returning.
>
> `health.rs:1236–1247`: completion failure calls termination, then returns without recording the termination result.

**Why it matters:** `ProcessAndTemporaryFileCleanup` is only constructed after a successful completion/cancellation sequence. A failed run can return with later cleanup stages marked SKIPPED while a process, listener, or temp tree remains. `HealthTempDir::Drop` is also best effort (`health.rs:635–638`) and its result is absent from these reports.

**Concrete fix:** give every run a cleanup guard that executes on all exits, records process-tree, listener, and temp-tree results, and returns a failure if cleanup is unresolved. Do not let an earlier failure suppress cleanup evidence.

##### PROC-14 — Medium — Readiness timing is not a strict 120-second/cancellation bound

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/health.rs:1082–1105, 1137–1190`

**Evidence:**

> `health.rs:1082–1086`: one readiness request may block for two seconds.
>
> `health.rs:1104–1105`: the deadline is checked only at the top of the loop.
>
> `health.rs:1190`: every iteration adds a fixed 100 ms sleep.

**Why it matters:** a request already inside `send()` does not observe cancellation or the stage deadline until its two-second timeout completes; the sleep adds another delay. The stage can exceed the advertised 120-second bound and cancellation latency is dependent on the request timeout.

**Concrete fix:** calculate the remaining deadline for each request, use a short per-attempt timeout, and use a cancellable request path rather than a blocking raw client. Replace fixed polling sleeps with readiness signaling or a bounded backoff tied to the remaining budget.

##### PROC-15 — Medium — Security-sensitive certificate/key pairing uses a handwritten 233-line DER walker

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/local_client.rs:132–364`

**Evidence:**

> `local_client.rs:132–134`: the file introduces a custom DER frame parser and length walker.
>
> `local_client.rs:268–294`: it manually dispatches among PKCS#8, PKCS#1, and SEC1 forms.
>
> `local_client.rs:349–363`: it manually reconstructs EC SPKI material for equality comparison.

**Why it matters:** this is not handwritten HTTP/TLS, but it is handwritten ASN.1/X.509 key material processing on a trust boundary. It duplicates parser semantics maintained by crypto libraries and creates future algorithm/encoding edge cases. The existing tests cover selected RSA/EC encodings, not the full certificate/key grammar.

**Concrete fix:** use a maintained X.509/private-key parser for SPKI extraction or isolate the narrow pair-match operation behind a maintained parser API. If the custom path remains, document the exact supported grammar and add malformed/alternate-encoding property tests; do not expand it into a protocol stack.

##### PROC-16 — Medium — Benchmark and tuning callers detach log-drain handles

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/measurement_service.rs:458–459`; `src-tauri/src/tune_service.rs:40–41`; server join path `src-tauri/src/server_service.rs:258–269`

**Evidence:**

> `measurement_service.rs:458–459`: `spawn_server` returns log drains into `_drains` and the caller never joins them.
>
> `tune_service.rs:40–41`: tuning does the same.
>
> `server_service.rs:261–269`: Stop waits two seconds, then simply leaves any unfinished drain detached.

**Why it matters:** child termination and log-file completion are separate lifecycles. Detached drains can still hold file handles and append after a result is reported or after the next launch prunes the directory. A failed drain is discarded, so the retained evidence can be incomplete without an operation failure.

**Concrete fix:** return a shared drain owner with explicit bounded join semantics from every `spawn_server` caller. If a drain cannot finish after the child/pipe handles are closed, mark log evidence incomplete and retain/report unresolved ownership rather than silently dropping the handle.

##### PROC-17 — Medium — Tuning uses a fixed sleep instead of waiting for port release

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/tune_service.rs:94–100`

**Evidence:**

> `tune_service.rs:97`: cleanup status is captured.
>
> `tune_service.rs:98–100`: the code sleeps 600 ms and then proceeds to the next trial.

**Why it matters:** 600 ms is neither a proof that Windows released the listener nor a bound derived from observed state. Under load it can be too short and make the next trial fail on a still-busy port; on a fast host it adds needless delay. This is synchronization by hope.

**Concrete fix:** after confirmed tree termination, poll the exact configured endpoint/owner until it is available or a deadline expires. Do not launch the next trial on elapsed time alone.

##### PROC-18 — Medium — Stop holds the server mutex during process termination and drain waiting

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/server_service.rs:249–272`

**Evidence:**

> `server_service.rs:250–253`: the server mutex is acquired before cleanup.
>
> `server_service.rs:254–269`: termination and up to two seconds of drain polling run while `slot` remains held.

**Why it matters:** a 10-second `ContainedProcess` termination deadline plus drain waiting blocks status, validation snapshots, and competing lifecycle operations behind the same mutex. A UI status call can appear frozen while Stop is resolving a child.

**Concrete fix:** take the managed server out of the slot under the lock, release the lock, terminate/join outside it, then commit the empty state under the operation generation that initiated Stop. Preserve the reservation across that transition.

##### PROC-19 — Medium — Log-retention failures are swallowed, so quotas are advisory

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/log_sink.rs:228–235`; caller `src-tauri/src/lib.rs:780–782`

**Evidence:**

> `log_sink.rs:232–235`: any `read_dir` error is converted to `Ok(0)`.
>
> `lib.rs:780–782`: the launch path calls pruning and discards its result with `let _ =`.

**Why it matters:** permissions, I/O errors, or a replaced log directory silently disable pruning. The stated 64 MiB directory quota and retention counts then stop being enforced without an observable launch failure or diagnostic.

**Concrete fix:** distinguish “directory absent” from permission/I/O failure, propagate or record the latter in bounded launch evidence, and test that a denied/unreadable retention directory cannot be reported as a normal zero-removal result.

##### PROC-20 — Medium — Source-text tests self-match and do not test the safety property

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/proc.rs:606–613`; `src-tauri/src/health.rs:1551–1556, 1357–1399, 1481–1549`

**Evidence:**

> `proc.rs:608–612`: the Windows test searches the source for `"ProcessTree::assign"`.
>
> `health.rs:1552–1555`: the test searches the source for `"ProcessJob::assign"`.
>
> Those exact strings occur in the assertion lines themselves; the production implementation uses `job.assign` (`proc.rs:303–305`).

**Why it matters:** both count assertions can return one because the test source contains the searched literal, even when production never uses that API. The other `include_str!` checks inspect spelling/comments rather than observable containment, redirect, cleanup, or evidence behavior. This creates false-green safety coverage.

**Concrete fix:** delete source-pattern assertions and test behavior: spawn a child that immediately creates a descendant, assert job membership/termination, redirect readiness to a second listener, and assert cleanup state. Use a syntax-aware static check only for the raw-command policy, not as the process-safety test.

##### PROC-21 — Medium — Test synchronization and PID-only temp names are still flake-prone

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** `src-tauri/src/local_client.rs:1125–1260, 1331–1337, 1664–1968, 1882–1928, 2008–2203`; `src-tauri/src/proc.rs:646–816`; `src-tauri/src/health.rs:1743–1895`; `src-tauri/src/server_service.rs:458–539`; `src-tauri/src/property_tests.rs:9–35`

**Evidence:**

> `local_client.rs:1331–1337`: `temp_path` is only `temp_dir()/localmotive-local-client-{pid}-{name}`.
>
> `property_tests.rs:9–15`: the property campaign uses `temp_dir()/localmotive-s19-gguf-{pid}` and writes a shared `fuzz.gguf` repeatedly.
>
> The scoped tests use fixed sleeps for synchronization at `proc.rs:657,707,733,753,803,813`, `health.rs:1763,1827,1868`, `local_client.rs:1533,1577,1923,2032,2097,2145,2186`, and `server_service.rs:483,519`.

**Why it matters:** no scoped test binds a fixed numeric listener port—socket fixtures use port `0`—but PID-only paths can collide with stale artifacts after an interrupted run or PID reuse, and sleeps do not establish that a listener/descendant is ready or gone. Several fixture listener threads are intentionally spawned without a returned close/join handle, so a test process can accumulate live fixture threads.

**Concrete fix:** allocate every test directory with `create_dir`/`create_new` plus randomness and a cleanup guard; make each fixture return a shutdown signal and join handle; replace sleeps with readiness/exit channels and bounded polling. Keep load-aware bounds, but assert observed events rather than elapsed delay.

#### Low

##### PROC-22 — Low — Audit-ticket narration is embedded throughout production code

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Location:** examples include `src-tauri/src/proc.rs:126–132, 357–359`, `src-tauri/src/log_sink.rs:1–13`, `src-tauri/src/local_client.rs:419–423`, `src-tauri/src/health.rs:779–783`, `src-tauri/src/server_service.rs:26–32`, and `src-tauri/src/tune_service.rs:94–99`

**Evidence:**

> `log_sink.rs:1–7`: the module documentation carries `OPS-01`/`S-01` audit history and implementation narrative.
>
> `local_client.rs:419–423`: a historical follow-up review is embedded in the certificate-validation comment.
>
> `server_service.rs:29–32`: a review delegation identifier and historical failure shape are embedded in lifecycle code.

**Why it matters:** labels such as `OPS-01`, `G-05`, `R10`, `R15`, `RT-03`, `RT-04`, `MT-06`, and `S-27` are not executable evidence and will become stale as the ledger moves. They obscure the invariant and encourage source-text tests to treat narration as proof.

**Concrete fix:** retain short comments explaining the current invariant and why a non-obvious operation is necessary; move ticket mappings and historical incident narration to the project ledger/review document. Delete labels that do not drive code or test selection.

### Answers to specific questions

#### 1. Raw process-command construction

An exhaustive literal scan of every `*.rs` file under `src-tauri/src` found **no `Command::new`, `std::process::Command`, or `tokio::process::Command` site outside `src-tauri/src/proc.rs`**.

The only production constructions are:

| Location | Classification | Evidence |
|---|---|---|
| `src-tauri/src/proc.rs:25–32` | production, approved | `hidden_command` calls `Command::new(program)` and applies `CREATE_NO_WINDOW` on Windows. |
| `src-tauri/src/proc.rs:288–291` | production, approved but awkward | `spawn_contained` replaces the caller command with `Command::new("")` before wrapping the original command; this creates no process and is only a placeholder move. |

The remaining matches in `proc.rs:559,586,590,591,595` are test comments/string scans, not process construction. There are **zero outside-proc sites to report**, so the raw-command bypass question is clean at the source level. That does not make the source-text guard sufficient for containment; see PROC-20.

#### 2. Job containment, output bounds, and pipe drainage

- **Kill-on-close is configured, but not applied before child execution can begin.** `JobHandle::create` sets `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` at `proc.rs:149–175`; `wrapped.spawn()` occurs at `proc.rs:293`; `job.assign` occurs at `proc.rs:303–305`. There is no suspended-create/resume sequence. The source therefore has a real spawn-to-assignment race. Whether Windows retroactively captures every descendant created in that interval is **UNKNOWN here because execution was prohibited**; the code does not prove it.
- **Retained output is byte-bounded, not line-bounded.** `proc::read_bounded` retains at most `max_stream_bytes` per stdout/stderr stream (`proc.rs:35–50`) and returns `OutputLimit` after draining excess. Health subprocesses use 1 MiB per stream (`health.rs:14,324–330`); runtime probes use 2 MiB per stream (`core.rs:1723–1726`); the centralized HTTP response is 16 MiB (`local_client.rs:16–17,781–791`); request serialization is 4 MiB (`local_client.rs:16,647–665`); launch logs share an 8 MiB quota (`log_sink.rs:21–28,151–179`) and failure evidence is truncated to 64 KiB (`log_sink.rs:207–216`). There is no maximum line count.
- **Pipes are drained in the normal path.** `output_with_timeout_and_cancel` starts one reader thread per pipe (`proc.rs:423–424`), and `spawn_server` starts log drain threads for stdout/stderr (`lib.rs:811–838`). `LogWriter::drain` keeps consuming and discarding after quota (`log_sink.rs:151–179`). This prevents the ordinary full-pipe deadlock.
- **The normal drainage claim is not a bounded cleanup guarantee.** After failed termination, the process helper joins readers without a deadline (PROC-02). If a descendant holds a pipe, the child cannot deadlock on a full pipe, but the supervising Rust thread can deadlock waiting for EOF.

#### 3. HTTP/1.1, TLS, parsing, limits, and crate choice

- `local_client.rs` is **not a handwritten HTTP/1.1 client or TLS stack**. `Cargo.toml:36` already uses `reqwest 0.12` with `blocking`, `json`, `rustls-tls`, and compression features. The production wrapper is `local_client.rs:386–877`; request execution is delegated to `reqwest::blocking::Client` at `local_client.rs:509–527,754–806`. HTTP framing, headers, chunked decoding, and TLS record handling are library responsibilities. The chunked-response regression at `local_client.rs:1473–1495` exercises this path.
- The handwritten part is certificate/key material handling: a 233-line DER/PKCS#8/PKCS#1/SEC1 parser and SPKI reconstruction at `local_client.rs:132–364`. That is a security-sensitive ASN.1 helper, not an HTTP implementation; PROC-15 recommends shrinking it or replacing it with a maintained parser.
- **Header/chunked bug:** no direct header/chunked parsing bug was found in the centralized client. The chunked test passes by source inspection, but tests were not executed in this review. The separate readiness client is a policy bug because it does not explicitly disable redirects (PROC-05).
- **Read bounds:** request bodies are serialized through `BoundedVec` and response bodies are read through `Response::take(MAX_LOCAL_RESPONSE_BYTES + 1)` (`local_client.rs:641–665,771–791`). The `content_length` precheck is supplemented by a bounded read, so a lying length or chunked body is not unbounded in retained memory.
- **Timeouts:** the central client sets a five-second connect timeout and per-request `.timeout(budget)` (`local_client.rs:509–517,754–760`). Cancellable calls use a 500 ms polling slice, but the abandoned worker retains the whole request budget (`local_client.rs:18–25,677–737`). The health readiness path has a 250 ms connect/2 s request timeout (`health.rs:1082–1086`), but its stage deadline is not propagated into each call (PROC-14).
- **TLS verification:** no `danger_accept_invalid_certs` occurrence exists in the scoped source. `LocalHttpClient` keeps verification enabled and adds the profile certificate as an explicit root (`local_client.rs:518–524`). That is the justified policy for a loopback server with a user-selected self-signed certificate; disabling verification would be unsafe. The certificate/key pair is also structurally matched before launch, subject to the path race in PROC-07 and parser risk in PROC-15.
- **Replacement crate:** no replacement is needed; reqwest is already the established crate the owner prefers. `ureq` could provide a smaller synchronous client but would require revalidating the current rustls trust-root, redirect/no-proxy, body-limit, compression, and response semantics. `hyper` is lower-level and would push HTTP/TLS/body/cancellation mechanics into this code, increasing rather than reducing the handwritten surface. Switching crates would not fix the health readiness client unless the policy is centralized and tested.

#### 4. Health-stage honesty, timing, and cancellation

- **What is honest today:** production stages normally require actual observations: successful bounded process exits (`health.rs:317–344`), selected accelerator-device mapping (`health.rs:450–494`), benchmark records for both workloads (`health.rs:509–538,972–985`), a structured readiness response plus owner check (`health.rs:1137–1173`), exact completion content/token evidence (`health.rs:1250–1300`), and an observed child stop (`health.rs:1302–1321`). `finish_run` sets `passed` from every stage status (`health.rs:230–245`), and skipped stages prevent an overall pass.
- **What is not honest enough:** the generic PASS constructor/validator permits evidence-free PASS in release (PROC-10); CPU enumeration does not parse output (PROC-11); cleanup treats any connect error as closed (PROC-09); completion errors collapse to timeout (PROC-12); and failed paths skip cleanup evidence (PROC-13). A current happy path can still be real while the result contract remains too weak to defend future changes.
- **Timing assumptions:** the constants are 30 seconds for discovery, 120 seconds for backend/model operations, and 10 seconds for process cancellation (`health.rs:10–14`). The readiness loop sleeps 100 ms and uses a 2 s raw request timeout, so its 120 s stage bound is not strict (PROC-14). The completion supervisor polls every 25 ms (`health.rs:690–703`) and the inner local client uses 500 ms slices; these are bounded but layered.
- **Cancellation:** the completion supervisor checks cancellation before and after receiving a result (`health.rs:676–701`), so the response-versus-cancel race is biased toward cancellation. However, it does not own the nested request worker (PROC-06), and process termination results are ignored on several early exits (PROC-03/13). Cancellation is therefore responsive in the caller but not proven to resolve all owned work.

#### 5. Test hazards in scope

- **Fixed ports:** no scoped test binds a fixed numeric TCP port. Socket fixtures bind `127.0.0.1:0` or `(LOCALHOST, 0)` (`health.rs:995–996,1561–1562,1816–1817,1837–1838`; `local_client.rs:1127–1128,1161–1162,1196–1197,1265–1266,1590–1591,1887–1896`). The only fixed port is a URL-normalization expectation at `local_client.rs:1351`, not a listener.
- **Sleep synchronization:** process-tree tests use sleeps/poll loops at `proc.rs:657,707,733,753,803,813`; health cancellation/fixture tests at `health.rs:1763,1827,1868`; local-client cancellation/redirect/slow-body tests at `local_client.rs:1533,1577,1923,2032,2097,2145,2186`; server ownership tests at `server_service.rs:483,519`. These are listed in PROC-21. Production also uses polling sleeps at `health.rs:1190`, `local_client.rs:580`, `server_service.rs:247,264`, and a fixed tuning sleep at `tune_service.rs:99`.
- **Temp isolation:** `proc::unique_test_dir` uses random creation and `log_sink::scratch` includes `new_run_id`; `HealthTempDir` uses a random 128-bit suffix and `create_dir`. The weaker scoped paths are `local_client.rs:1331–1337` (`pid + name` only), `local_client.rs:1668,1820,1954` (PID-only directories), and `property_tests.rs:10–12` (PID-only directory). They are distinct by label in the current suite, so no deliberate same-directory sharing was found, but stale directories and PID reuse are not excluded.
- **Source assertions/comments:** `proc.rs:608–613`, `health.rs:1357–1399`, `health.rs:1481–1549`, and `health.rs:1551–1556` inspect source strings/comments. The `ProcessTree::assign` and `ProcessJob::assign` needles are present in their own assertion strings, making those count checks self-matching. These tests do not prove runtime behavior.
- **Fixture scope:** the health fixtures are canned parser inputs. CUDA/Vulkan fixtures name an RTX 5090, while the CPU fixture says `(none)`; they do not qualify a packaged runtime or current machine. The ignored live qualification test at `health.rs:1568–1619` is the only scoped live-hardware path and was not run.

#### 6. Dead code, duplicated logic, audit narration, and shrink plan

- **Dead code:** a literal search found no `#[allow(dead_code)]` in the seven scoped source files. Compiler-confirmed unused-item status is **UNKNOWN** because Cargo/Clippy were prohibited. The only directly related suppression found while checking callers is outside scope at `lib.rs:71–72` for `runtime_lease`; it documents a drop-only ownership field rather than proving it is dead.
- **Duplicated logic:** health readiness constructs a second raw reqwest client (`health.rs:1082–1086`) while `local_client.rs` owns the central redirect/no-proxy/body policy. Process/log cleanup is also repeated across `proc.rs`, `server_service.rs`, `measurement_service.rs`, and `tune_service.rs`, with different join/deadline behavior. These copies are the reason redirect and drain semantics diverge.
- **Longest function:** `health::run_managed_health` spans `775–1351` and owns all seven stages, process startup, HTTP supervision, and cleanup. Split it into stage functions returning typed evidence plus one cleanup guard. Keep stage ordering in one small coordinator.
- **How to shrink:** centralize readiness on `LocalHttpClient`; replace/reduce the custom DER walker; centralize contained-child cleanup and drain ownership; replace source-pattern safety tests with executable integration tests; remove historical ticket prose from production comments.
- **Audit-ticket narration:** scoped code contains labels including `OPS-01`, `S-01`, `G-05`, `MT-06`, `RT-03`, `RT-04`, `R10`, `R15`, `R16`, and `S-27`. They are useful in a review ledger but should not be the primary explanation or test oracle. PROC-22 covers the maintenance cost.

### Cross-cutting observations

- The raw-command policy is centralized correctly at the source level: no out-of-scope `Command::new` bypass was found. The more serious process defect is not construction bypass; it is the non-atomic post-spawn job assignment and the cleanup/reporting behavior after termination fails.
- The project has several real bounds, but they are different kinds of bounds: retained bytes are capped, while drain lifetime, child spawn latency, and some blocking joins are not. The code often describes these as one “bounded” operation. Those dimensions must be reported separately.
- The centralized client is the right architectural direction. Do not replace it with handwritten HTTP/TLS. Remove the duplicate health readiness transport and make every local request share redirect, proxy, TLS, body, and cancellation policy.
- Cleanup evidence is path-dependent: successful health runs reach `ProcessAndTemporaryFileCleanup`, but earlier failures return with that stage skipped. For a process-control product, cleanup must be an unconditional finally-style phase, not an optional tail after all functional stages pass.
- No `taskkill`, `/IM`, `Get-Process`, `netstat`, or name/port kill-by-process logic was found in the source scan. The Windows listener-owner checks are identity checks, not unrelated-process termination.
- No tests or runtime gates were executed in this review. The report therefore does not claim that the current Windows build passes or fails; it identifies source defects and test gaps that the prohibited execution would need to verify after repair.

---

## Part 7 — Rust catalog, downloads, file formats and cloud (finding prefix DL)

### Summary

Scope reviewed completely: `download.rs`, `catalog.rs`, `catalog_db.rs`, `catalog_service.rs`, `artifact.rs`, `gguf.rs`, `cloud.rs`, and `catalog/README.md`. `lib.rs`, `runtime.rs`, and `Cargo.toml` were read only where they establish the download/catalog/cloud boundary, retry behavior, or dependencies. No repository file was modified. No Cargo, npm, Tauri, or network command was run, as required.

The downloader has the right high-level integrity order: it probes the remote size/identity, validates range responses, resumes only when the saved URL/size/digest/validators still match, streams SHA-256 over the completed `.part`, and publishes with a no-replace hard link. The Tauri download boundary canonicalizes the selected model folder, rejects symlink/reparse ancestors, validates the catalog filename, and the capability directory is checked against the opened handle.

The review still finds four High issues and twelve Medium issues. The most consequential are:

- A mutable SQLite row marked as a user override is explicitly accepted as download authority, despite the database module saying SQLite never authorizes downloads. The read path trusts the flags and stored digest without re-running override validation.
- A file-controlled GGUF `split.no` can overflow `no + 1` and panic in checked builds.
- Artifact safety checks are path-based and then reopen the path, leaving a reparse-point/symlink replacement race.
- OpenRouter key-exchange responses are read with unbounded `Response::text()`, unlike the bounded chat/model-list readers.
- The GGUF parser has strong individual limits, but permits a one-million-pair container and does not cap the retained pair/fact container overhead.
- `download.rs` and `catalog.rs` are 4,064 and 3,902 lines respectively, with 255-line and 260-line production functions and large test modules. Their audit-ticket comments are numerous enough to obscure the actual security boundaries.

No Critical finding was established from the source-only review. The release/build/test state is UNKNOWN because the task explicitly prohibited running the gates.

### Per-file verdicts

- `src-tauri/src/download.rs` — **SPLIT** — 4,064 lines. Production functions over 150 lines: `download_file` 1154–1408 (255), `fetch_chunk` 1539–1744 (206); the largest test is `dc12_checkpoint_publication_is_ordered_after_a_data_sync` 3308–4064 (757). Separate planning/resume state, filesystem/capability operations, HTTP policy/probe, transfer orchestration, and tests.
- `src-tauri/src/catalog.rs` — **SPLIT** — 3,902 lines. `fetch_catalog_verified` 1223–1482 (260); `dc01_local_load_uses_bundled_data_for_missing_corrupt_or_untrusted_caches` 3596–3902 (307). Separate catalog types/validation, signature/cache refresh, and Hugging Face credential storage; move the 1800–3902 test module beside those units.
- `src-tauri/src/catalog_db.rs` — **FIX** — 1,363 lines. No function exceeds 150 lines; largest production functions are `read_catalog_db_models` 273–406 (134), `validate_user_override` 480–588 (109), and `insert_model_with_ownership_guard` 166–267 (102). Fix the read-time trust and allocation bounds without adding another authority layer.
- `src-tauri/src/catalog_service.rs` — **KEEP** — 282 lines. Orchestration is small and cohesive; the cache-root fallback and poisoned-lock behavior still need fixes at their existing boundaries.
- `src-tauri/src/artifact.rs` — **FIX** — 825 lines. No function exceeds 150 lines; `inspect_artifact` is 416–516 (101). Fix checked arithmetic and handle-based file opening/race resistance.
- `src-tauri/src/gguf.rs` — **FIX** — 1,078 lines. No function exceeds 150 lines; `parse_inner` is 501–640 (140). Keep the parser design, but cap container/output overhead and use a secure opened-file boundary.
- `src-tauri/src/cloud.rs` — **SPLIT** — 1,604 lines. No function exceeds 150 lines; `wait_for_code` is 487–589 (103), and `s21_contract_fixtures_cover_every_provider_and_case` is 1322–1423 (102). Split Credential Manager, OAuth callback, and provider transport; fix the unbounded exchange path and redirect policy in place.
- `catalog/README.md` — **KEEP** — 145 lines. It correctly states that the detached signature protects the JSON and that model bytes are separately verified, but it does not document the CRLF normalization or the user-override exception now present in Rust.

### Findings

#### High

##### DL-01 — High — `src-tauri/src/lib.rs:2010-2044`; `src-tauri/src/catalog_db.rs:727-777` — Mutable SQLite override rows are accepted as download authority without read-time revalidation

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Evidence**

> `catalog_db.rs:727-731` — “Resolve one user-owned file row for download authorization ... the exact stored SHA-256 and size are the authorization.”
> `lib.rs:2036-2040` — `Some(file) => Ok(AuthorizedDownload { file, authority: "user" })`.
> `catalog_db.rs:560-564` — override validation checks revision length only; it is not called by `user_override_file`.

**Why it matters**

The generic mirror is documented as non-authoritative (`catalog_db.rs:1-7`), but `resolve_catalog_download` explicitly falls through to the same SQLite file and accepts any row with both `user_sourced` flags set. A normal save validates the row first, but a damaged or locally edited database can set those flags and supply a different Hugging Face repo/revision/filename/digest. The destination host remains the fixed Hugging Face host, but the mutable database can still authorize a different remote object and expected SHA-256. This is either a trust-boundary violation or an undocumented intentional exception; the code and module contract currently say both.

**Concrete fix**

Choose and document one rule. If user overrides are not authority, remove the `user_override_file` fallback and resolve downloads only from the current signature-verified/bundled catalog. If user overrides are intended authority, put them behind an explicit authority type/store, re-run the complete validator on every read, reject malformed/tampered rows, validate revision syntax, and never treat a generic mirror row as sufficient.

##### DL-02 — Medium (lead; reviewer said High) — `src-tauri/src/artifact.rs:54-74`; `src-tauri/src/gguf.rs:591-595` — Hostile GGUF split metadata can overflow and panic

> **Lead check: CORRECTED.** The release profile does not set overflow-checks. Release builds wrap and report a false mismatch. Only debug builds panic.

**Evidence**

> `artifact.rs:63-70` — `let recorded = (no + 1, count);`
> `gguf.rs:591-595` — `summary.split_no = general("split.no").and_then(Value::as_u64);` and `split_count` are read directly from file metadata.

**Why it matters**

A local GGUF can encode `split.no = u64::MAX`. In a checked/debug build, `no + 1` panics instead of returning a file error. In an unchecked release build it wraps and produces an incorrect split verdict. This violates the project rule that hostile file input must not panic and can make artifact inspection crash rather than report an invalid shard set.

**Concrete fix**

Use `checked_add(1)`. Return a structured invalid/mismatch result when the zero-based metadata cannot be represented as a one-based index, and add a regression fixture with `u64::MAX`.

##### DL-03 — High — `src-tauri/src/artifact.rs:277-302`; `src-tauri/src/gguf.rs:649-657` — Reparse/symlink validation is separated from the file open

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Evidence**

> `artifact.rs:277-287` — `symlink_metadata` and reparse checks run before opening the regular file.
> `artifact.rs:301-302` — `File::open(path)` re-resolves the path after those checks.
> `gguf.rs:653-657` — `read_summary_cancellable` validates the path, then opens it by path.

**Why it matters**

A concurrent process can replace a checked file or ancestor with a symlink/junction/reparse point between `symlink_metadata` and `File::open`. The parser and hasher then follow the replacement. The current checks are useful against static links but do not prove that the opened handle is the file that was checked. The project explicitly says lexical/path checks do not prove filesystem containment.

**Concrete fix**

Open through a capability directory with no-follow/reparse-safe semantics, then inspect the opened handle and parse/hash that handle. On Windows, compare stable file identity to the preflight identity where preflight is required; do not re-open a path after validation.

##### DL-04 — High — `src-tauri/src/cloud.rs:596-616` — OAuth key exchange buffers an unbounded response

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Evidence**

> `cloud.rs:596-607` — fixed OpenRouter POST followed by `let text = response.text()`.
> `cloud.rs:687-710` — bounded response reading exists, but is used only by the model/chat paths.

**Why it matters**

A malformed or compromised OpenRouter response can cause an arbitrary body allocation before JSON parsing. The normal chat/model paths cap bodies at 2 MiB/512 KiB, but the credential-establishing path bypasses that control. A large response can hang or exhaust memory before any key is stored.

**Concrete fix**

Read the exchange response through a small dedicated cap before UTF-8/JSON parsing, validate that the returned key is non-empty and below the cloud credential maximum, and route failures through the same bounded-body helper. Add an oversized exchange fixture.

#### Medium

##### DL-05 — Medium — `src-tauri/src/gguf.rs:16-34`, `535-605` — GGUF parser bounds individual values but not container/output overhead tightly enough

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Evidence**

> `gguf.rs:23-34` — `MAX_KV_COUNT = 1_000_000`, retained metadata 2 MiB, work 64 MiB.
> `gguf.rs:535-554` — a one-million-entry `pairs` vector is built after only the count check.
> `gguf.rs:596-605` — relevant values are cloned again into `metadata_facts` without a fact-count cap.

**Why it matters**

The parser does not have an unbounded individual string/array allocation: the important limits are real and listed in the Answers section. However, a hostile header can still drive up to one million `(String, Value)` entries, allocations, tuple/container overhead, and repeated lookup work. The 2 MiB retained-byte budget counts selected input strings, not all `Vec`/`Value`/`MetadataFact` container overhead or the second set of cloned strings. This is a bounded but unnecessarily large local-file DoS surface.

**Concrete fix**

Set a materially smaller production KV/tensor budget, charge container entries and cloned output bytes, cap retained `metadata_facts`, and reject duplicate/overly numerous relevant keys before producing an IPC summary. Keep the tiny-limit test seam for boundary tests.

##### DL-06 — Medium — `src-tauri/src/catalog_db.rs:307-373`, `396-405` — SQLite row bound is enforced after materializing rows and all file children

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Evidence**

> `catalog_db.rs:307-308` — `let mut models = Vec::new(); for row in rows {`.
> `catalog_db.rs:330-373` — every model's file rows are fully collected before the model is stored.
> `catalog_db.rs:396-404` — only after that work does the code check `models.len() > MAX_CATALOG_MIRROR_ROWS`.

**Why it matters**

A corrupt local SQLite file can contain thousands of models and arbitrary-sized text/blob values. The nominal 5,000-model check does not bound allocations before the check, and there is no per-model file count or aggregate byte limit on the read path. This can stall catalog load or create a large IPC response before recovery is selected.

**Concrete fix**

Use SQL `LIMIT MAX_CATALOG_MIRROR_ROWS + 1`, bound every text column while reading, cap files per model and aggregate file rows/serialized bytes, and reject before cloning or returning the result. Apply the same limits to the override salvage path.

##### DL-07 — Medium — `src-tauri/src/catalog_db.rs:534-564`; `src-tauri/src/download.rs:394-400` — User override revisions are length-checked but not path-safe

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Evidence**

> `catalog.rs:560-568` rejects empty revisions and `..` segments for signed rows.
> `catalog_db.rs:560-564` checks only `file.revision.len()` for overrides.
> `download.rs:394-400` feeds `revision.split('/')` directly into URL path segments.

**Why it matters**

An override can save a revision containing `..`, empty path segments, or other values rejected by the signed catalog validator. The resolver then turns that string into multiple URL path segments. The host remains Hugging Face, but the request is no longer constrained to the documented revision grammar and can target an unintended endpoint/path.

**Concrete fix**

Reuse the same `is_safe_revision` rule in `validate_user_override`, validate the revision at the Tauri download command as well as at persistence, and add traversal/dot-segment tests for override rows.

##### DL-08 — Medium — `src-tauri/src/catalog.rs:603-616`; `catalog/README.md:79-82` — Signature verification is not over the exact fetched bytes

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Evidence**

> `catalog.rs:603-610` — `verify_catalog_signature` calls `normalize_crlf(body)` before verification.
> `catalog.rs:611-616` — Ed25519 verifies the normalized bytes.
> `catalog/README.md:79-82` — the contract says the signature covers the “exact bytes.”

**Why it matters**

A CRLF/LF-only mutation is accepted even though the detached signature was not made over the fetched byte sequence. JSON semantics usually remain equivalent for line-ending whitespace, so this is not currently an obvious model-substitution bypass, but it is a cryptographic protocol mismatch and makes the README’s exact-byte claim false. Future signed data or string handling could make the normalization security-relevant.

**Concrete fix**

Verify the raw response bytes and keep the repository artifact’s line endings stable. If normalization is a deliberate compatibility requirement, sign and verify a formally specified canonical representation and change the README/workflow contract; do not silently normalize only in the client.

##### DL-09 — Medium — `src-tauri/src/catalog.rs:20-23`, `603-616` — There is no production key-rotation protocol

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Evidence**

> `catalog.rs:20-23` contains one compiled 32-byte `CATALOG_VERIFYING_KEY`.
> `catalog.rs:603-616` tries an isolated verifier key, then falls back to that single shipped key; no key id or signed keyset is present.

**Why it matters**

If the maintainer key is lost or compromised, rotation requires shipping a new application binary. There is no overlapping-key period, revocation, key identifier, or signed transition. That is an operational release blocker during key compromise, not a current signature bypass.

**Concrete fix**

Define a pinned root/key-transition format with key IDs, validity/overlap rules, and explicit revocation. Keep the compiled root as the trust anchor and require a signed transition; do not make arbitrary runtime keys authoritative.

##### DL-10 — Medium — `src-tauri/src/cloud.rs:623-645`, `901-916` — Cloud clients do not install an explicit redirect policy

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Evidence**

> `cloud.rs:623-629` builds the provider client with timeout only; no `.redirect(...)` policy is set.
> `cloud.rs:901-916` sends the bearer-authenticated request to `{base_url}/chat/completions`.

**Why it matters**

The direct provider base URLs are fixed in `PROVIDERS` (`cloud.rs:41-107`), so a caller cannot directly supply an arbitrary host. The default reqwest redirect behavior is nevertheless server-controlled, and this module does not prove that every redirect is rejected or that sensitive headers are never forwarded on every supported reqwest path. At minimum, prompts and response traffic can be redirected to an unintended host. Whether a cross-host redirect can forward the bearer key is UNKNOWN from this source review and should not be left to an implicit dependency behavior.

**Concrete fix**

Use `redirect(reqwest::redirect::Policy::none())` for provider API calls, or install an explicit exact-host allowlist and test same-host and cross-host redirects. Keep the base URL private to the fixed provider table.

##### DL-11 — Medium — `src-tauri/src/cloud.rs:223-237`, `848-879` — Cloud credential and request inputs have no byte bounds

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Evidence**

> `cloud.rs:223-237` trims/rejects whitespace in a key but has no maximum length.
> `cloud.rs:848-868` accepts `model`, `system`, and `user` strings without size validation.

**Why it matters**

The frontend is an untrusted IPC caller. A compromised webview can submit a very large key to Credential Manager or very large prompt/model fields to the provider. Response bodies are capped, but request memory, serialization, and outbound body sizes are not. The cloud path therefore has weaker input bounds than the catalog/HF-token path.

**Concrete fix**

Add explicit maximum byte lengths before keyring writes and before JSON request construction. Enforce them in the Tauri command and in `save_credential`/`chat_with_deadline`, not only in UI controls.

##### DL-12 — Medium — `src-tauri/src/runtime.rs:2545-2556`; `src-tauri/src/download.rs:925-945`; `src-tauri/src/cloud.rs:716-730` — Retry-After policy is duplicated and inconsistent

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Evidence**

> `runtime.rs:2545-2555` parses a numeric header into an unrestricted `u64`.
> `download.rs:925-945` clamps values to 1–30 seconds and parses a handwritten date.
> `cloud.rs:716-730` separately clamps to `MAX_RETRY_AFTER_SECS`.

**Why it matters**

The downloader retries up to three attempts, cloud paths perform at most one 429 retry, and the runtime catalog returns an unbounded header value to its IPC error. The same HTTP concept has three policies, one of which can expose an absurd delay. This is a maintenance and user-visible behavior risk, especially during a release or provider outage.

**Concrete fix**

Create one bounded Retry-After helper using a maintained HTTP-date parser, with a named maximum and one documented retry policy per operation. Clamp before storing or serializing runtime error data.

##### DL-13 — Medium — `src-tauri/src/download.rs:948-1005`; `src-tauri/src/cloud.rs:716-729`; `src-tauri/Cargo.toml:36` — Handwritten HTTP-date parsing should be replaced

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Evidence**

> `download.rs:948-995` implements a custom IMF-fixdate parser and `998-1005` implements its civil-calendar arithmetic.
> `cloud.rs:716-729` reuses that custom parser for provider retry dates.
> `Cargo.toml:36` has reqwest but no `httpdate` dependency.

**Why it matters**

The parser checks broad ranges but does not validate that the weekday matches the date or that day/month is a real calendar date. Its behavior is duplicated across download/cloud policy rather than delegated to a maintained HTTP-date crate. The impact is bounded retry timing, not direct data corruption, but invalid server headers can be interpreted inconsistently.

**Concrete fix**

Use the maintained `httpdate` crate for HTTP-date parsing, retain the existing 1–30 second bound, and keep one shared helper. Add tests for invalid calendar dates, weekday mismatches, past dates, and leap-second behavior according to the crate contract.

##### DL-14 — Medium — `src-tauri/src/download.rs:1-4064`; `src-tauri/src/catalog.rs:1-3902` — Security-critical workflows are oversized and audit narration is mixed into implementation

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Evidence**

> `download.rs:1154-1165` starts a 255-line `download_file` state machine; `fetch_chunk` spans 1539–1744.
> `catalog.rs:1223-1227` starts a 260-line `fetch_catalog_verified` state machine.
> The seven reviewed Rust files contain at least 142 lines with S/DC/RT/CLD/GH/PR audit markers, including test-only incident narration.

**Why it matters**

The code is difficult to review as one unit: transport, filesystem authority, resume state, publication, signature/cache policy, credential migration, and their long fixtures are interleaved. Long functions and labels such as `S-10`, `DC-08`, `DC-12`, `RT-04`, and `PR #18` make comments look like proof while the real invariant is spread across several functions. This directly increases the chance that release failures or a security regression survive review.

**Concrete fix**

Split download into `plan/resume`, `paths/capability`, `http/policy`, and `transfer/publication` modules, with focused integration tests. Split catalog into `types/validation`, `signature/cache`, and `hf_credentials`. Keep short invariant comments in code; move ticket IDs, run numbers, and historical incident narration to the audit ledger.

##### DL-15 — Medium — `src-tauri/src/catalog_service.rs:10-21`; `src-tauri/src/catalog_db.rs:48-55` — Cache-root fallback and SQLite open do not enforce the same reparse boundary as downloads

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Evidence**

> `catalog_service.rs:16-21` accepts the configured cache root, then falls back to `app_cache_dir()` or `temp_dir()/localmotive`.
> `catalog_db.rs:51-55` creates the root and calls `Connection::open(catalog_db_path(root))` without a reparse/handle identity check.

**Why it matters**

The model-download target has explicit canonical/reparse protections, but the catalog cache and override database do not. A junction/symlink or replacement race can redirect cache/SQLite reads and writes outside the intended app-data directory. Because user override rows can currently participate in download authorization (DL-01), this is more than a cosmetic cache concern.

**Concrete fix**

Resolve and validate the app-data root once, reject symlink/reparse ancestors, open it through a capability directory, and open the SQLite/cache files relative to that handle. Do not silently use a shared temporary fallback without a safe-root check; surface the cache-path failure instead.

##### DL-16 — Medium — `src-tauri/src/download.rs:431-435`, `1520-1535`, `1591-1652` — Weak ETags are treated as resumable identity

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

**Evidence**

> `download.rs:431-434` strips quotes and stores any non-empty ETag, including `W/...`.
> `download.rs:1591-1595` sends the stored value as `If-Match`.
> `catalog.rs:529-535` shows the stricter strong-ETag policy that the downloader does not reuse.

**Why it matters**

A weak ETag is not a strong byte identity and is not appropriate for `If-Match`/byte-range resume proof. The downloader can persist a weak validator, send a malformed/ineffective quoted form, and fail every transfer against a server that exposes one. The final SHA-256 still protects publication, but resumability becomes brittle and the identity policy differs between catalog and model HTTP.

**Concrete fix**

Preserve the raw validator only when it is a syntactically valid strong ETag; otherwise discard it and rely on URL/size/digest plus a valid Last-Modified signal. Build `If-Match` from the original quoted validator rather than manually re-quoting a stripped value.

### Answers to specific questions

#### 1. Download range, 200 behavior, remote changes, digest/publication, paths, and filename

- **Content-Range is parsed and checked against the requested offset.** The probe requires `bytes 0-0/<total>` and uses the total from `Content-Range` (`download.rs:1114-1138`). Every 206 transfer checks `content_range_matches(v, request_start, request_end, total)` (`download.rs:1623-1635`), and the parser requires exact start, end, and total (`download.rs:1502-1518`). A response body cannot write past the requested range: the loop bounds `remaining` and rejects an oversized read (`download.rs:1677-1681`).
- **A 200 to a Range request is accepted only for a complete single whole-file transfer starting at offset zero.** `range_response_is_usable` accepts 206, or 200 only when `whole_file && requested_start == 0` (`download.rs:1493-1500`). A 200 while resuming at a non-zero cursor or while doing parallel ranges is rejected (`download.rs:1615-1621`). A server proven to ignore ranges is deliberately retried as one sequential whole-file chunk (`download.rs:1562-1587`).
- **Resume state is bound to identity.** The sidecar carries URL, size, expected SHA-256, ETag, Last-Modified, and chunk geometry (`download.rs:211-222`). Reuse requires exact URL/digest/Last-Modified plus size and ETag-presence/equality (`download.rs:277-290`), and invalid state/geometry/size removes the part and sidecar (`download.rs:1224-1255`). Each transfer also sends `If-Match`/`If-Unmodified-Since` and rejects mismatched response validators (`download.rs:1589-1652`). If the server changes and exposes no usable validator, the final digest still rejects the bytes before publication. Weak ETags are the exception documented in DL-16.
- **The final digest is computed before atomic/no-replace publication.** The completed `.part` is opened and streamed through `sha256_reader_with`; a mismatch deletes part and sidecar (`download.rs:1374-1400`). Only after the digest succeeds does `publish_verified_part` hard-link the part under the final name and re-check the opened-file identity (`download.rs:1402-1457`). It does not replace an existing target; conflicts retain both files (`download.rs:1420-1439`). This is a hard-link publication rather than a same-filesystem rename, but it is atomic with respect to the name and no-replace.
- **The Tauri destination boundary is canonicalized and rejects reparse ancestors.** `lib.rs:1945-1980` inspects every ancestor and the selected root for symlink/reparse attributes, canonicalizes the root, and joins only the validated filename. `catalog.rs:639-686` rejects separators, drive syntax, NUL/control/trailing-space/trailing-dot names, Windows device names, and non-GGUF names; the effective Windows filename limit is 255 bytes (`catalog.rs:641-673`). `download.rs:539-566` opens the canonical parent as a capability directory and compares the opened handle to the captured path before network or mutation. Direct internal callers still rely on their own root validation; `download_file` itself is not an API that accepts a root policy.
- **Filename sanitization is applied to catalog/HF metadata and to user overrides on save.** Signed catalog parsing calls `is_safe_filename` (`catalog.rs:454-464`); the Tauri command repeats repo/filename validation (`lib.rs:2072-2075`); override persistence calls `validate_download_target` (`catalog_db.rs:534-537`). The missing defense is override revision syntax (DL-07), not the filename grammar.

#### 2. Catalog signature bytes, key, rotation, failure behavior, and SQLite/override authority

- **Bytes:** the verifier base64-decodes a trimmed signature and requires exactly 64 bytes, then calls Ed25519 `verify_strict` (`catalog.rs:570-584`). The body passed to that verifier is `normalize_crlf(body)`, not the raw fetched bytes (`catalog.rs:603-615`). This contradicts the README’s exact-byte contract (`catalog/README.md:79-82`); see DL-08.
- **Key:** normal production verification uses the compiled `CATALOG_VERIFYING_KEY` at `catalog.rs:20-23`. In an isolated verifier profile, a key supplied through `LOCALMOTIVE_CATALOG_PUBKEY` is tried first, then the shipped key is tried (`catalog.rs:603-616`). The environment source is only accepted under `LOCALMOTIVE_VERIFY_ISOLATED_ROOT` and localhost/catalog-root checks (`catalog.rs:42-68`), but the production trust model has one embedded key and no rotation protocol (DL-09).
- **Signature failure is fail-closed for network authority.** A bad candidate calls `fallback` with the last verified cached body (`catalog.rs:1394-1400`). The cache is loaded only after its body/signature pair verifies (`catalog.rs:1228-1231`), and fallback uses the bundled catalog if no validated cache remains (`catalog.rs:1497-1529`). Invalid/malformed/unsupported candidates do not replace the cache (`catalog.rs:1402-1415`).
- **The SQLite mirror does not authorize ordinary curated rows.** `catalog_service.rs:102-109` explicitly says the mirror is for browse/filter and that downloads resolve against the signed snapshot. `lib.rs:2025-2029` first looks up the active signed/bundled catalog.
- **User overrides do authorize in the current implementation.** If the signed lookup misses, `lib.rs:2031-2044` calls `catalog_db::user_override_file`; that query requires both model and file `user_sourced = 1` and returns the stored size/digest (`catalog_db.rs:738-777`). That is a deliberate local-authority route in code, but it conflicts with the module-level “SQLite ... never authorizes a download” statement and is not revalidated on read (DL-01).

#### 3. GGUF bounds and hostile-file behavior

The parser has meaningful bounds:

- Header bytes: **256 MiB** (`MAX_HEADER_BYTES`, `gguf.rs:14-16`).
- Array elements: **16,777,216** (`MAX_ARRAY_ELEMENTS`, `gguf.rs:17`); captured array values: **4,096** (`gguf.rs:18`); nesting depth: **4** (`gguf.rs:19`).
- Tensor count: **1,000,000**; recorded tensor descriptors: **4,096**; dimensions per tensor: **8** (`gguf.rs:20-22`).
- KV count: **1,000,000**; key bytes: **16 KiB**; one retained string: **64 KiB**; aggregate retained metadata: **2 MiB**; parser work: **64 MiB** (`gguf.rs:23-34`).
- Declared strings are checked before allocation (`gguf.rs:328-343`); irrelevant fixed-width arrays are skipped by checked byte count, variable arrays are discarded through bounded readers, and arrays are capped/depth-checked (`gguf.rs:380-455`); tensor descriptor dimensions and count are checked (`gguf.rs:607-637`).

Therefore a short hostile file cannot cause an unbounded string/array allocation through the ordinary parser path, and the truncation tests exercise clean errors (`gguf.rs:834-915`, `958-977`). It can still cause a large bounded container allocation through one million pairs and cloned facts (DL-05). Separately, the artifact split metadata arithmetic can panic on `u64::MAX` (DL-02), and path validation/opening is raceable (DL-03). Those are the remaining hostile-file/path concerns; do not describe the parser as “panic-proof” until the overflow is fixed.

#### 4. `cloud.rs`: key storage/leak paths, provider URL control, and retries

- **Storage:** interactive cloud keys are read through `SecretStore::get`/`KeyringStore` under service `Localmotive` (`cloud.rs:22-25`, `130-153`), then `require_secret` obtains the key only for a request (`cloud.rs:249-257`). Hugging Face uses the separate `Localmotive HF` service (`catalog.rs:1596-1603`).
- **IPC/log/error exposure:** normal credential status returns only `configured` and a masked suffix (`cloud.rs:184-220`); `save_credential` returns that status, not the key (`cloud.rs:223-237`). The OpenRouter login keeps the exchanged key inside the backend and saves it without returning it to the frontend (`lib.rs:1885-1904`). The key is passed to reqwest as bearer authentication (`cloud.rs:632-645`), and the source contains no production `println!`/`eprintln!` of the secret. Error/status bodies are capped and may include provider detail, but the reviewed formatting paths do not interpolate `secret`. Direct key leakage is **not observed** in this source review; library-generated error text and provider behavior remain an external **UNKNOWN** and justify explicit redirect/error tests.
- **Provider URLs:** production `provider()` accepts only an ID from the fixed `PROVIDERS` table (`cloud.rs:41-114`), and `chat_with_deadline` uses that table’s static base URL (`cloud.rs:869-879`). The only injectable `base_url` is the private `chat_via` test seam (`cloud.rs:882-893`). Direct caller-controlled SSRF is therefore not present. Default redirects are not explicitly constrained (DL-10), so cross-host prompt/key behavior is not proven by this code.
- **Retries:** model listing and chat each allow at most one retry after a 429 when a usable Retry-After exists (`cloud.rs:794-823`, `919-947`). Retry-After is clamped to 1–30 seconds (`cloud.rs:713-730`), and requests have 30-second model-list or 180-second chat ceilings, further clamped by a tuning deadline (`cloud.rs:894-900`). OAuth exchange has a 30-second client timeout but no body cap or retry (`cloud.rs:596-616`).

#### 5. Handwritten HTTP/date parsing and crate replacement

- Production has one handwritten HTTP-date parser plus one handwritten civil-date helper: `download.rs:948-1005`, roughly 58 lines. It is called by the downloader’s bounded Retry-After helper (`download.rs:925-945`) and cloud’s Retry-After path (`cloud.rs:716-729`). `httpdate` is absent from `src-tauri/Cargo.toml:23-47`.
- The parser is intentionally small but does not validate weekday/date consistency or actual day-of-month validity. Use the maintained `httpdate` crate and retain the explicit delay cap.
- Cloud also hand-parses the loopback HTTP request/query (`percent_decode` 325–353, `parse_callback_request_line` 355–403, `read_bounded_request_line` 431–481). That parser is a deliberately bounded protocol seam, not a date parser; `httparse`/URL form-decoding could replace it, but the immediate correctness gap is the shared HTTP-date implementation.
- `download.rs` also constructs URL path segments with the existing `reqwest::Url` API (`376-404`); that is not a handwritten URL serializer, although revision validation must be fixed.

#### 6. Splitting, largest functions, dead code, retry duplication, and audit narration

**Recommended download split**

- `download/plan.rs`: `Chunk`, `ResumeState`, planning and resume predicates, current `download.rs:154-291`.
- `download/paths.rs`: canonical/opened-directory checks, lock, sidecar, hard-link and sync operations, current `47-152` and `447-850`.
- `download/http.rs`: URL construction, redirect policy, client, Retry-After/date, probe and response validators, current `293-445`, `852-1148`, and `1489-1744`.
- `download/transfer.rs`: `download_file`, final verification, identity-checked publication, current `1150-1487`.
- Keep tests beside these units instead of one 2,319-line `#[cfg(test)]` module (`download.rs:1746-4064`).

**Recommended catalog split**

- `catalog/types.rs`: `CatalogFile`, `CatalogModel`, `Catalog`, snapshot/freshness types, current `132-307` and `1010-1082`.
- `catalog/validate.rs`: IPC bounds, schema/row validation, safe repo/filename/revision rules, filters and facets, current `309-1008`.
- `catalog/signature_cache.rs`: bounded body reads, signature verification, cache record, freshness, cooldown, fetch/fallback/load, current `110-121`, `524-616`, `1036-1581`.
- `catalog/credentials.rs`: Hugging Face Credential Manager code, current `1592-1799`.
- Move the 2,102-line catalog test module (`catalog.rs:1800-3902`) into focused test modules/fixtures.

**Dead code**

No confirmed dead production function was found by the source/reference scan. Test seams are intentionally test-only: `fetch_catalog_verified` is called by production `fetch_catalog` and injected tests (`catalog.rs:1215-1227`); download `probe`/resume helpers marked `#[cfg(test)]` support real socket/file tests; the ignored S-25 measurement is explicitly an ignored harness (`catalog.rs:3813-3818`). Do not delete these based on low call counts alone. A full compiler dead-code report is UNKNOWN because Cargo was prohibited.

**Retry duplication**

- `download.rs`: `MAX_TRANSFER_ATTEMPTS = 3` (`36`), bounded numeric/date Retry-After (`925-995`), cancellable 50 ms sleep slices and linear fallback (`1721-1740`).
- `cloud.rs`: one 429 retry, `MAX_RETRY_AFTER_SECS = 30` (`681-685`), one-second sleep slices (`766-779`).
- `runtime.rs`: no in-function retry loop for the runtime catalog; it parses an unrestricted numeric Retry-After into an IPC error (`2545-2556`) and falls back to cache on failure (`2874-2885`). These are separate policies and should share parsing/bounds even if retry counts remain operation-specific.

**Audit-ticket narration**

The reviewed seven Rust files contain 166 lines matching audit/incident markers such as `S-*`, `DC-*`, `RT-*`, `CLD-*`, `GH-*`, or `PR #...`; the largest concentrations are `download.rs` (50), `catalog.rs` (46), `catalog_db.rs` (29), `gguf.rs` (20), and `cloud.rs` (14), with six in `catalog_service.rs` and one in `artifact.rs`. Some are useful invariant references, but test comments also carry historical run details such as `PR #18` and old flake narratives (`download.rs:1750-1764`). Keep the invariant and a stable ledger ID in source; move incident history, run IDs, and release narration out of these security-critical modules.

### Cross-cutting observations

1. **What is verified versus what is merely claimed:** the model-file digest is cryptographic and checked before publication; HTTP size, ETag, Last-Modified, and Content-Range are consistency signals. The catalog signature currently verifies a CRLF-normalized representation, so the README’s “exact bytes” claim is not true.
2. **Local authority is the boundary to settle first:** signed catalog rows and bundled rows are handled as authority; the SQLite mirror is not used for ordinary curated downloads; user overrides are a separate mutable path that currently does authorize. Make that exception explicit or remove it. Do not let a comment saying “never authorizes” coexist with `AuthorizedDownload { authority: "user" }`.
3. **Path safety is stronger for downloads than for artifact inspection and catalog storage:** download command input checks the canonical root and reparse ancestors, while artifact parsing and SQLite/cache open paths remain path-based or unvalidated against reparse races.
4. **Parser limits are real but not equivalent to a memory bound:** GGUF bounds bytes/work/string/array declarations, yet container capacity and cloned IPC output need their own budgets. SQLite checks the row count only after materialization for the same reason.
5. **Cloud secrets are not visibly logged or returned:** keys stay in Credential Manager and status is masked. The remaining trust question is the reqwest redirect policy; make it explicit instead of relying on library defaults.
6. **Date/retry behavior should be one policy:** download and cloud clamp and sleep differently; runtime exposes an unclamped value. A maintained date parser plus one shared cap removes avoidable release-time ambiguity.
7. **No runtime/build verification was performed:** this is a static review with complete file reads and read-only searches. Cargo tests, Clippy, the packaged matrix, and live Hugging Face/provider behavior remain UNKNOWN and must be run by the parent agent under the project’s release gates.

---

## Part 8 — Rust measurement, evidence and tuning (finding prefix MT)

### Summary

This review covers the six requested Rust files in full: 9,314 physical lines total. The `#[cfg(test)]` module boundaries account for approximately 4,848 production lines and 4,465 test lines (47.9% of the scope). The test volume is not a substitute for a coherent production boundary: the measurement path still has two benchmark contracts, the tuner has a separate cancellation path, and the manifest validator does not enforce the same invariants as the acquisition-time validator.

There are **15 findings: 0 Critical, 8 High, 7 Medium, 0 Low**.

The most damaging problems are:

- “Final verification” is two more samples of the same fixed prompt, but the report/UI calls the result `confirmed`; there is no confidence interval, held-out workload, or quality validation.
- A warmup-only failure/cancellation cannot be persisted although the UI says raw failures remain in the saved manifest.
- Cold v2 benchmarking removes the managed server and never restores it after success or after most failure paths.
- v2 can hold the benchmark slot forever if an abandoned HTTP worker does not drain; the tuner has the opposite bug and releases its operation while its legacy HTTP worker may still be alive.
- The v2 compatibility key is generated even when material identities are explicitly marked unknown. Replay does not consult the unknown list.
- The old benchmark and the v2 benchmark are both shipped. The old path is weaker: it accepts `predicted_per_second` without checking that the runtime generated the requested token count, and it produces no v2 manifest.
- Cloud disclosure is opt-in, but the full brief serializes whole hardware/GGUF/profile/trial structures, including stable adapter identity and paths. The displayed disclosure is only a top-level summary, and minimal redaction does not actually remove arbitrary user names.

No cargo, npm, Tauri, application, or network command was run, as required. The two research files named by `AGENTS.md` (`research/measuring/README.md` and `research/measuring/SYNTHESIS.md`) are absent from this checkout, so research-specific comparison is UNKNOWN rather than inferred.

### Per-file verdicts

| File | Verdict | Size and largest function | Why |
|---|---|---:|---|
| `src-tauri/src/tune.rs` | **SPLIT** | 3,267 lines; `run_tuning` 1023–1675 (653 lines) | One function owns baseline measurement, local search, advisor calls, rejection accounting, cancellation, final verification, and report construction. Split the pure search phases before changing behavior. |
| `src-tauri/src/tune_service.rs` | **FIX** | 489 lines; `start_tuning` 229–366 (138 lines) | The command owns process launch, HTTP clients, cloud setup, and lifecycle cleanup. Its `LiveBench` path does not drain abandoned cancellable request workers. |
| `src-tauri/src/measurement.rs` | **SPLIT** | 1,996 lines; `run_workload_with` 320–472 (153 lines) | Statistics, response parsing, warm/cold orchestration, HTTP framing, persistence, replay checks, and 1,258 test lines are in one module. The parser and validator need one contract. |
| `src-tauri/src/measurement_service.rs` | **SPLIT** | 1,641 lines; `run_benchmark_snapshot` 332–559 (228 lines) | Legacy benchmark, v2 lifecycle, snapshot identity, cold-server transition, replay, and ownership tests are mixed. Delete legacy first, then split identity from execution. |
| `src-tauri/src/evidence.rs` | **SHRINK** | 1,414 lines; `validate_attempt_consistency` 813–946 (134 lines; no function over 150) | Keep the typed evidence contract and raw observations, but remove unenforced identity fields and make warmup/metric validation consistent. |
| `src-tauri/src/calibration.rs` | **SHRINK** | 507 lines; `mt07_snapshot_key_changes_for_every_material_field` 243–342 (100-line test; no production function over 150) | This is an execution-snapshot identity/hash module, not a calibration model. Keep a small canonical-key module and delete dead serialized metadata. |

#### Line accounting by purpose

- The six files contain 4,465 test lines. The largest production function is `run_tuning`; it is a maintainability and review boundary, not merely a long test fixture.
- Approximately **874 production lines** are identity/replay/calibration plumbing: `evidence.rs:623-946` (324), `measurement.rs:663-736` (74), `measurement_service.rs:143-318` (176), `measurement_service.rs:507-558` (52), `measurement_service.rs:752-808` (57), and all 191 production lines of `calibration.rs`.
- That 874-line total is not all disposable: raw observations, runtime/model facts, and replay protection are user-facing evidence. The disposable/overgrown part is the duplicate launch/workload identity fields, the unused unknown metadata, and audit-only narration around them.
- `calibration.rs` has no calibration arithmetic. Its production code computes workload/artifact digests, execution-snapshot keys, compatibility-key syntax, and launch-argument redaction.
- The six files contain 70 lines with audit-ticket labels or `audit ...` narration, counted from the actual source. This is concentrated in `tune.rs` (24), `tune_service.rs` (12), `measurement.rs` (12), `measurement_service.rs` (9), `evidence.rs` (8), and `calibration.rs` (5). Several comments describe contracts that the code does not enforce, so the comments are creating false confidence rather than replacing tests.

### Findings

#### Critical

None observed in the read-only review. That is not a release approval: the High findings below include evidence corruption, privacy over-disclosure, and operation-lifecycle failures.

#### High

##### MT-01 — High — Heuristic in-sample repeat is presented as confirmation

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/tune.rs:976-982, 1551-1649`; `src/screens/TuneScreen.tsx:344-349`; `src-tauri/src/measurement.rs:14, 19-28`.
- **Title:** Final verification has no independent validation or statistical confidence.
- **Evidence:**
  > `1551|    // Final verification (audit MT-11 I4): remeasure the baseline and`
  > `1552|    // the finalist, and require a material improvement beyond the observed`
  > `1553|    // baseline drift.`
- **Why it matters:** The tuner chooses the best mean from the same fixed prompt, then measures the same baseline and finalist again on the same machine and calls the threshold result `confirmed` (`tune.rs:1621-1627`). The fixed harness prompt is explicitly one constant (`measurement.rs:14`), and the objective explicitly says quality and latency are not measured (`tune.rs:976-982`). The threshold is a heuristic: twice observed baseline drift, floored at 3%. It is not a confidence interval, a significance test, a held-out workload, or a quality check. The UI prints “confirmed” (`TuneScreen.tsx:348`), which is stronger than the evidence supports.
- **Concrete fix:** Rename the result to something like `repeatMaterialityGatePassed`, display “repeat comparison passed,” and state that it is in-sample. If the product needs a validation claim, add a separate held-out prompt/workload and an explicitly named uncertainty method; do not turn the existing two repeats into a confidence claim.

##### MT-03 — High — Warmup-only failure cannot produce the promised saved manifest

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/measurement.rs:342-390`; `src-tauri/src/evidence.rs:756-823`; `src-tauri/src/measurement_service.rs:537-550`; `src/V03EvidencePanel.tsx:569-587`.
- **Title:** A failure or cancellation during the only warmup is discarded before persistence.
- **Evidence:**
  > `387|                    error: Some(bound_observation_error(&error)),`
  > `388|                    ..WarmupObservation::default() `
  > `389|                });`
- **Why it matters:** `run_workload_with` returns a `WorkloadRun` with a failed/cancelled warmup, `terminal_outcome`, and **zero measured observations** (`measurement.rs:380-390`). `validate_attempt_consistency` unconditionally rejects an empty observation list (`evidence.rs:818-823`). `run_benchmark_snapshot` validates before it computes or persists the result (`measurement_service.rs:537-550`). With the default one warmup (`evidence.rs:270-283`), a timeout or Stop during warmup returns an error instead of the “Raw failures remain in the saved manifest” behavior promised by the panel (`V03EvidencePanel.tsx:572-574`).
- **Concrete fix:** Define a terminal warmup-only manifest shape and validate it, including exact warmup IDs and terminal outcome; or append a clearly marked failed/cancelled first observation when no measured trial started. Add regression tests for cancel, timeout, and launch failure during warmup and assert that a manifest is actually written.

##### MT-05 — High — Cold benchmark removes the managed server and does not restore it

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/measurement_service.rs:613-688, 689-733`; `src/V03EvidencePanel.tsx:591-598`.
- **Title:** Cold mode changes global server state without a restoration path.
- **Evidence:**
  > `676|                slot.take().expect("validated server was present")`
  > `677|            };`
  > `678|            if !managed.child.terminate_and_wait() {`
- **Why it matters:** The v2 command requires a running managed server, takes it out of `state.server`, terminates it, and then runs fresh contained runtimes for cold trials (`measurement_service.rs:664-688`). After that, the command only drains request workers, clears the benchmark cancellation slot, checks the reservation, and returns (`measurement_service.rs:709-733`). It never puts the managed server back or restarts it on success. If snapshot assembly, preparation, or the cold run fails after the take, the same server is also gone. The UI says only “Cold (requires a fresh runtime)” (`V03EvidencePanel.tsx:597`); it does not disclose that the selected profile will end stopped.
- **Concrete fix:** Treat the managed server as a transaction: retain its ownership, run the cold attempts, then restore/restart the exact validated server before releasing the operation. If the product intentionally leaves it stopped, say so before the run and make the post-run state explicit; still restore the state on error unless cleanup is unresolved.

##### MT-06 — High — Worker-drain failure can reserve the benchmark slot forever

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/measurement_service.rs:28-42, 709-723`; `src-tauri/src/local_client.rs:677-737`.
- **Title:** Fail-closed ownership has no bounded recovery state.
- **Evidence:**
  > `37|pub(crate) fn drain_owned_workers(client: &LocalHttpClient, ceiling: Duration) {`
  > `38|    if client.wait_for_worker_drain(ceiling) { return; }`
  > `41|    while !client.wait_for_worker_drain(Duration::from_secs(1)) {}`
- **Why it matters:** A cancelled local request returns to the caller while its worker continues until the full request deadline (`local_client.rs:677-686`). v2 deliberately waits before clearing `state.benchmark` (`measurement_service.rs:709-723`), which is correct while work is alive, but after the ceiling it polls forever. A stuck worker therefore keeps the command, benchmark slot, and operation reservation alive indefinitely. This directly answers the ownership question: yes, a cancelled run can leave a slot reserved indefinitely if the worker does not drain. The code has no “unresolved owner” state, diagnostic, or controlled recovery.
- **Concrete fix:** Make the HTTP transport interruptible or give the owned worker a real termination mechanism, then prove it with a withheld-response test. If termination remains impossible, transition to an explicit poisoned-owner state with a safe, owned recovery action; never silently clear the slot while the request may still be inferring, and never leave the UI looking merely busy forever.

##### MT-07 — High — Tuner cancellation releases its operation while its legacy HTTP worker may continue

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/tune_service.rs:25-101, 228-365`; `src-tauri/src/core.rs:2325-2345`; `src-tauri/src/local_client.rs:677-737`.
- **Title:** `LiveBench` does not drain abandoned cancellable requests.
- **Evidence:**
  > `85|            // (audit MT-04).`
  > `86|            let client = crate::local_client::LocalHttpClient::from_profile(profile)?;`
  > `87|            core::benchmark_server_cancellable(&client, self.tokens, self.repeats, &self.cancel)`
- **Why it matters:** `benchmark_server_cancellable` uses `post_json_cancellable`; on Stop, the caller returns but the worker is intentionally abandoned until the request deadline (`local_client.rs:677-686`). `LiveBench` terminates only the trial child (`tune_service.rs:94-100`) and never calls `wait_for_worker_drain`. `start_tuning` clears `state.tuning` and returns after the blocking task settles (`tune_service.rs:361-365`); it does not publish this client in `state.benchmark`, so the next operation has no cross-check for the abandoned request. Killing the trial server will usually make the request fail, but that is not an ownership proof and is not bounded cleanup.
- **Concrete fix:** Route tuner measurements through the v2 shared client/runner and drain it before the tuning reservation is released, or add the same owned-worker tracking and recovery contract to `LiveBench`. Add a test that withholds a completion response, cancels tuning, and proves the next launch cannot proceed until the owned request exits.

##### MT-08 — High — Unknown execution identity is recorded but ignored by replay

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/measurement_service.rs:242-268, 286-317, 792-807`; `src-tauri/src/calibration.rs:81-84`; `src-tauri/src/evidence.rs:637-644`.
- **Title:** A compatibility key can validate a record whose material machine identity is explicitly unknown.
- **Evidence:**
  > `247|    let host_cpu_model = String::new();`
  > `258|    if host_cpu_model.trim().is_empty() {`
  > `261|        unknown_identities.push("hostCpuModel".into());`
- **Why it matters:** The snapshot includes an empty CPU model and marks it unknown, and it may also mark effective context, memory, driver, or adapter identity unknown (`measurement_service.rs:249-268`). `ExecutionSnapshotV2::reuse_supported` returns false when unknowns exist (`calibration.rs:81-84`), but `benchmark_execution_snapshot_from_profile` still creates a compatibility key and `replay_benchmark_manifest_worker` compares only that key plus raw launch arguments (`measurement_service.rs:792-807`; `measurement.rs:709-736`). A different CPU can therefore match a key containing the same empty CPU identity if the other fields happen to match. The serialized `execution_snapshot_unknowns` field is not consulted by complete-manifest validation or replay.
- **Concrete fix:** Either collect every material identity used for reuse, or make replay/reuse reject manifests with nonempty unknown identities. Put the check in the authoritative replay/compatibility gate, not only in a metadata field that the UI never evaluates.

##### MT-09 — High — Shipped legacy benchmark is weaker and duplicates v2

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/measurement_service.rs:52-129, 603-734`; `src-tauri/src/core.rs:2240-2346`; `src/screens/BenchmarkScreen.tsx:45-76`; `src/V03EvidencePanel.tsx:569-587`; `src-tauri/src/tune_service.rs:83-87`.
- **Title:** “Run benchmark,” “Run v2 benchmark,” and tuner measurement do not share one evidence contract.
- **Evidence:**
  > `2281|pub fn parse_tps(body: &str) -> Result<f64, String> {`
  > `2283|    value["timings"]["predicted_per_second"]`
  > `2285|        .filter(|v| *v > 0.0)`
- **Why it matters:** The legacy parser accepts a positive server-reported rate without checking `predicted_n` against the requested token count. It can therefore report throughput for a short response, while v2 checks both prompt accounting and generated token count (`measurement.rs:591-607`). The old path returns only `BenchmarkSummary` and has no v2 manifest, workload identity, or raw failure record. The old screen exposes “Run benchmark” (`BenchmarkScreen.tsx:45-49`) while the evidence panel exposes “Run v2 benchmark” and “Replay manifest” (`V03EvidencePanel.tsx:569-587`), and the tuner directly calls the old core loop (`tune_service.rs:83-87`).
- **Concrete fix:** Delete `benchmark_server`, `run_legacy_benchmark`, `BenchmarkSummary`, `benchmark_server_cancellable`, and the old benchmark UI state after migrating the tuner and screen to the v2 runner. Keep one v2 harness and one manifest contract; keep replay as a validation-plus-new-v2-run action, not as a third measurement implementation. The obvious duplicated production surface is about 275 lines before its tests: roughly 79 in `measurement_service.rs`, 107 in the legacy core block, and 89 in `BenchmarkScreen.tsx`.

##### MT-10 — High — Cloud wire payload is broader than the displayed disclosure

> **Lead check: VERIFIED.** tune.rs:326-345 serializes the full HardwareInfo, including adapterId, compatibilityId, and physicalId (runtime.rs:360-382). The disclosure at tune.rs:236-240 omits them. The section is sent in minimal mode.

- **Location:** `src-tauri/src/tune.rs:218-256, 326-356`; `src-tauri/src/runtime.rs:161-177, 363-380`; `src-tauri/src/gguf.rs:102-132`; `src/screens/TuneScreen.tsx:189-225, 237-240`; `src-tauri/src/cloud.rs:901-914`.
- **Title:** The advisor receives whole structured objects, including stable hardware IDs and full profile/trial data.
- **Evidence:**
  > `332|    pub hardware: &'a HardwareInfo,`
  > `333|    pub system_ram_bytes: Option<u64>,`
  > `334|    pub gguf: Option<&'a GgufSummary>,`
- **Why it matters:** `TuningBrief` serializes the complete `HardwareInfo`, `GgufSummary`, `baseline_profile`, and every prior `TuningTrial` (`tune.rs:326-345`). `HardwareInfo` includes `detection_status`, `recommendation`, all adapter records, `manual_overrides`, and unassigned telemetry (`runtime.rs:163-177`); each adapter can include `adapter_id`, `compatibility_id`, and a physical GPU identity (`runtime.rs:363-380`). `GgufSummary` includes relevant metadata facts and tensor descriptors, not just the architecture/size summary (`gguf.rs:102-132`). Full mode also sends model/runtime/companion paths as the disclosure admits. The UI disclosure names only high-level “GPU names and VRAM” and “GGUF metadata” (`TuneScreen.tsx:218-241`), so the user is not shown the actual nested payload. The provider request sends the JSON wire brief in the user message (`cloud.rs:901-914`).
- **Concrete fix:** Build an explicit cloud-wire DTO containing only fields needed for the advisor. Remove physical IDs, adapter IDs, evidence source details/timestamps, tensor descriptors, and arbitrary metadata unless each is separately disclosed. Default the optional cloud step to the least identifying mode and test the serialized wire payload against the disclosure list.

#### Medium

##### MT-02 — Medium — Invalid optional timing is silently downgraded to missing and derived overflow is unchecked

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/measurement.rs:272-317`.
- **Title:** Response parser does not distinguish absent `first_token_ms` from malformed present data.
- **Evidence:**
  > `296|        .and_then(serde_json::Value::as_f64)`
  > `297|        .filter(|value| value.is_finite() && *value > 0.0);`
- **Why it matters:** A present zero, NaN-like JSON value, negative value, or wrong type becomes `None` without an error, so a malformed runtime response is accepted as a successful observation with less evidence. Separately, `prompt_ms + predicted_per_token_ms` is formed directly (`measurement.rs:315`); two finite values can overflow to infinity, which is caught only later by a different validator and causes a whole run to fail rather than producing a named parser error. The normal observation path rejects non-finite metrics (`measurement.rs:43-65`), but the parser gap still loses the distinction between “runtime omitted this metric” and “runtime returned invalid data.”
- **Concrete fix:** Treat a present `first_token_ms` as a required positive finite value; reserve `None` for absent/null. Compute derived TTFT with checked/finite arithmetic and return a parser error naming the field. Add malformed and extreme-value response tests.

##### MT-04 — Medium — Manifest completeness checks are weaker than acquisition checks

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/evidence.rs:697-753, 756-807, 813-946`; `src-tauri/src/measurement.rs:43-65`.
- **Title:** Warmups are only upper-bounded, and optional zero metrics can pass one validator but fail another.
- **Evidence:**
  > `699|            || self.warmups.len() > self.workload.warmups as usize`
  > `700|            || self.observations.len() > self.workload.trials as usize`
  > `701|        {`
- **Why it matters:** A complete manifest may contain fewer warmups than requested, including zero warmups for a nonterminal record; warmup IDs are not checked for positivity, uniqueness, or sequence. `validate_attempt_consistency` checks measured trial IDs but ignores warmups (`evidence.rs:842-920`). The manifest-level optional metric check rejects only negative values (`evidence.rs:738-746`), while `validate_observation` rejects zero as well (`measurement.rs:59-63`). A deserialized manifest with `prefillTps: 0` or `derivedTtftMs: 0` can pass `validate_complete` but fail when summarized. This is a split persistence contract, not a harmless compatibility choice.
- **Concrete fix:** Centralize positive-finite metric validation and validate warmup IDs/counts/outcome consistency in the same function used by persistence and replay. Require exactly the requested warmups for a nonterminal complete record.

##### MT-11 — Medium — Minimal disclosure does not remove arbitrary user names

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/tune.rs:259-323`; `src/screens/TuneScreen.tsx:203-213`.
- **Title:** The UI promises “user names removed,” but redaction only replaces an exact home-path string or path-looking tokens.
- **Evidence:**
  > `265|    if let Some(home) = home {`
  > `266|        if !home.is_empty() {`
  > `267|            text = text.replace(home, "<local>");`
- **Why it matters:** Minimal mode does not derive the username from `USERPROFILE`, perform case-insensitive replacement, or remove a username appearing in a profile alias, rationale, provider error, model metadata, or ordinary prose. Path-looking strings are shortened later (`tune.rs:282-300`), but `"Mubarak RTX profile"` and an error containing a bare username remain unchanged. The UI explicitly says “directories and user names removed” (`TuneScreen.tsx:212`), so this is a disclosure correctness failure.
- **Concrete fix:** Prefer an allowlisted minimal wire DTO. If free-form text remains, remove the case-insensitive profile-name component and test aliases, errors, path case variants, and usernames outside path strings. Do not claim complete user-name removal without such a test.

##### MT-12 — Medium — Cloud advisor request ignores Stop until the blocking call returns

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/tune.rs:837-840, 1388-1407`; `src-tauri/src/cloud.rs:858-947`; `src-tauri/src/tune_service.rs:374-393`.
- **Title:** Cloud cancellation is polling around a blocking provider request, not cancellation of the request.
- **Evidence:**
  > `858|/// The chat call with an optional deadline: the request timeout never`
  > `861|pub fn chat_with_deadline<S: SecretStore>(`
  > `867|    deadline: Option<std::time::Instant>,`
- **Why it matters:** The tuning loop checks the user flag before and after `advisor.propose`, but `chat_via` performs a blocking `reqwest` send and only has a deadline/timeout, not a cancellation flag (`cloud.rs:894-916`; `tune.rs:1390-1407`). A Stop during the advisor call can therefore wait for the provider timeout/retry path before `start_tuning` clears state. This is separate from the local llama-server worker leak in MT-07.
- **Concrete fix:** Use a cancellation-aware provider transport or run the request under a bounded owned worker whose lifecycle is drained before releasing tuning. At minimum, surface “waiting for cloud request to finish” and bound retry/request time independently of the 45-minute tuning deadline.

##### MT-13 — Medium — Runtime manifest identity has dead fields and audit-ticket narration that contradicts code

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/evidence.rs:623-652`; `src-tauri/src/measurement_service.rs:405-416, 632-640`; `src-tauri/src/calibration.rs:28-190`; `src-tauri/src/cloud.rs:963-988`.
- **Title:** Release/reuse paperwork is serialized into the app without an authoritative consumer.
- **Evidence:**
  > `632|    // F9-02: the fallible client is built BEFORE the active slot is published.`
  > `638|    let client = publish_benchmark_slot(&state.benchmark, cancelled.clone(), || {`
  > `639|        local_client(&server.profile)`
- **Why it matters:** The v2 comment says construction happens before publication, but `publish_benchmark_slot` publishes first and invokes the fallible constructor inside its closure (`measurement_service.rs:568-600`). More materially, `launch_compatibility_key`, `execution_snapshot_schema`, and `execution_snapshot_unknowns` are assigned into manifests (`measurement_service.rs:510-514`) but no production code in this scope reads them to authorize replay or calibration; replay uses only `compatibility_key`, raw args, and model identity (`measurement.rs:709-736`). `CloudAdvisor.last_raw_reply` is initialized and assigned (`tune_service.rs:337-342`; `cloud.rs:963-988`) but not consumed by the product. These are audit/release records that enlarge the shipped contract while their comments imply enforcement that does not exist.
- **Concrete fix:** Decide which identity fields are user-facing and enforce them in one gate. Delete write-only launch/schema/unknown fields and `last_raw_reply` if no consumer needs them; otherwise add a real consumer and tests. Move audit-ticket rationale into design documentation, leaving code comments to describe invariants that the code actually enforces.

##### MT-14 — Medium — Tuning and cold benchmark detach log-drain threads

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/tune_service.rs:40-100`; `src-tauri/src/measurement_service.rs:456-488`; `src-tauri/src/lib.rs:73-75, 745-753`; `src-tauri/src/server_service.rs:260-267`.
- **Title:** Callers discard the `JoinHandle`s that the spawn contract says must be joined.
- **Evidence:**
  > `458|            let (mut child, _, log_path, _lease, _drains) =`
  > `459|                spawn_server(&profile, "benchmark-cold")?;`
  > `477|            let cleanup = child.terminate_and_wait();`
- **Why it matters:** `spawn_server` returns bounded log-drain handles, and `ManagedServer` documents that they are joined after child exit so the log is complete (`lib.rs:73-75`). The normal server stop path actually drains/joins them (`server_service.rs:260-267`), but cold benchmark and tuner bind them as `_drains` and let the handles detach. On cleanup failure or slow file draining, launch evidence can remain incomplete and untracked while the operation is reported as finished. This is also inconsistent with the process-tree ownership rule.
- **Concrete fix:** Carry the drain handles through both temporary-server paths and join them with the same bounded cleanup policy as managed-server stop. If a join times out, report unresolved cleanup and retain ownership rather than silently detaching.

##### MT-15 — Medium — The same product exposes two incompatible standard-deviation conventions

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src-tauri/src/measurement.rs:139-160`; `src-tauri/src/tune.rs:985-997`.
- **Title:** v2 `standardDeviation` is population SD while tuner `stdDev` is sample SD.
- **Evidence:**
  > `145|        .sum::<f64>()`
  > `146|        / count as f64;`
  > `996|        / (samples.len() as f64 - 1.0);`
- **Why it matters:** `MetricStats.standard_deviation` divides by `n`, while `sample_std_dev` divides by `n-1`. Both are exposed as ordinary standard-deviation fields, and the tuner UI renders `stdDev` with a `±` label (`TuneScreen.tsx:367-369`). They are not comparable for small repeat counts, yet no contract names the convention. This is not an off-by-one percentile bug; it is an evidence-schema inconsistency.
- **Concrete fix:** Choose and document one convention, or rename the fields to `populationStandardDeviation` and `sampleStandardDeviation`. Add a shared statistic test vector so future benchmark-path deletion cannot silently change the meaning.

### Answers to specific questions

#### 1. Statistics, percentiles, intervals, NaN/inf, and validation

- **v2 metric set:** `MetricStats` contains count, mean, median, p50, p95, min, max, and standard deviation (`measurement.rs:17-28`). `metric_stats` drops non-finite and non-positive values, sorts with `f64::total_cmp`, computes a max-scaled mean to avoid overflow, uses an even-count midpoint for `median`, and uses nearest-rank indexing `ceil(percent * count / 100) - 1` for p50/p95 (`measurement.rs:122-160`). The rank is one-based and the implementation is not off by one. For small samples p95 is often the maximum; that is a consequence of nearest rank, not a 95% interval.
- **Median versus p50:** They deliberately differ for even-sized samples. The test asserts that `[10,20,30,40]` has median 25 and nearest-rank p50 20 (`measurement.rs:835-841`). The UI prints both p50 and median (`V03EvidencePanel.tsx:614-622`). This is statistically defensible but not explained to users.
- **v2 standard deviation:** population SD (`measurement.rs:139-146`). **Tuner SD:** sample SD (`tune.rs:985-997`). This inconsistency is MT-15.
- **Legacy statistics:** `core::summarize_benchmark` validates positive finite samples, sorts them, uses the same scaled mean and midpoint median, and exposes only min/max/mean/median (`core.rs:2250-2278`). It has no percentile or SD contract. `parse_tps` only reads a positive `predicted_per_second` (`core.rs:2281-2286`), which is part of MT-09.
- **NaN/inf handling:** Normal v2 observations reject non-finite or non-positive metric values before summary (`measurement.rs:43-65`), and legacy summary rejects non-finite samples (`core.rs:2255-2260`). The gaps are the optional `first_token_ms` downgrade and unchecked derived-TTFT addition in MT-02. `metric_stats` filtering is not a normal-path silent data loss because `summarize_observations` validates the source observations first.
- **Intervals/confidence:** There is no confidence interval, uncertainty interval, coverage, or confidence field in `MetricStats` or the manifest. The UI shows p50/p95 and explicitly says derived TTFT is not an observed first token (`V03EvidencePanel.tsx:614-630`). The tuner’s “confirmed” flag is only a heuristic repeat/materiality gate; it must not be described as statistical confirmation (MT-01).
- **In-sample versus validation:** Tuning remeasures the same baseline and finalist using the same fixed harness prompt, then applies a 3%/drift threshold (`tune.rs:1551-1627`). There is no held-out prompt, independent workload, or quality test. The UI does include the honest caveat “Best measured in this session — never optimal” (`TuneScreen.tsx:344-348`), but the separate word “confirmed” still overstates the result.

#### 2. Duplicate benchmark paths and what should be deleted

There are two actual benchmark implementations and a replay wrapper:

1. **Legacy command:** `measurement_service::benchmark_server` delegates to `run_legacy_benchmark` (`measurement_service.rs:52-129`), which calls `core::benchmark_server_cancellable`. The old `BenchmarkSummary`/`summarize_benchmark`/`parse_tps` path is in `core.rs:2240-2346`.
2. **v2 command:** `measurement_service::benchmark_v2` owns a workload, manifest, snapshot identity, raw observations, warm/cold mode, worker drain, and persistence (`measurement_service.rs:603-734`; `run_benchmark_snapshot` at 332-559).
3. **Replay manifest:** `replay_benchmark_manifest` validates current model/launch/snapshot identity and returns the workload (`measurement_service.rs:752-808`). The UI then calls `benchmarkV2` with that workload (`V03EvidencePanel.tsx:377-385`), so replay is a validation-plus-new-v2-run action, not a third measurement algorithm.
4. **Tuner measurement:** `LiveBench` launches a trial server but calls the **legacy** `core::benchmark_server_cancellable` (`tune_service.rs:83-87`). It therefore inherits the weaker token-count/evidence contract and the separate worker lifecycle.

The old screen says “Run benchmark” (`BenchmarkScreen.tsx:45-49`); the evidence panel says “Run v2 benchmark” (`V03EvidencePanel.tsx:569-587`). Delete the legacy command, old summary/parser, old screen state, and tuner call after routing those callers through the v2 runner. Keep v2 and keep replay as a gate around v2. The legacy production surface is approximately 275 lines before tests, and its existence is already causing materially different statistics, evidence, and cancellation behavior.

#### 3. Process and HTTP ownership/cancellation

- **Legacy benchmark:** ownership is comparatively disciplined. It publishes a benchmark slot, runs the cancellable legacy loop, drains the shared client before clearing the slot, and finalizes only if the reservation is still current (`measurement_service.rs:88-129`).
- **v2 warm benchmark:** the shared client tracks cancellable workers. Cancellation returns to the run, but the worker continues until the request deadline (`local_client.rs:677-737`). v2 waits before clearing `state.benchmark` (`measurement_service.rs:709-723`). This prevents overlap while the worker is alive but can block forever after the ceiling (MT-06).
- **v2 cold benchmark:** each fresh server is contained and `terminate_and_wait` reports false when exit is not observed (`proc.rs:363-391`), but the cold path does not restore the pre-existing managed server and detaches its log drains (MT-05, MT-14). A fresh runtime can remain unresolved when termination cannot be confirmed; the code reports the failure, but the application has no recovery state.
- **Tuner:** `LiveBench` kills the trial child but does not drain the `LocalHttpClient` worker (MT-07). Its operation reservation lasts only until `start_tuning` returns; the abandoned client is not represented in the global benchmark slot.
- **Direct answer:** Yes, a cancelled v2 run can leave the benchmark slot/reservation held indefinitely if the worker does not drain. A warm managed `llama-server` is intentionally left alive because v2 measures it; that is not itself a leak. A cold or tuner trial server can remain alive when `terminate_and_wait` cannot confirm exit, and the tuner can release its operation before its HTTP worker has been proven dead.

#### 4. Cloud advisor payload, disclosure, consent, and command safety

**Payload sent off-machine:** `CloudAdvisor` sends the selected provider/model request plus a user message containing `brief.wire` (`cloud.rs:901-914`, `cloud.rs:973-988`). The brief includes:

- objective, target context, remaining budget, tunable field names and allowed speculative types (`tune.rs:218-256, 326-345`);
- complete `HardwareInfo` and system-memory evidence, including adapter identity and optional physical identity (`runtime.rs:161-177, 363-380`);
- the complete `GgufSummary`, including selected metadata facts and recorded tensor descriptors (`gguf.rs:102-132`);
- runtime build, companion strings, the serialized baseline profile, and every tuning trial, including commands, rationales, measurements, and error text (`tune.rs:326-345`);
- in full mode, local runtime/model/companion and other file paths. In minimal mode, path-looking values are shortened, but stable hardware IDs and arbitrary non-path strings remain.

**Disclosure/consent:** This is opt-in at the advisor level: `useAdvisor` defaults false (`App.tsx:325`) and the checkbox says the advisor adds one extra request (`TuneScreen.tsx:237-240`). The UI has a disclosure panel and a Full/Minimal radio choice (`TuneScreen.tsx:189-225`). However, Full is the default both in the frontend (`App.tsx:329-332`) and Rust (`tune.rs:193-204`), and the displayed top-level list does not enumerate the nested IDs/metadata actually serialized (MT-10). Minimal also overpromises user-name removal (MT-11). The secret value itself is retrieved from keyring and placed in the HTTP authorization header, not in the brief (`cloud.rs:869-879, 766-769`); the brief still carries paths and user/runtime facts.

**Can advisor output reach an unvalidated command line?** No direct path was found. `parse_proposal` produces JSON; `apply_changes_with_companions` rejects non-whitelisted fields, coerces to the existing field type, restricts `specType` to runtime-advertised types, resolves draft companions, and calls `profile.build_args()` (`tune.rs:487-556`). `spawn_server` then calls `prepare_launch`, which inspects the selected runtime and calls `validate_launch_arguments` (`lib.rs:621-639`). Capability filtering is based on parsed real `--help` flags (`core.rs:1577-1598`). Domain/range checks also run in `LaunchProfile::build_args` (`core.rs:740-926`). The remaining risk is semantic quality and privacy, not shell injection from the advisor response.

#### 5. Is evidence/manifest functionality user need or release paperwork?

It is both, and the boundary is visible:

- **User functionality to keep:** workload definition and validation, raw per-trial observations, warmups/failures/cancellation outcomes, runtime/model/hardware facts, scope caveats, saved manifests, and replay of the same workload against a current identity. The UI exposes these directly: it says raw failures remain (`V03EvidencePanel.tsx:569-574`), offers replay (`V03EvidencePanel.tsx:585-587`), and renders p50/p95/derived-TTFT and sampled CPU working-set scope (`V03EvidencePanel.tsx:613-661`).
- **Release/reuse paperwork that leaked into the runtime path:** execution-snapshot schema/version, launch/workload compatibility keys, unknown-identity lists, estimator version, sanitized effective-argument fingerprints, and audit-ticket labels. The identity/replay/calibration ranges account for approximately 874 production lines (see the accounting above). `calibration.rs` is entirely this identity machinery; it does not calibrate memory or benchmark uncertainty.
- **Dead/unenforced portion:** `launch_compatibility_key`, `execution_snapshot_schema`, and `execution_snapshot_unknowns` are written into the manifest but are not authoritative replay gates. `CloudAdvisor.last_raw_reply` is write-only in the product path. Either enforce these fields or delete them. Do not keep release paperwork in a serialized user contract merely because an audit ticket once named it.

#### 6. Longest functions, shrink plan, dead code, and audit-ticket narration

- **`tune.rs`:** `run_tuning` (653 lines) should become a coordinator over baseline, grid, nudge, advisor, and final-repeat functions. Delete the legacy summary dependency after v2 migration. Keep scoring and proposal application pure and separately tested.
- **`measurement_service.rs`:** `run_benchmark_snapshot` (228 lines) mixes artifact inspection, identity assembly, warm/cold lifecycle, workload execution, validation, summary classification, and persistence. Move identity construction to a small module, make cold server restoration a transaction, and keep one benchmark runner.
- **`measurement.rs`:** `run_workload_with` (153 lines) mixes warmup/trial state transitions and evidence construction. Split state transitions from timing conversion, then make parser/manifest validators share positive-finite rules.
- **`tune_service.rs`:** `start_tuning` is 138 lines and currently owns too much lifecycle glue. Extract a single supervised run guard that owns cancellation, client drain, child cleanup, and reservation release.
- **`evidence.rs`:** `validate_attempt_consistency` is 134 lines. Keep one manifest validator, but make it validate warmups and metrics with the same contract as the acquisition path; remove unused launch/snapshot fields rather than adding more compatibility branches.
- **`calibration.rs`:** no production function exceeds 150 lines. Shrink it to canonical identity hashing/redaction, or rename it to execution identity; do not call it calibration when no calibration estimator exists.
- **Dead/weak surfaces:** legacy `BenchmarkSummary`/`parse_tps`; manifest launch/schema/unknown fields with no production consumer; `CloudAdvisor.last_raw_reply`; and comments that claim stronger lifecycle or identity enforcement than the code performs. `metric_stats_for_test` is test-only by design and is not a dead production surface.
- **Audit narration:** the source has 70 audit-ticket-labelled lines in the six files. The stale `measurement_service.rs:632` comment is a concrete example: it says the client is built before slot publication, while `publish_benchmark_slot` publishes first and constructs inside the closure (`measurement_service.rs:568-600`). Replace ticket prose with short invariant comments and tests that assert observable ownership/evidence behavior.

### Cross-cutting observations

1. **One evidence contract is the priority deletion.** The v2 path already has the stronger token accounting, failure retention, scope caveat, and manifest. The legacy path is not a harmless compatibility layer because the tuner still uses it and users can select it from a different screen.
2. **“Unknown” is used as a label but not always as a gate.** The code correctly records unknown hardware/identity facts, but replay and manifest acceptance can still treat the resulting hash as reusable. Unknown should remain visible and block claims that require the missing fact.
3. **Ownership is inconsistent by caller.** v2 intentionally retains ownership until workers drain; the tuner uses the same cancellable worker primitive but does not retain or drain it. Process containment protects against killing unrelated processes, but it does not by itself prove that the request worker or log-drain threads have exited.
4. **The product is honest in several UI caveats but undermines them with names.** “Best measured in this session — never optimal,” “quality and latency are not measured,” and “derived TTFT ... not an observed first token” are correct. `confirmed`, top-level cloud disclosure, and “Replay manifest” need similarly precise semantics.
5. **The research baseline is unavailable.** `AGENTS.md` requires `research/measuring/README.md` and `research/measuring/SYNTHESIS.md`, but neither exists in this checkout. No claim here relies on those absent documents.
6. **No verification result is claimed.** This is a source review only. Tests, Clippy, packaging, and runtime cancellation behavior remain UNKNOWN until the parent agent's build gates exercise them.

---

## Part 9 — Frontend (finding prefix FE)

### Summary

Read-only review of the requested React, TypeScript, CSS, design-system, configuration, and test files. All 41 in-scope files were read completely in bounded chunks. No npm, Vitest, TypeScript, Cargo, Tauri, application, or network command was run, as required; runtime and visual claims that cannot be established statically are marked UNKNOWN.

Finding counts: **1 Critical, 5 High, 5 Medium, 2 Low**.

The frontend has one concrete IPC payload defect: `CatalogQuery` sends three snake_case keys while Rust deserializes that struct with `rename_all = "camelCase"`; those filters silently fall back to empty/zero defaults. Every one of the 50 unique command names used by the frontend has a matching `#[tauri::command]` and appears registered in `lib.rs`; nine call sites still rely on inferred/untyped `invoke` results. The largest structural risk is `App.tsx`: one 1,689-line component owns inventory, runtime, catalog, server, benchmark, credentials, tuning, persistence, listeners, and the entire shell.

Credentials are not written to `localStorage` or `sessionStorage`, but raw HF and cloud secret drafts are held in React state and passed through screen props, which violates the project rule that credentials never live in frontend state. The current `V03EvidencePanel` and `v0.4.1` names are not dead history: both are active compatibility/product paths. They should be migrated and renamed deliberately, not deleted casually.

### Per-file verdicts

- `src/App.tsx` — **1,858 lines — SPLIT**. `App` spans 170–1858; it contains 90 textual `useState` occurrences/89 calls (import excluded), 17 textual `useEffect` occurrences/16 calls, and 15 textual `useRef` occurrences/14 calls. It is the application controller, persistence layer, IPC coordinator, async race guard, and shell renderer at once.
- `src/main.tsx` — **12 lines — KEEP**. Root creation and boundary wiring only.
- `src/ErrorBoundary.tsx` — **59 lines — FIX**. `ErrorBoundary.render` spans 34–59; the reset path needs storage-failure handling.
- `src/model.ts` — **2,098 lines — SHRINK**. No component; it mixes IPC mirrors, validators, runtime/catalog decisions, persistence normalization, benchmark evidence, and tuning decisions. Keep pure decisions here, but separate clearly by domain rather than continuing one shared contract file.
- `src/evidence-adapter.ts` — **44 lines — FIX**. The adapter boundary is the right seam, but every production `invoke` is left without an explicit result generic.
- `src/V03EvidencePanel.tsx` — **668 lines — SPLIT**. `V03EvidencePanel` spans 126–668 and owns hardware, artifact inspection, preflight, benchmark, replay, cancellation, stale-input identity, and all rendering. Rename the component semantically during the split.
- `src/vite-env.d.ts` — **1 line — KEEP**.
- `src/App.css` — **690 lines — FIX**. One stylesheet contains the whole product; the two remaining box shadows violate the flat-cabinet rule, although lamp radii, motion reduction, mobile targets, and 110px clearance are otherwise explicit.
- `src/screens/AboutScreen.tsx` — **101 lines — KEEP**.
- `src/screens/BenchmarkScreen.tsx` — **89 lines — SPLIT**. The component spans 37–89 and combines the legacy benchmark controls at 43–78 with the active v2 evidence panel mount at 79–86.
- `src/screens/CatalogScreen.tsx` — **293 lines — SPLIT**. `CatalogScreen` spans 84–293 and combines credential entry, filters, row readiness, download progress, cancellation, and catalog rendering.
- `src/screens/DashboardScreen.tsx` — **141 lines — KEEP**. `DashboardScreen` spans 42–141; no component exceeds 150 lines.
- `src/screens/InventoryScreen.tsx` — **139 lines — FIX**. The screen declares itself presentational but directly invokes `cancel_scan` at 55.
- `src/screens/PathText.tsx` — **31 lines — KEEP**.
- `src/screens/ProfileEmptyScreen.tsx` — **39 lines — KEEP**.
- `src/screens/ProfileScreen.tsx` — **375 lines — SPLIT**. `ProfileScreen` spans 40–375 and owns identity, runtime, numeric options, speculative decoding, security, advanced sections, command preview, save, and start actions.
- `src/screens/RuntimeScreen.tsx` — **407 lines — SPLIT**. `RuntimeScreen` spans 65–407 and combines hardware/runtime setup, catalog, managed installation, identity, and seven-stage health.
- `src/screens/TuneScreen.tsx` — **383 lines — SPLIT**. `TuneScreen` spans 117–383 and combines provider credentials, disclosure, workload setup, live trials, and report adoption.
- `src/screens/evidence-run.ts` — **6 lines — KEEP**.
- `src/model.test.ts` — **1,240 lines — SHRINK**. Pure-decision coverage is correctly centralized, but it also carries legacy/v0.3 naming and a 300-input property campaign; keep the contract coverage while removing historical names and unsafe fixture casts.
- `src/App.catalog.test.tsx` — **1,609 lines — SHRINK**. It tests catalog, runtime, inventory, persistence, error recovery, download identity, accessibility, tuning disclosure, paths, and filter races; the largest named regions are the public catalog suite at 209–347 and delayed/rejected catalog IPC at 1473–1609.
- `src/App.staleResponses.test.tsx` — **659 lines — SHRINK**. It covers stale port/preview, legacy benchmark, tuning numeric fields, and provider races from 271–658; retain coverage but separate current and compatibility contracts.
- `src/App.storage.test.tsx` — **393 lines — KEEP**. Focused storage-denial coverage at 265–393, with fake timers where needed.
- `src/App.v041Recovery.test.tsx` — **282 lines — FIX**. It is a live migration contract, not dead code; rename it to a semantic persistence-migration name and document the supported compatibility window.
- `src/V03EvidencePanel.preflight.test.tsx` — **395 lines — FIX**. The preflight/validator/staleness region spans 204–395; rename away from `V03` and replace partial `as never` fixtures with typed factories.
- `src/V03EvidencePanel.cancel.test.tsx` — **412 lines — FIX**. Cancellation scenarios span 299–412; preserve the lifecycle coverage, but rename the test with the product concept and use fake timers or deterministic deferred promises.
- `src/V03EvidencePanel.calibration.test.tsx` — **202 lines — FIX**. The active reduced-panel contract spans 139–202; the file name is historical and the test asserts retired strings plus real timers.
- `src/screens/AboutScreen.test.tsx` — **25 lines — KEEP**.
- `src/screens/DashboardScreen.test.tsx` — **108 lines — KEEP**.
- `src/screens/ProfileScreen.test.tsx` — **80 lines — KEEP**.
- `src/screens/RuntimeScreen.test.tsx` — **121 lines — KEEP**.
- `src/screens/TuneScreen.test.tsx` — **198 lines — KEEP**. The test file is larger than the component-specific screen tests but remains focused on disclosure, tuning readiness, and trial adoption.
- `index.html` — **21 lines — KEEP**. Correct root entry and product metadata only.
- `vite.config.ts` — **40 lines — FIX**. Test inclusion is correctly restricted, but line 6 suppresses a Node global type error.
- `tsconfig.json` — **25 lines — KEEP**. Strict/no-unused settings are appropriate.
- `tsconfig.node.json` — **10 lines — FIX**. It does not declare Node types, contributing to the Vite config suppression.
- `docs/DESIGN.md` — **581 lines — FIX**. It is the normative visual contract, but it does not name the active evidence panel and is not mechanically tied to the CSS token source.
- `docs/theme.css` — **106 lines — FIX**. It is a separate token syntax that is not imported by the application entry path.
- `docs/tokens.json` — **567 lines — FIX**. It duplicates the color/type source without a checked generation/link step.
- `.impeccable/config.json` — **3 lines — KEEP**.
- `.impeccable/design.json` — **420 lines — KEEP** as design metadata; establish whether it is generated or authoritative so it cannot silently diverge from `DESIGN.md` and `App.css`.

### Findings

#### Critical

##### FE-01 — Medium (lead; reviewer said Critical) — Credential drafts are retained in React state

> **Lead check: CORRECTED.** App.tsx:957 and :762 clear the drafts after a successful save. A draft stays only while the user types and after a failed save.

- **Location:** `src/App.tsx:273-274,307-308,757-764,949-958`; `src/screens/TuneScreen.tsx:79,270-273`; `src/screens/CatalogScreen.tsx:43,271-276`.
- **Evidence:**
  > `App.tsx:273-274` — `const [credential, setCredential] = useState<CredentialStatus | null>(null);` / `const [keyDraft, setKeyDraft] = useState("");`
  > `App.tsx:307-308` — `const [hfToken, setHfToken] = useState<TokenStatus>(...);` / `const [hfTokenDraft, setHfTokenDraft] = useState("");`
  > `TuneScreen.tsx:270-273` — the password input is controlled by `keyDraft` and the Store action reads it from that state.
- **Why it matters:** The project rule is stricter than “do not persist secrets”: credentials must never be in frontend state. A cloud API key and an HF token remain in React state and are propagated through parent props until a successful save clears them. The reviewed storage calls persist model roots, runtime/profile/report/settings data, and quarantine records; they do not persist these raw tokens, so the defect is in-memory state retention rather than localStorage at-rest exposure.
- **Fix:** Remove `keyDraft` and `hfTokenDraft` from `App` state and from screen props. Use a short-lived child-owned password input or a Rust-owned credential prompt, submit the value directly to the credential command, clear the DOM value in `finally`, and return only `CredentialStatus`/`TokenStatus` with masked suffixes.

#### High

##### FE-02 — High — Catalog query payload uses the wrong wire keys

> **Lead check: VERIFIED.** model.ts:1771-1774 uses snake_case keys. catalog.rs:771 expects camelCase with serde default. App.tsx:1360 and :1365-1366 send the ignored keys.

- **Location:** `src/model.ts:1762-1775`; `src/App.tsx:1354-1367`; `src-tauri/src/catalog.rs:770-798`.
- **Evidence:**
  > `model.ts:1771-1774` — `pipeline_tag?: string;` / `architecture?: string;` / `fit_per_mille?: number;` / `budget_bytes?: number;`
  > `catalog.rs:770-772` — `#[serde(rename_all = "camelCase", default)]` on `CatalogQuery`.
  > `App.tsx:1361-1366` — the request is constructed with `pipeline_tag`, `fit_per_mille`, and `budget_bytes`.
- **Why it matters:** Rust expects `pipelineTag`, `fitPerMille`, and `budgetBytes`. Because the Rust struct has `default`, the wrong keys are ignored rather than rejected: the pipeline filter and hardware-fit rule silently become unfiltered/disabled even while the UI says the fit filter is enabled. The catalog UI and tests therefore can look correct while the native filter receives defaults.
- **Fix:** Rename the TypeScript fields and the App payload to `pipelineTag`, `fitPerMille`, and `budgetBytes`; add a frontend IPC payload assertion that checks exact serialized keys. Keep the explicitly snake_case catalog-file fields only where Rust has an explicit serde name for signed catalog compatibility.

##### FE-03 — Low (lead; reviewer said High) — A maintained CDP gate searches for a button the current UI no longer renders

> **Lead check: CORRECTED.** g05_dc01.mjs has no caller. It is dead code, not a gate.

- **Location:** `scripts/g05_dc01.mjs:25-31`; `src/screens/CatalogScreen.tsx:139-152`.
- **Evidence:**
  > `g05_dc01.mjs:29` — `...includes("Refresh catalog")...`
  > `CatalogScreen.tsx:150-152` — the current button text is `Refresh list`.
- **Why it matters:** Any lane that runs `g05_dc01.mjs` cannot find the current refresh control. The direct `release.yml` block read here uses other generated gates and its own hard-coded selectors; whether `dc01` is active in the packaged matrix is UNKNOWN from `release.yml` alone. Either way, the repository contains a release-facing UI driver with an objectively stale selector, exactly the kind of coupling that makes release failures look like product failures.
- **Fix:** Replace text/class scraping with stable `data-testid`/role contracts (`catalog-refresh`, `runtime-path`, `inventory-row`, `notice`) and update every script in one pass. If text is retained as an accessibility assertion, assert it separately rather than using it as the click locator. Add a selector-contract test that compares all maintained gate IDs to rendered controls.

##### FE-04 — High — `App` is a controller, persistence service, async coordinator, and shell renderer in one component

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src/App.tsx:87-164,170-359,362-755,757-1028,1030-1239,1243-1501,1503-1858`.
- **Evidence:**
  > `App.tsx:170` — `function App() {` is followed by 89 state hooks and 30+ IPC/action functions before the render.
  > `App.tsx:1640-1814` — the render passes dozens of catalog, runtime, profile, tuning, credential, and evidence props through one shell.
  > `App.tsx:1531-1855` — the same function renders navigation, notices, every screen, and the permanently mounted benchmark surface.
- **Why it matters:** State transitions for unrelated resources share `busy`, notices, identity refs, effects, and closure lifetimes. A change to one feature requires reasoning about every other feature's render and async cleanup. This is the primary maintainability hazard and makes stale-response and release-contract regressions expensive to isolate.
- **Fix:** Extract controller hooks without adding another generic framework: `usePersistence` (87–129), `useInventoryController` (199–208, 362–416, 1150–1241), `useRuntimeController` (183–198, 260–261, 417–681), `useCatalogController` (280–316, 683–856, 1336–1389), `useCloudCredentials` (262–279, 872–1028), `useTuningController` (317–354, 1030–1230, 1449–1473), and `useServerController` (203–219, 1243–1334, 1475–1501). Leave `AppShell` with navigation, the notice strip, and screen composition.

##### FE-05 — High — Legacy benchmark and “v2” evidence are two active product systems, not one migrated feature

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src/App.tsx:173-177,259,1301-1333`; `src/screens/BenchmarkScreen.tsx:43-86`; `src/V03EvidencePanel.tsx:126,572-580`; `src/App.staleResponses.test.tsx:379-492`; `src/model.ts:1102-1110`.
- **Evidence:**
  > `BenchmarkScreen.tsx:48-50` — the legacy `Run benchmark` action remains in the same screen.
  > `BenchmarkScreen.tsx:79-86` — the same screen mounts `V03EvidencePanel` beside it.
  > `App.tsx:174-177` — separate `panelEvidenceRun`, `legacyEvidenceRun`, and `legacyBenchmarkActive` state are still live.
- **Why it matters:** `benchmark_server`/`BenchmarkSummary` and `benchmark_v2`/`BenchmarkRunResult` have separate state, cancellation, persistence, and UI contracts. The shared screen and global `busy`/evidence banner make it possible for one run owner to obscure or disable the other. The historical names are not evidence of dead code: the current UI renders `Benchmark v2`, `V03EvidencePanel` is imported, and the legacy path has integration tests.
- **Fix:** Decide which benchmark contract is the product contract. Migrate the surviving manifest/evidence behavior into one semantic `EvidencePanel`, then remove the legacy command/state only after the compatibility/release matrix no longer depends on it. Rename `V03EvidencePanel.*` tests and `App.v041Recovery.test.tsx` to semantic names while retaining migration coverage for the documented support window.

##### FE-06 — High — The presentation boundary is contradicted by a direct screen IPC call and untyped adapter calls

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src/screens/InventoryScreen.tsx:1-13,51-58`; `src/evidence-adapter.ts:18-43`.
- **Evidence:**
  > `InventoryScreen.tsx:11-13` — the contract says “Acquisition ... stays in `App.tsx`”.
  > `InventoryScreen.tsx:2,55` — the screen imports `invoke` and calls `invoke("cancel_scan")` itself.
  > `evidence-adapter.ts:36-43` — all six production adapter calls use `invoke(...)` without explicit result generics.
- **Why it matters:** The screen can bypass App-level cancellation/error policy, and the adapter interface can drift from Rust without a visible command/result annotation at the boundary. The command currently exists and is registered, but the ownership rule is already broken; future cancellation changes will have two acquisition paths.
- **Fix:** Pass `cancelScan` from App as a typed callback and remove the Tauri import from `InventoryScreen`. Add explicit `invoke<ResultType>` generics in the evidence adapter for hardware, artifact, preflight, benchmark, cancellation, and replay; keep the interface types as the compile-time contract.

#### Medium

##### FE-07 — Medium — A hooks lint suppression hides a callback dependency in the run-ownership effect

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src/V03EvidencePanel.tsx:227-238`.
- **Evidence:**
  > `V03EvidencePanel.tsx:230-237` — the effect calls `onRunStateChange` and `cancelBenchmark`, but its dependency list is only `[busy]` and suppresses `react-hooks/exhaustive-deps`.
- **Why it matters:** The current App passes a stable state setter, but the component's public prop is not guaranteed stable. A consumer that replaces `onRunStateChange` can leave an old callback installed until `busy` changes. The suppression also removes the linter's only automatic warning when cancellation ownership changes.
- **Fix:** Keep the latest callback in a ref or make the callback stable with a narrow hook, include the actual dependencies, and remove the suppression. Add a test that rerenders with a new owner callback while a benchmark is active.

##### FE-08 — Medium — Error-boundary reset can fail in the same storage failure mode it is meant to recover

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src/ErrorBoundary.tsx:8-21,49-52`.
- **Evidence:**
  > `ErrorBoundary.tsx:8-20` — `clearApplicationStorage` enumerates and removes `localStorage` keys without a `try/catch`.
  > `ErrorBoundary.tsx:49-52` — the reset button calls it before `window.location.reload()`.
- **Why it matters:** If storage access is denied or throws, the click handler aborts before reload and the user remains on the error screen. The application already tests denied storage in `src/App.storage.test.tsx:265-393`, but the boundary's recovery path is not protected by the same policy.
- **Fix:** Wrap enumeration/removal in `try/finally` and always attempt reload; report that storage could not be cleared rather than throwing a second UI error. Add a boundary test with `Storage.prototype.length/key/removeItem` throwing.

##### FE-09 — Medium — The flat design rule still has two box shadows

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src/App.css:43-44`; `docs/DESIGN.md:557-581`.
- **Evidence:**
  > `App.css:43-44` — `.signal` has an inset `box-shadow`; `.signal.live` has a glow `box-shadow`.
  > `DESIGN.md:571` — `Don't add box-shadow ... the cabinet is flat.`
- **Why it matters:** The application stylesheet is not flat according to the project rule. `border-radius: 50%` at `App.css:43,51` is limited to the two lamp indicators, and rectangular inputs explicitly use `border-radius: 0` at 268; those are not the defect. The signal glow may be intentional, but no documented exception permits a shadow and the requested review explicitly calls out shadow compliance.
- **Fix:** Remove both shadows and use the existing border/background lamp treatment, or document and mechanically exempt a lamp-only rule if the product owner approves that exception. Keep the existing `prefers-reduced-motion` treatment at 522–524.

##### FE-10 — Medium — Type assertions bypass real IPC/domain shapes in production and fixtures

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src/App.tsx:135`; `src/V03EvidencePanel.tsx:73-78`; test fixtures including `src/V03EvidencePanel.calibration.test.tsx:112-124`, `src/V03EvidencePanel.preflight.test.tsx:188-198`.
- **Evidence:**
  > `App.tsx:135` — `window` is coerced through `unknown` to detect `__TAURI_INTERNALS__`.
  > `V03EvidencePanel.tsx:73-78` — a complete `LaunchProfile` is coerced into `Record<string, unknown>` for fingerprinting.
  > `V03EvidencePanel.calibration.test.tsx:112-124` — partial model/profile/server fixtures are accepted with `as unknown as`/`as never`.
- **Why it matters:** The scan found 18 `as unknown as` occurrences and 12 `as never` occurrences in `src`; there is no actual TypeScript `any` type. The production assertions hide structural drift, while partial component fixtures can continue passing after required IPC fields change. The casts are concentrated around the exact evidence and IPC surfaces this review needs to trust.
- **Fix:** Add a typed `Window` augmentation for the Tauri marker, use an explicit ordered profile-key list or typed object projection for fingerprints, and build complete fixture factories from the mirrored types. Reserve casts for a narrow, commented boundary with runtime validation.

##### FE-11 — Medium — Tests couple to DOM classes/text and use real timers for debounce/async flushing

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src/App.catalog.test.tsx:177-192,386-450,1420-1463`; `src/V03EvidencePanel.calibration.test.tsx:75-95,148-165`; `src/V03EvidencePanel.preflight.test.tsx:116-136,340-392`.
- **Evidence:**
  > `App.catalog.test.tsx:1430-1432` — the test asserts `button.className` contains `secondary`.
  > `V03EvidencePanel.preflight.test.tsx:383-392` — the test asserts `strong.className` contains `tone-pending`/does not contain `tone-ok`.
  > `App.catalog.test.tsx:1458-1463` — the filter test waits 12ms and 350ms with real `setTimeout` calls; the file does not enable fake timers.
- **Why it matters:** Exact rendered copy is a release-gate contract in this project, but CSS class spelling is implementation detail. A harmless visual refactor can break tests, while a behavior regression can be hidden behind broad `textContent` checks. Real debounce waits make the catalog suite slower and timing-sensitive; the V03 files use real zero-delay timers for flushes as well. The tests do not read source files, but they do assert many DOM selectors/classes and UI strings.
- **Fix:** Assert roles, accessible names, disabled state, emitted IPC payloads, and visible status semantics. Add stable `data-testid` only for controls without a semantic target. Use fake timers for the 120ms debounce and deterministic deferred promises for adapter responses; retain a small number of real integration waits only where the browser event loop itself is under test.

#### Low

##### FE-12 — Low — Vite config suppresses a Node type error instead of fixing the config contract

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `vite.config.ts:1-7`; `tsconfig.node.json:1-10`.
- **Evidence:**
  > `vite.config.ts:6-7` — `// @ts-expect-error process is a nodejs global` immediately before `process.env.TAURI_DEV_HOST`.
  > `tsconfig.node.json:4-9` — the Node config has no `types: ["node"]` declaration.
- **Why it matters:** The repository has one `@ts-` suppression even though the application source has none. The directive makes the config compile only while the expected error remains and hides the actual environment contract from readers and tooling.
- **Fix:** Import the required Node process API or add the correct Node type package/config to `tsconfig.node.json`, then remove the suppression. Keep the existing `strict` application config unchanged.

##### FE-13 — Low — Design tokens and component documentation are parallel, unenforced sources of truth

> **Lead check: UNVERIFIED.** Reviewer report. The lead reviewer did not check it.

- **Location:** `src/main.tsx:1-5`; `src/App.tsx:7`; `src/App.css:1-19`; `docs/theme.css:1-8`; `docs/tokens.json:1-14`; `docs/DESIGN.md:153-340`.
- **Evidence:**
  > `main.tsx:1-5` and `App.tsx:7` — the runtime entry imports `App.css`; neither imports `docs/theme.css` or `docs/tokens.json`.
  > `App.css:1-19` and `docs/theme.css:1-8` — the product CSS and documented theme define separate variable naming layers for the same palette.
  > `DESIGN.md:153-340` — documented components do not name the active `V03EvidencePanel`/evidence-panel component.
- **Why it matters:** The values currently match for the primary palette, but there is no checked linkage preventing future drift. New evidence surfaces can be added without a corresponding normative component entry, while design lint cannot verify the product CSS against the JSON/theme artifacts from the files read here.
- **Fix:** Choose one canonical token source, generate or import the runtime CSS from it, and add the evidence panel/benchmark component to `DESIGN.md`. If the theme and JSON are generated artifacts, state that and add a read-only consistency check rather than maintaining three hand-edited sources.

### Answers to specific questions

#### 1. `App.tsx` hook census, responsibilities, and split

Static count from the complete file:

- `useState`: **90 textual occurrences**, including the import; **89 hook calls**.
- `useEffect`: **17 textual occurrences**, including the import; **16 hook calls**.
- `useRef`: **15 textual occurrences**, including the import; **14 hook calls**.

The old simple-parenthesis count undercounts generic/multiline `useState` and `useRef` calls. The counts above exclude only the import occurrence and include generic calls.

The god-component responsibilities are separated by these actual ranges:

- Persistence and browser migration helpers: `App.tsx:87-129`.
- All application state and identity refs: `App.tsx:170-359`.
- Inventory scan, selection, and profile loading: `App.tsx:362-416,1150-1241`.
- Runtime inspection, managed install, health, and repair: `App.tsx:417-681`.
- Catalog load, facets, downloads, and cancellation: `App.tsx:683-856,1336-1389`.
- Cloud providers, credential operations, and probes: `App.tsx:872-1028`.
- Tuning, reports, trial application, and progress listeners: `App.tsx:1030-1230,1459-1473`.
- Legacy benchmark and server lifecycle: `App.tsx:1243-1334,1475-1501`.
- Shell, notices, navigation, and screen composition: `App.tsx:1503-1858`.

Recommended extraction is listed in FE-04. The boundary should be hooks/controllers by resource, not a new generic state framework.

#### 2. Every frontend `invoke(...)` and Rust command match

Static extraction found **51 invocation sites and 50 unique command names** in production frontend files. Every unique name matched a `#[tauri::command]` and was present in the `generate_handler!` list at `src-tauri/src/lib.rs:2262-2324`. No frontend command name was unmatched.

`[untyped]` means the call has no explicit `invoke<ResultType>` generic; its surrounding interface may provide contextual typing, but the IPC result contract is not visible at the call site.

| Frontend site | Command | Result type / backend declaration |
|---|---|---|
| `src/App.tsx:369` | `scan_models_report` | `ScanReport` / `lib.rs:1427` |
| `src/App.tsx:427` | `inspect_runtime` | `RuntimeCapabilities` / `lib.rs:1469` |
| `src/App.tsx:428` | `describe_runtime` | `RuntimeIdentity` / `lib.rs:1506` |
| `src/App.tsx:429` | `list_managed_runtimes` | `ManagedRuntimeRecord[]` / `lib.rs:1514` |
| `src/App.tsx:472` | `load_runtime_setup` | `RuntimeSetupResponse` / `runtime_service.rs:13` |
| `src/App.tsx:550` | `fetch_runtime_catalog` | `RuntimeCatalog` / `runtime_service.rs:80` |
| `src/App.tsx:578` | `install_managed_runtime` | `InstalledRuntime` / `runtime_service.rs:125` |
| `src/App.tsx:595` | `cancel_managed_runtime_install` | `boolean` / `runtime_service.rs:150` |
| `src/App.tsx:610` | `check_managed_runtime_health` | `ManagedHealthResult` / `runtime_service.rs:161` |
| `src/App.tsx:632` | `cancel_managed_runtime_health` | `boolean` / `runtime_service.rs:263` |
| `src/App.tsx:649` | `repair_health_model` | `[untyped]`, backend `()` / `runtime_service.rs:212` |
| `src/App.tsx:660` | `cancel_health_model_repair` | `[untyped]`, backend `boolean` / `runtime_service.rs:249` |
| `src/App.tsx:691` | `load_model_catalog` | `CatalogSnapshot` / `catalog_service.rs:29` |
| `src/App.tsx:700` | `fetch_model_catalog` | `CatalogSnapshot` / `catalog_service.rs:59` |
| `src/App.tsx:709` | `catalog_local_models` | `CatalogModel[]` / `catalog_service.rs:157` |
| `src/App.tsx:712` | `catalog_facets` | `[string[], string[]]` / `catalog_service.rs:235` |
| `src/App.tsx:715` | `catalog_rich_facets` | `CatalogFacets` / `catalog_service.rs:245` |
| `src/App.tsx:718` | `hf_token_status` | `TokenStatus` / `catalog_service.rs:270` |
| `src/App.tsx:761` | `save_hf_token` | `TokenStatus` / `catalog_service.rs:275` |
| `src/App.tsx:775` | `clear_hf_token` | `TokenStatus` / `catalog_service.rs:280` |
| `src/App.tsx:816` | `download_catalog_file` | `string` / `lib.rs:2063` |
| `src/App.tsx:845` | `cancel_download` | `boolean` / `lib.rs:2190` |
| `src/App.tsx:877` | `cloud_providers` | `CloudProvider[]` / `lib.rs:1845` |
| `src/App.tsx:880` | `cloud_credential_status` | `CredentialStatus` / `lib.rs:1850` |
| `src/App.tsx:888` | `cloud_list_models` | `CloudModel[]` / `lib.rs:1868` |
| `src/App.tsx:955` | `cloud_save_credential` | `CredentialStatus` / `lib.rs:1855` |
| `src/App.tsx:973` | `cloud_clear_credential` | `CredentialStatus` / `lib.rs:1863` |
| `src/App.tsx:991` | `cloud_openrouter_login` | `CredentialStatus` / `lib.rs:1889` |
| `src/App.tsx:1011` | `cloud_probe` | `string` / `lib.rs:1877` |
| `src/App.tsx:1041` | `read_gguf_summary` | `GgufSummary` / `lib.rs:1519` |
| `src/App.tsx:1055` | `tune_disclosure_list` | `DisclosureSection[]` / `tune_service.rs:224` |
| `src/App.tsx:1096` | `start_tuning` | `TuningReport` / `tune_service.rs:229` |
| `src/App.tsx:1131` | `cancel_tuning` | `boolean` / `tune_service.rs:375` |
| `src/App.tsx:1174` | `apply_tuning_trial_changes` | `LaunchProfile` / `tune_service.rs:410` |
| `src/App.tsx:1215` | `suggest_port` | `number` / `lib.rs:1911` |
| `src/App.tsx:1253` | `preview_command` | `CommandPreview` / `lib.rs:1790` |
| `src/App.tsx:1277` | `start_server` | `ServerStatus` / `server_service.rs:21` |
| `src/App.tsx:1292` | `stop_server` | `ServerStatus` / `server_service.rs:170` |
| `src/App.tsx:1309` | `cancel_benchmark` | `void` / `measurement_service.rs:737` |
| `src/App.tsx:1316` | `benchmark_server` | `BenchmarkSummary` / `measurement_service.rs:53` |
| `src/App.tsx:1368` | `filter_catalog` | `CatalogModel[]` / `catalog_service.rs:224` |
| `src/App.tsx:1420` | `about_info` | `AboutInfo` / `lib.rs:2219` |
| `src/App.tsx:1484` | `server_status` | `ServerStatus` / `server_service.rs:276` |
| `src/App.tsx:1488` | `read_server_log` | `string` / `server_service.rs:327` |
| `src/evidence-adapter.ts:37` | `detect_hardware` | `[untyped]`, interface `HardwareInfo` / `runtime_service.rs:68` |
| `src/evidence-adapter.ts:38` | `inspect_model_artifact` | `[untyped]`, interface `ArtifactInspection` / `lib.rs:1558` |
| `src/evidence-adapter.ts:39` | `preflight_model` | `[untyped]`, interface `PreflightResult` / `lib.rs:1609` |
| `src/evidence-adapter.ts:40` | `benchmark_v2` | `[untyped]`, interface `BenchmarkRunResult` / `measurement_service.rs:604` |
| `src/evidence-adapter.ts:41` | `cancel_benchmark` | `[untyped]`, interface `void` / `measurement_service.rs:737` |
| `src/evidence-adapter.ts:43` | `replay_benchmark_manifest` | `[untyped]`, interface `Workload` / `measurement_service.rs:752` |
| `src/screens/InventoryScreen.tsx:55` | `cancel_scan` | `[untyped]`, backend `boolean` / `lib.rs:1455` |

The explicit generic result types reviewed match the corresponding Rust return structs/enums. The exception is FE-02, which is a request-payload field-name mismatch rather than a return-type mismatch. The nine untyped calls are UNKNOWN for compile-time drift until explicit generics are added.

#### 3. Unsafe/type-suppression/storage grep

- **`any`:** 21 whole-word matches, all comments or the intentional string literal in `profileNumberLimits` (`"any"`); no `: any`, `Array<any>`, or `as any` type was found.
- **`as unknown as`:** 18 occurrences. Production: `src/App.tsx:135` and `src/V03EvidencePanel.tsx:74`; the other 16 are test fixtures/setup. See FE-10.
- **`as never`:** 12 occurrences, all in tests/model fixtures; these are additional type-contract bypasses not covered by the literal `as unknown as` search.
- **`// @ts-`:** none in `src`; one `// @ts-expect-error` at `vite.config.ts:6`. See FE-12.
- **`eslint-disable`:** one occurrence at `src/V03EvidencePanel.tsx:237`, suppressing `react-hooks/exhaustive-deps`. See FE-07.
- **`sessionStorage`:** no occurrences.
- **`localStorage`:** production reads/writes are in `src/App.tsx:87-121` and `src/ErrorBoundary.tsx:8-20`. Stored values are model-root/runtime paths, cloud-provider/model selections, disclosure choice, serialized profiles, tuning reports, benchmark summaries, and quarantined malformed records. The old storage-key prefix of the former product name is read for migration. Raw cloud/HF secrets are not written to localStorage; they are nevertheless held in React state (FE-01).
- **Non-null assertions on production values:** `src/App.tsx:1464` asserts `event.payload.trial!` after a truthiness check; `src/screens/CatalogScreen.tsx:241` asserts `activeJob!` after a same-render `running` check; `src/screens/TuneScreen.tsx:330` asserts the selected tuning origin after a truthiness check. The latter two are locally guarded; the first is IPC event data and should be narrowed once into a local constant rather than asserted twice.

#### 4. Version/campaign-named code

- `V03EvidencePanel` is active product code, imported and rendered by `BenchmarkScreen.tsx:5,79`; its visible panel heading is `Benchmark v2` at `V03EvidencePanel.tsx:572`. It is not dead. The name is historical and should become a semantic name after the benchmark migration decision.
- `App.v041Recovery.test.tsx` is an active migration test. Its header explicitly says it recovers v0.4.1 persisted values and its tests at 201–257 seed/read those records. It should not be deleted until the supported upgrade window ends; rename it to a semantic persistence-migration test and keep the old key fixtures as compatibility evidence.
- `Benchmark v2` is visible product UI, not a dead label; it is backed by `benchmark_v2` and three component test files. The legacy `Run benchmark` path is also active through `App.tsx:1301-1333` and `benchmark_server`.
- `legacy` appears in active source, tests, and compatibility adapters. The appropriate action is consolidate/rename, not blanket deletion. FE-05 covers the duplicate benchmark ownership.

#### 5. CSS versus `DESIGN.md`

- **Box shadow:** two uses at `src/App.css:43-44`; `DESIGN.md:571` forbids shadows for the flat cabinet. This is FE-09.
- **Border radius:** the only nonzero values are `border-radius: 50%` on the two lamp indicators at 43 and 51; rectangular inputs explicitly use `border-radius: 0` at 268. This matches the square-corners rule with lamp exceptions.
- **Signal colors:** `App.css:12-17` uses primary `#9edc72`, amber `#f0b95c`, and red `#ffb1aa`, matching `DESIGN.md:6-12` (red uses the documented lit tertiary). Red/green/amber are used for status and actions rather than a fourth signal color. Screen-area percentage is UNKNOWN without rendering; the red tint at `App.css:194` is `.06` alpha, below the approximate 10% guidance.
- **Motion:** `.spin` is the single work indicator at `App.css:400`; reduced motion collapses animation at 522–524, matching `DESIGN.md:579`.
- **Mobile constraints:** fixed bottom navigation, 44px inputs/toggles, 46px buttons, and 110px screen bottom padding are explicit at `App.css:456-471` and 465. Static CSS supports the documented target; visual overlap at every viewport is UNKNOWN without packaged rendering.
- **Documentation gap:** the active evidence panel is not named in `DESIGN.md`; the generic evidence/layout prose is not a component entry.

#### 6. Test quality, duplication, timers, and `model.test.ts`

Implementation-coupled assertions include:

- `App.catalog.test.tsx` uses broad `querySelector`/`textContent` scraping throughout, exact CSS selectors such as `.runtime-loading`, `.runtime-catalog-message.error`, `.inventory-table`, `.row-target`, `.advanced-zone`, `.catalog-screen`, `.path-text-value`, and a direct `button.className` assertion at 1430–1432.
- `V03EvidencePanel.preflight.test.tsx:383-392` checks `className` tone tokens directly.
- `V03EvidencePanel.calibration.test.tsx:148-165` asserts the absence of a list of retired button strings and a retired textarea class. That is a migration guard, not a stable behavioral contract.
- No test reads `App.tsx`/`App.css` source text; the coupling is to rendered text, DOM structure, CSS class names, and accessibility attributes.

Real timers without `useFakeTimers` in the same file:

- `App.catalog.test.tsx:1458,1463` waits 12ms and 350ms for the catalog debounce/async work.
- `V03EvidencePanel.calibration.test.tsx:91-95`, `V03EvidencePanel.cancel.test.tsx:186`, and `V03EvidencePanel.preflight.test.tsx:116,340` use real zero-delay timers as flush helpers.
- `App.staleResponses.test.tsx:159` and `App.storage.test.tsx:336` do use fake timers, so those two are not in this specific timer defect.

Overlapping coverage is intentional in places but needs pruning/renaming:

- `model.test.ts:324-365` unit-tests workload validation while `V03EvidencePanel.preflight.test.tsx:204-291` verifies the same validator through UI dispatch.
- `model.test.ts:1074-1085` tests evidence tones while `V03EvidencePanel.preflight.test.tsx:380-395` checks rendered tone classes.
- `model.test.ts:492-504,861-927` tests pure stale-response guards while `App.staleResponses.test.tsx:271-378,551-658` verifies late UI responses.
- `App.catalog.test.tsx:594-687` covers quarantine/error-boundary persistence while `App.storage.test.tsx:265-393` covers denied storage; these are complementary failure modes, not exact duplicates.
- `App.staleResponses.test.tsx:379-492` and `BenchmarkScreen.tsx:43-78` preserve the legacy benchmark path; this is the largest historical surface that should disappear only after migration.

`src/model.test.ts` is **1,240 lines**. It covers legacy/tuning input, runtime catalog/health, launch evidence, v0.3 evidence/artifact contracts, hardware budget, stale guards, profile normalization/suggestion, runtime option state, downloads, progress/origin, catalog fit, extra-argument parsing, persistence normalization, evidence tone, sample caveats, companion reconciliation, and a 300-generated-input property campaign. The breadth is useful, but the file is now a second index of product history and should be shrunk around current pure decisions while integration tests own rendered behavior.

#### 7. Load-bearing release-gate UI strings and selector recommendation

The direct UI selectors in `.github/workflows/release.yml` are at 352–356, 374, 394, and 409–445. They currently rely on:

| Selector/string | Current source | Current UI evidence | Risk/recommendation |
|---|---|---|---|
| `Runtime` nav | `release.yml:352`; also many `scripts/*.mjs` | `App.tsx:1507` | Use `data-testid="nav-runtime"` plus accessible name assertion. |
| `Path to llama-server.exe` placeholder | `release.yml:352-354` | `RuntimeScreen.tsx:338-343` | Use `data-testid="runtime-path"`; keep placeholder as human help, not automation identity. |
| `Inspect` button | `release.yml:356` | `RuntimeScreen.tsx:343-347` | Use `data-testid="inspect-runtime"`. |
| `Inventory` nav, `Model root` aria-label, `Rescan` | `release.yml:374` | `InventoryScreen.tsx:45-63` | Keep the accessible label; add stable nav/action IDs. |
| `.button.row-target` | `release.yml:374` | `InventoryScreen.tsx:106-110` | Replace CSS class dependency with `data-testid="inventory-row-target"` or a semantic row button query. |
| `Profile`, `SSL private key`, `SSL certificate`, `API key file`, `Save` | `release.yml:394` | `ProfileScreen.tsx:79,292-300` | Add field IDs and form submit ID; do not locate exact label text by DOM traversal. |
| `Start` and `Started .* on port \\d+` | `release.yml:409-416` | `ProfileScreen.tsx:79-80`; `App.tsx:1277-1280` | Use a start action ID and a structured status attribute/IPC result; retain text as a human-readable assertion. |
| `.notice-line` | `release.yml:409,438` and many `g05_*` scripts | `App.tsx:1580-1593` | Add `data-testid="system-notice"` and expose structured status code/result for automation. |
| `Control` and `Stop server` | `release.yml:438-445`; `g05_churn_repro`, `g05_stop_supervision`, `g05_tamper_dll` | `App.tsx:1505`; `DashboardScreen.tsx:58-60` | Use nav/action IDs; distinguish `Cancel start` from `Stop server` with a status attribute. |
| `HF Catalog`, `Choose where`, `Download` | `g05_partial_retention.mjs:39,47,61`; `verify_060_catalog.mjs:226-403` | `CatalogScreen.tsx:142,158,237-245` | Add catalog destination/download IDs. |
| `Refresh catalog` | `g05_dc01.mjs:29` | **No current match**; UI is `Refresh list` at `CatalogScreen.tsx:150-152` | Update the gate immediately; this is the known stale selector in FE-03. |
| `Benchmark`, `Run benchmark`, `Run v2 benchmark`, `Cancel` | `g05_run_cancel.mjs`, `g05_v2_cancel.mjs`, `g05_mt01d.mjs` | `BenchmarkScreen.tsx:48-50`; `V03EvidencePanel.tsx:580`; cancellation controls in the panel | Add IDs for legacy/v2 only while both exist; remove the legacy selector set after migration. |
| `Run 7-stage health`, `Check latest release`, managed runtime labels | `g05_health.mjs:14-43`, `verify_s27_runtime.mjs:27-64`, `verify_g05_managed_install.mjs` | `RuntimeScreen.tsx:353-386` and runtime setup sections | Use structured state/data attributes; keep text checks as secondary copy/a11y checks. |

`release.yml` currently finds the exact `Start` on the Profile screen, not Dashboard's `Start profile`; that specific path is internally consistent. The remaining contract is still brittle because all of these checks depend on mutable text, placeholders, CSS class names, or broad `.textContent` scans. IPC-level checks should verify command payload/result contracts and structured status where the check does not need to prove the UI affordance itself; packaged CDP should use stable IDs for the affordance proof.

### Cross-cutting observations

- **Backend command names:** no unmatched frontend command name was found. Registration is also present in `lib.rs:2262-2324`.
- **Real contract defect:** `CatalogQuery` field names are wrong on the wire (FE-02); the explicit TypeScript return types otherwise match the Rust structs reviewed.
- **Secrets:** no raw cloud/HF token is stored in localStorage/sessionStorage, but both are retained in React state and controlled password inputs (FE-01). Profile fields store file paths, not file contents.
- **Type suppression:** no actual `any` type; one Vite `@ts-expect-error`, one hooks lint suppression, 18 `as unknown as`, and 12 `as never` occurrences.
- **Design:** color tokens, mobile target sizes, bottom clearance, square rectangles, reduced motion, and status words are mostly explicit. The box shadows and undocumented active evidence component are the remaining static design violations/gaps.
- **Verification status:** tests, type checking, production build, packaged CDP, and release gates were not executed under the read-only task restriction. Their runtime status is UNKNOWN; this report does not claim them passing.

---
