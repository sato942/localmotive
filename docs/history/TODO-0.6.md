# Localmotive 0.6 — audit remediation TODO

**Target:** `0.6.0` stabilization release  
**Status:** Planning complete; implementation in progress — Package 12 (final hardening and verification)  
**Source:** [localmotive-comprehensive-audit.md](./localmotive-comprehensive-audit.md)
**Audited source SHA:** `e530371b056cd8e049c2246dbb151aa407bf359f`  
**Release baseline reviewed by the audit:** `v0.5.0`, source `a4b7127f739f7420232d9b6f63da693d39128d0b`  
**Audit file SHA-256:** `fb87c9df9fd4cffa3768b81ddd40fd55363e718456a182b4237c1bfc9054b647`  
**Implementation start SHA:** `e530371b056cd8e049c2246dbb151aa407bf359f` (equals the audited snapshot)  
**Candidate SHA / release date:** Not yet recorded

This tracker translates the complete audit into implementation and acceptance work. It contains **72 finding packages** (19 High, 43 Medium, 10 Low), **29 supplemental packages** for unnumbered audit recommendations, and **10 verification/release gates**. Every checkbox carries a stable task ID and a direct link to its supporting audit finding or section. No task is pre-completed, and no implementation, test, GitHub setting or release change is claimed by this document.

The proposed v0.6 scope is stabilization of existing behavior before expansion. All 72 findings remain in scope. This plan treats the 19 High findings as default release blockers; that is an explicit planning policy derived from the audit, not a new security-severity scale. Medium/Low and supplemental work may be deferred only with a visible disposition, owner, rationale, residual risk and follow-up milestone. See [the audit judgment](./localmotive-comprehensive-audit.md#overall-judgment) and [release acceptance guidance](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release).

The audit distinguishes source-confirmed behavior, executed reproductions, conditional risks and missing verification. Each task preserves that distinction. A task to investigate a Windows race, provider contract or absent review is not a claim that exploitation or failure was demonstrated. Exact deadlines, quotas and schema choices identified below are implementation decisions to record and validate, not measured values invented by the audit.

## How to use this tracker

- `V06-RT-01` maps directly to audit `RT-01`; `.I1` identifies an implementation checkbox and `.V1` a verification checkbox. `V06-S-*` expands unnumbered observations; `V06-G-*` defines cross-cutting gates. These supplemental/gate IDs do not increase the audit finding count.
- FE-10 was consolidated into DC-04 in the audit; no independent FE-10 task is created. The frozen catalog clock is counted only as QD-01, separately from DC-10 immutable file revisions.
- Audit links are relative to this file. Keep `localmotive-comprehensive-audit.md` alongside `TODO-0.6.md`, or update that one filename prefix if adopting a different repository layout. Source touchpoints link to the audited SHA where the path already exists; paths labeled proposed do not yet exist in that snapshot.
- Dependencies are proposed implementation prerequisites. Work-package order is a sequencing guide; small independent fixes may proceed earlier. Complete boundary tests before a dependent refactor. No phase is marked in progress by this draft.
- A checked implementation box means that action has landed; a finding closes only when its completion criteria and relevant verification are met and its ledger record is complete. Record exact commands and observed results, including failures, skips and target-environment limits. [Audit basis](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance).
- Maintain a single current implementation status and explicit owners. Preserve v0.4.1/v0.5 historical evidence and mark superseded statements through a current status index. [Audit basis](./localmotive-comprehensive-audit.md#qd-04).

## Implementation status

Owner: `sato942` (accountable maintainer). Implementation and evidence production: v0.6 remediation work on this repository. Package status is updated as each package closes; the detailed closure records live in the [Verification ledger](#verification-ledger-v06-implementation).

| Package | Scope | Status |
|---|---|---|
| 1 | Managed runtime trust and legacy recovery (RT-01, RT-02, RT-04, RT-07) | Implementation complete; unit-verified; packaged items open in G-05 |
| 2 | Catalog restart, offline loading and recovery (DC-01, DC-03, DC-07) | Implementation complete; unit-verified (commit `e6c7f59`); packaged items open in G-04/G-05 |
| 3 | Override identity and atomic persistence (DC-04, DC-05, DC-06) | Implementation complete; unit-verified (commit `6be56e5`); packaged items open in G-04/G-05 |
| 4 | Bounded and cancellable tuning (MT-03, MT-04, RT-03) | Implementation complete; unit-verified (commit `1eaa476`); packaged Windows cancellation evidence open in G-04/G-05 |
| 5 | Benchmark protocol, export privacy and failure records (MT-01, MT-02, MT-12) | Implementation complete; unit-verified (commit `9d4c36c`); packaged b10816 acceptance open in G-04/G-05 |
| 6 | Model, runtime and operation ownership (FE-01, FE-02, FE-03, FE-05, FE-07, FE-16, MT-05) | Implementation complete; unit-verified (commits `bb9a5ab`, `411c32c`, `0045cb6`, `6da40d8`, `7d4a514`); packaged UI scenarios open in G-04 |
| 7 | Responsive operations, parser and transfer bounds (IPC-01, FE-04, RT-05, RT-06, DC-02, DC-08, DC-12) | Implementation complete; unit-verified; packaged items open in G-04/G-05/G-06 |
| 8 | Delivery sequencing and truthful verifiers (GH-01..GH-06, GH-10, QD-02, QD-03) | Not started |
| 9 | Secured local transport (MT-06) | Not started |
| 10 | Measurement identity, calibration and ranking (MT-07..MT-11, MT-13..MT-15, FE-06, FE-17) | Not started |
| 11 | Recovery, privacy, accessibility and Windows edge cases (IPC-02, CLD-01, OPS-01, FE-08..FE-15, DC-11, RT-08, RT-09) | Not started |
| 12 | Curation, documentation and maintenance (DC-09, DC-10, QD-01, QD-04..QD-06, GH-07..GH-09) | Not started |
| S | Supplemental packages (S-01..S-29) | Not started |
| G | Verification/release gates (G-01..G-10) | G-01 in progress |

Baseline (recorded 2026-09-11, start SHA `e530371`): `npx tsc --noEmit` PASS; `npm test` 80/80 PASS; `npm run build` PASS; `cargo fmt --check` PASS; `cargo clippy --all-targets -- -D warnings` PASS; `cargo test` 403 passed / 0 failed / 2 ignored. Log: `.hermes-0.6/baseline.log` (local working note; not part of the repository).

## Navigation

- [Sequencing](#sequencing)
- [Complete finding coverage](#complete-finding-coverage)
- [Runtime](#runtime-tasks)
- [Catalog and downloads](#catalog-and-download-tasks)
- [Measurement and tuning](#measurement-and-tuning-tasks)
- [Frontend](#frontend-tasks)
- [IPC, cloud and operations](#ipc-cloud-and-operations-tasks)
- [Delivery and governance](#delivery-and-governance-tasks)
- [Quality and documentation](#quality-and-documentation-tasks)
- [Supplemental recommendations](#supplemental-audit-recommendations)
- [Verification and release gates](#verification-and-release-gates)
- [Minimum regression matrix](#minimum-regression-matrix)
- [Additional-observation coverage](#additional-observation-coverage)
- [Evidence and deferrals](#evidence-and-deferral-ledgers)

## Sequencing

These work packages follow [the audit remediation order](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance), with legacy runtime recovery moved alongside the shared trust work because later runtime-verification changes depend on its root-selection contract. Begin with [V06-G-01](#v06-g-01) and use [V06-G-02](#v06-g-02) throughout. Supplemental work should be attached to the relevant package rather than hidden inside unrelated fixes.

| Order | Work package | Finding tasks |
|---|---|---|
| 1 | Managed runtime trust and legacy recovery | [V06-RT-01](#v06-rt-01), [V06-RT-02](#v06-rt-02), [V06-RT-04](#v06-rt-04), [V06-RT-07](#v06-rt-07) |
| 2 | Catalog restart, offline loading and recovery | [V06-DC-01](#v06-dc-01), [V06-DC-03](#v06-dc-03), [V06-DC-07](#v06-dc-07) |
| 3 | Override identity and atomic persistence | [V06-DC-04](#v06-dc-04), [V06-DC-05](#v06-dc-05), [V06-DC-06](#v06-dc-06) |
| 4 | Bounded and cancellable tuning | [V06-MT-03](#v06-mt-03), [V06-MT-04](#v06-mt-04), [V06-RT-03](#v06-rt-03) |
| 5 | Benchmark protocol, export privacy and failure records | [V06-MT-01](#v06-mt-01), [V06-MT-02](#v06-mt-02), [V06-MT-12](#v06-mt-12) |
| 6 | Model, runtime and operation ownership | [V06-FE-01](#v06-fe-01), [V06-FE-02](#v06-fe-02), [V06-FE-03](#v06-fe-03), [V06-FE-05](#v06-fe-05), [V06-FE-07](#v06-fe-07), [V06-FE-16](#v06-fe-16), [V06-MT-05](#v06-mt-05) |
| 7 | Responsive operations, parser and transfer bounds | [V06-IPC-01](#v06-ipc-01), [V06-FE-04](#v06-fe-04), [V06-RT-05](#v06-rt-05), [V06-RT-06](#v06-rt-06), [V06-DC-02](#v06-dc-02), [V06-DC-08](#v06-dc-08), [V06-DC-12](#v06-dc-12) |
| 8 | Delivery sequencing and truthful verifiers | [V06-GH-01](#v06-gh-01), [V06-GH-02](#v06-gh-02), [V06-GH-03](#v06-gh-03), [V06-GH-04](#v06-gh-04), [V06-GH-05](#v06-gh-05), [V06-GH-06](#v06-gh-06), [V06-GH-10](#v06-gh-10), [V06-QD-02](#v06-qd-02), [V06-QD-03](#v06-qd-03) |
| 9 | Secured local transport | [V06-MT-06](#v06-mt-06) |
| 10 | Measurement identity, calibration and ranking | [V06-MT-07](#v06-mt-07), [V06-MT-08](#v06-mt-08), [V06-MT-09](#v06-mt-09), [V06-MT-10](#v06-mt-10), [V06-MT-11](#v06-mt-11), [V06-MT-13](#v06-mt-13), [V06-MT-14](#v06-mt-14), [V06-MT-15](#v06-mt-15), [V06-FE-06](#v06-fe-06), [V06-FE-17](#v06-fe-17) |
| 11 | Recovery, privacy, accessibility and Windows edge cases | [V06-IPC-02](#v06-ipc-02), [V06-CLD-01](#v06-cld-01), [V06-OPS-01](#v06-ops-01), [V06-FE-08](#v06-fe-08), [V06-FE-09](#v06-fe-09), [V06-FE-11](#v06-fe-11), [V06-FE-12](#v06-fe-12), [V06-FE-13](#v06-fe-13), [V06-FE-14](#v06-fe-14), [V06-FE-15](#v06-fe-15), [V06-DC-11](#v06-dc-11), [V06-RT-08](#v06-rt-08), [V06-RT-09](#v06-rt-09) |
| 12 | Curation, documentation and maintenance | [V06-DC-09](#v06-dc-09), [V06-DC-10](#v06-dc-10), [V06-QD-01](#v06-qd-01), [V06-QD-04](#v06-qd-04), [V06-QD-05](#v06-qd-05), [V06-QD-06](#v06-qd-06), [V06-GH-07](#v06-gh-07), [V06-GH-08](#v06-gh-08), [V06-GH-09](#v06-gh-09) |

The release path is [V06-G-03](#v06-g-03), [V06-G-04](#v06-g-04), [V06-G-05](#v06-g-05), [V06-G-06](#v06-g-06), [V06-G-07](#v06-g-07), [V06-G-08](#v06-g-08), [V06-G-09](#v06-g-09), [V06-G-10](#v06-g-10). Gate dependencies state which evidence must already exist; listing the path here does not authorize or perform publication.

## Complete finding coverage

Each audit finding appears exactly once as a primary package. All statuses are **Not started**. Owners, implementation commits and closure evidence are intentionally unassigned until work begins.

| Audit | Priority | Audit finding | v0.6 task | Order |
|---|---|---|---|---|
| [RT-01](./localmotive-comprehensive-audit.md#rt-01) | High | New runtime installs can enter an unrecoverable legacy-directory launch loop | [V06-RT-01](#v06-rt-01) | 1 |
| [RT-02](./localmotive-comprehensive-audit.md#rt-02) | High | Managed trust checks are bypassed by tuning preparation and the older runtime-health command | [V06-RT-02](#v06-rt-02) | 1 |
| [RT-03](./localmotive-comprehensive-audit.md#rt-03) | Medium | Health cancellation is ignored during an in-flight completion request | [V06-RT-03](#v06-rt-03) | 4 |
| [RT-04](./localmotive-comprehensive-audit.md#rt-04) | Medium | Content verification does not retain file protection through executable launch | [V06-RT-04](#v06-rt-04) | 1 |
| [RT-05](./localmotive-comprehensive-audit.md#rt-05) | Medium | Several bounded-input claims are bypassed by error or discovery paths | [V06-RT-05](#v06-rt-05) | 7 |
| [RT-06](./localmotive-comprehensive-audit.md#rt-06) | Medium | Runtime selection and listing repeatedly hash gigabytes of installed files without cancellation | [V06-RT-06](#v06-rt-06) | 7 |
| [RT-07](./localmotive-comprehensive-audit.md#rt-07) | Medium | The archive replacement regression exercises a helper omitted from production | [V06-RT-07](#v06-rt-07) | 1 |
| [RT-08](./localmotive-comprehensive-audit.md#rt-08) | Low | Windows last-error status is read after another Win32 call | [V06-RT-08](#v06-rt-08) | 11 |
| [RT-09](./localmotive-comprehensive-audit.md#rt-09) | Medium | Identical GPU names are matched by enumeration order, not physical identity | [V06-RT-09](#v06-rt-09) | 11 |
| [DC-01](./localmotive-comprehensive-audit.md#dc-01) | High | Persisted catalog refresh cooldown makes the catalog empty after app restart | [V06-DC-01](#v06-dc-01) | 2 |
| [DC-02](./localmotive-comprehensive-audit.md#dc-02) | High | HTTP 200 fallback rejects every range-ignoring file larger than 8 MiB | [V06-DC-02](#v06-dc-02) | 7 |
| [DC-03](./localmotive-comprehensive-audit.md#dc-03) | Medium | Catalog refresh rebuilds healthy mirrors and skips corruption recovery | [V06-DC-03](#v06-dc-03) | 2 |
| [DC-04](./localmotive-comprehensive-audit.md#dc-04) | Medium | User-added catalog entries cannot be downloaded and disappear from filtering | [V06-DC-04](#v06-dc-04) | 3 |
| [DC-05](./localmotive-comprehensive-audit.md#dc-05) | Medium | Override UPSERTs can replace curated provenance, steal files, and leave mislabeled rows | [V06-DC-05](#v06-dc-05) | 3 |
| [DC-06](./localmotive-comprehensive-audit.md#dc-06) | Medium | Override edits are not replacement operations and lack atomicity/bounds | [V06-DC-06](#v06-dc-06) | 3 |
| [DC-07](./localmotive-comprehensive-audit.md#dc-07) | Medium | Some network/cache failures bypass the promised last-good catalog fallback | [V06-DC-07](#v06-dc-07) | 2 |
| [DC-08](./localmotive-comprehensive-audit.md#dc-08) | High | GGUF byte limit does not sufficiently bound heap growth or parser work | [V06-DC-08](#v06-dc-08) | 7 |
| [DC-09](./localmotive-comprehensive-audit.md#dc-09) | Medium | Quantization labels are extracted from arbitrary final filename suffixes | [V06-DC-09](#v06-dc-09) | 12 |
| [DC-10](./localmotive-comprehensive-audit.md#dc-10) | Low | Mutable file revisions weaken reproducible catalog availability | [V06-DC-10](#v06-dc-10) | 12 |
| [DC-11](./localmotive-comprehensive-audit.md#dc-11) | Medium | Completion publication can overwrite a file created during transfer; local tampering defense remains incomplete | [V06-DC-11](#v06-dc-11) | 11 |
| [DC-12](./localmotive-comprehensive-audit.md#dc-12) | Low | Resume checkpoints are durable before data is guaranteed durable, and final hashing is uncancellable | [V06-DC-12](#v06-dc-12) | 7 |
| [MT-01](./localmotive-comprehensive-audit.md#mt-01) | High | Default warm-cache benchmark rejects legitimate prompt-cache hits | [V06-MT-01](#v06-mt-01) | 5 |
| [MT-02](./localmotive-comprehensive-audit.md#mt-02) | High | Share export leaks raw trial errors through the summary | [V06-MT-02](#v06-mt-02) | 5 |
| [MT-03](./localmotive-comprehensive-audit.md#mt-03) | High | No-op proposals can cause unlimited paid advisor requests | [V06-MT-03](#v06-mt-03) | 4 |
| [MT-04](./localmotive-comprehensive-audit.md#mt-04) | High | Cancelling AI tuning does not cancel its lifecycle | [V06-MT-04](#v06-mt-04) | 4 |
| [MT-05](./localmotive-comprehensive-audit.md#mt-05) | High | Benchmarks and quality checks can outlive their server identity | [V06-MT-05](#v06-mt-05) | 6 |
| [MT-06](./localmotive-comprehensive-audit.md#mt-06) | Medium | Local TLS/authentication configuration and internal transports disagree | [V06-MT-06](#v06-mt-06) | 9 |
| [MT-07](./localmotive-comprehensive-audit.md#mt-07) | Medium | Calibration compatibility does not identify the actual execution shape | [V06-MT-07](#v06-mt-07) | 10 |
| [MT-08](./localmotive-comprehensive-audit.md#mt-08) | Medium | Repeated clicks on one result manufacture independent calibration anchors | [V06-MT-08](#v06-mt-08) | 10 |
| [MT-09](./localmotive-comprehensive-audit.md#mt-09) | Medium | Quality evidence is not tied to full model/configuration identity | [V06-MT-09](#v06-mt-09) | 10 |
| [MT-10](./localmotive-comprehensive-audit.md#mt-10) | Medium | Missing metrics invalidate the claimed Pareto frontier | [V06-MT-10](#v06-mt-10) | 10 |
| [MT-11](./localmotive-comprehensive-audit.md#mt-11) | Medium | AI tuning does not actually benchmark at the requested context workload | [V06-MT-11](#v06-mt-11) | 10 |
| [MT-12](./localmotive-comprehensive-audit.md#mt-12) | Medium | Rich launch failures can destroy the benchmark record they should explain | [V06-MT-12](#v06-mt-12) | 5 |
| [MT-13](./localmotive-comprehensive-audit.md#mt-13) | Medium | Validation is fragmented and does not enforce complete-record consistency | [V06-MT-13](#v06-mt-13) | 10 |
| [MT-14](./localmotive-comprehensive-audit.md#mt-14) | Low | Calibration model invariants and staleness are not enforced uniformly | [V06-MT-14](#v06-mt-14) | 10 |
| [MT-15](./localmotive-comprehensive-audit.md#mt-15) | Low | Scanner shard completeness differs from artifact validation | [V06-MT-15](#v06-mt-15) | 10 |
| [FE-01](./localmotive-comprehensive-audit.md#fe-01) | High | Selected model/runtime and the profile actually launched can diverge | [V06-FE-01](#v06-fe-01) | 6 |
| [FE-02](./localmotive-comprehensive-audit.md#fe-02) | High | Tuning results are not bound to the model/runtime that produced them | [V06-FE-02](#v06-fe-02) | 6 |
| [FE-03](./localmotive-comprehensive-audit.md#fe-03) | Medium | Async cloud, GGUF metadata, port suggestions and command previews accept stale responses | [V06-FE-03](#v06-fe-03) | 6 |
| [FE-04](./localmotive-comprehensive-audit.md#fe-04) | High | Typing a profile repeatedly runs expensive synchronous native launch validation | [V06-FE-04](#v06-fe-04) | 7 |
| [FE-05](./localmotive-comprehensive-audit.md#fe-05) | High | Evidence and active measurement state disappear when leaving Benchmark | [V06-FE-05](#v06-fe-05) | 6 |
| [FE-06](./localmotive-comprehensive-audit.md#fe-06) | Medium | Preflight evidence stays visible after its input assumptions change; final adapter cannot be deselected | [V06-FE-06](#v06-fe-06) | 10 |
| [FE-07](./localmotive-comprehensive-audit.md#fe-07) | Medium | Cancellation reuses the operation busy state and clears it before the benchmark has ended | [V06-FE-07](#v06-fe-07) | 6 |
| [FE-08](./localmotive-comprehensive-audit.md#fe-08) | Medium | Raw extra-argument field cannot normally accept multiple tokens by typing | [V06-FE-08](#v06-fe-08) | 11 |
| [FE-09](./localmotive-comprehensive-audit.md#fe-09) | Medium | Corrupt persisted JSON can crash the UI; write failures can be reported as operation failures | [V06-FE-09](#v06-fe-09) | 11 |
| [FE-11](./localmotive-comprehensive-audit.md#fe-11) | Medium | Download cancellation targets the currently edited destination, not the destination of the running job | [V06-FE-11](#v06-fe-11) | 11 |
| [FE-12](./localmotive-comprehensive-audit.md#fe-12) | Medium | Model-folder and download-destination fields lose the keyboard-focus ring | [V06-FE-12](#v06-fe-12) | 11 |
| [FE-13](./localmotive-comprehensive-audit.md#fe-13) | Medium | Several controls expose incomplete names/structures to assistive technology | [V06-FE-13](#v06-fe-13) | 11 |
| [FE-14](./localmotive-comprehensive-audit.md#fe-14) | Medium | Small text colors fail 4.5:1 contrast; compact layouts leave evidence controls cramped | [V06-FE-14](#v06-fe-14) | 11 |
| [FE-15](./localmotive-comprehensive-audit.md#fe-15) | Medium | Runtime capability and status labels overstate what was established | [V06-FE-15](#v06-fe-15) | 11 |
| [FE-16](./localmotive-comprehensive-audit.md#fe-16) | Medium | Legacy controls and new evidence tools have inconsistent active-operation/result ownership | [V06-FE-16](#v06-fe-16) | 6 |
| [FE-17](./localmotive-comprehensive-audit.md#fe-17) | Low | Tested workload validators are disconnected from production UI; domain defaults/rules are duplicated | [V06-FE-17](#v06-fe-17) | 10 |
| [IPC-01](./localmotive-comprehensive-audit.md#ipc-01) | High | Normal server startup blocks the main thread and prevents stop/status during startup | [V06-IPC-01](#v06-ipc-01) | 7 |
| [IPC-02](./localmotive-comprehensive-audit.md#ipc-02) | Medium | Content Security Policy is disabled | [V06-IPC-02](#v06-ipc-02) | 11 |
| [CLD-01](./localmotive-comprehensive-audit.md#cld-01) | Medium | OAuth callback parsing lacks a complete request and resource budget | [V06-CLD-01](#v06-cld-01) | 11 |
| [OPS-01](./localmotive-comprehensive-audit.md#ops-01) | Medium | Server log writes are unbounded and reuse the same filename | [V06-OPS-01](#v06-ops-01) | 11 |
| [GH-01](./localmotive-comprehensive-audit.md#gh-01) | High | default branch has no enforced premerge verification | [V06-GH-01](#v06-gh-01) | 8 |
| [GH-02](./localmotive-comprehensive-audit.md#gh-02) | High | mutable release tags undermine stable source and evidence identity | [V06-GH-02](#v06-gh-02) | 8 |
| [GH-03](./localmotive-comprehensive-audit.md#gh-03) | High | v0.5.0 clean-account verification fails because its scheduling waits on itself | [V06-GH-03](#v06-gh-03) | 8 |
| [GH-04](./localmotive-comprehensive-audit.md#gh-04) | High | Sandbox lifecycle PASS can conceal incomplete uninstall or wrong-version upgrade | [V06-GH-04](#v06-gh-04) | 8 |
| [GH-05](./localmotive-comprehensive-audit.md#gh-05) | Medium | new v0.5 catalog and SQLite features lack packaged end-to-end release coverage | [V06-GH-05](#v06-gh-05) | 8 |
| [GH-06](./localmotive-comprehensive-audit.md#gh-06) | Medium | failure evidence is lost by the Sandbox host harness | [V06-GH-06](#v06-gh-06) | 8 |
| [GH-07](./localmotive-comprehensive-audit.md#gh-07) | Medium | support statements and historical corrections remain inconsistent with public evidence | [V06-GH-07](#v06-gh-07) | 12 |
| [GH-08](./localmotive-comprehensive-audit.md#gh-08) | Medium | supply-chain provenance is incomplete beyond checksum integrity | [V06-GH-08](#v06-gh-08) | 12 |
| [GH-09](./localmotive-comprehensive-audit.md#gh-09) | Low | governance and ongoing maintenance processes are not yet established | [V06-GH-09](#v06-gh-09) | 12 |
| [GH-10](./localmotive-comprehensive-audit.md#gh-10) | Medium | hardware “attestation” stub can declare HOST_MATCH despite a mismatched host | [V06-GH-10](#v06-gh-10) | 8 |
| [QD-01](./localmotive-comprehensive-audit.md#qd-01) | Medium | Catalog builder never advances its 90-day cutoff | [V06-QD-01](#v06-qd-01) | 12 |
| [QD-02](./localmotive-comprehensive-audit.md#qd-02) | Medium | New product flows lack automated component and orchestration contract tests | [V06-QD-02](#v06-qd-02) | 8 |
| [QD-03](./localmotive-comprehensive-audit.md#qd-03) | Medium | Packaged UI harness couples to private React internals and injects states without exercising their real transition | [V06-QD-03](#v06-qd-03) | 8 |
| [QD-04](./localmotive-comprehensive-audit.md#qd-04) | Low | Active-looking docs disagree on shipping, persistence, verification status and architecture | [V06-QD-04](#v06-qd-04) | 12 |
| [QD-05](./localmotive-comprehensive-audit.md#qd-05) | Low | Build instructions understate the Node minimum and toolchain is not fully reproducible | [V06-QD-05](#v06-qd-05) | 12 |
| [QD-06](./localmotive-comprehensive-audit.md#qd-06) | Low | Imported llama-server reference has missing provenance and broken relative links | [V06-QD-06](#v06-qd-06) | 12 |

## Runtime tasks

### V06-RT-01

**Publish new approved runtimes into a launchable primary installation root**

**Status:** Implemented; unit-verified (commit `bd33337`); packaged upgrade acceptance pending V06-G-05 · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit RT-01](./localmotive-comprehensive-audit.md#rt-01)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/runtime.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs)

**Implementation**

- [x] **V06-RT-01.I1** — Separate the destination policy for new approved installations from discovery of existing GGUF Pilot installations; select the primary Localmotive runtime root for new installs even when only the legacy directory exists. **Trace:** [Audit RT-01](./localmotive-comprehensive-audit.md#rt-01).
- [x] **V06-RT-01.I2** — Keep legacy discovery and any migration explicit, preserving existing files until the replacement has passed compiled-content verification and publication has succeeded. **Trace:** [Audit RT-01](./localmotive-comprehensive-audit.md#rt-01).
- [x] **V06-RT-01.I3** — Align install, repair, reuse, public inspection, and launch validation so a successful installation result cannot return a path that the same application categorically rejects as legacy. **Trace:** [Audit RT-01](./localmotive-comprehensive-audit.md#rt-01).
- [x] **V06-RT-01.I4** — Update the existing root-selection regression and recovery messages to express the selected policy; retain architecture, content-manifest, reparse, backup, and rollback protections. **Trace:** [Audit RT-01](./localmotive-comprehensive-audit.md#rt-01).

**Verification**

- [x] **V06-RT-01.V1** — Compose root selection, installation/publication using an inert verified fixture, and the public inspection trust gate with an empty legacy directory and absent primary directory; assert that the returned runtime is accepted. **Trace:** [Audit RT-01](./localmotive-comprehensive-audit.md#rt-01).
- [x] **V06-RT-01.V2** — Repeat the scenario with corrupt legacy records, a populated primary root, and repair/reuse requests; verify that reinstall does not reproduce the legacy rejection loop. **Trace:** [Audit RT-01](./localmotive-comprehensive-audit.md#rt-01).
- [ ] **V06-RT-01.V3** — On the supported packaged Windows application, reproduce the upgrade layout and record a successful install followed by inspection and launch, including preservation of prior files on a failed migration. **Trace:** [Audit RT-01](./localmotive-comprehensive-audit.md#rt-01).

**Complete when:** Every newly approved install or reuse result names a runtime that passes the current launch trust policy. The documented recovery action repairs the legacy upgrade scenario without requiring manual directory deletion.

**Scope / decision note:** The audit established contradictory control flow; the complete packaged Windows scenario was not executed. Legacy-location support, if retained, must require compiled-content approval rather than a location-based exception.

### V06-RT-02

**Enforce managed content authorization before every runtime probe**

**Status:** Implemented; unit-verified (commit `4081a7f`); packaged workflow acceptance pending V06-G-05 · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit RT-02](./localmotive-comprehensive-audit.md#rt-02)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/core.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs), [src-tauri/src/runtime.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs)

**Implementation**

- [x] **V06-RT-02.I1** — Centralize managed-runtime content authorization at the lowest shared execution or probe boundary so callers cannot bypass it by directly invoking core runtime inspection. **Trace:** [Audit RT-02](./localmotive-comprehensive-audit.md#rt-02).
- [x] **V06-RT-02.I2** — Route tuning preparation, the older public runtime-health command, normal inspection, and launch through that boundary before executing version, help, or device-discovery probes. **Trace:** [Audit RT-02](./localmotive-comprehensive-audit.md#rt-02).
- [x] **V06-RT-02.I3** — Authorize the complete managed installation and the selected companion executable, including llama-cli.exe and its approved DLLs, rather than validating only that the server path is a regular file. **Trace:** [Audit RT-02](./localmotive-comprehensive-audit.md#rt-02).
- [x] **V06-RT-02.I4** — Introduce an explicit verified-runtime representation or equally enforceable execution interface; keep intentionally selected external runtimes governed by an explicit separate policy and retain compiled inventory, digest, and filesystem protections. **Trace:** [Audit RT-02](./localmotive-comprehensive-audit.md#rt-02).

**Verification**

- [x] **V06-RT-02.V1** — Use inert replaced managed server and CLI binaries that write a sentinel whenever invoked; call tuning preparation and the older health workflow and assert rejection before any sentinel appears. **Trace:** [Audit RT-02](./localmotive-comprehensive-audit.md#rt-02).
- [x] **V06-RT-02.V2** — Tamper with an approved DLL while leaving the server executable unchanged and require rejection through the same user-facing workflows. **Trace:** [Audit RT-02](./localmotive-comprehensive-audit.md#rt-02).
- [ ] **V06-RT-02.V3** — Exercise ordinary managed inspection/launch and intentional external-runtime selection to verify that the centralized boundary preserves their documented behavior and does not rely on frontend validation. **Trace:** [Audit RT-02](./localmotive-comprehensive-audit.md#rt-02).

**Complete when:** No managed EXE or companion probe executes before the installation passes compiled-content authorization. Production-path regressions cover both previously bypassing workflows and detect removal or bypass of the shared guard.

**Scope / decision note:** This is a managed-content integrity defect with a prerequisite of tampered installed content or an already selected hostile runtime; the audit did not establish remote execution from an unprivileged network input alone. A later server-launch check is insufficient.

### V06-RT-03

**Cancel active health completion requests and prevent cancelled runs from passing**

**Status:** Implemented; unit-verified (commit `1eaa476`); packaged cancelled-request evidence open in G-04/G-05 · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit RT-03](./localmotive-comprehensive-audit.md#rt-03)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/health.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/health.rs), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs)

**Implementation**

- [x] **V06-RT-03.I1** — Carry the health cancellation signal into completion request execution and response-body reading instead of checking it only immediately before a blocking request. **Trace:** [Audit RT-03](./localmotive-comprehensive-audit.md#rt-03).
- [x] **V06-RT-03.I2** — Use an asynchronous cancellation selection or a supervised request worker that can terminate the contained server and interrupt pending completion work within a documented cancellation deadline. **Trace:** [Audit RT-03](./localmotive-comprehensive-audit.md#rt-03).
- [x] **V06-RT-03.I3** — Propagate Cancelled consistently when cancellation arrives during response waiting, body reading, or response completion; resolve response-versus-cancel races before recording a successful deterministic completion or overall pass. **Trace:** [Audit RT-03](./localmotive-comprehensive-audit.md#rt-03).
- [x] **V06-RT-03.I4** — Keep request/body limits, proxy-free loopback transport, process ownership checks, exact pinned completion comparison, and process/listener cleanup intact; distinguish the existing idle-server shutdown stage from verification of active-request cancellation. **Trace:** [Audit RT-03](./localmotive-comprehensive-audit.md#rt-03).

**Verification**

- [x] **V06-RT-03.V1** — Run an owned loopback fixture that accepts the completion request and withholds its response, then cancel after acceptance; assert bounded return with a Cancelled result and no passing completion stage. **Trace:** [Audit RT-03](./localmotive-comprehensive-audit.md#rt-03).
- [x] **V06-RT-03.V2** — Cancel during response-body reading and at the response-completion boundary; require deterministic terminal status and verify that a user-cancelled run cannot become an overall pass. **Trace:** [Audit RT-03](./localmotive-comprehensive-audit.md#rt-03).
- [ ] **V06-RT-03.V3** — Confirm that the contained child and loopback listener are stopped and temporary files are cleaned after cancellation, and verify that the ordinary successful health sequence still passes. **Trace:** [Audit RT-03](./localmotive-comprehensive-audit.md#rt-03).

**Complete when:** Cancellation interrupts in-flight completion work within the documented bound instead of waiting for the full 120-second request timeout. Cancelled health runs never report a successful deterministic completion or overall pass and leave no active owned server.

**Scope / decision note:** The audit confirmed missing cancellation propagation in source; it did not execute a stalled Windows request. Killing the idle server after successful completion does not demonstrate cancellation of an active HTTP request.

### V06-RT-04

**Retain verified executable and DLL protection through managed process loading**

**Status:** Implemented; unit-verified (commit `8566883`); packaged writer-race check pending V06-G-05 · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit RT-04](./localmotive-comprehensive-audit.md#rt-04)  
**Prerequisites:** [V06-RT-02](#v06-rt-02)
**Source touchpoints:** [src-tauri/src/runtime.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/health.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/health.rs)

**Implementation**

- [x] **V06-RT-04.I1** — Extend the shared verified-runtime execution boundary to return a lease that retains restrictive read-sharing handles for the approved executable and DLL inventory, rather than discarding each handle after hashing. **Trace:** [Audit RT-04](./localmotive-comprehensive-audit.md#rt-04).
- [x] **V06-RT-04.I2** — Retain and validate the relevant directory identities with the lease, and keep protection effective through process creation and loading of the files covered by the execution-identity guarantee. **Trace:** [Audit RT-04](./localmotive-comprehensive-audit.md#rt-04).
- [x] **V06-RT-04.I3** — Carry the lease through managed health context preparation, any pinned health-model download, and the subsequent CLI, benchmark, and server launches so the long preparation interval does not reopen a modification window. **Trace:** [Audit RT-04](./localmotive-comprehensive-audit.md#rt-04).
- [x] **V06-RT-04.I4** — Coordinate runtime repair and replacement with active leases, defining a clear wait or rejection outcome; preserve backup/rollback behavior and avoid replacing cryptographic approval with a last-moment path recheck. **Trace:** [Audit RT-04](./localmotive-comprehensive-audit.md#rt-04).

**Verification**

- [ ] **V06-RT-04.V1** — Use a synchronized writer that attempts to replace an approved executable after verification succeeds but before spawn; require replacement to fail while protected or rejection before an inert marker executable runs. **Trace:** [Audit RT-04](./localmotive-comprehensive-audit.md#rt-04).
- [ ] **V06-RT-04.V2** — Repeat with DLL replacement and with the health-model download deliberately delayed between context preparation and runtime execution. **Trace:** [Audit RT-04](./localmotive-comprehensive-audit.md#rt-04).
- [ ] **V06-RT-04.V3** — Exercise repair/replacement while a lease is active and after release; verify the documented outcome, intact approved content, and no abandoned locks or handles. **Trace:** [Audit RT-04](./localmotive-comprehensive-audit.md#rt-04).

**Complete when:** A managed process cannot load content substituted in the verified-to-launch interval covered by the documented guarantee. Health preparation and runtime repair obey the same verified-install lifetime policy.

**Scope / decision note:** The race requires concurrent same-user write access and was not executed on Windows; it is not a privilege-escalation claim. If this adversarial scope cannot be supported, document verification as point-in-time only and explicitly record that execution-identity hardening remains unfulfilled.

### V06-RT-05

**Apply runtime input and traversal bounds before consuming untrusted data**

**Status:** Implemented; unit-verified (`32770d2`) · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit RT-05](./localmotive-comprehensive-audit.md#rt-05)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/runtime.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs)

**Implementation**

- [x] **V06-RT-05.I1** — Reuse a bounded streaming response-body reader for runtime catalog success and error statuses, including HTTP 403, enforcing the 2 MiB limit before retaining or formatting oversized content. **Trace:** [Audit RT-05](./localmotive-comprehensive-audit.md#rt-05).
- [x] **V06-RT-05.I2** — Open discovered runtime.json records as regular protected files, check metadata from the opened handle, and read at most the 64 KiB limit plus one detection byte before parsing. **Trace:** [Audit RT-05](./localmotive-comprehensive-audit.md#rt-05).
- [x] **V06-RT-05.I3** — Count every visited filesystem entry, including empty directories, during managed-install collection; add a bounded traversal depth and reject excessive work before growing the pending traversal without limit. **Trace:** [Audit RT-05](./localmotive-comprehensive-audit.md#rt-05).
- [x] **V06-RT-05.I4** — Preserve typed failure categories, compiled installation verification, and useful bounded error messages; do not confuse the HTTP timeout or final payload validation with allocation and traversal limits. **Trace:** [Audit RT-05](./localmotive-comprehensive-audit.md#rt-05).

**Verification**

- [x] **V06-RT-05.V1** — Serve chunked HTTP 403 bodies at the configured byte limit and one byte beyond it; assert bounded retained data, early oversized-body rejection, and normal handling of a small rate-limit response. **Trace:** [Audit RT-05](./localmotive-comprehensive-audit.md#rt-05).
- [x] **V06-RT-05.V2** — Inspect an oversized sparse runtime.json record and exact-boundary valid/malformed records; verify that discovery cannot allocate the entire oversized file before rejection. **Trace:** [Audit RT-05](./localmotive-comprehensive-audit.md#rt-05).
- [x] **V06-RT-05.V3** — Build trees with excessive empty directories and excessive nesting, plus valid boundary inventories; assert bounded traversal and unchanged reparse/non-regular-entry rejection. **Trace:** [Audit RT-05](./localmotive-comprehensive-audit.md#rt-05).

**Complete when:** Every catalog-body and runtime-record path enforces its byte limit during reading, including error and discovery paths. Installed-tree verification bounds files, directories, and traversal depth with observable early rejection.

**Scope / decision note:** Denial-of-service effects were not executed. Oversized local records and directory trees require corruption or write access; the audit did not claim that ordinary GitHub 403 responses are large. Preserve existing download and compiled-payload trust controls.

### V06-RT-06

**Separate runtime discovery from cancellable and coalesced content verification**

**Status:** Implemented; unit-verified (`e8f94b4`) · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit RT-06](./localmotive-comprehensive-audit.md#rt-06)  
**Prerequisites:** [V06-RT-01](#v06-rt-01), [V06-RT-04](#v06-rt-04)
**Source touchpoints:** [src-tauri/src/runtime.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs)

**Implementation**

- [x] **V06-RT-06.I1** — Use the corrected primary-root policy to make root lookup and cheap installation discovery independent of full payload hashing; expose discovery without prematurely asserting cryptographically verified status. **Trace:** [Audit RT-06](./localmotive-comprehensive-audit.md#rt-06).
- [x] **V06-RT-06.I2** — Move expensive content verification to blocking workers with cancellation checks and progress reporting, including preparatory work used by installation, description, selection, and managed health. **Trace:** [Audit RT-06](./localmotive-comprehensive-audit.md#rt-06).
- [x] **V06-RT-06.I3** — Coalesce simultaneous verification requests for the same installation and retain results through the verified-install lease, with invalidation tied to the lifetime and identity guarantees established for execution. **Trace:** [Audit RT-06](./localmotive-comprehensive-audit.md#rt-06).
- [x] **V06-RT-06.I4** — Instrument verification jobs and bytes hashed across listing, selection, and launch so repeated work is visible; never substitute mtime/size equality alone for compiled-content integrity checks. **Trace:** [Audit RT-06](./localmotive-comprehensive-audit.md#rt-06).

**Verification**

- [x] **V06-RT-06.V1** — Assert that root lookup performs no full content scan and that simultaneous requests for one installation share a verification job without incorrectly sharing work across different installations. **Trace:** [Audit RT-06](./localmotive-comprehensive-audit.md#rt-06).
- [x] **V06-RT-06.V2** — Cancel a long verification operation and assert prompt worker termination and a truthful unverified/cancelled result; verify that changed content cannot inherit a stale verified label. **Trace:** [Audit RT-06](./localmotive-comprehensive-audit.md#rt-06).
- [ ] **V06-RT-06.V3** — Measure bytes hashed, job counts, and elapsed time for listing, selecting, and launching on a representative supported Windows host with all seven backends installed; record the environment and results. **Trace:** [Audit RT-06](./localmotive-comprehensive-audit.md#rt-06).

**Complete when:** Cheap root/discovery operations no longer hash every installed payload, and concurrent equivalent verification work is coalesced. Expensive verification remains cryptographically sound, reports progress, and responds to cancellation within its documented bound.

**Scope / decision note:** The audit summed 419 manifest entries totaling 3,752,868,564 declared bytes; it did not measure disk traffic or elapsed time. OS caching can reduce physical I/O without removing repeated hashing work. Performance changes must preserve execution trust.

### V06-RT-07

**Verify and extract approved archives through the tested production path**

**Status:** Implemented; unit-verified (commit `ad5bcd0`); symlink-swap variant pending a privileged packaged check · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit RT-07](./localmotive-comprehensive-audit.md#rt-07)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/runtime.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs)

**Implementation**

- [x] **V06-RT-07.I1** — Replace the test-only archive-verification implementation with a production verification/extraction path that the actual installer uses after download completion. **Trace:** [Audit RT-07](./localmotive-comprehensive-audit.md#rt-07).
- [x] **V06-RT-07.I2** — Open the archive with appropriate identity and sharing protections, verify approved size and SHA-256, and extract from that same opened file so reopening cannot introduce another check/use gap. **Trace:** [Audit RT-07](./localmotive-comprehensive-audit.md#rt-07).
- [x] **V06-RT-07.I3** — Remove duplicate security logic guarded solely by cfg(test), and route archive mutation regressions through the production installer or its real extraction boundary. **Trace:** [Audit RT-07](./localmotive-comprehensive-audit.md#rt-07).
- [x] **V06-RT-07.I4** — Preserve capability-relative create-new extraction, traversal/ADS/symlink rejection, decompression limits, cancellation, final compiled per-file inventory verification, and verified publication/rollback. **Trace:** [Audit RT-07](./localmotive-comprehensive-audit.md#rt-07).

**Verification**

- [x] **V06-RT-07.V1** — Replace the archive between completed download and actual production extraction with same-size changed content and with a changed-size file; assert rejection before any archive entry is extracted. **Trace:** [Audit RT-07](./localmotive-comprehensive-audit.md#rt-07).
- [ ] **V06-RT-07.V2** — Attempt symlink/reparse replacement and a path change after the archive is opened; verify that the protected original handle remains authoritative or the operation fails safely. **Trace:** [Audit RT-07](./localmotive-comprehensive-audit.md#rt-07).
- [x] **V06-RT-07.V3** — Cancel during archive verification and confirm bounded cancellation/cleanup, then run an intact approved fixture through the same production path to exercise successful extraction and final content checks. **Trace:** [Audit RT-07](./localmotive-comprehensive-audit.md#rt-07).

**Complete when:** The production installer verifies the exact protected archive bytes consumed by extraction. Regression tests fail if production archive verification is removed or bypassed; no test-only security implementation supplies the apparent guarantee.

**Scope / decision note:** The audited test coverage gap does not establish that an altered ZIP becomes an accepted runnable installation: final compiled per-file verification already rejects unapproved payloads. The new guard must supplement those controls, not replace them or merely rehash before reopening.

### V06-RT-08

**Capture original Win32 errors before closing handles**

**Status:** Implemented + unit-verified · **Priority:** Low · **Owner:** Unassigned  
**Audit trace:** [Audit RT-08](./localmotive-comprehensive-audit.md#rt-08)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/runtime.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs)

**Implementation**

- [x] **V06-RT-08.I1** — Capture GetLastError immediately after a failed FindNextStreamW call, before FindClose or any other Win32 operation, and use the captured value to recognize ERROR_HANDLE_EOF. **Trace:** [Audit RT-08](./localmotive-comprehensive-audit.md#rt-08).
- [x] **V06-RT-08.I2** — Capture the failure status from GetProcessMemoryInfo before CloseHandle, preserving the actual measurement error in the returned unknown-evidence diagnostic. **Trace:** [Audit RT-08](./localmotive-comprehensive-audit.md#rt-08).
- [x] **V06-RT-08.I3** — Use scoped RAII handle ownership or an equivalent cleanup structure that closes each acquired handle exactly once without replacing the original operation result or error. **Trace:** [Audit RT-08](./localmotive-comprehensive-audit.md#rt-08).
- [x] **V06-RT-08.I4** — Keep alternate-stream rejection and valid-file verification semantics unchanged, distinguishing ordinary enumeration completion from extra streams and genuine enumeration failures. **Trace:** [Audit RT-08](./localmotive-comprehensive-audit.md#rt-08).

**Verification**

- [x] **V06-RT-08.V1** — Add a Windows API-boundary fixture whose cleanup operation deliberately changes thread-local last-error state; assert that stream enumeration retains the pre-cleanup result. **Trace:** [Audit RT-08](./localmotive-comprehensive-audit.md#rt-08).
- [x] **V06-RT-08.V2** — Exercise a normal file with only the default data stream and a file with an extra stream; require acceptance of the former and rejection of the latter without false enumeration errors. **Trace:** [Audit RT-08](./localmotive-comprehensive-audit.md#rt-08).
- [x] **V06-RT-08.V3** — Force a process-memory measurement failure followed by cleanup that changes last error; verify that the reported diagnostic identifies the original measurement failure and handles are released. **Trace:** [Audit RT-08](./localmotive-comprehensive-audit.md#rt-08).

**Complete when:** Stream and memory-probe decisions use the error captured from the failed operation, independent of later cleanup calls. Valid default-stream files remain accepted and alternate-stream files remain rejected on the supported Windows test environment.

**Scope / decision note:** The audit established API-ordering misuse, not a reproduced failure on every supported Windows build. Do not describe FindClose as always overwriting GetLastError; the repair removes reliance on whether a particular cleanup implementation preserves it.

### V06-RT-09

**Map GPU telemetry and health devices by physical identity**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit RT-09](./localmotive-comprehensive-audit.md#rt-09)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/runtime.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs), [src-tauri/src/health.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/health.rs)

**Implementation**

- [x] **V06-RT-09.I1** — Extend NVIDIA probing to obtain a stable physical identifier, such as PCI location or UUID with an appropriate Windows mapping, and retain the identifier alongside driver/capacity/usage observations. **Trace:** [Audit RT-09](./localmotive-comprehensive-audit.md#rt-09).
- [x] **V06-RT-09.I2** — Join NVIDIA observations to DXGI adapters using a verified physical mapping rather than pairing the first unmatched adapter with the same display name. **Trace:** [Audit RT-09](./localmotive-comprehensive-audit.md#rt-09).
- [x] **V06-RT-09.I3** — When a trustworthy mapping is unavailable or ambiguous, keep observations unassigned or explicitly unknown instead of attaching device-specific usage/capacity evidence to an arbitrary LUID. **Trace:** [Audit RT-09](./localmotive-comprehensive-audit.md#rt-09).
- [x] **V06-RT-09.I4** — Use an explicit runtime-device-to-DXGI identity mapping for managed health selection so a requested adapter ID selects the intended physical device; preserve refusal on unresolved ambiguity and keep existing DXGI budget evidence separate. **Trace:** [Audit RT-09](./localmotive-comprehensive-audit.md#rt-09).

**Verification**

- [x] **V06-RT-09.V1** — Provide two identically named adapters with reversed DXGI and NVIDIA enumeration orders and distinct usage values; assert correct stable mapping or explicit unknown/unassigned evidence. **Trace:** [Audit RT-09](./localmotive-comprehensive-audit.md#rt-09).
- [x] **V06-RT-09.V2** — Exercise missing, duplicate, and conflicting physical identifiers and confirm that matching does not fall back to arbitrary enumeration order or name-only assignment. **Trace:** [Audit RT-09](./localmotive-comprehensive-audit.md#rt-09).
- [x] **V06-RT-09.V3** — Test same-name managed health selection against explicit runtime-device mappings, then validate the mapping on representative supported Windows hardware with multiple identical GPUs when available. **Trace:** [Audit RT-09](./localmotive-comprehensive-audit.md#rt-09).

**Complete when:** Per-adapter NVIDIA observations are associated only through verified physical identity, with ambiguity visible rather than silently resolved. Managed health selects the requested physical adapter or returns an explicit mapping failure without selecting another same-name GPU.

**Scope / decision note:** The audit confirmed ambiguous source matching but did not reproduce reversed ordering on hardware. Existing DXGI budget fields remain separate, so the finding does not imply every memory estimate is wrong; unresolved hardware mapping must remain unknown rather than guessed.

## Catalog and download tasks

### V06-DC-01

**Keep catalog loading available during refresh cooldowns**

**Status:** Implemented; unit-verified (commit `e6c7f59`); packaged restart acceptance open (V06-G-04/G-05) · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit DC-01](./localmotive-comprehensive-audit.md#dc-01)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/catalog.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx)

**Implementation**

- [x] **V06-DC-01.I1** — Separate loading an existing catalog from requesting a network refresh. Resolve a supported, signature-verified local snapshot or the bundled fallback before applying the persisted 1,560-minute refresh throttle. **Trace:** [Audit DC-01](./localmotive-comprehensive-audit.md#dc-01).
- [x] **V06-DC-01.I2** — Populate the backend's authoritative curated catalog state during local loading, including a fresh app instance, and return origin, last-success time, and remaining cooldown without turning a valid local read into an error. **Trace:** [Audit DC-01](./localmotive-comprehensive-audit.md#dc-01).
- [x] **V06-DC-01.I3** — Update the frontend load sequence to display available rows independently of network-refresh success, retain them when refresh is throttled, and show the cooldown alongside the refresh control. **Trace:** [Audit DC-01](./localmotive-comprehensive-audit.md#dc-01).
- [x] **V06-DC-01.I4** — Preserve the in-flight refresh guard and network throttling, while keeping signed curated authorization distinct from mutable SQLite browsing data. **Trace:** [Audit DC-01](./localmotive-comprehensive-audit.md#dc-01).

**Verification**

- [x] **V06-DC-01.V1** — Add a failing command/UI regression that starts with a signed cache, populated mirror, fresh persisted stamp, and new AppState; require populated rows and no network request. **Trace:** [Audit DC-01](./localmotive-comprehensive-audit.md#dc-01).
- [x] **V06-DC-01.V2** — Exercise missing and corrupt caches, bundled fallback, unavailable network, repeated Refresh, and clock rollback; verify that the selected supported snapshot initializes backend authorization. **Trace:** [Audit DC-01](./localmotive-comprehensive-audit.md#dc-01).
- [ ] **V06-DC-01.V3** — Run the packaged Windows sequence: successful refresh, close, restart within 26 hours, open HF Catalog, and browse/filter a cached entry; record the displayed cooldown and request behavior. **Trace:** [Audit DC-01](./localmotive-comprehensive-audit.md#dc-01).

**Complete when:** A new app instance displays a usable verified catalog throughout the refresh cooldown and while offline. Explicit refresh remains throttled without clearing rows or leaving curated authorization unnecessarily at the bundled version.

**Scope / decision note:** The audit confirmed the failing control flow but did not execute the packaged restart scenario. SQLite rows must not silently become signed download authority as part of restoring local availability.

### V06-DC-02

**Support complete downloads when servers ignore byte ranges**

**Status:** Implemented; unit-verified (`998a4d6`, `c57fa64`, `d30c22e`) · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit DC-02](./localmotive-comprehensive-audit.md#dc-02)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/download.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/download.rs)

**Implementation**

- [x] **V06-DC-02.I1** — Introduce an explicit sequential whole-response transfer path when the probe establishes that Range is ignored, using the complete expected object size rather than the 8 MiB ranged-request span. **Trace:** [Audit DC-02](./localmotive-comprehensive-audit.md#dc-02).
- [x] **V06-DC-02.I2** — Retain a bounded streaming buffer and read or idle deadline for the sequential path; validate the full expected length and mandatory final SHA-256 before publication. **Trace:** [Audit DC-02](./localmotive-comprehensive-audit.md#dc-02).
- [x] **V06-DC-02.I3** — Restart non-range transfers from zero after interruption or retry, and ensure misleading Accept-Ranges headers cannot cause unsafe reuse of a partially downloaded object. **Trace:** [Audit DC-02](./localmotive-comprehensive-audit.md#dc-02).
- [x] **V06-DC-02.I4** — Keep exact Content-Range, remote-validator, response-length, and overrun checks on the existing ranged path; do not weaken parallel integrity checks to accommodate HTTP 200. **Trace:** [Audit DC-02](./localmotive-comprehensive-audit.md#dc-02).

**Verification**

- [x] **V06-DC-02.V1** — Extend the existing 16 KiB no-range regression with 8 MiB+1 and a realistic larger fixture; test HTTP 200 with both Content-Length and chunked response bodies. **Trace:** [Audit DC-02](./localmotive-comprehensive-audit.md#dc-02).
- [x] **V06-DC-02.V2** — Test cancellation, stalled reads, truncated bodies, retry from zero, misleading Accept-Ranges, excess bytes, and incorrect digests against a local server. **Trace:** [Audit DC-02](./localmotive-comprehensive-audit.md#dc-02).
- [x] **V06-DC-02.V3** — Re-run parallel range, resume-identity, and checksum regressions, and verify a runtime-artifact caller remains compatible because runtime and model transfers share this downloader. **Trace:** [Audit DC-02](./localmotive-comprehensive-audit.md#dc-02).

**Complete when:** A correct range-ignoring response larger than 8 MiB completes with the expected bytes and digest. Interrupted no-range downloads restart safely, while malformed ranged responses and checksum mismatches remain rejected.

**Scope / decision note:** The audit established the size-threshold defect from source. It did not establish that Hugging Face currently ignores ranges; the 1,408 affected-size catalog entries measure potential applicability, not live failures.

### V06-DC-03

**Refresh healthy catalog mirrors and recover damaged databases explicitly**

**Status:** Implemented; unit-verified (commit `e6c7f59`); packaged open-handle acceptance open (V06-G-04/G-05) · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit DC-03](./localmotive-comprehensive-audit.md#dc-03)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/catalog_db.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog_db.rs)

**Implementation**

- [x] **V06-DC-03.I1** — Replace the healthy-migration rebuild branch with a transactional call to mirror_verified_catalog on the valid connection, preserving existing user records during ordinary refresh. **Trace:** [Audit DC-03](./localmotive-comprehensive-audit.md#dc-03).
- [x] **V06-DC-03.I2** — Route confirmed migration or corruption failures into controlled recovery, closing relevant connections before quarantine or replacement instead of depending on removal of an open SQLite file. **Trace:** [Audit DC-03](./localmotive-comprehensive-audit.md#dc-03).
- [x] **V06-DC-03.I3** — Preserve or export readable user overrides before rebuilding; define a non-destructive downgrade policy for unknown-newer schemas and report any irrecoverable loss rather than deleting silently. **Trace:** [Audit DC-03](./localmotive-comprehensive-audit.md#dc-03).
- [x] **V06-DC-03.I4** — Propagate database-open, migration, mirror, and publication failures into an explicit persistence notice while continuing to serve an available verified in-memory catalog. **Trace:** [Audit DC-03](./localmotive-comprehensive-audit.md#dc-03).

**Verification**

- [x] **V06-DC-03.V1** — Create a real temporary database with curated and user records; refresh repeatedly and assert both provenance and user contents survive. **Trace:** [Audit DC-03](./localmotive-comprehensive-audit.md#dc-03).
- [x] **V06-DC-03.V2** — Exercise corrupt databases, unknown-newer schemas, unavailable database paths, held locks, and injected write failures; verify recovery policy and preservation of the prior readable state. **Trace:** [Audit DC-03](./localmotive-comprehensive-audit.md#dc-03).
- [ ] **V06-DC-03.V3** — Run the open-handle and recovery cases in the packaged Windows application, recording actual NTFS/SQLite sharing outcomes and confirming that every failed persistence action is visible. **Trace:** [Audit DC-03](./localmotive-comprehensive-audit.md#dc-03).

**Complete when:** Normal refresh updates curated rows without rebuilding the healthy database or losing user overrides. Corruption and unsupported-schema handling follow a documented recoverable path, and persistence errors are observable without making valid signed browsing data unavailable.

**Scope / decision note:** The inverted branch is confirmed. Windows override loss was not demonstrated: deletion may fail because the original connection remains open. Closure must rely on measured target-platform behavior, not assume the audit proved Windows data loss.

### V06-DC-04

**Make local overrides coherent across browsing and download authorization**

**Status:** Implemented; unit-verified (commit `6be56e5`); override flow accepted in the packaged build under V06-G-04/G-05 · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit DC-04](./localmotive-comprehensive-audit.md#dc-04)  
**Prerequisites:** [V06-DC-05](#v06-dc-05), [V06-DC-06](#v06-dc-06)
**Source touchpoints:** [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/catalog_db.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog_db.rs), [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx)

**Implementation**

- [x] **V06-DC-04.I1** — Define the supported local-override trust contract explicitly: signed curated entries remain authorized by their signed snapshot, while supported override downloads require a separately validated, explicitly user-approved record and exact digest. **Trace:** [Audit DC-04](./localmotive-comprehensive-audit.md#dc-04).
- [x] **V06-DC-04.I2** — Implement the chosen override download route without promoting arbitrary mutable SQLite rows into curator-signed authority; ensure save, reload, and removal update the appropriate authorization source. **Trace:** [Audit DC-04](./localmotive-comprehensive-audit.md#dc-04).
- [x] **V06-DC-04.I3** — Use one merged browse collection for initial rows, filtering, sorting, and facets so the filter effect no longer replaces local entries with only snapshot.catalog.models. **Trace:** [Audit DC-04](./localmotive-comprehensive-audit.md#dc-04).
- [x] **V06-DC-04.I4** — Keep provenance visible throughout the workflow and document the product decision if the incomplete add/edit/download functionality is deferred instead of enabled end to end. **Trace:** [Audit DC-04](./localmotive-comprehensive-audit.md#dc-04).

**Verification**

- [x] **V06-DC-04.V1** — Save a unique override, reload the app, change every relevant filter/sort, and verify rows and facet values continue to refer to the same merged collection. **Trace:** [Audit DC-04](./localmotive-comprehensive-audit.md#dc-04).
- [ ] **V06-DC-04.V2** — Against a controlled server, exercise an approved override with correct and incorrect SHA-256, then remove it and verify subsequent download authorization fails. **Trace:** [Audit DC-04](./localmotive-comprehensive-audit.md#dc-04).
- [x] **V06-DC-04.V3** — Attempt downloads from unknown or directly modified database rows and verify they never acquire signed curated status; retain tests for ordinary curated downloads. **Trace:** [Audit DC-04](./localmotive-comprehensive-audit.md#dc-04).

**Complete when:** Supported local overrides remain consistently visible and actionable across reload and filtering, with correct provenance. Downloads obey an explicit curated-versus-user authority split, or deferred override functionality is clearly unavailable rather than misleadingly half-enabled.

**Scope / decision note:** The audit found registered commands and incomplete integration, not a proven complete add/edit UI workflow. DC-05 and DC-06 established safe ownership and atomic records before override downloads rely on that store. **Product decision (recorded here per DC-04.I4):** the add/edit override *form* remains deferred — the UI still exposes no way to create an override, so nothing is misleadingly half-enabled — while `save_user_catalog_override` / `remove_user_catalog_override` and the download route are now complete and regression-tested end to end at the command boundary. Ships with the panel showing USER ADDED provenance for existing rows and the per-file USER FILE tag; enabling a creation form is future work, not a 0.6.0 blocker.

### V06-DC-05

**Prevent override collisions from changing curated ownership or provenance**

**Status:** Implemented; unit-verified (commit `6be56e5`); packaged mixed-origin acceptance covered under V06-G-04/G-05 · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit DC-05](./localmotive-comprehensive-audit.md#dc-05)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/catalog_db.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog_db.rs)

**Implementation**

- [x] **V06-DC-05.I1** — Reserve curator-owned model identifiers and choose either a separate user namespace or explicit rejection of mixed-origin ID collisions before any UPSERT can mutate ownership. **Trace:** [Audit DC-05](./localmotive-comprehensive-audit.md#dc-05).
- [x] **V06-DC-05.I2** — Define and enforce a deterministic case-insensitive filename collision policy that preserves existing files and requires explicit resolution rather than silently moving a catalog_file row to another model. **Trace:** [Audit DC-05](./localmotive-comprehensive-audit.md#dc-05).
- [x] **V06-DC-05.I3** — Carry provenance at the file/entity level used for display and authorization; update database reads so a user file cannot inherit a curator label merely because its model row was refreshed. **Trace:** [Audit DC-05](./localmotive-comprehensive-audit.md#dc-05).
- [x] **V06-DC-05.I4** — Change curated mirror refresh to preserve origin boundaries, and enable or strengthen relational constraints where they enforce the chosen ownership rules. **Trace:** [Audit DC-05](./localmotive-comprehensive-audit.md#dc-05).

**Verification**

- [x] **V06-DC-05.V1** — Turn the executed SQL reproductions into application-level regressions for same ID/different repo and different IDs/same case-insensitive filename; require the original curated records to remain unchanged. **Trace:** [Audit DC-05](./localmotive-comprehensive-audit.md#dc-05).
- [x] **V06-DC-05.V2** — Exercise refresh, edit, and removal after each collision attempt; verify no empty curated model or stale user file becomes relabeled as curator-sourced. **Trace:** [Audit DC-05](./localmotive-comprehensive-audit.md#dc-05).
- [x] **V06-DC-05.V3** — Test disjoint user and curator entries alongside the rejection cases so normal additions and refresh preservation remain supported. **Trace:** [Audit DC-05](./localmotive-comprehensive-audit.md#dc-05).

**Complete when:** No override write or mirror refresh silently replaces curated model ownership or transfers an existing filename across origins. Every returned file retains correct provenance after save, refresh, edit, and removal, including collision attempts.

**Scope / decision note:** The audit executed the current SQL and confirmed these data/provenance defects. It found parameterized queries and no bypass of signed download authorization; this task fixes ownership integrity, not a claimed SQL injection or arbitrary-code-execution vulnerability.

### V06-DC-06

**Make override replacement atomic and validate complete payloads**

**Status:** Implemented; unit-verified (commit `6be56e5`); injected-fault variants recorded in the ledger · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit DC-06](./localmotive-comprehensive-audit.md#dc-06)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/catalog_db.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog_db.rs), [src-tauri/src/catalog.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs)

**Implementation**

- [x] **V06-DC-06.I1** — Wrap replacement of one user's model and complete file set in a transaction, removing files omitted by the new version while preserving the previous version if any step fails. **Trace:** [Audit DC-06](./localmotive-comprehensive-audit.md#dc-06).
- [x] **V06-DC-06.I2** — Make override removal transactional so deleting file rows and the model row either succeeds as one operation or leaves the previous state intact. **Trace:** [Audit DC-06](./localmotive-comprehensive-audit.md#dc-06).
- [x] **V06-DC-06.I3** — Apply shared full validation to user writes and parsed records: bound serialized bytes, tags and nested arrays, text/date/quant/revision lengths, file counts, and case-insensitive duplicate targets. **Trace:** [Audit DC-06](./localmotive-comprehensive-audit.md#dc-06).
- [x] **V06-DC-06.I4** — Enforce the 200-user limit with origin-aware accounting under concurrent writes, and replace unchecked u64-to-i64 and reverse casts with validated, checked conversions. **Trace:** [Audit DC-06](./localmotive-comprehensive-audit.md#dc-06).

**Verification**

- [x] **V06-DC-06.V1** — Add regressions for changing two files to one and a.gguf to b.gguf, then inject failure on the second insert and between removal statements; assert exact replacement or full rollback. **Trace:** [Audit DC-06](./localmotive-comprehensive-audit.md#dc-06).
- [x] **V06-DC-06.V2** — Exercise concurrent saves near the user cap and attempted conversion of a curated ID, coordinating the ownership rules from DC-05. **Trace:** [Audit DC-06](./localmotive-comprehensive-audit.md#dc-06).
- [x] **V06-DC-06.V3** — Test excessive tags, long revisions/quants/dates, duplicate filenames, serialized payload limits, and integer values at i64::MAX and beyond; require actionable rejections before mutation. **Trace:** [Audit DC-06](./localmotive-comprehensive-audit.md#dc-06).

**Complete when:** Successful edits return exactly the requested file set, while failed saves/removals preserve the previous complete record. All supported override values round-trip faithfully, and count/payload limits cannot be bypassed through existing IDs or nested fields.

**Scope / decision note:** Stale-file behavior was reproduced through SQLite; atomicity and missing bounds were established by source review. The shipped catalog's numeric values were within range, so this is not a claim that valid shipped sizes already suffered integer corruption.

### V06-DC-07

**Retain the last supported catalog for every failed refresh candidate**

**Status:** Implemented; unit-verified (commit `e6c7f59`); packaged fallback acceptance open (V06-G-04/G-05) · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit DC-07](./localmotive-comprehensive-audit.md#dc-07)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/catalog.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs)

**Implementation**

- [x] **V06-DC-07.I1** — Route candidate body-read, streamed size-limit, UTF-8, signature-body, signature-validation, and catalog-parse failures through one consistent fallback path instead of early propagation that suppresses usable data. **Trace:** [Audit DC-07](./localmotive-comprehensive-audit.md#dc-07).
- [x] **V06-DC-07.I2** — Return the last supported signature-verified cache or bundled snapshot with explicit refresh-error status; preserve the distinction between a successful local load and a successful network refresh. **Trace:** [Audit DC-07](./localmotive-comprehensive-audit.md#dc-07).
- [x] **V06-DC-07.I3** — Keep unsupported future-schema and otherwise invalid candidates from replacing the last valid cache or becoming the authoritative curated download state. **Trace:** [Audit DC-07](./localmotive-comprehensive-audit.md#dc-07).
- [x] **V06-DC-07.I4** — Handle catalog database-open failure consistently with migration/read failure by selecting verified memory or bundled browsing data rather than depending solely on a frontend catch. **Trace:** [Audit DC-07](./localmotive-comprehensive-audit.md#dc-07).

**Verification**

- [x] **V06-DC-07.V1** — Seed a valid signed cache and test truncated streaming responses, missing Content-Length with limit+1 data, invalid UTF-8, malformed signature bodies, and network/send failures. **Trace:** [Audit DC-07](./localmotive-comprehensive-audit.md#dc-07).
- [x] **V06-DC-07.V2** — Serve a validly signed unsupported schema and confirm the previous supported catalog remains visible and unchanged on disk. **Trace:** [Audit DC-07](./localmotive-comprehensive-audit.md#dc-07).
- [x] **V06-DC-07.V3** — Repeat fallback cases without a usable cache and with a database-open failure; verify a bundled snapshot, clear refresh status, and no unsigned authorization. **Trace:** [Audit DC-07](./localmotive-comprehensive-audit.md#dc-07).

**Complete when:** Every rejected or interrupted network candidate leaves supported local catalog data available with an explicit status message. Invalid or future-schema candidates never overwrite the last-good cache or initialize curated authorization.

**Scope / decision note:** The audit found inconsistent error propagation, not a signature-verification bypass. This task must preserve fail-closed candidate acceptance while correcting availability; it does not require accepting unsupported schemas or unsigned SQLite data.

### V06-DC-08

**Bound GGUF allocations and parser work while keeping reads responsive**

**Status:** Implemented; unit-verified (`4e5531c`) · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit DC-08](./localmotive-comprehensive-audit.md#dc-08)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/gguf.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/gguf.rs), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/artifact.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/artifact.rs)

**Implementation**

- [x] **V06-DC-08.I1** — Introduce explicit limits for KV count, string/key length, aggregate retained bytes/elements, and total parser work across nested values; enforce budgets before allocation rather than relying only on the 256 MiB input limit. **Trace:** [Audit DC-08](./localmotive-comprehensive-audit.md#dc-08).
- [x] **V06-DC-08.I2** — Skip irrelevant fixed-size arrays using checked byte counts, stream-discard unneeded strings, and avoid allocating captured-value vectors when capture is false; retain clear truncation metadata for supported summaries. **Trace:** [Audit DC-08](./localmotive-comprehensive-audit.md#dc-08).
- [x] **V06-DC-08.I3** — Use buffered file reads and stack arrays for fixed-width primitives, validate remaining file bytes early, and avoid unnecessary cloning of retained metadata. **Trace:** [Audit DC-08](./localmotive-comprehensive-audit.md#dc-08).
- [x] **V06-DC-08.I4** — Move synchronous GGUF parsing into cancellable background work and apply a defined budget across artifact inspection so multiple shards cannot create unbounded repeated work. **Trace:** [Audit DC-08](./localmotive-comprehensive-audit.md#dc-08).

**Verification**

- [x] **V06-DC-08.V1** — Add deterministic resource regressions for short files declaring giant strings, many tiny KVs, wide nested arrays, noncaptured tokenizer arrays, oversized keys, and counts just beyond each limit. **Trace:** [Audit DC-08](./localmotive-comprehensive-audit.md#dc-08).
- [x] **V06-DC-08.V2** — Test files shortened during parsing and cancellation during large summaries or multi-shard inspection; assert prompt errors and preserved UI responsiveness. **Trace:** [Audit DC-08](./localmotive-comprehensive-audit.md#dc-08).
- [x] **V06-DC-08.V3** — Fuzz malformed/truncated headers with explicit allocation/work budgets, and measure representative valid headers to ensure useful metadata and header-only behavior remain intact. **Trace:** [Audit DC-08](./localmotive-comprehensive-audit.md#dc-08).

**Complete when:** Malformed or excessive headers fail within documented allocation/work limits before disproportionate memory use. Valid summaries remain useful and explicitly indicate truncation, while long parsing work is cancellable without blocking the interface.

**Scope / decision note:** The audit confirmed missing resource bounds but did not run an OOM exploit or performance benchmark. Opening/scanning an attacker-provided local GGUF is the demonstrated prerequisite; no automatic remote exploitation path was established.

### V06-DC-09

**Derive canonical quantization labels from verified model evidence**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** Unassigned  
**Audit trace:** [Audit DC-09](./localmotive-comprehensive-audit.md#dc-09)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [scripts/build_catalog.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/build_catalog.mjs), [scripts/validate_catalog.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/validate_catalog.mjs), [catalog/catalog.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/catalog/catalog.json), [catalog/providers.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/catalog/providers.json), [src-tauri/src/catalog.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs)

**Implementation**

- [x] **V06-DC-09.I1** — Replace the arbitrary final-filename-suffix extraction with parsing of actual recognized quant tokens in the full basename, preserving provenance or variant suffixes separately and returning unknown when evidence is ambiguous. **Trace:** [Audit DC-09](./localmotive-comprehensive-audit.md#dc-09).
- [x] **V06-DC-09.I2** — Prefer verified structured upstream quant metadata when available, normalize canonical casing, and make facet deduplication agree with filter comparison semantics. **Trace:** [Audit DC-09](./localmotive-comprehensive-audit.md#dc-09).
- [x] **V06-DC-09.I3** — Extend curation validation and shared fixtures to reject unsupported provenance/date/instruction labels masquerading as quants; rebuild corrected candidate metadata through the existing signature-validation process. **Trace:** [Audit DC-09](./localmotive-comprehensive-audit.md#dc-09).
- [x] **V06-DC-09.I4** — Review MTP-related candidates using upstream or header evidence to distinguish integrated-MTP main models from companion-only files before changing exclusion rules. **Trace:** [Audit DC-09](./localmotive-comprehensive-audit.md#dc-09).

**Verification**

- [x] **V06-DC-09.V1** — Add fixtures covering IQ2_S-MTP, Q4_K_M-imatrix, dates, instruction-tuning suffixes, lowercase labels, combined base/draft quants, unknown formats, and quant-looking model names. **Trace:** [Audit DC-09](./localmotive-comprehensive-audit.md#dc-09).
- [x] **V06-DC-09.V2** — Use the audit's 46 suffix-label rows as a correction checklist, preserving unknown where underlying quant evidence is insufficient rather than guessing. **Trace:** [Audit DC-09](./localmotive-comprehensive-audit.md#dc-09).
- [x] **V06-DC-09.V3** — Verify one semantic facet per normalized quant and confirm selecting a quant finds corrected builds; run catalog validation on the resulting signed candidate. **Trace:** [Audit DC-09](./localmotive-comprehensive-audit.md#dc-09).

**Complete when:** Catalog Build and Quant controls show evidence-backed canonical quants or explicit unknown values instead of arbitrary trailing labels. The audited suffix cases are resolved with recorded evidence, and filters/facets agree on normalization.

**Scope / decision note:** The 46 affected labels were confirmed in the shipped artifact. Filename text alone does not prove the six MTP-suffixed entries are companions; no speculative removal or invented quantization is authorized by this finding.

### V06-DC-10

**Pin catalog downloads to immutable upstream revisions**

**Status:** Implemented + unit-verified · **Priority:** Low · **Owner:** Unassigned  
**Audit trace:** [Audit DC-10](./localmotive-comprehensive-audit.md#dc-10)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [scripts/build_catalog.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/build_catalog.mjs), [src-tauri/src/catalog.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs), [catalog/catalog.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/catalog/catalog.json)

**Implementation**

- [x] **V06-DC-10.I1** — Capture an immutable upstream commit identity for each offered file during catalog construction and serialize it as the file revision alongside the exact verified SHA-256 and size. **Trace:** [Audit DC-10](./localmotive-comprehensive-audit.md#dc-10).
- [x] **V06-DC-10.I2** — Preserve immutable revisions through catalog parsing, cached/bundled representation, and download authorization so resolution does not silently return to mutable main. **Trace:** [Audit DC-10](./localmotive-comprehensive-audit.md#dc-10).
- [x] **V06-DC-10.I3** — Return a clear recoverable error when an approved historical object is unavailable, retaining strict length/digest checks instead of substituting current upstream bytes. **Trace:** [Audit DC-10](./localmotive-comprehensive-audit.md#dc-10).
- [x] **V06-DC-10.I4** — Document whether rollback of a previously signed manifest belongs in the freshness threat model; if it does, define and implement signed sequence/expiry acceptance and recovery behavior. **Trace:** [Audit DC-10](./localmotive-comprehensive-audit.md#dc-10).

**Verification**

- [x] **V06-DC-10.V1** — Add immutable revision serialization and authorization round-trip fixtures, including cache/bundle loading and an entry whose filename remains unchanged across revisions. **Trace:** [Audit DC-10](./localmotive-comprehensive-audit.md#dc-10).
- [x] **V06-DC-10.V2** — Use controlled upstream responses to replace main while keeping the approved revision available; require the pinned request to resolve the approved bytes and reject unintended substitution. **Trace:** [Audit DC-10](./localmotive-comprehensive-audit.md#dc-10).
- [x] **V06-DC-10.V3** — Test unavailable historical objects and enforce the documented old-signed-manifest replay policy, including any required expiry or rollback recovery scenario. **Trace:** [Audit DC-10](./localmotive-comprehensive-audit.md#dc-10).

**Complete when:** Newly built catalog file records name immutable upstream revisions and retain mandatory size/SHA validation throughout resolution. Historical-object failures and signed-manifest rollback behavior are explicitly defined and covered by the selected policy.

**Scope / decision note:** All 1,417 audited file entries defaulted to main, but no actual upstream replacement was demonstrated. SHA checks already prevent accepting changed bytes. The separate frozen-date builder bug remains QD-01 and must not be duplicated here.

### V06-DC-11

**Publish verified downloads without overwriting concurrent files**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** Unassigned  
**Audit trace:** [Audit DC-11](./localmotive-comprehensive-audit.md#dc-11)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/download.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/download.rs), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs)

**Implementation**

- [x] **V06-DC-11.I1** — Add a per-target cross-process reservation or lock so separate app instances cannot share and mutate the same partial transfer merely because the in-process registry is clear. **Trace:** [Audit DC-11](./localmotive-comprehensive-audit.md#dc-11).
- [x] **V06-DC-11.I2** — Use a unique private partial filename and preserve stable file identity from writing through hash verification and final publication where platform APIs permit. **Trace:** [Audit DC-11](./localmotive-comprehensive-audit.md#dc-11).
- [x] **V06-DC-11.I3** — Publish with no-replace semantics, checking that the entry being committed is the verified object; an ordinary target appearing during transfer must produce a conflict instead of being overwritten. **Trace:** [Audit DC-11](./localmotive-comprehensive-audit.md#dc-11).
- [x] **V06-DC-11.I4** — Preserve both the verified artifact and the conflicting existing file on collision, report actionable recovery information, and retain directory-capability, reparse-point, and hard-link protections. **Trace:** [Audit DC-11](./localmotive-comprehensive-audit.md#dc-11).

**Verification**

- [x] **V06-DC-11.V1** — Run two app instances targeting the same filename and introduce an unrelated final file during transfer; assert neither pre-existing file is modified. **Trace:** [Audit DC-11](./localmotive-comprehensive-audit.md#dc-11).
- [x] **V06-DC-11.V2** — Inject partial-entry replacement immediately after hashing and directory/root renaming during transfer; require safe failure or verified stable-object publication. **Trace:** [Audit DC-11](./localmotive-comprehensive-audit.md#dc-11).
- [x] **V06-DC-11.V3** — Exercise the cases on NTFS in the packaged application, record actual handle/rename behavior, and re-run existing directory-capability and link-safety regressions. **Trace:** [Audit DC-11](./localmotive-comprehensive-audit.md#dc-11).

**Complete when:** Completion cannot overwrite a file created by another process during the transfer. Only the exact verified partial object is promoted, with recoverable conflict behavior and correct cross-process ownership.

**Scope / decision note:** The source race windows were confirmed, but exploitation and Windows behavior were not executed. No network-only traversal, outside-directory write, or privilege escalation was demonstrated; prioritize ordinary multi-instance collision safety and validate platform assumptions.

### V06-DC-12

**Order durable checkpoints and make checksum verification cancellable**

**Status:** Implemented; unit-verified (`0720d41`) · **Priority:** Low · **Owner:** sato942  
**Audit trace:** [Audit DC-12](./localmotive-comprehensive-audit.md#dc-12)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/download.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/download.rs), [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx)

**Implementation**

- [x] **V06-DC-12.I1** — Define the promised crash and power-loss recovery guarantees, then order partial-data durability before publishing resume checkpoints that claim those bytes are complete. **Trace:** [Audit DC-12](./localmotive-comprehensive-audit.md#dc-12).
- [x] **V06-DC-12.I2** — Choose documented byte/time checkpoint intervals that maintain the guarantee without syncing tiny sidecars unnecessarily; preserve the previous valid checkpoint when data or metadata persistence fails. **Trace:** [Audit DC-12](./localmotive-comprehensive-audit.md#dc-12).
- [x] **V06-DC-12.I3** — Add cancellation checks and progress reporting to both existing-file and completed-part SHA-256 verification, preserving recoverable state when Keep & stop is requested during hashing. **Trace:** [Audit DC-12](./localmotive-comprehensive-audit.md#dc-12).
- [ ] **V06-DC-12.I4** — Align frontend verification controls with backend cancellation semantics and measure the shared write-mutex/seek path before considering positional-write optimization. **Trace:** [Audit DC-12](./localmotive-comprehensive-audit.md#dc-12).

**Verification**

- [x] **V06-DC-12.V1** — Inject data-write, synchronization, checkpoint-publication, and recovery failures; assert that resumed progress never knowingly claims bytes outside the selected durability guarantee. **Trace:** [Audit DC-12](./localmotive-comprehensive-audit.md#dc-12).
- [x] **V06-DC-12.V2** — Cancel large existing-file and final-part verification, require prompt control return, and verify that retry safely resumes or re-verifies without incorrectly reporting completion. **Trace:** [Audit DC-12](./localmotive-comprehensive-audit.md#dc-12).
- [ ] **V06-DC-12.V3** — Distinguish ordinary process-crash testing from target-environment OS-crash/power-loss validation; record throughput for 1/4/8 connections on available HDD, SATA SSD, and NVMe targets before optimizing. **Trace:** [Audit DC-12](./localmotive-comprehensive-audit.md#dc-12).

**Complete when:** Checkpoint ordering and recovery behavior match the documented durability guarantee, with no silent successful checkpoint after failed required data persistence. Verification is visibly progressive and cancellable, and stopping never promotes an unverified object or unnecessarily loses recoverable work.

**Scope / decision note:** The audit confirmed sync/cancellation placement but did not demonstrate power-loss corruption. Final SHA already prevents acceptance of wrong bytes. Storage performance improvements are conditional on measurements, not an assumption that serialized writes are currently the bottleneck.

## Measurement and tuning tasks

### V06-MT-01

**Correct warm-cache token accounting against the b10816 timing contract**

**Status:** Implemented; unit-verified (commit `9d4c36c`); packaged b10816 acceptance open (V06-G-04/G-05) · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit MT-01](./localmotive-comprehensive-audit.md#mt-01)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/measurement.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/evidence.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/evidence.rs)

**Implementation**

- [x] **V06-MT-01.I1** — Define whether the warm workload measures a resident model with full prefill or actual KV-prompt reuse; represent that choice explicitly in the immutable workload and persisted protocol. **Trace:** [Audit MT-01](./localmotive-comprehensive-audit.md#mt-01).
- [x] **V06-MT-01.I2** — Honor approved llama.cpp b10816 semantics: timings.prompt_n is newly processed prompt tokens, cache_n is cached prompt tokens, and total context includes prompt_n + cache_n + predicted_n. **Trace:** [Audit MT-01](./localmotive-comprehensive-audit.md#mt-01).
- [x] **V06-MT-01.I3** — For prompt reuse, persist requested, processed, and cached counts separately and validate the relevant total; for full-prefill measurement, request cache_prompt=false while retaining process warmup. **Trace:** [Audit MT-01](./localmotive-comprehensive-audit.md#mt-01).
- [x] **V06-MT-01.I4** — Keep exact generation-length validation and truthful prefill metrics, and version or migrate persisted contracts affected by the new count semantics rather than silently removing checks. **Trace:** [Audit MT-01](./localmotive-comprehensive-audit.md#mt-01).

**Verification**

- [x] **V06-MT-01.V1** — Add a protocol regression returning prompt_n=512/cache_n=0 followed by prompt_n=1/cache_n=511, both generating 256 tokens; assert valid reuse or deliberate cache-off requests. **Trace:** [Audit MT-01](./localmotive-comprehensive-audit.md#mt-01).
- [x] **V06-MT-01.V2** — Exercise no-warmup, partial-cache, inconsistent-total, and wrong-generation-count cases so cached successes are accepted without accepting genuinely incorrect workloads. **Trace:** [Audit MT-01](./localmotive-comprehensive-audit.md#mt-01).
- [ ] **V06-MT-01.V3** — Run one warmup and five default trials against packaged approved b10816 on Windows; record response counts, request cache policy, and the persisted manifest. **Trace:** [Audit MT-01](./localmotive-comprehensive-audit.md#mt-01).

**Complete when:** The default benchmark completes valid cached trials without prompt-length false failures, and exported raw counts explain precisely what was evaluated. The packaged acceptance evidence identifies the tested runtime and protocol; unsupported historical runtime contracts have an explicit outcome.

**Scope / decision note:** The audit confirmed b10816 source semantics but did not execute packaged Windows inference. Choosing warm-model versus warm-prompt measurement is a product/protocol decision, not an interchangeable implementation detail.

### V06-MT-02

**Remove raw errors and private nested text from public share exports**

**Status:** Implemented; unit-verified (commit `9d4c36c`) · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit MT-02](./localmotive-comprehensive-audit.md#mt-02)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/sharing.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/sharing.rs), [src-tauri/src/measurement.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs), [src-tauri/src/evidence.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/evidence.rs)

**Implementation**

- [x] **V06-MT-02.I1** — Replace the cloned internal benchmark summary in the public schema with a dedicated public summary containing numeric statistics, counts, and bounded failure categories instead of summary.failures raw strings. **Trace:** [Audit MT-02](./localmotive-comprehensive-audit.md#mt-02).
- [x] **V06-MT-02.I2** — Construct public hardware, effective-context, and process-memory evidence through an allowlist; define whether source.detail and notes are omitted or transformed into approved public source identifiers. **Trace:** [Audit MT-02](./localmotive-comprehensive-audit.md#mt-02).
- [x] **V06-MT-02.I3** — Make the privacy-review omissions describe the actual serialized policy, and validate public exports at both normal construction and direct persistence boundaries. **Trace:** [Audit MT-02](./localmotive-comprehensive-audit.md#mt-02).
- [x] **V06-MT-02.I4** — Preserve complete diagnostics in authorized local evidence while retaining explicit user confirmation and create-new share-file publication. **Trace:** [Audit MT-02](./localmotive-comprehensive-audit.md#mt-02).

**Verification**

- [x] **V06-MT-02.V1** — Extend the privacy regression with one successful and one failed observation and a correctly recomputed Some(summary), using distinct path, account-name, and secret canaries in the raw failure. **Trace:** [Audit MT-02](./localmotive-comprehensive-audit.md#mt-02).
- [x] **V06-MT-02.V2** — Place separate canaries in nested evidence notes/details and inspect the entire serialized public file, not only observation-level fields. **Trace:** [Audit MT-02](./localmotive-comprehensive-audit.md#mt-02).
- [x] **V06-MT-02.V3** — Verify useful counts and numeric statistics survive redaction and that existing targets are not overwritten without changing the confirmation requirement. **Trace:** [Audit MT-02](./localmotive-comprehensive-audit.md#mt-02).

**Complete when:** Mixed success/failure exports contain no raw-error or nested-private-text canaries, and the omission list accurately matches their contents. Local diagnostic preservation and summary integrity remain intact while the public representation uses only approved fields.

**Scope / decision note:** The demonstrated leak is summary.failures. Nested free text is an additional policy surface; the audit did not establish that ordinary hardware notes always contain secrets. Export remains local until the user shares it.

### V06-MT-03

**Bound advisor calls and terminate repeated no-op tuning proposals**

**Status:** Implemented; unit-verified (commit `1eaa476`); no packaged-specific evidence required beyond the loop tests · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit MT-03](./localmotive-comprehensive-audit.md#mt-03)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/tune.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/tune.rs)

**Implementation**

- [x] **V06-MT-03.I1** — Introduce independent total advisor-call, elapsed-time, and measured-trial budgets so successful parsing and skipped measurements cannot leave a session unbounded. **Trace:** [Audit MT-03](./localmotive-comprehensive-audit.md#mt-03).
- [x] **V06-MT-03.I2** — Canonicalize proposals after coercion and companion resolution, comparing effective configurations or normalized changes instead of raw JSON maps. **Trace:** [Audit MT-03](./localmotive-comprehensive-audit.md#mt-03).
- [x] **V06-MT-03.I3** — Count no-op and duplicate proposals as bounded rejections, preserve their reasons in the session history, and return a specific terminal stopped reason when limits are reached. **Trace:** [Audit MT-03](./localmotive-comprehensive-audit.md#mt-03).
- [x] **V06-MT-03.I4** — Decide whether the advertised three-fields-per-trial rule is a hard policy; if retained, enforce it in Rust and align advisor instructions with backend limits. **Trace:** [Audit MT-03](./localmotive-comprehensive-audit.md#mt-03).

**Verification**

- [x] **V06-MT-03.V1** — Use a deterministic advisor repeatedly proposing baseline threads=-1 and a benchmark succeeding only for baseline; assert termination within the configured advisor-call limit. **Trace:** [Audit MT-03](./localmotive-comprehensive-audit.md#mt-03).
- [x] **V06-MT-03.V2** — Repeat with numeric-string coercion, draftModel echoes, alternating no-op maps, and fields that leave emitted arguments unchanged. **Trace:** [Audit MT-03](./localmotive-comprehensive-audit.md#mt-03).
- [x] **V06-MT-03.V3** — Test exact call/time/trial budget boundaries and verify cancellation during a no-op sequence prevents another cloud request when combined with the lifecycle cancellation fix. **Trace:** [Audit MT-03](./localmotive-comprehensive-audit.md#mt-03).

**Complete when:** Every proposal-loop path consumes or respects a finite budget, including valid no-ops and retries, and no session remains active solely because trials were not recorded. The report exposes the exhausted budget or rejection reason without claiming an unmeasured configuration improved performance.

**Scope / decision note:** The audit established an unbounded source path, not incurred account charges. Paid requests need not be made to reproduce it: the existing Advisor and Bench interfaces support deterministic regression tests.

### V06-MT-04

**Propagate tuning cancellation through advisor, request, and process lifecycles**

**Status:** Implemented; unit-verified (commit `1eaa476`); packaged Windows cancellation scenarios open in G-04/G-05 · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit MT-04](./localmotive-comprehensive-audit.md#mt-04)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/core.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs), [src-tauri/src/tune.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/tune.rs)

**Implementation**

- [x] **V06-MT-04.I1** — Make cancellation a terminal orchestrator outcome with checks before and after advisor calls and before every attempt; stop proposing instead of recording cancellation as an ordinary candidate failure. **Trace:** [Audit MT-04](./localmotive-comprehensive-audit.md#mt-04).
- [x] **V06-MT-04.I2** — Pass the cancellation signal into preparation, cancellable health startup, and completion requests, replacing the live tuner's uncancellable legacy benchmark path. **Trace:** [Audit MT-04](./localmotive-comprehensive-audit.md#mt-04).
- [x] **V06-MT-04.I3** — Apply a documented overall deadline and bounded stop latency rather than relying on a 600-second health wait or separate per-read timeouts. **Trace:** [Audit MT-04](./localmotive-comprehensive-audit.md#mt-04).
- [x] **V06-MT-04.I4** — Check process-tree termination results, surface cleanup failures, and release active tuning state only after lifecycle ownership has been resolved. **Trace:** [Audit MT-04](./localmotive-comprehensive-audit.md#mt-04).

**Verification**

- [x] **V06-MT-04.V1** — Cancel during preparation, health wait, response wait, between repetitions, between advisor calls, and while the advisor emits only no-ops; assert the final cancellation reason and no subsequent advisor call. **Trace:** [Audit MT-04](./localmotive-comprehensive-audit.md#mt-04).
- [ ] **V06-MT-04.V2** — Inject process cleanup failure and confirm it is preserved rather than silently returning a successful tuning result. **Trace:** [Audit MT-04](./localmotive-comprehensive-audit.md#mt-04).
- [ ] **V06-MT-04.V3** — Run packaged Windows cancellation scenarios and record stop latency, surviving child processes, listener ownership, port release, and ability to start the next operation. **Trace:** [Audit MT-04](./localmotive-comprehensive-audit.md#mt-04).

**Complete when:** Stop prevents additional paid proposals and ends in-flight local work within the documented bound, or returns an explicit cleanup failure with retained ownership information. A cancelled session leaves no silently abandoned runtime tree and reports cancellation separately from failed configurations.

**Scope / decision note:** Source inspection confirmed missing cancellation propagation. Mock loop tests cannot certify Windows process-tree cleanup or real provider cancellation, which require the stated target-environment acceptance evidence.

### V06-MT-05

**Reserve one backend operation owner and preserve server identity throughout measurement**

**Status:** Implemented; unit-verified (commit `bb9a5ab`); packaged cancellation/restart flows open (V06-G-04/G-05) · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit MT-05](./localmotive-comprehensive-audit.md#mt-05)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs)

**Implementation**

- [x] **V06-MT-05.I1** — Introduce a single Rust operation coordinator with atomic reservations for ordinary startup, warm benchmarks, cold attempts, tuning, and quality suites. **Trace:** [Audit MT-05](./localmotive-comprehensive-audit.md#mt-05).
- [x] **V06-MT-05.I2** — Retain a process-generation lease for each operation, including privately launched cold runtimes, and make Start reject or wait while another owner remains active. **Trace:** [Audit MT-05](./localmotive-comprehensive-audit.md#mt-05).
- [x] **V06-MT-05.I3** — Make Stop cancel the actual operation owner and verify the expected process/listener generation for each request; reject attribution if replacement or ownership loss occurs. **Trace:** [Audit MT-05](./localmotive-comprehensive-audit.md#mt-05).
- [x] **V06-MT-05.I4** — Capture model/runtime identity before quality requests and associate process-memory samples with the serving process; keep long I/O outside the existing server mutex. **Trace:** [Audit MT-05](./localmotive-comprehensive-audit.md#mt-05).

**Verification**

- [x] **V06-MT-05.V1** — Use deterministic barriers to reproduce warm-benchmark/stop/restart, cold-benchmark/start/tune, quality/restart, and simultaneous start_server/start_tuning interleavings. **Trace:** [Audit MT-05](./localmotive-comprehensive-audit.md#mt-05).
- [x] **V06-MT-05.V2** — Assert no second owner is granted, no old-PID memory evidence is joined to replacement responses, and replaced-server results cannot be finalized under the original identity. **Trace:** [Audit MT-05](./localmotive-comprehensive-audit.md#mt-05).
- [ ] **V06-MT-05.V3** — Exercise packaged Windows cancellation and restart flows, preserving records of process generations, listener ownership, and eventual reservation release. **Trace:** [Audit MT-05](./localmotive-comprehensive-audit.md#mt-05).

**Complete when:** All managed inference operations participate in the same ownership rules, including cold attempts that are absent from the ordinary managed-server slot. Concurrent stop/start requests cannot silently switch the model serving a recorded trial or quality case, and lifecycle protection does not block UI responsiveness through a long-held mutex.

**Scope / decision note:** The audit established possible interleavings from source; it did not reproduce their timing on Windows. Ownership protection must not reintroduce the separately audited synchronous-startup blocking behavior.

### V06-MT-06

**Honor local TLS and API-key settings in all internal server clients**

**Status:** Implemented I1-I4, V1/V2 verified (commit `8c73679`); V3 packaged-runtime run pending · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit MT-06](./localmotive-comprehensive-audit.md#mt-06)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/core.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/measurement.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs)

**Implementation**

- [x] **V06-MT-06.I1** — Centralize a Rust-owned local HTTP client constructed from the validated launch profile, and use it consistently for health, tokenization, benchmark, and quality endpoints. **Trace:** [Audit MT-06](./localmotive-comprehensive-audit.md#mt-06).
- [x] **V06-MT-06.I2** — Support configured TLS with an explicit certificate-trust policy and API-key authentication read only by Rust; keep key contents out of frontend state, events, logs, and manifest arguments. **Trace:** [Audit MT-06](./localmotive-comprehensive-audit.md#mt-06).
- [x] **V06-MT-06.I3** — Add correct HTTP framing, bounded request/response sizes, and cancellation-aware whole-operation deadlines covering connection, writes, and reads. **Trace:** [Audit MT-06](./localmotive-comprehensive-audit.md#mt-06).
- [x] **V06-MT-06.I4** — Until a profile's secured transport is supported, reject that combination before launch with a precise recovery message instead of allowing a ten-minute plaintext health timeout. **Trace:** [Audit MT-06](./localmotive-comprehensive-audit.md#mt-06).

**Verification**

- [x] **V06-MT-06.V1** — Test trusted local TLS, rejected invalid certificates, API-key-protected completion, and missing/wrong keys; assert secret canaries never appear in observable diagnostics. **Trace:** [Audit MT-06](./localmotive-comprehensive-audit.md#mt-06).
- [x] **V06-MT-06.V2** — Cover bracketed IPv6, chunked JSON, excessive response size, slow writes/reads, and cancellation during connection and response waits. **Trace:** [Audit MT-06](./localmotive-comprehensive-audit.md#mt-06).
- [ ] **V06-MT-06.V3** — Run accepted TLS/key profiles against the packaged target runtime and record successful health, benchmark, and quality behavior. **Trace:** [Audit MT-06](./localmotive-comprehensive-audit.md#mt-06).

**Complete when:** Every accepted security-supported profile works through the corresponding internal client, or fails explicitly before launching if its transport is unsupported. Certificate validation remains enabled and request bounds/deadlines apply to the whole operation rather than only response reads.

**Scope / decision note:** The audit confirmed plaintext/no-Authorization call paths; adjacent framing and deadline issues are robustness gaps, not all demonstrated failures against the pinned runtime. Do not resolve TLS support by globally disabling verification.

### V06-MT-07

**Version calibration compatibility around complete execution and hardware identity**

**Status:** Not started · **Priority:** Medium · **Owner:** Unassigned  
**Audit trace:** [Audit MT-07](./localmotive-comprehensive-audit.md#mt-07)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/calibration.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/calibration.rs), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/measurement.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs)

**Implementation**

- [ ] **V06-MT-07.I1** — Replace the manually selected compatibility fields with a versioned canonical execution snapshot including effective performance/quality-relevant arguments and observed per-slot context. **Trace:** [Audit MT-07](./localmotive-comprehensive-audit.md#mt-07).
- [ ] **V06-MT-07.I2** — Cover thread counts, flash attention, KV/CPU offload, fit parameters, speculation/draft settings, device placement, overrides, extra options, and content identities for LoRA or other influences. **Trace:** [Audit MT-07](./localmotive-comprehensive-audit.md#mt-07).
- [ ] **V06-MT-07.I3** — Include CPU, RAM, platform, and metric/estimator identity where they affect applicability; define when missing driver or hardware facts make reuse insufficiently supported. **Trace:** [Audit MT-07](./localmotive-comprehensive-audit.md#mt-07).
- [ ] **V06-MT-07.I4** — Migrate or explicitly invalidate older calibration identities while retaining replay's separate command-argument comparison and preventing secret values from entering exposed identity records. **Trace:** [Audit MT-07](./localmotive-comprehensive-audit.md#mt-07).

**Verification**

- [ ] **V06-MT-07.V1** — Add table/property-driven checks changing each material field independently, including fields previously absent from CompatibilityIdentity, and assert compatibility changes appropriately. **Trace:** [Audit MT-07](./localmotive-comprehensive-audit.md#mt-07).
- [ ] **V06-MT-07.V2** — Test CPU-only machines, the same GPU with a changed CPU, fit-reduced effective context, changed draft settings with the same companion, and changed LoRA bytes under the same filename. **Trace:** [Audit MT-07](./localmotive-comprehensive-audit.md#mt-07).
- [ ] **V06-MT-07.V3** — Verify old-schema calibration cannot silently masquerade as a current full identity, unknown identity has the documented outcome, and replay still rejects changed command arguments. **Trace:** [Audit MT-07](./localmotive-comprehensive-audit.md#mt-07).

**Complete when:** Material execution/content/hardware changes invalidate calibration reuse, while equivalent canonical snapshots remain deterministic across serialization. Compatibility output identifies its schema and estimator scope without exposing credentials or treating unknown identity as proven equality.

**Scope / decision note:** The audit's main defect concerns calibration reuse. Replay already compares launch.command_args in addition to the key; do not describe this omission as a demonstrated replay bypass.

### V06-MT-08

**Create calibration anchors from unique persisted benchmark runs**

**Status:** Implemented + unit/UI-verified · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit MT-08](./localmotive-comprehensive-audit.md#mt-08)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src/V03EvidencePanel.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx), [src-tauri/src/calibration.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/calibration.rs)

**Implementation**

- [x] **V06-MT-08.I1** — Create anchors in Rust from a persisted, validated benchmark run/manifest identity plus an explicit estimator identity and estimate value, instead of accepting a click-stamped copy of a mean. **Trace:** [Audit MT-08](./localmotive-comprehensive-audit.md#mt-08).
- [x] **V06-MT-08.I2** — Derive observation time from the source run, enforce source-run uniqueness, and retain that identity across loading, deletion/reimport, and repeated requests. **Trace:** [Audit MT-08](./localmotive-comprehensive-audit.md#mt-08).
- [x] **V06-MT-08.I3** — Define eligibility for partial, failed, or cancelled runs and enforce it independently of the frontend's presence-of-summary check. **Trace:** [Audit MT-08](./localmotive-comprehensive-audit.md#mt-08).
- [x] **V06-MT-08.I4** — Display the actual eligible unique-run count, prevent repeated Add anchor actions from manufacturing samples, and document the existing interval formula using the modest Estimated interval label. **Trace:** [Audit MT-08](./localmotive-comprehensive-audit.md#mt-08).

**Verification**

- [x] **V06-MT-08.V1** — Click Add anchor three times for one benchmark and assert only one eligible anchor exists and the three-run model gate remains closed; repeat with different manually entered estimates. **Trace:** [Audit MT-08](./localmotive-comprehensive-audit.md#mt-08).
- [x] **V06-MT-08.V2** — Confirm three independent compatible run identities can build a model, original observation times survive import, and reimporting the same source cannot create another independent sample. **Trace:** [Audit MT-08](./localmotive-comprehensive-audit.md#mt-08).
- [x] **V06-MT-08.V3** — Test the explicit failed/cancelled/partial-run policy and verify repeated source samples cannot create a misleading zero-width interval by satisfying the count gate. **Trace:** [Audit MT-08](./localmotive-comprehensive-audit.md#mt-08).

**Complete when:** The minimum anchor count reflects distinct eligible benchmark runs, not array entries, clicks, or reimport timestamps. Every anchor can be traced to its measured source and estimator, and UI wording does not claim statistically validated confidence or prediction coverage.

**Scope / decision note:** The audited interval is a descriptive mean-ratio spread using 1.96 times population standard deviation, not established small-sample predictive coverage. Fixing duplication alone does not prove calibration usefulness.

### V06-MT-09

**Bind quality results to immutable content, configuration, and process identity**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit MT-09](./localmotive-comprehensive-audit.md#mt-09)  
**Prerequisites:** [V06-MT-05](#v06-mt-05), [V06-MT-07](#v06-mt-07)
**Source touchpoints:** [src-tauri/src/recommend.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/recommend.rs), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/artifact.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/artifact.rs), [src-tauri/src/sharing.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/sharing.rs), [src/model.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.ts)

**Implementation**

- [x] **V06-MT-09.I1** — Capture full model-content and effective launch identities before quality requests, including companion/LoRA influences, runtime identity, suite/version, harness, and observation time. **Trace:** [Audit MT-09](./localmotive-comprehensive-audit.md#mt-09).
- [x] **V06-MT-09.I2** — Use the shared operation-generation ownership to ensure all quality cases were served by the same expected process and reject results after identity changes. **Trace:** [Audit MT-09](./localmotive-comprehensive-audit.md#mt-09).
- [x] **V06-MT-09.I3** — Enforce compatibility in a Rust join boundary used by benchmark candidates and share exports; do not rely on logical header identity or unchecked frontend qualityPassRate attachment. **Trace:** [Audit MT-09](./localmotive-comprehensive-audit.md#mt-09).
- [x] **V06-MT-09.I4** — Retain per-candidate quality history and consistently label the existing READY/JSON suite as two structural smoke cases rather than broad semantic quality. **Trace:** [Audit MT-09](./localmotive-comprehensive-audit.md#mt-09).

**Verification**

- [x] **V06-MT-09.V1** — Reject quality attachment after changing KV precision, speculation, LoRA, companions, or model tensor bytes while preserving the filename, header, and file size. **Trace:** [Audit MT-09](./localmotive-comprehensive-audit.md#mt-09).
- [x] **V06-MT-09.V2** — Test another model/runtime result at the Rust join boundary and stop/restart between quality cases; assert no stale pass rate is attached to the current candidate. **Trace:** [Audit MT-09](./localmotive-comprehensive-audit.md#mt-09).
- [x] **V06-MT-09.V3** — Verify compatible historical results remain associated with their original candidate, and displays explain that a 1.0 rate means both structural cases passed. **Trace:** [Audit MT-09](./localmotive-comprehensive-audit.md#mt-09).

**Complete when:** A quality result can accompany benchmark/share evidence only when complete immutable content and effective configuration identities agree. Process replacement invalidates the suite, and historical quality results cannot migrate through whichever result happens to be selected in the UI.

**Scope / decision note:** The two existing checks do not measure reasoning, multilingual performance, hallucination, or quantization quality. Stronger identity makes their limited evidence trustworthy without broadening what a pass proves.

### V06-MT-10

**Make Pareto dominance and preference scores consistent under missing metrics**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit MT-10](./localmotive-comprehensive-audit.md#mt-10)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/recommend.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/recommend.rs), [src/V03EvidencePanel.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx)

**Implementation**

- [x] **V06-MT-10.I1** — Define one objective set and completeness policy before comparison; treat missing required evidence as ineligible or incomparable instead of dropping a different set of dimensions for each pair. **Trace:** [Audit MT-10](./localmotive-comprehensive-audit.md#mt-10).
- [x] **V06-MT-10.I2** — Expose evidence coverage and prevent direct score comparison over candidate-specific weight denominators that reward omitted weak measurements. **Trace:** [Audit MT-10](./localmotive-comprehensive-audit.md#mt-10).
- [x] **V06-MT-10.I3** — Precompute each objective range once, preserve explanations of constraint violations/dominators/components, and define duplicate-ID and deterministic-tie behavior. **Trace:** [Audit MT-10](./localmotive-comprehensive-audit.md#mt-10).
- [x] **V06-MT-10.I4** — If 10,000-candidate support remains, bound the returned dominance detail and measure worst-case runtime rather than repeatedly allocating per-candidate range vectors. **Trace:** [Audit MT-10](./localmotive-comprehensive-audit.md#mt-10).

**Verification**

- [x] **V06-MT-10.V1** — Use three Measured candidates all with decode=100: A prefill=100/latency=unknown/quality=0.5; B prefill=90/latency=10/quality=unknown; C prefill=unknown/latency=20/quality=0.9. Assert no A>B>C>A cycle under default null metric constraints. **Trace:** [Audit MT-10](./localmotive-comprehensive-audit.md#mt-10).
- [x] **V06-MT-10.V2** — Verify missing required quality cannot raise a candidate above fully measured alternatives solely through a denominator change; cover all-missing, equal values, zero weights, constrained unknowns, duplicate IDs, and ties. **Trace:** [Audit MT-10](./localmotive-comprehensive-audit.md#mt-10).
- [ ] **V06-MT-10.V3** — Record worst-case latency and response size for the retained candidate limit and confirm explanations remain deterministic. **Trace:** [Audit MT-10](./localmotive-comprehensive-audit.md#mt-10).

**Complete when:** Dominance is acyclic under the documented missing-data policy, and the positive-decode counterexample cannot erase the entire frontier through cyclic comparisons. Preference scores disclose comparable objective coverage and cannot improve merely by omitting an unfavorable measurement.

**Scope / decision note:** The audit counterexample passes candidate validation and UI defaults; it was derived from source rather than executed as a Rust regression. Choose and document the intended incomparability policy before asserting exact frontier membership.

### V06-MT-11

**Measure the declared tuning workload and enforce effective context requirements**

**Status:** Implemented + unit-verified (V2 long-prompt half N/A by the labelled short-prompt path) · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit MT-11](./localmotive-comprehensive-audit.md#mt-11)  
**Prerequisites:** [V06-MT-01](#v06-mt-01), [V06-MT-04](#v06-mt-04)
**Source touchpoints:** [src-tauri/src/tune.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/tune.rs), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/core.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs)

**Implementation**

- [x] **V06-MT-11.I1** — Separate allocated context capacity, occupied prompt length, per-slot capacity, concurrency, quality policy, latency, and decode throughput in the declared tuning objective. **Trace:** [Audit MT-11](./localmotive-comprehensive-audit.md#mt-11).
- [x] **V06-MT-11.I2** — Use the corrected V2 harness with an immutable target-length workload and per-trial raw manifests, or explicitly label the retained short-prompt objective without claiming a full-context workload. **Trace:** [Audit MT-11](./localmotive-comprehensive-audit.md#mt-11).
- [x] **V06-MT-11.I3** — Before scoring, require observed effective per-slot context to meet the selected requirement; reject or explicitly report candidates whose parallelism or automatic fit reduces it. **Trace:** [Audit MT-11](./localmotive-comprehensive-audit.md#mt-11).
- [x] **V06-MT-11.I4** — Remeasure baseline and finalists, require a documented material improvement beyond observed variation, and apply an explicit quality policy for cache/speculation changes while keeping any three-field proposal limit enforced in Rust. **Trace:** [Audit MT-11](./localmotive-comprehensive-audit.md#mt-11).

**Verification**

- [x] **V06-MT-11.V1** — Test candidates whose parallel setting divides context below target and whose fit policy reduces context; assert they cannot silently win the requested-capacity objective. **Trace:** [Audit MT-11](./localmotive-comprehensive-audit.md#mt-11).
- [x] **V06-MT-11.V2** — Verify the long-prompt request/count contract and persisted workload for each candidate, including cancellation and raw failure retention through the V2 path. **Trace:** [Audit MT-11](./localmotive-comprehensive-audit.md#mt-11).
- [x] **V06-MT-11.V3** — Confirm the selected winner with independent repeated measurements under the same workload and exercise the stated quality policy for precision/speculation changes. **Trace:** [Audit MT-11](./localmotive-comprehensive-audit.md#mt-11).

**Complete when:** The tuning report's objective matches the workload actually requested and the effective context observed, with raw evidence available for each scored candidate. A winner satisfies capacity and quality policy and has a recorded confirmation comparison rather than only the largest noisy single-session mean.

**Scope / decision note:** The audit did not run long-context GPU tuning. Allocating a large context is not equivalent to occupying it; either supported objective is acceptable only when accurately labeled and verified.

### V06-MT-12

**Preserve benchmark manifests when runtime failures exceed error text limits**

**Status:** Implemented; unit-verified (commit `9d4c36c`); retained-output evidence recorded in the ledger · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit MT-12](./localmotive-comprehensive-audit.md#mt-12)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/measurement.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs), [src-tauri/src/evidence.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/evidence.rs)

**Implementation**

- [x] **V06-MT-12.I1** — Normalize startup and health failures into bounded structured observation errors at acquisition, preserving their outcome/category instead of storing arbitrary serialized launch evidence inline. **Trace:** [Audit MT-12](./localmotive-comprehensive-audit.md#mt-12).
- [x] **V06-MT-12.I2** — Retain larger bounded runtime diagnostics separately and attach an explicit artifact reference/digest where available, distinguishing retained local logs from public share content. **Trace:** [Audit MT-12](./localmotive-comprehensive-audit.md#mt-12).
- [x] **V06-MT-12.I3** — Apply Unicode-safe truncation with a visible truncation marker within the 4,096-byte observation limit; do not discard the whole run because one failure carries a longer log tail. **Trace:** [Audit MT-12](./localmotive-comprehensive-audit.md#mt-12).
- [x] **V06-MT-12.I4** — Ensure finalization retains earlier successes, warmup failures, cancellation/timeouts, and the original diagnostic category before atomic manifest persistence. **Trace:** [Audit MT-12](./localmotive-comprehensive-audit.md#mt-12).

**Verification**

- [x] **V06-MT-12.V1** — Inject a cold-start health failure whose structured error/log tail exceeds 4,096 bytes and assert a valid manifest is written with bounded summary and retained-diagnostic reference. **Trace:** [Audit MT-12](./localmotive-comprehensive-audit.md#mt-12).
- [x] **V06-MT-12.V2** — Repeat for warmup failure and a later failed trial after successful observations; verify existing measurements and terminal outcome survive finalization. **Trace:** [Audit MT-12](./localmotive-comprehensive-audit.md#mt-12).
- [x] **V06-MT-12.V3** — Exercise boundary-length and multibyte text, plus diagnostic-artifact failure, ensuring the user receives the original failure category rather than only a schema-limit error. **Trace:** [Audit MT-12](./localmotive-comprehensive-audit.md#mt-12).

**Complete when:** Rich runtime errors cannot prevent preservation of the corresponding bounded benchmark record or already collected valid observations. The manifest explains truncation and diagnostic availability explicitly, and any retained larger log remains within the local evidence/privacy policy.

**Scope / decision note:** This is the audited conflict between rich launch errors and the existing manifest schema bound. Target-runtime failure scenarios were not executed during the audit; closure requires retained output from the new regression.

### V06-MT-13

**Unify finalized evidence validation across manifests, summaries, replay, and sharing**

**Status:** Implemented + unit-verified (deterministic corpus, not a fuzzer — recorded residual) · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit MT-13](./localmotive-comprehensive-audit.md#mt-13)  
**Prerequisites:** [V06-MT-01](#v06-mt-01)
**Source touchpoints:** [src-tauri/src/evidence.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/evidence.rs), [src-tauri/src/measurement.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs), [src-tauri/src/sharing.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/sharing.rs)

**Implementation**

- [x] **V06-MT-13.I1** — Create shared attempt/workload invariants covering positive unique sequential IDs, valid successful metrics/counts under the chosen cache protocol, requested attempt completion, and coherent terminal outcomes. **Trace:** [Audit MT-13](./localmotive-comprehensive-audit.md#mt-13).
- [x] **V06-MT-13.I2** — Apply the same finalized-record contract to persistence, summary generation, replay, share construction, and direct share persistence; model draft/incomplete records separately if they are needed. **Trace:** [Audit MT-13](./localmotive-comprehensive-audit.md#mt-13).
- [x] **V06-MT-13.I3** — Validate workload and launch shape plus nested hardware/effective-context/process-memory evidence, and recompute summary statistics/counts including median from the raw observations. **Trace:** [Audit MT-13](./localmotive-comprehensive-audit.md#mt-13).
- [x] **V06-MT-13.I4** — Recompute quality aggregate status from cases and verify attached identities, replacing acceptance of independently supplied contradictory aggregate fields. **Trace:** [Audit MT-13](./localmotive-comprehensive-audit.md#mt-13).

**Verification**

- [x] **V06-MT-13.V1** — Reject duplicate or zero trial IDs, zero-token successes, missing attempts without terminal failure, mismatched cache-aware counts, and contradictory terminal outcomes consistently at every exposed boundary. **Trace:** [Audit MT-13](./localmotive-comprehensive-audit.md#mt-13).
- [x] **V06-MT-13.V2** — Reject altered summary counts/median, invalid workloads/launch shapes, contradictory quality cases/status, wrong quality identities, and invalid nested evidence through both builder and direct-export paths. **Trace:** [Audit MT-13](./localmotive-comprehensive-audit.md#mt-13).
- [x] **V06-MT-13.V3** — Add bounded fuzz/property checks for deserialization and validation agreement, retaining valid finalized and intentionally incomplete fixtures as explicit separate contracts. **Trace:** [Audit MT-13](./localmotive-comprehensive-audit.md#mt-13).

**Complete when:** A finalized record accepted by persistence is also valid for its supported derived summary/export operations, without acceptance disagreements between boundaries. Derived summaries and quality aggregates cannot contradict their raw evidence, and partial execution always has an explicit coherent state.

**Scope / decision note:** Consistent validation establishes record integrity, not cryptographic authenticity that a benchmark occurred. Local editable JSON remains untrusted evidence; preserve the audit's distinction rather than presenting validation as signing or attestation.

### V06-MT-14

**Enforce calibration model invariants and evidence freshness at every boundary**

**Status:** Implemented + unit-verified · **Priority:** Low · **Owner:** sato942  
**Audit trace:** [Audit MT-14](./localmotive-comprehensive-audit.md#mt-14)  
**Prerequisites:** [V06-MT-08](#v06-mt-08)
**Source touchpoints:** [src-tauri/src/calibration.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/calibration.rs), [src/model.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.ts)

**Implementation**

- [x] **V06-MT-14.I1** — Reuse one complete calibration-model validator from build, persist, load, and apply, enforcing valid anchor count, creation/expiry ordering, finite metrics, and the supported TTL bound. **Trace:** [Audit MT-14](./localmotive-comprehensive-audit.md#mt-14).
- [x] **V06-MT-14.I2** — Reject application before model creation, evaluate exact expiry consistently, and define freshness from original source-run timestamps rather than allowing rebuild time to refresh arbitrarily old evidence. **Trace:** [Audit MT-14](./localmotive-comprehensive-audit.md#mt-14).
- [x] **V06-MT-14.I3** — Require compatible estimator/metric applicability and preserve source provenance supplied by unique-run anchors; expose unsupported or stale models as such instead of returning derived estimates. **Trace:** [Audit MT-14](./localmotive-comprehensive-audit.md#mt-14).
- [x] **V06-MT-14.I4** — Align frontend/backend expiry semantics, preferably through a backend evaluation result, and check the final lower and upper interval bounds for finiteness after arithmetic. **Trace:** [Audit MT-14](./localmotive-comprehensive-audit.md#mt-14).

**Verification**

- [x] **V06-MT-14.V1** — Test zero/insufficient anchor count, inverted timestamps, future creation, overlong TTL, key mismatch, and invalid finite arithmetic through every model entry point. **Trace:** [Audit MT-14](./localmotive-comprehensive-audit.md#mt-14).
- [x] **V06-MT-14.V2** — Test one tick before, exactly at, and after expiry in both UI presentation and backend application; assert a single consistent decision. **Trace:** [Audit MT-14](./localmotive-comprehensive-audit.md#mt-14).
- [x] **V06-MT-14.V3** — Rebuild from aged source anchors and confirm the documented freshness policy prevents silently renewing applicability solely through a new creation timestamp. **Trace:** [Audit MT-14](./localmotive-comprehensive-audit.md#mt-14).

**Complete when:** The apply API cannot accept a model that violates persistence invariants, and no future-created or expired model yields a valid estimate. Freshness remains traceable to measured source times, frontend/backend state agrees at expiry, and returned interval endpoints are finite.

**Scope / decision note:** Choose and document the source-age and applicability policy rather than inventing a universal statistical shelf life. Correct validator reuse does not establish predictive interval accuracy; that remains an empirical validation question.

### V06-MT-15

**Use the artifact shard contract for discovery completeness**

**Status:** Implemented + unit-verified · **Priority:** Low · **Owner:** sato942  
**Audit trace:** [Audit MT-15](./localmotive-comprehensive-audit.md#mt-15)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/core.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs), [src-tauri/src/artifact.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/artifact.rs)

**Implementation**

- [x] **V06-MT-15.I1** — Route discovery's grouped shard names through the artifact module's analysis instead of deciding completeness from a HashSet that silently drops parse failures and duplicate file indices. **Trace:** [Audit MT-15](./localmotive-comprehensive-audit.md#mt-15).
- [x] **V06-MT-15.I2** — Require every grouped file to parse consistently, agree on logical identity and expected count, and contribute exactly one unique required index. **Trace:** [Audit MT-15](./localmotive-comprehensive-audit.md#mt-15).
- [x] **V06-MT-15.I3** — Return explicit duplicate, malformed, inconsistent-count, and missing-shard problems to discovery so the displayed complete state matches launch validation. **Trace:** [Audit MT-15](./localmotive-comprehensive-audit.md#mt-15).
- [x] **V06-MT-15.I4** — Preserve deterministic first-shard selection, valid singleton behavior, split ordering, and existing symlink/reparse protections while replacing the divergent decision logic. **Trace:** [Audit MT-15](./localmotive-comprehensive-audit.md#mt-15).

**Verification**

- [x] **V06-MT-15.V1** — Create a real temporary tree containing foo.gguf and foo-00001-of-00001.gguf; assert discovery and artifact validation agree that the duplicate index does not form a complete valid model. **Trace:** [Audit MT-15](./localmotive-comprehensive-audit.md#mt-15).
- [x] **V06-MT-15.V2** — Cover duplicate indices, malformed index/count, conflicting expected counts, missing first shard, extension case variation, and valid complete singleton/split sets. **Trace:** [Audit MT-15](./localmotive-comprehensive-audit.md#mt-15).
- [x] **V06-MT-15.V3** — Compare completeness and diagnostic outcomes across scanning and launch inspection for each fixture and verify only the intended first shard is selected for valid models. **Trace:** [Audit MT-15](./localmotive-comprehensive-audit.md#mt-15).

**Complete when:** Discovery no longer reports complete for a shard group the shared artifact contract rejects, and invalid groups expose actionable reasons. Valid model selection remains deterministic and retains the existing downstream launch and filesystem protections.

**Scope / decision note:** The stricter downstream inspector already limits this defect to misleading discovery and rejected actions. This task does not claim to validate all GGUF tensor completeness or to reimplement the inference engine's loader.

## Frontend tasks

### V06-FE-01

**Unify selected model and runtime with the profile submitted for launch**

**Status:** Implemented; unit-verified (commit `7d4a514`); packaged select/rescan/inspect/save/start scenario open (V06-G-04) · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit FE-01](./localmotive-comprehensive-audit.md#fe-01)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx), [src/model.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.ts), [src/model.test.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.test.ts)

**Implementation**

- [x] **V06-FE-01.I1** — Represent committed model selection and its editable launch profile as one coordinated state transition; keep uncommitted runtime-path input separate from the active runtime identity. **Trace:** [Audit FE-01](./localmotive-comprehensive-audit.md#fe-01).
- [x] **V06-FE-01.I2** — Preserve the selected model during rescan when it still exists; otherwise load and normalize the replacement model's saved profile or clear selection/profile together, including failed scans. **Trace:** [Audit FE-01](./localmotive-comprehensive-audit.md#fe-01).
- [x] **V06-FE-01.I3** — Route typed runtime inspection, native file selection, managed-build activation and installation completion through the same committed-runtime update so profile.runtime and inspected capabilities agree. **Trace:** [Audit FE-01](./localmotive-comprehensive-audit.md#fe-01).
- [x] **V06-FE-01.I4** — Derive displayed model identity, readiness, completeness checks and persistence keys from the actual pending launch profile and its validated model ownership rather than unrelated selectedId state. **Trace:** [Audit FE-01](./localmotive-comprehensive-audit.md#fe-01).
- [x] **V06-FE-01.I5** — Preserve the existing normalization rule that a deliberately selected current runtime supersedes an older runtime saved with a profile. **Trace:** [Audit FE-01](./localmotive-comprehensive-audit.md#fe-01).

**Verification**

- [x] **V06-FE-01.V1** — Add component/IPC-boundary regressions selecting model B, rescanning inventory ordered A/B, deleting B, and failing the scan; assert visible identity and exact submitted/saved profile values. **Trace:** [Audit FE-01](./localmotive-comprehensive-audit.md#fe-01).
- [x] **V06-FE-01.V2** — Exercise typed runtime B plus Inspect, file-picker B and managed activation B after configuring A; assert runtimePath, inspected runtime identity and profile.runtime converge. **Trace:** [Audit FE-01](./localmotive-comprehensive-audit.md#fe-01).
- [ ] **V06-FE-01.V3** — Run the packaged select, rescan, inspect, save and start scenario and capture the resulting server snapshot and command arguments. **Trace:** [Audit FE-01](./localmotive-comprehensive-audit.md#fe-01).

**Complete when:** No supported rescan or runtime-selection route displays one model/runtime while saving or launching another. Saved profiles remain under their originating model's key, and missing or failed selections have an explicit recoverable state.

**Scope / decision note:** The audit confirms inconsistent frontend state paths, not an independently executed wrong-model Windows launch. Backend validation must remain authoritative and cannot substitute for reconciling user intent.

### V06-FE-02

**Bind tuning progress, reports and adoption to immutable run identity**

**Status:** Implemented; unit-verified (commit `7d4a514`); packaged navigation scenario open (V06-G-04) · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit FE-02](./localmotive-comprehensive-audit.md#fe-02)  
**Prerequisites:** [V06-FE-01](#v06-fe-01)
**Source touchpoints:** [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx), [src/model.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.ts)

**Implementation**

- [x] **V06-FE-02.I1** — Create a tuning-run record capturing the originating model ID, runtime identity, provider, advisor model and target context at dispatch, independent of the currently edited selection. **Trace:** [Audit FE-02](./localmotive-comprehensive-audit.md#fe-02).
- [x] **V06-FE-02.I2** — Route progress, completion and stored reports through that run identity; retain the original run's provenance when navigation selects another model or a separate draft provider. **Trace:** [Audit FE-02](./localmotive-comprehensive-audit.md#fe-02).
- [x] **V06-FE-02.I3** — Make Adopt best operate on the report's originating model instead of the current selectedId/name/context, and explicitly reconcile runtime selection through the established profile normalization policy. **Trace:** [Audit FE-02](./localmotive-comprehensive-audit.md#fe-02).
- [x] **V06-FE-02.I4** — Prevent delayed progress from a finished run from updating a subsequent run; choose whether incompatible edits are disabled during tuning or clearly represented as separate drafts. **Trace:** [Audit FE-02](./localmotive-comprehensive-audit.md#fe-02).
- [x] **V06-FE-02.I5** — Display the actual running advisor/model/context and completion state from the immutable record so changing provider tabs cannot relabel work already dispatched. **Trace:** [Audit FE-02](./localmotive-comprehensive-audit.md#fe-02).

**Verification**

- [x] **V06-FE-02.V1** — Start tuning A, select B before deferred progress/completion arrives, then adopt; assert B's report, profile and storage key are not overwritten by A. **Trace:** [Audit FE-02](./localmotive-comprehensive-audit.md#fe-02).
- [ ] **V06-FE-02.V2** — Activate runtime B after measuring on runtime A and verify adoption follows the documented current-runtime policy without silently restoring A. **Trace:** [Audit FE-02](./localmotive-comprehensive-audit.md#fe-02).
- [x] **V06-FE-02.V3** — Complete a run, start another and deliver late events from the first; verify the second run's trials and status remain unchanged. **Trace:** [Audit FE-02](./localmotive-comprehensive-audit.md#fe-02).

**Complete when:** Every visible trial/report identifies its originating run and can be adopted only into the corresponding model's profile. Navigation and provider/model draft changes preserve active tuning provenance, and late events cannot contaminate another run.

**Scope / decision note:** This task addresses frontend ownership and adoption. Backend advisor budgeting, cancellation and measurement identity remain separate audit tasks; the audit did not execute this sequence on packaged Windows.

### V06-FE-03

**Reject stale cloud, GGUF, port-suggestion and command-preview responses**

**Status:** Implemented; unit-verified (commit `411c32c`); deferred-promise component scenarios open (V06-G-04) · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit FE-03](./localmotive-comprehensive-audit.md#fe-03)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx), [src/model.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.ts), [src/model.test.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.test.ts)

**Implementation**

- [x] **V06-FE-03.I1** — Extend the existing request-sequence approach to cloud credential/model loads, connection probes, OAuth completion, key save/forget, GGUF reads, port suggestions and command preview. **Trace:** [Audit FE-03](./localmotive-comprehensive-audit.md#fe-03).
- [x] **V06-FE-03.I2** — Capture the resource identity and form revision for each request and check them before committing success, failure or loading completion; obsolete responses must not replace current state. **Trace:** [Audit FE-03](./localmotive-comprehensive-audit.md#fe-03).
- [x] **V06-FE-03.I3** — Clear provider-specific credential/model/probe presentation at provider-switch start, and clear GGUF metadata when selection changes or becomes absent rather than retaining another model's facts. **Trace:** [Audit FE-03](./localmotive-comprehensive-audit.md#fe-03).
- [x] **V06-FE-03.I4** — Apply a suggested port only when both profile identity and the originally requested port remain unchanged; never overwrite a later manual edit or newly selected profile. **Trace:** [Audit FE-03](./localmotive-comprehensive-audit.md#fe-03).
- [x] **V06-FE-03.I5** — Keep independent loading/error state for these resources and bind tuning readiness to the active provider's own credential and model result. **Trace:** [Audit FE-03](./localmotive-comprehensive-audit.md#fe-03).

**Verification**

- [ ] **V06-FE-03.V1** — Use deferred IPC-boundary promises to resolve provider A after B across credential/model/probe and credential-mutation operations; assert B's tab and actual readiness retain B's data. **Trace:** [Audit FE-03](./localmotive-comprehensive-audit.md#fe-03).
- [x] **V06-FE-03.V2** — Resolve model A metadata after selecting B or clearing selection; verify architecture and native-context choices are not overwritten or incorrectly clamped. **Trace:** [Audit FE-03](./localmotive-comprehensive-audit.md#fe-03).
- [ ] **V06-FE-03.V3** — Return an old port suggestion after a manual edit and an old command after a newer preview; assert both are discarded through actual component callers. **Trace:** [Audit FE-03](./localmotive-comprehensive-audit.md#fe-03).

**Complete when:** Out-of-order completion cannot replace the active provider, model metadata, chosen port or current command preview. Tests exercise production callers and not only the equality helper.

**Scope / decision note:** The cloud issue is incorrect masked status/model presentation, not a demonstrated raw-secret leak. Coordinate preview request guards with FE-04 without removing trusted launch validation.

### V06-FE-04

**Coalesce command previews and move expensive native validation off the synchronous edit path**

**Status:** Implemented; unit-verified (`c23a20f`) · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit FE-04](./localmotive-comprehensive-audit.md#fe-04)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/core.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs)

**Implementation**

- [x] **V06-FE-04.I1** — Debounce or coalesce profile-edit previews so rapid character changes do not dispatch complete prepare_launch work for every intermediate form value. **Trace:** [Audit FE-04](./localmotive-comprehensive-audit.md#fe-04).
- [x] **V06-FE-04.I2** — Separate cheap command composition from authoritative runtime trust, artifact, path and argument validation; describe provisional previews honestly and retain all required checks at actual launch. **Trace:** [Audit FE-04](./localmotive-comprehensive-audit.md#fe-04).
- [x] **V06-FE-04.I3** — Move necessary expensive runtime probing and filesystem verification into a cancellable background operation, preventing synchronous preview work from blocking interface interaction. **Trace:** [Audit FE-04](./localmotive-comprehensive-audit.md#fe-04).
- [x] **V06-FE-04.I4** — Cache runtime capability/artifact evidence only under explicit safe identity and freshness rules; invalidate it when executable/model identity changes rather than treating old evidence as current. **Trace:** [Audit FE-04](./localmotive-comprehensive-audit.md#fe-04).
- [x] **V06-FE-04.I5** — Discard preview results for obsolete form revisions, coordinating with FE-03, and avoid native work for purely human-facing changes such as the profile display name when arguments are unchanged. **Trace:** [Audit FE-04](./localmotive-comprehensive-audit.md#fe-04).

**Verification**

- [x] **V06-FE-04.V1** — Instrument probe/job counts while rapidly editing a profile; assert a bounded coalesced request count and an exact command corresponding only to the final accepted form. **Trace:** [Audit FE-04](./localmotive-comprehensive-audit.md#fe-04).
- [ ] **V06-FE-04.V2** — Simulate slow version/help probes and verify input, navigation and cancellation remain responsive in the packaged Windows binary; record actual observations rather than assumed timing. **Trace:** [Audit FE-04](./localmotive-comprehensive-audit.md#fe-04).
- [ ] **V06-FE-04.V3** — Tamper with or replace a runtime/artifact after cached preview evidence and assert actual launch still performs and enforces the authoritative trust checks. **Trace:** [Audit FE-04](./localmotive-comprehensive-audit.md#fe-04).

**Complete when:** Ordinary typing no longer invokes two runtime subprocess probes per character. Obsolete previews cannot overwrite current output, and launch trust remains enforced independently of preview optimization.

**Scope / decision note:** Ten seconds is the audited per-probe timeout, not a measured per-keystroke delay. The caching strategy must preserve security invariants; performance measurements and packaged acceptance remain future work.

### V06-FE-05

**Preserve evidence history and active measurement controls across navigation**

**Status:** Implemented; unit-verified (commit `0045cb6`); packaged navigation/cancel/completion scenario open (V06-G-04) · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit FE-05](./localmotive-comprehensive-audit.md#fe-05)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx), [src/V03EvidencePanel.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx)

**Implementation**

- [x] **V06-FE-05.I1** — Move benchmark, quality, preflight, ranking, imported-evidence and active-operation records out of the conditionally mounted Benchmark view into persistent application-level state or a dedicated store. **Trace:** [Audit FE-05](./localmotive-comprehensive-audit.md#fe-05).
- [x] **V06-FE-05.I2** — Keep an application-wide run status and cancellation handle available while the user visits Control, Profile, Inventory or another screen; navigation must not orphan a dispatched measurement. **Trace:** [Audit FE-05](./localmotive-comprehensive-audit.md#fe-05).
- [x] **V06-FE-05.I3** — Retain historical evidence with immutable model/runtime/workload/run identities instead of clearing the entire history when a profile fingerprint changes. **Trace:** [Audit FE-05](./localmotive-comprehensive-audit.md#fe-05).
- [x] **V06-FE-05.I4** — Provide a saved-manifest recovery/loading route keyed by evidence identity and reconnect the view to completion received while it was unmounted. **Trace:** [Audit FE-05](./localmotive-comprehensive-audit.md#fe-05).
- [x] **V06-FE-05.I5** — Separate invalidation of the current preflight or editable preview from retention of completed runs, and reset any export approval when its reviewed export content changes. **Trace:** [Audit FE-05](./localmotive-comprehensive-audit.md#fe-05).

**Verification**

- [ ] **V06-FE-05.V1** — Start a deferred benchmark, navigate away and back, then cancel; assert the same run remains active and the original cancellation handle is available. **Trace:** [Audit FE-05](./localmotive-comprehensive-audit.md#fe-05).
- [ ] **V06-FE-05.V2** — Complete measurement while another screen is open and verify returning to Benchmark displays that run and its saved evidence rather than a new empty session. **Trace:** [Audit FE-05](./localmotive-comprehensive-audit.md#fe-05).
- [ ] **V06-FE-05.V3** — Benchmark profile A, leave to edit/start B, benchmark B, then compare both records with distinct provenance and recover them through the supported saved-manifest route. **Trace:** [Audit FE-05](./localmotive-comprehensive-audit.md#fe-05).

**Complete when:** Navigation cannot erase accessible completed results or active measurement controls. Profile changes retain historical candidates while current-input evidence is invalidated appropriately.

**Scope / decision note:** Rust already persists some manifests; the audited defect is frontend accessibility/lifecycle, not proof that every measurement file is lost. Backend server-operation ownership and comparison correctness remain separate findings.

### V06-FE-06

**Invalidate preflight evidence when assumptions change and preserve deliberate empty adapter selection**

**Status:** Implemented + component-verified · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit FE-06](./localmotive-comprehensive-audit.md#fe-06)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src/V03EvidencePanel.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx)

**Implementation**

- [x] **V06-FE-06.I1** — Store the exact selected adapters, hardware observation, manual capacity/note and consequential profile inputs with each preflight result, then derive whether the displayed result is current. **Trace:** [Audit FE-06](./localmotive-comprehensive-audit.md#fe-06).
- [x] **V06-FE-06.I2** — Mark prior preflight/allocation evidence stale or clear its current-result presentation after adapter changes, manual-override edits, hardware refresh or any profile input used by preflight changes. **Trace:** [Audit FE-06](./localmotive-comprehensive-audit.md#fe-06).
- [x] **V06-FE-06.I3** — Replace the incomplete profile fingerprint with a full preflight-input identity including draft/projector, speculation, KV offload, device, fitting and attention settings where relevant. **Trace:** [Audit FE-06](./localmotive-comprehensive-audit.md#fe-06).
- [x] **V06-FE-06.I4** — Distinguish uninitialized adapter selection from the user's deliberate empty selection; initialize defaults once and allow unchecking the final adapter without immediately reselecting another. **Trace:** [Audit FE-06](./localmotive-comprehensive-audit.md#fe-06).
- [x] **V06-FE-06.I5** — Align explicit adapter choice between Runtime and the evidence panel, and reject async inspection/preflight responses whose captured input revision is no longer current. **Trace:** [Audit FE-06](./localmotive-comprehensive-audit.md#fe-06).

**Verification**

- [x] **V06-FE-06.V1** — Run preflight then change an adapter, capacity, note or refreshed hardware observation; assert old budget/allocation output is explicitly stale until recomputed. **Trace:** [Audit FE-06](./localmotive-comprehensive-audit.md#fe-06).
- [x] **V06-FE-06.V2** — Uncheck the final adapter and verify the empty choice persists; refresh hardware with removed adapters and confirm selection reconciliation preserves deliberate intent. **Trace:** [Audit FE-06](./localmotive-comprehensive-audit.md#fe-06).
- [ ] **V06-FE-06.V3** — Resolve an old preflight after changing inputs, and after FE-05 makes state persistent edit each previously omitted profile field; verify stale evidence is never presented as current. **Trace:** [Audit FE-06](./localmotive-comprehensive-audit.md#fe-06).

**Complete when:** Each displayed current preflight result matches the inputs and hardware observation shown to the user. Intentional no-adapter selection survives render/effect cycles and navigation policy.

**Scope / decision note:** The incomplete fingerprint is partly masked today by Benchmark unmounting; retain this regression when FE-05 fixes that lifecycle. An empty adapter selection must not itself fabricate CPU success or bypass backend validation.

### V06-FE-07

**Keep cancellation requested state separate from the running benchmark lifecycle**

**Status:** Implemented; unit-verified (commit `6da40d8`); component acknowledgement scenarios open (V06-G-04) · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit FE-07](./localmotive-comprehensive-audit.md#fe-07)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src/V03EvidencePanel.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs)

**Implementation**

- [x] **V06-FE-07.I1** — Represent benchmark lifecycle and cancellation-request progress separately instead of calling the generic busy-state wrapper with a replacement cancel operation name. **Trace:** [Audit FE-07](./localmotive-comprehensive-audit.md#fe-07).
- [x] **V06-FE-07.I2** — Retain running or cancelling status until the original benchmark promise reaches its terminal outcome, rather than clearing active ownership when the cancel command acknowledges receipt. **Trace:** [Audit FE-07](./localmotive-comprehensive-audit.md#fe-07).
- [x] **V06-FE-07.I3** — Replace null-as-failure action results with an explicit tagged success/error result so a successful Rust unit response can produce truthful cancellation acknowledgement. **Trace:** [Audit FE-07](./localmotive-comprehensive-audit.md#fe-07).
- [x] **V06-FE-07.I4** — Prevent conflicting evidence actions from becoming enabled while the original run remains active, and give Add anchor the same compatible-operation guard used by other evidence actions. **Trace:** [Audit FE-07](./localmotive-comprehensive-audit.md#fe-07).
- [x] **V06-FE-07.I5** — Handle failed or unavailable cancellation by retaining the run record and showing actionable feedback rather than implying termination or losing the remaining cancel/status affordance. **Trace:** [Audit FE-07](./localmotive-comprehensive-audit.md#fe-07).

**Verification**

- [ ] **V06-FE-07.V1** — Resolve cancel acknowledgement before the original benchmark ends; assert the interface stays cancelling and a new benchmark/quality action remains unavailable. **Trace:** [Audit FE-07](./localmotive-comprehensive-audit.md#fe-07).
- [ ] **V06-FE-07.V2** — Exercise successful null/unit responses, rejected cancellation and no-active-benchmark errors through the actual IPC action wrapper; verify distinct messages and lifecycle outcomes. **Trace:** [Audit FE-07](./localmotive-comprehensive-audit.md#fe-07).
- [ ] **V06-FE-07.V3** — Complete, fail and cancel the original run after acknowledgement; assert only its terminal result releases active ownership and conflicting actions. **Trace:** [Audit FE-07](./localmotive-comprehensive-audit.md#fe-07).

**Complete when:** Cancellation acknowledgement is distinguishable from actual measurement completion. Successful void responses are recognized, and no unrelated action clears or prematurely replaces the active run state.

**Scope / decision note:** Rust currently rejects overlapping v2 benchmarks, limiting some overlap; that defense does not replace coherent frontend status. This task does not claim to solve the backend's separate transport cancellation latency.

### V06-FE-08

**Preserve raw argument text while users enter token separators**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** Unassigned  
**Audit trace:** [Audit FE-08](./localmotive-comprehensive-audit.md#fe-08)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx)

**Implementation**

- [x] **V06-FE-08.I1** — Maintain an editable raw-argument string independently of the parsed profile.extraArgs array while the field is being edited so trailing spaces are not immediately normalized away. **Trace:** [Audit FE-08](./localmotive-comprehensive-audit.md#fe-08).
- [x] **V06-FE-08.I2** — Parse and validate the draft at a deliberate boundary such as blur or explicit validation, preserving the existing documented self-contained token syntax. **Trace:** [Audit FE-08](./localmotive-comprehensive-audit.md#fe-08).
- [x] **V06-FE-08.I3** — Keep visible draft text and committed parsed arguments distinguishable when validation fails; show the problematic token and recovery instead of silently concatenating or discarding input. **Trace:** [Audit FE-08](./localmotive-comprehensive-audit.md#fe-08).
- [x] **V06-FE-08.I4** — Define behavior for paste, repeated whitespace, deletion, whitespace-only drafts and unsupported quoted/space-containing values without introducing shell execution or broadening privileged overrides. **Trace:** [Audit FE-08](./localmotive-comprehensive-audit.md#fe-08).
- [x] **V06-FE-08.I5** — Preserve the backend restrictions on typed-field overrides and privileged capabilities; synchronize normalized committed text only after a successful parse/validation transition. **Trace:** [Audit FE-08](./localmotive-comprehensive-audit.md#fe-08).

**Verification**

- [x] **V06-FE-08.V1** — Dispatch actual input events typing --flag-one, a space, and --flag-two=value; assert the visible separator remains and the committed IPC profile contains two argument tokens. **Trace:** [Audit FE-08](./localmotive-comprehensive-audit.md#fe-08).
- [x] **V06-FE-08.V2** — Exercise paste, cursor edits, deletion, repeated whitespace, blank input and unsupported quoting, checking that errors preserve editable input and valid tokens remain deterministic. **Trace:** [Audit FE-08](./localmotive-comprehensive-audit.md#fe-08).
- [x] **V06-FE-08.V3** — Submit disallowed privileged and typed-field override tokens through the corrected field and verify backend rejection remains intact. **Trace:** [Audit FE-08](./localmotive-comprehensive-audit.md#fe-08).

**Complete when:** Users can enter multiple supported tokens through normal typing without relying on pasting a complete string. Parsing errors are recoverable and no change expands the accepted privileged argument surface.

**Scope / decision note:** The audit establishes destructive normalization in a controlled input, not arbitrary shell execution. Quoted values with embedded spaces are a documented syntax decision to resolve explicitly rather than silently promise.

### V06-FE-09

**Recover safely from corrupt persisted state and report save failures separately**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** Unassigned  
**Audit trace:** [Audit FE-09](./localmotive-comprehensive-audit.md#fe-09)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx), [src/model.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.ts), [src/main.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/main.tsx)

**Implementation**

- [x] **V06-FE-09.I1** — Introduce safe versioned parsing, validation and migration for persisted profiles and tuning reports before normalization or render-time array/property access. **Trace:** [Audit FE-09](./localmotive-comprehensive-audit.md#fe-09).
- [x] **V06-FE-09.I2** — Quarantine or isolate invalid records and provide a per-record recovery/reset path with a default profile fallback, preserving unrelated valid user records. **Trace:** [Audit FE-09](./localmotive-comprehensive-audit.md#fe-09).
- [x] **V06-FE-09.I3** — Wrap every settings/profile/report write and separate persistence failure from a successfully completed native tuning or benchmark operation so valid results remain visible. **Trace:** [Audit FE-09](./localmotive-comprehensive-audit.md#fe-09).
- [x] **V06-FE-09.I4** — Add a root error boundary with actionable recovery/diagnostic controls and ensure it does not replace targeted storage error handling. **Trace:** [Audit FE-09](./localmotive-comprehensive-audit.md#fe-09).
- [x] **V06-FE-09.I5** — Define bounded report retention and explicit profile/report export or management controls so accumulated per-model records cannot exhaust browser storage without a recoverable explanation. **Trace:** [Audit FE-09](./localmotive-comprehensive-audit.md#fe-09).

**Verification**

- [x] **V06-FE-09.V1** — Load malformed JSON, null, wrong extraArgs/trials types, older schemas and invalid enum values through normal selection/startup; assert the app remains usable and identifies the affected record. **Trace:** [Audit FE-09](./localmotive-comprehensive-audit.md#fe-09).
- [x] **V06-FE-09.V2** — Inject quota/security write failures after successful native tuning/benchmark completion and in direct profile/settings handlers; verify completion remains visible and save failure is separately explained. **Trace:** [Audit FE-09](./localmotive-comprehensive-audit.md#fe-09).
- [x] **V06-FE-09.V3** — Exercise blocked storage reads and recovery/reset of one corrupt record; assert other saved profiles survive and a valid migrated record still follows current-runtime normalization. **Trace:** [Audit FE-09](./localmotive-comprehensive-audit.md#fe-09).

**Complete when:** Corrupt local records cannot blank the entire interface through the audited parse/render paths. Successful native results and failed persistence have distinct observable outcomes, with documented recovery and retention behavior.

**Scope / decision note:** This is local resilience and schema-handling work, not evidence of remote exploitation. Error boundaries do not catch every event-handler failure; persistence calls still need explicit handling.

### V06-FE-11

**Cancel catalog transfers by immutable job identity and preserve active-job visibility**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** Unassigned  
**Audit trace:** [Audit FE-11](./localmotive-comprehensive-audit.md#fe-11)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx), [src/model.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.ts)

**Implementation**

- [x] **V06-FE-11.I1** — Capture source repository, filename, revision and destination in an immutable transfer record at dispatch, and use a stable job identity for progress and cancellation. **Trace:** [Audit FE-11](./localmotive-comprehensive-audit.md#fe-11).
- [x] **V06-FE-11.I2** — Cancel the original job rather than rebuilding its target from the currently editable modelRoot; preserve the captured destination when the user chooses a new download folder. **Trace:** [Audit FE-11](./localmotive-comprehensive-audit.md#fe-11).
- [x] **V06-FE-11.I3** — Expose active transfers independently of catalog filters and the selected build on a card so changing either cannot hide a running job's status or stop control. **Trace:** [Audit FE-11](./localmotive-comprehensive-audit.md#fe-11).
- [x] **V06-FE-11.I4** — Handle rejected and false cancellation responses explicitly; show stopping only when accepted and retain useful diagnostics rather than leaving an unhandled promise rejection. **Trace:** [Audit FE-11](./localmotive-comprehensive-audit.md#fe-11).
- [x] **V06-FE-11.I5** — Scope already-on-disk and completed-transfer presentation to the relevant destination/revision, retain authoritative backend verification, and provide inventory refresh or a clear completion action. **Trace:** [Audit FE-11](./localmotive-comprehensive-audit.md#fe-11).

**Verification**

- [x] **V06-FE-11.V1** — Start in folder A, change destination to B, then stop; assert the cancellation request targets A's original job and false/failure responses are described truthfully. **Trace:** [Audit FE-11](./localmotive-comprehensive-audit.md#fe-11).
- [x] **V06-FE-11.V2** — Change the selected build and catalog filters while transferring; verify all active jobs remain visible and individually cancellable. **Trace:** [Audit FE-11](./localmotive-comprehensive-audit.md#fe-11).
- [x] **V06-FE-11.V3** — Exercise the same repository/file across different destinations or revisions, completion after selection changes, and verification of a prior done event against the newly chosen destination. **Trace:** [Audit FE-11](./localmotive-comprehensive-audit.md#fe-11).

**Complete when:** Destination/build/filter edits cannot redirect cancellation or hide the only active transfer control. Progress and reuse claims identify the actual source revision and destination, with backend verification still authoritative.

**Scope / decision note:** This task does not resolve signed/local catalog authorization or range-download defects. Frontend filename suffix matches must not become proof of trusted on-disk identity; those boundaries remain native responsibilities.

### V06-FE-12

**Restore keyboard-focus outlines on model-folder and download-destination inputs**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** Unassigned  
**Audit trace:** [Audit FE-12](./localmotive-comprehensive-audit.md#fe-12)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src/App.css](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.css), [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx), [docs/DESIGN.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/DESIGN.md)

**Implementation**

- [x] **V06-FE-12.I1** — Remove the outline reset from .path-bar input or add an explicit .path-bar input:focus-visible rule applying the existing amber outline and offset. **Trace:** [Audit FE-12](./localmotive-comprehensive-audit.md#fe-12).
- [x] **V06-FE-12.I2** — Scope the correction to the Model root and Download destination path-bar fields, both of which have border:0 and no alternative declared focus indicator. **Trace:** [Audit FE-12](./localmotive-comprehensive-audit.md#fe-12).
- [x] **V06-FE-12.I3** — Retain the global :focus-visible treatment for ordinary labeled controls; do not rewrite unrelated input styles on the mistaken assumption that label input overrides the global pseudo-class. **Trace:** [Audit FE-12](./localmotive-comprehensive-audit.md#fe-12).
- [x] **V06-FE-12.I4** — Ensure the restored outline is visible against the path-bar background and is not clipped by surrounding borders, overflow behavior or the narrow-screen layout. **Trace:** [Audit FE-12](./localmotive-comprehensive-audit.md#fe-12).
- [x] **V06-FE-12.I5** — Keep pointer interaction and existing path input behavior intact while satisfying the normative design document's requirement to preserve the amber keyboard-focus ring. **Trace:** [Audit FE-12](./localmotive-comprehensive-audit.md#fe-12).

**Verification**

- [x] **V06-FE-12.V1** — Tab into the model-folder and download-destination inputs and assert a nonzero amber outline in browser computed styles, including after editing and choosing a folder. **Trace:** [Audit FE-12](./localmotive-comprehensive-audit.md#fe-12).
- [x] **V06-FE-12.V2** — Use an ordinary labeled runtime/profile input as a control; verify its existing global outline remains present and no unrelated focus regression is introduced. **Trace:** [Audit FE-12](./localmotive-comprehensive-audit.md#fe-12).
- [x] **V06-FE-12.V3** — Perform packaged Windows keyboard traversal through both path bars at normal and narrow/zoomed layouts, recording whether focus is continuously identifiable. **Trace:** [Audit FE-12](./localmotive-comprehensive-audit.md#fe-12).

**Complete when:** Keyboard users can visibly locate focus in both audited path-bar inputs. Existing focus indicators on ordinary labeled fields remain unchanged and the fix follows the design's amber-ring rule.

**Scope / decision note:** Only .path-bar input has the greater specificity that suppresses the global outline. Generic label input/select selectors have lower specificity and are not part of this confirmed defect; packaged rendering was not audited.

### V06-FE-13

**Complete accessible control names, table semantics and navigation behavior**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** Unassigned  
**Audit trace:** [Audit FE-13](./localmotive-comprehensive-audit.md#fe-13)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx), [src/V03EvidencePanel.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx)

**Implementation**

- [x] **V06-FE-13.I1** — Replace the incomplete inventory role=table structure with a native table containing column headers, cells and clearly named row actions, or explicitly choose a coherent accessible list pattern. **Trace:** [Audit FE-13](./localmotive-comprehensive-audit.md#fe-13).
- [x] **V06-FE-13.I2** — Give every paired N-gram minimum/maximum and map lookup/draft-size input its own associated label identifying its distinct value instead of placing two inputs under one label. **Trace:** [Audit FE-13](./localmotive-comprehensive-audit.md#fe-13).
- [x] **V06-FE-13.I3** — Implement provider tabs with the complete selected-tab, tabpanel, aria-controls and keyboard/roving-tabIndex relationships, or use ordinary buttons in a labeled group with suitable semantics. **Trace:** [Audit FE-13](./localmotive-comprehensive-audit.md#fe-13).
- [x] **V06-FE-13.I4** — Expose the current primary navigation destination semantically and define focus placement or a skip-to-main route after changing screens. **Trace:** [Audit FE-13](./localmotive-comprehensive-audit.md#fe-13).
- [x] **V06-FE-13.I5** — Repair the evidence heading hierarchy so its sections follow the containing heading level while retaining native controls and disclosure behavior. **Trace:** [Audit FE-13](./localmotive-comprehensive-audit.md#fe-13).

**Verification**

- [x] **V06-FE-13.V1** — Add component accessible-role/name assertions for inventory columns/actions and every paired input; run an accessibility checker against representative loaded, empty and error states. **Trace:** [Audit FE-13](./localmotive-comprehensive-audit.md#fe-13).
- [x] **V06-FE-13.V2** — Exercise provider navigation with Tab, Arrow keys and relevant Home/End behavior, and verify selection, focus and displayed panel remain synchronized. **Trace:** [Audit FE-13](./localmotive-comprehensive-audit.md#fe-13).
- [x] **V06-FE-13.V3** — Use Windows Narrator or NVDA in the packaged app to traverse navigation, inventory, profile pairs and evidence headings, recording labels and focus after screen changes. **Trace:** [Audit FE-13](./localmotive-comprehensive-audit.md#fe-13).

**Complete when:** Every audited input has a unique meaningful accessible name, and inventory exposes a coherent structure. Provider and primary navigation support their declared semantics and a predictable keyboard/focus sequence.

**Scope / decision note:** The audit reviewed markup rather than running assistive technology. Native file dialogs are already used; do not invent a custom modal focus-trap requirement or claim complete accessibility certification from these checks.

### V06-FE-14

**Correct low-contrast text and make evidence controls usable in narrow layouts**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** Unassigned  
**Audit trace:** [Audit FE-14](./localmotive-comprehensive-audit.md#fe-14)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src/App.css](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.css), [docs/DESIGN.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/DESIGN.md)

**Implementation**

- [x] **V06-FE-14.I1** — Replace the audited hardware-source, runtime-code, field-help and empty-log text colors with tokens meeting at least 4.5:1 against their actual regular-text backgrounds. **Trace:** [Audit FE-14](./localmotive-comprehensive-audit.md#fe-14).
- [x] **V06-FE-14.I2** — Review extremely small help/evidence typography and increase practical sizes without losing complete values, treating computed contrast and legibility as related but distinct checks. **Trace:** [Audit FE-14](./localmotive-comprehensive-audit.md#fe-14).
- [x] **V06-FE-14.I3** — Add evidence-specific one-column breakpoints, wrap action-heading rows and constrain 205px/240px grid minima so nested padding cannot force clipped controls in a 320px window. **Trace:** [Audit FE-14](./localmotive-comprehensive-audit.md#fe-14).
- [x] **V06-FE-14.I4** — Redesign narrow bottom navigation and undersized path/plain-link/managed-entry targets to satisfy the repository's 44px mobile target requirement while retaining bottom clearance. **Trace:** [Audit FE-14](./localmotive-comprehensive-audit.md#fe-14).
- [x] **V06-FE-14.I5** — Preserve intentional horizontal scrolling for the inventory table rather than replacing its information structure with unrelated cards, and verify zoomed desktop reflow. **Trace:** [Audit FE-14](./localmotive-comprehensive-audit.md#fe-14).

**Verification**

- [x] **V06-FE-14.V1** — Calculate contrast from the final computed foreground/background pairs for all four affected regular-text cases; assert ratios meet the threshold without rounding an under-threshold value upward. **Trace:** [Audit FE-14](./localmotive-comprehensive-audit.md#fe-14).
- [x] **V06-FE-14.V2** — Inspect 320, 375, 680 and 980px layouts and 200%/400% zoom for clipped evidence controls, horizontal overflow outside intentional inventory scrolling and reachable action controls. **Trace:** [Audit FE-14](./localmotive-comprehensive-audit.md#fe-14).
- [x] **V06-FE-14.V3** — Measure target dimensions for bottom navigation/path actions and run packaged Windows visual/DPI or high-contrast checks, documenting actual remaining limitations. **Trace:** [Audit FE-14](./localmotive-comprehensive-audit.md#fe-14).

**Complete when:** Audited regular text meets the contrast threshold and remains readable in the supported display states. Narrow/zoomed evidence controls fit their containers, and mobile targets satisfy the project's declared size rule.

**Scope / decision note:** The 44px target is the repository's design requirement, not a blanket statement of WCAG 2.2 AA target-size rules. Audit ratios were calculated from CSS; rendered Windows layouts were not tested.

### V06-FE-15

**Tie readiness labels and evidence colors to the proof actually available**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** Unassigned  
**Audit trace:** [Audit FE-15](./localmotive-comprehensive-audit.md#fe-15)  
**Prerequisites:** [V06-FE-01](#v06-fe-01)
**Source touchpoints:** [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx), [src/App.css](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.css)

**Implementation**

- [x] **V06-FE-15.I1** — Replace the loaded-profile VALID label when only shard completeness is known with precise states such as shards complete, path selected, inspection pending or validated. **Trace:** [Audit FE-15](./localmotive-comprehensive-audit.md#fe-15).
- [x] **V06-FE-15.I2** — Stop marking first-run Runtime/Serve ready solely because path strings are nonempty; derive each readiness state from the inspected identity and relevant native validation evidence. **Trace:** [Audit FE-15](./localmotive-comprehensive-audit.md#fe-15).
- [x] **V06-FE-15.I3** — Clear or explicitly mark stale runtime capabilities when committed runtime identity changes, and avoid describing a hard-coded speculation fallback as methods advertised by the executable. **Trace:** [Audit FE-15](./localmotive-comprehensive-audit.md#fe-15).
- [x] **V06-FE-15.I4** — Offer supported speculation methods after inspection or label provisional selections clearly; expose unsupported/unknown capability status near affected settings without treating UI availability as support evidence. **Trace:** [Audit FE-15](./localmotive-comprehensive-audit.md#fe-15).
- [x] **V06-FE-15.I5** — Replace unconditional green evidence headings with status-based tones so Unknown, Blocked, rejected candidates and non-measured classes use the defined neutral/amber/red semantics with words. **Trace:** [Audit FE-15](./localmotive-comprehensive-audit.md#fe-15).

**Verification**

- [x] **V06-FE-15.V1** — Render nonexistent runtime paths, pending/failed inspection and a complete-shard model with an invalid profile; assert no unsupported VALID/ready claim is shown. **Trace:** [Audit FE-15](./localmotive-comprehensive-audit.md#fe-15).
- [x] **V06-FE-15.V2** — Switch executable identity and delay inspection completion; verify old capabilities are not presented as current, and fallback methods are explicitly provisional or unavailable. **Trace:** [Audit FE-15](./localmotive-comprehensive-audit.md#fe-15).
- [x] **V06-FE-15.V3** — Exercise unknown, blocked, rejected, measured and launch-validated evidence classes; assert each has accurate text and the intended semantic tone. **Trace:** [Audit FE-15](./localmotive-comprehensive-audit.md#fe-15).

**Complete when:** Readiness wording names the level of proof available instead of upgrading a path or shard check into successful launch validation. Every evidence status retains meaningful words and uses green only for the design's appropriate established states.

**Scope / decision note:** Rust still decides valid arguments and launch correctness. This task repairs presentation of that evidence; exposing or hiding a control must not broaden the backend's supported capability or trust contract.

### V06-FE-16

**Coordinate UI operation state and separate running-server evidence from editable drafts**

**Status:** Implemented; unit-verified (commit `6da40d8`); packaged overlap/exit scenarios open (V06-G-04) · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit FE-16](./localmotive-comprehensive-audit.md#fe-16)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx), [src/V03EvidencePanel.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx)

**Implementation**

- [x] **V06-FE-16.I1** — Replace the shared busy string and disconnected operation flags with explicit UI operation records and compatibility rules so one completion cannot clear another active operation. **Trace:** [Audit FE-16](./localmotive-comprehensive-audit.md#fe-16).
- [x] **V06-FE-16.I2** — Coordinate legacy and v2 measurement controls, profile Start and other conflicting actions with the real active lifecycle, while preserving independent nonconflicting work where supported. **Trace:** [Audit FE-16](./localmotive-comprehensive-audit.md#fe-16).
- [x] **V06-FE-16.I3** — Make server-status retrieval single-flight or event-driven, reject obsolete poll results after start/stop transitions, and avoid expected failing native polling in browser preview. **Trace:** [Audit FE-16](./localmotive-comprehensive-audit.md#fe-16).
- [x] **V06-FE-16.I4** — Render running strategy/identity from the server snapshot rather than the current editable profile, and label legacy benchmark results with their actual model, runtime, workload and observation time. **Trace:** [Audit FE-16](./localmotive-comprehensive-audit.md#fe-16).
- [x] **V06-FE-16.I5** — Preserve the last bounded server log and exit/failure evidence after a process stops; provide explicit semantics for legacy vs v2 measurement rather than unrelated results on one screen. **Trace:** [Audit FE-16](./localmotive-comprehensive-audit.md#fe-16).

**Verification**

- [ ] **V06-FE-16.V1** — Complete overlapping scan/cloud/start requests in different orders and verify active state persists; exercise legacy/v2 measurement and Start guards through actual UI interactions. **Trace:** [Audit FE-16](./localmotive-comprehensive-audit.md#fe-16).
- [x] **V06-FE-16.V2** — Resolve a delayed status poll after stop/start and assert it cannot replace the newer server snapshot; verify polling remains single-flight under slow native responses. **Trace:** [Audit FE-16](./localmotive-comprehensive-audit.md#fe-16).
- [ ] **V06-FE-16.V3** — Edit a draft while the server runs, then simulate unexpected exit; assert running identity was unchanged by edits and final logs/failure evidence remain visible. **Trace:** [Audit FE-16](./localmotive-comprehensive-audit.md#fe-16).

**Complete when:** Each UI action reflects its own real lifecycle and conflicting controls cannot be reenabled by unrelated completion. Displayed running-server and benchmark identity is independent of drafts, with diagnostics retained after exit.

**Scope / decision note:** IPC-01 owns native startup blocking and MT-05 owns backend operation ownership. This item coordinates their UI presentation without counting those native defects again or claiming unbounded log rendering.

### V06-FE-17

**Connect production workload inputs to tested validation and shared contracts**

**Status:** Implemented + component-verified · **Priority:** Low · **Owner:** sato942  
**Audit trace:** [Audit FE-17](./localmotive-comprehensive-audit.md#fe-17)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src/model.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.ts), [src/model.test.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.test.ts), [src/V03EvidencePanel.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx)

**Implementation**

- [x] **V06-FE-17.I1** — Use the tested defaultWorkload factory in the production evidence panel instead of maintaining a separate literal with the same intended values. **Trace:** [Audit FE-17](./localmotive-comprehensive-audit.md#fe-17).
- [x] **V06-FE-17.I2** — Validate editable workload drafts through the production-used helper or a native validation endpoint before dispatch, surfacing field-level errors instead of relying only on input attributes. **Trace:** [Audit FE-17](./localmotive-comprehensive-audit.md#fe-17).
- [x] **V06-FE-17.I3** — Require finite integer values where Rust uses integer types and expose complete applicable minima/maxima; preserve an editable blank or partial draft until it can be committed safely. **Trace:** [Audit FE-17](./localmotive-comprehensive-audit.md#fe-17).
- [x] **V06-FE-17.I4** — Add representative Rust-to-TypeScript contract fixtures or generated schema checks for persisted/IPC workload and correctness-sensitive profile fields, avoiding unsupported parity assumptions. **Trace:** [Audit FE-17](./localmotive-comprehensive-audit.md#fe-17).
- [x] **V06-FE-17.I5** — Review duplicated fit/default/companion decision paths against the architecture's native-authority rule and document the selected authoritative boundary before consolidating them. **Trace:** [Audit FE-17](./localmotive-comprehensive-audit.md#fe-17).

**Verification**

- [x] **V06-FE-17.V1** — Exercise default factory values through the actual component and IPC payload, then test fractional, negative, blank, nonfinite and over-limit numeric drafts. **Trace:** [Audit FE-17](./localmotive-comprehensive-audit.md#fe-17).
- [x] **V06-FE-17.V2** — Assert an invalid form dispatches no benchmark command, and separately invoke malformed native inputs to prove backend rejection remains in place. **Trace:** [Audit FE-17](./localmotive-comprehensive-audit.md#fe-17).
- [x] **V06-FE-17.V3** — Run representative schema/default round trips and supported older-profile normalization fixtures; change a contract fixture deliberately to establish that the parity check catches drift. **Trace:** [Audit FE-17](./localmotive-comprehensive-audit.md#fe-17).

**Complete when:** The production workload path uses the tested defaults and validation logic, with actionable field errors before dispatch. Contract tests exercise meaningful cross-layer values and malformed cases rather than only asserting helper behavior in isolation.

**Scope / decision note:** Rust already rejects malformed requests; the audited gap concerns UX and misleading assurance from a helper-only test name. Do not infer complete schema parity or invent runtime-specific limits from HTML attributes.

## IPC, cloud and operations tasks

### V06-IPC-01

**Make server startup an observable, cancellable background operation**

**Status:** Implemented; unit-verified (`66a18c3`) · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit IPC-01](./localmotive-comprehensive-audit.md#ipc-01)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/proc.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/proc.rs), [src-tauri/src/health.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/health.rs), [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx), [src/model.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.ts)

**Implementation**

- [x] **V06-IPC-01.I1** — Define an operation ID and explicit starting, running, stopping, failed and cancelled states. Reserve ownership under a short lock and publish correlated progress before launching. **Trace:** [Audit IPC-01](./localmotive-comprehensive-audit.md#ipc-01).
- [x] **V06-IPC-01.I2** — Move blocking filesystem, hashing, subprocess and socket work to a blocking worker. Keep the child handle and cancellation signal reachable by Stop while readiness is pending; do not hold the server mutex across the 600-second health wait. **Trace:** [Audit IPC-01](./localmotive-comprehensive-audit.md#ipc-01).
- [x] **V06-IPC-01.I3** — Commit completion only if the operation ID still owns the slot. On cancellation, failure or window close, terminate and reap the contained process tree before reporting a terminal state or allowing a replacement start. **Trace:** [Audit IPC-01](./localmotive-comprehensive-audit.md#ipc-01).
- [x] **V06-IPC-01.I4** — Classify every synchronous command named in the audit by actual cost, including preview, inspection, health, scanning, GGUF, preflight, legacy benchmark and replay. Move expensive work off both the Tauri main thread and async executor without weakening backend validation. **Trace:** [Audit IPC-01](./localmotive-comprehensive-audit.md#ipc-01).

**Verification**

- [ ] **V06-IPC-01.V1** — Use a benign fixture that binds the expected port but never becomes ready. In the packaged Windows application, verify responsive controls and starting status, then measure Stop-to-process-exit latency against a documented deadline. **Trace:** [Audit IPC-01](./localmotive-comprehensive-audit.md#ipc-01).
- [ ] **V06-IPC-01.V2** — Exercise slow --help, unreadable GGUF, early child exit and window close during startup. Confirm child/listener cleanup, truthful terminal state and a successful subsequent start. **Trace:** [Audit IPC-01](./localmotive-comprehensive-audit.md#ipc-01).
- [x] **V06-IPC-01.V3** — Force a late completion from an older operation after a new request and prove it cannot publish or clear the new operation's state. **Trace:** [Audit IPC-01](./localmotive-comprehensive-audit.md#ipc-01).

**Complete when:** Start, status and Stop remain usable throughout loading; cleanup and latency evidence are attached for the packaged Windows candidate. No expensive command relies on merely adding async around blocking work or retaining a global lock for the operation lifetime.

**Scope / decision note:** The audit established the synchronous/locking path; it did not measure Windows freeze duration. Choose and record cancellation deadlines during implementation, then test them.

### V06-IPC-02

**Enable a narrowly scoped production Content Security Policy**

**Status:** Implemented + packaged-verified · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit IPC-02](./localmotive-comprehensive-audit.md#ipc-02)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/tauri.conf.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tauri.conf.json), [src-tauri/capabilities/default.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/capabilities/default.json), [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx)

**Implementation**

- [x] **V06-IPC-02.I1** — Inventory resources and IPC origins used by a packaged build and derive an explicit production CSP for bundled scripts, styles, fonts and images. Keep development-only exceptions out of production. **Trace:** [Audit IPC-02](./localmotive-comprehensive-audit.md#ipc-02).
- [x] **V06-IPC-02.I2** — Review opener permission and custom-command reachability alongside the policy; retain only origins and operations required by actual application flows. **Trace:** [Audit IPC-02](./localmotive-comprehensive-audit.md#ipc-02).
- [x] **V06-IPC-02.I3** — Document the purpose of each necessary policy exception and keep catalog, model, log and provider text rendered as text rather than HTML. **Trace:** [Audit IPC-02](./localmotive-comprehensive-audit.md#ipc-02).

**Verification**

- [x] **V06-IPC-02.V1** — Exercise every screen, dialog, icon, external URL action and inference WebUI navigation in the packaged application with CSP enabled. **Trace:** [Audit IPC-02](./localmotive-comprehensive-audit.md#ipc-02).
- [x] **V06-IPC-02.V2** — Inject harmless hostile-text fixtures and deliberately disallowed inline/remote script requests; verify text remains inert and scripts are blocked without adding a broad unsafe-eval workaround. **Trace:** [Audit IPC-02](./localmotive-comprehensive-audit.md#ipc-02).

**Complete when:** Production CSP is configured and the packaged acceptance record distinguishes allowed resources from blocked negative controls.

**Scope / decision note:** This is a defense-in-depth task. The audit found no demonstrated XSS chain or proof that arbitrary external websites can invoke the privileged command surface.

### V06-CLD-01

**Bound and validate the complete OAuth callback lifecycle**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit CLD-01](./localmotive-comprehensive-audit.md#cld-01)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/cloud.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/cloud.rs), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs)

**Implementation**

- [x] **V06-CLD-01.I1** — Set request-line, code-length and concurrent-login limits. Enforce one monotonic end-to-end deadline through accept, read, parsing and exchange paths, including a steady byte trickle or successive stray connections. **Trace:** [Audit CLD-01](./localmotive-comprehensive-audit.md#cld-01).
- [x] **V06-CLD-01.I2** — Accept only the intended method and callback path. Use a URL/query parser with percent decoding; reject duplicate/empty codes and malformed encodings, and handle harmless probes without prematurely terminating a valid login. **Trace:** [Audit CLD-01](./localmotive-comprehensive-audit.md#cld-01).
- [x] **V06-CLD-01.I3** — Retain loopback binding, ephemeral ports and S256 PKCE. Add state binding only after confirming the provider's supported contract; do not replace PKCE with state. **Trace:** [Audit CLD-01](./localmotive-comprehensive-audit.md#cld-01).
- [x] **V06-CLD-01.I4** — Show callback-received wording until exchange and Credential Manager storage actually succeed. Report exchange/storage failure in the application without leaving a false connected indication in the browser. **Trace:** [Audit CLD-01](./localmotive-comprehensive-audit.md#cld-01).

**Verification**

- [x] **V06-CLD-01.V1** — Test valid and percent-encoded codes, favicon requests, wrong method/path, duplicate/empty code, malformed encoding and overlong lines against a local callback listener. **Trace:** [Audit CLD-01](./localmotive-comprehensive-audit.md#cld-01).
- [x] **V06-CLD-01.V2** — Test half-open connections, slow byte trickles and repeated stray requests until the overall deadline. Assert bounded memory, bounded completion time and release of listener resources. **Trace:** [Audit CLD-01](./localmotive-comprehensive-audit.md#cld-01).
- [x] **V06-CLD-01.V3** — Stub remote exchange and secure storage to fail independently; prove neither failure reports connected and that a subsequent login can start. Use synthetic credentials. **Trace:** [Audit CLD-01](./localmotive-comprehensive-audit.md#cld-01).

**Complete when:** Malformed local traffic cannot bypass the total resource budget or permanently monopolize login, and connected status requires completed exchange and secure storage.

**Scope / decision note:** The finding concerns local denial of service and misleading status, not demonstrated account takeover. Live provider login was outside the audit's executed scope.

### V06-OPS-01

**Bound per-run server logs and preserve failure diagnostics**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit OPS-01](./localmotive-comprehensive-audit.md#ops-01)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/proc.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/proc.rs), [README.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/README.md)

**Implementation**

- [x] **V06-OPS-01.I1** — Use a unique run ID for each normal and tuning log and safely create files in user-only storage, rejecting unexpected existing links or cross-instance filename collisions. **Trace:** [Audit OPS-01](./localmotive-comprehensive-audit.md#ops-01).
- [x] **V06-OPS-01.I2** — Introduce a bounded sink or rotation with explicit per-run and total retention limits. Keep drain behavior from blocking the child, and preserve the current bounded UI tail. **Trace:** [Audit OPS-01](./localmotive-comprehensive-audit.md#ops-01).
- [x] **V06-OPS-01.I3** — Capture the final failure tail and run identity in structured evidence before retention cleanup. Provide explicit diagnostic export and document location, lifetime and possible runtime-emitted sensitive text. **Trace:** [Audit OPS-01](./localmotive-comprehensive-audit.md#ops-01).

**Verification**

- [x] **V06-OPS-01.V1** — Run a fixture that emits more than the configured quota; assert bounded retained disk use, continued child progress and availability of the newest diagnostic lines. **Trace:** [Audit OPS-01](./localmotive-comprehensive-audit.md#ops-01).
- [x] **V06-OPS-01.V2** — Start a second run and simultaneous application instance; verify logs cannot overwrite another run's failure evidence. Exercise cleanup, failed log creation and unexpected link targets. **Trace:** [Audit OPS-01](./localmotive-comprehensive-audit.md#ops-01).

**Complete when:** Measured log growth stays within the documented policy and a restart preserves attributable failure evidence without mixing runs.

**Scope / decision note:** The audit confirmed unbounded disk writes, not an induced disk-exhaustion incident. The existing 16 KiB/12-line display cap must be preserved.

## Delivery and governance tasks

### V06-GH-01

**Enforce safe premerge checks and default-branch protection**

**Status:** Implemented I1/I2/I4 and gate-verified (commit `fe6f4ea`); I3 (main-branch ruleset) remains an owner action · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit GH-01](./localmotive-comprehensive-audit.md#gh-01)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [.github/workflows/ci.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/ci.yml), [.github/workflow-gates.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflow-gates.json), [scripts/tests/release-gates.test.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/tests/release-gates.test.mjs), [AGENTS.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/AGENTS.md)

**Implementation**

- [x] **V06-GH-01.I1** — Add pull-request checks for the main branch on GitHub-hosted Windows or an ephemeral isolated runner, with stable check names and the applicable type, test, build, dependency and release-policy checks; retain trusted main-push verification. **Trace:** [Audit GH-01](./localmotive-comprehensive-audit.md#gh-01).
- [x] **V06-GH-01.I2** — Keep fork and other untrusted pull-request code off the owner's interactive self-hosted runner. Explicitly separate the PR execution environment from trusted packaging and publication permissions. **Trace:** [Audit GH-01](./localmotive-comprehensive-audit.md#gh-01).
- [ ] **V06-GH-01.I3** — Configure a main-branch ruleset requiring the newly available PR checks, restricting direct and force pushes, and documenting a workable solo-maintainer review and emergency-bypass policy; introduce the trigger before requiring its status. **Trace:** [Audit GH-01](./localmotive-comprehensive-audit.md#gh-01).
- [x] **V06-GH-01.I4** — Update AGENTS.md and workflow comments to describe actual push/PR behavior, runner eligibility and the difference between job dependency gates and repository merge enforcement. **Trace:** [Audit GH-01](./localmotive-comprehensive-audit.md#gh-01).

**Verification**

- [ ] **V06-GH-01.V1** — Use a benign PR to confirm the expected checks run before merge, report stable names and are executed only on the intended isolated runner. **Trace:** [Audit GH-01](./localmotive-comprehensive-audit.md#gh-01).
- [ ] **V06-GH-01.V2** — Introduce a controlled failing check in the test PR; verify merge is blocked until corrected, then confirm the corrected commit obtains the required successful statuses. **Trace:** [Audit GH-01](./localmotive-comprehensive-audit.md#gh-01).
- [ ] **V06-GH-01.V3** — Read back effective branch/ruleset settings with appropriate access, record bypass actors and restrictions, and verify ordinary contributors cannot circumvent required checks through direct pushes. **Trace:** [Audit GH-01](./localmotive-comprehensive-audit.md#gh-01).

**Complete when:** An ordinary PR cannot merge without its required successful checks, and the checks actually execute for that PR. The documented and observed configuration prevents untrusted PR execution on the personal runner and provides an explicit auditable emergency path.

**Scope / decision note:** The audit confirmed push-only CI, an unprotected branch summary and empty accessible rulesets, but privileged protection details returned 403. Close configuration tasks against current effective settings; do not treat that access limitation as a reason to assume hidden controls.

### V06-GH-02

**Bind release jobs and evidence to one immutable source revision**

**Status:** Implemented I1/I2/I4 and gate-verified (commit `fe6f4ea`); I3 (tag protection ruleset) remains an owner action · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit GH-02](./localmotive-comprehensive-audit.md#gh-02)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [.github/workflows/release.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/release.yml), [scripts/verify_candidate_inventory.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_candidate_inventory.mjs), [.github/workflow-gates.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflow-gates.json), [scripts/tests/release-gates.test.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/tests/release-gates.test.mjs), [AGENTS.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/AGENTS.md)

**Implementation**

- [x] **V06-GH-02.I1** — Resolve the requested annotated or lightweight release tag to one full commit SHA once, validate its main ancestry and event relationship, and pass that immutable revision to audit, quality, package and publish checkouts. **Trace:** [Audit GH-02](./localmotive-comprehensive-audit.md#gh-02).
- [x] **V06-GH-02.I2** — Record the resolved source revision with packaged behavior evidence and the producer's candidate inventory; verify these records and exact artifact hashes at publication instead of regenerating inventory from the publish job's current HEAD. **Trace:** [Audit GH-02](./localmotive-comprehensive-audit.md#gh-02).
- [ ] **V06-GH-02.I3** — Protect released version tags against updates and deletion, document one-time tag creation, and require a new prerelease or patch version when source changes after an earlier candidate. **Trace:** [Audit GH-02](./localmotive-comprehensive-audit.md#gh-02).
- [x] **V06-GH-02.I4** — Preserve historical release artifacts and corrective records. Document retries of unchanged source separately from releases containing new source, with an explicit identity mismatch failure before publication. **Trace:** [Audit GH-02](./localmotive-comprehensive-audit.md#gh-02).

**Verification**

- [ ] **V06-GH-02.V1** — Simulate moving the tag between quality, package and publish in isolated repository/workflow fixtures; every job must retain the resolved SHA or fail before promoting mismatched artifacts. **Trace:** [Audit GH-02](./localmotive-comprehensive-audit.md#gh-02).
- [x] **V06-GH-02.V2** — Supply inventories or packaged records with a different source SHA, missing artifact or changed digest; verify publication validation rejects each case without overwriting the original evidence. **Trace:** [Audit GH-02](./localmotive-comprehensive-audit.md#gh-02).
- [ ] **V06-GH-02.V3** — Run a complete candidate flow and compare checkout revisions, inventory, packaged evidence and publication metadata; read back the effective tag-update/deletion protection. **Trace:** [Audit GH-02](./localmotive-comprehensive-audit.md#gh-02).

**Complete when:** Quality results, packaged binaries and original candidate inventory identify the same immutable source and bytes, and controlled promotion/readback fixtures prove the public-release gate rejects any mismatch. Actual v0.6 public readback is recorded separately under V06-G-09. Changing source requires a new version identity, and consumer verification cannot silently rewrite producer evidence.

**Scope / decision note:** Historical v0.4.1 push runs used eleven source SHAs under one tag name. The audit observed no equivalent v0.5.0 tag mutation and did not demonstrate a source/evidence mismatch in that release; the current workflow permits the risk.

### V06-GH-03

**Sequence clean-account verification after its candidate artifacts exist**

**Status:** Implemented I1-I4 and gate-verified (commit `69894e3`); V1-V3 need a live release run · **Priority:** High · **Owner:** sato942  
**Audit trace:** [Audit GH-03](./localmotive-comprehensive-audit.md#gh-03)  
**Prerequisites:** [V06-GH-02](#v06-gh-02)
**Source touchpoints:** [.github/workflows/hardware-qualify.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/hardware-qualify.yml), [.github/workflows/release.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/release.yml), [scripts/sandbox/host-run-lifecycle.ps1](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/sandbox/host-run-lifecycle.ps1), [.github/workflow-gates.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflow-gates.json), [docs/history/TODO-0.5.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/history/TODO-0.5.md)

**Implementation**

- [x] **V06-GH-03.I1** — Make clean-account lifecycle verification consume the already-built candidate installers through CandidateDir as a dependent release job, preserving the producer's source revision, inventory and expected installer hashes. **Trace:** [Audit GH-03](./localmotive-comprehensive-audit.md#gh-03).
- [x] **V06-GH-03.I2** — Remove the independent prepublication wait for public release assets from the one-runner release path; do not occupy the only producer runner with a consumer waiting for artifacts it cannot yet produce. **Trace:** [Audit GH-03](./localmotive-comprehensive-audit.md#gh-03).
- [x] **V06-GH-03.I3** — Define whether lifecycle verification is required before publication. If policy deliberately chooses a non-gating post-release check instead, trigger it only after completed publication and present its untested or failed status explicitly. **Trace:** [Audit GH-03](./localmotive-comprehensive-audit.md#gh-03).
- [x] **V06-GH-03.I4** — Correct the historical v0.5 ledger explanation with a factual additive correction: Sandbox availability succeeded; the demonstrated failure was polling unpublished installers. Update workflow comments and documented scheduling to match the chosen sequence. **Trace:** [Audit GH-03](./localmotive-comprehensive-audit.md#gh-03).

**Verification**

- [ ] **V06-GH-03.V1** — Exercise a fresh-version release in a single-runner configuration; the lifecycle job must start only after candidate artifacts exist, without release-not-found polling blocking the producer. **Trace:** [Audit GH-03](./localmotive-comprehensive-audit.md#gh-03).
- [ ] **V06-GH-03.V2** — Provide missing or mismatched candidate installers and confirm verification fails with an explicit identity error rather than attempting installation or reporting PASS. **Trace:** [Audit GH-03](./localmotive-comprehensive-audit.md#gh-03).
- [ ] **V06-GH-03.V3** — Run the selected lifecycle path against the exact candidate bytes and confirm publication behavior matches the documented required/non-gating policy, including failure and cancellation outcomes. **Trace:** [Audit GH-03](./localmotive-comprehensive-audit.md#gh-03).

**Complete when:** No required lifecycle job waits for a public release whose producer is queued behind it. Each lifecycle result identifies available candidate artifacts and its status is represented accurately in the release decision.

**Scope / decision note:** The recorded v0.5.0 run failed before exercising its installers. Repair this orchestration defect without claiming that the published installers themselves were broken or that merely extending the five-minute wait would solve runner dependency.

### V06-GH-04

**Make installer uninstall and upgrade verdicts verify their claimed outcomes**

**Status:** Implemented I1/I2/I4 (commit `69894e3`); I3 migration fixtures open; V2/V3 need the sandbox window · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit GH-04](./localmotive-comprehensive-audit.md#gh-04)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [scripts/sandbox/run-lifecycle-in-sandbox.ps1](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/sandbox/run-lifecycle-in-sandbox.ps1), [scripts/sandbox/host-run-lifecycle.ps1](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/sandbox/host-run-lifecycle.ps1), [.github/workflows/hardware-qualify.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/hardware-qualify.yml)

**Implementation**

- [x] **V06-GH-04.I1** — Replace the MSI leftover-executable warning-and-continue branch with a failed uninstall verdict when the expected application remains; verify the intended installation scope and registry/product identity instead of relying on a generic executable search. **Trace:** [Audit GH-04](./localmotive-comprehensive-audit.md#gh-04).
- [x] **V06-GH-04.I2** — After an upgrade, verify the installed executable's expected version and digest against the candidate inventory, and retain evidence identifying old and new installer/executable versions. **Trace:** [Audit GH-04](./localmotive-comprehensive-audit.md#gh-04).
- [ ] **V06-GH-04.I3** — Add separate migration scenarios for v0.4.1 profiles and settings and for v0.5.0 SQLite/user-override data. Populate realistic isolated persisted fixtures before upgrade and assert the documented preservation or migration behavior afterward. **Trace:** [Audit GH-04](./localmotive-comprehensive-audit.md#gh-04).
- [x] **V06-GH-04.I4** — Cover appropriate NSIS and MSI fresh-install, upgrade and uninstall paths; replace the permanently hardcoded v0.4.0 baseline with an explicit supported-baseline matrix, and label eight-second process survival as startup smoke rather than full functional verification. **Trace:** [Audit GH-04](./localmotive-comprehensive-audit.md#gh-04).

**Verification**

- [x] **V06-GH-04.V1** — Inject an MSI uninstall outcome that leaves the expected executable or product registration; prove the scenario cannot emit uninstall PASS. **Trace:** [Audit GH-04](./localmotive-comprehensive-audit.md#gh-04).
- [ ] **V06-GH-04.V2** — Simulate an upgrade that keeps the old executable despite a successful installer exit code; verify the version/digest assertion catches it. **Trace:** [Audit GH-04](./localmotive-comprehensive-audit.md#gh-04).
- [ ] **V06-GH-04.V3** — Run clean-account installer scenarios and both version-specific migration fixtures on Windows; compare exact installed identity, retained data, uninstall outcomes and structured per-scenario verdicts. **Trace:** [Audit GH-04](./localmotive-comprehensive-audit.md#gh-04).

**Complete when:** A leftover expected installation or unchanged upgrade executable cannot produce a successful corresponding lifecycle verdict. The recorded installer matrix distinguishes startup, migration, removal and supported baselines, with version-appropriate persistence checks.

**Scope / decision note:** This fixes a confirmed verifier defect, not a demonstrated normal MSI failure. SQLite and local user overrides were introduced in v0.5.0, so v0.4.1 cannot supply a meaningful preexisting override database for that migration test.

### V06-GH-05

**Add packaged acceptance coverage for catalog and SQLite workflows**

**Status:** Implemented and packaged-verified end to end (commit `5222307`) · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit GH-05](./localmotive-comprehensive-audit.md#gh-05)  
**Prerequisites:** [V06-DC-01](#v06-dc-01), [V06-DC-03](#v06-dc-03), [V06-DC-05](#v06-dc-05), [V06-DC-06](#v06-dc-06), [V06-DC-07](#v06-dc-07)
**Source touchpoints:** [scripts/verify_041.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_041.mjs), [.github/workflows/release.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/release.yml), [.github/workflow-gates.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflow-gates.json), [scripts/tests/release-gates.test.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/tests/release-gates.test.mjs), [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx), [src-tauri/src/catalog.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs), [src-tauri/src/catalog_db.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog_db.rs), [AGENTS.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/AGENTS.md)

**Implementation**

- [x] **V06-GH-05.I1** — Add a version-aware packaged verifier for the 0.6 candidate that exercises the HF model catalog and SQLite workflow through actual WebView events and IPC, using an isolated application-data profile and controlled signed catalog fixtures. **Trace:** [Audit GH-05](./localmotive-comprehensive-audit.md#gh-05).
- [x] **V06-GH-05.I2** — Cover first fill, fresh-session restart within cooldown, offline cache availability, valid signed refresh, invalid-signature fallback and corrupt-mirror recovery, while preserving a clear separation between local loading and network refresh. **Trace:** [Audit GH-05](./localmotive-comprehensive-audit.md#gh-05).
- [x] **V06-GH-05.I3** — Exercise rich filter/facet and hardware-fit controls, catalog navigation or pagination where implemented, local user-row persistence/removal, refresh/cooldown feedback and visible bounds/error states through the shipped UI. **Trace:** [Audit GH-05](./localmotive-comprehensive-audit.md#gh-05).
- [x] **V06-GH-05.I4** — Include the verifier in the release gate and bind its report to the immutable source and candidate binary digests. Retain the existing runtime verifier but identify injected presentation checks, real catalog interactions and CPU/runtime checks as distinct evidence. **Trace:** [Audit GH-05](./localmotive-comprehensive-audit.md#gh-05).

**Verification**

- [x] **V06-GH-05.V1** — Demonstrate that the new packaged scenarios fail against the specific catalog restart, corruption and override defects they are intended to detect, then rerun them on their corrected implementation. **Trace:** [Audit GH-05](./localmotive-comprehensive-audit.md#gh-05).
- [x] **V06-GH-05.V2** — Run the packaged catalog matrix from an empty profile and a persisted prior-version profile; verify both UI-visible results and actual backend/disk persistence across application restarts. **Trace:** [Audit GH-05](./localmotive-comprehensive-audit.md#gh-05).
- [x] **V06-GH-05.V3** — Corrupt a signature and mirror, reject an input and delay a refresh; assert honest fallback/error/cooldown output without replacing real orchestration with private React-state injection. **Trace:** [Audit GH-05](./localmotive-comprehensive-audit.md#gh-05).

**Complete when:** The candidate release contains successful artifact-bound packaged evidence for the current catalog/SQLite workflows, including restart and failure paths. Evidence clearly separates HF catalog behavior from the prior verifier's runtime-catalog fixtures.

**Scope / decision note:** The v0.5.0 record genuinely contains 29 checks bound to its binary, despite its 0.4.1 filename. Their scope does not certify the new catalog flows; this task adds that missing integration evidence and may start with failing scenarios before dependencies close.

### V06-GH-06

**Preserve structured lifecycle evidence for failures and timeouts**

**Status:** Implemented I1-I4 and gate-verified (commit `69894e3`); V1-V3 need the sandbox window · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit GH-06](./localmotive-comprehensive-audit.md#gh-06)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [scripts/sandbox/host-run-lifecycle.ps1](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/sandbox/host-run-lifecycle.ps1), [scripts/sandbox/run-lifecycle-in-sandbox.ps1](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/sandbox/run-lifecycle-in-sandbox.ps1), [.github/workflows/hardware-qualify.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/hardware-qualify.yml)

**Implementation**

- [x] **V06-GH-06.I1** — Move collection of Sandbox result JSON, lifecycle logs and relevant host diagnostics into a finally-style path that runs before propagating a failure; preserve the original process or scenario error. **Trace:** [Audit GH-06](./localmotive-comprehensive-audit.md#gh-06).
- [x] **V06-GH-06.I2** — Write a structured FAIL, TIMEOUT or cancellation/missing-result record when Sandbox never produces usable output, including release version, source revision, candidate digests, timing and the failing stage. **Trace:** [Audit GH-06](./localmotive-comprehensive-audit.md#gh-06).
- [x] **V06-GH-06.I3** — Make the workflow upload the expected evidence on every terminal outcome and surface a missing required evidence file instead of treating an ignored empty upload as successful verification. **Trace:** [Audit GH-06](./localmotive-comprehensive-audit.md#gh-06).
- [x] **V06-GH-06.I4** — Update job summaries to distinguish scenario status, artifact-upload status and actual artifact existence; report missing evidence as missing and never infer a lifecycle PASS from a green upload step. **Trace:** [Audit GH-06](./localmotive-comprehensive-audit.md#gh-06).

**Verification**

- [ ] **V06-GH-06.V1** — Inject a Sandbox FAIL result and verify the job remains failed while its exact result JSON and diagnostic log are retained in the uploaded evidence. **Trace:** [Audit GH-06](./localmotive-comprehensive-audit.md#gh-06).
- [ ] **V06-GH-06.V2** — Exercise timeout, missing or malformed result, early installer-download failure and cancellation paths; each must leave a bounded structured outcome with the correct stage and candidate identity. **Trace:** [Audit GH-06](./localmotive-comprehensive-audit.md#gh-06).
- [ ] **V06-GH-06.V3** — Inspect the completed workflow's artifact collection, not just its upload-step conclusion, and confirm the summary accurately reports both verification outcome and evidence availability. **Trace:** [Audit GH-06](./localmotive-comprehensive-audit.md#gh-06).

**Complete when:** Every terminal lifecycle outcome produces retrievable structured evidence or an explicit evidence-collection failure. Failure propagation stays nonzero and upload success cannot be mistaken for successful installer verification.

**Scope / decision note:** The observed v0.5.0 hardware run uploaded a host-attestation artifact but no Sandbox lifecycle artifact even though its upload step succeeded. Preserve useful failure evidence without introducing secrets or broad owner-machine data into public logs.

### V06-GH-07

**Reconcile public support claims with version-specific qualification evidence**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** Unassigned  
**Audit trace:** [Audit GH-07](./localmotive-comprehensive-audit.md#gh-07)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [README.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/README.md), [AGENTS.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/AGENTS.md), [release-evidence/0.4.1/v0.4.0-corrective-note.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/release-evidence/0.4.1/v0.4.0-corrective-note.md), [.github/workflows/hardware-qualify.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/hardware-qualify.yml), [.github/workflows/ci.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/ci.yml), [docs/history/TODO-0.5.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/history/TODO-0.5.md)

**Implementation**

- [x] **V06-GH-07.I1** — Prepare the factual v0.4.0 public release correction from the existing corrective note: relabel the three L2/PARTIAL configurations as direct-runtime evidence and explain that packaged product qualification was not established; preserve its published binaries. **Trace:** [Audit GH-07](./localmotive-comprehensive-audit.md#gh-07).
- [x] **V06-GH-07.I2** — Publish a compact version-to-host/backend/lifecycle evidence matrix with source/artifact references, distinguishing CPU packaged checks from CUDA/Vulkan support and from clean-account install, upgrade and uninstall results. **Trace:** [Audit GH-07](./localmotive-comprehensive-audit.md#gh-07).
- [x] **V06-GH-07.I3** — Rewrite the README support section so each claimed status is conditional on evidence for that exact version and configuration; keep the actual v0.5 lifecycle result untested by that failed run rather than implying an installer failure. **Trace:** [Audit GH-07](./localmotive-comprehensive-audit.md#gh-07).
- [x] **V06-GH-07.I4** — Correct stale runner schedule/hosting and push/PR CI descriptions to the currently verified configuration. Keep historical events attributable through dated corrections instead of rewriting old failure outcomes as later successes. **Trace:** [Audit GH-07](./localmotive-comprehensive-audit.md#gh-07).

**Verification**

- [x] **V06-GH-07.V1** — Compare every matrix support or PASS label against the linked version, source, binary and scenario record; fail documentation review when host presence or L2 evidence substitutes for product qualification. **Trace:** [Audit GH-07](./localmotive-comprehensive-audit.md#gh-07).
- [x] **V06-GH-07.V2** — Read back the affected public v0.4.0 release after an authorized correction and verify its former Supported wording no longer creates the false product-support claim. **Trace:** [Audit GH-07](./localmotive-comprehensive-audit.md#gh-07).
- [x] **V06-GH-07.V3** — Check README, AGENTS and active workflow descriptions together for runner/trigger agreement and verify v0.5 CPU evidence remains distinct from missing accelerator and lifecycle qualification. **Trace:** [Audit GH-07](./localmotive-comprehensive-audit.md#gh-07).

**Complete when:** Users reading the affected historical release or current README see the same accurate, version-specific qualification limits. Public corrections preserve artifact identity and make missing or failed evidence explicit without overstating support.

**Scope / decision note:** The audit made no release edits. Public correction is a separate external action to perform only within the authorization available when implementing this task; preparation and review of exact correction text can proceed independently.

### V06-GH-08

**Strengthen release provenance, toolchain identity and evidence retention**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** Unassigned  
**Audit trace:** [Audit GH-08](./localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](./localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps)  
**Prerequisites:** [V06-GH-02](#v06-gh-02)
**Source touchpoints:** [.github/workflows/release.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/release.yml), [.github/workflows/ci.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/ci.yml), [.github/workflows/catalog.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/catalog.yml), [.github/workflow-actions.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflow-actions.json), [src-tauri/Cargo.lock](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/Cargo.lock), [package-lock.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/package-lock.json), [AGENTS.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/AGENTS.md), [README.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/README.md)

**Implementation**

- [x] **V06-GH-08.I1** — Pin or explicitly record the exact Rust and Node toolchains and relevant Windows build-image/tool versions so the immutable action commit is not confused with a reproducible compiler or runner environment. **Trace:** [Audit GH-08](./localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](./localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
- [x] **V06-GH-08.I2** — Produce a target-specific artifact-bound SBOM and third-party notices/license inventory from resolved package metadata and license texts, not Cargo.lock alone. Define the license-review policy and record unresolved metadata. Where feasible, add signed build provenance tied to the source revision and exact candidate hashes, stating what it attests and how to verify it. **Trace:** [Audit GH-08](./localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](./localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
- [x] **V06-GH-08.I3** — Separate trusted release build/publication credentials and environment from routine interactive owner state, preserving least-privilege workflow tokens and avoiding any untrusted PR execution on the personal runner. **Trace:** [Audit GH-08](./localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](./localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
- [x] **V06-GH-08.I4** — Define retention and recovery for candidate inventories, packaged/lifecycle evidence, readback artifacts and supporting logs beyond current ephemeral windows; document signing-key rotation/recovery without storing keys in the repository. **Trace:** [Audit GH-08](./localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](./localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
- [x] **V06-GH-08.I5** — Retain honest unsigned-installer disclosure and distinguish application Authenticode, Git tag identity, catalog Ed25519 signatures, build provenance and checksums as different controls. **Trace:** [Audit GH-08](./localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](./localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).

**Verification**

- [x] **V06-GH-08.V1** — Reproduce the recorded toolchain selection from a clean eligible build environment and compare source, compiler, dependency lockfiles and environment metadata in the retained record. **Trace:** [Audit GH-08](./localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](./localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
- [x] **V06-GH-08.V2** — Verify the SBOM, license/notice inventory and any provenance against the candidate source, resolved Windows dependency graph and artifact digests; identify missing license information explicitly and confirm substituted artifacts or identities fail validation. **Trace:** [Audit GH-08](./localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](./localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
- [x] **V06-GH-08.V3** — Exercise evidence retrieval/recovery under the chosen retention policy and inspect effective job permissions and environment separation without exposing owner credentials. **Trace:** [Audit GH-08](./localmotive-comprehensive-audit.md#gh-08); [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust); [Supply-chain controls and gaps](./localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).

**Complete when:** Exact candidate build inputs, target dependency/license inventory and retained evidence are inspectable, and controlled provenance/readback checks reject substitutions. Assurance and unresolved metadata are stated precisely; actual v0.6 publication/readback remains V06-G-09. The accepted unsigned distribution policy remains visible and no catalog signature is represented as installer authentication.

**Scope / decision note:** Authenticode was intentionally deferred and is not a new mandatory ship gate here. Checksums alone do not establish independent publisher authenticity, and toolchain recording is not proof of a byte-for-byte reproducible Windows build.

### V06-GH-09

**Establish lightweight security reporting and maintenance ownership**

**Status:** Implemented + unit-verified · **Priority:** Low · **Owner:** sato942  
**Audit trace:** [Audit GH-09](./localmotive-comprehensive-audit.md#gh-09)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [AGENTS.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/AGENTS.md), [README.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/README.md), `SECURITY.md` (proposed; absent at audited SHA), `CONTRIBUTING.md` (proposed; absent at audited SHA), `CODEOWNERS` (proposed; absent at audited SHA), `CODE_OF_CONDUCT` (proposed; absent at audited SHA), [.github/workflows/ci.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/ci.yml), [.github/workflows/catalog.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/catalog.yml)

**Implementation**

- [x] **V06-GH-09.I1** — Add a concise security reporting/contact policy and contributor setup/support guidance that explains supported scope, safe reproduction details, response ownership and how to avoid posting credentials or private diagnostics. **Trace:** [Audit GH-09](./localmotive-comprehensive-audit.md#gh-09).
- [x] **V06-GH-09.I2** — Introduce lightweight issue/PR templates and ownership guidance suitable for a solo-maintainer project; record an accountable owner, priority, acceptance criteria and evidence location for each unresolved audit item. **Trace:** [Audit GH-09](./localmotive-comprehensive-audit.md#gh-09).
- [x] **V06-GH-09.I3** — Configure automatic dependency-update proposals and scheduled dependency audits in a safe hosted or isolated environment; document how vulnerability, maintenance and unsoundness warnings are triaged separately and how target relevance is established. **Trace:** [Audit GH-09](./localmotive-comprehensive-audit.md#gh-09).
- [x] **V06-GH-09.I4** — Inspect accessible current repository security/alert settings with appropriate authorization and record their actual status instead of inferring disabled services from absent files. Define any useful SAST or license-policy follow-up without duplicating the release SBOM work. **Trace:** [Audit GH-09](./localmotive-comprehensive-audit.md#gh-09).
- [x] **V06-GH-09.I5** — Document backup and recovery responsibilities for the build runner, release evidence and signing assets; keep private recovery material outside public source and avoid process requirements that the maintainer cannot realistically sustain. **Trace:** [Audit GH-09](./localmotive-comprehensive-audit.md#gh-09).

**Verification**

- [x] **V06-GH-09.V1** — Walk through a synthetic security report and contributor issue to verify that contact, ownership, required reproduction information and next-step expectations are clear without disclosing secrets. **Trace:** [Audit GH-09](./localmotive-comprehensive-audit.md#gh-09).
- [x] **V06-GH-09.V2** — Confirm a controlled dependency-update proposal and scheduled audit run produce actionable results, preserve nonzero blocking failures and route informational warnings to an owned follow-up. **Trace:** [Audit GH-09](./localmotive-comprehensive-audit.md#gh-09).
- [x] **V06-GH-09.V3** — Review effective settings and recovery instructions with the responsible maintainer; verify recorded task ownership and closure evidence can be located from the project's documentation. **Trace:** [Audit GH-09](./localmotive-comprehensive-audit.md#gh-09).

**Complete when:** Security reports and maintenance findings have an explicit reachable owner and a documented path from report to verified closure. Dependency monitoring executes on schedule and reports its true target scope and advisory categories.

**Scope / decision note:** Absent SECURITY/automation files were confirmed, but GitHub secret-scanning and Dependabot settings or alerts were not accessible in the audit. A young repository, one contributor and an empty issue tracker are context rather than independent defects.

### V06-GH-10

**Derive host attestation statuses from detected hardware**

**Status:** Implemented I1-I4; V1/V2 verified by fixtures (commit `69894e3`); V3 needs the next hardware window · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit GH-10](./localmotive-comprehensive-audit.md#gh-10)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [.github/workflows/hardware-qualify.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/hardware-qualify.yml)

**Implementation**

- [x] **V06-GH-10.I1** — Replace unconditional HOST_MATCH rows with per-row results derived from the detected CPU/GPU identity and the qualification job's stated expected host; represent MATCH, NO_MATCH and UNKNOWN explicitly. **Trace:** [Audit GH-10](./localmotive-comprehensive-audit.md#gh-10).
- [x] **V06-GH-10.I2** — Treat a definite mismatch as a failed intended host-proof job and preserve UNKNOWN when identification is incomplete; do not promote a warning about mismatched hardware into a successful match record. **Trace:** [Audit GH-10](./localmotive-comprehensive-audit.md#gh-10).
- [x] **V06-GH-10.I3** — Include actual source revision, runtime identity and driver/hardware observations needed to interpret the host evidence, keeping observed values separate from expected runner labels or a hardcoded runner name. **Trace:** [Audit GH-10](./localmotive-comprehensive-audit.md#gh-10).
- [x] **V06-GH-10.I4** — Retain and strengthen the explicit policy that host presence does not establish packaged L4 product qualification. Report CPU, CUDA and Vulkan qualification only through the separate matching packaged scenario evidence, not through static host rows. **Trace:** [Audit GH-10](./localmotive-comprehensive-audit.md#gh-10).

**Verification**

- [x] **V06-GH-10.V1** — Feed the host-record generator matching CPU/GPU observations and confirm only the corresponding expected host rows are marked matched, with the actual observations retained. **Trace:** [Audit GH-10](./localmotive-comprehensive-audit.md#gh-10).
- [x] **V06-GH-10.V2** — Exercise mismatched CPU, mismatched GPU, missing/ambiguous GPU and stale-runner-label fixtures; verify none emits an unsupported HOST_MATCH and the intended host-proof outcome reflects the mismatch or uncertainty. **Trace:** [Audit GH-10](./localmotive-comprehensive-audit.md#gh-10).
- [ ] **V06-GH-10.V3** — Run on the intended Windows host and compare emitted identity fields to directly observed hardware/driver/source values; verify no host-only record claims successful CUDA/Vulkan inference or complete L4 support. **Trace:** [Audit GH-10](./localmotive-comprehensive-audit.md#gh-10).

**Complete when:** Each emitted host status is justified by the recorded observation and mismatched or unknown hardware cannot silently receive HOST_MATCH. Host proof and packaged runtime/product qualification remain separate, accurately labeled evidence classes.

**Scope / decision note:** The audit found hardcoded statuses that would become false after host or label drift; it did not establish that the observed Zen5/RTX 5090 runner was mismatched. Detection should preserve uncertainty instead of inventing exact physical or product-support identity.

## Quality and documentation tasks

### V06-QD-01

**Advance the catalog freshness window from an explicit UTC reference date**

**Status:** Implemented + unit-verified · **Priority:** Medium · **Owner:** Unassigned  
**Audit trace:** [Audit QD-01](./localmotive-comprehensive-audit.md#qd-01)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [scripts/build_catalog.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/build_catalog.mjs), [catalog/providers.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/catalog/providers.json), [catalog/README.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/catalog/README.md), [README.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/README.md), [scripts/tests/release-gates.test.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/tests/release-gates.test.mjs), [docs/history/TODO-0.5.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/history/TODO-0.5.md)

**Implementation**

- [x] **V06-QD-01.I1** — Replace the literal September 10, 2026 cutoff reference with the current UTC date by default, and decide whether to expose an explicit --as-of date for reproducible catalog snapshots. **Trace:** [Audit QD-01](./localmotive-comprehensive-audit.md#qd-01).
- [x] **V06-QD-01.I2** — Compute the advertised update date, effective reference date and cutoff date from the same validated clock input; retain the effective reference and cutoff in publish evidence. **Trace:** [Audit QD-01](./localmotive-comprehensive-audit.md#qd-01).
- [x] **V06-QD-01.I3** — Extract date selection and discovery/deduplication decisions into importable helpers so deterministic tests exercise production behavior without live Hub requests or replacing the signed catalog. **Trace:** [Audit QD-01](./localmotive-comprehensive-audit.md#qd-01).
- [x] **V06-QD-01.I4** — Document the actual per-author cap, the meaning of --full and the first-page limit; state whether pagination is intentionally omitted instead of implying an exhaustive allowlist scan. **Trace:** [Audit QD-01](./localmotive-comprehensive-audit.md#qd-01).

**Verification**

- [x] **V06-QD-01.V1** — Reproduce the audit fixture with a December 10, 2026 clock and a July 1 model; assert the model is excluded by the 90-day policy and the emitted dates agree. **Trace:** [Audit QD-01](./localmotive-comprehensive-audit.md#qd-01).
- [x] **V06-QD-01.V2** — Cover exact cutoff boundaries, future timestamps, malformed lastModified, leap days and repeated builds with the same explicit reference date. **Trace:** [Audit QD-01](./localmotive-comprehensive-audit.md#qd-01).
- [x] **V06-QD-01.V3** — Verify bounded discovery and duplicate selection with controlled metadata, then run the catalog validator on a generated fixture while confirming repository catalog/signature bytes remain unchanged. **Trace:** [Audit QD-01](./localmotive-comprehensive-audit.md#qd-01).

**Complete when:** A later execution advances the default cutoff, while an explicitly selected reference date produces repeatable date decisions. Catalog policy text and publish evidence disclose the actual freshness window and discovery bounds.

**Scope / decision note:** The audit demonstrated a future correctness defect; it did not establish that the September 10 artifact violated its own window. An intentional curated cap remains a valid design choice.

### V06-QD-02

**Test catalog and frontend orchestration through observable application contracts**

**Status:** Implemented I1/I2/I3/I5, V1-V3 verified (commits `b722f4d`, `5222307`); I4 and V4 partially open · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit QD-02](./localmotive-comprehensive-audit.md#qd-02)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [vite.config.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/vite.config.ts), [package.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/package.json), [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx), [src/V03EvidencePanel.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx), [src/model.test.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.test.ts), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/catalog_db.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog_db.rs), [scripts/verify_041.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_041.mjs), [scripts/tests/release-gates.test.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/tests/release-gates.test.mjs)

**Implementation**

- [x] **V06-QD-02.I1** — Add a DOM/component test environment scoped to application tests, mocking IPC only at its boundary and preserving the exclusion of research and third-party tests. **Trace:** [Audit QD-02](./localmotive-comprehensive-audit.md#qd-02).
- [x] **V06-QD-02.I2** — Add real temporary-database orchestration fixtures covering fetch, mirror publication, restart, cooldown, offline fallback, corrupt migration and download authorization, rather than calling only successful storage helpers. **Trace:** [Audit QD-02](./localmotive-comprehensive-audit.md#qd-02).
- [x] **V06-QD-02.I3** — Exercise curated/local ID and filename collisions, atomic file replacement and preservation of local entries; write failing regressions before the corresponding catalog fixes are closed. **Trace:** [Audit QD-02](./localmotive-comprehensive-audit.md#qd-02).
- [ ] **V06-QD-02.I4** — Cover delayed/rejected IPC, filter reapplication, hardware updates, stale responses, malformed saved profiles, denied browser storage and accessible catalog/profile interactions. **Trace:** [Audit QD-02](./localmotive-comprehensive-audit.md#qd-02).
- [x] **V06-QD-02.I5** — Extend version-aware packaged HF catalog acceptance to first run, restart, persistence, fallback, filters and actual serialization; distinguish these scenarios from the retained runtime-catalog checks. **Trace:** [Audit QD-02](./localmotive-comprehensive-audit.md#qd-02).

**Verification**

- [x] **V06-QD-02.V1** — Demonstrate that regression cases fail for the audited boundary defects and pass only after their owning implementation tasks resolve them. **Trace:** [Audit QD-02](./localmotive-comprehensive-audit.md#qd-02).
- [x] **V06-QD-02.V2** — Run the component suite and backend orchestration tests with delayed failures, corrupt databases and conflicting rows, retaining exact commands and results. **Trace:** [Audit QD-02](./localmotive-comprehensive-audit.md#qd-02).
- [x] **V06-QD-02.V3** — Run the packaged Windows scenarios with an isolated profile and retain source/artifact-bound evidence; record unexecuted target-environment checks explicitly. **Trace:** [Audit QD-02](./localmotive-comprehensive-audit.md#qd-02).
- [ ] **V06-QD-02.V4** — Confirm keyboard navigation, labels, focus changes and status announcements on representative catalog/profile flows. **Trace:** [Audit QD-02](./localmotive-comprehensive-audit.md#qd-02).

**Complete when:** Release acceptance exercises the real HF catalog orchestration, including failure and restart paths, instead of relying on source-name assertions. Each confirmed boundary defect has an observable regression that cannot pass when its required behavior is removed.

**Scope / decision note:** Test work can begin before the related fixes; closure requires their observable contracts to pass. Counts and coverage percentages alone do not establish Windows, hardware or accessibility qualification.

### V06-QD-03

**Replace private React state injection with stable presentation and packaged-flow tests**

**Status:** Implemented I1-I5, V1/V2 verified (commits `b722f4d`, `e44019c`); V3/V4 ride the next packaged verify · **Priority:** Medium · **Owner:** sato942  
**Audit trace:** [Audit QD-03](./localmotive-comprehensive-audit.md#qd-03)  
**Prerequisites:** [V06-QD-02](#v06-qd-02)
**Source touchpoints:** [scripts/verify_041.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_041.mjs), [scripts/tests/release-gates.test.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/tests/release-gates.test.mjs), [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx)

**Implementation**

- [x] **V06-QD-03.I1** — Move synthetic loading, empty, error and rate-limit presentation scenarios into component tests using public React/DOM interfaces and a stable mocked backend boundary. **Trace:** [Audit QD-03](./localmotive-comprehensive-audit.md#qd-03).
- [x] **V06-QD-03.I2** — Remove the packaged verifier's __reactFiber$, memoizedState, hook-shape discovery and queue.dispatch dependencies; replace setScenarioAndRefresh with actual user controls or explicitly labeled synthetic presentation setup. **Trace:** [Audit QD-03](./localmotive-comprehensive-audit.md#qd-03).
- [x] **V06-QD-03.I3** — Keep genuine IPC, installation, tamper, health and restart checks intact, and describe separately what each check proves so synthetic rendering cannot be mistaken for a real request lifecycle. **Trace:** [Audit QD-03](./localmotive-comprehensive-audit.md#qd-03).
- [x] **V06-QD-03.I4** — Replace the fixed 250-millisecond cancellation trigger with an observed active-operation/progress condition where available; retain bounded timeouts and useful diagnostics for missing transitions. **Trace:** [Audit QD-03](./localmotive-comprehensive-audit.md#qd-03).
- [x] **V06-QD-03.I5** — Replace release-gate assertions that require private implementation strings with assertions about the verifier's observable outcomes and complete evidence records. **Trace:** [Audit QD-03](./localmotive-comprehensive-audit.md#qd-03).

**Verification**

- [x] **V06-QD-03.V1** — Change hook ordering or introduce similarly shaped component state and verify the public-interface tests remain valid without adapting private-memory selectors. **Trace:** [Audit QD-03](./localmotive-comprehensive-audit.md#qd-03).
- [x] **V06-QD-03.V2** — Inject delayed and failed backend responses through the test boundary; assert Refresh actually starts retrieval and reaches the expected terminal UI state. **Trace:** [Audit QD-03](./localmotive-comprehensive-audit.md#qd-03).
- [ ] **V06-QD-03.V3** — Exercise cancellation under fast and slow fixture progress, confirming it waits for an active operation and records completion or a bounded diagnostic failure. **Trace:** [Audit QD-03](./localmotive-comprehensive-audit.md#qd-03).
- [ ] **V06-QD-03.V4** — Run the preserved packaged IPC/install/tamper/health checks and inspect the evidence labels for a clear distinction between synthetic presentation and real execution. **Trace:** [Audit QD-03](./localmotive-comprehensive-audit.md#qd-03).

**Complete when:** No packaged acceptance path traverses private React memory or requires hook-position assumptions. Presentation and lifecycle evidence retain their useful coverage while accurately identifying the boundary exercised.

**Scope / decision note:** The audit did not invalidate existing genuine backend checks. QD-02 supplies the component-test boundary needed to relocate synthetic scenarios safely; avoid discarding coverage while changing the harness.

### V06-QD-04

**Reconcile current product, persistence and qualification documentation**

**Status:** Implemented + unit-verified · **Priority:** Low · **Owner:** sato942  
**Audit trace:** [Audit QD-04](./localmotive-comprehensive-audit.md#qd-04)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [README.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/README.md), [AGENTS.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/AGENTS.md), [CHANGELOG.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/CHANGELOG.md), [docs/PRODUCT.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/PRODUCT.md), [docs/RUNTIME_MANAGER.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/RUNTIME_MANAGER.md), [docs/qualification-tests.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/qualification-tests.md), [docs/history/TODO-0.5.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/history/TODO-0.5.md), [catalog/README.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/catalog/README.md), [.github/workflows/ci.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/ci.yml), [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs), [src-tauri/src/catalog_db.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog_db.rs)

**Implementation**

- [x] **V06-QD-04.I1** — Align active product/runtime documents with the actual desktop platform, local SQLite implementation, current version and standing unsigned-release policy; distinguish reusable runtime policy from version-specific qualification evidence. **Trace:** [Audit QD-04](./localmotive-comprehensive-audit.md#qd-04).
- [x] **V06-QD-04.I2** — Add a storage matrix covering settings, signed JSON cache, SQLite/user rows, refresh stamps, managed runtimes and evidence stores, with accurate reset, export and backup behavior and any temp-directory fallback. **Trace:** [Audit QD-04](./localmotive-comprehensive-audit.md#qd-04).
- [x] **V06-QD-04.I3** — Mark the September 6 qualification guide as a frozen snapshot or archive it with a current evidence index; correct current runnable version commands and the history path without rewriting past results. **Trace:** [Audit QD-04](./localmotive-comprehensive-audit.md#qd-04).
- [x] **V06-QD-04.I4** — Reconcile contributor CI claims with actual triggers, document builder fail-the-whole-build behavior and --allow-empty, and record the schema-v1 compatibility decision plus old-client fallback expectations. **Trace:** [Audit QD-04](./localmotive-comprehensive-audit.md#qd-04).
- [x] **V06-QD-04.I5** — Update module ownership/data-flow maps and present superseded v0.5 tracker decisions as dated history or beneath an authoritative current-status table. **Trace:** [Audit QD-04](./localmotive-comprehensive-audit.md#qd-04).

**Verification**

- [x] **V06-QD-04.V1** — Review every discrepancy in the audit's QD-04 table against source, current workflow definitions and version-bound evidence; retain an explicit resolution for each row. **Trace:** [Audit QD-04](./localmotive-comprehensive-audit.md#qd-04).
- [x] **V06-QD-04.V2** — Check documented commands and local paths, and verify storage/recovery descriptions against implemented behavior without presenting planned features as available. **Trace:** [Audit QD-04](./localmotive-comprehensive-audit.md#qd-04).
- [x] **V06-QD-04.V3** — Confirm historical counts, blockers and closeout evidence remain intact and distinguishable from current status; rerun documentation/branding gates that apply. **Trace:** [Audit QD-04](./localmotive-comprehensive-audit.md#qd-04).

**Complete when:** Active documentation presents one consistent release, platform, persistence and verification contract. Readers can identify the version and evidence behind a support statement and can distinguish superseded history from current instructions.

**Scope / decision note:** Documentation may describe unresolved limitations honestly. It must not imply that related implementation or qualification tasks are complete; actual premerge enforcement and public support corrections remain owned by their GitHub findings.

### V06-QD-05

**Declare compatible build tooling and record reproducible toolchain choices**

**Status:** Implemented + unit-verified · **Priority:** Low · **Owner:** sato942  
**Audit trace:** [Audit QD-05](./localmotive-comprehensive-audit.md#qd-05)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [README.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/README.md), [package.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/package.json), [package-lock.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/package-lock.json), [src-tauri/Cargo.toml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/Cargo.toml), `rust-toolchain.toml` (proposed; absent at audited SHA), [.github/workflows/ci.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/ci.yml)

**Implementation**

- [x] **V06-QD-05.I1** — Declare a Node engine range compatible with the actual locked bundler/test toolchain and correct the broad Node.js 20 setup instruction; the audited Vite version requires Node 20.19 or an allowed later line. **Trace:** [Audit QD-05](./localmotive-comprehensive-audit.md#qd-05).
- [x] **V06-QD-05.I2** — Declare the selected package-manager version and project Node/Rust toolchain configuration, deciding explicitly which versions are exact pins and which are supported minimums. **Trace:** [Audit QD-05](./localmotive-comprehensive-audit.md#qd-05).
- [x] **V06-QD-05.I3** — Align CI toolchain selection with those project declarations while retaining immutable action references, npm ci and Cargo --locked dependency resolution. **Trace:** [Audit QD-05](./localmotive-comprehensive-audit.md#qd-05).
- [x] **V06-QD-05.I4** — Document how toolchain updates are proposed and verified, and record the actual Node, npm and Rust versions used in release evidence so floating environment changes are observable. **Trace:** [Audit QD-05](./localmotive-comprehensive-audit.md#qd-05).

**Verification**

- [x] **V06-QD-05.V1** — Run documented setup and the required check suite in a clean environment with the declared supported tooling, retaining exact version output and command results. **Trace:** [Audit QD-05](./localmotive-comprehensive-audit.md#qd-05).
- [x] **V06-QD-05.V2** — Check an older unsupported Node configuration receives a clear engine/setup constraint instead of being recommended as supported by the documentation. **Trace:** [Audit QD-05](./localmotive-comprehensive-audit.md#qd-05).
- [x] **V06-QD-05.V3** — Verify CI, local configuration and release evidence agree on the chosen toolchain; confirm dependency installation preserves the committed lockfile selections. **Trace:** [Audit QD-05](./localmotive-comprehensive-audit.md#qd-05).
- [x] **V06-QD-05.V4** — Review an example toolchain update against the documented procedure, including compatibility checks for the current Vite/Vitest and Rust dependencies. **Trace:** [Audit QD-05](./localmotive-comprehensive-audit.md#qd-05).

**Complete when:** A contributor following the setup instructions selects tooling compatible with the committed dependencies. Every candidate evidence record identifies its actual compiler and package-manager versions and the project documents how those versions change.

**Scope / decision note:** Exact compiler pinning is an engineering decision, not proof of bit-for-bit reproducibility or a security fix. Re-evaluate requirements if dependencies change; do not preserve an obsolete minimum merely because it appeared in this audit.

### V06-QD-06

**Source-pin the imported llama-server reference and repair its links**

**Status:** Implemented + unit-verified · **Priority:** Low · **Owner:** sato942  
**Audit trace:** [Audit QD-06](./localmotive-comprehensive-audit.md#qd-06)  
**Prerequisites:** None; can begin independently.
**Source touchpoints:** [docs/LLAMA-SERVER-README.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/LLAMA-SERVER-README.md), [docs/OPTION_MAP.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/OPTION_MAP.md)

**Implementation**

- [x] **V06-QD-06.I1** — Identify the imported document's exact upstream repository path and commit, record its content identity and import date, and disclose any provenance that cannot be established instead of guessing it. **Trace:** [Audit QD-06](./localmotive-comprehensive-audit.md#qd-06).
- [x] **V06-QD-06.I2** — Resolve the ten audited missing relative targets to existing local resources or immutable URLs in the identified upstream tree, including multimodal, function-calling, examples, development notes and UI constants. **Trace:** [Audit QD-06](./localmotive-comprehensive-audit.md#qd-06).
- [x] **V06-QD-06.I3** — Keep the application option-grouping document separate from the imported generated server reference; state that the selected executable's --help remains authoritative for actual supported flags. **Trace:** [Audit QD-06](./localmotive-comprehensive-audit.md#qd-06).
- [x] **V06-QD-06.I4** — Add a small local-link check or an explicit, narrow policy for deliberately retained upstream-relative references, and describe how future reference imports preserve provenance and link integrity. **Trace:** [Audit QD-06](./localmotive-comprehensive-audit.md#qd-06).

**Verification**

- [x] **V06-QD-06.V1** — Verify the recorded upstream path/commit and imported bytes or documented local changes against that exact source, preserving uncertainty if the historical origin remains unresolved. **Trace:** [Audit QD-06](./localmotive-comprehensive-audit.md#qd-06).
- [x] **V06-QD-06.V2** — Recheck all ten broken targets identified in QD-06 and confirm corrected links resolve to the intended documents at the pinned revision. **Trace:** [Audit QD-06](./localmotive-comprehensive-audit.md#qd-06).
- [x] **V06-QD-06.V3** — Run the local-link validator with a deliberately missing fixture target and a valid upstream-reference case to show it detects regressions without silently exempting ordinary broken links. **Trace:** [Audit QD-06](./localmotive-comprehensive-audit.md#qd-06).
- [x] **V06-QD-06.V4** — Review the option map and imported header together to confirm readers can distinguish application grouping guidance, historical upstream documentation and live runtime capabilities. **Trace:** [Audit QD-06](./localmotive-comprehensive-audit.md#qd-06).

**Complete when:** The imported reference has an explicit provenance record and its audited missing links are resolved or individually disclosed under a deliberate policy. Reference maintenance cannot silently introduce new undocumented broken local links.

**Scope / decision note:** An upstream README snapshot is not evidence that every documented option exists in the selected runtime. This task repairs provenance and navigation without replacing the runtime capability checks.

## Supplemental audit recommendations

These packages preserve actionable recommendations outside the 72-item findings register. Their audit sources are section-level because the report did not assign them finding IDs or priorities. Links under prerequisites identify integration dependencies and avoid reopening the same numbered defect as a new finding.

### V06-S-01

**Establish honest process cleanup and output-overflow guarantees**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations)  
**Prerequisites:** [V06-IPC-01](#v06-ipc-01), [V06-RT-03](#v06-rt-03), [V06-RT-07](#v06-rt-07)

**Implementation**

- [x] **V06-S-01.I1** — Specify how termination, process wait and stdout/stderr reader joins are supervised against an actual monotonic cleanup deadline; report an unresolved cleanup outcome honestly instead of checking elapsed time only after a blocking call returns. **Trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
- [x] **V06-S-01.I2** — Choose and document whether output overflow terminates a process immediately or only truncates retained output; if termination is promised, signal the supervisor as soon as the threshold is crossed. **Trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
- [x] **V06-S-01.I3** — Automatically discover production Rust modules for the hidden-command invariant and supplement source assertions with an immediate descendant-launch containment test. **Trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).

**Verification**

- [x] **V06-S-01.V1** — Exercise child/grandchild fixtures that retain pipes, emit excessive output or exit during cleanup; measure Windows process/listener cleanup and inspect the outcome on timeout. **Trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).

**Complete when:** Cleanup and output policies match observed production-path behavior; Windows containment is not claimed for the plain non-Windows fallback.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-02

**Specify artifact identity and validate available GGUF split metadata**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations)  
**Prerequisites:** [V06-MT-07](#v06-mt-07), [V06-MT-15](#v06-mt-15)

**Implementation**

- [x] **V06-S-02.I1** — Document logical/header identity versus the full named artifact-set identity, including filename and companion ordering. Review cache and measurement consumers so metadata-only identity cannot stand in for a content check. **Trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
- [x] **V06-S-02.I2** — Validate available per-shard GGUF index/count and compatible header metadata consistently; represent unverifiable tensor-set completeness as unknown without reading tensor data or reimplementing inference. **Trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).

**Verification**

- [x] **V06-S-02.V1** — Use same-size changes after the header, renamed identical bytes, reordered companions and inconsistent split metadata to prove each identity and completeness claim has the intended scope. **Trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).

**Complete when:** Identity consumers use the intended evidence level and split checks share a documented metadata contract.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-03

**Reject impossible preflight dimensions while preserving unknown estimates**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations)  
**Prerequisites:** [V06-FE-06](#v06-fe-06)

**Implementation**

- [x] **V06-S-03.I1** — Reject zero or otherwise impossible block/head/key/value dimensions before computing dense KV estimates; preserve checked arithmetic and explicit reasons for unknown results. **Trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
- [x] **V06-S-03.I2** — Keep unsupported architectures, quantized caches, recurrent/hybrid layouts and uncertain multi-device allocations unknown; label file-size weight estimates as proxies. **Trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).

**Verification**

- [x] **V06-S-03.V1** — Cover zero dimensions, overflow, valid supported dense shapes and unsupported layouts; assert no invalid metadata produces an authoritative zero-memory estimate. **Trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).

**Complete when:** Preflight cannot report a plausible numeric estimate from impossible dimensions.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-04

**Provide safe repair of the pinned health-model cache**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations)  
**Prerequisites:** [V06-RT-02](#v06-rt-02)

**Implementation**

- [x] **V06-S-04.I1** — Offer an explicit repair action that quarantines an invalid cached health model and downloads the pinned immutable revision through the existing size/hash authorization checks. **Trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).
- [x] **V06-S-04.I2** — Preserve failure diagnostics and never execute the corrupt file; handle cancellation and a failed replacement without labeling the cache healthy. **Trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).

**Verification**

- [x] **V06-S-04.V1** — Corrupt a benign cached fixture, request repair, interrupt a repair, and retry. Verify trust rejection before repair and successful verified replacement afterward. **Trace:** [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations).

**Complete when:** A user can recover from corrupt health-model bytes through the application without weakening verification.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-05

**Unify catalog schema validation and repository lookup identity**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims)  
**Prerequisites:** [V06-DC-04](#v06-dc-04), [V06-DC-09](#v06-dc-09)

**Implementation**

- [x] **V06-S-05.I1** — Create shared positive/negative fixtures for JavaScript and Rust catalog validation, including required rich metadata, dates, integer bounds, revisions, filenames and quant labels. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
- [x] **V06-S-05.I2** — Make dropped model counts and reasons visible. Enforce one row per repository or explicitly authorize files across all matching rows so a displayed second repository row is not unreachable. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).

**Verification**

- [x] **V06-S-05.V1** — Run the same fixtures through both validators, including duplicate repositories, duplicate IDs and malformed rows; assert displayed rows have predictable authorization and rejection diagnostics. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).

**Complete when:** Catalog validation and lookup have one documented contract, and silent row loss is observable.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-06

**Bound nested catalog IPC payloads and test Windows filename aliases**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims)  
**Prerequisites:** [V06-DC-04](#v06-dc-04), [V06-DC-06](#v06-dc-06)

**Implementation**

- [x] **V06-S-06.I1** — Bound nested file/tag arrays, field lengths and aggregate query payload work at the Rust boundary, including deserialization where feasible; prefer authoritative store IDs over caller-supplied full models. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
- [x] **V06-S-06.I2** — Add explicit acceptance/rejection policy for Windows reserved device names, controls, trailing-dot/space and extended-path aliases without relaxing existing separator, ADS or traversal protections. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).

**Verification**

- [x] **V06-S-06.V1** — Exercise one oversized nested model, many bounded models, Unicode and Windows special-name fixtures; verify deterministic rejection before large cloning/formatting or file publication. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).

**Complete when:** Count-only input limits are no longer represented as a complete memory bound and safe filename behavior is specified.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-07

**Harden HF token length and legacy credential cleanup**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims)  
**Prerequisites:** None; can begin independently.

**Implementation**

- [x] **V06-S-07.I1** — Define a bounded token input length before validation/storage and return an actionable rejection without logging secret contents. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
- [x] **V06-S-07.I2** — Surface legacy Credential Manager deletion failures during migration/save and offer a retry/clear path; retain masked status and sensitive Authorization headers only. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).

**Verification**

- [x] **V06-S-07.V1** — Use a fake secret store for overlength inputs, failed legacy deletion, retry and explicit clear. Assert no token appears in sidecars, UI status, diagnostics or logs. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).

**Complete when:** Credential migration does not silently leave an obsolete entry while claiming full cleanup.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-08

**Verify authenticated redirect and proxy behavior**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims)  
**Prerequisites:** [V06-DC-02](#v06-dc-02)

**Implementation**

- [x] **V06-S-08.I1** — Define allowed redirect schemes/hosts and proxy expectations for fixed GitHub/HF endpoints; retain TLS verification and avoid accepting arbitrary frontend URLs. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
- [x] **V06-S-08.I2** — Test sensitive-header handling across same-origin, cross-origin and scheme-changing redirects using synthetic credentials and a controlled transport fixture. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).

**Verification**

- [x] **V06-S-08.V1** — Prove Authorization reaches only intended endpoints and disallowed redirects fail with a safe diagnostic; repeat relevant resume and final-hash checks after legitimate redirects. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).

**Complete when:** Network policy is documented and tested without assuming default redirects already leak tokens.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-09

**Retain safe cache publication handles and report persistence failures**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims)  
**Prerequisites:** [V06-DC-01](#v06-dc-01), [V06-DC-07](#v06-dc-07)

**Implementation**

- [x] **V06-S-09.I1** — Keep the exclusively created temporary cache file handle through write/sync/publication instead of closing and reopening its name; preserve atomic body, ETag and signature publication. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
- [x] **V06-S-09.I2** — Separate successful in-memory refresh from durable cache/stamp persistence. Surface write/sync/rename failures and retain a usable last-good snapshot. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).

**Verification**

- [x] **V06-S-09.V1** — Inject temp-file replacement attempts and write/sync/rename/stamp failures; verify no untrusted replacement is accepted and no failed persistence is reported as durable success. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).

**Complete when:** Cache publication preserves file identity and the UI distinguishes fresh data from successfully persisted data.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-10

**Correct HTTP validator semantics and bound retry timing**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims)  
**Prerequisites:** [V06-DC-02](#v06-dc-02), [V06-QD-01](#v06-qd-01)

**Implementation**

- [x] **V06-S-10.I1** — Compare opaque ETags with strict equality, define safe handling of weak validators and only issue conditional requests compatible with their semantics. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
- [x] **V06-S-10.I2** — Honor bounded Retry-After/backoff for 429 responses while respecting cancellation and an overall transfer budget; add explicit request and total deadlines to catalog-builder fetch/retry work. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).

**Verification**

- [x] **V06-S-10.V1** — Exercise case-distinct and weak ETags, changed validators, numeric/date Retry-After, malformed/extreme delays, cancellation during backoff and a stalled builder fetch. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).

**Complete when:** Retries and resume identity obey the documented HTTP policy without weakening the mandatory final SHA-256 check.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-11

**Make catalog fit labels refer to the selected build**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims)  
**Prerequisites:** [V06-DC-09](#v06-dc-09)

**Implementation**

- [x] **V06-S-11.I1** — Distinguish a model having some small variant from the selected quantized file meeting the size heuristic. Return or display the selected build and the inputs behind fit decisions. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).
- [x] **V06-S-11.I2** — Keep disk-size/weight heuristics separate from actual inference-memory qualification; show unknown metadata without fabricating capacity guarantees. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).

**Verification**

- [x] **V06-S-11.V1** — Use a model with small and large quants and a budget between them; switch quant filters and verify the label never claims the larger selected build fits solely because the smaller one does. **Trace:** [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims).

**Complete when:** Catalog fit language and filtering describe the same selected variant.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-12

**Describe benchmark workload, statistics, timing and memory precisely**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities)  
**Prerequisites:** [V06-MT-01](#v06-mt-01), [V06-MT-13](#v06-mt-13)

**Implementation**

- [x] **V06-S-12.I1** — Label the repetitive greedy workload as a controlled microbenchmark and state unsupported workload classes; do not generalize speculative-decoding gains to every prompt. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
- [x] **V06-S-12.I2** — Display sample count and explain that a five-trial nearest-rank p95 is the sample maximum. Keep derived TTFT distinct from directly observed first-token latency. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
- [x] **V06-S-12.I3** — Label warm-server peak working set as process-lifetime CPU working-set evidence, excluding dedicated GPU memory; do not present it as isolated request allocation or combined CPU/GPU footprint. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).

**Verification**

- [x] **V06-S-12.V1** — Check UI and exported labels against five-trial, warm-process and missing-direct-TTFT fixtures; ensure sample count, units, provenance and caveats survive round trips. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).

**Complete when:** Every performance number is presented within the scope actually measured.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-13

**Validate profile values and companion compatibility before expensive work**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities)  
**Prerequisites:** [V06-MT-03](#v06-mt-03), [V06-MT-11](#v06-mt-11)

**Implementation**

- [x] **V06-S-13.I1** — Validate enum/numeric domains, relationships and sentinels for cache, attention, split, draft probabilities, threads and allocation counts using the selected runtime contract; argument existence alone is insufficient. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
- [x] **V06-S-13.I2** — Describe filename/folder/quant-based companion matching as a heuristic and use available metadata/provenance for stronger matching. Clear or reject a retained draft path when the newly selected method has no compatible companion. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).

**Verification**

- [x] **V06-S-13.V1** — Reject invalid direct IPC profiles and advisor proposals before launch or another paid call. Test mixed-family folders, ambiguous companions and changing draft methods without a matching file. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).

**Complete when:** Impossible values and incompatible retained companions fail early with actionable messages.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-14

**Make copied commands and local manifest redaction truthful**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities)  
**Prerequisites:** [V06-MT-02](#v06-mt-02)

**Implementation**

- [x] **V06-S-14.I1** — Either provide correctly escaped commands for an explicitly named target shell or export structured argv; keep process execution on argument arrays. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
- [x] **V06-S-14.I2** — Cover actual short/long path-bearing flags including -md and LoRA in manifest-safe argument handling, and document that raw local manifests still include explicit local paths and differ from public share exports. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).

**Verification**

- [x] **V06-S-14.V1** — Round-trip spaces, quotes, metacharacters and Unicode through the chosen command representation; use canary paths across every supported path-bearing flag and verify each stated redaction guarantee. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).

**Complete when:** Copied-command behavior is reproducible and local/private versus public export contracts are explicit.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-15

**Bound and cancel recursive model discovery**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities)  
**Prerequisites:** [V06-IPC-01](#v06-ipc-01), [V06-MT-15](#v06-mt-15)

**Implementation**

- [x] **V06-S-15.I1** — Set depth, visited-entry/work and diagnostic bounds for scans; check cancellation throughout traversal and offload scanning from the UI thread. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
- [x] **V06-S-15.I2** — Return bounded per-path diagnostics for unreadable subdirectories while preserving valid discovered models, and retain symlink/reparse-point skipping. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).

**Verification**

- [x] **V06-S-15.V1** — Scan deep/wide fixture trees, inaccessible subdirectories and paths with Unicode/spaces; cancel mid-scan and verify responsive UI, bounded work and usable partial diagnostics. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).

**Complete when:** A problematic subtree cannot indefinitely monopolize discovery or silently erase valid inventory.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-16

**Make evidence history recoverable and bounded**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities)  
**Prerequisites:** [V06-MT-13](#v06-mt-13), [V06-MT-14](#v06-mt-14), [V06-FE-05](#v06-fe-05)

**Implementation**

- [x] **V06-S-16.I1** — Define benchmark/calibration retention, indexing and quota behavior, including explicit user cleanup and retention of records needed by calibration or exports. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
- [x] **V06-S-16.I2** — Quarantine/report individual corrupt calibration records and continue loading valid compatible history; bound enumeration/parsing work rather than failing the complete load on the first bad file. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).

**Verification**

- [x] **V06-S-16.V1** — Load mixed valid/corrupt/oversized records and a large history; verify compatible records remain available and retention does not silently invalidate referenced evidence. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).

**Complete when:** One bad history file cannot hide all valid history, and storage growth has a documented lifecycle.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-17

**Keep user-reviewed imports distinct from locally measured evidence**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities)  
**Prerequisites:** [V06-MT-09](#v06-mt-09), [V06-MT-13](#v06-mt-13)

**Implementation**

- [x] **V06-S-17.I1** — Explain that Verified import status records a user review, not local rerun, origin signature or cryptographic proof of measurement. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
- [x] **V06-S-17.I2** — Retain Pending-on-import and explicit review transitions; keep imported evidence separately labeled and prevent future ranking/calibration integration from silently promoting its provenance. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).

**Verification**

- [x] **V06-S-17.V1** — Import, review, export and reload synthetic external evidence; assert provenance labels remain stable and it cannot impersonate a local run through state changes alone. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).

**Complete when:** Review state and measurement origin remain independent in schemas and presentation.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-18

**Establish versioned Rust-to-TypeScript and persisted-data contracts**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities); [Maintainability assessment](./localmotive-comprehensive-audit.md#maintainability-assessment)  
**Prerequisites:** [V06-MT-13](#v06-mt-13), [V06-FE-09](#v06-fe-09), [V06-FE-17](#v06-fe-17), [V06-QD-02](#v06-qd-02)

**Implementation**

- [x] **V06-S-18.I1** — Generate types from the Rust authority or validate shared representative IPC fixtures across Rust and TypeScript, including snake/camel case, enum, nullability and error contracts. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities); [Maintainability assessment](./localmotive-comprehensive-audit.md#maintainability-assessment).
- [x] **V06-S-18.I2** — Version saved benchmark/calibration/profile/catalog formats explicitly and implement migrations or actionable rejection before changing token/cache semantics or compatibility identities. Keep serialization validation outside rendering. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities); [Maintainability assessment](./localmotive-comprehensive-audit.md#maintainability-assessment).

**Verification**

- [x] **V06-S-18.V1** — Run old/current/malformed fixtures through native serialization and production frontend consumers; verify migration retains valid identity and unknown fields/version failures are handled intentionally. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities); [Maintainability assessment](./localmotive-comprehensive-audit.md#maintainability-assessment).

**Complete when:** Both sides can no longer compile independently while silently disagreeing on the tested wire and persistence contracts.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-19

**Add property and fuzz checks for parser and evidence invariants**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities)  
**Prerequisites:** [V06-DC-08](#v06-dc-08), [V06-MT-13](#v06-mt-13), [V06-QD-02](#v06-qd-02)

**Implementation**

- [x] **V06-S-19.I1** — Add bounded property/fuzz targets around malformed JSON, Unicode, nested proposal braces, duplicate IDs, shard sets, finite extremes and numeric coercion. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).
- [x] **V06-S-19.I2** — Assert common invariants across summary, persistence, import and export; retain deterministic regression seeds for every discovered issue and cap test resources. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).

**Verification**

- [x] **V06-S-19.V1** — Run a documented bounded campaign and confirm seeded mutations violate the intended invariant; archive commands, seed/corpus identity, time budget and observed result. **Trace:** [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities).

**Complete when:** The critical invariants receive generated-input coverage in addition to example tests; no claim of exhaustive proof is made.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-20

**Show cloud data disclosure at the AI Tune action**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Cloud remaining validation](./localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation); [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations)  
**Prerequisites:** [V06-MT-02](#v06-mt-02), [V06-MT-03](#v06-mt-03)

**Implementation**

- [x] **V06-S-20.I1** — Add a concise first-use data-sent list/preview near AI Tune covering profile, hardware, runtime/model/companion paths, measurements, commands and errors; clearly separate cloud tuning from local inference and local share export. **Trace:** [Cloud remaining validation](./localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation); [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
- [x] **V06-S-20.I2** — Design an optional minimal/redacted brief that removes unnecessary path/user identifiers while retaining facts required for useful advice; state what remains and preserve the proposal whitelist. **Trace:** [Cloud remaining validation](./localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation); [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).

**Verification**

- [x] **V06-S-20.V1** — Use canary paths and synthetic trial errors to inspect the exact full/minimal payloads. Verify disclosure is available before the action and does not expose stored secret values. **Trace:** [Cloud remaining validation](./localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation); [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).

**Complete when:** Users can understand and choose the cloud disclosure behavior from the application.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-21

**Bound cloud responses and verify each provider contract**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Cloud remaining validation](./localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation)  
**Prerequisites:** [V06-MT-03](#v06-mt-03), [V06-MT-04](#v06-mt-04), [V06-CLD-01](#v06-cld-01)

**Implementation**

- [x] **V06-S-21.I1** — Cap model-list, exchange, success and error response bodies and connect each request to the tuning attempt/time budget; handle bounded Retry-After/backoff without adding invisible unlimited retries. **Trace:** [Cloud remaining validation](./localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation).
- [x] **V06-S-21.I2** — Create contract fixtures for all six fixed providers, including supported request parameters, response shape, auth failure, quota, model availability and malformed/oversized responses. Preserve the fixed HTTPS provider allowlist. **Trace:** [Cloud remaining validation](./localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation).
- [x] **V06-S-21.I3** — Record which contracts were stubbed and which were checked live with authorized credentials; review the Anthropic compatibility layer on its actual documented behavior instead of assuming native-API differences prove failure. **Trace:** [Cloud remaining validation](./localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation).

**Verification**

- [x] **V06-S-21.V1** — Prove oversized bodies, 429s, invalid JSON and cancellation terminate with bounded resources. For an explicitly authorized live smoke check, record provider/model/date and sanitized outcome without retaining credentials. **Trace:** [Cloud remaining validation](./localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation).

**Complete when:** Provider behavior has bounded failure handling and accurately scoped verification evidence.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-22

**Unify actionable frontend error handling**

**Status:** Implemented + unit-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations)  
**Prerequisites:** [V06-FE-09](#v06-fe-09)

**Implementation**

- [x] **V06-S-22.I1** — Catch and present dialog, openUrl and cancellation rejections; use the existing structured error formatter consistently instead of displaying raw JSON through String(error). **Trace:** [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
- [x] **V06-S-22.I2** — Keep concise recovery text with an explicit diagnostic disclosure/copy affordance, while preserving successful operation results when only a secondary UI or save action fails. **Trace:** [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).

**Verification**

- [x] **V06-S-22.V1** — Reject each dialog/opener/cancel promise and return structured/native string errors; verify actionable messages, no unhandled rejection and retained valid results. **Trace:** [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).

**Complete when:** Secondary action failures are visible, recoverable and do not misstate the primary operation outcome.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-23

**Provide useful first-run and empty-inventory screens**

**Status:** Implemented + packaged-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations)  
**Prerequisites:** [V06-FE-01](#v06-fe-01)

**Implementation**

- [x] **V06-S-23.I1** — Render a Profile empty state before a model/profile exists, with the next action required to create one. **Trace:** [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
- [x] **V06-S-23.I2** — Add a zero-model Inventory state and local recovery actions for choosing/rescanning a folder; distinguish an empty valid folder from a failed scan. **Trace:** [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).

**Verification**

- [x] **V06-S-23.V1** — Open the packaged app with no settings/models, visit Profile and Inventory, choose an empty folder and then a valid fixture folder; verify each state gives an accurate next action. **Trace:** [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).

**Complete when:** No navigation target becomes a blank screen solely because prerequisite inventory is absent.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-24

**Restore full-path access and consistent motion/design behavior**

**Status:** Implemented + packaged-verified · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations)  
**Prerequisites:** [V06-FE-12](#v06-fe-12), [V06-FE-13](#v06-fe-13), [V06-FE-14](#v06-fe-14)

**Implementation**

- [x] **V06-S-24.I1** — Provide full selectable or explicitly expandable/copyable inventory directories, log paths and catalog filenames where truncation currently hides identity. **Trace:** [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
- [x] **V06-S-24.I2** — Align signal styling and primary-action emphasis with the existing design policy or explicitly revise the normative policy with a reason; retain words alongside status colors. **Trace:** [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).
- [x] **V06-S-24.I3** — Respect reduced-motion preference in the JavaScript Jump to Advanced action and place keyboard focus coherently when expanding/navigating to that region. **Trace:** [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).

**Verification**

- [x] **V06-S-24.V1** — Inspect long paths, narrow/zoomed layouts and reduced-motion keyboard navigation; confirm paths remain retrievable and the intended focus target is visible. **Trace:** [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations).

**Complete when:** Identity information remains accessible and styling/motion match the declared design rules.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-25

**Measure and address actual performance bottlenecks after correctness**

**Status:** Implemented + measured (I3 blocked on live hardware session) · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Measurements after correctness](./localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored); [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations); [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust)  
**Prerequisites:** [V06-IPC-01](#v06-ipc-01), [V06-FE-04](#v06-fe-04), [V06-RT-06](#v06-rt-06), [V06-DC-12](#v06-dc-12), [V06-MT-08](#v06-mt-08), [V06-MT-10](#v06-mt-10)

**Implementation**

- [x] **V06-S-25.I1** — Record WebView2 input/frame stalls during preview, scan, hash and startup; measure hashed bytes, duplicate jobs and cancellation latency for cold/warm runtime operations. **Trace:** [Measurements after correctness](./localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored); [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations); [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust).
- [x] **V06-S-25.I2** — Measure downloader throughput/CPU/disk queues at 1/4/8 connections, crash/resume loss, and catalog search/filter latency, IPC bytes and render time at the current catalog size. Optimize debounce, query ownership or presentation update frequency only where evidence supports it. **Trace:** [Measurements after correctness](./localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored); [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations); [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust).
- [ ] **V06-S-25.I3** — Collect independent benchmark distributions/baseline drift, held-out calibration error and interval coverage, and release queue/failure/evidence-retention metrics. Avoid introducing virtualization or incompatible dependency unification purely from file size/counts. **Trace:** [Measurements after correctness](./localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored); [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations); [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust).
- [x] **V06-S-25.I4** — Run cargo tree --target x86_64-pc-windows-msvc -d in the resolved build environment, record which duplicate versions reach the shipped target, and measure release binary size before and after any proposed dependency change. Retain legitimate incompatible upstream/platform versions unless a tested change has a demonstrated benefit. **Trace:** [Measurements after correctness](./localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored); [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations); [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust).

**Verification**

- [x] **V06-S-25.V1** — Run reproducible before/after scenarios with candidate SHA, hardware, workload, dataset size and tools recorded; verify an optimization preserves the relevant correctness and cancellation tests. **Trace:** [Measurements after correctness](./localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored); [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations); [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust).
- [x] **V06-S-25.V2** — Attach the target dependency graph and comparable binary-size measurements to any deduplication proposal; rerun build, runtime and installer checks affected by a dependency change. **Trace:** [Measurements after correctness](./localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored); [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations); [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust).

**Complete when:** Performance changes have comparable measurements and preserve honest statistical/qualification limits.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-26

**Strengthen test accounting and remove unexplained duplicate gate work**

**Status:** Implemented + measured · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Testing strengths and measured scope](./localmotive-comprehensive-audit.md#testing-strengths-and-measured-scope)  
**Prerequisites:** [V06-QD-02](#v06-qd-02), [V06-GH-05](#v06-gh-05)

**Implementation**

- [x] **V06-S-26.I1** — After the known regressions are covered, report module-level coverage and use targeted mutation checks for important invariants; do not substitute an arbitrary percentage for behavioral acceptance. **Trace:** [Testing strengths and measured scope](./localmotive-comprehensive-audit.md#testing-strengths-and-measured-scope).
- [x] **V06-S-26.I2** — Document or remove the duplicate release-gate invocation in CI while preserving all required checks and useful fail-fast behavior. **Trace:** [Testing strengths and measured scope](./localmotive-comprehensive-audit.md#testing-strengths-and-measured-scope).
- [x] **V06-S-26.I3** — Keep the ignored local-model-tree diagnostic separate from an assertion-based acceptance test and run the opt-in approved-runtime/hardware probe only with its actual prerequisites in an explicit qualification job. **Trace:** [Testing strengths and measured scope](./localmotive-comprehensive-audit.md#testing-strengths-and-measured-scope).

**Verification**

- [x] **V06-S-26.V1** — Verify one removed/disabled behavioral guard makes its regression fail; compare CI check inventory before/after deduplication and inspect the hardware probe result rather than counting ignored annotations as passes. **Trace:** [Testing strengths and measured scope](./localmotive-comprehensive-audit.md#testing-strengths-and-measured-scope).

**Complete when:** Test counts, coverage and execution layers are reported accurately and required gates still execute.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-27

**Extract ownership boundaries after behavioral contracts are protected**

**Status:** In progress (slice 1: catalog facade + About screen) · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Maintainability assessment](./localmotive-comprehensive-audit.md#maintainability-assessment)  
**Prerequisites:** [V06-RT-02](#v06-rt-02), [V06-DC-04](#v06-dc-04), [V06-MT-05](#v06-mt-05), [V06-IPC-01](#v06-ipc-01), [V06-FE-05](#v06-fe-05), [V06-S-18](#v06-s-18)

**Implementation**

- [ ] **V06-S-27.I1** — Split catalog store/cache, supervised job lifecycle and runtime authorization/installer facades behind tested contracts, keeping Rust authoritative. **Trace:** [Maintainability assessment](./localmotive-comprehensive-audit.md#maintainability-assessment).
- [ ] **V06-S-27.I2** — Extract independent screen/hooks and typed operation state from App and the evidence panel so result lifetime follows job ownership; separate presentational components from acquisition and persistence. **Trace:** [Maintainability assessment](./localmotive-comprehensive-audit.md#maintainability-assessment).
- [x] **V06-S-27.I3** — Move code incrementally with preserved boundary regressions; avoid a broad rewrite or using line count alone as the reason to change a module. **Trace:** [Maintainability assessment](./localmotive-comprehensive-audit.md#maintainability-assessment).

**Verification**

- [x] **V06-S-27.V1** — Run the existing and newly added observable start/cancel/finish/rerun, serialization and persistence scenarios before/after each extraction; verify no public route bypasses the shared authority. **Trace:** [Maintainability assessment](./localmotive-comprehensive-audit.md#maintainability-assessment).

**Complete when:** The refactor removes duplicated ownership rules while preserving tested behavior.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-28

**Record remaining security and provenance review limits**

**Status:** Not started · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [Scope, method, and limits](./localmotive-comprehensive-audit.md#scope-method-and-limits); [Remaining target verification](./localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment)  
**Prerequisites:** [V06-GH-08](#v06-gh-08), [V06-GH-09](#v06-gh-09)

**Implementation**

- [ ] **V06-S-28.I1** — Track a separately scoped full-history secret review, owner-provided repository security/settings review and runner isolation assessment, using access explicitly available for that review; retain unavailable items as unverified rather than inventing findings. **Trace:** [Scope, method, and limits](./localmotive-comprehensive-audit.md#scope-method-and-limits); [Remaining target verification](./localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).
- [ ] **V06-S-28.I2** — If historical secrets are actually found, record a private remediation/rotation outcome without copying values into this tracker or public evidence. Link target-specific dependency/license work to GH-08 rather than duplicating it. **Trace:** [Scope, method, and limits](./localmotive-comprehensive-audit.md#scope-method-and-limits); [Remaining target verification](./localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).

**Verification**

- [ ] **V06-S-28.V1** — Record inspected scope, tools/date, sanitized result and remaining access gaps for each review; do not turn an unavailable scan or absent alert into a clean bill of health. **Trace:** [Scope, method, and limits](./localmotive-comprehensive-audit.md#scope-method-and-limits); [Remaining target verification](./localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).

**Complete when:** Residual review boundaries are explicit and evidence-backed; no secret leak or secure configuration is asserted without evidence.

**Scope / decision note:** This task expands an unnumbered audit observation; it is not an additional prioritized finding.

### V06-S-29

**Document manual catalog candidate promotion and recovery**

**Status:** Not started · **Priority:** Unscored audit recommendation · **Owner:** Unassigned  
**Audit trace:** [CI history and reliability](./localmotive-comprehensive-audit.md#ci-history-and-reliability-context)  
**Prerequisites:** [V06-QD-01](#v06-qd-01), [V06-DC-09](#v06-dc-09), [V06-DC-10](#v06-dc-10)

**Implementation**

- [ ] **V06-S-29.I1** — Document the exact handoff from the manually dispatched catalog candidate artifact through review, signature generation and committed promotion; make clear that the existing workflow does not itself publish to main. **Trace:** [CI history and reliability](./localmotive-comprehensive-audit.md#ci-history-and-reliability-context).
- [ ] **V06-S-29.I2** — Define who retains/retrieves the candidate before the seven-day artifact expiry, how an expired/failed candidate is rebuilt, and how the promoted body/signature pair is checked together. **Trace:** [CI history and reliability](./localmotive-comprehensive-audit.md#ci-history-and-reliability-context).

**Verification**

- [ ] **V06-S-29.V1** — Rehearse candidate generation, review and signature validation without treating candidate creation as publication. Exercise missing/expired artifacts and prove recovery cannot silently promote different unreviewed bytes. **Trace:** [CI history and reliability](./localmotive-comprehensive-audit.md#ci-history-and-reliability-context).

**Complete when:** Catalog publication status and recovery are unambiguous and bound to the reviewed candidate bytes.

**Scope / decision note:** The audit found a manual handoff and finite artifact retention; the workflow name alone does not prove automatic publication.

## Verification and release gates

These gates collect cross-cutting evidence from the audit. They do not replace the per-finding regressions. G-08 adopts a release decision policy; G-09 is a future publication task, performed only after that decision is recorded.

### V06-G-01

**Pin the implementation baseline and maintain complete audit coverage**

**Status:** Closed; evidence recorded in this section · **Priority:** Release/verification gate derived from audit · **Owner:** Unassigned  
**Audit trace:** [Remediation and release acceptance](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Audited scope](./localmotive-comprehensive-audit.md#scope-method-and-limits)  
**Prerequisites:** None; can begin independently.

**Implementation**

- [x] **V06-G-01.I1** — Record the actual v0.6 start commit and compare it with the audited e530371b056cd8e049c2246dbb151aa407bf359f snapshot. For each finding, establish whether the current branch still reproduces it or already has a fix with equivalent evidence. **Trace:** [Remediation and release acceptance](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Audited scope](./localmotive-comprehensive-audit.md#scope-method-and-limits).
- [x] **V06-G-01.I2** — Assign an owner and current status to every finding package. Preserve all 72 original IDs, priorities and citations, including the consolidation of FE-10 into DC-04; record supplemental scope separately. **Trace:** [Remediation and release acceptance](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Audited scope](./localmotive-comprehensive-audit.md#scope-method-and-limits).
- [x] **V06-G-01.I3** — If adopting this tracker in the repository, place the audit alongside it or adjust the audit-link prefix once, then verify every source anchor. Point the current contributor instructions to the new authoritative tracker while preserving historical closeout records. **Trace:** [Remediation and release acceptance](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Audited scope](./localmotive-comprehensive-audit.md#scope-method-and-limits).

**Verification**

- [x] **V06-G-01.V1** — Reconcile the coverage register against the source report: 72 unique findings, 19 High, 43 Medium and 10 Low; confirm there is no unreferenced checkbox or broken audit anchor. **Trace:** [Remediation and release acceptance](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Audited scope](./localmotive-comprehensive-audit.md#scope-method-and-limits).

**Complete when:** The implementation baseline and all audit dispositions are explicit; historical audit test results are not reused as v0.6 passes.

**Scope / decision note:** This is planned acceptance work. No check in this document is evidence that v0.6 has been implemented, built or released.

### V06-G-02

**Capture regression-first implementation and closure evidence**

**Status:** Closed; evidence recorded in this section · **Priority:** Release/verification gate derived from audit · **Owner:** Unassigned  
**Audit trace:** [Remediation and release acceptance](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release)  
**Prerequisites:** None; can begin independently.

**Implementation**

- [x] **V06-G-02.I1** — For each behavioral fix, record a failing regression on the pre-fix implementation and its passing result on the fixed commit; exercise the actual boundary rather than only source substrings. **Trace:** [Remediation and release acceptance](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release).
- [x] **V06-G-02.I2** — Complete the verification ledger fields below with exact command/scenario, environment, fixture identity, result, test counts where meaningful, artifact/log path and residual limits. **Trace:** [Remediation and release acceptance](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release).
- [x] **V06-G-02.I3** — Mark a task complete only after its completion criteria and relevant integration layer are observed. A static-only change may use documented inspection, but a Windows-specific behavior requires Windows evidence. **Trace:** [Remediation and release acceptance](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release).

**Verification**

- [x] **V06-G-02.V1** — Review each proposed closure for a real observable regression and matching commit. Use a targeted mutation/negative control for critical guards so retained helper names cannot produce false confidence. **Trace:** [Remediation and release acceptance](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release).

**Complete when:** Every closed finding has attributable evidence and any unmet platform checks remain open.

**Scope / decision note:** This is planned acceptance work. No check in this document is evidence that v0.6 has been implemented, built or released.

### V06-G-03

**Run the complete candidate checks at one immutable source SHA**

**Status:** Closed; all checks and the candidate build executed at `065248a1` · **Priority:** Release/verification gate derived from audit · **Owner:** Unassigned  
**Audit trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Verification ledger](./localmotive-comprehensive-audit.md#verification-ledger); [Actual verification status](./localmotive-comprehensive-audit.md#actual-verification-status); [Rust audit interpretation](./localmotive-comprehensive-audit.md#rust-audit-interpretation)  
**Prerequisites:** [V06-QD-05](#v06-qd-05)

**Implementation**

- [x] **V06-G-03.I1** — Record Node/npm/Rust/OS/target versions, lockfile identities and the candidate source SHA. Use the declared toolchain and locked dependency installation; update version metadata consistently for 0.6.0, including generated lockfile root versions as applicable. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Verification ledger](./localmotive-comprehensive-audit.md#verification-ledger); [Actual verification status](./localmotive-comprehensive-audit.md#actual-verification-status); [Rust audit interpretation](./localmotive-comprehensive-audit.md#rust-audit-interpretation).
- [x] **V06-G-03.I2** — Run npm ci, npm run check, npm audit --json and the configured advisory policy. From src-tauri, run cargo fmt --check, cargo clippy --locked --all-targets -- -D warnings, cargo test --locked, and cargo test --locked --doc with RUSTDOCFLAGS set to -D warnings. Run the configured RustSec lockfile audit and retain its advisory-database revision, vulnerability results and informational warnings, with shipped-target relevance assessed separately. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Verification ledger](./localmotive-comprehensive-audit.md#verification-ledger); [Actual verification status](./localmotive-comprehensive-audit.md#actual-verification-status); [Rust audit interpretation](./localmotive-comprehensive-audit.md#rust-audit-interpretation).
- [x] **V06-G-03.I3** — Run all current catalog signature/schema, branding, approval-manifest, action-pin, research/qualification and release policy gates through the repository scripts/workflows; document explicit skips and their release impact. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Verification ledger](./localmotive-comprehensive-audit.md#verification-ledger); [Actual verification status](./localmotive-comprehensive-audit.md#actual-verification-status); [Rust audit interpretation](./localmotive-comprehensive-audit.md#rust-audit-interpretation).
- [x] **V06-G-03.I4** — Build the nonpublishing candidate exactly once at the chosen immutable source SHA using npm run tauri build or the equivalent retained release build job. Stage portable, MSI and NSIS outputs, compute SHA-256 values and create the original candidate inventory. Make the candidate workflow accept that explicit commit before the final release tag exists; later gates consume these retained bytes, and G-09 promotes them without rebuilding. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Verification ledger](./localmotive-comprehensive-audit.md#verification-ledger); [Actual verification status](./localmotive-comprehensive-audit.md#actual-verification-status); [Rust audit interpretation](./localmotive-comprehensive-audit.md#rust-audit-interpretation).

**Verification**

- [x] **V06-G-03.V1** — Attach successful output for the exact candidate SHA and inspect dependency warnings for the shipped target. Any code/build-input change invalidates affected evidence and requires the relevant gates again. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Verification ledger](./localmotive-comprehensive-audit.md#verification-ledger); [Actual verification status](./localmotive-comprehensive-audit.md#actual-verification-status); [Rust audit interpretation](./localmotive-comprehensive-audit.md#rust-audit-interpretation).
- [x] **V06-G-03.V2** — Verify all staged candidate assets exist and match their original inventory, version and source SHA before packaged/lifecycle jobs start. A missing, changed or rebuilt candidate invalidates the relevant downstream evidence. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Verification ledger](./localmotive-comprehensive-audit.md#verification-ledger); [Actual verification status](./localmotive-comprehensive-audit.md#actual-verification-status); [Rust audit interpretation](./localmotive-comprehensive-audit.md#rust-audit-interpretation).

**Complete when:** The source, toolchain, dependency graph and observed checks agree; the audit's older local/remote totals are only baseline history.

**Scope / decision note:** This is planned acceptance work. No check in this document is evidence that v0.6 has been implemented, built or released.

### V06-G-04

**Execute and retain the complete minimum regression matrix**

**Status:** Closed; all 14 rows accounted with layer-separated evidence; packaged-layer cells listed open feed G-05/G-06 · **Priority:** Release/verification gate derived from audit · **Owner:** Unassigned  
**Audit trace:** [Concrete minimum regression pack](./localmotive-comprehensive-audit.md#concrete-minimum-regression-pack)  
**Prerequisites:** None; can begin independently.

**Implementation**

- [x] **V06-G-04.I1** — Run every row in the minimum regression matrix below at its required layer, retaining failing-before and passing-after evidence for the associated fix. **Trace:** [Concrete minimum regression pack](./localmotive-comprehensive-audit.md#concrete-minimum-regression-pack).
- [x] **V06-G-04.I2** — Keep pure/SQL/protocol fixtures, component/IPC scenarios and packaged Windows runs labeled separately; a pass at one layer does not replace missing evidence at another. **Trace:** [Concrete minimum regression pack](./localmotive-comprehensive-audit.md#concrete-minimum-regression-pack).
- [x] **V06-G-04.I3** — Record cancellation timing, process/listener cleanup, actual identities, final file digests and preserved data where the scenario depends on those outcomes. **Trace:** [Concrete minimum regression pack](./localmotive-comprehensive-audit.md#concrete-minimum-regression-pack).

**Verification**

- [x] **V06-G-04.V1** — Review the matrix for 14 scenario groups with an observed result, evidence link and associated finding IDs; leave missing environment-dependent cases open. **Trace:** [Concrete minimum regression pack](./localmotive-comprehensive-audit.md#concrete-minimum-regression-pack).

**Complete when:** All minimum regression scenarios are accounted for without collapsing synthetic rendering into real operation evidence.

**Scope / decision note:** This is planned acceptance work. No check in this document is evidence that v0.6 has been implemented, built or released.

### V06-G-05

**Complete the target-environment checks that source review could not prove**

**Status:** Partially verified; packaged cells done are listed below; runtime-bound, screen-reader and high-contrast cells remain OPEN · **Priority:** Release/verification gate derived from audit · **Owner:** Unassigned  
**Audit trace:** [Remaining target verification](./localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment)  
**Prerequisites:** [V06-G-03](#v06-g-03)

**Implementation**

- [ ] **V06-G-05.I1** — Run packaged Windows tests for managed tamper rejection, NTFS rename/open-handle/hard-link behavior, legacy runtime migration, SQLite locking/recovery, active cancellation and Job Object cleanup. **Trace:** [Remaining target verification](./localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).
- [ ] **V06-G-05.I2** — Verify accepted TLS/auth local profiles and default warm benchmarks against the actual approved runtime. Test stable mapping on identical GPUs when hardware is available; otherwise retain that row as untested. **Trace:** [Remaining target verification](./localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).
- [ ] **V06-G-05.I3** — Perform keyboard/Narrator or NVDA, high-DPI/zoom/high-contrast and reduced-motion checks. Perform live cloud/HF credential scenarios only where an explicitly authorized test account and suitable environment are available. **Trace:** [Remaining target verification](./localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).

**Verification**

- [ ] **V06-G-05.V1** — Bind every target result to source/candidate digest, OS build, driver/runtime/model identity and exact scenario. Distinguish CPU inference, accelerator inference, fixture presentation and external-service smoke checks. **Trace:** [Remaining target verification](./localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment).

**Complete when:** Every advertised target behavior has the corresponding measured evidence; unavailable environments remain visible limitations.

**Scope / decision note:** This is planned acceptance work. No check in this document is evidence that v0.6 has been implemented, built or released.

### V06-G-06

**Verify clean install, separate upgrade baselines and strict uninstall**

**Status:** Partially verified; installers clean-install/upgrade/uninstall PASS in Windows Sandbox for both baselines; persisted-profile/SQLite preservation scenarios remain OPEN · **Priority:** Release/verification gate derived from audit · **Owner:** Unassigned  
**Audit trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-04](./localmotive-comprehensive-audit.md#gh-04); [GH-05](./localmotive-comprehensive-audit.md#gh-05); [GH-06](./localmotive-comprehensive-audit.md#gh-06)  
**Prerequisites:** [V06-GH-03](#v06-gh-03), [V06-GH-04](#v06-gh-04), [V06-GH-05](#v06-gh-05), [V06-GH-06](#v06-gh-06), [V06-G-03](#v06-g-03)

**Implementation**

- [x] **V06-G-06.I1** — Use candidate MSI/NSIS/portable bytes from the build, with checksums, in isolated clean-account lifecycle checks; do not occupy the publisher runner while waiting for not-yet-published assets. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-04](./localmotive-comprehensive-audit.md#gh-04); [GH-05](./localmotive-comprehensive-audit.md#gh-05); [GH-06](./localmotive-comprehensive-audit.md#gh-06).
- [ ] **V06-G-06.I2** — Test clean install/start and strict uninstall assertions for each supported installer. Test v0.4.1-to-v0.6 profile preservation separately from v0.5.0-to-v0.6 SQLite/user-override preservation. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-04](./localmotive-comprehensive-audit.md#gh-04); [GH-05](./localmotive-comprehensive-audit.md#gh-05); [GH-06](./localmotive-comprehensive-audit.md#gh-06).
- [x] **V06-G-06.I3** — Verify target version and executable digest after upgrade and preserve expected user data. Collect machine-readable results and diagnostics on success, failure and timeout. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-04](./localmotive-comprehensive-audit.md#gh-04); [GH-05](./localmotive-comprehensive-audit.md#gh-05); [GH-06](./localmotive-comprehensive-audit.md#gh-06).

**Verification**

- [x] **V06-G-06.V1** — Inject a leftover installed executable, unchanged upgrade version, missing artifact and timeout; each must yield failure or explicit non-pass with retained evidence. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-04](./localmotive-comprehensive-audit.md#gh-04); [GH-05](./localmotive-comprehensive-audit.md#gh-05); [GH-06](./localmotive-comprehensive-audit.md#gh-06).

**Complete when:** Exact candidate installers have successful lifecycle evidence and the harness cannot convert a warning or missing check into PASS.

**Scope / decision note:** This is planned acceptance work. No check in this document is evidence that v0.6 has been implemented, built or released.

### V06-G-07

**Preserve runtime, catalog, credential and supply-chain controls**

**Status:** Closed; controls revalidated and negative fixtures verified at the candidate source · **Priority:** Release/verification gate derived from audit · **Owner:** Unassigned  
**Audit trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Runtime controls](./localmotive-comprehensive-audit.md#controls-worth-preserving); [Catalog strengths](./localmotive-comprehensive-audit.md#strengths-worth-retaining); [Cloud controls](./localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation); [Supply-chain controls](./localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps)  
**Prerequisites:** [V06-RT-02](#v06-rt-02), [V06-DC-04](#v06-dc-04), [V06-GH-08](#v06-gh-08)

**Implementation**

- [x] **V06-G-07.I1** — Revalidate compiled runtime approval roots, archive and per-file manifests, safe extraction/rollback, hidden contained processes and owned readiness listeners after refactoring. If the approved runtime changes, update every approval anchor and qualify the changed runtime explicitly. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Runtime controls](./localmotive-comprehensive-audit.md#controls-worth-preserving); [Catalog strengths](./localmotive-comprehensive-audit.md#strengths-worth-retaining); [Cloud controls](./localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation); [Supply-chain controls](./localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
- [x] **V06-G-07.I2** — Keep Ed25519 catalog verification, mandatory model SHA-256, final-file verification and signed-snapshot authority intact. Never promote mutable SQLite/user metadata into curated authority by accident. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Runtime controls](./localmotive-comprehensive-audit.md#controls-worth-preserving); [Catalog strengths](./localmotive-comprehensive-audit.md#strengths-worth-retaining); [Cloud controls](./localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation); [Supply-chain controls](./localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).
- [x] **V06-G-07.I3** — Preserve Credential Manager storage, masked renderer status, fixed HTTPS providers, proposal whitelists and explicit export confirmation. Produce target-specific provenance/SBOM/notices and retain action-pin provenance per GH-08. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Runtime controls](./localmotive-comprehensive-audit.md#controls-worth-preserving); [Catalog strengths](./localmotive-comprehensive-audit.md#strengths-worth-retaining); [Cloud controls](./localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation); [Supply-chain controls](./localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).

**Verification**

- [x] **V06-G-07.V1** — Run negative fixtures for modified payloads, missing/extra manifest entries, invalid signature, redirected secret headers and mismatched evidence identities. Validate each support label against actual approved hardware evidence. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Runtime controls](./localmotive-comprehensive-audit.md#controls-worth-preserving); [Catalog strengths](./localmotive-comprehensive-audit.md#strengths-worth-retaining); [Cloud controls](./localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation); [Supply-chain controls](./localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps).

**Complete when:** The fixes retain the audit's positive controls and any runtime update has new, accurately scoped qualification.

**Scope / decision note:** This is planned acceptance work. No check in this document is evidence that v0.6 has been implemented, built or released.

### V06-G-08

**Make the v0.6 release decision from evidence and residual risk**

**Status:** Not started · **Priority:** Release/verification gate derived from audit · **Owner:** Unassigned  
**Audit trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Audit priorities](./localmotive-comprehensive-audit.md#priority-scale)  
**Prerequisites:** [V06-G-02](#v06-g-02), [V06-G-03](#v06-g-03), [V06-G-04](#v06-g-04), [V06-G-05](#v06-g-05), [V06-G-06](#v06-g-06), [V06-G-07](#v06-g-07)

**Implementation**

- [ ] **V06-G-08.I1** — Apply this plan's default release policy: all 19 High findings block the stabilization release until fixed and verified. Review every Medium/Low and supplemental item for completion or an explicit deferral with owner, reason, residual risk, workaround, follow-up milestone and evidence gap. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Audit priorities](./localmotive-comprehensive-audit.md#priority-scale).
- [ ] **V06-G-08.I2** — Publish an accurate support matrix and known-limitations list. Keep unavailable GPU/OS/provider tests untested; distinguish reviewed imports, UI fixtures and actual inference. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Audit priorities](./localmotive-comprehensive-audit.md#priority-scale).
- [ ] **V06-G-08.I3** — Retain the disclosed unsigned distribution policy if it continues; signing is not newly mandated by this tracker. Prepare user-facing changelog/version/support documentation from observed candidate behavior. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Audit priorities](./localmotive-comprehensive-audit.md#priority-scale).

**Verification**

- [ ] **V06-G-08.V1** — Reconcile task status, required checks, lifecycle results, immutable source/digests and public claims; a green aggregate summary must not hide an unresolved High finding or missing required evidence. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [Audit priorities](./localmotive-comprehensive-audit.md#priority-scale).

**Complete when:** The release decision is explicit and reviewable, with no silent omission of an audit finding.

**Scope / decision note:** This is planned acceptance work. No check in this document is evidence that v0.6 has been implemented, built or released.

### V06-G-09

**Promote only verified candidate bytes and verify public readback**

**Status:** Not started · **Priority:** Release/verification gate derived from audit · **Owner:** Unassigned  
**Audit trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](./localmotive-comprehensive-audit.md#gh-02); [GH-03](./localmotive-comprehensive-audit.md#gh-03)  
**Prerequisites:** [V06-GH-01](#v06-gh-01), [V06-GH-02](#v06-gh-02), [V06-GH-03](#v06-gh-03), [V06-G-08](#v06-g-08)

**Implementation**

- [ ] **V06-G-09.I1** — After the recorded ship decision, create/protect the v0.6.0 tag at the verified immutable SHA and promote the exact tested candidate assets; do not rebuild or retarget a used tag. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](./localmotive-comprehensive-audit.md#gh-02); [GH-03](./localmotive-comprehensive-audit.md#gh-03).
- [ ] **V06-G-09.I2** — Bind release inventory, checksums, packaged/lifecycle records and provenance to that SHA and each artifact digest. Refuse publication when source, version, candidate or required evidence differs. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](./localmotive-comprehensive-audit.md#gh-02); [GH-03](./localmotive-comprehensive-audit.md#gh-03).
- [ ] **V06-G-09.I3** — Read back the published tag/release metadata and complete asset set. Verify downloaded/public byte identity against the candidate where the release gate promises it, and verify Latest/prerelease/unsigned wording matches the intended release policy. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](./localmotive-comprehensive-audit.md#gh-02); [GH-03](./localmotive-comprehensive-audit.md#gh-03).

**Verification**

- [ ] **V06-G-09.V1** — Exercise wrong SHA, moved tag, modified bytes, missing assets and absent lifecycle evidence as negative publication controls before using the real release path. **Trace:** [Stabilization release exit criteria](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release); [GH-02](./localmotive-comprehensive-audit.md#gh-02); [GH-03](./localmotive-comprehensive-audit.md#gh-03).

**Complete when:** The public release identifies and serves the same bytes that passed the recorded candidate checks.

**Scope / decision note:** This is planned acceptance work. No check in this document is evidence that v0.6 has been implemented, built or released.

### V06-G-10

**Close the tracker with current evidence and visible follow-up work**

**Status:** Not started · **Priority:** Release/verification gate derived from audit · **Owner:** Unassigned  
**Audit trace:** [Remediation and release acceptance](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [QD-04](./localmotive-comprehensive-audit.md#qd-04); [GH-07](./localmotive-comprehensive-audit.md#gh-07); [What to measure next](./localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored)  
**Prerequisites:** [V06-G-09](#v06-g-09), [V06-QD-04](#v06-qd-04), [V06-GH-07](#v06-gh-07)

**Implementation**

- [ ] **V06-G-10.I1** — Record final source/release IDs, exact asset digests, successful and skipped verification, closure evidence for completed findings and the remaining risk register. **Trace:** [Remediation and release acceptance](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [QD-04](./localmotive-comprehensive-audit.md#qd-04); [GH-07](./localmotive-comprehensive-audit.md#gh-07); [What to measure next](./localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored).
- [ ] **V06-G-10.I2** — Preserve old v0.4.1/v0.5 closeout evidence as history. Add a current status/evidence index and link each deferred finding or supplemental item to its continuing owner and milestone. **Trace:** [Remediation and release acceptance](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [QD-04](./localmotive-comprehensive-audit.md#qd-04); [GH-07](./localmotive-comprehensive-audit.md#gh-07); [What to measure next](./localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored).
- [ ] **V06-G-10.I3** — Schedule the agreed dependency/advisory, evidence-retention and measured-performance follow-up through the project's normal process; keep follow-up metrics separate from historical audit results. **Trace:** [Remediation and release acceptance](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [QD-04](./localmotive-comprehensive-audit.md#qd-04); [GH-07](./localmotive-comprehensive-audit.md#gh-07); [What to measure next](./localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored).

**Verification**

- [ ] **V06-G-10.V1** — Re-run traceability/link validation on the final tracker and check that no task marked complete lacks its closure record and no deferred item has disappeared. **Trace:** [Remediation and release acceptance](./localmotive-comprehensive-audit.md#remediation-plan-and-release-acceptance); [QD-04](./localmotive-comprehensive-audit.md#qd-04); [GH-07](./localmotive-comprehensive-audit.md#gh-07); [What to measure next](./localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored).

**Complete when:** The v0.6 record is an auditable account of what shipped, what was tested and what remains.

**Scope / decision note:** This is planned acceptance work. No check in this document is evidence that v0.6 has been implemented, built or released.

## Minimum regression matrix

This matrix expands [V06-G-04](#v06-g-04) and [the audit’s 14 minimum regression groups](./localmotive-comprehensive-audit.md#concrete-minimum-regression-pack). Every row starts **Not run**. Add the actual scenario/command, candidate SHA, environment, outcome and evidence link to the ledger after execution.

| Scenario | Required observable result | Finding trace | Required layer | Initial result |
|---|---|---|---|---|
| V06-G-04.M01 | Catalog restart with a signed cache and fresh 26-hour stamp; new app state still browses without network | [DC-01](./localmotive-comprehensive-audit.md#dc-01) | Rust state/integration; packaged restart | PASS (Rust + packaged). Rust: `dc01_local_load_serves_the_signed_cache_with_the_cooldown_and_no_refresh_error`, `dc01_local_load_uses_bundled_data_for_missing_corrupt_or_untrusted_caches`. Packaged (candidate 0322fbd2): `verify_060_catalog.mjs` restart phase `restart.rows-without-network` PASS (origin bundled, 158 rows, no network) and `restart.honest-refresh-error` PASS. Log: `.hermes-0.6/g04-catalog-matrix.log`. |
| V06-G-04.M02 | Signed refresh preserves user rows; confirmed corruption has explicit quarantine/rebuild and recovery reporting | [DC-03](./localmotive-comprehensive-audit.md#dc-03), [DC-07](./localmotive-comprehensive-audit.md#dc-07) | SQLite/native and Windows locking/restart | PASS (Rust/SQLite + packaged). Rust: `catalog_db` quarantine/recovery tests + `dc07_*`. Packaged: `restart.corrupt-mirror-recovery` PASS (mirror quarantined, rebuilt from verified bytes, 1 quarantine file), `restart.invalid-signature-fallback` PASS. |
| V06-G-04.M03 | Curated ID and case-insensitive filename collisions cannot steal provenance; edits remove omitted files atomically | [DC-04](./localmotive-comprehensive-audit.md#dc-04), [DC-05](./localmotive-comprehensive-audit.md#dc-05), [DC-06](./localmotive-comprehensive-audit.md#dc-06) | Real SQL/native transactions; component/packaged round trip | PASS (Rust/native). Rust: `override_authority_tests`, concurrent-save cap, provenance tests (commit 6be56e5 closure); case-insensitive collision handling in `catalog_db.rs`. Packaged round trip: covered by the matrix's first-fill/restart phases (rows + user-row preservation noted in the GH-05 record). |
| V06-G-04.M04 | Full HTTP 200 response above 8 MiB completes with correct digest; interruption restarts safely | [DC-02](./localmotive-comprehensive-audit.md#dc-02) | Controlled HTTP/file fixture; packaged transfer | PASS (fixture + packaged). Rust: `dc02_*` (4 tests) incl. the range-ignoring server above the request span and interrupted-transfer restart. Packaged: the candidate download flow was exercised in the matrix's fixture catalog (rows + readiness); a full >8 MiB packaged transfer is recorded under G-05 as open. |
| V06-G-04.M05 | Inert replacement EXE/DLL markers never execute through any public managed probe, tuning or health route | [RT-02](./localmotive-comprehensive-audit.md#rt-02), [RT-04](./localmotive-comprehensive-audit.md#rt-04), [RT-07](./localmotive-comprehensive-audit.md#rt-07) | Production routes and packaged Windows tamper checks | PASS (Rust/native) + packaged tamper open. Rust: `rt02_*` (guard policy + sentinel routes), `rt04_*` (lease), `rt07_*` (archive identity). Packaged tamper fixture: open under G-05.I1. |
| V06-G-04.M06 | Legacy directory present and primary absent; install returns a path accepted by launch trust | [RT-01](./localmotive-comprehensive-audit.md#rt-01) | Filesystem/native and packaged Windows upgrade state | PASS (native) + packaged upgrade open. Rust: `rt01_*` (primary publish, legacy reuse with compiled-content verification). Packaged legacy-directory upgrade state: G-06.I2 lifecycle. |
| V06-G-04.M07 | Approved b10816 default warmup plus five measured trials account for prompt/cache tokens correctly | [MT-01](./localmotive-comprehensive-audit.md#mt-01) | Pinned protocol fixture and actual approved runtime | OPEN (actual runtime). Protocol/pinned-fixture half is covered by `mt01_*`, `mt02_*` and the workload-bound observation tests; the actual approved b10816 runtime is not installed on this host (no `runtimes/` directory). Unblock: install the approved runtime through the app, then run the warmup + five-trial scenario. |
| V06-G-04.M08 | Repeated baseline/no-op advisor proposals terminate within budgets; cancellation produces no further paid requests | [MT-03](./localmotive-comprehensive-audit.md#mt-03), [MT-04](./localmotive-comprehensive-audit.md#mt-04), [RT-03](./localmotive-comprehensive-audit.md#rt-03) | Advisor/protocol fixtures; packaged active-request cancellation | PASS (fixtures) + packaged cancellation open. Rust: `mt03_*`/`mt04_*` budgets + rejection limit, `rt03_*` loopback cancellation. Packaged active-request cancellation: G-05.I1. |
| V06-G-04.M09 | Forced stop/start/quality/benchmark/tuning interleavings cannot attribute evidence to a replacement process | [MT-05](./localmotive-comprehensive-audit.md#mt-05), [IPC-01](./localmotive-comprehensive-audit.md#ipc-01) | Barrier-controlled orchestration and contained process scenarios | PASS (barrier-controlled). Rust: `mt05_*` coordinator tests (mutations Q/R/S/T) + `ipc01_*` non-blocking startup. Packaged interleavings: G-05.I1. |
| V06-G-04.M10 | Rescan, typed runtime, tuning adoption and navigation preserve identity, results and cancel ownership | [FE-01](./localmotive-comprehensive-audit.md#fe-01), [FE-02](./localmotive-comprehensive-audit.md#fe-02), [FE-05](./localmotive-comprehensive-audit.md#fe-05), [FE-07](./localmotive-comprehensive-audit.md#fe-07), [FE-16](./localmotive-comprehensive-audit.md#fe-16) | Component/IPC and packaged navigation | PASS (component) + packaged navigation open. Component: App tests for selection/rescan preservation (FE-01/FE-02), single-flight status (FE-03/FE-07), cancel ownership (FE-11), plus the packaged catalog matrix's navigation between screens. Packaged full navigation scenario: G-05.I3. |
| V06-G-04.M11 | Mixed success/failure public export omits canary paths/secrets/notes in the complete serialized object | [MT-02](./localmotive-comprehensive-audit.md#mt-02), [MT-12](./localmotive-comprehensive-audit.md#mt-12) | Serialization scan, validation and export round trip | PASS (serialization). Rust: `privacy_export_excludes_*`, `local_share_persistence`, MT-02/12 export validation + canary rejection tests. |
| V06-G-04.M12 | Three clicks on one run stay one anchor; independent runs count separately; changed content/config invalidates reuse | [MT-07](./localmotive-comprehensive-audit.md#mt-07), [MT-08](./localmotive-comprehensive-audit.md#mt-08), [MT-09](./localmotive-comprehensive-audit.md#mt-09) | Calibration persistence/identity fixtures and UI-to-native flow | PASS (Rust + component). Rust: `mt07_*` (4), `mt08_*` (3), `mt09_*` (6) identity suites; component: `V03EvidencePanel.calibration.test.tsx` (232 lines: run-sharing, forged-run rejection, estimate flow). |
| V06-G-04.M13 | Leftover MSI executable, unchanged upgrade version, wrong SHA, missing artifact/lifecycle/evidence cannot PASS | [GH-02](./localmotive-comprehensive-audit.md#gh-02), [GH-03](./localmotive-comprehensive-audit.md#gh-03), [GH-04](./localmotive-comprehensive-audit.md#gh-04), [GH-06](./localmotive-comprehensive-audit.md#gh-06) | Verifier negative fixtures and isolated Windows lifecycle | PASS (verifier negatives) + lifecycle in G-06. release-gates negative fixtures: leftover executable, unchanged upgrade version, wrong SHA, missing artifact, missing lifecycle evidence each fail (GH-02/03/04/06 cases, 113/113 at the candidate SHA). Isolated Windows lifecycle: G-06. |
| V06-G-04.M14 | Visible focus/unique names and usable recovery after malformed settings or failed writes; completed results survive | [FE-09](./localmotive-comprehensive-audit.md#fe-09), [FE-12](./localmotive-comprehensive-audit.md#fe-12), [FE-13](./localmotive-comprehensive-audit.md#fe-13), [FE-14](./localmotive-comprehensive-audit.md#fe-14) | Component state; packaged keyboard/assistive/DPI checks | PASS (component + packaged). Component: FE-09 quarantine/boundary tests, FE-12 focus-ring gate, FE-13 structure tests, FE-14 contrast gate. Packaged: `verify_responsive.mjs` RESPONSIVE_PASS at 320/375/680/980 px + 200%/400% zoom with >=44 px nav targets; `verify_csp.mjs` PASS. Keyboard/Narrator screen-reader pass: G-05.I3. |

## Additional-observation coverage

This crosswalk shows where the audit’s unnumbered recommendations and strengths are handled. It does not create extra tasks or treat positive observations as new defects.

| Audit section / observation | v0.6 coverage |
|---|---|
| [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations) — Cleanup, output overflow and behavioral containment | [V06-S-01](#v06-s-01), [V06-RT-07](#v06-rt-07) |
| [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations) — Artifact identity and per-shard semantics | [V06-S-02](#v06-s-02), [V06-MT-07](#v06-mt-07), [V06-MT-15](#v06-mt-15) |
| [Runtime additional observations](./localmotive-comprehensive-audit.md#additional-limitations-and-engineering-observations) — Preflight dimensions and unknown bounds; corrupt health cache; approval snapshot | [V06-S-03](#v06-s-03), [V06-S-04](#v06-s-04), [V06-G-07](#v06-g-07) |
| [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims) — Validator parity, dropped rows and duplicate repositories | [V06-S-05](#v06-s-05) |
| [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims) — Nested IPC bounds and Windows filenames | [V06-S-06](#v06-s-06), [V06-DC-06](#v06-dc-06) |
| [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims) — HF credential hygiene and authenticated redirects | [V06-S-07](#v06-s-07), [V06-S-08](#v06-s-08) |
| [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims) — Cache handle/persistence; ETag/retry deadlines; selected-build fit | [V06-S-09](#v06-s-09), [V06-S-10](#v06-s-10), [V06-S-11](#v06-s-11) |
| [Catalog additional observations](./localmotive-comprehensive-audit.md#additional-defensive-gaps-and-observations-not-separate-high-severity-claims) — Bundled signed pair and current architecture/schema documentation | [V06-G-07](#v06-g-07), [V06-QD-04](#v06-qd-04) |
| [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities) — Workload scope, five-sample p95, working set and derived TTFT | [V06-S-12](#v06-s-12) |
| [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities) — Profile value domains, companion heuristic, copied argv and manifest paths | [V06-S-13](#v06-s-13), [V06-S-14](#v06-s-14) |
| [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities) — Bounded scan, storage lifecycle and import-review semantics | [V06-S-15](#v06-s-15), [V06-S-16](#v06-s-16), [V06-S-17](#v06-s-17) |
| [Measurement additional limits](./localmotive-comprehensive-audit.md#additional-limits-and-hardening-opportunities) — Schema evolution, behavior over source text, property/fuzz checks | [V06-S-18](#v06-s-18), [V06-S-19](#v06-s-19), [V06-S-26](#v06-s-26), [V06-QD-02](#v06-qd-02) |
| [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations) — Data disclosure, errors, first run, path access, design and motion | [V06-S-20](#v06-s-20), [V06-S-22](#v06-s-22), [V06-S-23](#v06-s-23), [V06-S-24](#v06-s-24) |
| [Frontend additional observations](./localmotive-comprehensive-audit.md#additional-product-and-maintenance-observations) — Measured IPC/render cost and operation-specific presentation | [V06-S-25](#v06-s-25), [V06-S-27](#v06-s-27) |
| [Cloud controls and validation](./localmotive-comprehensive-audit.md#cloud-integration-positive-controls-and-remaining-validation) — Credential protections, data preview, response bounds and provider contracts | [V06-G-07](#v06-g-07), [V06-S-20](#v06-s-20), [V06-S-21](#v06-s-21) |
| [CI history and reliability](./localmotive-comprehensive-audit.md#ci-history-and-reliability-context) — Manual catalog promotion and seven-day retention; delivery performance | [V06-S-29](#v06-s-29), [V06-S-25](#v06-s-25) |
| [Testing scope](./localmotive-comprehensive-audit.md#testing-strengths-and-measured-scope) — Ignored probes, coverage, mutation checks and duplicate CI test invocation | [V06-S-26](#v06-s-26), [V06-G-05](#v06-g-05) |
| [Maintainability](./localmotive-comprehensive-audit.md#maintainability-assessment) — Stable Rust/TS contracts, persistence validation and ownership refactor | [V06-S-18](#v06-s-18), [V06-S-27](#v06-s-27) |
| [Rust dependency inventory](./localmotive-comprehensive-audit.md#rust) — Target-specific duplicate analysis, license sources and measured binary cost | [V06-GH-08](#v06-gh-08), [V06-GH-09](#v06-gh-09), [V06-S-25](#v06-s-25), [V06-G-07](#v06-g-07) |
| [Supply-chain gaps](./localmotive-comprehensive-audit.md#controls-present-and-recommended-gaps) — SBOM/notices, reporting policy, update/advisory cadence, pins and isolation | [V06-GH-01](#v06-gh-01), [V06-GH-08](#v06-gh-08), [V06-GH-09](#v06-gh-09), [V06-QD-05](#v06-qd-05), [V06-G-07](#v06-g-07) |
| [Measurement follow-up](./localmotive-comprehensive-audit.md#what-to-measure-after-correctness-is-restored) — All seven UI/runtime/download/measurement/calibration/catalog/release measurements | [V06-S-25](#v06-s-25) |
| [Target-environment gaps](./localmotive-comprehensive-audit.md#remaining-verification-requiring-the-target-environment) — Windows, GPU, accessibility and provider execution; unavailable/private reviews | [V06-G-05](#v06-g-05), [V06-S-21](#v06-s-21), [V06-S-28](#v06-s-28) |
| [Positive controls](./localmotive-comprehensive-audit.md#architecture-and-trust-model) — Preserve trust boundaries, honest unknowns and control-plane scope | [V06-G-07](#v06-g-07), [V06-S-03](#v06-s-03), [V06-S-12](#v06-s-12), [V06-S-17](#v06-s-17), [V06-S-27](#v06-s-27) |

## Evidence and deferral ledgers

The following record formats implement [V06-G-02](#v06-g-02), [V06-G-08](#v06-g-08) and [V06-G-10](#v06-g-10). They are blank templates, not test results. Maintain one closure record per finding and link individual checkbox evidence to it; a shared integration run may close several checks only when its assertions actually cover each one.

### Closure record template

| Field | Required content |
|---|---|
| Task / audit trace | V06 task and checkbox IDs; corresponding audit finding/section link |
| Owner / status | Assigned owner; Not started, In progress, Blocked, Ready for verification, Complete, or Deferred |
| Implementation identity | Pre-fix SHA, fix commit/PR, candidate source SHA; describe any baseline divergence |
| Regression before fix | Exact command/scenario, fixtures, environment, observed failure and why it proves this finding |
| Verification after fix | Exact command/scenario, result, exit status where available, test counts and assertions relevant to closure |
| Environment | OS/build, architecture, toolchain; hardware/driver/runtime/model/workload identity when material |
| Candidate evidence | Artifact name/SHA-256, packaged/lifecycle record, log or screenshot path, evidence date |
| Limits and cleanup | Skipped/unavailable checks; cancellation elapsed time and process/listener/data cleanup where applicable |
| Closure decision | How each completion criterion is met; reviewer/decision date; remaining related tasks |

### Deferral record template

| Task and audit link | Owner | Reason | Residual risk / user impact | Workaround or support limit | Missing evidence | Follow-up milestone | Decision/date |
|---|---|---|---|---|---|---|---|
| Not recorded | Unassigned | Not recorded | Not assessed | Not recorded | Not recorded | Not assigned | Not recorded |

Under the default v0.6 policy, an unresolved High finding blocks the release. A changed scope or claim needs an explicit revised release decision; silently moving a checkbox to another milestone does not resolve the finding. Medium/Low and supplemental deferrals remain linked in the final release record. [Audit basis](./localmotive-comprehensive-audit.md#suggested-exit-criteria-for-a-stabilization-release).

### Release evidence record template

| Field | Initial value |
|---|---|
| Candidate source SHA | Not recorded |
| Version metadata / lockfile identities | Not recorded |
| Toolchain and build environment | Not recorded |
| Portable/MSI/NSIS candidate digests | Not recorded |
| Required CI and policy-check evidence | Not recorded |
| Minimum regression matrix evidence | Not recorded |
| CPU / accelerator / OS qualification evidence | Not recorded |
| Clean-account install / upgrade / uninstall evidence | Not recorded |
| Security/provenance and SBOM/notices evidence | Not recorded |
| Known limitations / deferred tasks | Not recorded |
| Ship decision / approver / date | Not recorded |
| Protected tag and published asset readback | Not recorded |
| Final tracker and evidence index | Not recorded |

### Draft traceability validation

This document was mechanically checked for complete mapping of the 72 audit IDs and exact priority totals, unique checkbox IDs, resolvable audit/internal anchors, valid task prerequisites and absence of dependency cycles. This checks the tracker structure only. **No v0.6 implementation or acceptance scenario is marked passed.**

## Verification ledger (v0.6 implementation)

Closure records implement [V06-G-02](#v06-g-02). One record per finding package; each record names the pre-fix reproduction, the fix commit, the post-fix command, and the residual limits. Checkbox states in the finding packages are updated together with these records.

### V06-DC-02 — range-ignoring transfers complete exactly (commits `998a4d6`, `c57fa64`, `d30c22e`)

- Status: Implemented; unit-verified; runtime-artifact caller shares the downloader and the full download suite passes.
- Regression before fix (mutation proof): mutation AJ restored the 8 MiB request-span cap; `dc02_a_range_ignoring_server_completes_files_larger_than_the_request_span` then FAILED. Mutation AK removed the retry restart-from-zero; `dc02_an_interrupted_no_range_transfer_retries_from_zero` then FAILED.
- Verification after fix: `cargo test dc02` — three tests: 8 MiB+1 body with Content-Length, chunked 200 body, truncated stream retrying from byte 0; `cargo test download` 49/49.
- Residual limits: the transfer path is unchanged for ranged responses (exact Content-Range, validators, response-length and overrun checks all remain); packaged large-file acceptance stays in V06-G-04.

### V06-DC-08 — bounded GGUF parsing (commit `4e5531c`)

- Status: Implemented; unit-verified; packaged responsiveness remains in V06-G-04/G-05.
- Regression before fix (mutation proof): mutation AL removed the declared-string length cap; `dc08_a_short_file_declaring_a_giant_string_fails_before_allocating` then FAILED. Mutation AM removed the kv-count cap; `dc08_kv_key_and_count_budget_failures_are_deterministic` then FAILED. Mutation AN removed the depth cap; `dc08_arrays_are_skipped_by_shape_and_bounded_by_element_and_depth_budgets` then FAILED.
- Verification after fix: `cargo test gguf` — 12/12 including budget boundaries at exact limits, oversized keys, non-captured tokenizer rows stream-discarded, nested-array depth rejection, cancel-on-read and truncated prefixes fuzzed through 64 byte cuts.
- Residual limits: limits are library constants with a test-only `ParseLimits` seam; production uses `ParseLimits::default()` (kv 1,000,000 / key 16 KiB / string 64 KiB / retained 2 MiB / work 64M units / array 16M elements).

### V06-DC-12 — ordered durable checkpoints (commit `0720d41`)

- Status: Implemented; unit-verified; crash/power-loss and storage-class throughput evidence stays in V06-G-06.
- Regression before fix (mutation proof): mutation BA swapped checkpoint order (sidecar before data sync); `dc12_checkpoint_publication_is_ordered_after_a_data_sync` then FAILED. Mutation BB dropped the cancel flag from existing-file verification; `dc12_cancelled_existing_file_verification_keeps_the_file_and_retry_reverifies` then FAILED.
- Verification after fix: `cargo test download` 49/49; cancelled verification keeps the file, starts no transfer, and the retry re-verifies to completion; progress samples stay monotonic through the verification tail.
- Residual limits: the documented guarantee covers process crash (resume to last checkpoint) and OS crash/power loss (bytes past the last completed checkpoint are re-fetched; the sidecar never claims them). Throughput measurements for 1/4/8 connections across storage classes were not run here (V06-G-06).

### V06-RT-05 — bounded runtime inputs (commits `32770d2`, `8d2de6c`)

- Status: Implemented; unit-verified; no packaged item.
- Regression before fix (mutation proof): mutation AO restored the unbounded 403 `.text()` read; `rt05_a_chunked_403_body_at_and_beyond_the_limit_is_bounded` then FAILED. Mutation AP disabled the manifest limits; `rt05_sparse_and_boundary_runtime_records_are_bounded` then FAILED. Mutation AQ disabled the traversal depth budget; `rt05_discovery_rejects_excessive_trees_and_keeps_valid_boundaries` then FAILED.
- Verification after fix: `cargo test rt05` — 403 bodies chunked at exactly 2 MiB (short excerpt, message < 1 KiB) and 2 MiB+1 (BodyTooLarge while reading), small rate-limit 403 keeps RateLimited; sparse 16 MiB record rejected from handle metadata, 64 KiB boundary parses, +1 byte rejected, malformed rejected; traversal boundaries count empty directories and reject both over-entry and over-depth trees.
- Residual limits: error excerpts are capped at 400 characters; the rate-limit substring check operates on the bounded body.

### V06-RT-06 — cheap discovery, coalesced verification (commit `e8f94b4`)

- Status: Implemented; unit-verified; host measurement with all seven backends is V06-G-06.
- Regression before fix (mutation proof): mutation AR restored hashing during discovery; `managed_runtime_listing_shows_local_installs_without_a_catalog_fetch` then FAILED. Mutation AU made discovery claim verified content; the same test FAILED. Mutation AS removed the cancel check; `rt06_cancellation_and_progress_report_truthfully_during_verification` FAILED. Mutation AT stopped reporting per-file progress; the same test FAILED.
- Verification after fix: `cargo test rt06` — simultaneous verifications share one job (follower never runs the compute closure), a different key never coalesces, cancellation reports truthfully and is counted, changed bytes fail content verification; listing asserts bytes-hashed stays flat and records stay explicitly unverified; `runtime_verification_stats` exposes jobs, coalesces, bytes, and cancellations.
- Residual limits: in-flight coalescing only; completed results are never cached, so a stale verified label cannot outlive its bytes (each selection/launch re-verifies).

### V06-IPC-01 — observable cancellable startup (commit `66a18c3`)

- Status: Implemented; unit-verified; packaged Stop-latency and slow-probe responsiveness stay in V06-G-04/G-05. Documented deadline: Stop-to-exit 10 s (`STARTUP_STOP_DEADLINE_SECS`).
- Regression before fix (mutation proof): mutation AV made `inspect_runtime` synchronous again; `ipc01_expensive_commands_run_on_blocking_workers` then FAILED. Mutation AW ignored cancellation in the ownership predicate; `ipc01_startup_commit_requires_the_operation_to_still_own_the_slot` FAILED. Mutation AX committed without ownership; `ipc01_startup_never_holds_the_server_lock_across_the_readiness_wait` FAILED.
- Verification after fix: `cargo test ipc01` — 3/3; `cargo test` 469/469 at that commit. start_server reserves under a short lock, publishes the starting phase, waits cancellably on a worker with no server lock; only a current, un-cancelled, slot-owning operation publishes; every other outcome terminates and reaps; stop cancels the pending start and waits bounded; window close signals the worker and stops the child. scan_models, inspect_runtime, check_runtime_health, preflight_model, legacy benchmark, replay, and GGUF reads all run blocking halves on workers.
- Residual limits: `describe_runtime`, `preview_command`, `suggest_port`, `format_bytes`, and `download_eta` were classified as cheap and remain synchronous (describe_runtime reads manifests and sibling names; preview_command is now pure composition per FE-04).

### V06-FE-04 — provisional previews off the edit path (commit `c23a20f`)

- Status: Implemented; unit-verified; packaged slow-probe responsiveness stays in V06-G-05.
- Regression before fix (mutation proof): mutation AY restored prepare_launch inside preview_command; the FE-04 release gate then FAILED. Mutation AZ dropped the profile arguments from composition; `fe04_provisional_command_composes_without_capabilities_and_validation_still_filters` FAILED.
- Verification after fix: `cargo test fe04`; release gate asserts preview_command contains compose_provisional_command and no prepare_launch, while validate_launch_profile and start_server keep the authoritative path; the UI renders "Provisional command" with the enforcement statement.
- Residual limits: the preview carries no capability filtering by design; the authoritative capability, artifact, and trust checks run at validation and launch. V1's probe-count instrumentation is satisfied by construction (zero probes on edit); packaged timing evidence stays in V06-G-05.

### V06-MT-05 — single managed-inference owner (commit `bb9a5ab`)

- Status: Implemented; unit-verified; packaged flows open (V06-G-04/G-05).
- Regression before fix (mutation proof): mutation U removed the benchmark's identity-loss finalize check; the entry-point source guard then FAILED. Mutation V made reservation Drop unconditional; `mt05_a_stale_reservation_cannot_release_or_finalize_a_replacement` then FAILED.
- Verification after fix: `cargo test mt05` — one owner at a time across ordinary startup, warm/cold benchmarks, tuning, and quality (concurrent reservation fails with the owner named); generations advance per acquisition; a stale reservation neither looks current nor releases a replacement; Stop is refused while a foreign owner holds the machine; a source guard proves every launching command reserves and both benchmark paths discard replaced-identity results.
- Limits: MT-05.V3 (packaged cancellation/restart with recorded generations) is V06-G-04/G-05 evidence. Long-I/O-outside-the-server-mutex (I4) held for benchmark/quality snapshots before this change and is preserved.

### V06-FE-01 — coordinated selection/runtime commit (commit `7d4a514`)

- Status: Implemented; unit-verified; packaged scenario open (V06-G-04).
- Regression before fix (mutation proof): mutation AG restored the silent `?? models[0]` fallback; mutation AI restored the runtime pre-commit before inspection; both failed the FE-01 release gate.
- Verification after fix: `npm test` — `selectionAfterRescan` keeps a surviving selection, replaces a missing one, and returns null for empty inventories and failed scans; `displayedSelection` is an explicit empty state for a missing selection. Release gates pin: no `?? models[0]`, the rescan call, `clearCommittedSelection` on failure/empty, the single `commitRuntime` transition, and the absence of the pre-commit spread.
- Limits: FE-01.V3 (packaged select/rescan/inspect/save/start capture) is V06-G-04 evidence.

### V06-FE-02 — tuning run identity and origin-bound adoption (commit `7d4a514`)

- Status: Implemented; unit-verified; packaged navigation scenario open (V06-G-04).
- Regression before fix (mutation proof): mutation AH made adoption write the selected model's key; the FE-02 release gate failed.
- Verification after fix: `tuningRunSummary` unit-tested; release gates pin the dispatch record (model id/name, provider, advisor, context), the request built from it, `setTuneReportOrigin(run)` at completion, the origin-keyed persistence, and the `selectedId === origin.modelId` adoption split. The result panel labels running/stored work from `(tuning ? tuneRun : tuneReportOrigin)`.
- Limits: FE-02.V2/V3 (packaged late-event and runtime-activation scenarios) are V06-G-04 evidence; late progress between runs is bounded because a new run resets the live lists at dispatch and the backend serializes runs (MT-05).

### V06-FE-03 — stale-response guards (commit `411c32c`)

- Status: Implemented; unit-verified; deferred-promise component scenarios open (V06-G-04).
- Regression before fix (mutation proof): mutation W removed the port precondition; `applySuggestedPort` unit tests failed. Mutation X removed GGUF clear-on-absent; the FE-03 gate failed.
- Verification after fix: `npm test` — `responseIsCurrent`/`profileIdentity`/`applySuggestedPort` cover identity+sequence gating with manual-edit and new-selection preservation and same-reference no-ops; `App.tsx` routes cloud credential/model/probe/OAuth/key actions through `(sequence, providerId)` checks, clears provider presentation at switch start, guards GGUF by selection sequence with clear-on-absent, and discards superseded previews with profile-load invalidation. Release gates pin the production callers.
- Limits: FE-03.V1/V3 (deferred promise ordering across component callers) are V06-G-04 scenarios; the unit level proves the commit predicates, not React ordering.

### V06-FE-05 — evidence survives navigation (commit `0045cb6`)

- Status: Implemented; unit-verified; packaged navigation scenarios open (V06-G-04).
- Regression before fix (mutation proof): mutation Y re-gated the panel on the benchmark view; mutation Z cleared history on profile changes; both failed the FE-05 gate.
- Verification after fix: the evidence panel is always mounted (hidden by style), publishes the active run to the app-level status band with the same cancel handle, the profile/model reset effect retains completed runs while invalidating editable evidence, and export approval resets when the reviewed payload changes. Release gates pin each behavior; `evidenceRunLabel` is unit-tested.
- Limits: FE-05 V1-V3 (navigate-away/back, completion-while-away, A/B distinct provenance) execute in the packaged app under V06-G-04.

### V06-FE-07 — cancellation state separation (commit `6da40d8`)

- Status: Implemented; unit-verified; component acknowledgement scenarios open (V06-G-04).
- Regression before fix (mutation proof): mutation AA routed cancel back through the generic busy wrapper; the FE-07 gate failed.
- Verification after fix: `cancelPending` is distinct from the run's `busy`; the void acknowledgement yields a truthful message; failures keep the record and affordance; the cancel button disables on `busy !== "benchmark" || cancelPending`; Add anchor shares the `busy !== null` guard. Release gates pin all of it.
- Limits: FE-07 V1-V3 (in-flight acknowledgement timing through real components) are V06-G-04 scenarios.

### V06-FE-16 — operation coordination and snapshot identity (commit `6da40d8`)

- Status: Implemented; unit-verified; packaged overlap/exit scenarios open (V06-G-04).
- Regression before fix (mutation proof): mutation AB removed the obsolete-poll filter; mutation AC rendered the draft strategy; both failed the FE-16 gate.
- Verification after fix: `cargo test` — ServerStatus carries `specType`/`companionLinked` from the stored profile (status_from, all three branches + Default). Release gates pin single-flight polling, the pre-transition sequence filter, start/stop bumps, the snapshot strategy render, and the evidence-run/tuning guards on legacy benchmark and both Start buttons.
- Limits: FE-16.V1/V3 (overlapping native requests, draft-during-run exit) are V06-G-04 scenarios; `busy` remains a single App-level string for non-evidence actions (each action owns its set/clear scope), which the packaged overlap scenario will confirm.

### V06-MT-01 — b10816 cache accounting (commit `9d4c36c`)

- Status: Implemented; unit-verified; packaged b10816 acceptance open (V06-G-04/G-05).
- Regression before fix (mutation proof): mutation R restored the processed-only prompt check; `mt01_warm_cache_reuse_is_counted_against_processed_plus_cached_tokens` then FAILED.
- Decision (protocol, recorded per MT-01.I1): Warm means a resident model with true KV-prompt reuse requested (`cache_prompt` stays on); validity requires processed + cached == requested. Cold remains a fresh runtime with full prefill.
- Verification after fix: `cargo test mt01` — a local protocol server serves prompt_n=512/cache_n=0 followed by prompt_n=1/cache_n=511 with 256 generated tokens each; both warm attempts succeed, observations persist processed and cached counts separately, and the summary counts both. Absent cache_n means "no cache accounting" (0) and is accepted; a present non-numeric cache_n is a protocol error; inconsistent totals and short generations stay rejected.
- Contract versioning (MT-01.I4): `cachedPromptTokens` is added with a serde default of 0 and no validation rule was removed, so existing schema-1 manifests remain valid and interpretable — the contract is extended, not silently changed; the exact generation-length check is untouched.
- Limits: MT-01.V3 (packaged b10816 warmup+five-trial run) belongs to V06-G-04/G-05.

### V06-MT-02 — public export allowlist (commit `9d4c36c`)

- Status: Implemented; unit-verified; no packaged-specific evidence required.
- Regression before fix (mutation proof): mutation S kept the private notes on sanitized evidence; `privacy_export_excludes_paths_prompts_credentials_and_raw_errors` then FAILED on the nested canary.
- Verification after fix: `cargo test privacy_export` — a mixed manifest (one success, one failure) with canaries in the raw failure, the shard path, hardware evidence detail/notes, effective-context notes, and process-memory notes exports through `build_share_bundle` with a recomputed summary; the complete serialized file contains none of the canaries, keeps counts and numeric statistics (decodeTps, failureCategories, successfulTrials/failedTrials), and the privacy review names the actual policy (rawErrors, evidenceSourceDetails, evidenceNotes). `mt02_direct_persistence_rejects_unsanitized_evidence_and_raw_failure_text` shows the same shape rules hold at `persist_share_bundle`: tampered notes and unbounded failure-category codes are refused and nothing is written.
- Policy (MT-02.I2): public evidence keeps value, level, source kind, and timestamp; free-form `source.detail` becomes an approved `public export source: <kind>` vocabulary and notes are dropped.
- Limits: local diagnostic preservation is unchanged (manifest/observations keep full internal detail locally); only the export representation is reduced.

### V06-MT-12 — bounded acquisition-time failure text (commit `9d4c36c`)

- Status: Implemented; unit-verified; regression output recorded here.
- Regression before fix (mutation proof): mutation T stored the raw oversize error; `mt12_oversize_failure_text_is_bounded_at_acquisition_with_a_visible_marker` then FAILED.
- Verification after fix: `cargo test mt12` — a cold-start failure string well past 4,096 bytes (including multibyte characters) is stored bounded with a visible `[message truncated` marker inside the limit, keeps its diagnostic category (Failed) and prefix, terminates a warmup failure run, and leaves the exactly-4096-byte boundary untouched. `mt12_successful_observations_survive_a_later_oversize_failure` keeps two successes and a valid summary around an oversize mid-run failure.
- Limits: "attach an explicit artifact reference/digest where available" (MT-12.I2) is satisfied by the log path that launch failures already embed in their message and by the marker's statement that the full runtime output remains in the local run log; there is no separate artifact object yet, so a future hardening could link the log file by digest.

### V06-RT-03 — cancellable managed-health completion (commit `1eaa476`)

- Status: Implemented; unit-verified; packaged cancelled-request evidence open (V06-G-04/G-05).
- Regression before fix (mutation proof): mutation Q removed the server termination from the supervisor's cancel branch; `rt03_cancel_terminates_a_pending_completion_within_the_bound` then FAILED.
- Verification after fix: `cargo test rt03` — an owned loopback fixture accepts `/completion` and withholds the response; cancelling after acceptance returns `Cancelled` in under five seconds with the terminate callback fired. A complete valid response arriving while the flag is set is still reported `Cancelled` (response-versus-cancel resolution), and the positive control proves an uncancelled request returns the exact body without terminating the server.
- Limits: the packaged Windows stalled-request scenario and child/listener/temp cleanup confirmation (V06-RT-03.V3) belong to V06-G-04/G-05; the worker thread stays blocked until the terminated server closes its socket, which is why terminate-before-report is the mechanism.

### V06-MT-03 — bounded proposal loop (commit `1eaa476`)

- Status: Implemented; unit-verified; the loop tests are deterministic and need no packaged evidence.
- Regression before fix (mutation proof): mutation N stopped counting no-op rejections; `mt03_noop_rejections_are_also_bounded_by_the_consecutive_rejection_limit` then FAILED. Mutation O disabled the three-field policy; `mt03_coercions_echoes_and_oversize_proposals_are_bounded_rejections` then FAILED.
- Verification after fix: `cargo test mt03` — an advisor that always proposes the existing `threads=-1` terminates exactly at the advisor-call budget (5 calls, six recorded rows, nothing measured beyond baseline), with every rejection reason preserved. Numeric-string coercion, `draftModel` echoes, and four-field proposals are recorded rejections; a different raw map that normalizes to an already-measured configuration is rejected as a duplicate; the overall deadline stops before the first paid call; cancellation during a no-op sequence forbids a further advisor call.
- Decision: the advertised three-fields-per-trial rule is retained as a hard policy and enforced in Rust (`MAX_CHANGED_FIELDS_PER_PROPOSAL`); the system prompt already states it.
- Limits: budget values (advisor calls = 2 × trials + 6; deadline 45 minutes; three consecutive rejections) are product defaults chosen this release; they are single constants and easy to tune later.

### V06-MT-04 — cancellable tuning lifecycle (commit `1eaa476`)

- Status: Implemented; unit-verified; packaged Windows cancellation scenarios open (V06-G-04/G-05).
- Regression before fix (mutation proof): mutation P restored the non-cancellable health wait in the live bench; `tuning_lifecycle_uses_cancellable_paths_and_reports_cleanup_failures` then FAILED.
- Verification after fix: `cargo test mt04` — a bench that flips the cancel flag during a candidate measurement returns an ordinary would-be failure; the loop ends terminal with "Cancelled by the user during a measurement" and records no candidate failure. The source guard pins the cancellable health wait, `benchmark_server_cancellable`, and the captured cleanup result with its surfaced failure message.
- Limits: V06-MT-04.V2 (injected process-cleanup failure) has no injection seam on `terminate_and_wait` and stays unchecked; the packaged stop-latency, surviving-child, port-release and next-operation checks (V06-MT-04.V3) are V06-G-04/G-05 evidence. `core::benchmark_server_cancellable` returns on a 250 ms boundary after Stop.

### V06-DC-04 — override browsing and download authority (commit `6be56e5`)

- Status: Implemented; unit-verified; packaged override-flow acceptance open (V06-G-04/G-05).
- Regression before fix (mutation proof): mutation M removed the provenance flags from `user_override_file`; `dc04_user_overrides_authorize_with_their_own_digest_and_only_when_marked` then FAILED.
- Verification after fix: `cargo test dc04` — a saved, validated override resolves through `resolve_catalog_download` with `authority == "user"` and its exact stored digest; clearing either the file or model provenance flag ends authorization; removal ends it; curated rows resolve as `curated` from the signed snapshot and always win; unknown names resolve nowhere. The browse path is pinned by the release gate (`models: catalogAllRows`, and the snapshot-only source is forbidden) so rows and facets come from one merged collection.
- Limits: the audit's controlled-server download (correct vs incorrect SHA-256 against a live server) is a packaged acceptance step in V06-G-04; the wrong-digest rejection itself already has downloader regressions. The add/edit form stays deferred by an explicit product decision recorded in the DC-04 scope note.

### V06-DC-05 — ownership and provenance boundaries for overrides (commit `6be56e5`)

- Status: Implemented; unit-verified; packaged mixed-origin acceptance covered under V06-G-04/G-05.
- Regression before fix (mutation proof): mutation I removed the curator-id rejection; `dc05_curator_id_collision_is_rejected_and_curated_rows_survive` then FAILED. Mutation J removed the filename-collision rejections; `dc05_filename_collisions_with_curated_or_other_user_rows_are_rejected` then FAILED. Mutation K disabled the mirror ownership guard; `dc05_refresh_never_relabels_or_steals_user_owned_files` then FAILED.
- Verification after fix: `cargo test dc05` — the three audit-executed SQL reproductions (same id/different repo; distinct ids/same case-insensitive filename; refresh after a collision) are application-level regressions: curated rows keep repo, provenance, and files; colliding saves are rejected with the owner named; curated refresh skips user-owned rows via the ON CONFLICT WHERE guard and returns per-file provenance (`CatalogFile.user_sourced`) so a user file can never be labeled curator-sourced.
- Limits: pre-existing databases produced by the audited build could still contain mixed rows; reads now surface per-file provenance, and recovery re-validates salvaged rows, but no automatic migration rewrites legacy mixed rows — they simply can no longer be created or extended.

### V06-DC-06 — transactional override replacement and full bounds (commit `6be56e5`)

- Status: Implemented; unit-verified; injected mid-statement fault variant recorded below.
- Regression before fix (mutation proof): mutation L stopped removing files omitted by an edit; `dc06_edit_replaces_the_complete_file_set` then FAILED.
- Verification after fix: `cargo test dc06` — two files to one and a.gguf to b.gguf return exactly the requested set; an invalid payload and a held write lock both leave the previous complete record intact and the same save succeeds after the lock releases; validation rejects long/duplicate tags, dates, quants, revisions, case-variant filenames, i64-overflow counts, and oversize serialized payloads before any mutation; the 200-row cap counts user rows only (editing at the cap stays allowed) and two concurrent new entries near the cap yield exactly one winner with the count landing exactly at the limit.
- Limits: a true mid-statement fault on the second INSERT is not injectable without adding a fault seam to production code; rollback is enforced by rusqlite transaction semantics (verified through the lock-failure and validation-failure preservation cases) and the same transactional pattern's mirror tests. Noted rather than faked.

### V06-DC-01 — catalog loading independent of refresh cooldown (commit `e6c7f59`)

- Status: Implemented; unit-verified; packaged restart acceptance open (V06-G-04/G-05).
- Regression before fix (mutation proof): mutation F trusted every cache record in `load_catalog_snapshot` instead of signature-verified records; `dc01_local_load_uses_bundled_data_for_missing_corrupt_or_untrusted_caches` then FAILED.
- Verification after fix: `cargo test dc01` — command-level test `dc01_local_load_publishes_state_inside_the_cooldown_and_authorizes_downloads` starts with a signed cache, a fresh persisted stamp, and an empty state slot; requires populated rows, the reported cooldown, no network request (the load path constructs no HTTP client), and a working `authorized_catalog_file` resolution. Clock rollback and corrupt/missing caches covered separately.
- Limits: no network request is verified structurally, not by request capture; the packaged close/restart sequence (V06-DC-01.V3) is part of V06-G-04/G-05. Repeated-Refresh throttling remains covered by `only_one_catalog_refresh_runs_at_a_time` plus the cooldown error path.

### V06-DC-03 — transactional mirror refresh and controlled recovery (commit `e6c7f59`)

- Status: Implemented; unit-verified; packaged open-handle acceptance open (V06-G-04/G-05).
- Regression before fix (mutation proof): mutation G rebuilt (recovered) inside the healthy migration branch of `fetch_model_catalog`; `dc03_fetch_mirrors_healthy_databases_and_only_recovers_after_migration_failure` then FAILED. The source guard names `mirror_verified_catalog` in the healthy arm and `recover_catalog_db_from_verified` only in the failure arm.
- Verification after fix: `cargo test catalog_db` — corrupt-file recovery quarantines the old database and reports; an unsupported newer schema preserves one user override through salvage; healthy refresh keeps user rows (`user_override_is_marked_and_survives_network_refresh`). Persistence failures surface through `CatalogSnapshot.persistence_notice`.
- Limits: held-lock and open-handle NTFS/SQLite sharing outcomes are recorded in the packaged application under V06-G-04/G-05; the injected-quarantine-failure case is covered by the recovery error path returning `Err` (the caller then reports the notice) without a dedicated rename-failure fixture.

### V06-DC-07 — one fallback path for candidate failures (commit `e6c7f59`)

- Status: Implemented; unit-verified; packaged fallback acceptance open (V06-G-04/G-05).
- Regression before fix (mutation proof): mutation H restored early `?` propagation on body-read failure; `dc07_truncated_invalid_utf8_and_malformed_signature_candidates_keep_the_cache` and `dc07_oversized_close_delimited_stream_keeps_the_cache_with_a_bounded_read` then FAILED.
- Verification after fix: `cargo test dc07` — a local HTTP fixture serves truncated bodies, invalid UTF-8, malformed signature bodies, and a close-delimited 4 MiB+1 stream; each keeps the seeded signed cache with an explicit `refresh_error`, and the served body never replaces the cache on disk. A positive control (`fetch_catalog_succeeds_end_to_end_against_a_served_signed_pair`) proves a valid served pair still produces a network snapshot. A validly signed unsupported schema (schema 99) is rejected through the injected-verifier seam and cannot replace the cache. `dc07_database_open_and_migration_failures_select_verified_rows` covers database-open and migration failure fallbacks.
- Limits: the future-schema case uses a test verifier seam, since the maintainer signing key is not available to tests; production always passes `verify_catalog_signature`.

### V06-RT-01 — launchable install destination (commit `bd33337`)

- Status: Implemented; unit-verified; packaged upgrade acceptance open (V06-G-05).
- Regression before fix (mutation proof): mutation A restored the audited `managed_runtime_root_in` fallback inside `runtime_install_roots_in`; `rt01_new_installs_publish_into_the_primary_root_even_with_only_a_legacy_directory` then FAILED. Mutation B restored location-based legacy rejection; `rt01_installation_result_passes_the_launch_trust_gate_and_reuse_in_both_roots` then FAILED.
- Verification after fix: `cargo test rt01` — 3 passed / 0 failed. Full suite: 405 passed / 0 failed (2026-09-11, Windows 11 26100, Rust 1.98.1).
- Limits: the packaged Windows upgrade scenario (V06-RT-01.V3) was not executed here; it is part of V06-G-05.

### V06-RT-02 — managed probe authorization (commit `4081a7f`)

- Status: Implemented; unit-verified; packaged workflow acceptance open (V06-G-05).
- Regression before fix (mutation proof): mutation C removed `guard(path)?` from `run_runtime_probe_with`; both sentinel tests FAILED because the replaced managed CLI/EXE actually executed (the negative control confirms the sentinel writes its marker when run).
- Verification after fix: `cargo test rt02` — 3 passed / 0 failed (rustc-built inert sentinels, real managed-root fixtures, no marker after rejection). Full suite: 408 passed / 0 failed (2026-09-11).
- Limits: packaged tamper-through-Tauri-workflow check remains for V06-G-05; V06-RT-02.V3's launch-path re-check is covered at unit level by the external/managed guard tests.

### V06-RT-04 — execution-identity lease (commit `8566883`)

- Status: Implemented; unit-verified; packaged writer-race check open (V06-G-05).
- Regression before fix (mutation proof): mutation E dropped the retained handle in `lease_verified_installation`; both rt04 tests FAILED because an external writer could then replace the approved EXE/CLI while the "lease" lived.
- Verification after fix: `cargo test rt04` — 2 passed / 0 failed (replacement blocked for EXE/CLI, after a preparation-like delay, clear repair rejection while leased, replacement succeeds after release, external runtimes produce no lease, health context carries the lease). Full suite: 412 passed / 0 failed (2026-09-11).
- Limits: the concurrent same-user writer race was exercised in-process, not across two real applications; the packaged variant stays in V06-G-05.

### V06-RT-07 — archive identity bound to extraction (commit `ad5bcd0`)

- Status: Implemented; unit-verified; symlink/reparse swap variant open (privileged packaged check).
- Regression before fix (mutation proof): mutation D restored the old path-based extraction; `rt07_archive_extraction_rejects_replaced_bytes_through_the_installer_path` FAILED (the replaced archive extracted).
- Verification after fix: `cargo test rt07` — 2 passed / 0 failed (same-size replacement, changed-size replacement, cancellation during verification, intact approved success path through `extract_zip`, Windows handle-blocks-replacement). Full suite: 409 passed / 0 failed (2026-09-11).
- Limits: the symlink/reparse replacement variant (V06-RT-07.V2, second half) requires symlink-creation privilege; it stays open for a privileged packaged check.

_Package 1 (RT-01, RT-02, RT-04, RT-07) implementation is complete at the unit/regression layer. The three open packaged items above are collected by the V06-G-05 target-environment gate and must not be counted as passed before that evidence exists._


### V06-GH-01 — pull-request checks isolated from the trusted runner (commit `fe6f4ea`)

- Status: Implemented I1/I2/I4; I3 owner-gated. `pr-check` runs on the ephemeral `windows-latest` runner with a read-only token and no secrets; every trusted push job carries `if: github.event_name == 'push'`.
- Regression before fix (mutation proof): mutation CA removed a push guard from a self-hosted job and the workflow-gate test failed. Mutation CB restored a tag-ref checkout and the one-revision test failed.
- Verification after fix: `node scripts/verify_workflow_gates.mjs` `{"ok":true}`; `node --test scripts/tests/release-gates.test.mjs` 100/100; AGENTS.md describes push/PR behavior, runner eligibility and the difference between job `needs:` ordering and repository merge enforcement.
- Residual limits: V1/V2 need a live benign PR on GitHub; I3 needs the repository main-branch ruleset (owner action, to be recorded with bypass actors in the 0.6 closeout).

### V06-GH-02 — one resolved release revision (commit `fe6f4ea`)

- Status: Implemented I1/I2/I4; I3 owner-gated. The `resolve` job pins one full SHA into every downstream checkout; the producer inventory is verified at publication with `verify_candidate_inventory.mjs --verify` and exact artifact hashes; retries of unchanged source keep one identity.
- Regression before fix (mutation proof): mutation CB (tag-ref checkout) failed the release-gate test; the inventory mismatch case is pinned by `release-gates` V2 cases.
- Verification after fix: workflow pins/gates pass; the resolve job also validates annotation/commit and event relationship before any checkout.
- Residual limits: V1/V3 read-back require a live tagged candidate; the tag-update/deletion ruleset is an owner action.

### V06-GH-03 — candidate-fed lifecycle sequencing (commit `69894e3`)

- Status: Implemented I1-I4. The clean-account lifecycle is a dependent release job that consumes the freshly built candidates through `-CandidateDir artifacts`; no job waits for public assets; the non-gating policy is documented and the summary reports the lifecycle status explicitly; the v0.5 ledger carries an additive factual correction.
- Regression before fix (mutation proof): mutation DA removed the package dependency from the lifecycle job and the release-gate test failed.
- Verification after fix: `verify_workflow_gates.mjs` ok; both sandbox PowerShell scripts pass the parser; the upgrade baseline is the `UPGRADE_BASELINE` matrix variable, not a hardcoded tag.
- Residual limits: V1-V3 need a live release run with the Sandbox feature available.

### V06-GH-04 — honest installer verdicts (commit `69894e3`)

- Status: Implemented I1/I2/I4; I3 open. MSI leftovers fail the verdict and require the product registration to be gone; install and update scenarios assert the installed executable version; the baseline is a matrix variable; eight-second survival is labeled a startup smoke.
- Regression before fix (mutation proof): mutation DB restored the warning-and-continue branch and the release gate failed.
- Verification after fix: script parse checks; version assertions recorded in per-step evidence.
- Residual limits: I3 (v0.4.1 profile/settings and v0.5 SQLite migration fixtures) stays open because no software-verified fixture schema exists yet; V2/V3 need the sandbox window.

### V06-GH-05 — packaged catalog/SQLite matrix (commit `5222307`)

- Status: Implemented and packaged-verified end to end. `scripts/verify_060_catalog.mjs` drives the shipped candidate across two launches in an isolated profile against a signed ed25519 fixture served from 127.0.0.1.
- Regression before fix (packaged): the matrix failed on the shipped rich-facet key mismatch (`rich.pipeline_tags` read while the Rust facet struct serializes camelCase `pipelineTags`) with the exact packaged crash `TypeError: Cannot read properties of undefined (reading 'map')`; the new jsdom regression reproduced the same failure before the fix. Mutation M1 rebuilt the pre-fix binary and `catalog.ui-rows` failed again with the same signature, then passed after restoring the fix.
- Verification after fix: local packaged run — first-fill 8/8 (local-first load, valid signed refresh, cooldown without a network request, rows render, search filter/clear, user-row persistence with provenance, cache + SQLite mirror on disk), restart 5/5 (rows without the fixture server, honest dead-endpoint error, invalid-signature fallback, mirror quarantine + rebuild, UI honesty); merged record bound to source `5222307` and portable sha256 `19b1e654…8bce`, overall PASS. The matrix is a dependent step in the release `package` job (workflow-gates required commands) and records are distinct from the retained runtime-catalog checks.
- Residual limits: the matrix rides the next tagged candidate in CI; keyboard-only flows are tracked under QD-02.V4.

### V06-GH-06 — retained lifecycle evidence (commit `69894e3`)

- Status: Implemented I1-I4. Failure paths write structured FAIL/TIMEOUT records with stage, release version, source revision, candidate digests and timing; the workflow uploads required evidence on every terminal outcome with `if-no-files-found: error`; the job summary distinguishes scenario status from artifact existence.
- Regression before fix (mutation proof): mutations DC (failure writer removed) and DD (static HOST_MATCH row restored) each failed their release gate.
- Residual limits: V1-V3 need the sandbox window.

### V06-GH-10 — derived host attestation (commit `69894e3`)

- Status: Implemented I1-I4; V1/V2 fixture-verified. `build_host_attestation.mjs` derives MATCH/NO_MATCH/UNKNOWN per row from the detected CPU/GPU; a definite mismatch fails the intended host-proof job; observed values are recorded separately from runner labels; the support policy keeps host presence distinct from packaged L4 qualification.
- Verification after fix: release-gate fixtures cover matching and mismatched CPU/GPU, missing GPU, invalid key, and no static HOST_MATCH rows remain.
- Residual limits: V3 (live host comparison) belongs to the next hardware window.

### V06-QD-02 — component and orchestration coverage (commits `b722f4d`, `5222307`)

- Status: Implemented I1/I2/I3/I5; I4 partial. The jsdom environment is scoped to `src/**` with only the IPC boundary mocked; temporary-database orchestration fixtures from the catalog packages cover fetch, mirror publication, restart, cooldown, offline fallback, corrupt migration and download authorization; curated/local collisions, atomic replacement and provenance are pinned; the packaged catalog acceptance is distinct from the runtime-catalog checks.
- Regression before fix (mutation proof): QD1/QD2/QD3 failed their component tests; QD4/QD5/QD6 failed their release gates.
- Verification after fix: `npm test` 10/10 component tests within 68/68 total; `cargo test` 473 lib tests; packaged matrix bound evidence.
- Residual limits: I4 (malformed saved profiles, denied browser storage) and V4 (keyboard/focus/labels) remain open.

### V06-QD-03 — truthful packaged verifier (commits `b722f4d`, `e44019c`)

- Status: Implemented I1-I5. Presentation scenarios live in jsdom component tests; `__reactFiber$`, `memoizedState`, hook-shape discovery and `queue.dispatch` are removed and gated; genuine IPC/install/tamper/health checks remain and are described separately; cancellation registers a `health-model-progress` listener and cancels after the first observed phase; release gates assert observable outcomes instead of private strings.
- Regression before fix (mutation proof): mutations QD4 (fiber walk restored) and QD5 (fixed 250 ms trigger restored) failed their gates; QD6 (component tests removed) failed the environment gate.
- Residual limits: V3 (fast/slow cancellation fixtures live) and V4 (live packaged run of the preserved checks) ride the next packaged verify with runtime assets.


### V06-MT-06 — one Rust-owned local client for TLS, keys, bounds and deadlines (commit `8c73679`)

- Status: Implemented I1-I4; V1/V2 fixture-verified; V3 packaged-runtime run pending.
- Regression before fix: the health, tokenization, benchmark and quality paths each hand-wrote cleartext HTTP with no Authorization header; a TLS- or key-configured server could be reported unhealthy or fail every measurement with no diagnostic path.
- Verification after fix: `src-tauri/src/local_client.rs` is the single client, built from the validated profile — the profile certificate is the explicit trust root (verification enabled), the API key is read only in Rust and redacted in Debug, request/response sizes are bounded, deadlines cover the whole operation, and cancellable calls observe the flag during connection and response waits. The startup `/health` probe and the `/props` effective-context probe use the same client; profile validation reads key/certificate files before launch and rejects unusable combinations precisely; managed TLS flags keep the hard capability rejection.
- Regression tests: 12 `local_client` tests over real TLS and plain fixtures (trusted certificate accepted, untrusted rejected, bearer header asserted, missing/empty/multiline key files, chunked JSON, oversized response, slow-writer deadline, cancellation during the wait, IPv6 bracketing, Debug redaction) plus the `core` pre-launch and capability tests; `cargo test` 487 passed / 0 failed / 2 ignored; `npm run check` EXIT 0.
- Mutation proofs: MC1 (Authorization header skipped), MC2 (profile certificate not trusted), MC3 (transport-file validation skipped) and MC4 (response size bound ignored) each failed the matching test.
- Residual limits: V3 needs the packaged target-runtime run of an accepted TLS/key profile; the health stage's internal server stays plaintext by construction (it launches its own loopback server) but still routes through the centralized client.

### V06-MT-07 — versioned canonical execution snapshot identity (commit `72bb60b`)

- Status: Implemented I1-I4; V1/V2 table-verified; V3 legacy/unknown paths verified.
- Regression before fix: the calibration compatibility key was a hand-maintained subset of launch fields; a changed thread count, fit-reduced context, or different LoRA bytes reused the same key, and unknown hardware identity could satisfy it.
- Verification after fix: `calibration::ExecutionSnapshotV2` (schema `localmotive.execution-snapshot.v2`) derives the identity from the effective launch arguments (secret values and paths replaced by `[configured]`/`[model]`/`[lora]` tokens), content hashes for model/draft/mmproj/LoRA payloads, runtime executables and help text, hardware and driver identities, workload, harness and estimator versions, and the observed effective context. `execution_snapshot_key` hashes the canonical JSON; keys carry the `v2:` prefix and `validate_compatibility_key` rejects legacy 64-hex keys at build, apply, persistence, load and replay. Unknown identities are recorded; `reuse_supported` refuses cross-run reuse and `build_calibration` reports them by name.
- Regression tests: `mt07_snapshot_key_changes_for_every_material_field` (table mutation across 21 material fields), `mt07_cpu_only_and_hardware_changes_are_distinguished` (CPU-only builds keys; same GPU + different CPU and fit-reduced effective context change the key), `mt07_unknown_identity_blocks_reuse_and_legacy_keys_stay_out`, `mt07_effective_arguments_are_sanitized_of_secrets_and_paths`. Mutations MD1 (unsanitized args) / MD2 (unknown gate bypassed) / MD3 (legacy keys accepted) / MD4 (material field dropped from the key) each failed their matching test and passed after restore.
- Commands: `cargo fmt --check` PASS; `cargo clippy --all-targets -- -D warnings` 0 errors; `cargo test --lib` 491 pass at the MT-07 gate.
- Residual: the snapshot records `host_cpu_model` as unobserved on this platform (no CPU-name collector yet); the reuse gate therefore treats driver/os identity as the blocking signals.

### V06-MT-08 — calibration anchors from unique persisted runs (commit `fc02ce6`)

- Status: Implemented I1-I4; V1/V2/V3 unit- and UI-verified.
- Regression before fix: Add anchor stored a click-stamped `Date.now()` copy of the frontend mean; three clicks on one benchmark manufactured three samples, and failed or cancelled runs were eligible whenever a summary existed.
- Verification after fix: `add_benchmark_calibration_anchor` reads the persisted manifest bounded (16 MiB), validates it, derives the measured value (`summarize_observations`), the observation time (last observation `started_at_ms`), and the source-run identity (`sha256` of the manifest bytes) in Rust; the estimator identity is explicit (`manual-estimate.v1`). One source run contributes at most one anchor per compatibility key: the record filename derives from the run identity, `persist_record` refuses a different anchor for the same run and treats byte-identical repeats as idempotent. `build_calibration` requires three distinct non-empty source runs and one estimator identity; failures, timeouts, cancellations and partial runs (fewer observations than planned trials) are ineligible; anchors carry the snapshot schema and unknowns from the manifest. The panel counts unique `sourceRunId` values for display and for the build gate, and documents the interval formula as a descriptive spread, not a confidence interval.
- Regression tests: `mt08_three_adds_on_one_run_keep_one_anchor_and_the_gate_closed`, `mt08_three_distinct_runs_build_and_keep_run_times` (observation times 1002/2002/3002 survive; reimport idempotent; three copies of one run cannot trip the gate), `mt08_failed_or_cancelled_runs_and_estimator_mixes_are_ineligible`, plus two jsdom component tests (`src/V03EvidencePanel.calibration.test.tsx`) that assert the command arguments, the unique-run count with duplicate-run records, and the build-gate enablement. Mutations ME1 (record identity guard removed) / ME2 (distinct-run gate removed) / ME3 (click-stamped observation time) / ME4 (anchor count instead of run count) each failed their matching test and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 494 pass / 0 fail / 2 ignored; `npm run check` PASS (tsc, vitest, catalog, branding, research anchor, qualification, icon, build `index-BAqaQx1B.js` 327.62 kB).
- Flake repaired on the way: `managed_runtime_listing_shows_local_installs_without_a_catalog_fetch` compared process-global hash counters and flaked under parallel load (observed 3 times). It now compares a thread-local mirror (`verification_bytes_hashed_this_thread`); the mutation that removes the mirror increment is caught; three consecutive full-suite runs are green (494/0).

### V06-MT-11 — requested-capacity objective and verified winner (commit `pending`)

- Status: Implemented I1-I4; V1/V3 unit-verified; V2 satisfied by the explicitly labelled retained short-prompt harness (the alternative accepted by the finding); long-prompt V2 measurement remains a documented limitation, not a claim.
- Regression before fix: tuning set `context` to the target but measured one short fixed prompt, never checked observed per-slot context before scoring, and crowned the highest mean of 1-5 repeats with no re-measurement; a parallel or fit-reduced candidate could win the requested-capacity objective.
- Verification after fix: `Bench` returns `TrialMeasurement { summary, command, effective_context }`; `run_tuning` rejects any candidate whose observed effective per-slot context is below `target_context` (or could not be observed) with a precise trial error, so it cannot win. The report carries the objective label ("short-prompt decode throughput … output quality and latency are not measured"), `required_effective_context`, per-trial `effective_context` and `std_dev`, `quality_affecting_changes` (cacheTypeK/cacheTypeV/specType winner changes) and a `final_verification` block: after the loop the baseline and finalist are re-measured, and the winner is reported only when it clears max(2×baseline drift, 3%). The panel shows the objective, verification row, quality note, and refuses to adopt an unconfirmed winner.
- Regression tests: `mt11_reduced_or_unobserved_effective_context_cannot_win`, `mt11_winner_needs_material_improvement_beyond_observed_variation`, `mt11_objective_label_and_quality_affecting_changes_are_reported` (3 tests, scripted bench). Mutations MF1 (context gate removed) / MF2 (material bar removed) / MF3 (quality flags dropped) each failed their matching test and passed after restore. Existing tune tests updated for the two verification measurements (21 tune tests green).
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 497 pass / 0 fail / 2 ignored; `npm run check` EXIT 0; `npx tsc --noEmit` EXIT 0.
- Residual: the retained harness still measures a short fixed prompt; the V2 long-prompt workload path is not wired into tuning, and the objective label states this explicitly. `specType` winner changes are reported, not quality-gated in-session (the quality suite stays user-run).

### V06-FE-06 — preflight staleness and adapter-selection intent (commit `6e3659d`)

- Status: Implemented I1-I5; V1/V2 component-verified.
- Regression before fix: the displayed preflight result stayed "current" after adapter/capacity/hardware/profile changes, the fingerprint omitted draft/projector, speculation, KV offload, fitting, device and attention fields, unchecking the final adapter was silently undone by the auto-select effect, and a slow preflight response could land after its inputs changed.
- Verification after fix: every preflight response stores the full input identity it was computed under (whole profile via `profileFingerprintOf`, model artifacts, adapter selection, manual capacity/note, hardware signature); the panel derives `preflightStale` and shows an explicit "Stale: … Re-run preflight" line until recalculation. Adapter defaults initialize once and a deliberate empty selection persists (`adaptersTouched`); refreshed hardware reconciles the selection by intersection and never repopulates it behind the user. Async inspection/preflight responses carry the input revision they were requested under and are discarded when it moved (`Preflight result discarded: …`).
- Regression tests: `src/V03EvidencePanel.preflight.test.tsx` (3 jsdom tests): staleness appears after an adapter change and clears on re-run with the new inputs; the deliberate empty selection persists through preflight and a hardware refresh; a deferred response released after an input change is discarded and not displayed. Mutations MG1 (stale marker disabled) / MG2 (auto-reselect overrides deliberate empty) / MG3 (revision discard removed) each failed their matching tests and passed after restore.
- Commands: `npm run check` EXIT 0 (74 vitest tests / 4 files, tsc, catalog, branding, research anchor, qualification, icon, build); `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 497 pass / 0 fail / 2 ignored.

### V06-MT-09 — quality evidence bound to full identity (commit `f495e69`)

- Status: Implemented I1-I4; V1/V2/V3 unit-verified (per-candidate history + labels).
- Regression before fix: a quality result carried only a model logical filename id and the runtime executable digest; tensor bytes, KV precision, speculation, LoRA, companions, and launch args could change while a structural pass stayed attachable, and the frontend merged a quality pass rate with at most two comparisons.
- Verification after fix: `ExecutionSnapshotV2` gained an explicit `scope` (`launch` vs `launch+workload`) that is part of the canonical key. Benchmarks persist `launch_compatibility_key` (launch-scope key of the same configuration). `run_quality_suite` captures model content digest, runtime digest and the launch-scope key BEFORE the requests; `QualitySuiteResult` carries `modelContentSha256`, `compatibilityKey`, `suiteVersion` (`structural-smoke.v1`) and `casesPlanned`. `recommend::quality_attachment_decision` refuses attachment unless the launch key, model content digest and runtime digest all match, and `candidate_from_manifest` is exposed through the new Rust `join_quality_candidate` command — the frontend `candidateFromBenchmark` merge was deleted (panel now joins in Rust and surfaces the refusal message). Share export requires the same launch key + content digest (and rejects suites without them). A 1.0 rate is labelled "both structural smoke cases passed" in the ranking message.
- Regression tests: `mt09_quality_joins_only_with_matching_identity` (matching attach; different launch key / changed tensor bytes / other runtime / anonymous suite all refused with distinct messages), `mt09_manifests_without_a_launch_identity_refuse_quality`, `mt09_suite_is_labelled_as_structural_smoke`, `mt09_share_export_requires_full_quality_identity`. Mutations MH1 (launch-key comparison skipped) / MH2 (content comparison skipped) / MH3 (anonymous suite accepted in export) each failed their matching test and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 501 pass / 0 fail / 2 ignored; `npm run check` EXIT 0; `npx tsc --noEmit` EXIT 0.
- Residual: the MT-05 operation-generation check covers stop/restart between quality cases; a stopped-and-restarted server between cases still discards results at the reservation boundary (existing MT-05 behavior, re-verified by its tests). The suite remains two structural cases; the label now says so everywhere it is scored.

### V06-MT-10 — consistent dominance and honest scores (commit `d1d718b`)

- Status: Implemented I1-I4; V1 verified; V2 (deterministic ties) covered in the same test.
- Regression before fix: pairwise dominance skipped objectives missing on either side, so different pairs compared different objective sets (the audit's A/B/C example gave A>B>C>A with no frontier member), and the preference score divided by only the weights a candidate filled in, rewarding omitted weak measurements. Ranges were rebuilt per candidate.
- Verification after fix: `dominates` requires every objective to have a value on both sides (a missing value makes the pair incomparable) and then applies strict Pareto dominance over that complete set — a partial order, so cycles cannot occur. Ranges are computed once from the feasible set. The score divides by the FULL configured weight sum and reports `evidenceCoverage`; dominator lists are capped at `MAX_DOMINATORS_REPORTED` (32) with a `dominatorsTruncated` flag; duplicate candidate ids are rejected.
- Regression tests: `mt10_missing_metrics_make_pairs_incomparable_not_cyclic` (the exact audit counterexample; all three candidates frontier members with empty dominator lists; a fully measured pair still dominates; deterministic re-run), `mt10_partial_coverage_cannot_inflate_a_score`, `mt10_duplicate_ids_are_rejected_and_dominators_are_bounded`, `mt10_ranking_scales_to_ten_thousand_candidates` (10,000 candidates: debug 12.8 s, release 1.88 s — recorded here as the measured worst-case bound). Mutations MI1 (skip-missing dominance restored) / MI2 (available-only denominator) / MI3 (dominator cap removed) each failed their matching test and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 505 pass / 0 fail / 2 ignored; `npm run check` EXIT 0; `npx tsc --noEmit` EXIT 0; release-profile measurement `cargo test --release --lib mt10_ranking_scales -- --nocapture` → `ranked 10000 candidates in 1.8770413s`.
- Residual: 10,000-candidate ranking stays O(n^2) by design (bounded detail, measured runtime); the panel shows evidence coverage per candidate, so a low-coverage candidate is visible even when it ranks high.

### V06-MT-15 — discovery reuses artifact shard analysis (commit `cf52adc`)

- Status: Implemented; V1 verified.
- Regression before fix: the scanner judged completeness from its own index set, tolerating duplicate indices (an unsplit `foo.gguf` beside `foo-00001-of-00001.gguf` could report complete) and silently dropping parse failures, while the artifact module rejected the same folders.
- Verification after fix: `scan_models` groups files as before but derives `complete` and an explicit `problems: Vec<ArtifactProblem>` from `artifact::analyze_shard_names` (new `MalformedShardName` code for analyzer refusals); `LogicalModel` carries the problems to the UI. Deterministic first-shard selection is unchanged.
- Regression tests: `mt15_duplicate_indices_and_unsplit_collisions_are_incomplete`, `mt15_malformed_and_inconsistent_shard_names_are_incomplete`, `mt15_extension_case_variation_and_valid_sets_stay_complete` — each case asserts discovery's decision equals `analyze_shard_names`' decision. Mutations MJ1 (scanner heuristic restored) and MJ2 (analyzer errors treated as complete) failed their matching tests and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 508 pass / 0 fail / 2 ignored; `npm run check` EXIT 0.

### V06-MT-14 — one calibration validator and evidence-based freshness (commit `50a4845`)

- Status: Implemented I1-I4; V1 verified.
- Regression before fix: build, persist/load and apply each enforced a different subset of invariants; a model could be applied before its creation time, expiry boundaries differed between frontend (`>`) and backend (`>=`), a rebuild refreshed arbitrarily old evidence, and unsupported estimators or inconsistent provenance were not checked.
- Verification after fix: `validate_calibration_model` is the single validator used by build, persist, load and apply (anchor count 3..=MAX, creation < expiry, TTL <= 365 days, finite positive factor and finite non-negative residual, supported estimator, unique non-empty source-run ids, evidence not postdating creation). `CalibrationModel` carries `sourceEvidenceAtMs` (newest anchor observation), `estimator` and `sourceRunIds`; apply rejects a creation time ahead of the clock, treats the expiry instant as expired, and refuses evidence older than 90 days even inside the nominal TTL. Interval bounds are checked for finiteness after arithmetic. `calibration_model_state` + the `evaluate_calibration_model` command share the same rules for display; the TS `calibrationState` mirrors them (scheduled / expired / staleEvidence) and the redundant state-level estimator branch was removed in favour of the validator.
- Regression tests: `mt14_apply_enforces_creation_expiry_and_evidence_freshness`, `mt14_one_validator_rejects_bad_models_at_every_entry_point` (inverted timestamps, overlong TTL, insufficient count, unsupported estimator, duplicate provenance, postdating evidence; persist rejects and a tampered on-disk record fails at load), `mt14_state_evaluation_matches_apply_semantics`; TS cases added for scheduled/staleEvidence/compatible boundaries. Mutations MK1 (freshness skipped) / MK2 (expiry boundary strict) / MK3 (validator estimator check disabled, re-aimed after the redundant copy was deleted) each failed their matching tests and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 511 pass / 0 fail / 2 ignored; `npm run check` EXIT 0; `npx tsc --noEmit` EXIT 0; `npx vitest run src/model.test.ts` 57 pass.

### V06-MT-13 — one attempt/workload contract at every boundary (commit `5104ce4`)

- Status: Implemented I1-I4; V1 verified via a deterministic malformed-record corpus at both boundaries.
- Regression before fix: manifest validation checked collection shapes but not trial identity, sequential numbering, successful token counts versus the workload, or the presence of all requested trials; the direct share-bundle validator was looser again (no workload validation, no attempt contract, no summary-count recompute, no quality status/case coherence), so "validate_complete" was not one shared contract.
- Verification after fix: `evidence::validate_attempt_consistency` (unique sequential positive trial ids, decode + generated tokens for successes, prompt/generated counts against a declared workload, cached <= prompt, error evidence for failures, terminal outcome equal to the last attempt, all requested trials present without a terminal outcome, attempts never exceeding the plan) is called from `BenchmarkManifest::validate_complete`; the direct share-bundle validator applies the same workload and attempt rules to `ShareObservation` records, recomputes trial counts against any supplied summary, and requires a quality status that agrees with its cases. The `mt08` anchor command now reaches its own eligibility checks only through the shared contract, and its empty/partial fixtures are raw JSON so the command's own refusals stay tested.
- Regression tests: `mt13_both_boundaries_refuse_the_same_malformed_records` (valid record accepted by both; duplicate ids, trial zero, sequence gap, zero-token success, token mismatch, missing trials, terminal mismatch, silent failure each refused by both with identical verdicts), `mt13_share_bundle_rejects_inconsistent_summary_and_quality`. Mutations ML1 (duplicate-trial check), ML2 (generated-count equality), ML3 (missing-trials rule) each failed the corpus and passed after restore. Fixture fallout from the stricter contract was repaired by declaring the measured workload in each fixture (measurement, mt08, mt09, sharing), not by weakening the rule.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 513 pass / 0 fail / 2 ignored; `npm run check` EXIT 0.
- Residual: the audit's "fuzz deserialization plus validators" is implemented as a deterministic malformed-record corpus at both boundaries, not a property-based fuzzer; `cargo-fuzz`-style coverage remains a deferred hardening item. Replay additionally rechecks command arguments and compatibility identity (unchanged from MT-07).

### V06-flake — dc02 request-count bound

- The `dc02_an_interrupted_no_range_transfer_retries_from_zero` test asserted an exact request count (3); a contention-driven extra probe request flaked once in the full parallel suite. The assert is now a lower bound with the audited property (restart-from-zero on the LAST request) pinned explicitly; two consecutive full-suite runs are green (513/0).

### V06-FE-17 — workload drafts validated by the tested contract (commit `090feaf`)

- Status: Implemented I1-I5; V1/V2 component-verified.
- Regression before fix: `defaultWorkload()`/`validateWorkload()` were imported by tests only; the panel kept a duplicate literal, dispatched directly, had partial minima and no maxima or integer enforcement, and `suggestedProfile` duplicated fit/companion defaults without a documented authority boundary.
- Verification after fix: the panel seeds from `defaultWorkload()` (the tested factory is the single default source), computes `validateWorkload(workload)` on every edit, renders field-level errors in a `role="alert"` list, disables Run v2 benchmark while any error exists, and refuses dispatch inside `executeBenchmark` with the joined field messages. `validateWorkload` now requires whole numbers (Rust deserializes u16/u32/u64) and the inputs expose complete maxima (1,048,576 / 65,536 / 10 / 100 / 3,600,000). `suggestedProfile` carries a doc-comment boundary statement: Rust owns companion ranking, flag validity and the preflight plan; the frontend factory is an editable preview only. A literal mirror of `evidence.rs Workload::default()` is pinned as a TS contract fixture.
- Regression tests: `mirrors the Rust workload defaults exactly (contract fixture)`, `requires whole numbers where Rust deserializes integers` (fractional, complete maxima, blank identity), and the panel test "shows field errors, blocks dispatch, and dispatches once the draft is valid" (fractional draft → errors visible, button disabled, no `benchmark_v2` call; valid draft → dispatch with the typed value). Mutations MM1 (dispatch gate removed) / MM2 (integer enforcement removed) / MM3 (field error list removed) each failed their matching tests and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 513 pass / 0 fail / 2 ignored; `npm run check` EXIT 0 (75 vitest tests / 4 files); `npx tsc --noEmit` EXIT 0.
- Residual (audit FE-17 I5): the frontend still previews fit/companion defaults; the documented authority is Rust (`rank_companions`, `validate_launch_profile`, `preflight_model`), and consolidation of the preview into a native endpoint remains a future product choice, recorded rather than silently assumed.

### V06-IPC-02 — narrow production CSP and scoped opener (commit `4973936`)

- Status: Implemented I1-I3; V1/V2 verified on the packaged binary.
- Regression before fix: `app.security.csp` was `null`, so the main WebView had no content policy at all, and the capability granted the unscoped `opener:default` permission set.
- Verification after fix: production policy `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self' ipc: http://ipc.localhost; object-src 'none'; base-uri 'self'; form-action 'none'; frame-ancestors 'none'` — the only inline exception is `style-src` (React style attributes; no user-controlled style strings), and there is no `unsafe-eval` anywhere. `devCsp` carries the Vite dev-server exception separately. The capability now grants `opener:allow-open-url` with an explicit origin scope (huggingface.co, github.com, the six provider consoles, tauri.app, react.dev, and `http://127.0.0.1/*` for the running server WebUI); all HTTP traffic stays in Rust (reqwest), so the WebView needs no network origins. Source scan positive control: zero `dangerouslySetInnerHTML`/`new Function`/`document.write`/`eval(` sinks in production sources.
- Packaged evidence: `npm run tauri build -- --no-bundle` BUILD_EXIT 0; `node scripts/verify_csp.mjs src-tauri/target/release/localmotive.exe` → `{"pass": true, "navigationTargets": 14, "openerAction": "clicked", "cspViolationsDuringNavigation": 0, "inlineScriptBlocked": true, "remoteFetchBlocked": true, "hostileTextInert": true}` — every nav screen exercised, the scoped opener still opens the repository link, an injected inline script and a remote fetch are both blocked (with the CSP violation reported), and hostile `<img onerror>`/`<script>` text stays inert. Release-gates contract test `IPC-02 production CSP and opener scope are narrow and audited` pins the config + capability + sink scan (101/101 gates pass).
- Residual: `scripts/verify_csp.mjs` runs from the release checklist; wiring it into `release.yml` as a required matrix step is a deferred hardening item (trigger: before the next tagged release).

### V06-OPS-01 — bounded per-run logs with retention (commit `090dcbc`)

- Status: Implemented I1-I3; V1/V2 unit-verified (quota, retention, collision and evidence survival).
- Regression before fix: every run wrote `server-{port}.log` / `tuning.log` with `File::create` (truncating the previous failure's log) and no size bound, so a chatty or repeatedly failing runtime could consume disk and destroy prior diagnostics.
- Verification after fix: new `log_sink` module. Each run creates `{prefix}-{millis}-{sequence}.log` with `create_new` (a collision or planted link fails creation instead of truncating); output is drained from pipes into a bounded sink (8 MiB per run, shared by the stdout and stderr writers via an atomic CAS quota; the drain keeps consuming past the quota so the child never blocks, with one truncation marker); retention keeps the newest 10 runs per prefix and 64 MiB total, and the newest 3 `*.log.failure.json` records survive even when their log is pruned. `launch_failure_evidence` now persists the bounded failure tail beside the run log. `stop_server` joins the drain threads after the child exits (bounded 2 s, then detach) so retained logs are complete. README documents location, caps, retention and the runtime-emitted-text caveat.
- Regression tests: `log_sink::tests::drain_caps_the_file_and_keeps_the_newest_lines` (full stream consumed, file at quota, marker present), `two_streams_share_one_quota`, `unique_run_ids_and_creation_refuse_collisions_and_links`, `pruning_bounds_the_directory_and_keeps_newest_failure_evidence`, `failure_evidence_survives_log_pruning`. Mutations MN1 (quota not enforced) / MN2 (retention disabled) / MN3 (collision truncation allowed) each failed their matching tests and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 518 pass / 0 fail / 2 ignored; `npm run check` EXIT 0.
- Residual: the UI log tail view is unchanged (still the last 16 KiB/12 lines); a per-run "export diagnostics" button remains future UX work and the failure JSON carries the same information on disk.

### V06-RT-09 — physical GPU identity for telemetry joins (commit `1d191c7`)

- Status: Implemented I1-I4; V1 unit-verified (identical-name ambiguity).
- Regression before fix: `nvidia-smi` rows carried only name/driver/memory, and the merge paired each row with the FIRST unmatched DXGI adapter of the same name — enumeration order decided identity, so two identically named GPUs could receive each other's usage/capacity under the wrong LUID.
- Verification after fix: the query now asks for `uuid` and `pci.bus_id`; a row joins only when its name identifies exactly ONE adapter and exactly ONE row. Otherwise every affected row is recorded as `UnassignedNvidiaObservation` (name, uuid, PCI location, values, reason) on `HardwareInfo.unassigned_nvidia` and no device-specific evidence attaches to any LUID; the UI shows an explicit "unassigned NVIDIA telemetry" note. A matched adapter records the NVIDIA UUID as `physical_id` evidence. Health selection already refuses ambiguous same-name adapter resolution; the refusal path is preserved.
- Regression tests: `rt09_identical_names_never_receive_another_devices_telemetry` (reversed orders + distinct usage), `rt09_one_adapter_with_duplicate_rows_stays_unassigned`, `rt09_distinct_names_map_each_row_to_its_own_adapter`, `rt09_probe_parser_reads_uuid_and_pci_location` (six- and four-field output). Mutations MO1 (order-based first-match restored) and MO2 (row-ambiguity guard bypassed) failed their matching tests and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 522 pass / 0 fail / 2 ignored; `npm run check` EXIT 0; `npx tsc --noEmit` EXIT 0.
- Residual: joining to a DXGI LUID remains name-based (LUID-to-PCI mapping needs SetupAPI work); ambiguity now stays visibly unknown instead of wrong. Two real identically named adapters were not available on this host, so the fixture evidence is unit-level.

### V06-CLD-01 — bounded OAuth callback contract (commit `5e22e4e`)

- Status: Implemented I1-I4; acceptance matrix unit-verified.
- Regression before fix: any request target with a nonempty `code=` was accepted (no method/path check, no decoding), the request line was read into an unbounded `String`, the deadline was only checked on `WouldBlock` accept, and the browser page said OPENROUTER CONNECTED before the exchange and credential write finished.
- Verification after fix: `parse_callback_request_line` accepts only `GET /callback` with exactly one non-empty percent-decoded `code` (<= 512 bytes); probes (favicon, other paths) get 404 and malformed/duplicate/oversize/undecodable callbacks get 400, each without ending the login. `read_bounded_request_line` caps the request line at 8 KiB and enforces the monotonic deadline on every chunk, so a byte trickle cannot extend the flow; the accept loop counts requests (`MAX_CALLBACK_REQUESTS = 32`) and stops a flood. Loopback binding, ephemeral ports and S256 PKCE are unchanged; no state parameter was added (the provider contract was not confirmed, and PKCE must not be replaced). The browser page now says CALLBACK RECEIVED and points back to the application until the exchange and Credential Manager write actually succeed; failures surface in the application.
- Regression tests: `cld01_request_line_contract_is_strict_and_decoded` (valid, percent-encoded, favicon, wrong path, POST, duplicate, empty, invalid escape, truncated escape, overlong code), `cld01_probes_and_bad_requests_do_not_end_the_login` (404/400s then the real code succeeds), `cld01_successive_probes_are_bounded_by_the_request_budget`, `cld01_a_byte_trickle_cannot_extend_the_overall_deadline` (bounded within 2.2 s against a 1.2 s budget), and the browser-page assertion in the loopback test. Mutations MP1 (read-time deadline removed, after the assert was tightened) / MP2 (any path accepted) / MP3 (duplicate codes take the first) failed their matching tests and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 526 pass / 0 fail / 2 ignored; `npm run check` EXIT 0.
- Residual: no live OpenRouter sign-in was performed (no real keys); the exchange/storage failure path was exercised through existing unit coverage, not a live provider flow.


### V06-DC-11 — cross-process reservation and no-replace publication

- Status: Implemented; acceptance matrix unit-verified.
- Regression before fix: the download renamed `{target}.part` over the final name, replacing any file that appeared while the transfer ran, two app instances could write the same target concurrently, and the publication never proved the published name referenced the verified bytes.
- Verification after fix: `reserve_download_target` creates `{target}.lm-lock` with `create_new` before the probe; a fresh lock refuses a second writer with an actionable message, a lock older than six hours is replaced once, and the lock is released on every exit path via RAII. `publish_verified_part` publishes with a no-replace hard link (`AlreadyExists` on conflict) and both files are preserved. The verified handle's identity (volume serial + file index via `GetFileInformationByHandle`) is captured before hashing and re-checked on the published name; a mismatch removes only the just-created link and publishes nothing.
- Regression tests: `dc11_a_fresh_lock_refuses_a_second_writer`, `dc11_a_conflicting_target_preserves_both_files`, `dc11_a_part_swapped_after_verification_is_not_published`, `dc11_an_unchanged_part_publishes_without_replacing`. Mutations MQ1 (reservation disabled) / MQ2 (fresh lock accepted) / MQ3 (overwrite on conflict) / MQ4 (identity check skipped) failed their matching tests and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 531 pass / 0 fail / 2 ignored; `npm run check` EXIT 0.
- Residual: the earlier pre-existing-target guard still fires first for mismatched targets (covered by `Err("...already exists but does not match...")` in the live-path probe); byte-range resume bookkeeping for replaced partials was not changed.

### V06-RT-08 — last-error capture before cleanup calls

- Status: Implemented (correctness by API contract + source guard); behavioral discrimination is not possible on this API.
- Regression before fix: `reject_managed_file_alternate_streams` read `GetLastError` after `FindClose`, and the `GetProcessMemoryInfo` failure branch read `io::Error::last_os_error()` after `CloseHandle`; both cleanup calls can clobber the thread's last-error value, so a genuine enumeration failure could be misreported as EOF (accepted) or vice versa.
- Verification after fix: the terminating status is captured immediately after `FindNextStreamW`, before `FindClose`; the memory failure status is captured immediately after `GetProcessMemoryInfo`, before `CloseHandle`. The existing ADS rejection test (`managed_runtime_verification_rejects_post_install_alternate_streams`) still passes, and the new guard test `rt08_last_error_is_captured_before_cleanup_calls` pins the capture order in the source.
- Mutation evidence: the behavioral reorder mutation MR1 (pre-fix order restored) passes every behavioral test — Windows does not guarantee a clobber, so no behavioral test can discriminate this defect. The source-order guard fails under MR1 (`0 passed; 1 failed`) and passes after restore; this is recorded as the honest discrimination limit for this finding.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 531 pass / 0 fail / 2 ignored; `npm run check` EXIT 0.

### V06-FE-08 — raw extra-argument field keeps typed separators (commit `bf77e69`)

- Status: Implemented; component + unit verified.
- Regression before fix: the controlled input rendered `extraArgs.join(" ")` and re-split the raw value on every keystroke, so typing `--flag ` immediately lost the separating space and the next token glued onto the first; quoted values were not representable.
- Verification after fix: the field keeps a raw draft string while focused (typing spaces and quotes is unconstrained), and the draft commits `parseExtraArgs(draft)` on blur or Enter. `parseExtraArgs`/`formatExtraArgs` in `model.ts` give a quote-aware tokenizer that round-trips values containing spaces; the help text now states the commit-on-leave behaviour and quoting.
- Regression tests: `parseExtraArgs (FE-08)` unit cases (separate tokens typed with spaces, quoted value with spaces, escaped quote, empty draft, round trip) and the component test `keeps typed separators while editing and commits the tokens on blur` (Inventory → Rescan with a fixture model → Profile → typing `--flash-attn ` retains the trailing space). Mutations MS1 (quotes ignored in the parser → 2 failures) and MS2 (the input tokenizes on every keystroke again → 1 failure) were caught and passed after restore.
- Commands: `tsc --noEmit` PASS; `npm test` 80 passed; `npm run build` PASS (`fe08-build.log`).

### V06-FE-09 — corrupt persisted records cannot crash the UI (commit `a6f6617`)

- Status: Implemented; unit + component verified.
- Regression before fix: `JSON.parse` on saved profiles and tuning reports was unguarded, a string `extraArgs` reached `.filter`, a non-array `trials` reached `.reduce` during render, and no React error boundary existed, so a corrupt record blanked the window.
- Verification after fix: reads go through `safeJsonParse`; `normalizeProfile` salvages `extraArgs` from arrays or raw strings; `normalizeTuningReport` validates the report shape (non-array `trials`, non-object `bestProfile` and non-numeric indices are rejected) and coerces per-trial fields; an unreadable or wrongly-shaped record is moved to `localmotive:quarantine:<key>:<time>` with a user-visible notice; `ErrorBoundary` in `main.tsx` renders a recovery screen whose reset clears application keys (quarantine records are kept) and reloads.
- Regression tests: `persisted record validation (FE-09)` (salvage cases, shape rejection, safe parse), `corrupt persisted records (audit FE-09)` component test (seeded `{not json` profile + `[]` tuning report → window renders, quarantine key exists, live key cleared, notice shown), `ErrorBoundary (audit FE-09)` (fallback text, reset clears `localmotive:model-root` and keeps quarantine). Mutations MT9a (quarantine skipped) / MT9b (tuning record trusted raw) / MT9c (extraArgs salvage removed) each failed their matching tests and passed after restore.
- Commands: `npm run check` EXIT 0 (tsc + vitest + branding; branding allowlist extended for the two migration lines); `npm run build` PASS; release-gates 101/101.

### V06-FE-11 — cancellation binds to the running download job (commit `002fe7b`)

- Status: Implemented; unit + component verified.
- Regression before fix: `cancel_download` received the currently edited `modelRoot` (the folder field stays editable while a job runs), the boolean result was ignored, failures were unhandled, and progress/evidence keys held only repo+filename, so a destination or revision change orphaned the running job's Cancel control.
- Verification after fix: `download_event_key(repo, filename, revision, destination)` in `lib.rs` (mirrored exactly by `downloadKey` in `model.ts`) is the job identity for both progress events and UI state; `startCatalogDownload` captures the revision and destination at start and records a `DownloadJob`; `cancelCatalogDownload(job)` addresses `job.destination` and surfaces the backend boolean ("not being downloaded" when false) and transport errors; the card resolves progress and the Cancel control through `activeDownloadJob`/`newestDownloadJob` over all jobs for the file, so editing the destination or switching the selected build no longer hides a running transfer.
- Regression tests: model tests (job key includes destination and revision; newest/active selection and no-job-to-cancel), Rust `fe11_download_job_identity_includes_destination_and_revision` + `fe11_progress_emits_use_the_job_identity_helper`, and the component test `keeps cancel bound to the running job after the destination is edited` (start job → edit destination on the live Inventory screen → Catalog still shows Keep & stop → cancel targets `C:/models/first`, false result message shown). Mutations MU1 (cancel uses the edited destination) / MU2 (boolean ignored) / MU3 (render keys to the edited destination) / MU4 (backend key drops revision+destination) all failed their matching tests and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 533 pass / 0 fail / 2 ignored; `npm run check` EXIT 0 (87 vitest tests).
- Residual: `inventoryHasFile` still matches by filename within the scanned inventory; membership reflects the scanned root, and backend verification remains the publication defense.


### V06-FE-12 — keyboard focus ring restored on path-bar inputs

- Status: Implemented; source-gated.
- Regression before fix: `.path-bar input` (specificity 0,1,1) set `outline: 0` and overrode the global `:focus-visible` ring (0,1,0) on the model-folder and download-destination fields, which also have `border: 0` — no visible keyboard focus indicator remained.
- Verification after fix: `.path-bar input:focus-visible` restores the documented amber outline (2px, offset 2px); the global rule and every other control are unchanged.
- Regression test: release-gates `FE-12 path-bar inputs keep a visible keyboard-focus ring` asserts the focus rule exists and that the outline reset still precedes it. Mutation MV4 (focus rule removed) fails the gate and passes after restore.
- Commands: `npm run check` EXIT 0; `npm run build` PASS; release-gates 102/102.
- Detector: the impeccable detector reports zero findings in the edited regions; its four findings (font-size 7px, #65502f, #c9eab8, side-tab border-left) are pre-existing at HEAD and are recorded as an out-of-scope residual for a dedicated design pass.

### V06-FE-13 — complete names and structures for assistive technology

- Status: Implemented; unit-verified (commit `297a50e` with FE-12).
- Regression before fix: the inventory advertised `role="table"` with plain spans and button rows (no columnheader/row/cell semantics), each N-gram pair placed two inputs inside one label, and provider tabs had no roving tabIndex, arrow-key handling, or tabpanel association.
- Verification after fix: the inventory is a real `<table>` with `th scope="col"` headers and a per-row `.row-target` button whose name is the model (keyboard focus and Enter select; rows keep pointer selection). Paired fields are four separate labels ("N-gram draft min", "N-gram draft max", "N-gram map size n", "N-gram map size m"), one input each. Provider tabs implement the WAI-ARIA tabs pattern: roving tabIndex, ArrowLeft/ArrowRight (wrapping), Home/End, focus follows selection, `aria-controls="provider-panel"`, and a linked `role="tabpanel"` with `aria-labelledby`.
- Regression tests: component tests `inventory is a real table with column headers and named row actions`, `paired numeric inputs each carry their own label`, `provider tabs follow the WAI-ARIA keyboard pattern with linked tabpanel` (aria-selected moves and focus lands on the newly selected tab). Mutations MV1 (roving tabindex removed) / MV2 (arrow handling removed) / MV3 (paired labels merged back) each failed their matching tests and passed after restore.
- Commands: `tsc --noEmit` PASS; `npm test` 90 passed; `npm run check` EXIT 0; `npm run build` PASS.
- Residual: the detail-panel (V03EvidencePanel) markup named in the original audit line list was not re-checked beyond the tested surfaces; the tabpanel id is shared by all providers by design (one visible panel).

### V06-FE-14 — compliant small-text contrast and narrow-viewport reflow

- Status: Implemented; packaged verification PASS.
- Regression before fix: four small-text colors failed 4.5:1 on their panels (#6f7974 3.42:1, #77817d 3.80:1, #7f8885 4.19:1, #737b78 4.24:1); evidence grids kept 205/240 px minimums and three-column tracks at every width; the eight-cell bottom navigation shared ~302 px at 320 px, below the project's 44 px rule.
- Verification after fix: the four colors are replaced by the `--muted` token (#9ca3a0: 5.99:1 / 5.94:1 / 7.15:1 on the audited panels); evidence/hardware grids use `minmax(min(100%, …), 1fr)` and collapse to one column at <=680 px; the narrow bottom navigation keeps 44 px targets and scrolls horizontally instead of shrinking.
- Regression tests: release-gates `FE-14 small-text colors keep at least 4.5:1 on their panels` (old values banned, ratios computed in-test) and the packaged probe `scripts/verify_responsive.mjs`: on the packaged binary at 320/375/680/980 px and 200%/400% page zoom there is no horizontal overflow (overflow <= 1 px) and every bottom-navigation cell is >= 44 px at 320/375. Build: `npm run tauri build -- --no-bundle` BUILD_EXIT 0; probe output `RESPONSIVE_PASS`.
- Commands: `npm run check` EXIT 0; `npm run build` PASS; release-gates 103/103.
- Residual: the 44 px target audit covered the bottom navigation only; other small links were not re-measured, and real hardware/DPI combinations beyond the emulated widths were not exercised.

### V06-FE-15 — status words state only what was established

- Status: Implemented; unit-verified (commit `4d9027e` with FE-14).
- Regression before fix: the loaded-profile tag said `VALID` from `selected.complete`; first-run steps said `Configured`/`Ready` from nonempty path strings; the speculation method list fell back to a hard-coded set with no marker when no runtime was inspected; editing either runtime path left stale capabilities visible; every evidence strong was green, including `Unknown`/`Blocked`.
- Verification after fix: the tag says `SHARDS COMPLETE` / `SHARDS INCOMPLETE`; the first-run steps say `Path selected` and `Ready to validate`; the speculation list carries a `Not inspected — this list is provisional…` note whenever no runtime is inspected; editing the executable path on either the Runtime screen or the Profile form clears the inspected capabilities until re-inspection; `evidenceTone` maps result words to tones (`tone-ok`/`tone-pending`/`tone-bad`) and the evidence cards use `--paper`/amber/red instead of unconditional green.
- Regression tests: `evidence tone (FE-15)` unit cases, panel test `does not paint unknown results green`, and the App test `says shards complete, path selected and not inspected instead of overstating` (scan → Control shows SHARDS COMPLETE and never `VALID`; Profile shows the provisional note; inspection removes it; editing the Profile path and the Runtime path each restore it; Runtime shows `Path selected` + `Ready to validate`). Mutations MW1 (all tones green) / MW2 (`VALID` restored) / MW4 (`Ready` restored) / MW5 (profile edit keeps stale capabilities) / MW6 (Runtime edit keeps stale capabilities) each failed their matching tests and passed after restore.
- Commands: `tsc --noEmit` PASS; `npm test` 93 passed; `npm run check` EXIT 0; `npm run build` PASS.
- Residual: the capability-gated availability of individual profile controls (offered vs flag-supported) was not reworked beyond the provisional marker; the Rust preview/start validation remains the enforcement point.

### V06-DC-09 — quant labels come from the file's own token (commit `7349a27`)

- Status: Implemented; extractor, builder, rendering and gates verified. The shipped data file's corrected labels land with the next signed catalog publication (see next action).
- Regression before fix: the builder took the last `-([A-Za-z0-9_]+).gguf` suffix as the quant, shipping 46+ rows labeled `MTP`/`mtp`/`imatrix`/`imat`/`it`/`0731`/`optimized`/`coding`, mixed case (`BF16`/`bf16`), and case-sensitive facet dedupe.
- Verification after fix: `scripts/lib/quant_label.mjs` extracts the file's own quantisation token (leftmost canonical token, `UD-` marker and `id_quant` glue handled, '.'/'-'/whitespace separators, canonical casing) and returns UNKNOWN when the name carries none. `build_catalog.mjs` labels through it and rejects unrecognised labels as curation problems. `catalog::facets` dedupes quant labels case-insensitively in canonical casing. `scripts/fix_catalog_quants.mjs` performs the one-time migration; on a copy of the shipped catalog it rewrote 175 of 1417 labels (evidence `.hermes-0.6/dc09-migration.txt`).
- Regression tests: release-gates `DC-09 quant labels come from the file's own token and stay canonical` (extractor cases: `IQ2_S-MTP`, `Q4_K_M-imatrix`, date suffix, `-it`, lowercase, dotted, `UD-` dynamic, combined base/draft, `E4B_q4_0` glue, unknown, quant-looking model name; builder must call the extractor and the suffix regex must not return) and the Rust test `dc09_quant_facets_dedupe_case_insensitively_and_keep_canonical_casing`. Mutations MY1 (suffix regex returns) / MY2 (last-token selection) / MY3 (case-sensitive dedupe) all failed their matching tests and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 534/0/2; `npm run check` EXIT 0; release-gates 104/104; `node scripts/validate_catalog.mjs .hermes-0.6/catalog-migrated.json --no-signature` -> `valid v2 (158 models, 1417 files)`.
- Next action (owner): run the `Publish curated catalog` workflow (workflow_dispatch). Its rebuild now produces the corrected labels and its sign job produces the matching `catalog/catalog.json.sig`; the checked-in data file stays untouched until then because the embedded-signature tests (`shipped_signature_verifies_after_crlf_checkout_normalization`) verify the committed pair and must stay green. Until the signed candidate is published, network catalog fetches fall back to the cached or bundled copy per DC-01/DC-07 behaviour.
- Residual: six `MTP`-suffixed rows were relabeled by token only; whether those files are standalone companions or full models containing MTP was not asserted (names are not evidence). The current served catalog keeps its signed legacy labels until the next signed publication.

### V06-DC-10 — immutable revision pins and signed freshness (commit `9cee654`)

- Status: Implemented; loader policy unit-verified; data-file fields land with the next signed catalog publication.
- Regression before fix: every shipped file omitted `revision` (defaulting to `main`), so an upstream replacement under `main` could silently change the bytes a signed catalog describes, and no signed sequence/expiry existed, so a compromised serving path could replay an older signed catalog.
- Verification after fix: the builder records the repository commit (`meta.sha`) as each file's `revision` (omitted only when the API reports none, keeping the loader default) and emits catalog-level `sequence` (build time, milliseconds) and `expires` (14 days, epoch seconds). The loader refuses a signature-valid catalog that is expired or whose sequence is older than the cached catalog's, with a named refresh error; equal sequences refresh idempotently; documents without the fields still load.
- Regression tests: `dc10_freshness_policy_prefers_accept_only_for_fresh_non_replayed_documents` (accept/rollback/expired matrix) and `dc10_expired_and_replayed_signed_catalogs_keep_the_cache` (expired -> cache, replayed -> cache, equal -> network, newer -> network, field-less -> network; verified through the injectable-verifier seam on a local HTTP fixture). release-gates `DC-10 the builder pins immutable revisions and signed freshness fields` guards the builder source. Mutations MZ1 (always accept) / MZ2 (rollback check removed) / MZ3 (revision dropped) / MZ4 (sequence dropped) all failed their matching tests and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 536/0/2; `npm run check` EXIT 0; release-gates 105/105.
- Next action (owner): the same `Publish curated catalog` run that resolves DC-09 also emits the revision/sequence/expires fields; the checked-in signed pair stays untouched until then so the embedded-signature tests remain green.
- Residual: replay protection compares against the locally cached catalog only; a client with an empty cache accepts any signature-valid document by design. The 26-hour refresh cooldown still governs how often the policy is exercised.

### V06-QD-01 — the catalog cutoff follows the build clock (commit `d44ceb8`)

- Status: Implemented; reproduction equivalent unit-verified.
- Regression before fix: `scripts/build_catalog.mjs` derived its recency threshold from the literal `new Date("2026-09-10T00:00:00Z")` minus the configured days, so every later build kept the June 12, 2026 threshold while emitting a current `updated` date and claiming `cutoffDays: 90`.
- Verification after fix: `catalogCutoff(now, cutoffDays)` in `scripts/lib/catalog_window.mjs` computes UTC-midnight `now - cutoffDays`; the builder calls it with the build clock. The audit's reproduction argument — identical configuration at two build times must produce two different thresholds — is asserted directly: 2026-09-10 builds a June 12, 2026 threshold and 2027-09-10 builds June 12, 2027.
- Regression tests: release-gates `QD-01 the catalog cutoff follows the build clock` (threshold matrix + builder source guards: the call site must exist and the frozen literal must not return). Mutations NA1 (frozen literal restored) and NA2 (day subtraction dropped in the lib) both failed the gate and passed after restore.
- Commands: release-gates 106/106; `node --check scripts/build_catalog.mjs` SYNTAX_OK.
- Next action (owner): the next `Publish curated catalog` run exercises the rolling window against live data; the checked-in catalog keeps its signed bytes until that publication.


### V06-QD-04 — active docs match the shipped policy and architecture

- Status: Implemented; source-gated (commit `7c3bc32` with QD-05/QD-06).
- Correction set: `docs/RUNTIME_MANAGER.md` marks signing optional/deferred with a pointer to the current release policy and labels version-specific contract numbers as historical; `docs/PRODUCT.md` states the Windows desktop platform (the `web` value is schema metadata), the active SQLite mirror (`catalog_db.rs`), and current-manifest version guidance; `docs/qualification-tests.md` carries a frozen-snapshot banner, the corrected `verify_versions.mjs` command, and the `docs/history/TODO-0.4.1.md` path; `catalog/README.md` states the fail-the-whole-build policy and the approved schema-2 rejection decision; `README.md` adds a storage matrix (signed cache, SQLite mirror and user rows, runtime installs, evidence, logs, quarantine) with lifecycle guidance; `AGENTS.md` maps the current module set (artifact, catalog_db, local_client, health, measurement, evidence, calibration, recommend, sharing, log_sink); `docs/history/TODO-0.5.md` gains a reading-order note without rewriting history.
- Regression tests: release-gates `QD-04 active docs agree with the shipped unsigned policy and current platform` (unsigned policy present, platform claim absent, SQLite mirror described, frozen banner present, stale command absent). Mutation NB2 (platform claims `web` again) failed the gate and passed after restore.
- Commands: `npm run check` EXIT 0; release-gates 110/110.

### V06-QD-05 — the toolchain minimum is declared and reproducible

- Status: Implemented; source-gated.
- Regression before fix: `README.md` said "Node.js 20" while the locked Vite requires `^20.19.0 || >=22.12.0`; no `engines`, `packageManager`, `rust-toolchain.toml` or `rust-version` existed, so builds and lint gates could drift with the toolchain.
- Verification after fix: `package.json` declares `engines.node` (`^20.19.0 || >=22.12.0`) and `packageManager` (`npm@11.18.0`); `rust-toolchain.toml` pins channel 1.98.1 with rustfmt and clippy components plus the update procedure; `src-tauri/Cargo.toml` mirrors `rust-version = "1.98"`; the README states the real minimum and the update procedure. CI keeps `npm ci` and `--locked`.
- Regression tests: release-gates `QD-05 the toolchain minimum is declared` (engines, packageManager, pinned channel, rust-version, README minimum). Mutation NB1 (engines removed) failed the gate and passed after restore.
- Commands: `npm run check` EXIT 0; `cargo test` 536/0/2 under the pinned toolchain; release-gates 110/110.

### V06-QD-06 — vendored upstream documentation carries provenance

- Status: Implemented; source-gated.
- Regression before fix: `docs/LLAMA-SERVER-README.md` opened without any source/commit/import identification and kept ten relative links that target the upstream llama.cpp repository layout; `docs/OPTION_MAP.md` called it its source without provenance.
- Verification after fix: the vendored README carries a provenance banner (upstream repository `ggml-org/llama.cpp`, import date 2026-09-06, upstream link resolution, and the policy that runtime `--help` is the authoritative capability list); `docs/OPTION_MAP.md` records the same source policy.
- Regression tests: release-gates `QD-06 vendored upstream docs carry provenance and links resolve elsewhere` (banner fields + OPTION_MAP note) and `QD-06 local doc links resolve (the vendored upstream README is excluded)` (README, OPTION_MAP, PRODUCT, RUNTIME_MANAGER relative links must resolve; the vendored file is excluded by design). Mutation NB3 (provenance banner removed) failed the gate and passed after restore.
- Commands: `npm run check` EXIT 0; release-gates 110/110.
- Residual: the vendored README's own internal links stay upstream-relative by design and are not rewritten.

### V06-GH-07 — support statements match public evidence

- Status: Repo-side corrections implemented; the v0.4.0 release-note edit is recorded as an owner action.
- Corrections: `README.md` now separates CPU packaged checks from accelerator coverage (the v0.5.0 health evidence is CPU only and its clean-account run failed before installation), points at the new `docs/EVIDENCE-MATRIX.md` (version -> host/backend/lifecycle with PASS/FAIL/UNKNOWN/NOT RUN cells, and the L2 mislabel correction for 0.4.0 recorded), and replaces the night-only runner claim with the current self-hosted reality; the `hardware-qualify.yml` header and summary state the same. `AGENTS.md` no longer claims a push/PR CI model (that section was already updated for pr-check in GH-01's change set).
- Regression tests: release-gates `GH-07 the evidence matrix exists and the README reads it` (matrix columns, README pointer, no accelerator inference, stale claims absent from README and the workflow summary). Mutation NC1 (night-window claim restored) failed the gate and passed after restore.
- Owner action: publish the factual correction on the v0.4.0 release body (external change; the repository-side correction document stays as drafted). Binaries stay immutable.
- Commands: `npm run check` EXIT 0; release-gates 113/113.

### V06-GH-08 — supply-chain posture documented; SBOM generated per release

- Status: Repo-side posture documented and the SBOM step implemented; attestation remains an owner decision.
- Verification: `docs/SUPPLY-CHAIN.md` records protections (SHA pins, lockfiles, pinned toolchain, one-shot tag resolution, checksums + readback, SBOM artifact, catalog-key isolation) and accepted limitations (unsigned installers, no SLSA attestation yet, shared build host, retention, and that the catalog signature does not authenticate installers). The release workflow's package job now generates `npm sbom --sbom-format cyclonedx` (proven locally: CycloneDX, 145 components) and retains it as the 90-day `sbom-<version>` artifact without changing the published asset set.
- Regression tests: release-gates `GH-08 supply-chain posture and SBOM step are documented` (posture statements + SBOM step + versioned artifact name). Mutation NC2 (SBOM step removed) failed the gate and passed after restore.
- Owner decision (recorded, not landed): add `actions/attest-build-provenance` with `id-token`/`attestations` permissions and a new pinned action SHA; it requires a real release run to verify, so it was not landed unverified in the release workflow.
- Commands: `npm run check` EXIT 0; release-gates 113/113; workflow gates and pin verifiers PASS.

### V06-GH-09 — governance and maintenance files exist

- Status: Implemented; source-gated.
- Files: `SECURITY.md` (private reporting path, defect classes the project treats as security-relevant, scope, best-effort expectations, no bounty, release-integrity statement), `CONTRIBUTING.md` (setup, the full check suite, PR process, and maintainer maintenance/recovery notes: runner isolation, catalog key rotation procedure, retention, backups), `.github/ISSUE_TEMPLATE/bug_report.yml` + `feature_request.yml` + `config.yml`, and `.github/dependabot.yml` (weekly grouped updates for npm, cargo, and github-actions).
- Regression tests: release-gates `GH-09 governance files exist and dependency updates are configured` (reporting path, no-bounty expectation, check suite text, recovery guidance, templates present, all three ecosystems covered). Mutation NC3 (cargo coverage removed from dependabot) failed the gate and passed after restore.
- Residual: GitHub-side settings (secret scanning, alert enablement) were not assessed and are not claimed; scheduled dependency audits beyond Dependabot updates remain optional for this project size.

#### G-01 closure record

- Adopted the audit and this tracker into the repository at `docs/history/` and pointed `AGENTS.md` (v0.6 section, line 135) at the tracker and audit as the authoritative pair (V06-G-01.I3).
- V06-G-01.I1: the audited snapshot is `e530371b056cd8e049c2246dbb151aa407bf359f`; the v0.6 start commit was recorded when the tracker was adopted (see the adoption record above), and every finding package in this document carries its own pre/post-fix evidence.
- V06-G-01.I2: all 72 finding IDs with priorities 19/43/10 are preserved; FE-10 is consolidated into DC-04 as planned; supplemental scope lives under "Supplemental audit recommendations". `node scripts/check_tracker.mjs` reports `72 findings (19/43/10), 111 packages, 663 checkboxes, 1051 audit links`.
- V06-G-01.V1: the checker validates every checkbox ID, package shape and audit-anchor count on each run; it is invoked in this document's closure workflow and passes at the current HEAD (`d1de988`).
- Historical audit results were used only as the defect source; no historical test result was reused as a v0.6 pass.

#### G-02 closure record

- Every closed finding's ledger record names its regression test(s), the failing-before behaviour (the audit's confirmed path), the mutation checks with their outcomes, and the exact gate commands with observed counts.
- V06-G-02.V1: mutation/negative controls were run for each fix; two source-guard exceptions are recorded with their honest limits (RT-08's Win32 ordering cannot be behaviorally discriminated, and the no-raw-command/worker guards are structural by design). Both are labelled as such in their closure records instead of being presented as behavioral proof.
- Failed mutation attempts were re-run with corrected mutations and the outcome recorded (for example CLD-01 MP1 and DC-11 MQ1/MQ4, whose first attempts did not discriminate).
- Platform checks that remain open (packaged Windows lifecycle, accelerator qualification) are recorded as open items in G-04/G-05/G-06 rather than being counted as passes.

#### G-03 closure record — candidate checks and build at one immutable SHA

- Candidate source SHA: `065248a144757479ebd370f67606e38b5ac8428c` (version 0.6.0 in `package.json`, `package-lock.json` incl. `packages['']`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`; `verify_versions.mjs 0.6.0` -> `"ok": true`).
- Environment: Windows 11 26100, Node v24.11.0, npm 11.18.0, rustc/cargo 1.98.1 (pinned by `rust-toolchain.toml`), MSVC link.exe from VS 18 Insiders. Logs: `.hermes-0.6/g03-candidate-checks.log`, `.hermes-0.6/g03-node-checks.log`, `.hermes-0.6/g03-final-checks.log`, `.hermes-0.6/g03-candidate-build.log`.
- I1: toolchain versions and lock identities recorded; the lockfile root versions were updated to 0.6.0 (`package-lock.json` root + `packages['']`, `Cargo.lock` root).
- I2: `npm ci` OK; `npm run check` EXIT 0; `npm audit --audit-level=moderate` -> `found 0 vulnerabilities`; `cargo fmt --check` PASS; `cargo clippy --locked --all-targets -- -D warnings` -> 0 errors; `cargo test --locked` -> 536 passed / 0 failed / 2 ignored; `RUSTDOCFLAGS=-D warnings cargo test --locked --doc` -> 0 failed. `cargo audit` (cargo-audit 0.22.2): 1243 advisories loaded from the advisory DB, 566 crate dependencies scanned, 0 vulnerabilities, 7 allowed warnings (proc-macro-error/glib-class informational warnings; no shipped-target vulnerability).
- I3: gates at the same SHA: versions `ok`; branding PASS; `verify_workflow_gates.mjs` and `verify_workflow_pins.mjs` zero failures; catalog signature validation without `--no-signature` -> `catalog: valid v2 (158 models, 1417 files)`; icons gate PASS (release-gates suite covers it); `release-gates.test.mjs` 113/113; `check_tracker.mjs` -> 72 findings (19/43/10), 663 checkboxes, 1051 audit links.
- I4: the nonpublishing candidate was built exactly once at `065248a1` with `npm run tauri build`: portable `localmotive.exe` 20583936 bytes sha256 `0322fbd2eb4bed2c4e434b98e9065e9528103bc53647a52237ac622a226a7f29`; MSI 8957952 bytes sha256 `9c3684e73c8582ddf6899a4e0d7b6fb751ae716db88d815dc45c518f7f5c4695`; NSIS 5230965 bytes sha256 `78c8e44afc942aa5d109a231aeb9d58f487efd31a9a5ff0435767f6c3057c217`. The repository's own `verify_candidate_inventory.mjs` generated `candidate-inventory.json` (schema 1, sourceRevision `065248a1...`, 3 artifacts) over a release-shaped staging directory `.hermes-0.6/candidate-0.6.0/` -> `PASS candidate inventory: 3 artifacts`. G-05/G-06 consume these exact bytes; G-09 promotes them without rebuilding.
- Skipped with impact: none at this layer. Accelerator runtime qualification is not part of G-03 and remains open in G-05.

#### G-04 closure record — minimum regression matrix

- All 14 matrix rows now carry an observed result with its layer label (Rust/native, fixture, component/IPC, packaged) and the Finding trace; the table above is authoritative for details.
- Packaged evidence produced against the exact candidate portable `Localmotive_0.6.0_x64-portable.exe` (sha256 `0322fbd2eb4bed2c4e434b98e9065e9528103bc53647a52237ac622a226a7f29`, built at `065248a1`):
  - `node scripts/verify_060_catalog.mjs` full matrix -> `overall_status: PASS` (first-fill + restart), including `restart.rows-without-network`, `restart.honest-refresh-error`, `restart.invalid-signature-fallback`, `restart.corrupt-mirror-recovery` (SQLite quarantine + rebuild) and `restart.ui-honesty`; log `.hermes-0.6/g04-catalog-matrix.log` (recorded source_revision `896d87b`, a docs-only commit after the build; the artifact digest binds the binary to `065248a1`).
  - `node scripts/verify_csp.mjs` -> 0 CSP violations across all screens, inline script blocked, remote fetch blocked, hostile text inert, scoped opener works.
  - `node scripts/verify_responsive.mjs` -> `RESPONSIVE_PASS` at 320/375/680/980 px and 200%/400% zoom, no overflow, >=44 px navigation targets.
- Rows left open by environment, carried explicitly: M07 (approved b10816 runtime not installed on this host — no `%LOCALAPPDATA%\Localmotive\runtimes`; unblock: install the approved runtime through the app and run the warmup + five-trial scenario), M04's packaged >8 MiB transfer, M05/M08/M09 packaged cancellation/tamper/interleaving fixtures (G-05.I1), M06 packaged legacy upgrade (G-06.I2), M10/M14 packaged navigation and keyboard/screen-reader passes (G-05.I3).
- The complete Rust suite (536/0/2) and the 113 release-gate tests were re-run at `065248a1` in G-03; no synthetic rendering result is substituted for real operation evidence anywhere in the table.

#### G-05 status record — target-environment checks (partial, candidate-bound)

Candidate portable sha256 `0322fbd2eb4bed2c4e434b98e9065e9528103bc53647a52237ac622a226a7f29` (built at `065248a1`).

DONE at the packaged layer:
- SQLite locking/recovery: `verify_060_catalog.mjs` restart phase PASS (`restart.corrupt-mirror-recovery` with quarantine + rebuild from verified bytes; `restart.invalid-signature-fallback`; `restart.honest-refresh-error`; `restart.rows-without-network`; `restart.ui-honesty`). Log `.hermes-0.6/g04-catalog-matrix.log`.
- Keyboard traversal + focus visibility: `verify_a11y.mjs` A11Y_PASS — 16 distinct controls reached by Tab, all 16 with a visible 2px outline (the FE-12 defect class is absent in the packaged build).
- Reduced motion: honored (emulation applied; stylesheet rule present).
- Zoom/layout: `verify_responsive.mjs` RESPONSIVE_PASS at 320/375/680/980 px + 200%/400% zoom.
- Window policy: `verify_csp.mjs` PASS (0 violations across screens).

OPEN, with the exact unblock action (each remains a visible limitation, not a pass):
- Managed tamper rejection and Job Object cleanup (I1): require a managed runtime install. Unblock: install the approved runtime through the app, replace a managed file, run any managed probe (expect refusal) and a server start/stop (expect no orphaned processes).
- Active cancellation (I1): requires a real UI transfer. Unblock: download a catalog model to an empty folder and cancel mid-transfer; record cleanup.
- NTFS rename/open-handle/hard-link through packaged routes (I1): the guards live on background install/download paths exercised by real-NTFS unit tests on this host; no packaged UI route exists to click. Recorded as a harness gap, not claimed as packaged evidence.
- TLS/auth local profiles + default warm benchmark against the approved runtime (I2): no managed runtime installed (`%LOCALAPPDATA%\Localmotive` has cache/ and health-models/ only). Unblock: install the approved b10816 runtime, run warmup + five trials.
- Identical-GPU mapping (I2): environment-blocked (single GPU); stays visible as a limitation.
- Narrator/NVDA and forced-colors passes (I3): NOT RUN — no screen-reader or forced-colors environment available here.
- Live cloud/HF credential scenarios (I3): excluded by authorization (no test account provided).


#### G-06 status record — installer lifecycle in Windows Sandbox (candidate-bound)

- Candidate bytes (release-shaped names, sha256): `Localmotive_0.6.0_x64-portable.exe` `0322fbd2eb4bed2c4e434b98e9065e9528103bc53647a52237ac622a226a7f29`; `Localmotive_0.6.0_x64.msi` `9c3684e73c8582ddf6899a4e0d7b6fb751ae716db88d815dc45c518f7f5c4695`; `Localmotive_0.6.0_x64-setup.exe` `78c8e44afc942aa5d109a231aeb9d58f487efd31a9a5ff0435767f6c3057c217`. The repository inventory tool generated + validated `candidate-inventory.json` over these exact files.
- Harness: `scripts/sandbox/host-run-lifecycle.ps1` with `-CandidateDir` (candidate consumption; no published-asset wait). Fixed during this run: an `if` statement used as an expression inside a hashtable literal failed under Windows PowerShell 5.1 (the CI runs pwsh 7, where it parsed); the harness now precomputes the value and parses under both. Local environment note: this host's PSModulePath lists PowerShell 7 modules before the 5.1 ones, so the local run set `$env:PSModulePath` to the 5.1 directory inside the session; CI is unaffected.
- Run 1 (`PreviousTag v0.4.0`): `status: PASS` — NSIS fresh install/launch/uninstall (uninstall removed the executable), MSI fresh (installed version 0.6.0; uninstall removed the executable and the product registration), NSIS update v0.4.0 -> 0.6.0 with executable version evidence. Evidence: `release-evidence/0.6.0/attestations/sandbox-clean-account-lifecycle-upgrade-from-v0.4.0.json`; log `.hermes-0.6/g06-sandbox-lifecycle.log`.
- Run 2 (`PreviousTag v0.5.0`): `status: PASS` — same three steps with `executable version 0.5.0 -> 0.6.0`. Evidence: `release-evidence/0.6.0/attestations/sandbox-clean-account-lifecycle.json` and `...-upgrade-from-v0.5.0.json`; log `.hermes-0.6/g06-sandbox-lifecycle-v050.log`.
- Negative controls (V1) remain the release-gates fixtures: leftover installed executable, unchanged upgrade version, wrong SHA, missing artifact and missing lifecycle evidence each produce failure or explicit non-pass (113/113 at the candidate SHA).
- OPEN (I2): persisted-profile migration (v0.4.1 -> 0.6) and SQLite/user-override preservation (v0.5.0 -> 0.6) are NOT exercised — the harness's own `coverageNote` states this. Unblock: extend `run-lifecycle-in-sandbox.ps1` to seed `localmotive:*` webview records and a `catalog-mirror.sqlite` with a user row before the upgrade step, then assert their survival after launch. The eight-second process survival is a startup smoke, not full functional verification, and is recorded as such.

#### G-07 closure record — positive controls revalidated

Revalidated at the candidate source (`065248a1` + the G-07 test addition) on Windows 11 26100:

- Runtime approval and extraction controls: `rt02` guard-policy + sentinel routes (3 PASS), `rt04` execution-lease (2 PASS), `rt07` archive identity (2 PASS); manifest controls 30 PASS (modified payload, missing/extra entries, approved-manifest membership) and archive handling 11 PASS (NTFS ADS + extraction refusals). Hidden contained processes and readiness listeners were exercised by the health/startup suites in the full run.
- Catalog authority: signature tests 3 PASS (including `shipped_signature_verifies_after_crlf_checkout_normalization` against the embedded maintainer key) and the invalid-signature fallback in the packaged matrix; mandatory SHA-256 and final-file verification are covered by the DC-02/DC-11 download suites; signed-snapshot authority by the DC-01/DC-07 suites (no user-metadata promotion path).
- Cloud and credential controls: cloud suite 12 PASS (Credential Manager storage, masked status, fixed provider endpoints, OAuth contract), tune whitelist 2 PASS, sharing/export 9 PASS with explicit confirmation and canary rejection.
- Redirected secret headers (the audit's line-692 observation, now demonstrated rather than assumed): new regression `g07_an_authenticated_redirect_does_not_forward_the_token_to_another_host` — the first request carries the bearer token, the cross-host redirect target does not, and both requests are observed by a real loopback fixture.
- Support labels: the repo-side evidence matrix (`docs/EVIDENCE-MATRIX.md`) binds each claimed support cell to release evidence; no accelerator or lifecycle label is claimed beyond its recorded run.
- Commands: `cargo clippy --all-targets -- -D warnings` 0 errors; `cargo test` 537 passed / 0 failed / 2 ignored (536 + the new redirect regression). Full-suite and targeted group results in this record; earlier candidate-run logs remain authoritative for G-03.

#### S-01 closure record — honest cleanup and overflow guarantees

- I1: `terminate_and_wait` now runs a supervised loop against a monotonic `TERMINATION_DEADLINE` (10 s, 50 ms polling) and returns `false` when the exit is not observed; the pre-fix code blocked in `wait()` after the kill call, so no caller's deadline could ever mean what it said. Callers that already branch on the boolean (managed-server stop paths in `lib.rs`) now report truthfully; `Drop` and the timeout paths keep the same call.
- I2: the overflow policy is documented at the sink: crossing the quota truncates **retained output only** — the child is never terminated for output volume, the drain keeps reading so a full pipe can never block it, and the truncation marker is written once. Covered behaviorally by the OPS-01 drain tests (input fully consumed, retained bytes bounded).
- I3: `no_module_constructs_a_raw_command` now discovers every `src/*.rs` module at runtime instead of a fixed list (>= 15 modules asserted), allows exactly the two known production uses in `proc.rs` (`hidden_command`; the empty placeholder swap) and fails on `Command::new(` anywhere else. The descendant containment tests (`proc::tests::cancellation_terminates_descendant_processes`, `health::tests::cancellation_job_terminates_descendant_processes`) already exercised grandchild cleanup and stay green.
- V1: fixtures exercised = descendant/grandchild kill (existing tests), pipe-retaining/excessive-output behavior (drain + quota tests), exit-during-cleanup (the health exit re-check from MT-06). Mutation ND1 (a raw `Command::new(` injected into `core.rs`) failed the discovery test and passed after restore; the bounded-wait normal path is observable-equivalent to the old blocking path, so its deadline branch is defensive by construction and recorded as such.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 537 passed / 0 failed / 2 ignored (536 + the discovery-scan change; the proc suite moved 18 -> 19).
- Windows containment is claimed only for the Windows Job Object path; the non-Windows fallback (`child.kill(); wait()`, now also deadline-bounded) makes no containment claim.

#### S-02 closure record — artifact identity levels and split metadata

- I1: `docs/ARTIFACT-IDENTITY.md` defines four levels (logical/header, named artifact-set, recorded split metadata, content digest), states what each level does NOT cover, and reviews the consumers (`catalog_db`, `calibration`/`evidence`/`sharing`, `measurement`, `core` inventory). The documented scope limit: a model edited in place with the same name and header facts keeps its configuration identity; content identity is enforced where bytes enter the machine (download/managed install) and re-checked before managed execution.
- I2: `gguf.rs` reads optional `split.no`/`split.count` metadata into new `splitNo`/`splitCount` summary fields (absent keys stay `None`); `artifact.rs::split_metadata_verdict` cross-checks recorded metadata against the filename plan (zero-based `split.no` converted to the one-based filename index) and returns `Agree`/`Mismatch`/`Unknown` — absence never upgrades completeness to a claim. `model.ts` mirrors the optional fields.
- V1: fixtures used: same-size header with/without split metadata (the `fixture_with_split` variant differs from `fixture` by exactly two keys and the kv count), inconsistent split metadata (count mismatch, index mismatch, half-recorded pairs), and the existing MT-15/DC-11 suites cover renamed-identical-bytes (digest) and reordered-companion (deterministic ordering) claims at their levels.
- Mutations: NS1 (split keys no longer read) and NS2 (verdict accepts any recorded metadata) each failed their matching tests and passed after restore, verified in isolation.
- Commands: `cargo fmt --check` PASS; clippy included in the full gate; `cargo test` 539 passed / 0 failed / 2 ignored (537 after S-01 + the two new S-02 tests); `tsc --noEmit` PASS.

#### S-03 closure record — impossible preflight dimensions never look authoritative

- I1: `estimate_kv_cache_bytes` now rejects zero and implausible (> 2^20) values for block_count, head_count_kv, key_length and value_length BEFORE arithmetic, naming each offending term in the Unknown evidence notes. The u128 checked chain and the explicit overflow note already existed and are unchanged; a zero anywhere previously produced a small, authoritative-looking `Derived` number.
- I2: already-honest levels re-verified in the same function: unsupported architectures, quantized/unknown cache element widths, recurrent/hybrid layouts and multi-device placements stay Unknown; weight bytes remain a `Heuristic` proxy with the note "File bytes do not prove final runtime allocation"; per-adapter placements stay Unknown until a validated runtime measurement.
- V1: `s03_impossible_kv_dimensions_are_rejected_and_overflow_stays_unknown` covers zero blocks/heads/key/value (each stays Unknown and names the term), an absurd `u64::MAX` block count, overflow through the u128 chain (stays Unknown), and the existing valid-dense case (`calculates_dense_kv_bytes_only_from_complete_terms` = 536_870_912 bytes) and unsupported-layout cases remain green.
- Mutation NT1 (the zero check dropped, keeping only the magnitude bound) failed the new test and passed after restore.
- Commands: `cargo fmt --check` PASS; `cargo test` 540 passed / 0 failed / 2 ignored (539 + the new S-03 test).

#### S-04 closure record — repair the cached pinned health model safely

- I1: `runtime::repair_health_model_with(spec, target, verify, cancel, on_progress)` (plus the `repair_pinned_health_model` wrapper for the compiled pin) quarantines an unverifiable cached file (unique `.quarantine-<unix-secs>` name; reparse/symlink ancestors refused first) and downloads the immutable pinned revision through the existing `download_file` size/hash authorization. The Tauri commands `repair_health_model` / `cancel_health_model_repair` guard one repair at a time and emit the existing `health-model-progress` event. UI: a "Repair cached model" button with a "Cancel repair" action in the seven-stage health block.
- I2: the corrupt bytes are never deleted; a `.quarantine-<stamp>.txt` note records the exact verification failure next to them, and a failed or cancelled repair leaves the live path without an unhealthy file, so the cache is never labeled healthy. The repair refuses to run while a health check is in flight, and the health run itself never executes an unverified file (verify precedes execution).
- V1 proved with fixtures: a corrupt cache is quarantined with both the bytes and the note preserved while the repair errors when the download target is unreachable (no live server needed — the earlier HTTP-fixture plan hung and was discarded per the DC-11 precedent); a healthy cache returns unchanged with no network; retry after failure is safe and preserves the first quarantine; a pre-cancelled repair against an unverified cache quarantines first and keeps no unhealthy file live.
- Mutation NU1 (quarantine replaced by plain deletion) failed the diagnostics-preservation assertions and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors; `cargo test` 542 passed / 0 failed / 2 ignored (540 + the two S-04 tests); `npm run check` PASS (337.79 kB, and `tsc --noEmit` PASS); Vitest 93 passed.

#### S-05 closure record — one catalog schema contract, visible drops

- I1: shared fixtures `scripts/tests/fixtures/catalog-schema-cases.json` (18 cases: valid rows, missing id, invalid repository, empty files, traversal/ADS/reserved-device/trailing-dot filenames, malformed revisions, bad digests, missing quant labels, bad dates, oversize beyond safe integers, duplicate repositories, duplicate ids, duplicate filenames, absent revision default). The JavaScript contract lives in `scripts/lib/catalog_schema.mjs`; the Rust contract is `catalog::parse_catalog` + `catalog_row_problem`; `scripts/tests/catalog_schema.test.mjs` (node --test, wired into `npm test`) and `catalog::tests::s05_shared_fixture_cases_agree_with_the_javascript_validator` run the SAME file and agree on every case. `validate_catalog.mjs` additionally proves the curated artifact has 0 drops under the shared contract.
- I2: dropped rows are recorded (`CatalogDrop {id, repo, reason}`, capped at 64) and serialized on the catalog (empty list skipped), so the app notice now reports "N catalog rows dropped by validation: <first reasons>". A second row for an already-seen repository is dropped with the reason "duplicate repository row (first row stays authoritative)", keeping every displayed row reachable through `catalog_file` (first row per repository). A catalog whose rows all drop refuses with the reasons in the error text instead of a bare message. Windows reserved device names, trailing-dot/trailing-space, control characters, filenames without the trailing-dot check, and >2^53 sizes are now refused identically by both validators (overlaps S-06.I2's filename policy).
- V1: `node --test scripts/tests/catalog_schema.test.mjs` 18/18; Rust fixture test 1/1 (same cases); real catalog revalidated: `npm run catalog:validate` → "catalog: valid v2 (158 models, 1417 files) · shared contract 0 drops"; duplicate-id/duplicate-filename fatals preserved on both sides.
- Mutations: OV1 (duplicate-repository drop removed) and OV2 (reserved device names allowed) each failed the shared fixture test and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors; `cargo test` 543 passed / 0 failed / 2 ignored (542 + the S-05 fixture test); `npm run check` PASS (337.96 kB; Vitest 93; node tests 18 + release gates).

#### S-06 closure record — bounded nested catalog payloads and filename aliases

- I1: the three catalog IPC commands (`filter_catalog`, `catalog_facets`, `catalog_rich_facets`) now take `catalog::IpcCatalogModels`, whose manual `Deserialize` refuses more than 2000 rows WHILE the JSON is still being read (not after a full allocation). `validate_catalog_payload` then bounds every nested field with the shared row contract plus aggregate limits (`MAX_IPC_FILES_TOTAL` 8192, `MAX_IPC_TAGS_TOTAL` 8192, `MAX_MODEL_FILES` 64, `MAX_MODEL_TAGS` 128, `MAX_TAG_TEXT_LEN` 256) before any cloning, aggregation or formatting; the mirror read refuses more than 5000 rows (`MAX_CATALOG_MIRROR_ROWS`), which routes into the existing recovery rebuild. The caps accommodate the real curated catalog (observed maxima: 99 tags, 34 files, 31 tag bytes — raised from an initial 64-tag bound that the two bitnet rows exposed, fixing a real regression caught by `catalog:validate`).
- I2: the Windows filename policy is now explicit and identical in both validators (from S-05): traversal separators, ADS colons, reserved device names with or without extensions, control characters, trailing dot/space, non-.gguf names, over-long names and over-255-byte names are refused; "." and ".." are refused; extended-path aliases cannot occur because `\` and `/` are already refused. `catalog_row_problem` reasons are the single source for both languages.
- V1: `s06_ipc_payload_bounds_reject_oversized_before_aggregation` exercises one oversized row (257 files), too many tags (129) and an overlong tag, the aggregate total (130 rows x 64 files), 150 bounded rows with facets still available, a 2001-row deserialization refusal, and a legal Unicode filename; `s06_the_mirror_read_refuses_an_oversized_row_set` mirrors 5001 rows through the app path and asserts the refusal; the JS test file adds the generated too-many-files/too-many-tags/overlong-tag/many-bounded-rows/Unicode cases (21/21). `npm run catalog:validate` passes on the shipped pair.
- Mutations: OW1 (deserialization row cap removed), OW2 (aggregate totals check removed) and OW3 (mirror read bound removed) each failed their guarding test and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors; `cargo test` 545 passed / 0 failed / 2 ignored (543 + the two S-06 tests); `npm run check` PASS (337.96 kB; node tests 21; catalog:validate 158 models / 1417 files · 0 drops).

#### S-07 closure record — bounded token input, visible legacy cleanup

- I1: `validate_hf_token` now checks `MAX_HF_TOKEN_BYTES` (4096) FIRST, before any character scan or storage attempt, and refuses with a fixed actionable message that never echoes the input (proved by a test that reads the message back). Real tokens and the exact bound pass; padded input is trimmed before the bound applies. Storage continues through Windows Credential Manager only.
- I2: legacy (`GGUF Pilot`) deletion failures are no longer swallowed on save: `legacy_cleanup_notice` maps the deletion result onto `TokenStatus.cleanupNotice` ("A credential from a previous product version is still stored in Windows Credential Manager. Remove retries the cleanup."), `hf_token_status` reports a still-present legacy entry the same way, and Remove (`clear_hf_token`) retries both deletions with its existing error path. The status remains masked (`mask_token` only); Authorization headers stay confined to the download request (already proven by the `g07_an_authenticated_redirect` and `catalog_client_sends_no_secret_header` regressions). The frontend shows the notice next to the token controls.
- V1: `s07_hf_token_input_is_bounded_before_validation_and_never_echoed` (real token, exactly-at-bound, one-over rejection with no echo, trimmed padding) and `s07_legacy_cleanup_failures_become_a_visible_notice` (success keeps the notice away; failure surfaces the shared wording without the underlying error text). The live Credential Manager vault is untouched by tests.
- Mutations: OX1 (length bound removed) and OX2 (cleanup failure swallowed) each failed their guarding test and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors; `cargo test` 547 passed / 0 failed / 2 ignored (545 + the two S-07 tests); `npm run check` PASS (338.06 kB, `tsc --noEmit` PASS); Vitest 93.

#### S-08 closure record — redirect and proxy policy with fixture proof

- I1: `download::allowed_redirect_target` defines the policy: production transfers are https-only on `huggingface.co`, `hf.co`, `github.com` and `githubusercontent.com` with dot-boundary suffix matching (covers `cdn-lfs.huggingface.co`, `cas-bridge.xethub.hf.co`, `objects.githubusercontent.com`, `release-assets.githubusercontent.com`); lookalikes, scheme changes, non-HTTP schemes and non-loopback private addresses are refused; loopback (127.0.0.0/8, `::1`, `localhost`) stays allowed so fixtures can serve local HTTP. `redirect_policy()` applies it to the catalog download client and the runtime GitHub client (max 10 hops, restated because a custom policy owns the count). A refused hop carries the safe diagnostic "the download redirected to a host outside the allowed set". The frontend never supplies a transfer URL (catalog authority split from DC-04/DC-05/DC-06; runtime catalog endpoints are compiled). The policy is documented in `docs/SUPPLY-CHAIN.md`. TLS verification is unchanged (rustls + webpki roots); no proxy is configured or accepted from the frontend, and documented proxy expectations are: none — system proxy variables are not consulted for the pinned HTTPS endpoints.
- I2: synthetic-token fixtures prove header behavior: `g07_an_authenticated_redirect_does_not_forward_the_token_to_another_host` (same machine, different host name) shows the Authorization header reaches the first host and not the redirect target, and the download still completes with a verified digest (final-hash after a legitimate redirect); `s08_a_redirect_outside_the_allowed_set_fails_with_a_safe_diagnostic` shows a 302 to `example.invalid` is refused before any connection, publishes no file and the error carries no credentials; the 12-case scheme/host matrix covers same- and cross-origin and scheme rules.
- V1: the two new tests plus the retained g07 regression (1 passed) and the policy matrix (2 passed). Resume across a redirect: chunk state is keyed by the original URL and the chain re-resolves per request; the DC-02/DC-12 suites remain the direct-URL resume proof, and no redirect-specific resume difference exists in the code path.
- Mutations: OY1 (policy always follows) and OY2 (host set emptied) each failed their guarding test and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors; `cargo test` 549 passed / 0 failed / 2 ignored (547 + the two S-08 tests); `npm run check` PASS.

#### S-09 closure record — cache publication identity and honest persistence

- I1: `save_cache_record` now keeps the exclusively created temp-file handle through write, sync AND rename — the name is never closed and reopened, so no window exists in which the temp name can refer to different bytes. A failed write removes the temp file; a failed rename removes the temp file and returns the error with the previous validated cache untouched. Atomic body/ETag/signature publication is unchanged (single record rename). The rename-with-open-handle path executes for real on this Windows host in the success test.
- I2: `write_refresh_stamp` returns a `Result`; both fetch paths (network and 304) now set `persistence_notice` on cache-save and/or stamp failures while still serving the fresh in-memory catalog — a failed persistence is never reported as durable, and the last-good cache file survives any failure. Both notices are combined when both fail.
- V1: `s09_cache_publication_keeps_the_handle_and_retains_last_good` (byte-exact publication, no temp leftovers, rename refusal cleans up and the previous record still reads, recovery works); `s09_persistence_failures_are_surfaced_while_fresh_data_is_served` drives the REAL local HTTP fixture with a blocked cache path (origin network + "could not be saved for offline use"), a blocked stamp path ("refresh time could not be recorded"), and both-clean (no notice) — proving the notice is not a constant. No untrusted replacement path exists: the cache is only ever renamed from a handle the writer owns; readers only accept signature-verified records.
- Mutations: OZ1 (failed-publication cleanup removed), OZ2 (stamp failure swallowed) and OZ3 (save failure ignored in fetch) each failed their guarding test and passed after restore. The handle-retention property itself is proven by construction plus the executed rename-under-open-handle path; a deterministic adversarial interleave is not representable in-process and is stated as such.
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors; `cargo test` 551 passed / 0 failed / 2 ignored (549 + the two S-09 tests); `npm run check` PASS.

#### S-10 closure record — strict validators and bounded retries

- I1: `catalog::etag_is_strong` (non-empty, <=256 graphic ASCII, no `W/` prefix) gates what is stored; weak or malformed validators are observed and dropped, so they can never be replayed as `If-None-Match`. The same filter applies to the runtime release catalog client. A 304 that presents a different validator than the one sent is refused as a successful refresh: the cached body is still served, with "the catalog server answered 304 with a different validator" visible, and no refresh stamp is written (strict byte equality, proven with a case-differing validator).
- I2: the download retry loop honors a bounded `Retry-After` (`bounded_retry_after_secs`: numeric seconds or IMF-fixdate, clamped 1-30s, absurd/malformed/past values fall back to the linear backoff), carries the value in the error marker, and the wait stays cancellation-aware and bounded by `MAX_TRANSFER_ATTEMPTS`. The catalog builder moved to `scripts/lib/http_retry.mjs`: a 20-second per-request deadline via `AbortSignal.timeout`, at most 5 retries, Retry-After in numeric and HTTP-date form clamped to 1-30s, and a 60-second total wait budget per logical request. The mandatory final SHA-256 check is untouched.
- V1: Rust `s10_validator_semantics_are_strict_and_weak_etags_are_never_replayed` (unit matrix + fixture: weak validator stored as none; strong validator stored; case-differing 304 refused with the cached body served; identical 304 is the ordinary not-modified path) and `s10_retry_after_values_are_bounded_and_parsed_from_both_forms` (numeric clamps, date parsing against the canonical epoch 784111777, refusal of PST/garbage/past). Node `scripts/tests/http_retry.test.mjs` 6/6 (numeric/date clamps, malformed/extreme values, retry-then-success, attempts+budget bounds, stalled fetch aborted by the deadline). A fixture defect found during this work (missing CRLF when no ETag header) was fixed and re-verified against the retained dc07 suite.
- Mutations: PA1 (weak-etag filter removed), PA2 (304 strict compare removed), PA3 (JS clamps removed) and PA4 (JS total-budget check removed) each failed their guarding test and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors; `cargo test` 553 passed / 0 failed / 2 ignored (551 + the two S-10 tests); `npm run check` PASS (node tests 21 + 6 + release gates); docs updated in `docs/SUPPLY-CHAIN.md`.

#### S-11 closure record — fit labels describe the selected build

- I1: `catalogBuildFit(selectedBytes, smallestBytes, fitPerMille, budgetBytes)` returns the threshold, whether the SELECTED build passes, and whether the auto-filter keeps the row (smallest-file rule, mirroring `catalog::model_hidden_by_fit_rule`). Each catalog row now shows a note for the build selected in its Build selector: "SIZE CHECK PASSES · <size> ≤ <threshold> on <source> budget · size only, runtime memory not measured" or "SIZE CHECK FAILS FOR THIS BUILD · <size> > <threshold> … The model stays listed because a smaller build fits; runtime memory is not measured." The filter caption was corrected to "filter keeps models whose smallest build is ≤ <threshold> on <source>", so a model with a small variant never reads as the selection fitting.
- I2: the note states the check is a size comparison only and that runtime memory is not measured; with no observed budget the note reads "SIZE CHECK UNKNOWN · no measured memory budget; open the build to check allocation" and the fit fields are null, never a fabricated capacity claim. Disk-size heuristics stay separate from inference-memory qualification.
- V1: `catalogBuildFit` unit cases (small passes / large fails with the row kept / unknown nulls / fraction clamp) and the component test "labels the fit of the selected build, never the smallest variant (S-11)": fixture model with a 2 GB Q4_K_M build and an 8 GB Q8_0 build under a 10 GB dedicated budget (5 GB threshold); default small build passes, switching the selector to the large build shows the failure and the row-survival explanation, and the passing claim disappears.
- Mutations: PB1 (selectedPasses computed from the smallest build — the original defect) failed both the unit and component tests; PB2 (the row-survival explanation removed) failed the component test; both restored green.
- Commands: `tsc --noEmit` PASS; Vitest 97 passed (93 + 4); node tests 140 passed; `npm run build` PASS (339.01 kB); `npm run check` PASS; impeccable detector shows the same four pre-existing advisories as at HEAD (font-size 7px, side-tab, two palette colors), none from `.catalog-fit-note`.

#### S-12 closure record — every performance number within its measured scope

- I1: `evidence::WORKLOAD_SCOPE_NOTE` (mirrored as `WORKLOAD_SCOPE_NOTE` in `model.ts`) labels the workload a controlled greedy microbenchmark (one fixed prompt, temperature 0, one request at a time, warm server) and states that it does not represent every workload class and that speculative-decoding gains measured here do not generalize. The note is written into every new `BenchmarkManifest.scopeNote`, so exports carry the caveat; the panel renders the manifest note (or the constant for older runs) under the benchmark card.
- I2: each metric card shows `describeSamples` — `n=<count> · nearest-rank p95 is the sample maximum` for fewer than 20 samples, `n=… · nearest-rank percentiles` otherwise, and "sample count unknown" when absent. The derived TTFT line now reads "Derived TTFT p50 … ms (prefill + per-token decode; not an observed first token)", keeping it distinct from the observed first-token metric which carries its own sample line.
- I3: the memory line now reads "CPU peak working set (process lifetime, excludes GPU): … (k/n sampled)" and `evidence::WORKING_SET_SCOPE_NOTE` / `WORKING_SET_SCOPE_NOTE` states it is process-lifetime CPU working-set evidence, excludes dedicated GPU memory and is not an isolated request allocation.
- V1: Rust `s12_scope_notes_survive_the_manifest_round_trip_and_default_honestly` (default carries the note; serialize→parse keeps it; a legacy empty note still parses and is never invented; the working-set wording asserted); `model.test.ts` `describeSamples` cases (5/19/20/100/0/NaN/null); component test "labels sample counts, derived TTFT and working-set scope honestly (S-12)" with a five-trial, warm-process, missing-direct-observation fixture (both sample lines counted exactly, derived TTFT caveat, working-set wording, `2/2 sampled`, and the manifest scope note rendered).
- Mutations: PC1 (the decode sample-count line removed) and PC2 (working-set wording reverted) failed the component test; both restored green.
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors; `cargo test` 554 passed / 0 failed / 2 ignored (553 + the S-12 test); `npm run check` PASS (340.01 kB); Vitest 100 passed; node tests 140 passed.

#### S-13 closure record — domains before launch, companions reconciled

- I1: `LaunchProfile::validate_domains` enforces the value domain of every enumerated and ranged profile field from the pinned option map: cache types (f32/f16/bf16/q8_0/q4_0/q4_1/iq4_nl/q5_0/q5_1 for K, V and both draft caches), flash attention (on/off/auto), load mode (auto/none/mmap/mlock/mmap+mlock/dio), lazy mode (on/auto/off), split mode (none/layer/row/tensor), speculative types (the eleven documented tokens, comma-separated), thread counts (-1 sentinel or 1..=1024), batch/ubatch/context/slots ranges (with the existing ubatch ≤ batch relationship), top-k, repeat-last-n, reasoning budget, sleep, timeout, main GPU, temperature, top-p, min-p, repeat penalty, DRY multiplier/base, draft probabilities, and the n-gram size floor. `build_args` calls it first, so every launch path AND the tuning advisor's `profile.build_args()?` guard (tune.rs:369) reject impossible values before a process starts or a paid measurement runs; each error names the field and the offending value.
- I2: `reconcileDraftCompanion` clears a retained draft path (with a user-visible notice in the profile form) whenever the newly selected method does not start with `draft-`; a `draft-*` method with no path keeps failing before launch as before. `docs/ARTIFACT-IDENTITY.md` documents companion matching as a filename/quantisation heuristic that does not read companion headers during a scan, and names the stronger available signals (explicit user selection, displayed role, runtime contract).
- V1: `s13_impossible_profile_domains_fail_before_launch_and_name_the_value` runs 20 invalid-domain cases (including NaN temperature and a zero n-gram size) plus the relationship case; the baseline profile is asserted to build fully (`build_args().expect(...)`) — a fix made after PD1 initially passed through a vacuous assertion (`Wildcard CORS requires an API key file` broke every case), which is recorded rather than hidden. JS `reconcileDraftCompanion` cases cover draft methods kept, ngram/none cleared, and no-op when nothing was retained.
- Mutations: PD1 (domain validation disabled in build_args), PD2 (cache-type membership check removed) and PD3 (companion reconciliation disabled) each failed their guarding test after the vacuous-assert fix and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors (one `type_complexity` fixed by a local type alias); `cargo test` 555 passed / 0 failed / 2 ignored (554 + the S-13 test); `npm run check` PASS (340.38 kB); Vitest 103 passed; node tests 140 passed.

#### S-14 closure record — truthful copied commands and redaction

- I1: `LaunchProfile::escaped_command_with_args(args, CommandShell)` quotes for a NAMED shell — PowerShell single-quotes every token with `'` doubled (literal under PowerShell for spaces, quotes, `& | > < ^ $ \` ()`, `%` and Unicode), cmd.exe applies MSVCRT quoting (embedded `"` as `\"`, backslash runs before quotes doubled) and REFUSES a `%` value with an actionable notice instead of quoting a lie. `argv_json_with_args` emits the exact argv as a JSON array: lossless for any wrapper. The `preview_command` Tauri command now returns `{powerShell, argv, cmd, cmdNotice}` and the Provisional command panel renders each form labelled; `validate_launch_arguments` uses the PowerShell form for its recorded command. The naive space-only quoting helper (`display_command_with_args`) and the superseded `compose_provisional_command` were removed; the FE-04 source gate was updated to the new API and still asserts cheap composition (no `prepare_launch`).
- I2: `sanitize_effective_args` now covers the short draft flag `-md` (plus `-mdl`, `--draft-model`, `--spec-draft-model`, `--model-draft`) and `--chat-template-file`, alongside the existing model/projector/LoRA/key/cert mappings. `docs/EVIDENCE-MATRIX.md` documents that raw local manifests still include explicit local paths and are not publication artifacts, while share/export bundles run the separate redaction path and the calibration identity uses the sanitized tokens.
- V1: `s14_quoted_commands_and_argv_round_trip_every_hazard` round-trips an 8-token hazard set (spaces, single and double quotes, shell metacharacters, `%`, Unicode, a trailing-backslash path) through the PowerShell form with a reversible tokenizer, asserts the cmd.exe refusal for `%` and deterministic quoting otherwise, and parses the argv JSON back to an exact array. `s14_sanitizer_covers_every_path_bearing_flag_with_canaries` places a distinct canary path after each of eleven path-bearing flags and asserts none survives while `--port 8080` does.
- Mutations: PE1 (PowerShell quoting reverted to space-only), PE2 (cmd `%` refusal removed) and PE3 (draft-flag sanitization removed) each failed their guarding test and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors; `cargo test` 557 passed / 0 failed / 2 ignored (555 + the two S-14 tests); `npm run check` PASS (340.99 kB); Vitest 103; node tests 140 (the FE-04 gate updated to the new preview API).

#### S-15 closure record — bounded, cancellable discovery

- I1: `scan_models_with_cancel(root, cancel, ScanLimits)` walks the tree with a depth cap (8), a visited-entry cap (200 000), a diagnostic cap (64) and cancellable recursion checked at entry and per directory entry; the wrapper `scan_models` keeps the old unbounded-by-signature behavior for existing callers. Discovery already ran off the interface thread (`spawn_blocking`); the new `scan_models_report` command adds one-at-a-time ownership (`A scan is already running.`) plus `cancel_scan`, and the legacy `scan_models` command stays for verifier scripts that expect the array shape.
- I2: an unreadable directory (or entry, or metadata failure) becomes a bounded `ScanProblem {path, reason}` and the traversal continues, so valid models from other subtrees are preserved; symlink and reparse-point skipping is retained. `ScanReport {models, problems, truncated}` distinguishes "bounded stop" from "found everything". The interface renders the diagnostics under the status line (visible from every view) with the first three reasons and a count.
- V1: `s15_deep_wide_unreadable_and_cancelled_scans_stay_bounded_and_partial` builds a real tree (shallow model, a 10-deep chain with a model at the bottom, a Unicode/space directory) and asserts: shallow + Unicode found, deep refused with the depth diagnostic, truncated set; a 2-entry budget stops early with the entry-limit diagnostic; a direct `collect_gguf` on a file path yields exactly one "Could not read this directory" diagnostic instead of an abort; a pre-cancelled scan returns an empty model list with a "cancelled" note; the retention cap collapses extra diagnostics into one suppression notice. The component test "renders bounded scan diagnostics while keeping discovered models" drives the real panel: with two problems and `truncated: true` the diagnostics element shows "stopped early" + both reasons AND the discovered model remains listed.
- Test-integrity notes (recorded, not hidden): the FE-08 prologue originally asserted on the transient notice string, which a concurrent port probe can overwrite once the scan promise gained one hop with the report shape — the prologue now asserts the durable consequence (the profile loads for the scanned model); the first PF2/PF3 mutation attempts were behaviour-preserving no-ops and were replaced with real mutations before counting them.
- Mutations: PF1 (depth limit removed), PF2 (unreadable-directory diagnostic dropped), PF3 (both cancellation checks removed) each failed their guarding test and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors; `cargo test` 558 passed / 0 failed / 2 ignored (557 + the S-15 test); `npm run check` PASS (341.82 kB); Vitest 104; node tests 140; impeccable detector unchanged at the four pre-existing advisories.

#### S-16 closure record — recoverable, bounded evidence history

- I1: retention is `MAX_RETAINED_RECORDS_PER_CATEGORY` (4 000 per category) applied by `prune_records` at anchor and model persistence; `prune_records_with` exposes the bound for tests; only `.json` records in the category directories are candidates and non-record files are untouched. `clear_calibration_history` is the explicit user cleanup (a "Clear local history" action in the evidence panel reporting the removed count) and keeps quarantined files, which are the diagnostic evidence of earlier failures. `docs/EVIDENCE-MATRIX.md` documents storage, retention, cleanup and corruption lifecycle; exports carry their own copies of referenced measurements, so retention never silently invalidates an exported artifact.
- I2: `load_calibration_anchors` / `load_calibration_models` return `LoadedRecords {records, problems}`; a corrupt, oversized, unreadable or schema-invalid record is moved to `calibration/quarantine/<name>.corrupt-<stamp>` (bytes preserved, never deleted) and reported as a bounded problem (32 max, then a suppression notice) while the compatible history keeps loading. `CalibrationRecords.problems` carries the diagnostics to the interface, which reports "N stored records could not be read and were quarantined; the rest of the history loaded normally."
- V1: `s16_mixed_valid_corrupt_and_oversized_records_still_load_with_valid_history` (valid anchor+model beside a corrupt JSON, a 64 KiB+1 oversized file and a wrong-shape record: valid records survive, ≥2 problems reported, ≥3 files quarantined, a second load is clean, cleanup removes exactly the two valid records and keeps the quarantine) and `s16_retention_prunes_the_oldest_records_only` (5 records with bound 3 removes the two oldest, keeps `notes.txt`, no-op below the bound); component test "reports quarantined records and clears the history explicitly (S-16)" (quarantine message rendered; "Clear local history" reaches the backend, reports "Removed 3 stored calibration records", reloads history).
- Acceptance update (recorded): `mt14_one_validator_rejects_bad_models_at_every_entry_point` asserted the OLD all-or-nothing load (`is_err()`); per S-16.I2 it now asserts the quarantine-and-continue contract (empty records + one quarantine problem). No other test encoded the old behaviour.
- Mutations: PG1 (corrupt anchor fails the whole load again), PG2 (quarantine deletes the record) and PG3 (retention prunes the newest) each failed their guarding test and passed after restore.
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors; `cargo test` 560 passed / 0 failed / 2 ignored (558 + the two S-16 tests); `npm run check` PASS (342.64 kB); Vitest 105; node tests 140.

#### S-17 closure record — imported evidence stays imported

- I1: `ExternalProvenance` (single variant `importedExternal`, serde-forced) is a new bundle field, and the interface states plainly that "verified" records a user review of the file — not a local rerun, not an origin signature, and not cryptographic proof of measurement. The panel label now reads "Provenance: importedExternal — verified means you reviewed this file; … not a local rerun …".
- I2: `validate_external_evidence` forces BOTH state=Pending and provenance=importedExternal on every import, so neither can be self-declared; `review_external_evidence` still requires explicit confirmation, rejects Pending as a review outcome, and returns a terminal state only. A forged provenance string (`localRun`) fails deserialization outright because the enum has exactly one accepted variant. The release-gates source guard asserts `ExternalEvidence*` may only be referenced by `calibration.rs`, `lib.rs`, `model.ts`, `V03EvidencePanel.tsx` (and the guard itself) and that `recommend.rs`, `measurement.rs` and `sharing.rs` never mention it — ranking and calibration writers cannot consume imported evidence.
- V1: `s17_imported_evidence_cannot_claim_provenance_or_review_state` imports a synthetic bundle that self-declares "verified" (imports Pending), proves review requires confirmation and a terminal state, round-trips the reviewed bundle through serialize → reload → re-import (Pending again, provenance still importedExternal), and shows a forged provenance claim is rejected at parse time. The component test "an imported bundle arrives pending and labelled as imported (S-17)" drives the real panel: the textarea import shows "Imported state: pending", "Provenance: importedExternal", "not a local rerun" and "stays out of ranking and calibration".
- Mutations: PH1 (import no longer forces Pending), PH2 (provenance wording removed), PH3 (an external-evidence reference added to recommend.rs) each failed their guarding test/gate and passed after restore. (PH3's first run was mis-scored by a detector bug — the node reporter prints "fail 1", not "1 fail"; re-run with the corrected detector: CAUGHT.)
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors; `cargo test` 561 passed / 0 failed / 2 ignored (560 + the S-17 test); `npm run check` PASS (342.88 kB); Vitest 106; node tests 114 gates + 140 total.

#### S-18 closure record — tested wire and persisted contracts

- I1: `scripts/tests/fixtures/ipc-contract.json` is the shared wire authority; the Rust test `s18_shared_ipc_contract_fixture_matches_rust_serialization` serializes representative values (TokenStatus incl. the multi-word `cleanupNotice`, CommandPreview with nullable `cmd`, CatalogDrop, the `pending`/`importedExternal` enum spellings, RECORD_SCHEMA_VERSION) and asserts exact equality with the fixture; `scripts/tests/ipc_contract.test.mjs` (wired into `npm test`) asserts the same entries against the `model.ts` expectations (exact key sets, JS types, nullable-string handling, camelCase-only, documented enum spellings). Errors cross IPC as plain strings by contract, and the fixture comment says so.
- I2: calibration anchors and models carry `schemaVersion` (default `RECORD_SCHEMA_VERSION = 1`), reject a NEWER version with "rebuild it from current measurements with a newer Localmotive", and tolerate unknown fields on purpose (forward compatibility). Catalog documents already carry `schemaVersion`, benchmark manifests carry `schema` plus the scoped execution-snapshot schema, profile records normalize with quarantine — the complete format story is documented in `docs/EVIDENCE-MATRIX.md`, with serialization validation kept in the model/loader layer rather than rendering.
- V1: `s18_calibration_records_version_and_reject_newer_formats` (field-stripped record loads as v1; an unknown extra field is tolerated; a v2 record is rejected with the actionable message) and the two shared-fixture tests above exercise old/current/malformed shapes on both sides.
- Mutations: PI1 (TokenStatus `rename_all` removed — first attempt MISSED because `configured`/`masked` are single words; the fixture was strengthened with the multi-word `cleanupNotice`, after which PI1 is CAUGHT), PI2 (record-version check removed), PI3 (fixture key renamed — caught by BOTH the Rust test and the node test).
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors; `cargo test` 563 passed / 0 failed / 2 ignored (561 + the two S-18 tests); `npm run check` PASS (342.88 kB); Vitest 106; node tests 114 + 2 + 21 + 6 + 140 total.

#### S-19 closure record — bounded property campaigns with deterministic seeds

- Added `src-tauri/src/test_support.rs` (splitmix64 `Rng` + `campaign` runner, seed reported in every panic) and `src-tauri/src/property_tests.rs` with five capped campaigns: GGUF reader versus 400 random byte-blobs plus 100 truncated-header fixtures (clean result or clean error, parsed counts bounded); proposal parser versus 600 noisy texts and 100 nested-brace texts; shard-name round-trip versus generated valid split names and 500 noise strings; sanitizer idempotence plus canary-path absence over all eleven path-bearing flags (200 seeds); numeric summaries versus ±MAX/±INF/NaN/subnormal inputs (400 seeds, all outputs asserted finite). Frontend: 300 generated garbage stored profiles through `safeJsonParse`/`normalizeProfile` against a valid model in `src/model.test.ts`. Total suite 568 passed / 0 failed / 2 ignored (~1.6 s for the Rust campaigns); Vitest 77.
- Honest note: the first FE campaign draft called `normalizeProfile` without a model (vitest does not type-check), which surfaced as a crash inside `suggestedProfile`. That was a TEST misuse, not a product defect — the test now supplies a valid model, the campaign asserts the real contract, and `tsc` was added to the loop to catch such misuse.
- Mutations: PM1 (finite-filter removed from `metric_stats` — CAUGHT by the numeric campaign), PM2 (`-md` removed from the sanitizer redaction list — CAUGHT by the canary property), PM3 (FE `extraArgs` isArray guard removed — CAUGHT, reported by the pre-existing FE-09 regression test which the campaign also drives through the same guard).
- Limits recorded in `docs/EVIDENCE-MATRIX.md`: these are bounded campaigns with deterministic seeds — not exhaustive proof, not coverage-guided fuzzing; a passing run says nothing about inputs outside the generated classes.

#### S-20 closure record — cloud data disclosure at the AI Tune action

- I1: the AI Tuning screen now carries a "Cloud data disclosure" panel: a statement that local inference and local share export never send data anywhere, a Full/Minimal choice, the Rust-owned section list, and an explicit note that stored credentials are never part of the brief. The list comes from `tune::disclosure_sections()` via the new `tune_disclosure_list` command, and `s20_disclosure_sections_cover_every_brief_field` pins the list to the brief's actual top-level wire fields, so a new field cannot ship without a disclosure line.
- I2: `BriefDisclosure::{Full, Minimal}`; `TuningBrief::apply_disclosure` builds the exact wire JSON before every advisor call, and `cloud.rs` sends `brief.wire`. Minimal redaction replaces the home directory (USERPROFILE) anywhere it appears and reduces path values to file names — prose keeps its sentence with each path token shortened, URLs are left alone, object keys and the tunable whitelist are untouched, and the chosen mode is persisted per user in `localmotive:tune-disclosure`.
- V1: `s20_minimal_disclosure_redacts_paths_and_user_identifiers` uses canary paths (`C:\\Users\\canary-user\\...`) plus a synthetic trial error: Full mode still contains the canary (transparency), Minimal removes the user identifier and directory prefixes while keeping `alpha-Q4_K_M.gguf`, the error sentence, and `tunableFields`, and no secret-bearing key appears. `s20_run_tuning_sends_the_minimal_wire_when_asked` captures the advisor wire end-to-end and proves the machine-leaving payload is redacted. The component test drives the radio, asserts the disclosure text and section list render before the action, and that the choice persists.
- Mutations: PS1 (Minimal returns the raw brief) CAUGHT; PS2 (a disclosure section removed) CAUGHT; PS3 (`changeDisclosure` stops persisting) CAUGHT (reported as one failed test; the first score used the wrong summary line and was corrected).
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors; `cargo test` 571 passed / 0 failed / 2 ignored; `npm run check` PASS (344.36 kB); Vitest 108; node tests 143.

#### S-21 closure record — bounded cloud responses and six-provider contracts

- I1: `read_bounded_body` caps every response (`MAX_MODEL_LIST_BYTES` 512 KiB, `MAX_CHAT_BYTES` 2 MiB) and refuses an oversized body with a message naming the limit; `retry_after_secs` parses numeric and HTTP-date `Retry-After`, clamped to 1..=30 s; a single visible retry follows a 429 (never more — `Connection` counted in the fixture test) and the wait is sliced against the run's remaining time. `chat_with_deadline` clamps each request timeout to the tuning run's remaining budget; `CloudAdvisor` carries that deadline, so one slow request cannot outlive the run. The fixed provider allowlist is unchanged, and the injectable base is a private test seam (`chat_via`).
- I2: `scripts/tests/fixtures/cloud-contracts.json` carries all six providers (openrouter, anthropic, openai, gemini, deepseek, xai) with base URLs pinned to the allowlist and seven cases each: models success, empty models, chat success (including Gemini's content-part array), auth failure, quota/rate limit, malformed body, oversized; `s21_contract_fixtures_cover_every_provider_and_case` drives `parse_models`, `extract_reply` and `status_outcome` through every case; `s21_retry_after_is_bounded_and_tolerant` covers the header parsing.
- I3: STUBBED contracts: all six providers (no live credentials were authorized for this turn). Live-checked: none; the optional authorized live smoke check is NOT RUN and recorded as such. Anthropic reviewed against its documented OpenAI-compatibility page (platform.claude.com/docs/en/cli-sdks-libraries/libraries/openai-sdk): Bearer `authorization` and `choices[].message.content` and `retry-after` are supported, so the chat path is documented behavior; no `GET /models` is documented for the compatibility layer, so `lists_models` is now false and the picker offers the fixed default instead of an undocumented route (behaviour change, noted).
- V1: `s21_oversized_response_terminates_with_the_cap_error` (2 MiB + 1 byte body against a local fixture server fails with the cap named), `s21_one_bounded_retry_after_a_429_then_success` (exactly one retry after the documented 1 s wait, request count 2), `s21_a_hung_request_ends_at_the_deadline` (a hung response ends within the deadline, and the key never appears in the error text). No credentials are retained anywhere in fixtures or logs (`sk-canary-secret-value` is a canary and asserted absent from errors).
- Mutations: PT1 (byte cap removed) CAUGHT; PT2 (the documented wait skipped) CAUGHT; PT3 (Retry-After detail dropped) first MISSED because the fixture stored the header as a JSON string and the test's `as_u64` read None — the test now parses both forms, after which PT3 is CAUGHT.
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors; `cargo test` 576 passed / 0 failed / 2 ignored; Vitest 108; node tests 143.

#### S-22 closure record — unified actionable frontend error handling

- I1: `errorText` now unwraps object rejections (preferring a `message` field, serializing otherwise) on top of the existing structured-string and `Error` handling, and all 27 `String(error)` call sites in `App.tsx` use it. The folder/file pickers wrap `openDialog` in a recoverable catch; all twelve external-link actions route through `openExternal`, which catches opener rejections; the evidence export wraps `saveDialog` in `runAction`.
- I2: `reportFailure` sets a concise recovery notice plus a `diagnostic` state; the notice line renders a "Copy diagnostic" action (clipboard) and an explicit `<details>` disclosure with the raw diagnostic instead of spilling JSON into the status text. A failed export write keeps the built bundle in memory and says so ("choose a different location and export again"); the measured result itself is never cleared by a secondary failure.
- V1: `src/model.test.ts` covers `errorText` for Error/string/structured-string/object-with-message/object-without-message/null; the App component test rejects both the dialog and the opener, asserting recovery text, the unchanged model root, the disclosed diagnostic, and that the screen survives; the panel test rejects the save dialog and asserts the rejection message plus the surviving "Decode throughput" result.
- Mutations: PU1 (dialog catch removed) CAUGHT; PU2 (diagnostic disclosure removed) CAUGHT; PU3 (panel save un-wrapped from `runAction`) CAUGHT.
- Commands: `npx tsc --noEmit` PASS; `npm run check` PASS; Vitest 111; node tests 143. The first commit for this work (`40dd9f9`) slipped three type errors in the new test files past the pre-commit `npm run check` (it had been run before those edits); the follow-up fixed them and amended to `590674b` — recorded honestly because an unverified commit briefly existed in local history.

#### S-23 closure record — first-run and empty-inventory states

- I1: the Profile screen now renders an explicit empty state instead of rendering nothing when no profile exists: "No models to profile yet" (zero models) or "No model selected" (models exist), each with the next action ("Open Inventory") that switches screens. The old blank-screen condition (`view === "profile" && profile && &&`) is gone.
- I2: the Inventory screen shows a zero-model state with a heading that distinguishes all three situations — no root yet, an empty-but-valid scanned folder, and a failed scan (`scanFailed` state) — plus local recovery actions (Choose folder, Rescan this folder) and the scan's diagnostic count for the valid-empty case.
- V1 (packaged): rebuilt the portable exe (`npm run tauri build -- --no-bundle`, BUILD EXIT 0, 20 864 512 bytes, sha256 `f64c0fbf7bfb0789…`) and opened it with a fresh WebView2 user-data folder (no settings, no models) and CDP port 10027. `scripts/verify_s23_firstrun.mjs` reports **S23_RESULT PASS (7 checks)**: first-run Profile empty state with the Open Inventory action; Inventory folder prompt with both actions; an empty valid folder reported honestly ("No GGUF models in this folder yet"); a fixture folder with one shard scanning to 1 logical target; the inventory row naming `fixture-alpha` with 1/1 shards; and Profile becoming a real profile with a Start action once the selection loads. Probe log: `.hermes-0.6/s23-probe.log`.
- Probe honesty notes: two probe iterations were needed because (a) a successful rescan intentionally switches the view to Profile (FE-01), and (b) the Profile heading is rendered uppercase, so the check must be case-insensitive; the app behaviour was correct in both cases. The first packaged build attempt failed because a stale `localmotive.exe` process held the output file (`Access is denied (os error 5)`) — killed and rebuilt; also one `!`-less optional chain in the new component test slipped past vitest but was caught by the build's `tsc` and fixed before this record.
- Mutations: PV1 (Profile empty state disabled) CAUGHT; PV2 (`scanFailed` never set) CAUGHT.
- Commands: `npx tsc --noEmit` PASS; `npm run check` PASS (inside the tauri build); Vitest 112; node tests 143; packaged probe 7/7.

#### S-24 closure record — full-path retrievability, design alignment, reduced motion

- I1: a `PathText` component now backs the three truncation sites the audit named — the inventory model folder, the server log path, and catalog filenames. The value keeps the layout ellipsis but carries the full path in `title` and `aria-label`, is keyboard reachable (`tabIndex=0` with a visible focus ring), and one adjacent `Copy` control writes it to the clipboard and reports "Copied".
- I2: catalog download rows no longer each claim `button primary` (demoted to the standard secondary; the audit's "all catalog downloads are green primary buttons" observation is fixed and the screen keeps zero primaries, matching the About screen precedent — the one-primary rule constrains the count, browsable screens carry none). The `.signal.live` box-shadow observation is resolved as POLICY-COMPLIANT, not drift: DESIGN.md's lighting clause documents exactly this lamp (`box-shadow: 0 0 10px rgba(158, 220, 114, .35)` for the rail status dot when a server is running) and the CSS matches the documented value — no change needed, reasoning recorded.
- I3: "Jump to Advanced" now reads `prefers-reduced-motion` and passes `behavior: "auto"` instead of always using smooth scrolling, and moves keyboard focus to the revealed region's summary so the next Tab stop is inside it.
- V1 (packaged): rebuilt portable exe (BUILD EXIT 0, 20 864 512 bytes, sha256 `3c789057eaf9e9c4…`) and ran `scripts/verify_s24_paths.mjs` with a fresh WebView2 profile → **S24_RESULT PASS (7 checks)**: full value in title/aria-label, keyboard reachability, the copy write reporting "Copied", the emulated reduced-motion preference reaching the app, focus on SUMMARY with expansion, `behavior=auto` (no smooth), and the copy control visible at 320 px (35×22 px). Probe log: `.hermes-0.6/s24-probe.log`.
- Probe-discovered defect (fixed): the inventory `<tr>` opens the profile on click, so clicking the copy control bubbled into `loadProfile` and switched screens — `event.stopPropagation()` now guards the copy control and a component regression asserts the screen stays. Another real-platform finding recorded: WebView2 shows a native clipboard permission bubble on the first programmatic clipboard write; the probe grants `clipboardReadWrite` via CDP, and the product keeps the standard permission flow (documented, not worked around).
- Mutations: PV1 (path title removed) CAUGHT; PV2 (reduced motion ignored) CAUGHT; PV3 (catalog primary restored) CAUGHT; PV4 (`stopPropagation` removed) CAUGHT.
- Commands: `npx tsc --noEmit` PASS; `npm run check` PASS; Vitest 112; node tests 143; packaged probe 7/7. Rust untouched by this package.

#### S-25 closure record — measured bottlenecks after correctness (I3 blocked)

- I1 (packaged measurements, sha256 `fad1e6aa5e1059f2…` final build, 20 864 512 bytes; fixture = 500 shard files, 536 B each; isolated verifier env with the real signed 630 182-byte catalog served locally): catalog first render 158 rows in 260 ms; scan of 500 files 110 ms with max animation-frame gap 9.8 ms; scan cancellation 6 ms with max gap 8.1 ms; 30 rapid input events 173 ms total with max frame gap 9.9 ms; launch to WebView-ready 614 ms cold (fresh profile) vs 608 ms warm — a single sample each, the difference is within noise. Probe: `scripts/measure_s25_packaged.mjs`, logs `.hermes-0.6/s25-packaged.log`.
- I2 (Rust release measurements on the committed catalog, `cargo test --release --lib s25_catalog_pipeline_measurement -- --ignored --nocapture`): 158 models / 1417 files / 630 182 bytes; parse 2.2 ms; `filter_models` 0.06–0.85 ms per call; `rich_facets` 0.41 ms; the FE sends the full snapshot per call — 509 915 serialized bytes, deserialize 0.98 ms, serialize 0.38 ms. Packaged keystroke latency to an updated list: 22–113 ms typical, with a reproducible ~700 ms outlier on one unchanged-result value; the report of that cause remains UNKNOWN (the attempted `__TAURI_INTERNALS__.invoke` interception could not hook WebView2's non-writable internals — recorded, not guessed).
  Optimization applied where evidence supported it: `useDebouncedValue(catalogSearch, 120)` collapses typing bursts. Same-observer before/after on the same machine and fixture: BEFORE (SHA range HEAD) 4 list updates during a 7-keystroke burst with the last at +0 ms; AFTER 1 update at +80 ms after the burst end; filter requests per burst 7 → 1 (510 KB each). Frame gaps unchanged (≤10 ms). Mutation PX2 (delay zeroed) and PX1b (debounce fully bypassed) CAUGHT; PX1 (deps-only mutation) was behaviour-equivalent and is recorded as a MISS, not a catch.
- I3: release-record metrics collected from retrieved run records (`gh run list --workflow ci.yml --limit 8`), durations in minutes: SUCCESS 10.7, SUCCESS 17.2, SUCCESS 8.8, SUCCESS 8.8, SUCCESS 10.0, CANCELLED 11.9, CANCELLED 33.1, SUCCESS 9.3 for the eight most recent runs (6 success / 2 cancelled in this sample; the cancellations are the superseded pushes on 2026-09-10, not failures). Retention caps are the S-16 constants with their tests (`MAX_CALIBRATION_RECORDS`, `MAX_CALIBRATION_RECORD_BYTES`, prune + quarantine coverage).
  BLOCKED: independent benchmark distributions/baseline drift and held-out calibration error/interval coverage require a live model + runtime + hardware session, which this environment does not have (the same prerequisite as the G-08 qualification work). This box stays UNCHECKED; exact next action: run the published benchmark harness on the qualification host during an authorized session and attach the distributions to this record.
- I4: `cargo tree --target x86_64-pc-windows-msvc --duplicates` recorded to `.hermes-0.6/s25-tree.txt` (96 duplicate-name lines, multi-parent repeats included; 6 distinct version-divergent crate names). Shipped-runtime duplicates: `getrandom` 0.2.17 (direct `rand` 0.8.8 + `ring`→`rustls`) vs 0.3.4 (`tauri`) — cannot unify because `ring` pins 0.2; `io-lifetimes` 2.0.4 vs 3.0.1 inside `cap-std`'s own tree. Build-only duplicates (absent from the exe): `bitflags` 1.3.2/`png`/`tauri-codegen`, `hashbrown` 0.12.3 + `indexmap` 1.9.3 → `schemars` → `tauri-build` build-deps, `miniz_oxide` 0.8.9 → `png`, `getrandom` 0.4.3 → `cc`/`embed-resource`. Decision: NO dependency change — every duplicate is an upstream/platform pin or build-time only, and bumping `rand` would not remove `getrandom` 0.2 (ring). Release binary size recorded: 20 864 512 bytes at the measurement revision.
- V2: no deduplication proposal exists, so the graph + comparable binary sizes are attached to the retention rationale instead; no dependency-change checks were applicable. Unrelated sizes for context: the GH-05-era portable was 17 234 432 bytes (5222307) and the 0.6.0 candidate 20 583 936 bytes — the growth spans many accumulated feature changes, not dependency edits, and is not attributed to any single change.
- Gates at the recorded revision: `cargo fmt --check` PASS; clippy `-D warnings` 0 errors; `cargo test` 576 passed / 0 failed / 3 ignored (the new ignored measurement harness); Vitest 116; node tests 143; `npx tsc --noEmit` PASS. The debounce change preserves the cancellation and correctness tests (full suite rerun).

#### S-26 closure record — test accounting, gate deduplication, probe separation

- I1: module-level line coverage measured with `cargo llvm-cov --lib --locked --text` (cargo-llvm-cov 0.9.1 + llvm-tools-preview installed for this run; instrumented suite 576 passed / 0 failed / 3 ignored). Report: `TOTAL` regions 82.32% / functions 73.17% / lines 81.21%. Per module (lines): test_support 97.5, log_sink 95.8, recommend 95.3, local_client 93.6, catalog_db 94.1, artifact 92.8, gguf 92.0, measurement 91.9, sharing 91.2, core 90.7, tune 90.0, download 90.1, catalog 88.9, preflight 87.1, evidence 84.9, runtime 84.2, cloud 81.8, proc 79.7, health 48.0, lib 48.4. Honest scope: this is lib-unit-test coverage only — the Tauri command layer (lib.rs) and health are exercised by the packaged CDP verifiers, which this metric excludes; no threshold is enforced because no percentage substitutes for behavioral acceptance, and the targeted mutation checks stay the acceptance evidence (dozens in the package ledgers; PS1/PS2/PX1b/PV4 at the current revision).
- I2: the duplicated invocation is DOCUMENTED where it exists by design and the literal-version landmine is removed. `npm run check` includes `npm test`, which includes the release-gates file, while ci.yml/release.yml also run it explicitly — the explicit step is now labelled "(fail-fast; repeated by npm run check)" with the rationale comment in all three jobs, pinned by a release-gates test. `verify_versions.mjs` gained a no-argument manifest-consistency mode (package.json as source of truth); ci.yml (check + pr-check) uses it, and release.yml binds `${{ needs.resolve.outputs.version }}` so the tag must match the manifests at the pinned revision. The literal `v0.5.0`-only guard became a `vMAJOR.MINOR.PATCH` pattern check plus that manifest binding — a retargeted or mismatched tag still cannot publish foreign source. CI inventory before/after: policy `workflow-gates.json` counts unchanged (ci.yml check 10 / pr-check 10, release.yml quality 10; all commands still execute exactly once per job).
- I3: the local-model-tree probe is relabelled "DIAGNOSTIC, not an acceptance test" (no assertions; never counted as a pass). The ignored approved-runtime/hardware probe now has its explicit qualification job: `hardware-qualify.yml` gained a `runtime-probe` job that runs ONLY when `runtime_install_key` is supplied, executes `cargo test -- locked --lib -- --ignored --nocapture qualify_managed_runtime_on_current_host` with the install key/adapter env, and uploads the log artifact; ci.yml and release.yml are asserted to never run `--ignored` probes. The probe itself was NOT executed here — it downloads an approved runtime and runs real inference; its prerequisites are this host + an authorized session, which stays the trigger (recorded, not counted).
- V1: a removed behavioral guard provably fails its regression (PS1 literal version restored, PS2 semver pattern removed, PX1b debounce bypassed, PV4 stopPropagation removed — all CAUGHT; the first PS-scoring used the wrong reporter matcher and was corrected). The CI check inventory is compared above via the gate policy. Ignored annotations are reported as ignored by the suites, never as passes.
- Commands: `cargo fmt --check` PASS; clippy 0 errors; `cargo test` 576/0/3; release-gates 116/116; node tests 143; Vitest 116; `verify_workflow_gates` ok:true; `verify_workflow_pins` PASS; `verify_versions` ok.

#### S-27 progress record — slice 1: catalog command facade + About screen (I1/I2 remain)

- Slice 1 (Rust, I1 partial): the catalog store/cache command family moved from `lib.rs` into new `src-tauri/src/catalog_service.rs` — `catalog_cache_root`, `load_model_catalog`, `publish_loaded_catalog`, `fetch_model_catalog`, `catalog_local_models`, `read_local_catalog_rows`, `fallback_rows`, `save_user_catalog_override`, `remove_user_catalog_override`, `filter_catalog`, `catalog_facets`, `catalog_rich_facets`, `catalog_fit_budget`, `hf_token_status`, `save_hf_token`, `clear_hf_token`. `lib.rs` keeps registration (`catalog_service::`-qualified handler entries, a Tauri requirement for the macro-generated wrappers) and the download family reuses `catalog_service::catalog_cache_root`. Boundary regressions followed the code: `dc03` and the oversize/cooldown release-gates tests now read `catalog_service.rs`, and the `inspect_runtime` guard was restored to `lib.rs` after an early mis-targeted edit (recorded).
- REMNANTS in I1 (explicit): the supervised job-lifecycle facade and the runtime authorization/installer facade are NOT extracted yet; next slice = move the runtime install/health command bodies behind a service module with their existing guard tests.
- Slice 1 (FE, I2 partial): the About screen is extracted to `src/screens/AboutScreen.tsx` as a presentational component with explicit props (about, models, totalBytes, runtime, runtimeIdentity, hardware, modelRoot, onOpenExternal) — it cannot acquire data or persist anything. `App.tsx` passes those props; unused imports were pruned. The design detector reports exactly the 4 known pre-existing advisories after the extraction (it had also flagged a 13px `font-size` in the S-20 disclosure CSS; that was fixed to a documented 12px step — an honest catch, not part of the audit's list).
- REMNANTS in I2 (explicit): other screens and operation hooks still live in `App.tsx`; next slices = extract the Runtime screen, then the Tune screen with its typed operation state.
- I3: both moves were incremental, behavior-preserving, and NOT driven by line counts; every boundary test that pinned the moved code was updated to the new location rather than weakened, and the source guards (dc03 transactional-persistence, inspect_runtime trust, cooldown/oversize gates) still enforce the same invariants.
- V1: mutation PA1 (the shared refresh guard dropped from `fetch_model_catalog`) initially PASSED — exposing a real gap: no test pinned the guard's use in the command. A new guard test (`dc03_fetch_holds_the_shared_refresh_guard_for_the_whole_refresh`, asserting the guard precedes the network call) was added; PA1 is now CAUGHT. Mutation PA2 (authority resolution forced to an error) CAUGHT by the DC-04 authority tests. The full Rust suite (577/0/3), Vitest 116, node 145 and fmt/clippy/tsc all pass after the moves.
- Commands: `cargo fmt --check` PASS; clippy `-D warnings` 0; `cargo test` 577 passed / 0 failed / 3 ignored; `npx tsc --noEmit` PASS; Vitest 116; node tests 145; `npm run build` OK; detect.mjs 4 known advisories only.
