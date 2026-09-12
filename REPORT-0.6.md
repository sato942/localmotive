# Localmotive 0.6.0 remediation report

**Snapshot:** 2026-09-12. Local branch `main` at `b80f56a` (docs/evidence tip);
candidate code freeze at `57bde64e` (`fix(local-client): stop discarding
slow-but-healthy cancellable responses (MT-06)`).
**Baseline:** `v0.5.0` (source `a4b7127f739f7420232d9b6f63da693d39128d0b`).
**Range covered:** every commit between `v0.5.0` and HEAD - 173 commits, 199
files changed, +48 190 / -5 206 lines.
**Push state:** the only commit on `origin/main` beyond `v0.5.0` is `e530371`
("docs(0.5): Phase 8 ship ledger"); the other 172 commits are local only and
have not been pushed. This is deliberate: the release flow plants a tag at a
resolved SHA, and the tag/publish steps are owner-gated (section 6).

This report compares the work against the authoritative tracker
[`docs/history/TODO-0.6.md`](docs/history/TODO-0.6.md), whose evidence source is
[`docs/history/localmotive-comprehensive-audit.md`](docs/history/localmotive-comprehensive-audit.md)
(audit SHA-256 `fb87c9df9fd4cffa3768b81ddd40fd55363e718456a182b4237c1bfc9054b647`).
Detailed per-item dispositions live in
[`docs/RELEASE-REVIEW-0.6.md`](docs/RELEASE-REVIEW-0.6.md); committed release
evidence lives in `release-evidence/0.6.0/`; the working logs of the final
re-bind live in the gitignored `.hermes-0.6/` scratch directory and are named
throughout this report.

---

## 1. What the plan asked for

The audit produced **72 findings** (19 High, 43 Medium, 10 Low), **29
supplemental packages** (S-01..S-29) for unnumbered recommendations, and **10
verification and release gates** (G-01..G-10). The tracker translated them into
**111 packages**, **663 checkboxes**, and **1051 audit links**. Its default
release policy: all 19 High findings block the stabilization release until fixed
and verified; Medium/Low items may be deferred only with a visible disposition
carrying owner, reason, residual risk, workaround, follow-up milestone, and
evidence gap.

## 2. Where 0.6.0 stands against TODO-0.6

| Tracker element | Planned | Actual at HEAD |
|---|---|---|
| Packages 1-12 | Sequenced remediation of all 72 findings | All closed; every finding's implementation landed with regression tests and ledger records |
| Supplemental S-01..S-29 | Unnumbered audit recommendations | 28 closed with tests/evidence; S-25.I3 remains an explicit hardware-program deferral |
| Gates G-01..G-04, G-06, G-07 | Candidate identity, verification wave, advisories | Closed with evidence |
| Gate G-05 | Packaged Windows wave | Executed in seven batches, a High-finding campaign, and two final-candidate re-binds (2026-09-12); only the environment-blocked I3 residuals remain |
| Gate G-08 | Release decision (support matrix, disclosure, deferral review) | Closed: review + support matrix + unsigned disclosure + reconciliation; all deferrals recorded |
| Gates G-09, G-10 | Authorized publication and closeout | **Open - owner-gated** (section 6) |
| Checkboxes | 663 | 639 checked, 24 open - every open item has a written disposition |

Test suites grew from the v0.5.0-era baseline (Rust 403 tests; Vitest 80) to,
at the final revision:

- **Rust: 593 passed / 0 failed / 7 ignored** (`cargo test`, `.hermes-0.6/fix-cargo-gates2.log`),
  with `cargo fmt --check` clean and `clippy --all-targets -- -D warnings`
  reporting zero warnings at every package closure.
- **Vitest: 141 passed / 141** (`npx vitest run`, `.hermes-0.6/final-gates-7dcc34f.log`).
- **Node suites: 154 passed / 0 failed** (`npm test`, measured 2026-09-12):
  release gates 119, catalog schema, HTTP retry contract, IPC contract, and the
  QD-03 health-cancellation fixtures 6.
- `npm test` is exactly `vitest run && node --test scripts/tests/release-gates.test.mjs
  scripts/tests/catalog_schema.test.mjs scripts/tests/http_retry.test.mjs
  scripts/tests/ipc_contract.test.mjs scripts/tests/health_cancel.test.mjs`,
  so one command covers everything above.

## 3. The work between v0.5.0 and HEAD

Commit composition since `v0.5.0` (173 commits): 62 `docs`, 60 `fix`, 21 `test`,
16 `feat`, 9 `refactor`, 2 `release`, 2 `ci`, 1 `perf`. Commits touching each
area: 75 `src-tauri/src`, 51 `src/` (frontend), 58 `scripts/` (verifiers,
fixtures, probes), 5 `.github/` (workflows).

