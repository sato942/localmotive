# Localmotive qualification tests — have and need

## Purpose

This file lists qualification tests for Localmotive.
This file uses simple language for a developer new to qualification.
Qualification proves the packaged app works on real computers.
A qualification test passes only with direct evidence.
A qualification test fails without direct evidence.

## How to read this file

Section 1 lists tests GitHub runs today.
Section 2 lists proofs the project still needs.
Section 3 gives the next actions in order.
Section 4 lists exact evidence sources.
All paths use repository root as base.
All commands preserve exact text.

## Terms in one paragraph each

CI means GitHub Actions on every push and pull request to `main`.
Release means GitHub Actions on a version tag such as `v0.4.1`.
Candidate means the exact packaged files for one version.
Evidence means a file or log that proves one behavior.
Attestation means a signed JSON record of one hardware run.
P0 row means one computer type in the qualification matrix.
The matrix contains 19 P0 rows.
L2 means a direct runtime run without the packaged app.
L4 means a packaged product run on a clean computer with an attestation.
L5 means a signed public release with complete P0 evidence.
Support claim means the release notes call one row supported.
The project forbids a support claim without `L4_PASS`.

## 1. Tests GitHub runs today

The table uses one row for one check.
Column `Where` names the workflow job or local command.
Column `Code` names the exact script or file.
Column `Today` gives the result observed on 2026-09-06 unless stated otherwise.

