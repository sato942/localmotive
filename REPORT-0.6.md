# Localmotive 0.6.0 remediation report

**Snapshot:** 2026-09-11. Local branch `main` at `46f526e5c7d47520c1a4c48c2f05b9c2a7c1e5d2`.
**Baseline:** `v0.5.0` (source `a4b7127f739f7420232d9b6f63da693d39128d0b`).
**Range covered:** every commit between `v0.5.0` and HEAD - 166 commits, 174 files
changed, +44 644 / -5 196 lines.
**Push state:** the only commit on `origin/main` beyond `v0.5.0` is `e530371`
("docs(0.5): Phase 8 ship ledger"); the 165 remediation commits are local only and
have not been pushed. This is deliberate: the release flow plants a tag at a
resolved SHA, and the tag/publish steps are owner-gated (section 5).

This report compares the work against the authoritative tracker
[`docs/history/TODO-0.6.md`](docs/history/TODO-0.6.md), whose evidence source is
[`docs/history/localmotive-comprehensive-audit.md`](docs/history/localmotive-comprehensive-audit.md)
(audit SHA-256 `fb87c9df9fd4cffa3768b81ddd40fd55363e718456a182b4237c1bfc9054b647`).
Detailed per-item dispositions live in
[`docs/RELEASE-REVIEW-0.6.md`](docs/RELEASE-REVIEW-0.6.md); release evidence lives in
`release-evidence/0.6.0/`.

---

## 1. What the plan asked for

The audit produced **72 findings** (19 High, 43 Medium, 10 Low), **29 supplemental
packages** (S-01..S-29) for unnumbered recommendations, and **10 verification and
release gates** (G-01..G-10). The tracker translated them into **111 packages**,
**663 checkboxes**, and **1051 audit links**. Its default release policy: all 19
High findings block the stabilization release until fixed and verified; Medium/Low
items may be deferred only with a visible disposition carrying owner, reason,
residual risk, workaround, follow-up milestone, and evidence gap.

## 2. Where 0.6.0 stands against TODO-0.6

| Tracker element | Planned | Actual at HEAD |
|---|---|---|
| Packages 1-12 | Sequenced remediation of all 72 findings | All closed; every finding's implementation landed with regression tests and ledger records |
| Supplemental S-01..S-29 | Unnumbered audit recommendations | 28 closed with tests/evidence; S-25.I3 remains an explicit hardware-program deferral |
| Gates G-01..G-04, G-06, G-07 | Candidate identity, verification wave, advisories | Closed with evidence |
| Gate G-05 | Packaged Windows wave | Executed in seven batches plus a High-finding campaign; I3 residual deferred (screen reader, OS-level contrast, live providers) |
| Gate G-08 | Release decision (support matrix, disclosure, deferral review) | Closed: review + support matrix + unsigned disclosure + reconciliation; all deferrals recorded |
| Gates G-09, G-10 | Authorized publication and closeout | **Open - owner-gated** (section 5) |
| Checkboxes | 663 | 629 checked, 34 open - every open item has a written disposition |

Test suites grew from the v0.5.0-era baseline (Rust 403 tests; Vitest 80) to
**Rust 590 passed / 0 failed / 7 ignored**, **Vitest 122**, **Node gates 146/146**,
with `cargo fmt --check` and `clippy --all-targets -- -D warnings` clean at every
package closure. The release-gates suite grew from 86 cases at the first gate checkpoint to 145, and the full Node test run (release gates + catalog schema + HTTP retry contract tests) reports 146 pass / 0 fail.

## 3. The work between v0.5.0 and HEAD

Commit composition since `v0.5.0`: 58 `fix`, 16 `feat`, 18 `test`, 60 `docs`,
9 `refactor`, 2 `ci`, 1 `perf`, 2 `release:`. By area: 73 commits touched
`src-tauri/src`, 49 touched `src/` (frontend), 54 touched `scripts/`
(verifiers, fixtures, probes), 5 touched `.github/` (workflows).

### 3.1 Rust core (`src-tauri/src`)

- **Runtime trust and lifecycle** (RT): new installs publish into the primary
  `Localmotive` root while legacy `GGUF Pilot` installs are discovered and reused
  only after compiled-content verification; a centralized managed-execution
  authorization gate with a sentinel test; archive extraction bound to the
  approved file identity through a single handle; an execution-identity lease so
  a replaced runtime cannot serve a launch validated against older bytes;
  bounded `runtime.json`/`--help`/`--version` reads; hard-link refusal; Windows
  last-error capture; per-physical-GPU identity parsing.
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
  verdict with an exact recovery instruction.
- **Operations**: bounded per-run launch logs (8 MiB cap, newest-ten retention,
  failure-evidence budget), bounded OAuth callback (loopback-only, 8 KiB cap,
  single-use state), background cancellable startup, single managed-inference
  owner (`OperationCoordinator`) with generation discard, supervised process
  termination, per-child `KILL_ON_JOB_CLOSE` job objects.
