# Localmotive 0.6.0 remediation report
**Snapshot:** 2026-09-13, ninth pass (freeze 9). Candidate code freeze `eb01bc9f97417e74ce7de7da85c86582f42ceb67`; portable `97e1a4fb2c7622c2...`, setup `8398cf8a25949049...`, msi `2c00e0d3f9e2dc05...` (staged `candidate-inventory-0.6.0.json`, PASS, 3 artifacts). Freeze-8 and every earlier freeze remain visible unlabelled: `da091a4` (freeze 8; preserved under `release-evidence/0.6.0/history/freeze8-da091a4/`), `85edee6`, `7c32fa9`, `3a2b06e` and the lineage table below.

**What the ninth pass changed (R16):** benchmark ownership now holds until every owned request and worker exits (derived drain bound, one client per run, drain before slot release on every exit path); the packaged immediate-restart proof attempts the replacement before any drain wait and records the refusal; the raw SEC1 private-key defect is fixed with a complete-structure fixture; the lifecycle harness owns an atomic lock with ownership asserted before every evidence write and refuses live owners; the qualification manifest generator/verifier and the installer-payload identity tool are committed with fourteen manifest negative controls and eight promotion negative controls; the tag-push release workflow verifies without publishing and a separate authorized promotion workflow consumes the exact qualified bundle; and the installed NSIS/MSI payloads carry recorded identities plus a CDP functional probe of their exact bytes. Gates on the freeze: `npx tsc --noEmit` 0; Vitest 141/141; `cargo fmt --check` 0; clippy `-D warnings` 0; `cargo test` 613 passed / 0 failed / 7 ignored; packaged verification 25/25 `overall_status=PASS` `source_dirty=false` on the freeze-9 portable; MT-06 59/59; RT-06 22/22 with the backend scope recorded; a11y A11Y_PASS; the four-leg lifecycle matrix 4/4 (the v0.5.0 upgrade and v0.4.1 preservation legs re-ran natively on the freeze-9 bytes; the v0.4.0 upgrade and v0.5.0 preservation legs are carried forward through the ledger against the committed freeze-8 producer inventory with a written source-delta reason).