| # | Test | What the test proves in simple words | Where | Code | Today |
|---|---|---|---|---|---|
| H1 | Version consistency | All version fields name the same version | CI `check`, Release `quality` | `scripts/verify_versions.mjs` | PASS for `0.4.1` |
| H2 | Immutable action references | Every GitHub Action pins to one exact commit | CI `check`, Release `quality` | `scripts/verify_workflow_pins.mjs`, `.github/workflow-actions.json` | PASS |
| H3 | Fail-fast workflow gates | No failed test can hide behind a later command | CI `check`, Release `quality` | `scripts/verify_workflow_gates.mjs`, `.github/workflow-gates.json` | PASS |
| H4 | Deterministic release-gate unit tests | The gate scripts reject bad input | CI `check`, Release `quality` | `scripts/tests/release-gates.test.mjs` | PASS, 45 tests pass |
| H5 | TypeScript type check | The frontend types contain no errors | `npm run check` | `npx tsc --noEmit -p tsconfig.json` | PASS |
| H6 | Frontend Vitest suite | Pure frontend decisions behave as specified | `npm test` | `src/model.test.ts` | PASS, 50 tests pass |
| H7 | Curated catalog validation | The catalog schema, file names, sizes, digests, and Ed25519 signature are valid | `npm run catalog:validate`, Catalog workflow `verify` | `scripts/validate_catalog.mjs`, `catalog/catalog.json`, `catalog/catalog.json.sig` | PASS, 8 models and 16 files valid |
| H8 | Branding guard | No screen shows the obsolete product name | `npm run branding:verify` | `scripts/verify_branding.mjs` | PASS |
| H9 | Icon and publisher guard | The Windows icon contains all required layers and the correct publisher | `npm run icon:verify` | `scripts/verify_icons.mjs`, `assets/app-icon.svg`, `src-tauri/icons/icon.ico` | PASS |
| H10 | Qualification policy guard | The 19-row matrix stays frozen and undisclosed gaps never become support claims | `npm run qualification:verify` | `scripts/verify_qualification.mjs`, `release-evidence/0.4.1/qualification-matrix.json` | FAIL as designed, `win-x64-clean-account:lifecycle-not-waived` |
| H11 | Research anchor guard | The frozen research manifest matches the tracked anchor and the independent review is complete | `npm run research:verify-anchor` | `scripts/verify_research_anchor.mjs`, `release-evidence/0.4.1/research-freeze-anchor.json` | FAIL as designed, `verification:independent-review` with status `PENDING` |
| H12 | Rust formatting | Rust code follows `cargo fmt` output | CI `check`, Release `quality` | `cargo fmt --check` in `src-tauri` | UNKNOWN in this task, last report records drift in `catalog.rs` and `download.rs` |
| H13 | Rust linting | Rust code has zero Clippy warnings | CI `check`, Release `quality` | `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | UNKNOWN in this task, blocked by a compile error in the last report |
| H14 | Rust unit and integration tests | Rust decisions behave as specified | CI `check`, Release `quality` | `cargo test --locked` in `src-tauri`, 362 `#[test]` functions across 17 modules | UNKNOWN in this task, last report records compiler error `E0425` at `src-tauri/src/runtime.rs:5584` |
| H15 | Rust documentation tests | Rust doc examples behave as specified | CI `check`, Release `quality` | `cargo test --locked --doc` with `RUSTDOCFLAGS: -D warnings` | UNKNOWN in this task |
| H16 | Rust dependency audit | Rust dependencies have no known advisories | CI `rust-audit`, Release `rust-audit` | `rustsec/audit-check@v2.0.0` with `working-directory: src-tauri` | UNKNOWN in this task, no local run performed |
| H17 | Frontend dependency audit | JavaScript dependencies have no moderate or higher advisories | CI `check`, Release `quality` | `npm audit --audit-level=moderate` | UNKNOWN in this task, no local run performed |
| H18 | Production frontend build | The frontend builds for release | `npm run build`, `npm run check` | `tsc && vite build` | UNKNOWN in this task, no local build performed |
| H19 | Package smoke on CI | The portable executable exists and starts without an immediate exit | CI `package-smoke` | `npm run tauri build`, PowerShell artifact checks, 8-second launch check | UNKNOWN in this task, no local packaging performed |
| H20 | Packaged behavior verifier | The exact packaged app passes IPC, UI, tamper, health, cancellation, and restart checks over CDP | Release `package` | `scripts/verify_041.mjs`, 25 `runCheck` calls plus schema and error guards, output `artifacts/packaged-verification-0.4.1.json` | UNKNOWN in this task, no packaged candidate exists |
| H21 | Candidate inventory | SHA-256 values bind the three staged files to the checksum file | Release `package` | `scripts/verify_candidate_inventory.mjs`, `artifacts/SHA256SUMS-0.4.1.txt`, `artifacts/candidate-inventory-0.4.1.json` | FAIL as designed without a build, script reports missing checksum input |
| H22 | Authenticode signature gate | Every staged file carries a valid timestamped signature | Release `publish` | `signtool.exe verify /pa /all` plus `Get-AuthenticodeSignature` with status `Valid` and a timestamp | BLOCKED, SignPath Foundation application submitted, no approval or CI wiring yet |
| H23 | Public release read-back | The public download matches the verified candidate | Release `publish` | `gh release view`, `gh release download`, `cmp`, `sha256sum -c` | UNKNOWN in this task, publication has no authority yet |
| H24 | Catalog signing guard | The signing workflow never exposes the private key and never pushes to `main` | Release-gate tests | `catalog-signing.json`, `release-gates.test.mjs` catalog workflow tests | PASS for the recorded run `33983819073` |
| H25 | Hardware qualify on self-hosted runner | Zen 5 plus Blackwell host proof and Sandbox lifecycle run on version tags during the night window | `.github/workflows/hardware-qualify.yml` on `origin/main` only, jobs `hardware-qualify` and `clean-account-lifecycle` | Runner `DESKTOP-HPTF57N-zen5-blackwell`, `scripts/sandbox/host-run-lifecycle.ps1`, `scripts/sandbox/run-lifecycle-in-sandbox.ps1` | NOT RUN, 0 runs exist, local checkout lacks the workflow, `origin/main` is 5 commits ahead |

Notes on rows H10, H11, H21, H22, and H25 follow.
If a guard reports FAIL, fix the missing proof.
Do not change the guard to obtain PASS.
If signing lacks authority, keep the gate BLOCKED.

## 2. Proofs the project still needs

The table uses one row for one missing proof.
Column `Why` states the user-visible risk in simple words.
Column `Gate` names the check that stays red until the proof exists.