### 3.1 Rust core (`src-tauri/src`)

- **Runtime trust and lifecycle** (RT): new installs publish into the primary
  `Localmotive` root while legacy `GGUF Pilot` installs are discovered and reused
  only after compiled-content verification; a centralized managed-execution
  authorization gate with a sentinel test; archive extraction bound to the
  approved file identity through a single handle; an execution-identity lease so
  a replaced runtime cannot serve a launch validated against older bytes;
  bounded `runtime.json`/`--help`/`--version` reads; hard-link refusal; Windows
  last-error capture; per-physical-GPU identity parsing. The final window added
  `rt07_path_and_reparse`: a junction/reparse swap after archive open and a
  symlink payload are both refused, with a caught mutation (MR1).
- **Catalog** (DC): signature and freshness policy, offline-first loading with
  `last_success_secs`/`refreshError` in the snapshot, stale fallback on
  tamper/truncation/parse failure, SQLite mirror quarantine and rebuild under
  corruption and NTFS sharing violations, database-authority downloads,
  bounded IPC payloads, catalog-drop visibility, immutable revision pinning,
  schema contract shared between the JS validator and the Rust parser
  (18-case fixture), quant-label extraction from the file's own token.
- **Downloads**: resumable parallel transfers with a one-writer job reservation
  across processes, no-replace publication, `.part`/sidecar retention across
  cancellation, sidecar-metered resume, cancellation plumbing, redirect policy
  (HTTPS-only hosts, bounded hops), HTTP validator semantics (strong ETag
  gating, bounded `Retry-After`), and a measured shared write-mutex path
  (2 363.8 MB/s aggregate at 4 workers - no positional-write optimization
  warranted).
- **Measurement, evidence, tuning** (MT): versioned execution snapshots
  (`ExecutionSnapshotV2`), run-derived calibration anchors, launch-scope quality
  binding, complete-set ranking dominance with coverage weighting and bounded
  dominators, discovery parity through `analyze_shard_names`, calibration TTLs,
  centralized observation-consistency validation, workload honesty factory +
  validation, bounded proposal loops with rejection limits, cancellable
  measurement and supervised completion, trial-outcome cleanup enforcement
  (a measured result is *withheld* when its server cannot be stopped).
- **Transport** (MT-06): one Rust-owned local HTTP client - TLS pinning,
  API-key files, loopback-only hosts, bounded bodies, deadlines, cancellation -
  threaded through measurement, health, and the app, plus a fail-fast TLS
  verdict with an exact recovery instruction. The final window rewrote the
  cancellable path (section 5.1): requests now run on a worker thread with
  their full deadline while the caller observes the cancel flag in 500 ms
  slices.
- **Operations**: bounded per-run launch logs (8 MiB cap, newest-ten retention,
  a per-run `.log.failure.json` failure record with `phase`, `exitCode`,
  `logTail`), bounded OAuth callback (loopback-only, 8 KiB cap, single-use
  state), background cancellable startup, single managed-inference owner
  (`OperationCoordinator`) with generation discard, supervised process
  termination, per-child `KILL_ON_JOB_CLOSE` job objects.
- **Parsers**: bounded GGUF reader with an explicit `Limits` seam, optional
  split metadata, impossible KV-dimension rejection, 5-way deterministic
  property campaigns (splitmix64).

### 3.2 Frontend (`src/`)

- Single-flight status, server-snapshot identity, selection/identity
  preservation across rescans, run generations, cancellation pending states,
  cancel bound to its job, raw extra-args with validation, corrupt-persisted-JSON
  quarantine with an error boundary, honest empty/first-run states, path
  truncation with copy, product-name demotions, preflight staleness by input
  revision, workload defaults factory.
- The final window added: **`persistRecord`** (every settings/profile/report
  write returns success and surfaces a bounded failure note instead of throwing;
  guarded reads survive a denied origin at first render - this fixed a real
  crash found in the disclosure initializer), stale-response guards pinned by
  tests (provider swap, credential save, probe reply, command preview discard,
  manual port edit vs late suggestion), the v2 cancel lifecycle surface, and the
  **retained log well** on the Control screen (the log panel keeps the last
  bounded output under a two-line stopped note after a stop or an unexpected
  exit) plus the restored stopped-log copy text (a refactor placeholder had
  leaked into user-visible copy: "Start this props.profile...").
- Accessibility: focus restored where overridden, WAI-ARIA tabs with keyboard
  support, real table semantics for N-gram results, small-text contrast to at
  least 4.5:1, reflow at 320-980 px and 200%/400% zoom (packaged probes PASS),
  forced-colors emulation (packaged PASS).