- **Parsers**: bounded GGUF reader with an explicit `Limits` seam, optional
  split metadata, impossible KV-dimension rejection, 5-way deterministic
  property campaigns (splitmix64).

### 3.2 Frontend (`src/`)

- Single-flight status, server-snapshot identity, selection/identity preservation
  across rescans, run generations, cancellation pending states, cancel bound to
  its job, raw extra-args with validation, corrupt-persisted-JSON quarantine with
  an error boundary, honest empty/first-run states, path truncation with copy,
  product-name demotions, preflight staleness by input revision, workload
  defaults factory.
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

- 20+ packaged CDP probe drivers (`g05_*`), engine-level accessibility,
  high-contrast, responsive, CSP, and branding verifiers; a wrong-candidate
  negative control; template-driven catalog matrix probes.
- Release workflow: one-shot tag resolution to a full SHA, SHA-pinned actions,
  candidate-fed lifecycle jobs, evidence upload with `if: always()`, an SBOM
  step, and gate tests asserting all of it (146 cases).
- Governance: `SECURITY.md`, `CONTRIBUTING.md`, issue templates,
  `docs/EVIDENCE-MATRIX.md`, `docs/SUPPORT-MATRIX.md`, `docs/SUPPLY-CHAIN.md`,
  `docs/ARTIFACT-IDENTITY.md`, security-review limits.

## 4. Verification: what was actually run

- **Unit/integration**: every finding's regression tests, with mutation checks
  kept through the process; flaky tests were fixed at the root (thread-local
  stat mirrors, load-aware asserts) rather than pinned.
- **Packaged Windows (final candidate)**: seven G-05 batches plus a High-finding
  campaign. Highlights: real managed CUDA 13.3 install (672 MB) with seven-stage
  health PASS; default v2 workload 1 warmup + 5 trials, decode means 974.36-1002.60 tok/s
  across recorded runs (5/5 sampled each); tamper family (renamed runtime, swapped DLL, hard-link alias)
  all refused before spawn; TLS/API-key profile live (1004.76 tok/s, `/props`
  401 -> 200); catalog offline/stale/quarantine behavior; download resume
  (+7.6 MB delta); cancellation accepted at first poll; clean stop under 1 s
  with no surviving children.
- **Lifecycle**: Windows Sandbox clean-account install/launch/uninstall/update
  for the candidate and both upgrade baselines (v0.4.1, v0.5.0), with user-data
  and SQLite canary preservation verified host-side and bound to source revision
  plus candidate digests.
- **Negative controls**: wrong-version candidate bytes fail with an explicit
  identity error and *attributed* failure evidence; a corrupt model fails the
  artifact gate with zero processes; a killed child and a killed app leave no
  orphans.
- **Advisory/supply chain**: SBOM generation, duplicate-dependency report, and
  the security-review limits document.

## 5. What is NOT complete

### 5.1 Owner-gated (the only blocking remainder)

These require explicit owner authority and are prepared, not executed:

1. **Repository rulesets** - `gh api repos/sato942/localmotive/rulesets` returns
   none and `branches/main.protected` is false. Ready-to-apply payloads for a
   main-branch ruleset (require `pr-check`, forbid force pushes/deletions) and a
   `v*` tag ruleset (immutable tags) are in `docs/RELEASE-REVIEW-0.6.md`.
2. **Authorized PR runs** - a benign PR and a controlled failing-check PR would
   each consume a GitHub-hosted `pr-check` run; they are not started without
   approval (GH-01.V1/V2).
3. **Publication** - `git tag -a v0.6.0` plus the release workflow run and
   readback (G-09), then closeout (G-10). Nothing is tagged, signed, or published
   yet.

### 5.2 Explicit deferrals (visible, with the required fields)

All 34 open checkboxes are these owner-gated items plus the deferral register in
`docs/RELEASE-REVIEW-0.6.md`, including:

- **FE-05.V3** - two-profile provenance walk through the saved-manifest route.
- **G-05.I3 residuals** - Narrator/NVDA (no screen reader available), OS-level
  high-contrast (engine emulation done), live cloud/HF scenarios (no authorized
  test account).
- **S-25.I3** - the independent performance program (distributions, drift,
  calibration error, queue metrics) needs real target hardware sessions.
- **Hardware/storage cells** - seven-backend measurements, CPU-only machine
  classes, power-loss validation, HDD/SATA targets.
- **DC-04.V2** - no override UI surface exists; the command path is covered.
- **RT-07.V2, RT-04.V2, GH-06.V2, QD-02.I4, QD-03.V3, FE-03.V1/V3,**
  **FE-07.V2, FE-16.V1/V3** - scripted negative/interleaving legs not yet
  executed; each row names the evidence gap and the follow-up.

### 5.3 Not-yet-pushed state