| # | Missing proof | Why the proof matters | Gate that stays red | Required action |
|---|---|---|---|---|
| N1 | Zero L4 attestations exist for 19 P0 rows | No buyer can trust a support claim without a packaged run on the exact hardware | `scripts/verify_qualification.mjs` | Record one attestation per P0 row with the versioned schema |
| N2 | Directory `release-evidence/0.4.1/attestations/` is absent | The qualifier has nowhere to store hardware proofs | `scripts/verify_qualification.mjs` | Create one JSON attestation per completed hardware run |
| N3 | 9 rows have status `UNKNOWN` with evidence `NONE` | Nine computer types have no test at all | Qualification matrix | Run packaged tests on AMD Zen 2, AMD Zen 4, Intel pre-AVX2, Intel AVX2, NVIDIA Pascal, AMD RDNA 2, AMD Ryzen AI APU, Intel Arc B-series, and a clean account |
| N4 | 5 rows have status `UPSTREAM_ONLY` | Upstream CI does not prove the Localmotive package works | Qualification matrix | Run packaged tests on NVIDIA Turing, Ampere, Ada, Intel Iris Xe, and Intel Arc A-series |
| N5 | 2 rows have status `UPSTREAM_FAILURE` | A failed upstream job proves nothing about the Localmotive package | Qualification matrix | Run packaged tests on AMD RDNA 3 and AMD RDNA 4 after the upstream failure is understood |
| N6 | 3 rows have status `DIRECT_L2` only | A direct runtime run does not prove the packaged app works | Qualification matrix | Upgrade AMD Zen 5 CPU, NVIDIA Blackwell CUDA, and NVIDIA Blackwell Vulkan from L2 to L4 with packaged attestations, night runner covers this host |
| N7 | Clean-account installer lifecycle is absent | No evidence shows install, launch, update, and uninstall on a fresh computer | `win-x64-clean-account:lifecycle-not-waived` | Run NSIS install and launch in Windows Sandbox through `hardware-qualify.yml` job `clean-account-lifecycle` |
| N8 | MSI installer lifecycle is absent | No evidence shows the MSI path works | Phase 6 acceptance | Run MSI install and launch in Windows Sandbox through `hardware-qualify.yml` job `clean-account-lifecycle` |
| N9 | Windows 10 packaged evidence is absent | The scope names Windows 10 without a Windows 10 run | Phase 6 acceptance | Run install and launch on every claimed Windows version |
| N10 | Update test from 0.4.0 is absent | No evidence shows an old install updates cleanly | Phase 6 acceptance | Install 0.4.0, then update to the 0.4.1 candidate in Sandbox through `hardware-qualify.yml` job `clean-account-lifecycle` |
| N11 | Both uninstallers lack evidence | No evidence shows the computer returns to a clean state | Phase 6 acceptance | Run both NSIS and MSI uninstallers in Sandbox through `hardware-qualify.yml` job `clean-account-lifecycle` |
| N12 | Packaged `verify_041` has no PASS record | No evidence shows the current candidate passes all packaged checks | Release `package` | Build the candidate, then run `node scripts/verify_041.mjs 10041` with an isolated profile |
| N13 | Candidate inventory has no PASS record | No evidence binds names, sizes, and digests for 0.4.1 | Release `package` | Build MSI, NSIS setup, and portable files, then run `node scripts/verify_candidate_inventory.mjs` |
| N14 | Authenticode signing has no PASS record | The files ship unsigned and Windows shows an unknown publisher | Release `publish` | Wait for SignPath Foundation approval, then wire signing into Release and verify with `signtool.exe` and `Get-AuthenticodeSignature`, do not buy a personal cert unless SignPath declines or stays quiet about 2 weeks |
| N15 | Independent review remains `PENDING` | No second reviewer confirmed the frozen research tree | `scripts/verify_research_anchor.mjs` | Complete the independent review and track the report file |
| N16 | Public read-back has no PASS record | No evidence shows the public download equals the verified candidate | Release `publish` | Publish only after explicit authority, then run the `gh release view` and `sha256sum -c` steps |
| N17 | Human approval for publication is absent | Publication without approval violates release policy | Phase 6 gate map | Request explicit approval before signing or publication |
| N18 | Current Rust suite does not compile in the worktree | New code cannot close any phase while tests stay red | `cargo test --locked` | Fix name `extract_verified_zip_with_limits_and_cancel`, then run formatting, tests, and linting |
| N19 | `L5 RELEASE` claim is forbidden today | A release claim requires signing plus complete P0 evidence | Qualification policy | Do not claim `L5 RELEASE` until N13, N14, N1, and N16 all pass |