- Screens extracted to presentational components (`RuntimeScreen`,
  `TuneScreen`, `DashboardScreen`, `BenchmarkScreen`, `InventoryScreen`,
  `CatalogScreen`, `ProfileEditorScreen`, `ProfileEmptyScreen`) with typed props
  and `evidence-adapter.ts` as the acquisition seam; `App.tsx` keeps
  orchestration and acquisition.

### 3.3 Verification infrastructure (`scripts/`, `.github/`)

- 20+ packaged CDP probe drivers (`g05_*`), including the final-window additions
  `g05_fe16.mjs` (FE-16 overlap/guards/exit walk, 14 checks) and
  `g05_fe05v3.mjs` (two-profile provenance walk, 14 checks); engine-level
  accessibility, high-contrast, responsive, CSP, and branding verifiers; a
  wrong-candidate negative control; template-driven catalog matrix probes.
- The QD-03 shared contract `scripts/lib/health_cancel.mjs` (classifier with
  `completed` / `cancelled` / `refused` / bounded diagnostic and an
  observation-bound constant) with fixture legs in
  `scripts/tests/health_cancel.test.mjs`, wired into `npm test` and into the
  packaged `verify_041` health-cancellation check.
- `scripts/sandbox/host-run-lifecycle.ps1` gained default-off `FaultSimulation`
  legs (`timeout`, `malformed-result`, `missing-assets`) and a stale-document
  guard; `scripts/sandbox/test-fault-evidence.ps1` drives the witness legs and
  requires digest equality with the staged candidate.
- Release workflow: one-shot tag resolution to a full SHA, SHA-pinned actions,
  candidate-fed lifecycle jobs, evidence upload with `if: always()`, an SBOM
  step, and gate tests asserting all of it (the release-gate suite is now 119
  cases).
- Governance: `SECURITY.md`, `CONTRIBUTING.md`, issue templates,
  `docs/EVIDENCE-MATRIX.md`, `docs/SUPPORT-MATRIX.md`, `docs/SUPPLY-CHAIN.md`,
  `docs/ARTIFACT-IDENTITY.md`, security-review limits.

## 4. Verification: what was actually run

### 4.1 Local gates at the final revision

Exact commands and observed results (2026-09-12):

