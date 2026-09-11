# Localmotive qualification tests — have and need

> **Frozen snapshot (2026-09-06, 0.4.1).** The counts, FAIL/PENDING/BLOCKED labels and command examples below describe the 0.4.1 qualification cycle and are kept as history. They are not the current status. Current evidence lives in the 0.6 tracker: `docs/history/TODO-0.6.md`; version manifests are authoritative for the release number.


## Purpose

This file lists qualification checks for Localmotive.

This file uses simple language for a developer new to qualification.

Qualification proves the packaged app works on real computers.

A check passes only with direct evidence.

A check fails without direct evidence.

## How to read this file

Section 1 lists checks GitHub runs today.

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

Attestation means a JSON record of one hardware run.

P0 row means one computer type in the qualification matrix.

The matrix contains 19 P0 rows.

L2 means a direct runtime run without the packaged app.

L4 means a packaged product run on a clean computer with an attestation.

L5 means a signed public release with complete P0 evidence.

Support claim means the release notes call one row supported.

The project forbids a support claim without `L4_PASS`.

## 1. Checks GitHub runs today

The table uses one row for one check.

Column `Where` names the workflow job or local command.

Column `Code` names the exact script or file.

Column `Result` gives the result observed on 2026-09-06 unless stated otherwise.

| # | Check | What check proves | Where check runs | Code | Result |
|---|---|---|---|---|---|
| H1 | Version consistency | All version fields name the same version | CI `check`, Release `quality` | `scripts/verify_versions.mjs` | PASS for `0.4.1` |
| H2 | Immutable action references | Every GitHub Action pins to one exact commit | CI `check`, Release `quality` | `scripts/verify_workflow_pins.mjs`, `.github/workflow-actions.json` | PASS |
| H3 | Fail-fast workflow gates | No failed check hides behind a later command | CI `check`, Release `quality` | `scripts/verify_workflow_gates.mjs`, `.github/workflow-gates.json` | PASS |
| H4 | Deterministic release-gate unit tests | The gate scripts reject bad input | CI `check`, Release `quality` | `scripts/tests/release-gates.test.mjs` | PASS, 48 checks pass |
| H5 | TypeScript type check | The frontend types contain no errors | `npm run check` | `npx tsc --noEmit -p tsconfig.json` | PASS |
| H6 | Frontend Vitest suite | Pure frontend decisions behave as specified | `npm test` | `src/model.test.ts` | PASS, 50 checks pass |
| H7 | Curated catalog validation | The catalog schema, file names, sizes, digests, and Ed25519 signature stay valid | `npm run catalog:validate`, Catalog workflow `verify` | `scripts/validate_catalog.mjs`, `catalog/catalog.json`, `catalog/catalog.json.sig` | PASS, 8 models and 16 files valid |
| H8 | Branding guard | No screen shows the obsolete product name | `npm run branding:verify` | `scripts/verify_branding.mjs` | PASS |
| H9 | Icon and publisher guard | The Windows icon contains all required layers with the correct publisher | `npm run icon:verify` | `scripts/verify_icons.mjs`, `assets/app-icon.svg`, `src-tauri/icons/icon.ico` | PASS |
| H10 | Qualification policy guard | The 19-row matrix stays frozen. Undisclosed gaps never become support claims | `npm run qualification:verify` | `scripts/verify_qualification.mjs`, `release-evidence/0.4.1/qualification-matrix.json` | FAIL as designed, `win-x64-clean-account:lifecycle-not-waived` |
| H11 | Research anchor guard | The frozen research manifest matches the tracked anchor. The independent review stays complete | `npm run research:verify-anchor` | `scripts/verify_research_anchor.mjs`, `release-evidence/0.4.1/research-freeze-anchor.json` | FAIL as designed, `verification:independent-review` with status `PENDING` |
| H12 | Rust formatting | Rust code follows `cargo fmt` output | CI `check`, Release `quality` | `cargo fmt --check` in `src-tauri` | UNKNOWN in this task, no local run occurred |
| H13 | Rust linting | Rust code carries zero Clippy warnings | CI `check`, Release `quality` | `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | UNKNOWN in this task, no local run occurred |
| H14 | Rust unit and integration tests | Rust decisions behave as specified | CI `check`, Release `quality` | `cargo test --locked` in `src-tauri` | UNKNOWN in this task, no local run occurred |
| H15 | Rust documentation tests | Rust doc examples behave as specified | CI `check`, Release `quality` | `cargo test --locked --doc` with `RUSTDOCFLAGS: -D warnings` | UNKNOWN in this task, no local run occurred |
| H16 | Rust dependency audit | Rust dependencies carry no known advisories | CI `rust-audit`, Release `rust-audit` | `rustsec/audit-check@v2.0.0` with `working-directory: src-tauri` | UNKNOWN in this task, no local run occurred |
| H17 | Frontend dependency audit | JavaScript dependencies carry no moderate or higher advisories | CI `check`, Release `quality` | `npm audit --audit-level=moderate` | UNKNOWN in this task, no local run occurred |
| H18 | Production frontend build | The frontend builds for release | `npm run build`, `npm run check` | `tsc && vite build` | UNKNOWN in this task, no local build occurred |
| H19 | Package smoke on CI | The portable executable exists and starts without an immediate exit | CI `package-smoke` | `npm run tauri build`, PowerShell artifact checks, 8-second launch check | UNKNOWN in this task, no local packaging occurred |
| H20 | Packaged behavior verifier | The exact packaged app passes IPC, UI, tamper, health, cancellation, and restart checks over CDP | Release `package` | `scripts/verify_041.mjs`, output `artifacts/packaged-verification-0.4.1.json` | No PASS record. Local file shows 28 PASS and 1 FAIL for `candidate.clean-source`. |
| H21 | Candidate inventory | SHA-256 values bind the three staged files to the checksum file | Release `package` | `scripts/verify_candidate_inventory.mjs`, `artifacts/SHA256SUMS-0.4.1.txt`, `artifacts/candidate-inventory-0.4.1.json` | PASS locally on 2026-09-06 for 3 artifacts. No Release record exists yet. |
| H22 | Authenticode signature gate | Every staged file carries a valid timestamped signature | Release `publish` | `signtool.exe verify /pa /all` plus `Get-AuthenticodeSignature` with status `Valid` and a timestamp | BLOCKED. `release-evidence/0.4.1/approvals.json` records `USER_CONFIRMED_BLOCKED_PENDING_INSTALLATION`. |
| H23 | Public release read-back | The public download matches the verified candidate | Release `publish` | `gh release view`, `gh release download`, `cmp`, `sha256sum -c` | NOT RUN. Publication has no authority yet. |
| H24 | Catalog signing guard | The signing workflow never exposes the private key. The signing workflow never pushes to `main` | Release-gate checks | `release-evidence/0.4.1/catalog-signing.json`, `scripts/tests/release-gates.test.mjs` catalog workflow checks | PASS for the recorded run `33983819073`. |
| H25 | Hardware qualify on self-hosted runner | Zen 5 plus Blackwell host proof and Sandbox lifecycle run on version tags during the night window | `.github/workflows/hardware-qualify.yml`, jobs `hardware-qualify` and `clean-account-lifecycle` | Runner `DESKTOP-HPTF57N-zen5-blackwell`, `scripts/sandbox/host-run-lifecycle.ps1`, `scripts/sandbox/run-lifecycle-in-sandbox.ps1` | NOT RUN. Zero runs exist. Directory `release-evidence/0.4.1/attestations/` is absent. |

Notes on rows H10, H11, H20, H21, H22, and H25 follow.

If a guard reports FAIL, fix the missing proof for the guard.

Do not change the guard to obtain PASS.

If signing lacks authority, keep the gate BLOCKED.

## 2. Proofs the project still needs

The table uses one row for one missing proof.

Column `Why` states the user-visible risk in simple words.

Column `Gate` names the check that stays red until the proof exists.

| # | Missing proof | Why the proof matters | Gate that stays red | Required action |
|---|---|---|---|---|
| N1 | Zero L4 attestations exist for 19 P0 rows | No buyer trusts a support claim without a packaged run on the exact hardware | `scripts/verify_qualification.mjs` | Record one attestation per P0 row with the versioned schema |
| N2 | Directory `release-evidence/0.4.1/attestations/` is absent | The qualifier has nowhere to store hardware proofs | `scripts/verify_qualification.mjs` | Create one JSON attestation per completed hardware run |
| N3 | 9 rows carry status `UNKNOWN` with evidence `NONE` | Nine computer types have no check at all | Qualification matrix | Run packaged checks on AMD Zen 2, AMD Zen 4, Intel pre-AVX2, Intel AVX2, NVIDIA Pascal, AMD RDNA 2, AMD Ryzen AI APU, Intel Arc B-series, and a clean account |
| N4 | 5 rows carry status `UPSTREAM_ONLY` | Upstream CI does not prove the Localmotive package works | Qualification matrix | Run packaged checks on NVIDIA Turing, Ampere, Ada, Intel Iris Xe, and Intel Arc A-series |
| N5 | 2 rows carry status `UPSTREAM_FAILURE` | A failed upstream job proves nothing about the Localmotive package | Qualification matrix | Run packaged checks on AMD RDNA 3 and AMD RDNA 4 after the upstream failure is understood |
| N6 | 3 rows carry status `DIRECT_L2` only | A direct runtime run does not prove the packaged app works | Qualification matrix | Upgrade AMD Zen 5 CPU, NVIDIA Blackwell CUDA, and NVIDIA Blackwell Vulkan from L2 to L4 with packaged attestations. The night runner covers this host. |
| N7 | Clean-account installer lifecycle is absent | No evidence shows install, launch, update, and uninstall on a fresh computer | `win-x64-clean-account:lifecycle-not-waived` | Run NSIS install and launch in Windows Sandbox through `hardware-qualify.yml` job `clean-account-lifecycle` |
| N8 | MSI installer lifecycle is absent | No evidence shows the MSI path works | Phase 6 acceptance | Run MSI install and launch in Windows Sandbox through `hardware-qualify.yml` job `clean-account-lifecycle` |
| N9 | Windows 10 packaged evidence is absent | The scope names Windows 10 without a Windows 10 run | Phase 6 acceptance | Run install and launch on every claimed Windows version |
| N10 | Update check from 0.4.0 is absent | No evidence shows an old install updates cleanly | Phase 6 acceptance | Install 0.4.0, then update to the 0.4.1 candidate in Sandbox through `hardware-qualify.yml` job `clean-account-lifecycle` |
| N11 | Both uninstallers lack evidence | No evidence shows the computer returns to a clean state | Phase 6 acceptance | Run both NSIS and MSI uninstallers in Sandbox through `hardware-qualify.yml` job `clean-account-lifecycle` |
| N12 | Packaged `verify_041` has no PASS record | No evidence shows the current candidate passes all packaged checks | Release `package` | Build the candidate from a clean checkout, then run `node scripts/verify_041.mjs 10041` with an isolated profile |
| N13 | Candidate inventory has no Release record | No Release evidence binds names, sizes, and digests for 0.4.1 | Release `package` | Build MSI, NSIS setup, and portable files, then run `node scripts/verify_candidate_inventory.mjs` |
| N14 | Authenticode signing has no PASS record | The files ship unsigned and Windows shows an unknown publisher | Release `publish` | Wait for SignPath Foundation approval, then wire signing into Release and verify with `signtool.exe` and `Get-AuthenticodeSignature`. Do not buy a personal cert unless SignPath declines or stays quiet about 2 weeks. |
| N15 | Independent review remains `PENDING` | No second reviewer confirmed the frozen research tree | `scripts/verify_research_anchor.mjs` | Complete the independent review and track the report file |
| N16 | Public read-back has no PASS record | No evidence shows the public download equals the verified candidate | Release `publish` | Publish only after explicit authority, then run the `gh release view` and `sha256sum -c` steps |
| N17 | Human approval for publication is absent | Publication without approval violates release policy | Phase 6 gate map | Request explicit approval before signing or publication |
| N18 | Fresh Rust result is absent in this task | No fresh log shows whether Rust code still passes | `cargo test --locked` | Run formatting, Clippy, Rust checks, and doc checks from a clean checkout |
| N19 | `L5 RELEASE` claim is forbidden today | A release claim requires signing plus complete P0 evidence | Qualification policy | Do not claim `L5 RELEASE` until N13, N14, N1, and N16 all pass |

If hardware is unavailable, keep the row disclosed.

Do not promote a missing row to `L4_PASS`.

Do not claim `SUPPORTED` for a row without an attestation.

## 3. What to do next

Complete proofs in the following order.

The order minimizes rework.

Each step unblocks the next step.

1. Run formatting, Clippy, Rust checks, and doc checks from a clean checkout.
2. Build the portable executable, NSIS setup, and MSI.
3. Run `verify_041` against the exact candidate with an isolated profile.
4. Run the candidate inventory script and retain the three checksum-bound files.
5. Run NSIS and MSI lifecycle checks in Windows Sandbox.
6. Record L4 attestations for each completed hardware run.
7. Complete the independent research review and track the report file.
8. Request signing authority only after steps 1 through 7 pass.
9. Request publication authority only after signing passes.

To run the fast local gates, execute each command separately.

Use `node scripts/verify_versions.mjs` for versions; the script reads the three manifests (`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`).

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

- Workflows: `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `.github/workflows/catalog.yml`, `.github/workflows/hardware-qualify.yml`
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
- Plan and acceptance criteria: `docs/history/TODO-0.4.1.md`
- Attempt review: `REPORT-0.4.1.md`
- Changelog limitations: `CHANGELOG.md`
- Supported platform claims: `README.md` section `Supported platforms`
- Hardware workflow: `.github/workflows/hardware-qualify.yml`, jobs `hardware-qualify` and `clean-account-lifecycle`
- Sandbox scripts: `scripts/sandbox/host-run-lifecycle.ps1`, `scripts/sandbox/run-lifecycle-in-sandbox.ps1`
- Night runner: `DESKTOP-HPTF57N-zen5-blackwell`, online 01:00-06:00 Asia/Dubai, offline outside that window by design