The 165 local commits must be pushed before a tag can be planted at the
verified lineage. Until then, origin serves `v0.5.0`-era code and the release
workflow has nothing newer to build from.

## 6. Issues faced and how they were resolved

1. **Three chained defects in the flagship v2 evidence flow** - found only by
   exercising the packaged app: `preflight_model` invoked with flat fields while
   the command requires `request` (preflight could never run); the inspected
   runtime version stored the raw multi-line `--version` output, which the
   manifest rejects for control characters; warm-cache trials recorded only the
   evaluated prompt slice, so every warm trial failed the declared-count
   contract. All three were fixed regression-first with caught mutations and
   re-verified packaged (1002.60 tok/s, 5/5 sampled). The lesson recorded in the
   tracker: unit fixtures had encoded the same wrong shapes, so only the packaged
   flow caught them.
2. **Orphaned inference process** - app kill left a `llama-server` alive because
   `process-wrap` defaulted to `kill_on_drop=false` on a shared job with
   `ACCESS_DENIED`. Fixed with a per-child `KILL_ON_JOB_CLOSE` job and
   `TerminateJobObject`; proven by killing the app mid-run and counting zero
   children.
3. **TLS traps** - a CN-only certificate fails SAN validation, and the common
   `openssl req -x509` default emits `CA:TRUE`, which rustls refuses as
   `CaUsedAsEndEntity`; the health loop then retried silently for the full
   timeout. Fixed with a fail-fast `health_tls` verdict carrying an exact
   recovery instruction, and a timeout that names the last transport error.
4. **Signed catalog versus label migration** - correcting quant labels changed
   embedded-signature bytes, so the data file could not be edited without an
   owner-run resign. The migration pipeline was proved on a copy (175/1417
   labels) and the committed catalog was left untouched by design.
5. **Multi-instance probe confusion** - a kill pattern matched `localmotive.exe`
   but the portable binary is `localmotive-portable.exe`, so several app
   instances stacked during probing. Identified, cleaned, and the affected
   observations re-verified single-instance; the corrected kill targets
   `localmotive*`.
6. **Harness and fixture lessons** - background-wrapped Node drivers died with
   "stdin is not a tty" (foreground runs with `< /dev/null` fixed it); WebView2
   cold start exceeded 30 s under sandbox load (CDP readiness gates added);
   MSI-installed bytes differ from NSIS-installed bytes for the same build
   (per-family digest consistency adopted instead of a single expected hash);
   `Get-FileHash` needs the Windows PowerShell 5.1 module path isolated from
   PS7; a DLL tamper probe hit EBUSY while a child held the DLL (kill children,
   then tamper).
7. **Flakes fixed at the root** - a global verification-bytes counter polluted
   parallel tests (thread-local mirror), a load-sensitive download count assert
   was hardened, and the Windows fixture retry pattern replaced a racy bind.
8. **Gates versus improvements** - release-gate tests that snapshotted old
   harness text or README wording failed when the behavior improved (for
   example the enriched failure evidence and the support-matrix wording); each
   was updated to assert the stronger contract rather than reverted.
9. **Tracker integrity incidents** - a section-boundary duplication after an
   automated edit was caught by the structural checker, the file was restored,
   and the record re-applied; the header status table was reconciled late after
   it had gone stale.

## 7. Release identity and next steps

Final candidate (re-cut after all fixes, re-bound):

- portable `localmotive.exe` sha256 `075daa54027c7234d36b5ff869eb6ac4b264c5e4abe14db3de944944d4b34f2a`
- MSI `Localmotive_0.6.0_x64_en-US.msi` sha256 `2dd036c6a9397e0491e8ebb276af187bf0c9701c8f176c56cba42e6f4bd0512d`
- NSIS `Localmotive_0.6.0_x64-setup.exe` sha256 `a4d14496d14a4f98ffcd1e9c9ae5d78f36dab74d309d86ed6e2dfdb38eb2b7de`
- source code state `a0ed247`; lifecycle evidence re-run and bound at `6384df0`
- host attestation: Zen 5 / RTX 5090 / driver 610.74 / Windows 11 build 26100, verdict MATCH

Owner actions, in order: push `main`; apply the two rulesets; authorize the PR
runs; plant `v0.6.0` and run the release workflow; read back the published
assets and close out (G-09/G-10). The exact commands are in
`docs/RELEASE-REVIEW-0.6.md`, section "Owner package for G-09".

## 8. Honest limits of this report

- Claims about packaged behavior come from the recorded probe logs and evidence
  files, not from this document.
- The support matrix is deliberately conservative: only Windows 11 x64
  lifecycle, the RTX 5090 CUDA path, and the listed packaged scenarios are
  claimed; everything else is marked untested in `docs/SUPPORT-MATRIX.md`.
- Deferrals remain unresolved by definition; none of them may be read as passed.
- Measured numbers are scoped to this host and the shipped workloads.