| Gate | Command | Result |
|---|---|---|
| Types | `npx tsc --noEmit -p tsconfig.json` | 0 errors (`TSC_OK`) |
| Component/unit (web) | `npx vitest run` | 141 passed / 141 |
| Node suites | `npm test` (see section 2) | 154 passed / 0 failed |
| Format | `cargo fmt --check` | clean (`FMT_OK`) |
| Lints | `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| Rust tests | `cargo test` | 593 passed / 0 failed / 7 ignored |
| Design detector | `detect.mjs --json src/App.tsx src/App.css` | unchanged 4 known advisories, no new findings |
| Design doc lint | `npx -y @google/design.md lint docs/DESIGN.md` | 0 errors |

The Rust verification runs at the `57bde64e` tree; the docs tip `b80f56a`
differs only in `docs/`, `scripts/` and evidence files - `git diff --stat
57bde64 -- src/ src-tauri/` is empty, so the shipped code under the docs tip is
byte-identical to the verified freeze.

### 4.2 Unit and component suites added or strengthened in the final window

| File | Tests | What it pins |
|---|---|---|
| `src/App.storage.test.tsx` | 4 | Denied-origin storage matrix: runtime-inspect save failure, tuning-report save failure, benchmark save failure, denied-storage first-render survival |
| `src/App.staleResponses.test.tsx` | 5 | Stale provider credential relabel, stale credential-save, stale probe reply, stale command preview discard, manual port edit vs late suggestion |
| `src/V03EvidencePanel.cancel.test.tsx` | 4 | Void ack keeps ownership; rejected cancel keeps the run; late cancel after completion retains the measured result; no-active-benchmark error does not lock the UI |
| `src/screens/DashboardScreen.test.tsx` | 3 | Retained log well after stop; empty state without a retained tail; the corrected "Start this profile..." copy (no refactor placeholder) |
| `src/App.catalog.test.tsx` (extended) | +3 legs | Newest-filter-wins after slow-then-fast resolve; rejected filter keeps prior rows and shows the failure; null `catalog_local_models` tolerance |
| `scripts/tests/health_cancel.test.mjs` | 6 | Fast/slow fixture progress and refused variants through the shared classifier |
| `src-tauri/src/runtime.rs` | +1 | `rt07_path_and_reparse`: junction swap after open, symlink payload, reparse-ancestor refusal |
| `src-tauri/src/local_client.rs` | +2 | The response that outlives the cancel slice completes exactly once; cancellation is still observed during a long wait |

### 4.3 Mutation checks (a test that cannot fail is worse than no test)

Every new behavior in the final window was mutation-checked by breaking the
implementation and confirming the intended test fails, then restoring:

| Mutant | Broken on purpose | Caught by |
|---|---|---|
| M-A | `persistRecord` returns success after a write failure | storage matrix (3 failures) |
| M-C | revert the guarded read to a bare `getItem` | storage matrix (first-render crash) |
| MB1 | preview discards a stale response without a current-ness check | `App.staleResponses` |
| MB2 | `applySuggestedPort` accepts a late suggestion over a manual edit | `App.staleResponses` |
| MB3 | provider-switch guard removed | `App.staleResponses` |
| MB4 | credential-save guard removed | `App.staleResponses` |
| MB5 | probe guard removed | `App.staleResponses` |
| MC1 | catalog filter guard (`sequence >= 0`) disabled | `App.catalog` filter legs |
| MF1 | cancel ack clears `busy` prematurely | `V03EvidencePanel.cancel` |
| MF2 | cancel of `undefined` throws instead of a message | `V03EvidencePanel.cancel` |
| MW1 | removed the retained-log branch (stopped state falls back to empty) | `DashboardScreen.test.tsx` |
| MW2 | restored the "props.profile" copy leak | `DashboardScreen.test.tsx` |
| MR1 | removed `FILE_SHARE_READ` / verification from the managed-file open | `rt07_path_and_reparse` |
| MQ1 | bounded-timeout classifier was made to silently pass | `health_cancel` fixtures |
| MG1 | the witness timeout leg relabeled TIMEOUT as PASS | witness runner + catch-guard |
| MG2 | old catch guard + a planted superseded witness document | digest-equality assertion (message: `setup digest 'a4d14496...' does not match the staged candidate '255bfdd2...'`) |
| ML1 | worker budget re-sliced to 500 ms (the old behavior) | outlives-the-slice regression |

Earlier windows ran the same discipline (recorded in the tracker); this table
covers the final re-bind window.

### 4.4 Packaged Windows campaign on the final candidate

All probes drive the packaged `localmotive-portable.exe` over CDP (port 10085)
with the real profile state. Working logs in `.hermes-0.6/`:

| Probe | Result | Numbers |
|---|---|---|
| Stop supervision (`g05_stop_supervision.mjs`) | PASS | real LaunchProfile started; stop clears the child in 1 s; state idle |
| Seven-stage health (`g05_health.mjs`) | PASS | "CPU passed all seven managed-runtime health stages": device enumeration, backend operation, pinned model load (SmolLM2-135M), loopback server health, deterministic completion, cancellation, process + temp-file cleanup |
| Default v2 workload (`g05_mt01d.mjs`) | PASS | decode mean **486.51 tok/s**, p50 490.02, p95 491.62, n=5, **5/5 sampled**, 1 warmup, ~8 s, on the approved managed CUDA 13.3 runtime |
| Tamper negative (`g05_tamper_dll.mjs`) | PASS | DLL replaced -> Start refused with content-verification notice, 0 processes; exact bytes restored -> Start reaches live; final 0 processes |
| FE-16 walk (`g05_fe16.mjs`) | PASS 14/14 | overlap orderings keep the same pid; legacy control disabled during the v2 run; run survives navigation + rescan; cancel releases guards; identity unchanged by a draft edit; external kill -> non-live, 0 children, retained log well (1313 chars), `server-8080-*.log.failure.json` with `phase=runtime_exit`, `exitCode=1`, tail 1313; restart after exit reaches live |
| FE-05.V3 walk (`g05_fe05v3.mjs`) | PASS 14/14 | profile A (`-c 8192`) and profile B (`-c 4096`) measured; both manifests persisted with distinct paths; distinct compatibility keys `v2:b18f8a68...` / `v2:fcb63186...`; two anchor records; "Replay manifest" restores the saved workload |
| FE-01.V3 + IPC-01.V2 (`g05_vitems_c.mjs`) | PASS all legs | preview command carries the managed runtime path; adoption keeps live; child killed mid-run -> non-live, 0 processes, restart reaches live; corrupt model refused with a truthful artifact message and 0 processes; app killed during startup -> 0 orphans; relaunch reaches live |
| FE-02.V2 adoption walk (`g05_vitems_d.mjs`) | PASS D1-D3 | adopt CUDA -> start live (legacy cuda-12.4 child), adopt CPU -> start live (cpu child), adopt CUDA -> start live (cuda child) - the adoption is proven by child image paths, not labels |
| FE-05.V1/V2 legs (`g05_vitems_e.mjs`) | partial races | the fast runtime made some tail clicks race (recorded honestly in the log); the acceptance coverage for V1/V2 on this candidate rides `g05_fe16.mjs` (cancel-after-navigation) and `g05_vitems_d.mjs` (completed-while-away, decode 974.72 tok/s) |
| MT-05 reservation cycle (`rebind2-mt05-cycle.log`) | PASS | start live -> stop 0 processes -> start live (new pid) -> stop 0 processes |
| Churn reproducer (`g05_churn_repro.mjs`, G-05.I1 residual) | not reproduced | three start/cancel/start/stop rounds, zero surviving children |
| Witness legs (`scripts/sandbox/test-fault-evidence.ps1`) | PASS 4/4 | `witness-timeout` TIMEOUT at `sandbox-timeout`; `witness-malformed-result` FAIL at `sandbox-run`; `witness-missing-assets` FAIL at `resolve-installers`; cancellation kill leaves no PASS artifact - all bound to `sourceRevision 57bde64e` and the staged digests |

Historical packaged context (earlier windows, kept for the record): the same
workloads measured 974.36-1002.60 tok/s on the legacy CUDA 12.4 build, and the
TLS/API-key profile set (patch, ~1004.76 tok/s).

### 4.5 `verify_041` packaged suite

- First isolated run (dirty worktree): 22/24 - `candidate.clean-source` failed
  by design (uncommitted evidence edits) and `ui.blocked-backend` bound to a
  live upstream release whose required jobs are all green (no blocked card
  exists to inspect; the rendering is unit-covered). The verifier now reports
  that case as an explicit skip instead of a false failure, with a release-gate
  assertion pinning the skip.
- Clean-tree run: **`overall_status: PASS`, 25/25 checks** - candidate identity,
  IPC rejections (adapter/digest/size/backend/tag/legacy option), the tamper
  family (isolated root, approve-install, write, health-rejection, reinstall
  repair), seven-stage health, `health.cancellation` through the adaptive
  fast/slow classifier, and restart. Record:
  `release-evidence/0.6.0/attestations/packaged-verification-0.6.0.json`.

### 4.6 Sandbox lifecycle (clean account)

Four Windows Sandbox runs on the final candidate installers, each carrying
`sourceRevision 57bde64e` and the candidate digests
(`255bfdd2...` setup / `01b62b76...` msi):

| Run | Previous tag | Result |
|---|---|---|
| 1 | v0.4.0 | PASS - NSIS fresh install/launch/uninstall, MSI fresh, NSIS update v0.4.0 -> 0.6.0, plus a fourth preservation step (`nsis-preservation-from-v0.4.0`) |
| 2 | v0.5.0 | PASS - the same four steps with `nsis-preservation-from-v0.5.0` (`0.5.0 -> 0.6.0`) |
| 3 | v0.4.1 | PASS - preservation run with host-side canary verification (SQLite mirror + user data collected and checked) |
| 4 | v0.5.0 | PASS - preservation run, same verification |
| Negative | v0.5.0 bytes renamed as 0.6.0 | FAIL-with-identity as designed: `NSIS fresh install expected executable version 0.6.0 but found '0.5.0'`, digests `22a7ef75...` / `581ae3d0...` |

## 5. Defects found during the final re-bind and their fixes

### 5.1 MT-06 cancellation livelock (High - fixed at `57bde64e`)

- **Symptom:** the default v2 workload never completed on the approved managed
  CUDA 13.3 runtime. The run stayed in flight, the server kept generating
  (~476 tok/s, ~560 ms per completion), `tokens_predicted_total` grew past
  211 968 tokens (~540 duplicate generations), and the only exit was the
  per-request budget (600 s each). The same workload on the legacy CUDA 12.4
  build (~985-1002 tok/s, ~300 ms per completion) always stayed under the
  boundary - which is why every earlier campaign passed.
- **Root cause:** `local_client::execute` bounded every cancellable attempt to
  `CANCEL_ATTEMPT_SLICE` (500 ms) and re-issued the request until the
  whole-operation deadline. Any response slower than the slice was discarded
  while the server completed it anyway; the client could never observe a
  healthy slow response.
- **Fix:** cancellable requests run on a worker thread with the full deadline;
  the caller observes the cancel flag in 500 ms slices and abandons the worker
  on cancel (the worker exits by itself at the deadline). Cancel latency is
  unchanged.
- **Regression:** `a_cancellable_response_that_outlives_the_cancel_slice_still_completes`
  (900 ms fixture response completes exactly once; RED before the fix with the
  exact `llama-server did not answer within 6 seconds` signature), plus
  `a_cancellable_request_is_still_cancelled_during_a_long_slow_response`; mutant
  ML1 caught; release gate "the cancellable local client never re-issues a
  slow-but-healthy response (MT-06)".
- **Packaged re-verification:** default v2 completes at 486.51 tok/s, 5/5
  (section 4.4). The fix also removes the duplicate-generation churn: a cancel
  abandons exactly one worker, and no queued abandoned requests pile up.

### 5.2 Fault-witness stale-document defect (harness - fixed in the same window)

- **Symptom:** after the candidate re-cut, `witness-malformed-result.json` still
  carried the superseded candidate's digests while the witness run reported OK.
- **Root cause:** the harness catch guard
  (`if (-not (Test-Path $EvidencePath) -or status -eq "PASS")`) skipped
  rewriting whenever any older FAIL document existed, and the witness assertion
  accepted that stale document.
- **Fix:** `host-run-lifecycle.ps1` treats a document older than the current run
  as stale identity and rewrites it; `test-fault-evidence.ps1` deletes each
  witness document before its leg and requires `candidateDigests` to equal the
  staged candidate's hashes.
- **Proof:** mutant MG2 (old guard + planted `a4d14496`-era document) is caught
  by the digest-equality check; all three witnesses re-verified bound to
  `57bde64e` + `255bfdd2...` / `01b62b76...`.

### 5.3 FE-16 copy and retained-log fixes (shipped code)

- A refactor placeholder had leaked into user-visible copy on the Control
  screen's stopped-log empty state ("Start this props.profile..."). Restored to
  "Start this profile...", with mutants MW2 pinning it.
- The log well previously erased the last output on stop. It now retains the
  last bounded output under a two-line stopped note (mutant MW1) and the
  `DESIGN.md` log-well section documents the behavior.

### 5.4 Evidence-tooling corrections (scripts only, not shipped code)

- `g05_tamper_dll.mjs`: the start control is "Start" on Profile and "Start
  profile" on Control; the matcher now accepts both, and click results are
  logged.
- `g05_vitems_c.mjs` / `g05_churn_repro.mjs`: kill patterns cover
  `localmotive-portable.exe`, rounds re-anchor their view, and start clicks wait
  for the non-live state (several tail legs had raced the app's fast
  transitions).
- `g05_health.mjs`: terminal matching no longer mistakes catalog card text for
  health results.
- `g05_fe05v3.mjs`: benchmarks live under the Roaming app data
  (`%APPDATA%\io.github.localmotive.app\benchmarks`), manifests are matched by
  newest mtime, and the calibration-record call guards a missing key.
- `g05_fe16.mjs` (V3): the exit-status command is one-shot - the first observer
  consumes the rich exit record and later reads see the cleared slot - so the
  walk asserts the durable route instead: the retained well plus the persisted
  `.log.failure.json` beside the run log.

## 6. What is NOT complete

24 checkboxes remain open. None is a High finding without a disposition; 17 are
owner-gated and 7 are environment-blocked.

### 6.1 Owner-gated (the only blocking remainder)

These require explicit owner authority and are prepared, not executed:

1. **Repository rulesets** - `gh api repos/sato942/localmotive/rulesets` returns
   none and `branches/main.protected` is false. Ready-to-apply payloads for a
   main-branch ruleset (require `pr-check`, forbid force pushes/deletions) and a
   `v*` tag ruleset (immutable tags) are in `docs/RELEASE-REVIEW-0.6.md`.
   (GH-01.I3/V1/V2/V3, GH-02.I3/V3)
2. **Authorized PR runs** - a benign PR and a controlled failing-check PR would
   each consume a GitHub-hosted `pr-check` run; they are not started without
   approval. (GH-01.V1/V2)
3. **Publication** - push `main`, `git tag -a v0.6.0` at the verified SHA, run
   the release workflow, read back artifacts (GH-03.V1/V3, GH-06.V3, G-09.*),
   then close out (G-10.*). Nothing is tagged, signed, or published yet.

### 6.2 Environment-blocked deferrals (six-field register rows in the review)

- **RT-04.V2** - the delayed health-model download between context preparation
  and runtime execution has no packaged harness; the DLL-replacement half is
  proven (packaged refusal) and unit lease tests pin the window. Follow-up: a
  throttled local health-model mirror in a later window.
- **RT-06.V3** - seven-backend measurement needs all seven backends installed
  on a representative host; this host has one verified CUDA backend plus a CPU
  record. Follow-up: multi-backend host session.
- **DC-04.V2** - `save_user_catalog_override` has no frontend surface to drive
  from the UI; the command path is covered by Rust authority tests. Follow-up:
  a reviewed override UI, then the controlled-server run.
- **DC-12.V3** - OS-crash/power-loss validation is unsafe on the live
  verification host; resume semantics are tested at process level. Follow-up:
  storage fault-injection lab.
- **MT-07.V2** - real CPU-only and mixed-machine identity classes are
  unavailable; unit tests cover the classes. Follow-up: hardware matrix.
- **S-25.I3** - the independent performance program (distributions, drift,
  calibration error, queue metrics) needs real target hardware sessions.
- **G-05.I3** - Narrator/NVDA, OS-level high-contrast, and live cloud/HF
  credential scenarios need a screen reader, an interactive settings session,
  and an authorized test account; engine-level emulation is done.

None of these may be read as passed. Each carries owner, reason, residual risk,
workaround, follow-up milestone, and evidence gap in
`docs/RELEASE-REVIEW-0.6.md`.

### 6.3 Not-yet-pushed state

The 172 local commits must be pushed before a tag can be planted at the
verified lineage. Until then, origin serves `v0.5.0`-era code and the release
workflow has nothing newer to build from.

## 7. Issues faced and how they were resolved

1. **Three chained defects in the flagship v2 evidence flow** (batch 7) -
   found only by exercising the packaged app: `preflight_model` invoked with
   flat fields while the command requires `request` (preflight could never
   run); the inspected runtime version stored the raw multi-line `--version`
   output, which the manifest rejects for control characters; warm-cache trials
   recorded only the evaluated prompt slice, so every warm trial failed the
   declared-count contract. All three were fixed regression-first with caught
   mutations and re-verified packaged (1002.60 tok/s, 5/5 sampled). The lesson
   recorded in the tracker: unit fixtures had encoded the same wrong shapes, so
   only the packaged flow caught them.
2. **MT-06 cancellation livelock** - section 5.1. Found by the re-bind probes
   when the managed runtime's decode rate crossed the 500 ms slice boundary.
3. **Fault-witness stale-document acceptance** - section 5.2. Found by chasing
   digests during the same window.
4. **Orphaned inference process** - app kill left a `llama-server` alive because
   `process-wrap` defaulted to `kill_on_drop=false` on a shared job with
   `ACCESS_DENIED`. Fixed with a per-child `KILL_ON_JOB_CLOSE` job and
   `TerminateJobObject`; proven by killing the app mid-run and counting zero
   children.
5. **TLS traps** - a CN-only certificate fails SAN validation, and the common
   `openssl req -x509` default emits `CA:TRUE`, which rustls refuses as
   `CaUsedAsEndEntity`; the health loop then retried silently for the full
   timeout. Fixed with a fail-fast `health_tls` verdict carrying an exact
   recovery instruction, and a timeout that names the last transport error.
6. **Signed catalog versus label migration** - correcting quant labels changed
   embedded-signature bytes, so the data file could not be edited without an
   owner-run resign. The migration pipeline was proved on a copy (175/1417
   labels) and the committed catalog was left untouched by design.
7. **Multi-instance probe confusion** - a kill pattern matched `localmotive.exe`
   but the portable binary is `localmotive-portable.exe`, so several app
   instances stacked during probing. Identified, cleaned, and the affected
   observations re-verified single-instance; the corrected kill targets
   `localmotive*`.
8. **Harness and fixture lessons** - background-wrapped Node drivers died with
   "stdin is not a tty" (foreground runs with `< /dev/null` fixed it); WebView2
   cold start exceeded 30 s under sandbox load (CDP readiness gates added);
   MSI-installed bytes differ from NSIS-installed bytes for the same build
   (per-family digest consistency adopted instead of a single expected hash);
   `Get-FileHash` needs the Windows PowerShell 5.1 module path isolated from
   PS7; a DLL tamper probe hit EBUSY while a child held the DLL (kill children,
   then tamper); in the final window a stale DLL lock from the v2 run required
   stopping the server before the tamper leg.
9. **Flakes fixed at the root** - a global verification-bytes counter polluted
   parallel tests (thread-local mirror), a load-sensitive download count assert
   was hardened, and the Windows fixture retry pattern replaced a racy bind.
10. **Gates versus improvements** - release-gate tests that snapshotted old
    harness text or README wording failed when the behavior improved (for
    example the enriched failure evidence and the support-matrix wording); each
    was updated to assert the stronger contract rather than reverted. In the
    final window the FE-09 gate moved from a raw-`setItem` match to the
    `persistRecord` contract, and the blocked-backend check gained its
    healthy-upstream skip.
11. **Tracker integrity incidents** - a section-boundary duplication after an
    automated edit was caught by the structural checker, the file was restored,
    and the record re-applied; the header status table was reconciled late after
    it had gone stale; a CRLF rewrite of `docs/DESIGN.md` introduced by an
    editing tool was normalized back to LF before commit.

## 8. Release identity and next steps

Final candidate (staged in `.hermes-0.6/final-candidates/`, inventoried by the
repository tool at `sourceRevision 57bde64e`; `candidate-inventory-0.6.0.json`):

| Artifact | Bytes | SHA-256 |
|---|---|---|
| `Localmotive_0.6.0_x64-portable.exe` | 20 956 672 | `7961595091822b6d4fd1b8efcd4d4f8bec33b1a75a913ce3774e36ac2c3371fa` |
| `Localmotive_0.6.0_x64.msi` | 9 052 160 | `01b62b7646850d4c86d92e5d05cc6f0b5b492d28a9a3ebc228851c927535b124` |
| `Localmotive_0.6.0_x64-setup.exe` | 5 291 075 | `255bfdd24d13905cb58f4d210c700b0c8facc1b8fbef1e56d3ebd4feb8e9c13f` |

- Source code freeze `57bde64e`; docs/evidence tip `b80f56a`; this re-cut
  supersedes `075daa54...` (stale after `06cfa99` changed shipped code after its
  build) and `272ae15` (superseded by the MT-06 fix); `git diff --stat 57bde64 --
  src/ src-tauri/` is empty at the tip.
- Committed evidence: `release-evidence/0.6.0/attestations/` contains the
  packaged verification record, four lifecycle records with their sandbox logs,
  collected mirror/user-data canaries and host-side verify logs, three witness
  pairs, and the negative control.
- Host attestation for the packaged runs: Zen 5 / RTX 5090 / driver 610.74 /
  Windows 11 build 26100, verdict MATCH.

Owner actions, in order: (1) apply the two rulesets; (2) authorize the two PR
runs; (3) push `main`; (4) authorize `git tag -a v0.6.0` at the freeze and the
release workflow; (5) read back the published assets and close out
(G-09/G-10). The release workflow rebuilds the installers from the resolved tag
- the digests above are the local anchor for the verified source revision, and
the published set is compared against the recorded inventory during readback
(G-09.I3). The exact commands are in `docs/RELEASE-REVIEW-0.6.md`, section
"Owner package for G-09".

## 9. Honest limits of this report

- Claims about packaged behavior come from the recorded probe logs and evidence
  files, not from this document. Working logs under `.hermes-0.6/` are scratch
  and gitignored; the committed evidence is under `release-evidence/0.6.0/`.
- The support matrix is deliberately conservative: only Windows 11 x64
  lifecycle, the RTX 5090 CUDA path, and the listed packaged scenarios are
  claimed; everything else is marked untested in `docs/SUPPORT-MATRIX.md`.
- Deferrals remain unresolved by definition; none of them may be read as passed.
- Measured numbers are scoped to this host, the shipped workloads, and the
  active runtime build (the decode rate differs between the legacy CUDA 12.4
  and managed CUDA 13.3 builds; both are recorded above).
- The distro ships unsigned by policy (disclosed); no signing or provenance
  claim beyond the SHA-256 pairing and the workflow records is made.

## Appendix A - exact commands to reproduce the final checks

```bash
# Local gates (from the repository root)
npx tsc --noEmit -p tsconfig.json
npm test
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