**Snapshot:** 2026-09-12, fourth pass. Candidate code freeze `7c32fa9`; docs/driver tip follows on `main` (this pass's docs commit). The candidate was rebuilt at the freeze and restaged: portable `fcf1d3b2…`, msi `c18eecc2…`, setup `e405cd1e…` (`SHA256SUMS-0.6.0.txt` and `candidate-inventory-0.6.0.json` record `sourceRevision 7c32fa90b9dd04057ba3c089392b0642f3b0545c`). Superseded freezes and their digests remain visible unrelabelled: `3a2b06e` (portable `fbbd2a1a…`), `57bde64e` (portable `79615950…`), `a0ed247`/`6384df0` in the lineage table.

**What the fourth pass changed:** the publication path now consumes the retained verified candidate with its original inventory and validates source identity plus every digest/size before promoting those bytes (the `--verify` CLI resolved its artifact directory from `argv[2]` — the `--verify` flag itself under the exact workflow invocation — and was repaired RED-first with mutation MUT-W3 caught). Lifecycle evidence binds to the candidate inventory's full source SHA with the harness revision recorded separately, and a substituted environment revision is refused. The health-model fetch gained the verifier-profile loopback seam and RT-04.V2 is sealed packaged 17/17 with a controlled slow-server delay fixture. MT-06 gained resource-bound proofs beyond caller latency (three slow-body unit tests plus a packaged six-cycle resource driver, 37/37). Gates on the freeze: `npx tsc --noEmit` 0; Vitest 141/141; node gates 126/126; `cargo fmt --check` 0; clippy `-D warnings` 0; `cargo test` 600 passed / 0 failed / 7 ignored; isolated `verify_041` re-run on the committed tree 25/25 PASS.
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
| Packages 1-12 | Sequenced remediation of all 72 findings | Implementation landed for all 72 findings with regression tests and ledger records; the verification/closure state per item is categorised in 2.1 - closure is claimed only where both kinds of box are checked |
| Supplemental S-01..S-29 | Unnumbered audit recommendations | 28 closed with tests/evidence; S-25.I3 remains an explicit hardware-program deferral |
| Gates G-01..G-04, G-06, G-07 | Candidate identity, verification wave, advisories | Closed with evidence |
| Gate G-05 | Packaged Windows wave | Executed in seven batches, a High-finding campaign, and two final-candidate re-binds (2026-09-12); only the environment-blocked I3 residuals remain |
| Gate G-08 | Release decision (support matrix, disclosure, deferral review) | Closed: review + support matrix + unsigned disclosure + reconciliation; all deferrals recorded |
| Gates G-09, G-10 | Authorized publication and closeout | **Open - owner-gated** (section 6) |
| Checkboxes | 663 | 639 checked, 24 open - every open item has a written disposition |

### 2.1 Verification state by category (2026-09-12 third pass)

The statuses below distinguish implementation, verification and closure
explicitly. An item is "fully closed" only when both its implementation and its
verification boxes are checked with recorded evidence.

- Findings (72): **63 fully closed**; 4 environment-blocked where the required
  hardware/environment does not exist on this host (RT-04, RT-06, DC-12,
  MT-07); 4 owner-gated governance findings (GH-01, GH-02, GH-03, GH-06);
  FE-05 is implementation- and verification-complete but is held open at
  finding level for the release decision on its stated completion criteria
  (owner directive of 2026-09-12).
- Supplemental packages (29): 28 closed; S-25.I3 environment-blocked
  (independent benchmark distributions).
- Gates (10): G-01-G-04, G-06, G-07 and G-08 closed; G-05 carries only its
  environment residual G-05.I3 (human-operated accessibility session); G-09
  (4 rows) and G-10.I1 are owner-gated on the authorized publication.
- Checkboxes: **643 checked / 20 open**. Every open box carries a category, a
  completion criterion and an unblock action in the tracker's third-pass
  reconciliation; no finding is called closed without its rows.

Test suites grew from the baseline recorded at the audited start (`e530371`,
2026-09-11: `cargo test` 403 passed / 0 failed / 2 ignored; **Vitest 52** -
`src/model.test.ts` carried 52 cases; **Node release-gate suite 80**) to, at
the current freeze `3a2b06e`:

- **Rust: 595 passed / 0 failed / 7 ignored** (`cargo test`, `.hermes-0.6/dc04-cargo-gates.log`),
  with `cargo fmt --check` clean and `clippy --all-targets -- -D warnings`
  reporting zero warnings at every package closure.
- **Vitest: 141 passed / 141** (`npx vitest run`).
- **Node suites: 157 passed / 0 failed** (measured 2026-09-12): release gates
  122 (including the new publish-promotes-verified-bytes, DC-04 seam, DC-04
  distinction and download-seam gates), health-cancellation fixtures 6, and
  the catalog-schema, HTTP-retry and IPC-contract files.
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
  `Localmotive` root while legacy installs are discovered and reused
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
| Rust tests | `cargo test` | 595 passed / 0 failed / 7 ignored |
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

Third-pass re-run on `3a2b06e` (portable `fbbd2a1a…`), after the DC-04 seam
and the release-path fixes; every row below ran on this freeze:

| Probe | Result | Numbers |
|---|---|---|
| Stop supervision | PASS | child terminated within 1 s, no surviving listener |
| Seven-stage health | PASS | 7/7 stages on the pinned SmolLM2-135M |
| Default v2 workload | PASS | decode mean **978.38 tok/s**, 5/5 sampled, 8.1 s (legacy CUDA-12.4 adopted runtime; the v2 code path is the object) |
| Tamper negative | PASS with enforced preconditions | the new guards refused two wrong-state attempts (server live; legacy runtime active) before the real leg; refusal `Managed file llama-server-impl.dll failed content verification`, 0 processes; exact SHA restore; clean restart LIVE; final 0 processes |
| FE-16 walk | ALL-PASS | retained tail (1 321 chars), no leaked placeholders, restart after exit |
| FE-05.V3 walk | ALL-PASS | anchors=3, distinct provenance keys, replay route restores workload `fe05v3-b` |
| vitems_c walk | DONE | P2-P5 including relaunch |
| vitems_d walk | DONE | completed-while-away retention + adoption legs |
| MT-05 combined cycle | PASS | live -> 0 processes -> live -> 0 processes, reservation released |
| Churn reproducer | not reproduced | 3 rounds, zero surviving children |
| Witness legs | ALL PASS | missing-assets / malformed-result / timeout (stays TIMEOUT) / cancellation (killed, no PASS artifact), digest-equality guard enforced against the new digests |

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

- Third isolated run (`3a2b06e`, clean worktree, port 10086): **25/25 PASS**
  (`overall_status: PASS`; `.hermes-0.6/rebind3-verify041-clean3.log`;
  attestation `release-evidence/0.6.0/attestations/packaged-verification-0.6.0.json`
  refreshed). Two intervening attempts did not count: one failed only
  `candidate.clean-source` by design (uncommitted evidence edits at run time),
  and one raced a leftover app instance and was discarded; the clean rerun
  after the third-pass commit is the recorded result.

### 4.6 Sandbox lifecycle (clean account)

Current chain on the `3a2b06e` candidate (portable `fbbd2a1a…`), every run
carrying `sourceRevision 3a2b06e7f8cf7d49f86f759799a0418149735009` and setup digest
`b83afa2c509405b2…`; these are the runs the release binds to:

| Run | Previous tag | Result |
|---|---|---|
| 1 | v0.4.0 | PASS - NSIS fresh install/launch/uninstall, MSI fresh, NSIS update v0.4.0 -> 0.6.0, preservation step |
| 2 | v0.5.0 | PASS - main candidate flow with preservation from v0.5.0 |
| 3 | v0.4.1 | PASS - upgrade-from-v0.4.1 with host-side canary verification |
| 4 | v0.5.0 | PASS - preservation run with canary verification |
| Negative | v0.5.0 bytes renamed as 0.6.0 | FAIL-with-identity as designed: the harness refuses the doctored bytes (digests `581ae3d093423bcb…` / `22a7ef75d15900fd…`) |

The superseded second-pass chain (four PASS runs bound to `57bde64e` and
digests `255bfdd2…` / `01b62b76…`, plus the same negative control) stays in
git history as its own record; no document was relabelled.

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

### 5.5 Release-path defects found while confirming publication (governance)

Two defects in `.github/workflows/release.yml` were found while confirming that
publication promotes the verified bytes:

- The publish ship guard compared the resolved tag against the literal
  `'v0.5.0'` (audit S-26 I2 replaced the same literal in the `resolve` job but
  missed this one), so pushing `v0.6.0` would have skipped publication
  entirely.
- `publish` ordered only after `[rust-audit, quality, package]` while
  `clean-account-lifecycle` ran in parallel, so a lifecycle FAIL could still
  publish. GH-03's original "must not gate on the interactive Sandbox feature"
  note is explicitly superseded: absent or failed lifecycle evidence must not
  ship.

Fix commit `1776146` (workflow + `.github/workflow-gates.json` policy +
release gates): the guard now binds `needs.quality.outputs.tag ==
github.ref_name` on a `v*` tag push (the dispatch republish path is unchanged),
`publish` needs `clean-account-lifecycle`, and the policy checker enforces the
dependency. Gates 120 -> 122; mutant WF1 (lifecycle dependency dropped) and
mutant WF2 (literal guard restored) were each caught by three independent
layers. Publication still promotes exactly the bytes the `package` job built,
verified and uploaded as `localmotive-<version>-verified`: `publish` downloads
that artifact, re-checks `sha256sum -c` without rewriting it, and no build step
exists in the publish job.

### 5.6 DC-04.V2 executed through the supported command path

The 2026-09-11 blocker ("driving raw IPC would not exercise a user-acceptable
flow") was resolved by the owner's 2026-09-12 directive: the test does not
require an override UI. Enabler commit `3a2b06e` adds
`download::resolve_url_with`, which honours `LOCALMOTIVE_HF_BASE` only when
`LOCALMOTIVE_VERIFY_ISOLATED_ROOT` is set and only for plain-HTTP loopback
values (unit tests + mutant MUT-HB1 caught; source pin in the release gates).

Run: `.hermes-0.6/run-dc04.sh` (isolated root, CDP 10087, loopback fixture
serving 69 632 controlled bytes) + `scripts/g05_dc04_override.mjs` ->
**13/13 PASS** (`.hermes-0.6/dc04-final.log`; attestation
`release-evidence/0.6.0/attestations/dc04-v2-command-path-verification.log`):
override saved with correct digest; user row keeps user provenance while 159
curated rows keep curated provenance; correct digest publishes and the
published bytes match the fixture (2 fixture hits: probe + ranged read); a
wrong digest refuses with `The downloaded file failed its SHA-256 checksum and
was deleted. Please try again.` and publishes nothing; override removal makes
the download refuse with `That repository, file, or revision is not in the
validated catalog or in your local overrides.` **without touching the network**
(fixture hit delta 0); the user row disappears after removal; a curated id
cannot be removed here (`That entry ships with the curated catalog and cannot
be removed here.`).

### 5.7 Publication consumes the retained inventory (release contract)

The publish job never builds. It downloads the retained `localmotive-<version>-verified` artifact produced (and verified once) by the package job, then validates before upload: `LOCALMOTIVE_SOURCE_REVISION="$RESOLVED_SHA" node scripts/verify_candidate_inventory.mjs --verify "artifacts/candidate-inventory-${VERSION}.json"` followed by `sha256sum -c`. The verifier checks the recorded `sourceRevision` against the resolved tag revision (both must be full SHAs), every artifact size, and every artifact digest against both the inventory and the checksum file.

Fourth pass: replaying that exact invocation against the staged candidate exposed a latent defect — the CLI resolved its artifact directory from `argv[2]`, which under `--verify <inventory>` is the flag itself, so the step would have failed closed with `ENOENT …‑‑verify\SHA256SUMS-0.6.0.txt`. RED was reproduced first; the fix resolves the artifact directory from the inventory's own directory. Proof (`.hermes-0.6/publication-inventory-proof.log`): `PASS producer inventory verified: 3 artifacts at source 7c32fa90b9dd04057ba3c089392b0642f3b0545c`; a wrong expected source refuses with `candidate inventory records source 7c32fa9…, expected 000…`; mutant MUT-W3 (drop the directory resolution) is caught by the new gate assertions in `scripts/tests/release-gates.test.mjs`.

### 5.8 Lifecycle evidence binds to the candidate inventory (not HEAD)

`host-run-lifecycle.ps1` now reads `candidate-inventory-<version>.json` from the candidate directory and binds every evidence document to its full source SHA; when `LOCALMOTIVE_SOURCE_REVISION` is set to anything else the run refuses (`MISMATCH_EXIT=1`, log `.hermes-0.6/freeze5-harness-mismatch.log`). The harness revision is recorded separately per document, so evidence names both the candidate source and the tooling that produced it. Re-run on freeze `7c32fa9`: up40 PASS, main PASS, preservation v0.4.1 PASS, preservation v0.5.0 PASS — every document shows `sourceRevision 7c32fa90b9dd…` with distinct `harnessRevision` values (`627dda1…`, `2f0a544…`), and the negative control (v0.5.0 bytes labelled 0.6.0) fails with the identity error as required. Fault-simulation legs may run without staged candidates and record a null binding; the witness suite passes all four legs (missing-assets FAIL at `resolve-installers`, timeout TIMEOUT at `sandbox-timeout`, malformed FAIL at `sandbox-run`, cancellation killed with no PASS artifact). The witness script no longer exports a HEAD-derived revision — that export was the exact substitution the binding refuses.

### 5.9 MT-06 extension: resources, not just caller latency

The cancellable client's fix is now proven beyond the caller-return measurement, in both layers:

- Unit (`src-tauri/src/local_client.rs`): a slow-body fixture dribbles responses while counting requests, live connections, completions and aborts. `a_cancelled_body_read_resolves_worker_ownership_without_duplicate_requests` cancels mid-body, proves the caller returns within one slice, the abandoned worker exits at its own deadline (worker-liveness counter reaches zero), the connection is torn down exactly once and the server saw exactly one request. `a_cancelled_call_lets_a_short_body_finish_once_on_the_owned_connection` proves the worker owns a short in-flight response to natural completion with no re-issue. `repeated_cancel_restart_cycles_keep_workers_bounded_and_requests_exact` runs six cancel/restart cycles: exactly one request per cycle, live workers return to zero between cycles. Mutations MUT-W1 (drop the lease decrement) and MUT-W2 (inflate the worker deadline) are both caught.
- Packaged (`scripts/g05_mt06_cycles.mjs`, 37/37 checks PASS): six run→cancel mid-flight→settle cycles on the real binary. Per cycle the panel settles, `llamacpp:requests_processing` drains to 0, exactly one owned `llama-server` child exists during the run and zero after stop, restarts reach LIVE, and the final stop releases the listener.

### 5.10 RT-04.V2 sealed and the store-repair disclosure

The delayed-download window now has its own harness. A verifier-profile seam rebases the pinned health-model URL onto a loopback fixture (`download::rebase_download_url`, honored only with the isolated-root gate and plain-HTTP loopback only; production verification of the downloaded bytes is untouched). The packaged run (`scripts/g05_rt04v2_delay.mjs`, 17/17 PASS, attestation `release-evidence/0.6.0/attestations/rt04v2-delayed-download-verification.log`): the fixture streams the pinned model at ~4 MB/s; the run is observed mid-download; a swap attempt inside the pinned window is denied by the execution lease's read-shared handles (`EBUSY` — the no-time-based-release mechanism); the delayed run completes on untampered content with the full byte count served; tampering BOTH verified runtime copies (primary and legacy roots) outside the window yields `trust_failure` with the content-verification message and zero children; restoring the exact bytes (`87c4e9d0…`) returns a full seven-stage PASS. A single-copy tamper is defeated by the verified-copy fallback, which is why the refusal leg tampers both.

Honest disclosure: the first RT-04 attempt had a driver defect — its restore path wrote the caller-supplied health-model backup over `llama-server-impl.dll`, corrupting the managed store. The store was repaired from the approved release archive (`llama-b10816-bin-win-cuda-13.3-x64.zip`, digest `f362882b…` matching `approved_runtimes.json`), installed the manifest-exact `llama-server-impl.dll` (`87c4e9d0…`, 8 896 000 bytes), and a full re-scan verified all 55 manifest files (`.hermes-0.6/rt04-repair/repair.log`). The driver now captures and restores from its own DLL backup, and the lesson is recorded in the packaged-verification skill.

## 6. What is NOT complete

24 checkboxes remain open. None is a High finding without a disposition; 17 are
owner-gated and 7 are environment-blocked.

### 6.1 Owner-gated (repository settings and the authorized release run)

| rows | finding | completion criterion | unblock action |
| --- | --- | --- | --- |
| GH-01.I3/V1/V2/V3 | governance High | ruleset requiring `pr-check` (no bypass for ordinary contributions; force-push/deletion forbidden) applied; benign PR shows the checks; a controlled failing check blocks merge; readback recorded | owner applies the ruleset, then one PR run |
| GH-02.I3/V3 | governance High | `v*` tag ruleset (updates/deletions blocked) applied; candidate-flow checkout/inventory comparison on a real run | owner applies the ruleset; authorized release run |
| GH-03.V1/V3 | governance High | fresh-version release exercised in the single-runner configuration; publish reads the published bytes back | authorized `v0.6.0` release run |
| GH-06.V3 | governance High | completed release run's artifact collection inspected, not just its code | authorized `v0.6.0` release run |
| G-09.I2/I3/V1 (and I1 beyond the prepared package) | release gate | tag/publish authorized; inventory+provenance bound to the published assets; readback; negative controls exercised on the real path | owner publish decision |
| G-10.I1 | release gate | final record written from the shipped release | after G-09 |

Live recheck 2026-09-12: `gh api repos/sato942/localmotive/rulesets` returns
`[]`; `branches/main` is `protected=false`; `git ls-remote --tags origin`
lists `v0.4.0`, `v0.4.1`, `v0.5.0` only. No owner gate has been granted since
the 2026-09-11 package.

### 6.2 Environment-blocked deferrals (six-field register rows in the review)

| rows | specific missing prerequisite |
| --- | --- |
| RT-06.V3 | a large multi-backend library installed on a representative host (this host has one verified CUDA backend) |
| DC-12.V3 | a controlled OS-crash/power-loss harness (ordinary process-kill coverage exists: vitems_c D7, FE-16 v3) |
| MT-07.V2 | CPU-only and changed-CPU machines for the portability matrix (fit-reduced rows need the same) |
| S-25.I3 | independent held-out benchmark distributions/baseline drift data |
| G-05.I3 | a human-operated keyboard/Narrator/high-DPI/reduced-motion session |

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

### 8.0 Fourth-pass release identity

- Candidate code freeze: `7c32fa90b9dd04057ba3c089392b0642f3b0545c` (`feat(verification): RT-04.V2 delay seam, MT-06 resource proofs, inventory-bound lifecycle, publication gate pins`), rebuilt and restaged in the fourth pass.
- Retained inventory: `.hermes-0.6/final-candidates/candidate-inventory-0.6.0.json` (schema 1, `release 0.6.0`, `sourceRevision 7c32fa9…`, three artifacts with sizes and digests, `checksumFile SHA256SUMS-0.6.0.txt`).
- Candidate digests (freeze `7c32fa9`): portable `fcf1d3b2…`, msi `c18eecc2…`, setup `e405cd1e…`; negative-candidate doctored bytes under `.hermes-0.6/negative-candidates-060/` with their own inventory.
- Publication validates these exact records before promoting (section 5.7); lifecycle evidence binds to the same `sourceRevision` (section 5.8); verify_041 re-ran on the committed tree (25/25) after this pass's tooling fixes.

### 8.1 Candidate lineage (full SHAs)

| revision | role | verified digests (portable / msi / setup) | verification binding | status |
| --- | --- | --- | --- | --- |
| `a0ed247` | 2026-09-11 candidate source | `075daa54…` / `2dd036c6…` / `a4d14496…` | 2026-09-11 campaign | superseded (`6384df0` docs tip) |
| `6384df0` | 2026-09-11 docs tip | - | none (no code) | historical |
| `57bde64` | MT-06 livelock fix | `79615950…` / `01b62b76…` / `255bfdd2…` | second-pass re-bind (packaged set, lifecycle, negative, verify_041 25/25) | superseded by `3a2b06e` |
| `1776146` | release-path corrections (workflow/gates/policy) | - | release gates | current (infrastructure; not compiled into the app) |
| `3a2b06e` | DC-04 verification seam | `fbbd2a1a…` / `1d217350…` / `b83afa2c…` | third-pass re-bind (this report) | **current binding** |

The intended release commit is the final `main` tip whose `src/` and
`src-tauri/` trees equal `3a2b06e`'s; any further code change forces another
re-cut and re-bind before tagging. Superseded sets stay in the ledger as
history - no artifact is ever relabelled.

### 8.2 Publication promotes the verified bytes without rebuilding them

`release.yml` builds the installers once in its `package` job, verifies the
packaged executable, and uploads `localmotive-<version>-verified`;
`clean-account-lifecycle` downloads and tests exactly those bytes; `publish`
downloads the same artifact, re-checks `sha256sum -c` **without rewriting it**
and uploads exactly those files. No build step exists in the publish job. The
tag push rebuilds once from the resolved SHA inside the same run - that run's
verified bytes are what ships.

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

Third-pass additions (2026-09-12):

```bash
# Traceability/link validation on the tracker (G-10.V1)
node scripts/verify_tracker_links.mjs              # TRACEABILITY OK

# DC-04.V2 through the supported command path (isolated app + loopback fixture)
bash .hermes-0.6/run-dc04.sh                       # 13/13 PASS

# Tamper negative with enforced preconditions
node scripts/g05_tamper_dll.mjs 10085              # G05_TAMPER_FINAL PASS

# Release-path gates (publish guard + lifecycle dependency + verified-bytes pin)
node --test scripts/tests/release-gates.test.mjs   # 122/122
node scripts/verify_workflow_gates.mjs             # ok: true
```



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

## Follow-up review remediation (R01-R15, reviewed `db548c8`, fifth pass)

On 2026-09-12 the owner supplied `localmotive-0.6-followup-review-db548c8.md`
(R01-R15) against revision `db548c820cbdf2d8b289f797ce27d31f1ff338f3`. The review
record is committed verbatim in this repository. Every finding was verified
against the tree before any change; all fifteen were confirmed and fixed (none
was disproved), each with regression evidence that fails against the old
behavior:

| R | Fix (short) | Regression evidence |
|---|---|---|
| R01 | `release.yml` direct `needs` dependencies; checker enforces output reachability for every workflow | RED list captured; MUT-R01a caught |
| R02 | Tag publication requires `event_name = push`; the predicate is evaluated semantically over the event/ref/input matrix | RED case `dispatch at tag + publish=false`; MUT-R02 caught |
| R03 | Cancellation defined at result acceptance (client, warmup/trial loops, tuning finalist confirmation) | RED first at all three layers; MUT-R03a/b-trial/b-warmup/c caught |
| R04 | One client per benchmark run; the run drains abandoned workers before finalizing; production drain accessor | RED test with a slow-body fixture; MUT-R04 + MUT-R04g caught |
| R05 | Lifecycle: anything short of verified preservation PASS fails the job; evidence flips to FAIL at `preservation-verification` | Witness leg 5; MUT-R05b (false green) caught |
| R06 | Explicit `-CandidateDir` never falls back; installers verified against the inventory (sha256 + sizeBytes) | Doctored-byte negative refused before the sandbox |
| R07 | Real released-version persistence fixtures (v0.5.0-schema mirror with a user override; v0.4.1 cache record) + the four-leg qualification matrix | Six verifier control cases; matrix in `release.yml`; live legs in this campaign |
| R08 | Candidate re-cut; canonical qualification manifest binds source + artifact digests + every record with its own harness revision | `.hermes-0.6/qualification-manifest-0.6.0.json`; older identities labeled superseded |
| R09 | Exact installed-version identity (4-part canonical normalizes; near misses fail); NSIS/MSI payload expectations recorded | Gate added; fresh lifecycle docs carry `nsisPayloadDigest`/`msiPayloadDigest`/`installedPayloadNote` |
| R10 | Real PEM/X.509 parsing (webpki) + structural SPKI extraction + key SPKI derivation (PKCS#8/PKCS#1/SEC1, RSA + EC) + pair match | Marker-junk tests flipped to expect failure; real RSA and EC pairs PASS; unrelated and cross-family pairs FAIL; MUT-R10 caught |
| R11 | Packaged cancellation driver: owned PIDs (Win32 parent scope, enumeration failure throws), server-accepted request gate, persisted `terminalOutcome` assertion, bounded cleanup, overlap-free immediate restart, digest-bound result file | 33/33 checks at this freeze; per-cycle records `cancelled`; MUT via gate pins |
| R12 | Consumer inventory validation fails closed (schema, release identity, exact canonical set, duplicates) | Eight-case behavioral test on the real verifier; MUT-R12 caught |
| R13 | Publication re-resolves the remote tag before publishing; public inventory byte-compared to the producer copy and re-verified; failure diagnostics retained; preflight evidence always written | Gate pins; preflight probe wrote FAIL evidence at stage `initialization` |
| R14 | Owner handoff corrected: remote state, trusted-job scoping, required-PR ruleset rule, tag bound to the reviewed SHA, supported Latest readback, explicit 3-run PR campaign | Gate pin on the corrected text |
| R15 | No redirect hops; environment proxies ignored; request bodies through a bounded writer; file reads bounded by the handle | Three tests with three mutants (follow-redirect, unbounded writer, unbounded read) caught |

The candidate was re-cut at freeze `85edee63d060c43d05df3770d168a81b7421d6df`
because R01-R05 and R10-R15 changed shipped code. Packaged campaign at that
freeze: packaged verification 25/25 `overall_status=PASS` (`source_dirty`
false), MT-06 cancellation 33/33, DC-04 command path 13/13, RT-04.V2 17/17,
supervision/tamper/FE-16/FE-05.V3/catalog/churn clean, lifecycle matrix 4/4 PASS with the R07 flavors (preservation=PASS on every leg), witnesses 7/7 (including the live-lock refusal and stale-lock takeover legs added after a campaign incident where a killed attempt's orphaned harness overwrote a clean preservation record; the harness now locks each scenario and re-authorizes every evidence write), packaged health 7/7, MT-01d decode 867.25 tok/s (p95 878.93), and both negative controls (historical doctored installer FAIL-with-identity; freeze-6 byte-doctored setup refused at `resolve-installers` naming both digests). All scores, digests and records are in the tracker's fifth-pass reconciliation section and the canonical qualification manifest `.hermes-0.6/qualification-manifest-0.6.0.json`.

- **Freeze-8 correction (CI-proven).** The open CI run for the first fixture diagnostic (`3b97899`) FAILED: `parent_exit=Some(0)` at ~10.6 s proved the parent always exits on its own schedule, so breaking the wait on its exit was wrong. The corrected fixture has the parent record the spawned child id and ends the wait early only when the parent exited AND that record is missing: MUT-PROC1 (never spawns) fails fast (`spawned=false`, 183 ms); MUT-PROC2 (descendant needs ~5 s, parent exits at ~1 s) passes in 7.9 s. The candidate was re-cut as freeze 8 (`da091a47ca6595ec37690ab5ae63542671c5f7f6`; portable `f46ed292...`, setup `fd33cb12...`, msi `c167997926...`), the process-affecting packaged set re-ran green on those bytes (verify_041 25/25, supervision, FE-16, MT-06 33/33, RT-06 12/12, DC-04 13/13, a11y, lifecycle 4/4 `preservation=PASS`), the freeze-8 negative refusal failed with identity, and CI for `da091a4` is green. The manifest (19 records) is verified from a depth-1 clone.

### Fifth-pass follow-up additions (2026-09-12, after the campaign at `85edee6`)

- **MT-07.V2 closed.** The five audit scenarios (CPU-only snapshot, same GPU with
  changed CPU, fit-reduced effective context, changed draft settings with the same
  companion, same LoRA filename with changed content) are covered as canonical
  execution-snapshot identity tests (`calibration::tests::
  mt07_cpu_only_and_hardware_changes_are_distinguished` plus the every-field table
  test). Focused run 4/4 green; mutation MUT-MT07 (key derivation ignores
  `lora_sha256`) caught. The prior `environment-blocked` disposition described the
  live portability matrix, which the criterion's regression-test wording does not
  require; that residual stays in the RT-06/S-25 hardware register.
- **RT-06.V3 (historical).** *(Superseded 2026-09-13: the scoped reading below and in the tracker splits this criterion into the achievable four-backend scope and the still-open seven-installed-backend acceptance portion. This paragraph is the historical freeze-8 record; it is preserved verbatim rather than rewritten.)* Packaged all-backend benchmark on this host (RTX 5090,
  catalog `b10816`): the catalog exposes all seven Windows x64 backends; four are
  eligible on a single-vendor host and were exercised (cpu/cuda-12.4/cuda-13.3
  reused in 0.17-0.75 s; vulkan freshly installed at 35,228,033 bytes in 2.9 s);
  the three non-NVIDIA backends are refused by the runtime-compatibility guard
  with named reasons (a seven-backend host needs three GPU vendors - proven
  refusals, not a code gap). Listing: 4 records with a zero-byte second pass
  (verified-install lease works); selecting 379-388 ms with `managedVerified=true`;
  launching (health) PASS in 3.3-3.4 s; instrumentation counters captured around
  every step. `scripts/g05_rt06_all_backends.mjs` 12/12 checks PASS; evidence
  `release-evidence/0.6.0/attestations/rt06-all-backends.json`.
- **Accessibility probe extended** for G-05.I3's automatable scope: keyboard,
  reduced motion, high-DPI/zoom and forced-colors legs plus a new
  accessibility-tree leg (148 nodes, 10/10 interactive controls carry accessible
  names). Residue: a manual Narrator/NVDA listening pass and live-credential
  scenarios.
- **Soak watch item resolved.** A capturing soak reproduced the one-off failure:
  `proc::tests::dropping_a_contained_process_terminates_descendants` failed its
  "descendant fixture did not start" assertion at the 15 s bound under parallel
  load (nested PowerShell cold start). Fix: 45 s load-aware bound plus early
  parent-exit diagnostics; mutation MUT-PROC1 caught (`parent_exit=Some(0)` in
  162 ms); post-fix soak: three concurrent full suites with zero failure markers
  and 10/10 focused repetitions green.
- **Candidate re-cut (freeze 7, `cb8ab64753d96b61fa43665d2cf1dd6b4ee7928c`).**
  The `proc.rs` fix is the only shipped-tree delta since freeze 6 (16 insertions /
  5 deletions, `#[cfg(test)]` only). The candidate was rebuilt and re-staged
  (portable `9cf0f695…`, setup `4ad8822e…`, msi `09d4dc07…`; inventory PASS) and
  every process-affecting packaged check re-ran on the new bytes: packaged
  verification 25/25 `source_dirty=false`, supervision, FE-16, MT-06 33/33,
  RT-06 12/12, DC-04 13/13, accessibility A11Y_PASS, lifecycle matrix 4/4
  `preservation=PASS` bound to `cb8ab64` via the inventory. Tamper, health,
  MT-01d, FE-05.V3, catalog, churn, RT-04.V2, witnesses and the negative controls
  are carried forward under the disclosed test-only delta. The qualification
  manifest was regenerated at `release-evidence/0.6.0/qualification-manifest-0.6.0.json`
  with the freeze-6 set labeled superseded, never rewritten. The manifest now records EOL-normalized digests per record plus the release-workflow revision, references only committed records (the mt06 result moved out of gitignored scratch), and is validated end to end by `scripts/verify_qualification_manifest.mjs` - proven on a depth-1 clone of the pushed revision (18/18 records, `MANIFEST VERIFY PASS`) and pinned by a release gate whose mutation (a scratch-state reference) fails verification. A freeze-7 byte-doctored setup refusal with both digests named is retained as the candidate's own negative control.