If hardware is unavailable, keep the row disclosed.
Do not promote a missing row to `L4_PASS`.
Do not claim `SUPPORTED` for a row without an attestation.

## 3. What to do next

Complete proofs in the following order.
The order minimizes rework.
Each step unblocks the next step.

1. Fix the Rust compile error and formatting drift.
2. Run formatting, Clippy, Rust tests, and doc tests from a clean checkout.
3. Build the portable executable, NSIS setup, and MSI.
4. Run `verify_041` against the exact candidate with an isolated profile.
5. Run the candidate inventory script and retain the three checksum-bound files.
6. Run NSIS and MSI lifecycle tests in Windows Sandbox.
7. Record L4 attestations for each completed hardware run.
8. Complete the independent research review and track the report.
9. Request signing authority only after steps 1 through 8 pass.
10. Request publication authority only after signing passes.

To run the fast local gates, execute each command separately.
Use `node scripts/verify_versions.mjs 0.4.1` for versions.
Use `node scripts/verify_workflow_pins.mjs` for action pins.
Use `node scripts/verify_workflow_gates.mjs` for fail-fast gates.
Use `node --test scripts/tests/release-gates.test.mjs` for gate unit tests.
Use `npx tsc --noEmit -p tsconfig.json` for types.
Use `npm test` for Vitest.
Use `node scripts/validate_catalog.mjs catalog/catalog.json` for the catalog.
Use `node scripts/verify_branding.mjs` for branding.
Use `node scripts/verify_icons.mjs` for icons.
Use `node scripts/verify_qualification.mjs` for the qualification matrix.
Use `node scripts/verify_research_anchor.mjs` for the research anchor.
Stop before signing or publication without explicit authority.

## 4. Evidence index

- Workflows: `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `.github/workflows/catalog.yml`, `.github/workflows/hardware-qualify.yml` on `origin/main` only
- Gate policy: `.github/workflow-gates.json`, `.github/workflow-actions.json`
- Gate tests: `scripts/tests/release-gates.test.mjs`
- Version gate: `scripts/verify_versions.mjs`
- Pin gate: `scripts/verify_workflow_pins.mjs`
- Workflow gate: `scripts/verify_workflow_gates.mjs`
- Packaged verifier: `scripts/verify_041.mjs`
- Inventory: `scripts/verify_candidate_inventory.mjs`
- Qualification matrix: `release-evidence/0.4.1/qualification-matrix.json`
- Qualification guard: `scripts/verify_qualification.mjs`
- Approvals and risk decision: `release-evidence/0.4.1/approvals.json`
- Research ledger: `release-evidence/0.4.1/research-verification.json`
- Research anchor: `release-evidence/0.4.1/research-freeze-anchor.json`
- Catalog signing evidence: `release-evidence/0.4.1/catalog-signing.json`
- Plan and acceptance criteria: `TODO-0.4.1.md`
- Attempt review: `REPORT-0.4.1.md`
- Changelog limitations: `CHANGELOG.md`
- Supported platform claims: `README.md` section `Supported platforms`
- Hardware workflow on `origin/main`: `.github/workflows/hardware-qualify.yml`, jobs `hardware-qualify` and `clean-account-lifecycle`
- Sandbox scripts on `origin/main`: `scripts/sandbox/host-run-lifecycle.ps1`, `scripts/sandbox/run-lifecycle-in-sandbox.ps1`
- Night runner: `DESKTOP-HPTF57N-zen5-blackwell`, online 01:00-06:00 Asia/Dubai, offline outside that window by design