# Candidate build and staging
npm run tauri build
cd .. && cp src-tauri/target/release/bundle/msi/Localmotive_0.6.0_x64_en-US.msi \
  .hermes-0.6/final-candidates/Localmotive_0.6.0_x64.msi
cp src-tauri/target/release/bundle/nsis/Localmotive_0.6.0_x64-setup.exe .hermes-0.6/final-candidates/
cp src-tauri/target/release/localmotive.exe .hermes-0.6/final-candidates/localmotive-portable.exe
cp src-tauri/target/release/localmotive.exe .hermes-0.6/final-candidates/Localmotive_0.6.0_x64-portable.exe
(cd .hermes-0.6/final-candidates && sha256sum Localmotive_* > SHA256SUMS-0.6.0.txt)
node scripts/verify_candidate_inventory.mjs .hermes-0.6/final-candidates 0.6.0 \
  .hermes-0.6/final-candidates/candidate-inventory-0.6.0.json

# Packaged probes (app launched with the debugger port; see AGENTS.md)
WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=10085' \
  ./.hermes-0.6/final-candidates/localmotive-portable.exe < /dev/null &
node scripts/g05_stop_supervision.mjs 10085
node scripts/g05_health.mjs 10085
node scripts/g05_mt01d.mjs 10085
node scripts/g05_tamper_dll.mjs 10085
node scripts/g05_fe16.mjs 10085
node scripts/g05_fe05v3.mjs 10085

# Isolated packaged verification (verify_041)
bash .hermes-0.6/run-verify041.sh

# Sandbox lifecycle, witnesses and negative control
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/sandbox/test-fault-evidence.ps1
export GH_TOKEN=$(gh auth token); export LOCALMOTIVE_SOURCE_REVISION=$(git rev-parse HEAD)
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/sandbox/host-run-lifecycle.ps1 \
  -Tag candidate-0.6.0 -Version 0.6.0 -CandidateDir .hermes-0.6/final-candidates \
  -PreviousTag v0.5.0 -EvidenceName sandbox-clean-account-lifecycle
```
