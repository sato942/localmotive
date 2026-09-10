# Localmotive — comprehensive project audit

**Repository:** [sato942/localmotive](https://github.com/sato942/localmotive)  
**Audited source:** [`e530371b056cd8e049c2246dbb151aa407bf359f`](https://github.com/sato942/localmotive/tree/e530371b056cd8e049c2246dbb151aa407bf359f), `main` snapshot captured for this audit  
**Latest release reviewed:** [v0.5.0](https://github.com/sato942/localmotive/releases/tag/v0.5.0), built from [`a4b7127f739f7420232d9b6f63da693d39128d0b`](https://github.com/sato942/localmotive/commit/a4b7127f739f7420232d9b6f63da693d39128d0b), published 10 September 2026  
**Audit mode:** read-only code and GitHub audit, local JavaScript verification, targeted isolated reproductions, independent subsystem reviews, and evidence reconciliation. No repository fixes or GitHub changes were made.

## Overall judgment

**Localmotive has a substantial engineering foundation, but v0.5.0 still has high-priority correctness and trust-boundary defects. I would prioritize a focused stabilization release before expanding functionality or broadening support claims.**

The strongest parts are unusually explicit for a young project: Rust owns most validation; runtime downloads are checked against compiled approval records and per-file manifests; paths receive Windows-specific reparse/hard-link/alternate-stream scrutiny; contained processes and listener ownership support real launch checks; catalog signatures and model hashes separate discovery from authorization; and release jobs produce actual packaged verification evidence. Documentation openly discloses unsigned installers, cloud metadata transfer, and qualification gaps. These are useful controls, not cosmetic checkboxes.

The main weakness is integration consistency. Some public inspection paths execute managed runtime binaries without the trust check used by normal launch. The catalog refresh cooldown is applied before loading cached data, so a fresh session can lose catalog access after a previous successful refresh. SQLite override and file identity handling can lose or misattribute catalog data. The warm-cache benchmark path assumes timing fields mean something different from the server's contract. AI tuning can make unbounded no-op advisor requests, and several UI/backend operations lose or confuse the model, runtime, or evidence identity they are supposed to represent.

Release confidence also needs careful interpretation. The audited main and release candidate pass CI. The v0.5.0 portable binary has 29 successful packaged checks tied to its exact SHA-256, primarily CPU runtime-management checks. The independent clean-account lifecycle run failed before testing installers because it waited for release assets while occupying the runner required to publish them. It does not prove the installer is broken; it does mean that run supplies no successful v0.5.0 clean-account lifecycle evidence. A separate verifier defect can report an uninstall PASS even when its expected executable remains.

No critical, remotely exploitable compromise was demonstrated. High-priority security findings in this report include concrete prerequisites, such as tampering with files inside a managed runtime directory; they must not be misrepresented as unauthenticated internet attacks. Conversely, a green build or zero blocking dependency advisories does not establish that all local trust rules are enforced.

## Reading guide

Read the judgment, verification ledger, and findings register first. Use the IDs to jump to the full evidence, prerequisites, fix, and regression proposal. The final remediation plan groups related fixes into reviewable work.

| Section | Contents |
|---|---|
| [Runtime installation, execution trust, processes, hardware and preflight](#runtime-installation-execution-trust-processes-hardware-and-preflight) | Detailed findings, strengths, and verification limits |
| [Catalog, downloads, SQLite integrity and GGUF parsing](#catalog-downloads-sqlite-integrity-and-gguf-parsing) | Detailed findings, strengths, and verification limits |
| [Measurement, tuning, recommendation, calibration and sharing](#measurement-tuning-recommendation-calibration-and-sharing) | Detailed findings, strengths, and verification limits |
| [Frontend correctness, user experience and accessibility](#frontend-correctness-user-experience-and-accessibility) | Detailed findings, strengths, and verification limits |
| [Cloud integration, IPC and operational controls](#cloud-integration-ipc-and-operational-controls) | Detailed findings, strengths, and verification limits |
| [GitHub delivery, releases, governance and project health](#github-delivery-releases-governance-and-project-health) | Detailed findings, strengths, and verification limits |
| [Tests, dependencies, maintainability and documentation](#tests-dependencies-maintainability-and-documentation) | Detailed findings, strengths, and verification limits |
| [Remediation plan](#remediation-plan-and-release-acceptance) | Ordered work packages and release acceptance |
| [File inventory](#appendix-repository-file-inventory) | Every tracked path and audit treatment |

## Scope, method, and limits

The audit inventoried all **152 tracked files**, including **20 Rust files** (19 under `src-tauri/src` plus `build.rs`), **30,643 physical Rust lines**, **5,602 TypeScript/TSX lines**, four GitHub workflows, 21 JavaScript verification/build scripts, both dependency lockfiles, the catalog, seven runtime content manifests, release schemas/evidence, and documentation. Physical line counts include tests, comments, and blank lines; they are not complexity or coverage scores.

All major application subsystems were reviewed across six independent workstreams and a cross-cutting cloud/IPC review. This is a comprehensive repository audit within the accessible evidence, not a claim that every possible input, machine, binary, operating-system policy, or external service has been exhaustively tested.

| Area | Work performed | What remains outside the verified scope |
|---|---|---|
| Source and architecture | Pinned local clone; source and call-path review; data/operation identity tracing | Formal verification; exhaustive path coverage |
| Security | Runtime trust routes, filesystem/archive handling, IPC, credentials, network boundaries, input limits, sharing redaction | Full penetration test; Windows ACL/keyring exploitation; complete history secret scan |
| Correctness | Catalog/SQLite, download ranges, runtime lifecycle, launch profiles, benchmark/tuning/calibration, UI state | Real GGUF inference across hardware/runtime combinations |
| Tests/build | Local Node install, `npm run check`, `npm audit`; GitHub Rust/packaged logs | Independent local Rust/Clippy execution; Windows packaged UI run in this Linux workspace |
| Supply chain | Both lockfiles, pinned actions, approved manifest validation, audit logs, release asset metadata | Independent download/rehash of every binary; reproducible rebuild; package license legal review |
| Delivery/governance | All 156 accessible workflow runs, 110 main-history commits, three releases, tags, four PRs, branch/ruleset summary | Privileged administration settings, private alerts/secrets, owner machine configuration |
| UX/accessibility | Source inspection of labels, controls, state changes, focus/keyboard patterns and styling; calculated CSS color contrasts | Screen-reader session, Windows high-contrast/DPI tests, sampled rendered contrast and visual audit |
| External services | GitHub reads; npm advisory query; authoritative framework/server contract documentation | Paid cloud calls, actual OAuth login, provider quotas/account state, gated HF download credentials |

GitHub API and repository observations are snapshot facts. The default branch, releases, rules, dependency advisories, and upstream service contracts may change after capture. Repository source links are pinned to the audited SHA; live workflow and release links identify the evidence inspected. The one commit between release source and audited main is a post-shipping documentation ledger update, not a different application implementation.

### Evidence labels

- **Executed locally:** the audit ran the command or isolated reproduction in this workspace and inspected the result.
- **Observed remotely:** GitHub metadata, step output, logs, or release evidence were read. This is independent inspection of recorded evidence, not an independent repeat of that Windows run.
- **Source-confirmed:** a complete code path establishes the behavior, but the application scenario was not executed here.
- **Conditional risk:** impact depends on a stated platform, concurrency, input, or attacker prerequisite that was not reproduced.
- **Improvement / validation gap:** a missing control or test merits action without proving a shipped exploit or user-visible failure.

Some reproduced SQL findings execute the repository's SQL statements through Python's SQLite binding. They establish SQL semantics; they do not reproduce Rust error handling, Windows file-sharing behavior, or a complete packaged UI session. The catalog builder reproduction executes the actual JavaScript script with controlled time and mock network responses. These distinctions matter.

### Priority scale

| Priority | Meaning in this report |
|---|---|
| Critical | Demonstrated severe compromise or broadly destructive failure with weak prerequisites. None established. |
| High | Core workflow failure, meaningful trust-check bypass, data/provenance loss, uncontrolled paid work, or unreliable release gate. Address before confidently broadening the release. |
| Medium | Conditional correctness/availability issue, significant hardening or coverage gap, or misleading evidence under a narrower scenario. |
| Low | Maintenance, documentation, usability, or process improvement with limited immediate impact. |

These are engineering priorities, not CVSS scores. A high-priority correctness bug is not automatically a high-severity security vulnerability. The detailed finding carries its prerequisites and evidence level; the headline alone is insufficient for exploitability judgments.

## Architecture and trust model

The product is a Windows x64 desktop control plane for local `llama.cpp` inference. React/TypeScript renders model discovery, catalog browsing, runtime management, launch profiles, tuning, and measurement. Tauri exposes a large Rust command surface in `lib.rs`. Rust modules manage approval records, file inspection, subprocesses, HTTP downloads, SQLite mirroring, launch validation, measurements, calibration, and cloud proposals. There is no application backend service operated by Localmotive; SQLite is a local mirror, and the inference server is a child process. The cloud advisor is optional and external.

| Boundary | Input crossing it | Intended authority/control | Principal audit concern |
|---|---|---|---|
| WebView → Rust | Paths, profiles, catalog queries, operation requests | Rust validation and managed operation state | Heavy synchronous work, incomplete common guards, conflicting operation identities |
| GitHub → runtime installation | Release metadata and ZIP payloads | Compiled approvals, archive digest, per-file manifests, safe extraction | Inspection bypasses, legacy root migration, expensive repeated verification |
| Signed catalog → local SQLite/UI | Models and file metadata | Signed snapshot authorizes downloads; DB is a presentation mirror | Cooldown blocks cache loading; schema identity and override provenance errors |
| Hugging Face → disk | Large model bytes, redirects/ranges | Authorized catalog target, safe paths, resume validation, final hash | No-range fallback above 8 MiB; immutable revision/freshness gaps |
| Model files → parsers/server | GGUF metadata and shards | Bounded validation, artifact grouping, runtime checks | Aggregate allocation/work limits; companion and split metadata assumptions |
| Localmotive → llama-server | Argument vector and HTTP requests | Capability checks, contained process, owned listener | TLS/auth mismatch, cancellation/serialization, stale benchmark identity |
| Localmotive → cloud advisor | Hardware/profile/metadata/trial brief | Fixed providers, secure key storage, proposal whitelist | Aggregate request budget, cancellation, callback robustness |
| Evidence → calibration/export | Measurements, identities, errors | Versioned manifests and provenance | Incomplete compatibility keys, repeated anchors, raw-error leakage |
| Source → public release | Tags, build artifacts, test evidence | CI dependencies, inventory and checksum/readback gates | Mutable tags, no premerge enforcement, disconnected lifecycle workflow |

The largest architectural risk is duplicated decision paths. Runtime trust is strong at `prepare_launch` but weaker at other probe entry points. The new measurement harness and older tuner benchmark implement different contracts. Catalog data exists in the signed snapshot, SQLite, frontend snapshot, and filtered rows. Profile selection, runtime selection, currently running server, and benchmark identity are separate state values. Consolidating the authoritative boundary for each of these is more valuable than a broad cosmetic refactor.

The codebase has substantial test code, so file length alone should not drive rewrites. Nevertheless, `runtime.rs` is 7,069 lines, `lib.rs` 4,226, `core.rs` 2,996, `download.rs` 2,383, and `App.tsx` 2,136. These files now combine several responsibilities and make bypasses and stale-state bugs easier to introduce. Split around stable services and contracts after reproducing the affected behavior: runtime authorization/probing, catalog store/refresh, operation coordinator, transport client, measurement identity, and screen-level state.

## Verification ledger

| Check | Result | Evidence interpretation |
|---|---|---|
| Local clone SHA | Exact match to `e530371…` | All source references pinned |
| Source changes | None; `git diff --stat` and `git status --porcelain` empty after checks | Audit did not change project behavior |
| Node / npm | Node 24.19.0 / npm 11.9.0 | Local environment differs from CI's Node 20 configuration |
| `npm ci --ignore-scripts --no-audit --no-fund` | Exit 0, 108 packages installed | Dependency lifecycle scripts deliberately skipped; subsequent full frontend check/build passed |
| `npm run check` | Exit 0 | TypeScript, tests, catalog/branding/research/qualification/icon gates and production build all completed |
| Vitest | 52 passed, one test file | Pure/model tests; not component or packaged UI coverage |
| Node release-gate suite | 80 passed, zero failed | Includes behavior tests and many source/wiring assertions; scope discussed later |
| Production frontend | Vite 7.3.6 build passed; 1,838 modules transformed | JS 323.29 kB / 95.40 kB gzip; CSS 40.43 kB / 8.41 kB gzip; HTML 1.14 kB |
| `npm audit --json` | Exit 0; zero reported advisories | Current registry response, 168 dependencies in audit metadata; not a source security verdict |
| Rust tests/format/Clippy | Not executed locally: Cargo/Rust toolchain unavailable | Remote CI logs independently inspected instead |
| Remote Rust tests, audited main | 403 passed, zero failed, two ignored | Detailed ignored tests and audit warnings in delivery section |
| Runtime content manifest consistency | Seven manifests passed independent structural/hash-anchor checks | 419 entries; 3,752,868,564 declared extracted bytes; actual archive payloads not downloaded |
| Isolated SQLite reproduction | Confirmed stale file retention, cross-row file collision, and provenance reassignment behavior | Exact SQL semantics; not Windows DB-file deletion semantics |
| Actual catalog builder with controlled date/fetch | Reproduced stale July model retained under December date | No repository edit and no live provider request |
| v0.5.0 packaged record | 29 PASS tied to release portable digest | Remote evidence, chiefly CPU/runtime checks |
| v0.5.0 clean-account lifecycle | Failed waiting for not-yet-published assets | No successful installer lifecycle result from this run |

The first `npm run check` produced a complete successful log but its execution wrapper did not return a usable final status. It was repeated once to resolve that specific uncertainty; the repeat explicitly returned exit 0. It is not two independent test environments. Logs from the explicit repeat and npm audit are included in the evidence bundle.

The qualification policy check reports 19 disclosed P0 rows and one L4 attestation. Here **P0 is the project's hardware qualification matrix terminology**, not this audit's defect severity. Passing that policy checker means the evidence/disclosure schema meets its rules, not that all 19 hardware rows passed packaged qualification.


## Findings register

The report contains **72 prioritized findings and action items**: 19 High, 43 Medium, and 10 Low. They include bugs, conditional risks, and control/coverage improvements; this is not a count of exploitable security vulnerabilities. Related UI/backend symptoms are cross-referenced. FE-10 was consolidated into DC-04; the frozen catalog date is counted once as QD-01.

| ID | Priority | Finding | Evidence basis |
|---|---|---|---|
| [RT-01](#rt-01) | High | New runtime installs can enter an unrecoverable legacy-directory launch loop | Source review; scenario execution limits in finding |
| [RT-02](#rt-02) | High | Managed trust checks are bypassed by tuning preparation and the older runtime-health command | Source review; scenario execution limits in finding |
| [RT-03](#rt-03) | Medium | Health cancellation is ignored during an in-flight completion request | Source review; scenario execution limits in finding |
| [RT-04](#rt-04) | Medium | Content verification does not retain file protection through executable launch | Source review; scenario execution limits in finding |
| [RT-05](#rt-05) | Medium | Several bounded-input claims are bypassed by error or discovery paths | Source review; scenario execution limits in finding |
| [RT-06](#rt-06) | Medium | Runtime selection and listing repeatedly hash gigabytes of installed files without cancellation | Source review; scenario execution limits in finding |
| [RT-07](#rt-07) | Medium | The archive replacement regression exercises a helper omitted from production | Source review; scenario execution limits in finding |
| [RT-08](#rt-08) | Low | Windows last-error status is read after another Win32 call | Source review; scenario execution limits in finding |
| [RT-09](#rt-09) | Medium | Identical GPU names are matched by enumeration order, not physical identity | Source review; scenario execution limits in finding |
| [DC-01](#dc-01) | High | Persisted catalog refresh cooldown makes the catalog empty after app restart | Source review; scenario execution limits in finding |
| [DC-02](#dc-02) | High | HTTP 200 fallback rejects every range-ignoring file larger than 8 MiB | Source review; scenario execution limits in finding |
| [DC-03](#dc-03) | Medium | Catalog refresh rebuilds healthy mirrors and skips corruption recovery | Source review; scenario execution limits in finding |
| [DC-04](#dc-04) | Medium | User-added catalog entries cannot be downloaded and disappear from filtering | Source review; scenario execution limits in finding |
| [DC-05](#dc-05) | Medium | Override UPSERTs can replace curated provenance, steal files, and leave mislabeled rows | Executed source SQL |
| [DC-06](#dc-06) | Medium | Override edits are not replacement operations and lack atomicity/bounds | Executed source SQL |
| [DC-07](#dc-07) | Medium | Some network/cache failures bypass the promised last-good catalog fallback | Source review; scenario execution limits in finding |
| [DC-08](#dc-08) | High | GGUF byte limit does not sufficiently bound heap growth or parser work | Source review; scenario execution limits in finding |
| [DC-09](#dc-09) | Medium | Quantization labels are extracted from arbitrary final filename suffixes | Source review; scenario execution limits in finding |
| [DC-10](#dc-10) | Low | Mutable file revisions weaken reproducible catalog availability | Source review; scenario execution limits in finding |
| [DC-11](#dc-11) | Medium | Completion publication can overwrite a file created during transfer; local tampering defense remains incomplete | Source review; scenario execution limits in finding |
| [DC-12](#dc-12) | Low | Resume checkpoints are durable before data is guaranteed durable, and final hashing is uncancellable | Source review; scenario execution limits in finding |
| [MT-01](#mt-01) | High | Default warm-cache benchmark rejects legitimate prompt-cache hits | Source and pinned upstream b10816 |
| [MT-02](#mt-02) | High | Share export leaks raw trial errors through the summary | Source review; scenario execution limits in finding |
| [MT-03](#mt-03) | High | No-op proposals can cause unlimited paid advisor requests | Source review; scenario execution limits in finding |
| [MT-04](#mt-04) | High | Cancelling AI tuning does not cancel its lifecycle | Source review; scenario execution limits in finding |
| [MT-05](#mt-05) | High | Benchmarks and quality checks can outlive their server identity | Source review; scenario execution limits in finding |
| [MT-06](#mt-06) | Medium | Local TLS/authentication configuration and internal transports disagree | Source review; scenario execution limits in finding |
| [MT-07](#mt-07) | Medium | Calibration compatibility does not identify the actual execution shape | Source review; scenario execution limits in finding |
| [MT-08](#mt-08) | Medium | Repeated clicks on one result manufacture independent calibration anchors | Source review; scenario execution limits in finding |
| [MT-09](#mt-09) | Medium | Quality evidence is not tied to full model/configuration identity | Source review; scenario execution limits in finding |
| [MT-10](#mt-10) | Medium | Missing metrics invalidate the claimed Pareto frontier | Source review; scenario execution limits in finding |
| [MT-11](#mt-11) | Medium | AI tuning does not actually benchmark at the requested context workload | Source review; scenario execution limits in finding |
| [MT-12](#mt-12) | Medium | Rich launch failures can destroy the benchmark record they should explain | Source review; scenario execution limits in finding |
| [MT-13](#mt-13) | Medium | Validation is fragmented and does not enforce complete-record consistency | Source review; scenario execution limits in finding |
| [MT-14](#mt-14) | Low | Calibration model invariants and staleness are not enforced uniformly | Source review; scenario execution limits in finding |
| [MT-15](#mt-15) | Low | Scanner shard completeness differs from artifact validation | Source review; scenario execution limits in finding |
| [FE-01](#fe-01) | High | Selected model/runtime and the profile actually launched can diverge | Source review; scenario execution limits in finding |
| [FE-02](#fe-02) | High | Tuning results are not bound to the model/runtime that produced them | Source review; scenario execution limits in finding |
| [FE-03](#fe-03) | Medium | Async cloud, GGUF metadata, port suggestions and command previews accept stale responses | Source review; scenario execution limits in finding |
| [FE-04](#fe-04) | High | Typing a profile repeatedly runs expensive synchronous native launch validation | Source review; scenario execution limits in finding |
| [FE-05](#fe-05) | High | Evidence and active measurement state disappear when leaving Benchmark | Source review; scenario execution limits in finding |
| [FE-06](#fe-06) | Medium | Preflight evidence stays visible after its input assumptions change; final adapter cannot be deselected | Source review; scenario execution limits in finding |
| [FE-07](#fe-07) | Medium | Cancellation reuses the operation busy state and clears it before the benchmark has ended | Source review; scenario execution limits in finding |
| [FE-08](#fe-08) | Medium | Raw extra-argument field cannot normally accept multiple tokens by typing | Source review; scenario execution limits in finding |
| [FE-09](#fe-09) | Medium | Corrupt persisted JSON can crash the UI; write failures can be reported as operation failures | Source review; scenario execution limits in finding |
| [FE-11](#fe-11) | Medium | Download cancellation targets the currently edited destination, not the destination of the running job | Source review; scenario execution limits in finding |
| [FE-12](#fe-12) | Medium | Model-folder and download-destination fields lose the keyboard-focus ring | Source review; scenario execution limits in finding |
| [FE-13](#fe-13) | Medium | Several controls expose incomplete names/structures to assistive technology | Source review; scenario execution limits in finding |
| [FE-14](#fe-14) | Medium | Small text colors fail 4.5:1 contrast; compact layouts leave evidence controls cramped | Source review; scenario execution limits in finding |
| [FE-15](#fe-15) | Medium | Runtime capability and status labels overstate what was established | Source review; scenario execution limits in finding |
| [FE-16](#fe-16) | Medium | Legacy controls and new evidence tools have inconsistent active-operation/result ownership | Source review; scenario execution limits in finding |
| [FE-17](#fe-17) | Low | Tested workload validators are disconnected from production UI; domain defaults/rules are duplicated | Source review; scenario execution limits in finding |
| [IPC-01](#ipc-01) | High | Normal server startup blocks the main thread and prevents stop/status during startup | Source review; scenario execution limits in finding |
| [IPC-02](#ipc-02) | Medium | Content Security Policy is disabled | Source review; scenario execution limits in finding |
| [CLD-01](#cld-01) | Medium | OAuth callback parsing lacks a complete request and resource budget | Source review; scenario execution limits in finding |
| [OPS-01](#ops-01) | Medium | Server log writes are unbounded and reuse the same filename | Source review; scenario execution limits in finding |
| [GH-01](#gh-01) | High | default branch has no enforced premerge verification | GitHub/source observations |
| [GH-02](#gh-02) | High | mutable release tags undermine stable source and evidence identity | GitHub/source observations |
| [GH-03](#gh-03) | High | v0.5.0 clean-account verification fails because its scheduling waits on itself | GitHub/source observations |
| [GH-04](#gh-04) | High | Sandbox lifecycle PASS can conceal incomplete uninstall or wrong-version upgrade | GitHub/source observations |
| [GH-05](#gh-05) | Medium | new v0.5 catalog and SQLite features lack packaged end-to-end release coverage | GitHub/source observations |
| [GH-06](#gh-06) | Medium | failure evidence is lost by the Sandbox host harness | GitHub/source observations |
| [GH-07](#gh-07) | Medium | support statements and historical corrections remain inconsistent with public evidence | GitHub/source observations |
| [GH-08](#gh-08) | Medium | supply-chain provenance is incomplete beyond checksum integrity | GitHub/source observations |
| [GH-09](#gh-09) | Low | governance and ongoing maintenance processes are not yet established | GitHub/source observations |
| [GH-10](#gh-10) | Medium | hardware “attestation” stub can declare HOST_MATCH despite a mismatched host | GitHub/source observations |
| [QD-01](#qd-01) | Medium | Catalog builder never advances its 90-day cutoff | Executed builder with controlled inputs |
| [QD-02](#qd-02) | Medium | New product flows lack automated component and orchestration contract tests | Source review; scenario execution limits in finding |
| [QD-03](#qd-03) | Medium | Packaged UI harness couples to private React internals and injects states without exercising their real transition | Source review; scenario execution limits in finding |
| [QD-04](#qd-04) | Low | Active-looking docs disagree on shipping, persistence, verification status and architecture | Source review; scenario execution limits in finding |
| [QD-05](#qd-05) | Low | Build instructions understate the Node minimum and toolchain is not fully reproducible | Source review; scenario execution limits in finding |
| [QD-06](#qd-06) | Low | Imported llama-server reference has missing provenance and broken relative links | Source review; scenario execution limits in finding |

## Runtime installation, execution trust, processes, hardware and preflight

Audited commit: `e530371b056cd8e049c2246dbb151aa407bf359f`, independently verified by `git rev-parse HEAD`. Read `AGENTS.md`. Primary review covered production code and relevant tests in `src-tauri/src/runtime.rs`, `proc.rs`, `artifact.rs`, `health.rs`, and `preflight.rs`, plus compiled runtime manifests and relevant call chains in `lib.rs`, `core.rs`, and `evidence.rs`. This is a read-only audit; no source was changed. Static conclusions are distinguished from Windows behavior that was not executed. Cargo/Windows runtime execution was unavailable to this reviewer.

### Overall assessment

The managed-runtime installer has substantial defense in depth: approval is compiled into the application, executable payload files are bound to a separate compiled content manifest, downloads must match approved size and SHA-256, ZIP extraction is bounded and capability-relative, and published installs are reverified. This materially limits the consequences of a malicious remote release response or a changed downloaded archive. It is not accurate to describe this installer as simply downloading an arbitrary URL and executing it.

The principal problems are integration inconsistencies around those protections. Two public workflows can execute managed binaries without the content check used by ordinary inspection/launch. The legacy-root installation policy can put a newly approved runtime in a location that the launch policy categorically rejects. Health cancellation is not responsive while its blocking completion HTTP request is pending. Several claimed resource and verification guarantees are weaker than their comments or tests suggest.

### Findings with direct source evidence

### RT-01

**New runtime installs can enter an unrecoverable legacy-directory launch loop.**

**Priority: High.**

**Type:** confirmed control-flow defect; Windows packaged reproduction still required.

**Evidence:** [runtime.rs:2218–2226](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L2218-L2226), `3755–3778`, `3780–3792`, `3623–3640`; integration at [lib.rs:413–415](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L413-L415) and `1135–1138`. Existing regression at [runtime.rs:6118–6150](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L6118-L6150) asserts the old fallback selection but never composes it with the newer launch trust policy.

`managed_runtime_root_in(primary, legacy)` returns the legacy root when primary does not exist and legacy does. It can also prefer legacy when primary has no verified installs and legacy does. `install_runtime` uses this root for the newly approved build, writes its compiled-approval record, fully verifies it, and returns success. However, `verify_managed_runtime_for_launch` verifies only the primary Localmotive root and then unconditionally rejects any path in the GGUF Pilot legacy root, with the instruction to install a current runtime.

**Concrete prerequisite/reproduction:** An upgraded machine has `%LOCALAPPDATA%/GGUF Pilot/runtimes` and lacks `%LOCALAPPDATA%/Localmotive/runtimes`. Even an empty legacy directory is sufficient. Install a current CPU runtime. It is placed under the old root; inspect or launch it; it is rejected as legacy. Reinstall chooses the same root and may return `reused: true`, so the suggested recovery cannot repair the problem. No hostile actor is required.

**Impact:** Existing users can download a valid runtime and still be unable to inspect, launch, or tune it through protected paths. The installation result and recovery text contradict the actual launch policy.

**Remediation:** Always publish new approved installations into the primary root. Handle legacy discovery/migration separately. If approved content in a legacy location is intentionally supported, explicitly verify it with the same compiled manifest policy rather than rejecting it by location. Preserve old files until migration succeeds.

**Regression:** Create an empty legacy directory and an absent primary directory; run root selection, installation/publish with an inert verified fixture, and launch validation as one scenario. Require the returned path to pass the same trust gate used by the user-facing inspect command. Also cover corrupt legacy records, a populated primary root, and repair/reuse.

### RT-02

**Managed trust checks are bypassed by tuning preparation and the older runtime-health command.**

**Priority: High.**

**Type:** confirmed integration defect, shared with cross-cutting review; avoid duplicate headline finding.

**Evidence:** [lib.rs:2587–2589](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2587-L2589) directly invokes `core::inspect_runtime` during `start_tuning`; [core.rs:1272–1275](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs#L1272-L1275) validates only regular/non-reparse paths and executes `--version` and `--help`; actual spawn at [core.rs:1241–1249](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs#L1241-L1249). The protected public inspect command applies `runtime::verify_managed_runtime_for_launch` at [lib.rs:1135–1138](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L1135-L1138). Separately, public `check_runtime_health` at [lib.rs:1151–1160](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L1151-L1160) calls `core::check_runtime_health` without compiled-content trust verification. [core.rs:1657–1664](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs#L1657-L1664) checks only that server and sibling CLI are regular non-reparse files, and `1678–1679` executes `llama-cli.exe`.

**Prerequisite:** A managed runtime executable or its sibling CLI has been replaced/tampered with, and the user invokes tuning or the older health IPC workflow. This is not a remote-code-execution claim from an unprivileged network input alone: the attacker needs a way to alter the managed install or invoke the exposed workflow with an already selected hostile runtime. Managed tamper resistance is nevertheless an explicit existing application invariant.

**Impact:** Arbitrary native code in a tampered managed installation executes before the protected launch path can reject it. Passing `--help` or `--list-devices` does not make executing an untrusted EXE safe. The later `spawn_server` trust check is too late for the tuning preparation probe.

**Remediation:** Centralize the managed-content policy at the lowest executable probe boundary, and cover companion executable selection. Prefer a verified-runtime object rather than a raw path that can be passed around without its trust state. Ensure manual external runtime selection remains an intentional separate policy.

**Regression:** Replace an approved managed EXE or CLI fixture with an inert binary that writes a sentinel on any invocation. Through both Tauri workflows, assert rejection without the sentinel appearing. Include a tampered DLL with an unchanged server EXE.

### RT-03

**Health cancellation is ignored during an in-flight completion request.**

**Priority: Medium.**

**Type:** confirmed cancellation/control-flow defect; no live stalled Windows HTTP test executed here.

**Evidence:** [health.rs:654–691](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/health.rs#L654-L691) creates a blocking HTTP request with `MODEL_OPERATION_TIMEOUT = 120s` ([health.rs:12](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/health.rs#L12)). Cancellation is checked once immediately before the request at `1151–1166`, and `completion_request` does not receive a cancellation token. Public cancel merely sets the flag ([lib.rs:2373–2381](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2373-L2381)). After the request succeeds there is no subsequent cancellation check before recording completion/cleanup as passed ([health.rs:1181–1281](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/health.rs#L1181-L1281)).

**Prerequisite:** A managed health server stalls after accepting `/completion`, or completion is slow, and the user presses Cancel.

**Impact:** The application can acknowledge cancellation while the health worker and child server continue up to the 120-second request deadline. If the request eventually returns the expected output, a user-cancelled run can finish as passed. The separate stage called Cancellation at `1233–1252` kills an idle server after successful completion; it does not validate cancellation of an active request, so a passing stage does not cover this failure.

**Remediation:** Use an asynchronous cancellable request with a cancellation select, or a supervised worker that can terminate the contained server promptly and cause the request to fail when the token is set. Propagate Cancelled consistently, including cancellation received while body reading or immediately after response completion.

**Regression:** An owned loopback fixture accepts a completion request and deliberately withholds the response. Cancel after acceptance. Assert bounded return, a Cancelled result, no positive completion stage, and child/listener cleanup. Add a response-versus-cancel race test.

### RT-04

**Content verification does not retain file protection through executable launch.**

**Priority: Medium.**

**Type:** confirmed check/use gap; attack race not executed on Windows.

**Evidence:** [runtime.rs:3197–3205](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L3197-L3205) opens files with `FILE_SHARE_READ`, blocking conflicting writes/deletion while each file is open. `digest_regular_file` returns only `(size, hash)` (`3301–3353`), dropping that handle. `verify_installed_runtime` hashes files serially (`3547–3555`) and returns a `PathBuf` (`3561`). Callers later execute by path. For managed health the gap is particularly long: content is verified in `managed_health_context` (`3964–3981`); [lib.rs:2340–2366](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2340-L2366) may then download the 101,016,128-byte health model before `run_managed_health` eventually starts CLI, bench, and server at [health.rs:756](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/health.rs#L756), `819`, `891`, and `1004`.

**Prerequisite:** A concurrent process can modify the current user's writable runtime installation during the interval after verification and before image/DLL loading. This is the same-user tamper threat already addressed by the application's reparse/hard-link/ADS protections; it does not grant new filesystem permissions.

**Impact:** A previously verified EXE or DLL can change after its handle is closed but before it is loaded. The application may execute content different from the content it just approved. Serial hashing means the first file's gap includes hashing every later file. Repeating the check before spawn shortens but does not close the race.

**Remediation:** Return a verified-runtime lease retaining read-only sharing handles for the EXE, approved DLLs, and relevant directory identities through process creation/loading. Ensure any runtime repair/replacement operation coordinates with those leases. If practical constraints make same-user adversarial tamper protection out of scope, explicitly scope the guarantee to verification at a point in time instead of claiming execution identity.

**Regression:** Coordinate a writer with the point after a successful verification and before spawn. Require replacement either to fail while a lease exists or to be detected before any marker binary can execute. Cover DLL replacement and the delayed health-model download path.

### RT-05

**Several bounded-input claims are bypassed by error or discovery paths.**

**Priority: Medium.**

**Type:** confirmed missing bounds; denial-of-service effects not executed.

**Evidence and prerequisites:**

- [runtime.rs:2398–2410](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L2398-L2410) reads a HTTP 403 body using `response.text().await` before the 2 MiB success-body checks at `2421–2454`. An unexpectedly large 403 from the HTTPS endpoint, an enterprise interception proxy, or a test transport is read fully and may be embedded in the error string. The 15-second HTTP timeout limits time, not bytes or peak memory. Ordinary GitHub rate-limit responses are small; this is not a claim that GitHub currently sends oversized responses.
- [runtime.rs:3580–3585](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L3580-L3585) reads a discovered `runtime.json` with `fs::read` before checking its 64 KiB size. A huge local record is allocated completely before rejection. `verify_installed_runtime` has a prior metadata bound (`3513–3522`), but this earlier discovery read defeats it.
- `collect_install_files` counts only files ([runtime.rs:3363–3395](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L3363-L3395)); directories are pushed and skipped before the limit is evaluated. Arbitrarily many empty subdirectories can make managed verification traverse far more than the advertised 20,000 entries. The ZIP extraction entry cap does not constrain an already installed tree modified later.

**Impact:** Avoidable memory allocation and disk/CPU work in code whose documented design is bounded and fail-closed. The latter two cases require local corruption or an actor with write access to the runtime tree.

**Remediation:** Reuse one bounded body reader for success/error responses; open and read local records with `take(limit + 1)` after checking a handle's metadata; cap all visited filesystem entries and depth, not just files. Preserve error categories without embedding arbitrarily large upstream bodies.

**Regression:** Chunked 403 response of 2 MiB + 1 byte; an oversized sparse runtime record; more than the allowed count of empty directories; exact-limit boundary cases. Assert early rejection and bounded retained bytes.

### RT-06

**Runtime selection and listing repeatedly hash gigabytes of installed files without cancellation.**

**Priority: Medium.**

**Type:** confirmed algorithmic I/O cost; elapsed time not benchmarked.

**Evidence:** `list_managed_runtimes_in` verifies every candidate ([runtime.rs:1045](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L1045)); verification hashes the complete compiled inventory (`3547–3555`). Even `managed_runtime_root_in`, which is conceptually root selection, invokes full managed enumeration (`2219`). `describe_runtime` IPC hashes the chosen managed runtime again ([lib.rs:1164–1168](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L1164-L1168)). `install_runtime` and managed health use root selection and additional full verification. None of the file-verification functions accepts cancellation.

**Measured from checked-in manifest data:** The seven active compiled manifests contain 419 file entries totaling 3,752,868,564 bytes. CUDA 12.4 alone totals 1,166,447,389 bytes and ROCm 1,124,428,061 bytes. A fully populated listing therefore reads approximately 3.75 GB of payload data before returning, apart from filesystem overhead; this is an exact sum of manifest declarations, not a measured disk transfer or elapsed-time benchmark. OS caching can reduce physical I/O but does not eliminate hashing CPU work.

**Impact:** Repeated selection/list/root operations can be slow on HDDs, network/redirected profiles, or under antivirus scanning. Cancellation of an install/health run may appear ineffective during preparatory full verification. Multiple frontend requests can duplicate work.

**Remediation:** Separate cheap discovery/root policy from explicit content verification. Coalesce simultaneous verification for the same installation, put expensive work on a blocking worker, expose progress/cancellation, and use a carefully designed verified-install lease/cache with safe invalidation. Do not simply trust mtime/size as cryptographic content verification.

**Regression/measurement:** Instrument bytes hashed and number of verification jobs for listing, selecting, and starting one managed runtime. Validate that root lookup performs no full content scan, concurrent requests coalesce, and cancellation terminates hash work promptly. Benchmark an all-backend install on a representative supported Windows host.

### RT-07

**The archive replacement regression exercises a helper omitted from production.**

**Priority: Medium.**

**Type:** confirmed disconnected test implementation.

**Evidence:** `extract_verified_zip_with_limits_and_cancel` is annotated `#[cfg(test)]` ([runtime.rs:2921–2927](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L2921-L2927)). It rehashes the downloaded archive and checks expected size. The regression at [runtime.rs:6334–6378](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L6334-L6378) exercises that helper. Production `extract_zip` directly calls `extract_zip_with_limits_and_cancel` (`2964–2970`), which opens the archive normally (`2793`) without the rehash. `install_runtime` calls this production helper after the downloader returns (`3844`).

**Impact:** The archive-replacement test does not prove that the production extraction boundary enforces the behavior described by its helper's security comment. A same-user replacement of the downloaded ZIP can reach parsing/decompression. However, final compiled per-file verification at `3862–3875` still rejects unapproved payload content, so this is not by itself proof that a modified downloaded archive becomes an accepted runnable install. Bounded extraction also reduces the parser/decompression resource exposure.

**Remediation:** Exercise the production installer/extraction path in the test. If the intended invariant is approved archive bytes at extraction, hash and extract from the same opened and appropriately protected file handle; a rehash followed by reopening merely creates a second check/use race. Remove test-only duplicate security implementations.

**Regression:** Replace the archive between completed download and actual production extraction; require rejection before file extraction. Test a same-size mutation, changed size, symlink/reparse replacement, and cancellation during verification.

### RT-08

**Windows last-error status is read after another Win32 call.**

**Priority: Low.**

**Type:** confirmed API-ordering defect; observed failure on a supported Windows build not established.

**Evidence:** [runtime.rs:3278–3286](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L3278-L3286) calls `FindNextStreamW`, then `FindClose`, then reads `GetLastError` to distinguish expected `ERROR_HANDLE_EOF`. [runtime.rs:79–103](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L79-L103) similarly calls `GetProcessMemoryInfo`, closes the handle, then retrieves the failure error. Microsoft specifies that useful last-error state should be captured immediately because subsequent calls can overwrite it: [GetLastError documentation](https://learn.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror).

**Impact:** The stream verifier can interpret the close operation's error state instead of the enumeration's state and wrongly reject an otherwise valid managed file on an implementation that changes last-error state. Peak-memory diagnostics can also show the wrong reason. No claim is made that `FindClose` overwrites the value on every Windows version.

**Remediation:** Capture the last error immediately after the failed operation, before handle cleanup; use an RAII guard and preserve the original result/error.

**Regression:** Windows-only API abstraction/fixture that makes the cleanup operation change thread-local last error, then proves the original enumeration/measurement error is retained. Include a normal file with only the default data stream and a file with an extra stream.

### RT-09

**Identical GPU names are matched by enumeration order, not physical identity.**

**Priority: Medium.**

**Type:** confirmed ambiguous matching; wrong ordering not reproduced on hardware.

**Evidence:** [runtime.rs:1196–1199](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L1196-L1199) queries only `name,driver_version,memory.total,memory.used` from nvidia-smi. `merge_nvidia_probe_observations` pairs each row with the first unmatched DXGI adapter with the same name (`1138–1164`). DXGI adapters have a specific LUID identity (`444–450`) but the NVIDIA rows carry no identity that can be joined to it. Managed health subsequently resolves device selection by an adapter name substring and rejects multiple matches ([health.rs:457–487](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/health.rs#L457-L487)).

**Prerequisite:** Two physical NVIDIA adapters have identical reported names, and nvidia-smi order differs from DXGI high-performance enumeration order or individual GPUs have different usage.

**Impact:** Device A can receive device B's observed usage/capacity values under A's LUID. Managed health fails to identify an explicitly selected physical adapter when both identical names appear, despite the request carrying an adapter ID. The existing DXGI budget fields remain separate, so this is not evidence that every memory estimate uses the wrong NVIDIA figure.

**Remediation:** Query and correlate stable physical identity (PCI location/UUID with an appropriate Windows mapping), or retain ambiguous telemetry as unassigned observations instead of presenting it as exact per-adapter evidence. Provide an explicit runtime-device/LUID mapping for health.

**Regression:** Two same-name adapters with opposite enumeration orders and distinct usage numbers; require correct stable mapping or Unknown/ambiguous evidence. Same-name health selection must not be silently assigned to the wrong GPU.

### Additional limitations and engineering observations

1. **Cleanup timeouts are not enforced around blocking cleanup calls.** [proc.rs:163–172](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/proc.rs#L163-L172) directly invokes kill/start_kill and blocking wait. `output_with_timeout_and_cancel` joins stdout/stderr readers after termination without a deadline (`212–258`). Health checks its nominal 10-second cleanup limit only after `terminate_and_wait` has returned ([health.rs:1233–1245](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/health.rs#L1233-L1245)). Thus the source contains no independent deadline around cleanup itself. Windows JobObject handling is a useful safeguard; this audit did not inspect the fetched `process-wrap` implementation or reproduce a stuck Windows wait, so classify a guaranteed bounded-cleanup claim as unverified rather than claiming ordinary process termination always hangs. The non-Windows fallback contains only a plain child process and cannot provide the Windows descendant containment guarantee.

2. **Output limits cap retained bytes, not total bytes processed.** [proc.rs:35–50](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/proc.rs#L35-L50) keeps draining after the stream limit and reports overflow only after process termination (`262–266`). Memory retention is bounded, which is good. A noisy process can still spend its full allowed time writing/discarding data. If the desired policy is termination on excessive output, communicate overflow to the supervisor and kill immediately. This is a resource-policy improvement, not an unbounded-memory finding.

3. **Static source tests provide weak assurance in several places.** [proc.rs:337–340](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/proc.rs#L337-L340) asserts that the source contains `wrapped.wrap(JobObject)` and that the text `ProcessTree::assign` appears once; the asserted text itself supplies that latter occurrence. This does not establish assign-before-user-code behavior. The no-raw-Command test's hard-coded list (`313–322`) omits newer modules such as health/download/artifact. Real descendant marker tests exist (`407–487`, `491–527`) and are materially stronger. Add a behavioral immediate-child-launch containment test and discover modules automatically.

4. **Metadata-only model identity is not full content identity.** [artifact.rs:343–373](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/artifact.rs#L343-L373) distinguishes logical IDs (name, size, header hash) from content IDs requiring full SHA-256. That separation is sound, but `content_id` includes filenames and companion order, so the same bytes renamed or companions reordered produce a different ID. Treat it as a named artifact-set identity, not a canonical byte-only model identifier. Same-size tensor-data mutation after the header is not detected by a logical ID. Check all measurement/cache consumers use the intended identity level.

5. **Artifact inspection does not verify all GGUF split semantics.** [artifact.rs:78–112](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/artifact.rs#L78-L112), `159–194`, `429–438` compare names/count and generic model header identity; the represented identity does not include a header's shard index/count or tensor-set completeness. The compatibility of per-shard split metadata was not established; MT-15 separately covers the confirmed scanner/inspector completeness mismatch. Full tensor data is intentionally not parsed by the control plane.

6. **Preflight is deliberately conservative and incomplete.** Dense KV estimation only supports `llama`, and only f32/f16/bf16 cache types ([preflight.rs:23–33](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/preflight.rs#L23-L33)). Other architectures, quantized caches, and recurrent/hybrid layouts become Unknown (`35–115`). Weight memory is explicitly a file-size proxy (`465–485`), multi-device allocation is Unknown (`301–353`), and placement/reserves are not measured allocations. These are appropriate honesty boundaries, not bugs to fix by adding guessed support. The estimator accepts zero-valued supplied dimensions and can derive a zero KV allocation if block/head/key/value dimensions are zero; validate impossible dimensions before treating such metadata as an estimate. Overflow uses checked arithmetic, and selected resident files are not incorrectly charged as a second full disk allocation (`548–551`).

7. **Corrupt cached health model recovery is incomplete.** When the pinned health model already exists but fails verification, `ensure_pinned_health_model` returns a trust error ([runtime.rs:3908–3912](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L3908-L3912)) instead of quarantining/re-downloading it or providing a repair operation. Fail-closed behavior is correct, but repeated Check Health cannot recover until the corrupt cache file is removed by another path. Provide an explicit safe repair action and test a corrupt cache. Do not silently execute or retain a corrupt cached model as healthy.

8. **Approval is a deliberate snapshot.** The compiled release pin is `b10816`, commit `427291b5b34cd914a31b3fd3b61a68f6184f4b9f`, with seven active Windows x64 runtime content manifests. ARM64 entries are dormant. There is one compiled compatibility record. Matching exact OS/build/driver/asset identity ([runtime.rs:628–661](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L628-L661)) appropriately prevents a backend's mere availability from becoming product-support evidence. Runtime updates require maintenance of all approval anchors and hardware qualification; do not market successful download/upstream CI alone as a local hardware support guarantee.

### Independent manifest verification performed

A Python read-only validation of the seven checked-in content manifests completed successfully. It checked: SHA-256 of the manifest bytes against the approval asset's `contentManifestSha256`; release tag/commit identity; sorted, unique, relative paths with no traversal, colon, or backslash; one server path per inventory; approved archive name/byte/digest bindings including CUDA companions; and per-file SHA-256 format/size limits.

| Install key | File entries | Declared extracted bytes | Manifest SHA-256 anchor |
|---|---:|---:|---|
| cpu | 51 | 46,741,789 | `37a710415a8aa8d2f0aac6fbe2e64dd495a2d8678839a9d704c9a0315d2a0131` |
| cuda-12.4 | 55 | 1,166,447,389 | `29f31e8391c40d6485d61dbb178d69be14801151104ae9f9b4b213137b0861fd` |
| cuda-13.3 | 55 | 704,229,997 | `bf60d7f375628f2151d44bc3e78c38314a60b3d18f14c995e471e275ca056a71` |
| openvino | 79 | 236,568,282 | `857b45ed7646f270d69144fa71db73096e14fbd6a20b179486a2b79ddbe0460d` |
| rocm | 55 | 1,124,428,061 | `6ab930043b27d13a9b5e36cff80e4bb2ca951f54b5d1a071ab4d62ee8cb69937` |
| sycl | 72 | 372,138,265 | `7a766cbb86916cbd22d4a8c9dfd3f75111d298808150e7d0e87eecf04134e792` |
| vulkan | 52 | 102,314,781 | `580f7142142748e598f1e4d0c7c7f410198b747d90f6844ef152c2e39b520534` |
| **Total** | **419** | **3,752,868,564** | **7/7 internal consistency checks passed** |

These are internal manifest consistency results. They do not verify downloaded archive bytes, actual installed binaries, upstream binary provenance, Windows behavior, GPU compatibility, or successful packaged health runs.

### Controls worth preserving

- Installation accepts a bounded install key and resolves its approved URL/assets in Rust; it does not trust the frontend to supply authoritative executable URLs/digests ([runtime.rs:3011–3065](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L3011-L3065)).
- Full compiled per-file manifests bind payload inventory as well as content, rejecting added/unapproved files (`3430–3561`).
- Windows managed-file verification rejects multiple hard links and alternate streams and opens with restrictive sharing (`3197–3353`).
- ZIP paths use `enclosed_name`, ADS/symlink checks, capability-relative create-new writes, entry/path/file/total limits, and cancellation during extraction (`2787–2909`).
- Runtime replacement preserves a backup and attempts rollback if publication fails (`3694–3747`).
- Managed health pins a model to immutable revision, exact size and SHA-256, binds accelerator selection, tests multiple operations, requires exact deterministic output, uses proxy-free loopback clients, and checks process ownership of the readiness listener ([health.rs:381–442](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/health.rs#L381-L442), `445–489`, `718–1281`).
- Hardware and preflight evidence distinguish observed, heuristic, derived, override, and unknown facts. Unsupported arithmetic is often Unknown rather than fabricated numeric certainty.
- Windows process JobObjects and RAII process/temp cleanup are present; the audit findings concern coverage and edge conditions around these controls rather than their complete absence.

### Pinned source links

All links below refer to audited commit `e530371b056cd8e049c2246dbb151aa407bf359f`.

| Reference | Source |
|---|---|
| RT-01 legacy root selection | [runtime.rs 2218–2226](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L2218-L2226) |
| RT-01 categorical legacy launch rejection | [runtime.rs 3623–3640](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L3623-L3640) |
| RT-02 direct tuning probe | [lib.rs 2587–2589](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2587-L2589) |
| RT-02 unprotected older health command | [lib.rs 1151–1160](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L1151-L1160) |
| RT-02 core probe execution | [core.rs 1241–1278](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs#L1241-L1278) |
| RT-03 blocking health completion | [health.rs 654–691](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/health.rs#L654-L691) |
| RT-03 cancellation check and completion result | [health.rs 1151–1281](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/health.rs#L1151-L1281) |
| RT-04 per-file protected handle ends at digest return | [runtime.rs 3301–3353](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L3301-L3353) |
| RT-04 verified context precedes model download and execution | [lib.rs 2340–2366](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2340-L2366) |
| RT-05 unlimited 403-body read | [runtime.rs 2398–2410](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L2398-L2410) |
| RT-05 size check follows record read | [runtime.rs 3580–3585](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L3580-L3585) |
| RT-05 empty directories escape file count | [runtime.rs 3363–3395](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L3363-L3395) |
| RT-06 enumeration invokes full verification | [runtime.rs 1019–1055](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L1019-L1055) |
| RT-06 serial full-file hashing | [runtime.rs 3547–3561](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L3547-L3561) |
| RT-07 test-only helper and production helper | [runtime.rs 2921–2970](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L2921-L2970) |
| RT-08 cleanup before GetLastError | [runtime.rs 3278–3286](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L3278-L3286) |
| RT-09 GPU matching by name/order | [runtime.rs 1133–1164](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs#L1133-L1164) |
| Approval snapshot | [approved_runtimes.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/approved_runtimes.json) |
| Process cleanup/source-test observations | [proc.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/proc.rs) |
| Artifact identity and shard logic | [artifact.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/artifact.rs) |
| Memory arithmetic and uncertainty boundaries | [preflight.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/preflight.rs) |


## Catalog, downloads, SQLite integrity and GGUF parsing

Audit target: `sato942/localmotive`, commit `e530371b056cd8e049c2246dbb151aa407bf359f`. Read-only source review on 2026-09-10. Read `AGENTS.md` before review. No source changes. Rust toolchain not available; do not call the findings below executed Rust regressions. Source-based SQL reproductions were run in Python's real SQLite engine, extracting the current schema and INSERT statements verbatim from `catalog_db.rs`.

### Executive assessment

The strongest parts of this subsystem are the mandatory curator SHA-256, strict Ed25519 verification, bounded HTTP and resume-state bodies, exact range geometry validation, directory-capability writes, and substantial existing failure-path tests. The chief weaknesses are integration rather than missing basic controls: persisted cooldown blocks catalog browsing after restart; healthy database refresh takes the rebuild path while corrupt databases skip it; local overrides are neither reliably preserved nor downloadable; and the range-ignoring fallback fails at ordinary model-file sizes. The GGUF reader has input byte limits but inadequate allocation/CPU limits for attacker-controlled local files.


### Repository inventory actually measured

Python parsed the checked-in manifest and counted:

| Property | Observed value |
|---|---:|
| Manifest date | 2026-09-10 |
| Manifest size | 630,182 bytes |
| Models | 158 |
| Files | 1,417 |
| Distinct model authors | 19 |
| Files with omitted revision, defaulting to `main` | 1,417 |
| Files larger than 8 MiB | 1,408 |
| Smallest file | 3,141,856 bytes |
| Largest file | 428,521,390,560 bytes |
| Missing architecture | 1 model |
| Missing license | 7 models |
| Missing pipeline tag | 29 models |
| Empty parameter label | All 158 models |
| Split-shard filenames ending `-NNNNN-of-NNNNN.gguf` | 0 |
| Source `#[test]` annotations in download/catalog/catalog_db/gguf | 38 / 34 / 6 / 7 |

These are local artifact facts, not live verification of upstream HF model cards, licenses, current availability, file bytes, or metadata. The maximum file size is approximately 399.09 GiB. Empty metadata is represented as unknown in some UI fields, which is preferable to invented data, but it reduces useful search/filter coverage.

### Findings

### DC-01

**Persisted catalog refresh cooldown makes the catalog empty after app restart.**

**Priority: High.**

**Confidence:** confirmed source control flow; packaged reproduction pending.

**Evidence:** [src-tauri/src/catalog.rs:659-663](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs#L659-L663) defines 1,560 minutes (26 hours). [src-tauri/src/lib.rs:2716-2728](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2716-L2728) reads the persistent stamp and immediately returns `Err` while it is fresh. This occurs before `catalog::fetch_catalog` can load verified cache and before `state.catalog` is assigned at [lib.rs:2759](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2759). [src/App.tsx:463-497](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L463-L497) awaits `fetch_model_catalog` at line 467 before it calls `catalog_local_models` at 471. The catch only publishes a notice. `catalogSnapshot` starts null ([App.tsx:174](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L174)), and catalog entry at `858-860` initiates this flow.

**Reproduction:** successfully refresh once; close app; reopen within 26 hours; open HF Catalog. The persistent stamp causes the first fetch to fail; the existing signed JSON/SQLite data and bundled fallback are never consulted by the frontend load sequence. Repeating Refresh repeats the error. Backend authorization also remains pinned to bundled data because in-memory catalog state has not been populated.

**Impact:** normal returning users cannot browse/download newly fetched catalog entries despite valid local data. This breaks offline-first behavior and makes the refresh cooldown a UI availability limit.

**Remedy:** separate `load catalog` from `refresh catalog`. Loading should always return a verified local/bundled snapshot, populate the authoritative state, and include cooldown metadata; only an explicitly requested network refresh should be denied. The UI should load existing data independently and display the cooldown alongside it.

**Regression:** command/UI integration test with a valid signed cache, populated mirror, and a fresh persisted stamp in a new `AppState`. Expect populated catalog and no HTTP request, while explicit network refresh remains throttled. Exercise missing cache, corrupt cache, bundled fallback, and clock rollback. Existing cooldown arithmetic tests do not exercise this startup flow.

### DC-02

**HTTP 200 fallback rejects every range-ignoring file larger than 8 MiB.**

**Priority: High.**

**Confidence:** confirmed arithmetic/control flow; actual Rust HTTP regression not run.

**Evidence:** [download.rs:710-713](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/download.rs#L710-L713) correctly recognizes an HTTP 200 probe as no range support; `765-768` chooses one worker. However, `fetch_chunk` always caps the requested span at `MAX_REQUEST_BYTES = 8 MiB` (`35`, `1010-1014`). `1030-1036` allows HTTP 200 for the sole whole-file worker at offset zero, but `1069-1074` then demands `Content-Length == request_end - start + 1`, which is 8 MiB rather than total file size. A legitimate full-object HTTP 200 is rejected as an unexpected response length. Without Content-Length, body-overrun checks at `1092-1095` or a subsequent nonzero-range request still prevent completion.

**Impact:** servers/proxies that ignore Range cannot deliver normal multi-gigabyte GGUF files through the promised fallback. In this exact catalog 1,408/1,417 files exceed 8 MiB; this statistic measures potential applicability, not a claim that HF currently ignores ranges. Runtime callers reuse this downloader, so the bug can affect runtime archives as well.

**Remedy:** implement an explicit sequential full-body path for no-range responses, preserving a bounded buffer and read/idle deadline without imposing an 8 MiB whole-response limit. A retry after interruption should restart from zero. Do not simply remove range validation from the ranged path.

**Regression:** modify/extend `a_server_that_ignores_ranges_restarts_a_partial_file_from_zero`, whose current payload is only 16 KiB ([download.rs:1425-1429](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/download.rs#L1425-L1429)), with 8 MiB+1 and a realistic >8 MiB payload; test full Content-Length, chunked transfer, cancellation, interruption/restart, misleading Accept-Ranges, and a correct final digest. The existing 16 KiB test misses the threshold.

### DC-03

**Catalog refresh rebuilds healthy mirrors and skips corruption recovery.**

**Priority: Medium.**

**Confidence:** incorrect branch is confirmed; deletion consequence depends on platform/open-handle behavior.

**Evidence:** [lib.rs:2741-2755](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2741-L2755) opens a connection and, only when `migrate_catalog_db` succeeds, calls `rebuild_catalog_db_from_verified`. This is the inverse of the intended pattern described at `2736-2738` and `2743-2744`. [catalog_db.rs:322-331](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog_db.rs#L322-L331) first removes `catalog-mirror.sqlite`, then recreates/mirrors it; removal errors are ignored (`327`). The still-open original connection remains in scope during this call. On systems that permit unlinking the open database, all user overrides disappear. On Windows, deletion may fail due to sharing semantics and the second connection may happen to preserve rows; this needs packaged Windows observation and must not be stated as proven Windows data loss. Actual migration failures and open errors skip rebuild entirely. All results from this block are discarded ([lib.rs:2740](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2740), `2745`, `2758`).

**Impact:** preservation is accidental/platform-dependent; corrupt/newer databases can remain permanently unrecovered; UI receives network success even when local persistence failed. Rebuild also destroys local user data by design ([catalog_db.rs:319-327](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog_db.rs#L319-L327)) unless separately preserved.

**Remedy:** on a valid connection, use `mirror_verified_catalog(&mut connection, models)` transactionally. On confirmed database corruption/migration failure, close connections before controlled recovery; preserve/export user rows when readable, quarantine the old database, and report irrecoverable user-data loss. Do not silently destroy unknown-newer schemas after a downgrade. Surface the local persistence failure while still allowing browsing from the verified snapshot.

**Regression:** real temporary database containing curated and user entries; ordinary refresh retains user records; corrupt and newer-schema databases follow documented recovery; open-handle Windows test verifies actual behavior; injected database-lock/disk-write failure keeps the previous state and produces a visible notice.

### DC-04

**User-added catalog entries cannot be downloaded and disappear from filtering.**

**Priority: Medium.**

**Confidence:** confirmed source integration.

**Evidence:** the save command only writes and returns SQLite rows ([lib.rs:2794-2802](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2794-L2802)); it does not update the authoritative catalog state. `download_catalog_file` exclusively resolves `(repo, filename, revision)` against `state.catalog` or the bundled catalog ([lib.rs:2955-2966](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2955-L2966)), rejecting any new user file. This is a correct protection against trusting a mutable mirror as signed data, but it contradicts the user-override feature contract in [catalog_db.rs:24-27](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog_db.rs#L24-L27), which says user rows can download using the user's exact SHA. Frontend then drops local rows: it initially uses `catalog_local_models` ([App.tsx:471-483](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L471-L483)), but the filter effect passes only `catalogSnapshot.catalog.models` (`890-895`). Rich facets are built from local rows, so their choices may refer to rows that are immediately removed.

**Impact:** user overrides can exist on disk and be labeled in a transient view yet cannot perform their intended download and do not survive ordinary filtering. The current visible UI does not appear to expose a full add/edit workflow; commands are registered, so distinguish incomplete product functionality from a proven generally accessible UI flow.

**Remedy:** decide and document the trust contract. Keep curated signature authorization separate; maintain a validated, explicitly user-approved override store for override downloads, with exact SHA and explicit provenance. Use a single merged browse source for rows and facets; do not silently treat SQLite entries as curator-signed. Alternatively remove/defer exposed override functionality until supported end to end.

**Regression:** save a unique local model, reload app, filter/sort, download against a local test server with correct and incorrect digest, remove, then verify removal prevents download. Ensure unknown mutable database rows never become signed authority.

### DC-05

**Override UPSERTs can replace curated provenance, steal files, and leave mislabeled rows.**

**Priority: Medium.**

**Confidence:** confirmed in exact SQL executed through SQLite.

**Evidence:** [catalog_db.rs:139-155](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog_db.rs#L139-L155) updates every model field including `repo` and `user_sourced` on an ID collision. File UPSERT globally conflicts on lowercase filename and changes its `model_id` and provenance (`182-192`). `save_user_catalog_override` queries the existing provenance but merely checks presence (`396-404`), then unconditionally calls `insert_model` (`409-411`). It never rejects collisions with curator IDs/files. `mirror_verified_catalog` later clears network rows and reinserts them (`116-124`) but leaves user files attached to colliding model IDs. `read_catalog_db_models` reads file rows without file provenance (`274-288`) and applies model-level `user_sourced` (`313`), hiding the mixed origin.

**Executed SQL observations:**

1. Curated `curated/trusted-model/shared.gguf`, then a user row under a new ID with `shared.gguf`: the global file row moves to the user model; the curated model has no files.
2. Curated ID `curated` plus user override with that same ID: the model's repo and flag become user-controlled while original curator files remain attached.
3. Refresh after case 2: the model becomes `user_sourced=0` again, but the user file remains attached, so the returned model labels a user file curator-sourced.

No SQL injection was found: values use parameters. No bypass of the actual signed download authorization was found; that boundary rejects unknown files. The finding is wrong provenance/browse data and destructive collision behavior, not arbitrary code execution.

**Remedy:** reserve curator IDs; namespace user IDs or reject mixed-origin collisions; enforce a deterministic filename collision policy that requires explicit resolution. Store provenance at the entity level displayed/downloaded and include it in reads. Mirror refresh must not overwrite user rows through a shared key. Enable and test relational constraints where useful.

**Regression:** same ID/different repo, distinct IDs/same case-insensitive filename, refresh, edit, and remove in each combination; verify originals remain intact and every returned file has correct origin. Current preservation tests ([catalog_db.rs:574-595](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog_db.rs#L574-L595)) use disjoint IDs/files and miss these cases.

### DC-06

**Override edits are not replacement operations and lack atomicity/bounds.**

**Priority: Medium.**

**Confidence:** stale-file behavior confirmed in exact SQLite SQL; failure atomicity and missing bounds confirmed source.

**Evidence:** save executes model/file UPSERTs directly on `&Connection`, not inside a transaction ([catalog_db.rs:384-412](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog_db.rs#L384-L412)), and never deletes files omitted by an edit. Existing user row with only `a.gguf`, saved again with only `b.gguf`, returns both files in the executed reproduction. `remove_user_catalog_override` deletes files and then model in separate statements (`421-433`), also without a transaction. Validation caps model count, selected fields, and file count but omits `tags` count/length, model/file date string lengths, file `quant`, and file `revision` validation (`338-378`). The 200-user cap is checked only if an ID does not already exist (`404`), so converting existing curator IDs bypasses the intended cap. Numeric `u64` fields are cast to SQLite `i64` without range validation (`167-168`, `198`), and converted back with `as` (`284`, `304-305`).

**Impact:** editing does not remove obsolete targets; errors/crashes can leave partial mutations; oversized command payloads can inflate persistent data and browse responses; integer values beyond the signed range do not have faithful SQL semantics. This does not demonstrate current valid catalog corruption; shipped file sizes are safely within range.

**Remedy:** transactional replace of one user's complete model/file set, transactional remove, origin-aware cap accounting, shared full validation across parsing and user writes, and checked integer conversion. Bound serialized input bytes as well as row count. Preserve old version on failure.

**Regression:** edit two files to one; failure on second insert; remove failure between statements; concurrent saves near cap; long tags/revision/quant/date strings; duplicate case-insensitive files; values at `i64::MAX` and beyond.

### DC-07

**Some network/cache failures bypass the promised last-good catalog fallback.**

**Priority: Medium.**

**Confidence:** confirmed source error propagation.

**Evidence:** [catalog.rs:876-878](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs#L876-L878) uses `?` for body-read errors, size-limit overflow encountered during streaming, and invalid UTF-8. Signature body reading does the same (`893-897`). After signature validation, parse errors also propagate directly (`927`) instead of calling `fallback`. Advertised Content-Length over-limit and HTTP status/network send errors do call fallback (`839-845`, `865-874`, `899-916`), so behavior depends on where the same bad response is detected. A validly signed future-schema document, including a planned schema upgrade, can make an older client error despite a good supported cache. `catalog_local_models` similarly propagates DB-open failure at [lib.rs:2772](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2772), contrary to its comment promising fallback, though the current frontend catches that particular error.

**Impact:** chunked/stream-interrupted responses or a signed schema change can suppress a usable catalog. No signature bypass occurs; this is a fail-closed availability error.

**Remedy:** route all candidate-network read/parse/signature errors through one fallback function, retaining the supported last-good cache and a refresh-error notice. Ensure future-schema candidates do not overwrite old cache. Database-open failure should also select verified memory/bundle for browsing.

**Regression:** existing valid signed cache plus (a) truncated body after headers, (b) absent Content-Length and limit+1 stream, (c) invalid UTF-8, (d) malformed signature body, (e) validly signed unsupported schema; expect old data and explicit status in every case.

### DC-08

**GGUF byte limit does not sufficiently bound heap growth or parser work.**

**Priority: High.**

**Confidence:** confirmed missing allocation/work bounds; no live OOM or benchmark run.

**Evidence:** [gguf.rs:16-22](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/gguf.rs#L16-L22) caps bytes (256 MiB), array elements (16,777,216), nested depth (4), retained elements per array (4,096), and retained tensors. However `parse` accepts any `kv_count`, allocates initial capacity `min(kv_count,4096)`, then pushes every key/value into `pairs` (`323-331`) without a KV or retained-byte ceiling. Every scalar, even irrelevant string metadata, is decoded and stored; `capture=false` only limits array elements (`239-265`). Every array allocates capacity up to 4,096 even when `capture=false` (`257-261`). Nested captured arrays apply the element cap per array rather than to the whole tree. Converting to metadata facts clones relevant strings/values (`135`, `150-153`, `368-377`). A 256 MiB input-byte ceiling can therefore allow much more than 256 MiB heap, especially many small retained values/keys or nested arrays. A tiny file declaring a nearly-256-MiB string allocates its whole buffer before `read_exact` reports truncation (`188-215`).

`Reader::bytes` heap-allocates for every 1/2/4/8-byte scalar, and `read_summary` passes an unbuffered `File` (`414-418`). Large tokenizer arrays therefore cause huge numbers of tiny file reads/allocations even when their values are not retained. `read_gguf_summary` is a synchronous Tauri command ([lib.rs:1176-1178](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L1176-L1178)); cancellation/timeout/work budgeting is absent. Artifact inspection can repeat the parser across shards ([artifact.rs:414-427](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/artifact.rs#L414-L427)).

**Impact:** opening a crafted GGUF header can cause multi-gigabyte allocation pressure, process termination, or long UI unresponsiveness without reading tensor data. Existing caps prevent unlimited input reads but do not establish an acceptable total resource budget. No remote automatic-exploitation path was demonstrated; opening/scanning attacker-provided local model files is the prerequisite.

**Remedy:** set an explicit maximum KV count, maximum string/key size, aggregate retained-metadata bytes/elements, and aggregate parser work count. Skip irrelevant fixed-size arrays using checked byte counts; stream-discard unneeded strings. Allocate no captured-value vector when capture is false. Use `BufReader`, stack arrays for fixed-width primitives, early file-size/remaining-byte checks, and background execution with cancellation. Preserve useful metadata transparently with truncation indicators.

**Regression:** short file declaring giant string; many tiny KVs; large noncaptured tokenizer arrays; wide nested relevant arrays; oversized key; cap+1 counts; file shortened mid-read. Assert bounded allocation/work or prompt deterministic rejection. Fuzz GGUF parsing with malformed/truncated fixtures and resource budgets. Current 7 tests include a byte-limit arithmetic test (`584-588`) but not end-to-end memory/work enforcement.

### DC-09

**Quantization labels are extracted from arbitrary final filename suffixes.**

**Priority: Medium.**

**Confidence:** confirmed shipped manifest data and builder expression.

**Evidence:** [scripts/build_catalog.mjs:107](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/build_catalog.mjs#L107) takes the last `-([A-Za-z0-9_]+).gguf` suffix as quant. This captures provenance/version text rather than the quantization token. The checked-in catalog includes six `MTP`/`mtp` quants ([catalog/catalog.json:613-614](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/catalog/catalog.json#L613-L614), `637-638`, `685-686`, `15336-15353`), 21 `imatrix`, 9 `imat`, four `it`, four `0731`, one `optimized`, and one `coding` — 46 suffix-label rows in these categories. Examples at [catalog.json:2017-2041](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/catalog/catalog.json#L2017-L2041), `3577`, `7840-7904`. It also emits variants such as `BF16` and `bf16` separately. `catalog::facets` dedupes exact strings ([catalog.rs:567-571](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs#L567-L571)) while filtering compares lowercase (`466-470`), yielding duplicate semantic filter options.

**Impact:** selecting a true quant can omit builds labeled by trailing provenance; users see non-quant metadata in Build/Quant controls. This directly undermines model discovery. Six MTP-suffixed names also escape the provider `-mtp-` exclusion, but whether those files are standalone companions or full models containing MTP requires upstream/model-header review; do not assert they are companions merely from names.

**Remedy:** parse actual known quant tokens in the full basename, preserving variant/provenance separately. Prefer verified upstream structured quant metadata when available, normalize canonical casing, and emit unknown when ambiguous. Extend curation validation to reject non-quant labels; distinguish integrated-MTP main models from companion-only files using verified facts.

**Regression:** `IQ2_S-MTP`, `Q4_K_M-imatrix`, date suffix, instruction-tuning `-it`, lowercase quant, combined base/draft quant, unknown format, and model name containing a quant-looking token.

### DC-10

**Mutable file revisions weaken reproducible catalog availability.**

**Priority: Low.**

**Confidence:** confirmed source and artifact inventory. All 1,417 shipped files omit `revision`, which defaults to `main` ([catalog.rs:58, 68-70](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs#L58)); the builder emits no immutable revision ([scripts/build_catalog.mjs:108-115](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/build_catalog.mjs#L108-L115)).

**Impact:** if upstream replaces a file on `main`, the old signed catalog points to changed or unavailable bytes. Mandatory size/SHA checks prevent accepting the wrong file, but cannot keep the approved version downloadable. The 26-hour refresh policy can delay recovery. A previously signed catalog can also be replayed by a compromised serving path because there is no signed monotonic sequence/expiry policy. That does not authorize unsigned bytes; it can roll back curation.

**Fix:** pin catalog files to immutable upstream commits while retaining exact SHA-256 and size. Define signed sequence/expiry/rollback behavior if freshness under a compromised serving path belongs in the threat model. The distinct frozen-date builder bug is QD-01, which includes an executed reproduction; it is counted there once.

**Regression:** immutable revision serialization and authorization round trip; upstream `main` replacement leaves the pinned object resolvable; unavailable historical object fails clearly; documented old-signed-manifest replay policy is enforced.

### DC-11

**Completion publication can overwrite a file created during transfer; local tampering defense remains incomplete.**

**Priority: Medium.**

**Confidence:** confirmed path-based race window; exploitation/Windows behavior not executed.

**Evidence:** existing target is checked once before transfer ([download.rs:754-763](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/download.rs#L754-L763)). After downloading, `ensure_safe_write_entry(target)` only rejects links/reparse points/multiple hard links; it permits ordinary existing files (`468-499`, `925-929`). Rename replaces the final name, so a regular file created by another app after the initial check can be overwritten. The in-flight registry only covers this process ([lib.rs:2973-2978](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2973-L2978)); no cross-process reservation is visible. SHA is computed by opening `.part` by name (`911-915`), after which that read handle is dropped and publication renames by name (`929`). A same-user process with directory access can replace `.part` between verification and rename, causing unverified bytes to be promoted. Initial link checks happen before a potentially slow probe, while `.part` is later opened with ordinary follow-link semantics inside the directory capability (`745-750`, `806-810`); cap confinement helps prevent escape outside the root but is not equivalent to verifying a stable final file identity.

**Impact:** concurrent app instances or an external process can clobber a new model file or invalidate the integrity claim after hash verification. No network-only path traversal, outside-directory write, or privilege escalation was demonstrated. A local attacker already able to write model files has substantial existing access; prioritize ordinary cross-process collision safety accordingly.

**Remedy:** per-target cross-process lock/reservation, unique private partial name, no-replace publication, keep a stable verified handle through commit where platform APIs allow it, and verify entry identity at publication. Preserve both artifacts on conflicts and show an actionable error. Test NTFS behavior explicitly.

**Regression:** two app instances same filename; unrelated target appears during transfer; `.part` replaced just after verification; root renamed; safe failure without modifying either pre-existing file. Existing capability/link tests are valuable but do not cover final publication collisions.

### DC-12

**Resume checkpoints are durable before data is guaranteed durable, and final hashing is uncancellable.**

**Priority: Low.**

**Confidence:** confirmed sync/cancel placement; power-loss outcome unverified.

**Evidence:** every 400 ms checkpoint is written and `sync_all`ed ([download.rs:397-400](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/download.rs#L397-L400), `869-876`), but partial data is written with `write_all` only (`1100-1103`); no corresponding partial-file `sync_data`/`sync_all` occurs before recording completed ranges or final rename. After process closure, dirty pages normally remain in the OS cache; the problematic case is OS crash/power loss, where checkpoint progress may outrun durable bytes. On resume, final SHA catches corruption and deletes the entire partial file (`915-921`), protecting integrity at the cost of all prior work.

`sha256_reader` has no cancellation checks (`577-589`); both existing-file verification (`566`) and final verification (`915`) can read hundreds of GiB. UI shows Keep & stop during verification ([App.tsx:1263](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L1263), `1286-1287`), but a completed download ignores cancellation ([download.rs:898](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/download.rs#L898)) and hashing proceeds. One-millisecond or idle progress is not emitted through the hash loop. All file writes share one mutex/seek position (`1098-1104`), so network parallelism does not mean parallel writes; this is safe but potential throughput bottleneck needs measurement rather than assumption.

**Remedy:** establish checkpoint durability ordering and explicit power-loss guarantees; checkpoint on sensible intervals/bytes without syncing tiny files constantly. Add cancellable hash progress and preserve verified/partial state correctly when user stops. Consider positional writes if benchmarks demonstrate mutex/seek cost.

**Regression:** controlled write/checkpoint failure and recovery; cancellation during large existing/final hash; process crash versus OS-power-loss test distinguished; performance measurements for 1/4/8 connections on HDD, SATA SSD, NVMe.

### Additional defensive gaps and observations (not separate high-severity claims)

- `parse_catalog` deliberately drops unusable models rather than rejects each invalid record ([catalog.rs:168-178](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs#L168-L178)). This is acceptable only if dropped counts/reasons are visible; currently a signed catalog can silently lose entries. JS validator and Rust validator differ: JS checks stable slug/dates, safe integer and quant presence; Rust has stronger filename/revision rules but does not require rich fields despite the v1 rejection's rationale. Maintain one schema contract and shared fixtures.
- `catalog_file` finds the first matching repo and then searches its files (`204-216`). Duplicate IDs and filenames are rejected, but duplicate repos are not. A signed catalog with two IDs for the same repo would display the second row while its files fail authorization. The current manifest appears builder-generated one-row-per-repo; add explicit repo uniqueness or search all matching rows.
- User query validation bounds query strings and count of models, but it receives full caller-controlled `Vec<CatalogModel>` ([lib.rs:2819-2840](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2819-L2840)) without bounding nested file/tag arrays or model field lengths. A compromised webview can force large deserialization/formatting/cloning before/after count checks. Do not describe the current count-only bounds as a complete IPC memory defense.
- Filename validation prevents slash/backslash/colon/NUL and `.gguf` violations ([catalog.rs:292-304](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs#L292-L304)); Windows special names/control characters and extended-path aliases deserve explicit acceptance fixtures. No confirmed remote path traversal was found.
- HTTP token travels only in a sensitive Authorization header ([download.rs:597-603](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/download.rs#L597-L603)); source never writes it to sidecars. HF token is stored in keyring and UI receives only masked suffix ([catalog.rs:1067-1158](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs#L1067-L1158)). Legacy deletion failures are silently ignored on migration/save, allowing a revoked older token to remain until explicit clear. `validate_hf_token` has no maximum length (`1043-1064`); ordinary Credential Manager errors limit practical storage. These are lower-priority hygiene concerns, not a demonstrated token leak.
- HTTP clients use reqwest defaults for proxy/redirect behavior ([download.rs:605-610](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/download.rs#L605-L610), [catalog.rs:949-954](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs#L949-L954)). Initial model URLs are fixed HF URLs and catalog command URL is fixed GitHub ([lib.rs:2730](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2730)); no arbitrary frontend URL-based SSRF found in this path. Redirect host/scheme and sensitive-header behavior should be exercised with an authenticated redirect regression. Do not assume token leakage from default redirects without demonstrating it.
- `save_cache_record` creates an exclusive random temp file then closes/reopens it by name ([catalog.rs:784-805](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs#L784-L805)), leaving a same-user local race. Cache record body+ETag+signature publication is otherwise atomic and synchronized in-process. Cache and refresh stamp read/write failures are often ignored; refresh can appear successful despite failure to persist last-good bytes (`929-930`). No partial database/catalog state should silently be described as durable success.
- Cache signature verifies before parsing; bundled catalog is trusted as part of the application. No runtime signature check of bundled bytes is required to establish the binary's own trust, but release-time validation of the bundled signed pair remains a required gate.
- ETag comparisons are case-insensitive ([download.rs:971-986](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/download.rs#L971-L986)), though generic ETags are opaque; mandatory SHA prevents wrong final content, but strict equality is safer for resume/conditional-request identity. Weak ETags are not specially handled before constructing If-Match (`1017-1019`). Final checksum is the important defense here.
- 429 transfer retries ignore Retry-After and use only 500/1000 ms backoff ([download.rs:1136-1145](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/download.rs#L1136-L1145)), while the user-facing message recommends waiting a minute (`631-633`). Builder has exponential 429 retries but no explicit request deadline ([build_catalog.mjs:22-34](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/build_catalog.mjs#L22-L34)). These can cause needless failed retries/hung publishing, not proven corruption.
- Catalog hardware-fit filtering is explicitly a smallest-file heuristic ([catalog.rs:523-537](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs#L523-L537)), not an inference-memory guarantee. Quant selection does not prune a model's returned files; size budget uses the model's smallest build irrespective of selected quant (`466-478`). UI should avoid implying the selected larger build fits when only some variant does. Treat the displayed fit as a heuristic until the selected build is evaluated explicitly.
- `AGENTS.md` section 7 says deliberately no database and says schema loader should preserve prior-version compatibility, while current SQLite implementation and schema-1 rejection do the opposite. `catalog/README.md` documents these newer choices. The contributor contract needs updating to match current architecture and intentional breaking schema behavior.

### Strengths worth retaining

1. Mandatory exact SHA-256 from curator/user metadata rather than optional CDN ETag; existing final files must also hash correctly, and mismatched partials are deleted before exposing final name ([download.rs:542-566](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/download.rs#L542-L566), `909-922`).
2. Ed25519 strict verification with embedded key; bounded signature length; signed cache revalidated before use; invalid network signature selects cache/bundle ([catalog.rs:233-269](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs#L233-L269), `822-824`, `919-925`). No evidence of a network-only signed-catalog authorization bypass.
3. Range probe honors Content-Range total rather than one-byte Content-Length ([download.rs:685-709](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/download.rs#L685-L709)). Transfer validates exact span and total, all probed validators, response length, and no overrun (`1038-1095`).
4. Resume metadata is capped at 64 KiB before deserialization, denies unknown top-level fields, binds URL/size/digest/ETag/Last-Modified, and checks contiguous overflow-safe geometry and requested worker count (`155`, `195-232`, `365-374`, `772-788`).
5. Maximum 8 workers, 1 MiB memory buffer, bounded 8 MiB ranged requests, HTTP deadlines, cancellation between reads/retry steps, and explicit status remediation messages.
6. Capability-directory operations and Windows reparse/hard-link checks are stronger than ordinary path joining. Existing tests explicitly cover directory replacement, open-directory identity, link safety, and malicious resume geometry.
7. Signed JSON remains the authority for curated download authorization rather than mutable SQLite. Queries are parameterized; curated refresh uses a transaction at the low-level `mirror_verified_catalog` function.
8. Cache body/ETag/signature stored as one publication unit; random exclusive temp creation and reader/writer lock prevent normal in-process mismatches.
9. GGUF code reads headers only, has magic/version/array-depth/tensor-count/dimension/header-byte checks and truncation indicators; fixed-width `unwrap` conversions are downstream of exact byte counts, not arbitrary unchecked malformed input.
10. Catalog curation validates signatures, file digests, safe targets, uniqueness, model IDs, and records unknown metadata rather than fabricating many missing values. Builder refuses replacement when normal metadata failures are present unless `--allow-empty` is explicitly passed.

### Reproduction evidence and limitations

Runnable evidence is saved in `audit-notes/catalog-sql-repro.py`; verified output is `audit-notes/catalog-sql-repro-output.txt`. Run `python audit-notes/catalog-sql-repro.py /absolute/path/to/localmotive-source`. Executed once successfully with SQLite 3.53.1: all four SQL behavior assertions passed. The same script independently confirmed exactly 46 shipped suffix-label rows and prints every affected filename: `0731=4`, `MTP=4`, `mtp=2`, `coding=1`, `imat=9`, `imatrix=21`, `it=4`, `optimized=1`. Its fixed assertions intentionally fail if a later source/catalog revision no longer reproduces this audited snapshot.

The SQL reproduction extracted the `BEGIN...COMMIT` schema and the exact two `INSERT INTO ... ON CONFLICT ...` strings from `src-tauri/src/catalog_db.rs`, then executed them with `sqlite3.connect(':memory:', isolation_level=None)`. It used synthetic IDs/repos/files solely as test fixtures, not as claims about actual catalog entries. Observed output:

```text
same_id_update_leaves_old_files:
  model custom(owner/a,user=1); files a.gguf(user=1), b.gguf(user=1)
override_filename_collision_steals_curated_file:
  models curated(trusted/model,user=0), custom(other/model,user=1)
  only file shared.gguf owned by custom
override_id_collision_replaces_curated_provenance:
  model curated(other/model,user=1)
  files original.gguf(user=0), override.gguf(user=1)
refresh_relabels_leftover_user_file_under_curator:
  model curated(trusted/model,user=0)
  files original.gguf(user=0), override.gguf(user=1)
```

No Rust compiler/test suite or packaged Windows executable ran within this review. No live HF downloads, gated-token redirect experiments, malicious GGUF OOM execution, NTFS race exploitation, disk-full/power-loss tests, or current upstream model/license verification were performed. Existing source test counts are coverage inventory, not a statement those tests passed in this environment. Repository-wide checks and GitHub evidence appear in the verification and delivery sections.

### Suggested implementation order

1. Restore always-available local catalog loading and correct healthy/corrupt mirror control flow.
2. Fix range-ignoring transfers at the >8 MiB boundary.
3. Establish one explicit local-override trust/provenance contract; fix transactions/collisions; wire browse/filter/download consistently.
4. Put real allocation/work bounds and buffered I/O into GGUF parsing.
5. Correct quant extraction, rolling cutoff, immutable revision pinning, and network-fallback error handling.
6. Add cross-process download collision handling, cancellable hash progress, persistence-error reporting, and measured performance/crash-recovery gates.

Every behavioral fix should follow this repository's required failing-regression-first workflow and then run the full documented suite plus the packaged Windows scenarios relevant to the finding.


## Measurement, tuning, recommendation, calibration and sharing

Audit target: `sato942/localmotive`, commit `e530371b056cd8e049c2246dbb151aa407bf359f`.

This is a read-only source audit of `core.rs`, `measurement.rs`, `calibration.rs`, `evidence.rs`, `recommend.rs`, `sharing.rs`, `tune.rs`, and the relevant `lib.rs` / frontend integration. No repository source was changed and no Rust test suite was run by this reviewer; shared build/test results are reported in the verification ledger. Findings marked **source-confirmed** follow directly from the inspected control/data flow. **Upstream-confirmed** adds inspection of official llama.cpp source via the selected GitHub connector. These labels do not imply reproduction on a packaged Windows binary.

`AGENTS.md` was read. Its model-fit/measurement workflow refers to `research/measuring/README.md` and `research/measuring/SYNTHESIS.md`; `research/measuring` is absent from this checkout, so those research inputs could not be reviewed.

### MT-01

**Default warm-cache benchmark rejects legitimate prompt-cache hits.**

**Priority: High.**

**Locations:** [src-tauri/src/measurement.rs:264-270, 552-568](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs#L264-L270); [src-tauri/src/lib.rs:1733-1743, 1783-1793](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L1733-L1743); [src-tauri/src/evidence.rs:270-283](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/evidence.rs#L270-L283).

**Behavior:** The default workload has 512 prompt tokens, one warmup, five trials, and `CacheMode::Warm`. The request explicitly sends `cache_prompt: true` for this mode. The identical prepared token array is reused in the warmup and every measured request. `parse_completion_timing` stores the server's `timings.prompt_n` as `prompt_tokens`; the caller rejects a response unless that number exactly equals the full requested prompt size.

**Why this is wrong:** llama.cpp reports newly evaluated prompt tokens separately from tokens restored/reused from cache. Official `tools/server/server-common.cpp` maps `prompt_n` to `n_prompt_processed` and `cache_n` to `n_prompt_cached`. The official server README gives a response with `cache_n: 236` and `prompt_n: 1`, and explains that the context total is `prompt_n + cache_n + predicted_n`. Once the warmup populated the cache, a valid completion commonly reports only one or a small suffix count in `prompt_n`, so Localmotive rejects it as the wrong prompt length. This can make all five default measured trials fail despite successful inference.

Official sources inspected through GitHub, including the approved `b10816` tag:

- [llama.cpp b10816/tools/server/server-common.cpp#L67-L76](https://github.com/ggml-org/llama.cpp/blob/b10816/tools/server/server-common.cpp#L67-L76) — `cache_n` is `n_prompt_cached`; `prompt_n` is `n_prompt_processed`.
- [llama.cpp b10816/tools/server/server-context.cpp#L3201-L3204](https://github.com/ggml-org/llama.cpp/blob/b10816/tools/server/server-context.cpp#L3201-L3204) — with `cache_prompt`, reuse the common prefix from the preceding prompt.
- [llama.cpp b10816/tools/server/README.md#L493-L496](https://github.com/ggml-org/llama.cpp/blob/b10816/tools/server/README.md#L493-L496) — only the unseen prompt suffix is evaluated when prompt caching is enabled.
- [llama.cpp b10816/tools/server/README.md#L1406-L1423](https://github.com/ggml-org/llama.cpp/blob/b10816/tools/server/README.md#L1406-L1423) — example with `cache_n=236`, `prompt_n=1`; context total includes both counts.

The initial cross-check used upstream `master`; a follow-up explicitly fetched the approved `b10816` tag and confirmed the same behavior at the pinned links above. This establishes the protocol incompatibility for the selected upstream version, while packaged Windows reproduction remains an unexecuted acceptance test. It is not a claim that every historical/patched runtime has identical fields.

**Impact:** Default V2 benchmark failure; misleading trial errors; lost utility of the newer manifest/calibration/ranking workflow. With no warmup, an initial successful trial can coexist with later false failures, producing a measured summary from a partial success set.

**Fix:** Define whether “warm” means a resident, warmed model with full prefill each request or a reused KV prompt. For full-prefill measurement, disable prompt reuse while retaining the process warmup. For actual prompt-cache measurement, persist requested, processed, and cached token counts separately; validate the appropriate total; treat prefill throughput as throughput for processed tokens. Account for any runtime-specific cached-token field/version contract. Do not merely remove the exact-length check without preserving evidence of the actual workload.

**Regression/acceptance test:** Real/local fake-server protocol sequence: first response `prompt_n=512, cache_n=0`, second response `prompt_n=1, cache_n=511`, both with 256 generated tokens. Assert all attempts are accepted or that the harness deliberately requested cache off. Repeat against the pinned packaged llama-server on Windows with one warmup and five trials, and assert raw counts and cache policy. Existing protocol mocks around [measurement.rs:1091-1316](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs#L1091-L1316) return full prompt counts repeatedly and do not test this real cache transition.

### MT-02

**Share export leaks raw trial errors through the summary.**

**Priority: High.**

**Locations:** [src-tauri/src/measurement.rs:149-155](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs#L149-L155); [src-tauri/src/sharing.rs:379-393, 505-531](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/sharing.rs#L379-L393); [src-tauri/src/sharing.rs:175-297](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/sharing.rs#L175-L297).

**Behavior:** The summary creates `failures` by copying each raw `BenchmarkObservation.error` into `trial N: ...`. The share builder removes the observation-level error field but assigns `summary: summary.cloned()`. Consequently, any mixed success/failure benchmark exports those same raw errors in `summary.failures`. The exported privacy review explicitly says `filesystemPaths`, `credentialsAndArguments`, and `rawErrors` were omitted. The share validator validates numeric summary statistics but never scans or removes the failure strings.

**Concrete trigger:** A manifest with one successful trial and one failed trial whose error mentions a local path produces a valid summary with that path. Passing the correctly recomputed summary satisfies the integrity check and preserves the path in the share file. Cold-launch errors can contain structured failure evidence and a log tail, so this is relevant to ordinary runtime failures, not only hostile frontend inputs.

**Additional leak surface:** `Evidence` includes free-form `source.detail` and `notes` ([evidence.rs:89-97](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/evidence.rs#L89-L97)). Hardware evidence, effective context, and process-memory evidence are cloned into the public schema at [sharing.rs:477-479, 484, 514](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/sharing.rs#L477-L479). Those nested strings are not subjected to public-text validation. A privacy-preserving public format needs an explicit policy for them. The obvious demonstrated defect is the unredacted summary failures; do not claim ordinary generated hardware notes necessarily contain secrets without inspecting the concrete note source.

**Impact:** Users can share local paths, account names, error text, or log-derived sensitive content while the UI claims those categories were removed. The file is local until the user shares it; this is an export privacy breach, not an automatic upload.

**Fix:** Introduce a dedicated public summary with counts, numeric metrics, and bounded error codes/categories instead of raw error strings. Construct public evidence from an allowlist of safe source identifiers and numeric values; redact or omit free text. Generate the privacy-review description from the actual export policy, and validate that the serialized public schema cannot contain raw error/argument fields.

**Regression test:** Extend `privacy_export_excludes_paths_prompts_credentials_and_raw_errors` at [sharing.rs:615-627](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/sharing.rs#L615-L627) with a mixed successful+failed manifest and a computed `Some(summary)`. Put distinct canary strings in the raw failure and nested evidence notes/details. The current test passes `None` for the summary and therefore misses the leak. Assert canaries are absent from the complete serialized export.

### MT-03

**No-op proposals can cause unlimited paid advisor requests.**

**Priority: High.**

**Locations:** [src-tauri/src/tune.rs:458-459, 474-485, 496-503, 504-536](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/tune.rs#L458-L459).

**Behavior:** The tuning loop budget is `trials.len() < max_trials + 1`. A valid proposal that makes no effective profile change returns `applied.is_empty()`. The loop resets `consecutive_rejections` to zero and continues without recording a trial or consuming any independent advisor-attempt budget. Deduplication compares the uncoerced proposal map with previously recorded changes, so it does not catch a nonempty map that normalizes to the baseline.

**Concrete trigger:** If baseline `threads` is `-1`, repeatedly return `{"changes":{"threads":-1},"done":false}`. The baseline's recorded changes are `{}`, the proposal passes validation, and the normalized applied map is empty. The advisor is called again forever. An advisor that only echoes `draftModel` also encounters the ignored-field/no-op path ([tune.rs:233-237](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/tune.rs#L233-L237)). These are plausible model mistakes.

**Impact:** Unlimited cloud API requests/cost despite a visible 1–12 trial budget; permanently active tuning state; a session that may not respond to cancellation because `Bench::measure` is never called again. Three-in-a-row parse failures do not bound successful no-op replies.

**Fix:** Add independent total advisor-call, elapsed-time, and measurement budgets. Count no-op/duplicate proposals as rejections with a bounded terminal outcome. Compare a canonicalized effective configuration (or normalized applied changes), not the raw JSON values. Record the reason so the advisor/user can understand it. Enforce the advertised maximum three changed fields in Rust if it is intended as a hard policy; the system prompt alone currently enforces that limit.

**Regression test:** A deterministic `Advisor` that always proposes an existing value, with a `Bench` that succeeds once, must terminate within a small number of advisor calls. Repeat with numeric-string coercion, `draftModel` echoes, changes that do not affect emitted arguments, and alternating no-op maps. Verify user cancellation terminates before the next cloud request.

### MT-04

**Cancelling AI tuning does not cancel its lifecycle.**

**Priority: High.**

**Locations:** [src-tauri/src/lib.rs:2470-2473, 2485-2491, 2515-2523, 2644-2653](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2470-L2473); [src-tauri/src/core.rs:1770-1784, 1806-1810](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs#L1770-L1784); [src-tauri/src/tune.rs:458-548](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/tune.rs#L458-L548).

**Behavior:** `cancel_tuning` sets an atomic flag. The live benchmark reads it only before a launch and after waiting for health. It uses the non-cancellable health wait with a 600-second timeout, then calls the legacy benchmark, which does a warmup plus repeated blocking HTTP requests without the flag. The tuning loop itself receives no cancellation input. A cancelled candidate is recorded as an ordinary failure, after which the next iteration can still contact the cloud advisor. No-op proposals can continue indefinitely as described in MT-03. `terminate_and_wait()` is called after the benchmark closure but its result is discarded.

**Impact:** Stop may not stop an active launch or generation request for minutes. Additional paid advisor calls can happen after cancellation. Depending on backend cleanup failure, the tuning report can retain a successful result despite a failed termination, and subsequent launches may collide with retained resources.

**Fix:** Make cancellation a first-class terminal outcome for the orchestrator and advisor client. Check before/after each cloud request and before every attempt; propagate it into cancellable process startup and V2 HTTP requests; stop the loop instead of recording cancellation as a candidate failure. Record and surface process-tree cleanup failures. Implement a bounded overall deadline, not merely per-read timeouts.

**Regression tests:** Cancel during preparation, health wait, response wait, between repeats, between advisor calls, and during a no-op sequence. Assert a bounded stop latency, no subsequent advisor calls, no surviving child tree, released port, and a final stopped reason of cancellation. A mock cancellation test alone does not verify the packaged process behavior.

### MT-05

**Benchmarks and quality checks can outlive their server identity.**

**Priority: High.**

**Locations:** [src-tauri/src/lib.rs:1857-1874, 1895-1929, 1783-1793](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L1857-L1874); [lib.rs:2006-2038](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2006-L2038); [lib.rs:1416-1434, 1476-1486, 2546-2564](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L1416-L1434).

**Behavior:** V2 obtains a validated server snapshot, releases the server mutex, and marks a separate `benchmark` slot. Subsequent warm requests use only saved host and port. Neither `stop_server` nor `start_server` checks that benchmark slot; `start_tuning` checks only server/tuning state. A cold benchmark deliberately removes the managed server from `state.server` and launches fresh children privately, so the application state looks idle while the benchmark still owns that workload. Quality checks do not even reserve a quality operation slot: they snapshot host/port/model, release the lock, make two requests, then attach the original identity.

**Concrete races:**

1. Begin a warm benchmark; stop the server and start another model on the same endpoint before the next trial. The next response can be attributed to the original model/runtime/profile. The process-memory sample still targets the old PID.
2. Begin a cold benchmark; after it removes the managed slot, start an ordinary server or tuner. The operations race for ports/GPU memory and distort the supposedly controlled measurement.
3. Start a quality suite and restart a different model on the same port between its two requests. Both results are labeled with the old model identity, while the runtime file is hashed only afterward.
4. `start_tuning` releases the server lock before acquiring the tuning lock, whereas `start_server` checks tuning while holding server. An interleaving can pass both “nothing running” checks before either reservation is established.

These are source-confirmed possible interleavings. Their occurrence under a particular UI timing sequence has not been reproduced on Windows in this review.

**Impact:** Measurements/quality can be misattributed; inference operations can compete despite sequential-harness claims; Stop/Start may act on a different lifecycle than the benchmark; failures consume time and GPU resources.

**Fix:** Use one Rust operation coordinator with an atomic transition/reservation for server startup, benchmarking, cold-run ownership, tuning, and quality. Hold a generation/lease for the lifetime of the operation. Stop should cancel the owner safely; Start must reject or wait until it releases ownership. Verify the live listener still belongs to the expected process/generation for each request or reject the whole run if identity changes. Do not hold the existing server mutex during long I/O as the solution; that would recreate UI blocking problems.

**Regression tests:** Deterministic barrier-based races for benchmark/stop/restart, cold benchmark/start/tune, quality/restart, and start/start_tuning. Assert no second owner is granted and no evidence is emitted for a replaced server. Add packaged tests confirming process and listener ownership throughout cancellation.

### MT-06

**Local TLS/authentication configuration and internal transports disagree.**

**Priority: Medium.**

**Locations:** [src-tauri/src/core.rs:594-595, 620-624, 754-762](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs#L594-L595); [src-tauri/src/lib.rs:1049-1059](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L1049-L1059); [src-tauri/src/measurement.rs:415-449, 639-658](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs#L415-L449); [src-tauri/src/core.rs:1756-1780](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs#L1756-L1780).

**Behavior:** Profiles support an API-key file and SSL private-key/certificate pair, and non-loopback hosts require authentication. The health checker always opens a plain TCP stream and sends cleartext `GET /health`. Both benchmark implementations and quality checks similarly send cleartext HTTP without an `Authorization` header. The helper signatures accept only host/port, so they cannot honor the profile's authentication or certificate configuration.

**Subcase A — TLS:** A server correctly configured to expect TLS cannot answer the plaintext health request, so launch can be reported unhealthy and ultimately killed even though the configured runtime is functioning. This affects normal startup, tuning startup, and cold benchmarking.

**Subcase B — authenticated completion:** A server with a key-protected `/completion` endpoint receives no key from the internal clients. Even if `/health` is publicly available and startup passes, benchmarks and quality calls fail with authorization errors. This includes the non-loopback profiles that the validator requires to use an API-key file.

**Adjacent protocol gaps:** The hand-written V2 parser splits headers from body and assumes the remainder is JSON ([measurement.rs:483-492](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs#L483-L492)); it does not decode chunked transfer encoding. The legacy client has no response-size bound and a per-read timeout rather than an overall deadline ([core.rs:1771-1784](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs#L1771-L1784)). V2 connect/write time is outside the response deadline ([measurement.rs:439-454](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs#L439-L454)) and cancellation polling applies only to response reads. These increase compatibility/robustness risk but are not all separate confirmed failures against the selected runtime.

**Fix:** Centralize a Rust-owned local client built from the validated profile, with TLS, proper HTTP framing, bounded requests/responses, whole-operation timeouts, and cancellation. Access the API-key file only through Rust and keep its contents out of the frontend, events, logs, and manifest command strings. Validate certificates according to an explicit local trust configuration; do not globally disable TLS verification. Until supported, reject unsupported TLS/auth measurement combinations immediately with a precise explanation instead of waiting ten minutes.

**Regression tests:** A local TLS server with a trusted test certificate; an API-key-protected completion server; invalid-certificate rejection; missing/wrong key; bracketed IPv6; a chunked JSON response; a slow writer/reader and cancellation during connection/response wait. Run the accepted TLS/key profile against the packaged target runtime.

### MT-07

**Calibration compatibility does not identify the actual execution shape.**

**Priority: Medium.**

**Locations:** [src-tauri/src/calibration.rs:16-37, 64-100](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/calibration.rs#L16-L37); [src-tauri/src/lib.rs:1576-1641](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L1576-L1641); [src-tauri/src/measurement.rs:700-731](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs#L700-L731).

**Behavior:** The compatibility key includes model digests, runtime executable/help/build/backend, adapter and driver IDs, selected context/batch/GPU settings, workload, and harness version. It omits many settings that materially affect throughput or memory: `threads`, `threadsBatch`, `flashAttention`, `kvOffload`, `cpuMoe`, `cpuFfn`, `fit`/fit margins, observed effective per-slot context, speculative method and draft parameters, device selection, LoRA/overrides, and applicable extra options. The hardware part lacks CPU and system-RAM identity. A CPU-only run can have no adapter identifiers at all.

**Impact:** Distinct configurations can have the same calibration key. A calibration fitted on one thread/offload/speculation setup can be reused under another as “compatible,” giving stale or irrelevant estimated intervals. GPU adapter identity alone cannot distinguish CPU-driven throughput on different machines. Unknown driver values are normalized to the same string `unknown`, which is honest as a display but insufficient evidence of cross-machine compatibility.

**Important qualification:** Replay additionally compares `launch.command_args` at [measurement.rs:717-719](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs#L717-L719), so the missing fields do not by themselves allow a changed command to pass replay. The principal exposed error is calibration identity/reuse, plus any consumers that treat this key alone as a full run/configuration identity.

**Fix:** Derive compatibility from a versioned canonical execution snapshot rather than a hand-maintained subset of profile fields. Include all effective performance/quality-relevant arguments, content hashes for LoRA and other influences, observed effective context, CPU/RAM/platform identity where relevant, and a metric/estimator version. Preserve unknown identity as insufficient compatibility evidence for reuse when it matters. Avoid embedding secrets in the digest input; use safe identity tokens or digests where needed.

**Regression tests:** Property/table-driven identity tests changing one material field at a time. Include CPU-only hardware, same GPU with changed CPU, changed effective context under fit, draft settings with the same companion file, and same LoRA filename with changed content. Existing identity tests cover only fields already present in the struct and cannot catch omitted fields.

### MT-08

**Repeated clicks on one result manufacture independent calibration anchors.**

**Priority: Medium.**

**Locations:** [src/V03EvidencePanel.tsx:397-409, 420-430, 820](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx#L397-L409); [src-tauri/src/calibration.rs:105-109, 158-184, 186-210, 523-539](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/calibration.rs#L105-L109).

**Behavior:** “Add anchor” reads the current benchmark's decode mean, pairs it with the current user-entered estimate, and stamps a fresh `Date.now()` each time. The button remains enabled while any summary exists. An anchor stores only compatibility key, estimate, measured value, and timestamp; it has no source manifest/run ID. The backend requires at least three array entries but does not require three distinct measured runs. Exact-record deduplication does not remove copies with different timestamps.

**Concrete trigger:** Run one benchmark, leave the estimate unchanged, and click Add anchor three times. Build calibration. The backend receives three identical ratios with different observation timestamps, reports `anchor_count=3`, and calculates a residual standard deviation of zero. Applying the model then yields a zero-width estimated interval. The same weakness allows a single run to be reused with different manually entered estimates.

**Impact:** The minimum three-anchor gate offers false assurance; uncertainty is artificially reduced; old measurements can be given fresh timestamps and their apparent validity extended. This occurs through normal UI actions without forged IPC input.

**Fix:** Create calibration anchors in Rust from a persisted, validated benchmark manifest ID plus an explicit estimator identity/value. Derive observation time from the run, not the click. Enforce uniqueness of source run and compatible protocol. Decide whether a partially failed run is eligible; the current UI checks only summary existence, so it can also use a failed/cancelled run that retained some successful samples. Show the actual unique-run count.

**Regression tests:** Three clicks on one benchmark still produce one anchor and cannot build a three-anchor model. Three independent benchmark IDs can. Old manifests retain their original time. Failed or cancelled run eligibility is explicit. Deleting/reimporting the same file cannot create a new independent sample.

**Statistical limitation:** The interval is `estimate × (mean ratio ± 1.96 × population standard deviation of ratios)` ([calibration.rs:186-205, 233-247](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/calibration.rs#L186-L205)). This is a descriptive spread under assumptions, not a calibrated small-sample prediction interval or confidence interval. There is no held-out coverage check, small-sample correction, measurement variance, or extrapolation/domain check. The UI appropriately calls it an “Estimated interval”; retain that modest wording and document the formula until empirical validation supports a stronger claim.

### MT-09

**Quality evidence is not tied to full model/configuration identity.**

**Priority: Medium.**

**Locations:** [src-tauri/src/recommend.rs:63-70](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/recommend.rs#L63-L70); [src-tauri/src/lib.rs:2006-2038](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2006-L2038); [src-tauri/src/artifact.rs:343-361](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/artifact.rs#L343-L361); [src-tauri/src/sharing.rs:407-425](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/sharing.rs#L407-L425); [src/model.ts:420-440](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.ts#L420-L440).

**Behavior:** A quality result records a model logical ID and runtime executable SHA, but no full model-content SHA, launch/configuration key, companion/LoRA identity, workload compatibility, or harness version. The artifact logical ID uses filenames, sizes, and GGUF header hashes; changing tensor bytes without changing the header/size leaves that identity unchanged. Share export checks only model logical ID and runtime executable digest when attaching quality. `candidateFromBenchmark` does not even perform those two comparisons before merging a quality pass rate.

**Impact:** A structural-quality pass from one cache quantization, speculative mode, LoRA, or model-content revision can be represented alongside another configuration's throughput. Those are precisely settings where quality can change while speed improves. Server-replacement races in MT-05 add a separate live-attribution hazard.

**Fix:** Bind quality evidence to full immutable content and effective launch identity captured before the requests, plus a suite/version ID and observation time. Recheck the same process/generation through the suite. Require compatibility when joining quality with benchmark candidates and exports. Store per-candidate quality history instead of only attaching the latest selected UI result.

**Regression tests:** Reject attachment after changing KV precision, speculation, LoRA, companion bytes, or model tensor bytes while preserving header and file size. Reject a quality result from another model/runtime at the frontend-independent Rust join boundary. Test stop/restart between quality cases.

**Coverage limitation:** The entire suite is two tests: output exactly `READY`, and output exactly one JSON property ([recommend.rs:77-86](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/recommend.rs#L77-L86)). These are useful formatting/obedience smoke tests, not a semantic-quality, reasoning, multilingual, hallucination, or quantization-regression benchmark. A pass rate of 1.0 means two structural cases passed. The UI/report should consistently label it that way, especially where used as a weighted recommendation objective.

### MT-10

**Missing metrics invalidate the claimed Pareto frontier.**

**Priority: Medium.**

**Locations:** [src-tauri/src/recommend.rs:252-280, 390-400, 424-429, 436-459](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/recommend.rs#L252-L280).

**Behavior:** Pairwise dominance skips any objective missing on either candidate. Different pairs therefore compare different objective sets. This is not a consistent dominance relation and can be cyclic.

**Exact example derived from the implementation:** Consider three measured candidates with all unspecified values absent:

| Candidate | Decode tok/s (maximize) | Prefill tok/s (maximize) | p95 ms (minimize) | Quality ratio (maximize) |
|---|---:|---:|---:|---:|
| A | 100 | 100 | unknown | 0.5 |
| B | 100 | 90 | 10 | unknown |
| C | 100 | unknown | 20 | 0.9 |

The function reports A dominates B (prefill), B dominates C (latency), and C dominates A (quality); equal decode throughput neither prevents nor creates any of those relations. Consequently every candidate has a dominator and none belongs to the reported Pareto frontier. All three can be `FitClass::Measured` with positive decode throughput. `validate_candidate` accepts missing optional metrics ([recommend.rs:295-326](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/recommend.rs#L295-L326)); the UI defaults leave metric constraints null and require only measured classification ([V03EvidencePanel.tsx:88-95](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx#L88-L95)). Thus this counterexample passes both the candidate validation and default hard constraints. This is an algorithmic counterexample calculated directly from the source; it was not run as a Rust regression test in this review.

**Scoring issue:** A candidate's preference score is divided by only the weights of its available objectives. Candidates with different measurement coverage are scored on different denominators. A weak metric can be omitted, raising that candidate's apparent score; score coverage is not reflected in the ranking. `ranges(extract)` also recomputes the same per-objective ranges for every candidate, adding unnecessary allocation and O(n²) work to an already pairwise algorithm at a public limit of 10,000 candidates.

**Impact:** Incorrect exclusion of plausible options, potentially empty “frontiers,” and ranking that rewards missing evidence. The result can contradict the project's principle that unknown facts remain unknown and cannot become evidence through ranking.

**Fix:** Define a consistent objective set and completeness policy before comparisons. Missing required objectives should make candidates ineligible or incomparable, not simply disappear pair by pair. Report evidence coverage separately and avoid directly comparing normalized scores over different denominators. Precompute objective ranges once. If pairwise output at 10,000 items remains supported, bound response size and benchmark worst-case runtime.

**Regression tests:** The three-way example must not produce cyclic dominance; no candidate with missing required quality may beat a fully measured candidate solely because the denominator changed. Test missing-all metrics, equal values, duplicate IDs, zero weights, constrained unknown fields, and deterministic ties. Preserve the existing explanation of score components and constraint violations.

### MT-11

**AI tuning does not actually benchmark at the requested context workload.**

**Priority: Medium.**

**Locations:** [src-tauri/src/tune.rs:118-128, 391-393, 407-419](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/tune.rs#L118-L128); [src-tauri/src/lib.rs:2497-2518, 2592-2606](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2497-L2518); [src-tauri/src/core.rs:1762-1769, 1797-1811](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs#L1762-L1769).

**Behavior:** Tuning sets the allocated profile context to the target, but the actual benchmark always uses one short fixed sentence prompt and only 64–2,048 generated tokens. It does not fill the target context. It updates effective-context evidence after startup, but never checks that the observed per-slot context meets the target before scoring. `parallel` and `fit` are tunable, even though parallel slots divide context and automatic fitting may alter effective capacity. The winner is simply the highest mean of one to five repetitions of the legacy throughput measurement.

**Impact:** The result can be a valid winner for a short greedy completion on a server allocated a large context, while failing the user's intended long-context workload or per-slot capacity. Changes to speculation and KV precision can alter output quality; no quality gate participates in tuning. A tiny noisy mean improvement can become the winning profile without remeasurement or uncertainty. The full V2 protocol/manifests/raw observations are not used or persisted for each tuning trial.

**Fix:** Make the objective explicit: allocated context capacity, occupied prompt context, concurrent slots, quality requirements, and latency/throughput are separate. Use the V2 harness with a target-length prompt and immutable workload definition, or accurately label the present objective as short-prompt decode throughput at an allocated context. Require the observed effective per-slot capacity to meet the chosen requirement. Remeasure the baseline and finalists, retain raw observations/manifests, and require a material improvement beyond normal variation. Enforce any three-fields-per-proposal limit in code.

**Regression/acceptance tests:** A candidate whose parallel setting reduces effective per-slot context below target cannot win. A fit-reduced context must be rejected or explicitly reported. A long-prompt runtime response must account for the requested prompt correctly. Quality-degrading cache/speculation choices need explicit user policy or a suitable quality gate. Confirm the selected winner with additional independent measurements under the same workload.

### MT-12

**Rich launch failures can destroy the benchmark record they should explain.**

**Priority: Medium.**

**Locations:** [src-tauri/src/lib.rs:1750-1779, 1817-1834](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L1750-L1779); [src-tauri/src/measurement.rs:330-340, 375-383](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs#L330-L340); [src-tauri/src/evidence.rs:796-801](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/evidence.rs#L796-L801).

**Behavior:** Cold attempts preserve the entire error returned by startup/health in an observation. Those errors are structured launch failures and can include a bounded runtime log tail. Manifest validation rejects any error longer than 4,096 bytes. If a realistic runtime failure produces a larger serialized message, `validate_complete()` fails before `persist_manifest()` executes. The user gets a validation error instead of the original run manifest.

**Impact:** The worst failures are the least likely to leave a replayable/raw evidence record. A log-tail feature intended to improve diagnostics conflicts with the evidence schema bound; summaries and already collected successes can also be discarded when the final manifest fails validation.

**Fix:** Store a bounded structured error summary/code in each observation and keep larger log evidence in a separate bounded artifact with a reference/digest. Truncate safely at the acquisition boundary with an explicit truncation marker rather than rejecting the entire completed run at persistence time. Preserve the original failure category.

**Regression test:** A cold-start health failure with a log tail producing >4,096 bytes still writes a valid manifest, with the failed outcome, bounded summary, and a reference to retained diagnostics. Repeat for warmup failure and a later failed trial after successes.

### MT-13

**Validation is fragmented and does not enforce complete-record consistency.**

**Priority: Medium.**

**Locations:** [src-tauri/src/evidence.rs:642-717, 720-766](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/evidence.rs#L642-L717); [src-tauri/src/measurement.rs:44-90, 135-185](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs#L44-L90); [src-tauri/src/sharing.rs:146-172, 175-297](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/sharing.rs#L146-L172).

**Behavior:** Manifest validation limits collection lengths and validates timestamps/basic numerics, but does not validate trial/warmup identifiers, uniqueness/sequence, successful token counts against the workload, the presence of decode throughput for successes, completion of all requested attempts when no terminal outcome exists, or coherence between terminal outcome and the attempts. The summary validator independently applies stronger checks for positive trial/token/decode values but still does not ensure unique trial IDs or workload counts. A “complete” manifest can therefore pass the persistence validator while being unusable by summary generation.

The direct share-bundle validator is looser again: it does not call `workload.validate()`, validate launch shape/effective-context evidence, recursively validate hardware/process-memory evidence, recompute summary counts/statistics from observations, require quality status to match its cases, validate quality runtime/model identities, or check the median in `MetricStats` (the values array omits it). The build-from-manifest path performs some stronger checks, but `persist_share_bundle` accepts a supplied `ShareBundle` and uses only the weaker validator.

**Impact:** Corrupt or edited/imported frontend-supplied records can be exported as apparently valid evidence; consumers cannot rely on “validate_complete” as one shared contract. This is a data-integrity risk rather than proof of cryptographic authenticity: the application does not sign runs, and local user-editable JSON cannot establish that a benchmark truly happened.

**Fix:** Centralize attempt/workload and result consistency validation and reuse it for persistence, replay, share building, and direct export. Require terminal state for partial execution; verify unique sequential identifiers and successful raw metrics/counts under an explicit cache protocol. Recompute derived summary and quality aggregate fields. Introduce separate schema states for draft/incomplete versus finalized evidence if both are needed.

**Regression tests:** Duplicate trial IDs, trial zero, successful zero-token attempts, missing requested trials without terminal failure, mismatched token counts, summary totals/median inconsistent with observations, invalid workload/launch shape, contradictory quality cases/status, and invalid nested evidence should be rejected consistently at every exposed boundary. Fuzz deserialization plus validators and assert no acceptance disagreement for finalized records.

### MT-14

**Calibration model invariants and staleness are not enforced uniformly.**

**Priority: Low.**

**Locations:** [src-tauri/src/calibration.rs:174-184, 214-251, 381-400](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/calibration.rs#L174-L184); [src/model.ts:446-453](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.ts#L446-L453).

**Behavior:** Persisted calibration models require at least three anchors, valid creation/expiry ordering, and a TTL of at most one year. `apply_calibration()` does not call this validator. It accepts a deserialized model with zero anchors or an implausible creation time as long as the key, factor, residual, and current expiry checks pass. It does not require `now_ms >= created_at_ms`. Building a new model can reuse arbitrarily old anchors and grants a new TTL from the current build time; no maximum source age or estimator applicability domain is checked. The frontend expiry predicate is `now > expiresAt`, while Rust rejects at `now >= expiresAt`.

**Impact:** Backend callers can obtain derived estimates from models that could not be saved through the persistence API; old evidence can be refreshed by rebuilding rather than remeasurement; UI and backend disagree at the exact expiry boundary. The stronger normal UI duplication problem is MT-08.

**Fix:** Call one full model validator from build, persist, load, and apply. Derive expiry from a documented evidence freshness policy, preserve anchor provenance/timestamps, reject future-created models, and require compatible estimator/metric identity. Align frontend and backend expiry semantics; ideally expose a backend evaluation result rather than a second source of truth.

**Regression tests:** Invalid anchor count, inverted timestamps, future creation, overlong TTL, exact-expiry time, aged anchors, and key mismatch all have explicit consistent outcomes. Validate lower and upper interval bounds as finite after final arithmetic, not only intermediate value/uncertainty.

### MT-15

**Scanner shard completeness differs from artifact validation.**

**Priority: Low.**

**Locations:** [src-tauri/src/core.rs:205-221, 289-305](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs#L205-L221); [src-tauri/src/artifact.rs:159-193](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/artifact.rs#L159-L193).

**Behavior:** The scanner builds a `HashSet` of successfully parsed shard indices, then considers the model complete when that set covers the expected range. It does not require the number of files to equal the number of unique valid indices, and silently drops parse failures from this analysis. By contrast, `analyze_shard_names` rejects malformed names and records duplicate shard indices.

**Concrete example:** A folder containing an unsplit `foo.gguf` and `foo-00001-of-00001.gguf` groups both under logical key `foo`, with expected count one and index set `{1}`. The scanner can report complete while the stricter artifact inspector sees duplicate index one. Similar ambiguity is possible with malformed shard-looking names that the scanner's fallback groups.

**Impact:** Discovery can advertise a model as complete even though launch validation later rejects it. The stricter downstream validator limits this to misleading state and avoidable failed actions rather than loading an incomplete model silently.

**Fix:** Reuse the artifact module's shard analysis for discovery and report its explicit problems. Require every grouped file to parse consistently and have one unique required index. Keep the existing deterministic first-shard selection.

**Regression tests:** Duplicate index, unsplit/split collision, malformed index/count, inconsistent count, missing first shard, extension case variation, and complete valid singleton/split sets should yield the same completeness decision in discovery and launch validation.

### Strengths observed

- The code distinguishes exact, observed, derived, heuristic, and unknown evidence. Unknown facts require explanations; simple relabeling cannot promote provenance ([evidence.rs:154-218](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/evidence.rs#L154-L218)). This is a strong foundation for an honest hardware/measurement UI.
- V2 keeps warmups separate from measured trials and retains individual errors, cancellations, and timeouts. Summary statistics are computed only from successful samples and failures remain visible. Population standard deviation, median, nearest-rank p50/p95, and finite-value checks are explicit ([measurement.rs:94-185](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs#L94-L185)).
- V2 metric aggregation scales values before calculating means/variance to avoid unnecessary overflow, unlike the older implementation. There are dedicated tests for large finite inputs and for distinguishing median from nearest-rank p50.
- Direct first-token timing and derived TTFT are separate fields. The nonstreaming harness does not claim to measure direct streamed TTFT and explicitly rejects that unsupported mode ([measurement.rs:539-542](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs#L539-L542)). The sequential harness explicitly rejects concurrency greater than one.
- Token preparation uses the selected runtime's tokenizer and sends explicit token arrays; there are tests for exact prompt/generation counts and reuse of prepared tokens. MT-01 is a protocol-semantics integration issue, not absence of a reproducibility effort.
- Cold V2 attempts use a fresh contained runtime, a cancellable health wait, and explicit cleanup outcome. Process peak-working-set evidence is sampled from the process that served the request ([lib.rs:1750-1780, 1791](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L1750-L1780)). This is process lifetime working-set evidence, not a guarantee of per-request incremental memory or GPU VRAM.
- Benchmark persistence writes a new temporary file, syncs it, and renames it into place ([measurement.rs:661-698](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs#L661-L698)). Calibration records have per-record size limits, bounded collection counts, non-reparse file checks, and create-new temporary publication ([calibration.rs:403-503](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/calibration.rs#L403-L503)).
- Full artifact/runtime digests, runtime help digest, workload identity, launch snapshot, and compatibility key are present in the newer manifest. Replay checks both command arguments and compatibility identity. These are useful reproducibility controls, with the omissions noted above.
- AI changes are limited to an explicit whitelist; network, runtime/model identity, secret, and raw argument fields are not tunable. Values are coerced to known profile shapes and the profile validator is rerun before launch ([tune.rs:19-53, 222-302](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/tune.rs#L19-L53)). Speculative methods are checked against advertised runtime methods.
- The tuner measures a real baseline, records failed candidate launches, retains the best observed profile, and bounds consecutive malformed-advisor failures. Its pure `Bench` and `Advisor` boundaries make the missing budget/cancellation tests straightforward to add.
- Launch validation checks port zero, host identity, CORS/auth requirements, batch consistency, tensor splits, selected file paths, and runtime help. Directory scans skip symlinks/reparse points. Raw extra arguments have count/size bounds and restrictions against common privilege-expanding options.
- Ranking exposes hard-constraint violations, pairwise dominators, and normalized score components. It does not directly relabel estimated results as measured. Its missing-data policy needs correction, but the explainability approach is valuable.
- External imports enforce schema, digest format, count/size bounds, metric/unit allowlists, numeric domains, and positive timestamps. Import validation always resets state to Pending; a separate explicitly confirmed review selects a terminal state ([calibration.rs:282-359](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/calibration.rs#L282-L359)). This does not automatically authenticate a remote measurement, and should continue to be described as a user review.
- Share exports are local and require explicit confirmation. The normal builder recomputes a supplied summary and checks model/runtime identity for attached quality. File publication uses create-new semantics to avoid overwriting an existing target. MT-02 identifies a specific missed redaction channel rather than absence of an export design.

### Additional limits and hardening opportunities

These are coverage/design limits, not all separately prioritized defects:

1. **Benchmark scope:** The built-in prompt is repetitive and greedy decoding is fixed. It is useful for a controlled microbenchmark, but does not cover realistic chat templates, multilingual prompts, multimodal inputs, retrieval-heavy contexts, concurrency, or stochastic sampling. Speculative-decoding gains are workload-sensitive. Describe what the measured number represents.
2. **Tail-statistics precision:** The default five trials yield a nearest-rank p95 equal to the observed maximum. Display sample count prominently and avoid treating this as a stable production tail-latency estimate. No confidence interval for throughput or stability threshold is recorded.
3. **Memory interpretation:** Process peak working set is cumulative over the lifetime of a warm server and excludes dedicated GPU memory. It should not be presented as isolated per-trial peak allocation, nor used as a combined CPU+GPU footprint without a separate measurement model.
4. **Timing semantics:** The derived TTFT uses prompt time plus average predicted-per-token time. It intentionally differs from network-observed first-token latency and excludes/approximates important components. Keep the “derived” label. The direct first-token field should have explicit runtime semantics if supported.
5. **Profile value validation:** `build_args` validates several relationships but not all enums/numeric domains, such as all cache/attention/split strings, draft probabilities, thread sentinels, and extreme allocation counts. Runtime argument existence is not value validation. Reject impossible proposals earlier to avoid spending launches/cloud budget; do not invent runtime-specific limits without consulting that runtime.
6. **Companion inference:** Companions are attached by filename role, top-level folder “family,” and quantization proximity, not an explicit target-family compatibility relation. Flat/mixed-family folders can offer inappropriate companions. Present this as a candidate heuristic and use model metadata/catalog provenance for stronger matching. Selecting a new draft method can retain an existing draft path when no matching companion is present ([tune.rs:271-288](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/tune.rs#L271-L288)).
7. **Display command:** `display_command_with_args` only quotes strings containing a space. The process itself uses argument arrays, so this is not shell execution. A copied command containing quotes/metacharacters is not guaranteed to reproduce the exact argv in a user's shell. Label the target shell or offer a structured argv export.
8. **Manifest path redaction:** `manifest_safe_args` lists `--model-draft` while `build_args` emits `-md`; LoRA and some other path-bearing flags are not in its sensitive list ([core.rs:807-808, 903-907, 1071-1081](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs#L807-L808)). Local raw manifests already contain explicit runtime/model paths and are not equivalent to the public export. Clarify that distinction and fix the helper if its privacy promise includes all sensitive argument paths.
9. **Unbounded scan depth/work:** Recursive model discovery has no depth/file-count limit and errors on individual unreadable subdirectories. Large/deep or partially inaccessible model trees can make discovery slow or fail wholesale. Bound/cancel scans and return per-path diagnostics if very large libraries are an intended use case.
10. **Storage lifecycle:** Benchmark manifests accumulate without an explicit retention/index/quota policy. Calibration reads enumerate and parse every JSON record, and one corrupt record can fail the entire load before filtering for the requested compatibility key. Quarantine/report bad records and retain recoverable valid history; document retention controls.
11. **External review semantics:** `Verified` means the user confirmed a review state, not that the app re-ran or cryptographically verified the observation. The import schema contains no origin signature or raw-run provenance. Keep imported evidence distinct from locally measured evidence in future ranking integration.
12. **Schema evolution:** Rust structs mirror frontend types, but invariants differ across layers and version labels still refer to v0.3 in several current modules. Add shared schema/contract fixtures for externally persisted data and explicit migrations before changing raw-token/cache semantics.
13. **Tests of source text:** Some `lib.rs` tests assert that a code substring calls a particular helper rather than exercising process behavior. They can catch accidental bypasses but do not prove correct lifecycle or identity handling. Use them as supplements to behavioral tests, especially for MT-05.
14. **Fuzz/property coverage:** The parsers and validators are good candidates for property testing: malformed JSON, Unicode, nested proposal braces, duplicate identifiers, split sets, finite extremes, numeric coercion, and consistency across summary/persist/export paths. The present local unit tests cover many selected examples but no comprehensive proof of these invariants.

### Suggested acceptance sequence

1. Correct warm-cache token semantics and add a real pinned-runtime default-workload test. This restores trustworthy V2 baseline measurement.
2. Close share-export redaction channels with serialization-wide canary tests.
3. Add hard advisor budgets and first-class cancellation, then move tuning onto the V2 request/evidence pipeline.
4. Introduce a single operation reservation/identity model and exercise concurrent stop/start/tune/benchmark/quality operations.
5. Unify the local HTTP/TLS/auth client and validate the security-supported profiles end to end.
6. Version the full execution/content compatibility identity, bind quality and calibration to persisted run IDs, and deduplicate calibration evidence by source.
7. Define missing-data ranking policy, fix dominance/score comparability, and make finalized manifest/share validation consistent.
8. Run the repository's full gates and the packaged Windows/CDP scenarios. Record actual timings, process cleanup, HTTP protocol behavior, and resulting manifest contents rather than inferring success from unit tests or source-text assertions.

This review does not claim that GPU benchmarks, packaged Windows flows, TLS profiles, or the above regression scenarios were executed during the audit.


## Frontend correctness, user experience and accessibility

Repository revision: `e530371b056cd8e049c2246dbb151aa407bf359f`.

Scope: read-only source review of `src/App.tsx` (2,136 lines), `src/App.css` (509), `src/model.ts` (1,730), `src/model.test.ts` (823), `src/V03EvidencePanel.tsx` (863), `src/main.tsx` (9), plus relevant Rust command implementations to verify frontend assumptions. Read `AGENTS.md` and normative `docs/DESIGN.md`. No source edits. There was no live packaged Windows/WebView2, keyboard, screen-reader, or responsive-browser execution in this review. Findings marked confirmed are supported by reachable source paths; live timing, subjective usability, and pixel rendering remain unverified. Color contrast numbers below were calculated from the declared CSS colors using WCAG relative luminance, not sampled from screenshots.

### Assessment

The frontend has a clear product vocabulary, a coherent visual system, native controls, explicit evidence labels, and several good async cleanup helpers. Its principal risk is inconsistent ownership of model, runtime, profile, and measurement state. The source has 73 state declarations, 13 effects and 30 async functions in App alone, plus 22 separate states in the evidence panel. Routine workflows can display one model/runtime while retaining another launch profile, discard measurement results on navigation, or accept a stale cloud/metadata response. Pure helper tests do not exercise these assembled workflows.

The remediation priority is identity and lifecycle consistency before extracting components merely for file size. Preserve server state and evidence as immutable records keyed by actual model/runtime/workload/run identity; keep editable form drafts separate. Backend launch validation is a valuable safeguard, but it cannot detect that the human meant the different identity currently shown by the UI.

### Findings

### FE-01

**Selected model/runtime and the profile actually launched can diverge.**

**Priority: High.**

**Confirmed code paths:** [src/App.tsx:223–241](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L223-L241), `244–272`, `770–799`, `951–953`, `1086`, `1121–1133`, `1598–1601`, `1674`.

1. `scan()` replaces inventory and unconditionally selects its first model (`230–232`), while leaving `profile` untouched. The synchronization effect only creates a profile if none exists (`951–953`). If B is selected/configured and a rescan returns A first, the selected model becomes A but `profile.model` remains B. The dashboard prints the profile name/settings beside `selected.firstShard`; the start button's completeness guard reads the selected model rather than the profile's model. `saveProfile()` persists the old profile under the newly selected model's key (`796–799`). A scan failure similarly clears models without clearing the profile.
2. Editing the Runtime screen's `Existing llama-server.exe` field updates only `runtimePath` (`1598`); pressing Inspect updates capabilities, identity, and saved runtime (`244–272`) but never updates `profile.runtime`. The app can show inspected runtime B while launching the existing profile with A. The alternative file-picker/managed activation path does synchronize the profile (`274–279`), so two visually similar selection methods behave differently.

**Impact:** misleading model/runtime identity, launch of an unintended local target/build, incorrect save ownership, and tuning requests combining the profile for one model with companions/display identity for another. Rust validates the submitted profile; that does not repair human-intent mismatch.

**Reproduce:** select the second inventory model, modify/save it, rescan, return to Control/Profile, inspect the selected path vs the exact profile's `model`; separately set A active, type B into Runtime and Inspect, then compare `runtimePath`, `runtime.path`, and `profile.runtime`.

**Remedy:** represent selection and its editable profile together; retain a still-present selection during rescan, otherwise load/normalize the new selection's saved profile or clear it. Route all committed runtime changes through one activation function and track a draft path separately. Bind readiness and displayed model identity to the actual pending launch object. Persist using the profile's validated model identity.

**Regression:** component/integration tests for rescan after selecting B, deletion of B, failed scan, typed runtime selection vs picker, and saving after each transition. Assert exact IPC profile arguments, not just labels.

### FE-02

**Tuning results are not bound to the model/runtime that produced them.**

**Priority: High.**

**Confirmed paths:** [src/App.tsx:720–749](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L720-L749), `761–767`, `955–962`, `1032–1041`, `1172`, `1907–1909`, `1951–1962`, `1992`.

Starting tuning snapshots the current request in its closure but allows navigation and selection of another inventory model. Changing selection loads that model's previous report and clears live trials; progress events and eventual completion from the old run still overwrite global `tuneProgress`, `tuneLive`, and `tuneReport`. The report is correctly persisted under the original closure's selected ID, but `adoptTunedProfile()` uses the **current** selected ID/name and `tuneContext`. It can save model A's best profile as model B's profile. `adoptTunedProfile` also bypasses `normalizeProfile`, reintroducing the report's old runtime after a user has activated a new runtime. Provider tabs remain usable during the run while the actual advisor request remains the captured provider.

**Impact:** adoption into the wrong model, incorrect report provenance, unexpected runtime rollback, and contradictory run/provider labels. This is separate from merely showing a stale loading indicator.

**Remedy:** use a run record with immutable selected model ID, runtime identity, provider, advisor model, and target context. Index reports by that identity. Adopt against the originating model and explicitly reconcile the runtime using the existing normalization policy. Navigation should preserve the running record and its provenance; either prevent incompatible editing while running or clearly show a separate draft selection.

**Regression:** start A, select B before completion, complete A, then assert A's report cannot overwrite B's display or saved profile; activate runtime B before adopting a report measured on A; delayed progress from a completed run must not contaminate a new run.

### FE-03

**Async cloud, GGUF metadata, port suggestions and command previews accept stale responses.**

**Priority: Medium.**

**Confirmed paths:** [src/App.tsx:598–638](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L598-L638), `640–694`, `706–717`, `782–789`, `802–809`. The app already has the appropriate helper at [src/model.ts:1511–1513](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.ts#L1511-L1513) and applies it to runtime/catalog reads, but not these paths.

* `loadCloud(nextProvider)` sets global credential/model state after async requests without checking the currently selected provider. Select A then B, resolve B first and A last: B's tab can show A's key suffix, configured state, model list and chosen model. `canTune` (`1010`) trusts the unscoped credential and nonempty model. `probeCloud`, OAuth completion, save and forget completion have the same cross-provider stale-display possibility. This does **not** prove a raw credential leak: the actual Rust request still names a provider and frontend status contains masked suffixes. It is a correctness/provenance defect.
* `loadGguf` does not clear stale metadata on start or check model identity on completion. Late A metadata can replace B's architecture/context and clamp B's tuning-context controls. If selection becomes absent, it returns without clearing GGUF state.
* `suggestPortCandidate(A)` functionally updates whatever `current` profile exists when its response arrives; selecting B or manually editing the port meanwhile does not prevent an old suggestion from overwriting B/new input.
* Concurrent `preview()` requests set the exact command in completion order, without verifying the form version. A late command for old settings can appear next to current inputs.

**Remedy:** per-resource sequence IDs or request keys and a central helper that checks identity before committing results; clear provider-specific derived state at switch start. Port suggestions should apply only if profile identity and requested port remain unchanged. Do not reuse one global loading/error status for unrelated requests.

**Regression:** deferred IPC promises completed deliberately out of order for all four workflows. Existing helper-only tests at [src/model.test.ts:520–533](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.test.ts#L520-L533) do not establish that these callers use the helper.

### FE-04

**Typing a profile repeatedly runs expensive synchronous native launch validation.**

**Priority: High.**

**Confirmed chain:** [src/App.tsx:802–809](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L802-L809), `964–966` → [src-tauri/src/lib.rs:1406–1407](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L1406-L1407) → `411–416` → [src-tauri/src/core.rs:1272–1275](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs#L1272-L1275); production probe timeout at [core.rs:1238](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs#L1238) is 10 seconds per probe.

Every change to the full profile object triggers `preview_command` with no debounce/coalescing. That command calls `prepare_launch`, which validates all referenced paths, verifies managed-runtime trust, inspects artifacts and invokes the selected runtime for `--version` and `--help`. It is a synchronous Tauri command. Changing even a human-facing profile name or one digit can repeat filesystem work and two child-process probes. The code executes real native validation rather than only formatting text.

**Impact:** avoidable native process churn and filesystem work, potential visible input/UI latency under slow probes, queued obsolete previews, and stale exact-command text (FE-03). The precise packaged-app latency was not measured. A 10-second bound does not mean every keystroke takes 20 seconds; it establishes the worst probe timeout path.

**Remedy:** debounce/coalesce preview requests, discard stale completions, and split cheap command composition from authoritative launch validation. Cache runtime capability/artifact evidence using safe identity/freshness criteria; keep authoritative trust/path checks on actual launch. Move expensive probing to a cancellable background task so the UI remains responsive.

**Regression:** rapid edits produce bounded probe count and only the latest command; simulate slow native inspection and confirm navigation/input/cancel stays usable in the packaged binary. Preserve security checks at launch rather than removing them to optimize preview.

### FE-05

**Evidence and active measurement state disappear when leaving Benchmark.**

**Priority: High.**

**Confirmed paths:** [src/App.tsx:2086–2129](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L2086-L2129); [src/V03EvidencePanel.tsx:144–165](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx#L144-L165), `190–202`, `317–335`, `345–354`, `372–395`.

The entire `V03EvidencePanel` is mounted only inside `view === "benchmark"`. Its benchmark, history, quality, preflight, rankings, anchors, imported evidence, export confirmation and busy state are all component-local. Leaving to Control/Profile/Inventory unmounts it and returning creates an empty panel. A benchmark already dispatched to Rust continues after unmount, but the remounted panel has `busy=null`, no result handle, and a disabled Cancel button. Results may be persisted by Rust, but the UI provides no manifest-history loading path (only replay of the current in-memory benchmark). The ordinary compare-settings workflow requires leaving Benchmark to edit/start a different profile, so prior candidates are lost before comparison. Even in a persistent mount, `190–202` clears all history on fingerprint changes.

**Impact:** loss of accessible measurement evidence and cancellation control; session Pareto comparison cannot naturally span profile configurations; saved manifest data is harder to recover than the interface implies.

**Remedy:** move evidence/run state to App or a dedicated persistent store/service; load saved manifests by identity; render view components from that durable state. Keep an application-wide running operation/cancel affordance. Separate invalidating the **current preview** from deleting historical evidence.

**Regression:** start measurement, navigate away/back, cancel; complete while away and verify the result reappears; benchmark A, change profile/start B, benchmark B, compare both with their immutable identities.

### FE-06

**Preflight evidence stays visible after its input assumptions change; final adapter cannot be deselected.**

**Priority: Medium.**

**Confirmed paths:** [src/V03EvidencePanel.tsx:167–208](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx#L167-L208), `252–259`, `280–314`, `643–685`.

Changing adapter checkboxes, manual capacity/note, or refreshing hardware does not clear/mark stale existing `preflight` evidence. The form can visibly show a new selection/override next to an old required/available budget and allocation plan. The fingerprint used for invalidation covers only 12 profile fields and omits several consequential fields (for example draft model, projector, speculation mode, KV offload, device, fit settings, and flash attention). In normal use editing those fields unmounts the panel anyway, but the incomplete fingerprint remains a latent problem when lifecycle is fixed. Async preflight/inspection responses have no input-revision guard either.

The effect at `204–208` automatically selects the first hardware adapter whenever the selection is empty. Unchecking the final checkbox immediately reselects the first adapter. This conflicts with the explicit-selection copy and prevents representing an intentional no-GPU selection in the form. The selection also defaults independently from Runtime's explicit selected adapter.

**Remedy:** key preflight by all inputs it actually used, retain those inputs with the result, and label an old result stale until rerun. Initialize adapter defaults once, distinguish uninitialized from deliberate empty selection, and align explicit selection across Runtime and evidence views.

**Regression:** run preflight then change adapter/manual budget/hardware, assert stale state; uncheck final adapter and verify it stays empty; resolve an old preflight after changing selection and do not replace current evidence.

### FE-07

**Cancellation reuses the operation busy state and clears it before the benchmark has ended.**

**Priority: Medium.**

**Confirmed paths:** [src/V03EvidencePanel.tsx:239–249](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx#L239-L249), `317–342`, `707–718`, `820`; [src-tauri/src/lib.rs:1945–1952](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L1945-L1952).

`cancelBenchmark()` wraps the cancel request in `runAction("cancel")`; this overwrites `busy="benchmark"` and then sets busy to null as soon as the cancellation request returns. The original benchmark still owns work and will finish later. New runs/quality/ranking actions are enabled early; the Cancel affordance disappears immediately. Rust's active-benchmark check rejects another v2 benchmark, which limits overlap, but that is an error rather than coherent UI state. `invoke<void>` corresponds to Rust `Result<(), String>`; success serializes as null, the same sentinel `runAction` uses for failure, so `cancelled !== null` does not reliably produce the intended cancellation message. `Add anchor` also lacks the `busy !== null` guard other evidence actions use.

**Remedy:** independent `runState` and `cancellationRequested` flags; keep the run active until its original promise settles. Return a tagged success/error result from action wrappers rather than using null where a successful command may return null. Coordinate all native operations through their real lifecycle.

**Regression:** cancel request returns before measurement termination; assert running/cancelling state stays active and no new work becomes enabled prematurely. Test successful void responses as well as rejected cancellation.

### FE-08

**Raw extra-argument field cannot normally accept multiple tokens by typing.**

**Priority: Medium.**

**Confirmed path:** [src/App.tsx:1857–1860](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L1857-L1860).

The controlled input renders `profile.extraArgs.join(" ")` and on every character trims/splits the input into tokens. Typing a space after `--flag` immediately removes that trailing space when React renders the joined array. Typing the next token then concatenates it onto the first. Pasting a complete token list can work, but ordinary typing of separate arguments does not. Quoted/space-containing values are not representable either; the help intentionally calls for self-contained tokens, so the primary confirmed defect is the inability to enter the separator.

**Remedy:** keep an editable raw string while focused and parse on blur/explicit validation, preserving whitespace and surfacing precise syntax errors. Retain backend privilege/override rejection.

**Regression:** real input events typing `--flag-one`, space, `--flag-two=value`; assert the visible string retains the separator and IPC receives two tokens. Include paste, delete, whitespace-only, and disallowed overrides.

### FE-09

**Corrupt persisted JSON can crash the UI; write failures can be reported as operation failures.**

**Priority: Medium.**

**Confirmed paths:** [src/App.tsx:100–109](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L100-L109), `741`, `765`, `770–775`, `798`, `849`, `955–962`; [src/model.ts:1452–1472](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.ts#L1452-L1472); [src/main.tsx:5–9](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/main.tsx#L5-L9).

Storage reads catch access errors but `JSON.parse` for profiles and tuning reports is unguarded. A malformed tuning record throws from an effect after inventory selection, with no React error boundary anywhere in `src`. Valid JSON with wrong shapes is also trusted (`as TuningReport`; `normalizeProfile` spread cast). For example a string `extraArgs` reaches `.filter`, or a non-array `trials` later reaches `.reduce` during render. There is no schema version, quarantine, per-record reset, or safe fallback in this path.

Most writes are also unguarded. Some occur after successful expensive native actions inside their try blocks; a quota/security exception from saving the tuning/benchmark report then changes a successfully completed operation into an apparent error. Other setting/profile writes throw directly in event handlers. Full reports accumulate by model ID with no retention/export management.

**Impact:** recoverable local-state corruption or browser storage problems can blank the interface or hide valid results. This is local resilience, not evidence of a remote exploit.

**Remedy:** versioned safe parse/validate/migrate functions, isolate corrupt records and restore defaults with recovery copy; distinguish completed native work from failed persistence; bounded retention and export/import of user profiles. Add a root error boundary with restart/reset diagnostics.

**Regression:** malformed JSON, JSON null, wrong field types, older records, unsupported flashAttention values, storage quota failure after successful native run, and blocked storage access.

### FE-11

**Download cancellation targets the currently edited destination, not the destination of the running job.**

**Priority: Medium.**

**Confirmed paths:** [src/App.tsx:539–584](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L539-L584), `1214–1218`, `1257–1265`, `1283–1287`; [src/model.ts:1651–1664](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.ts#L1651-L1664).

A job starts using the then-current `modelRoot`. That field and folder picker remain editable while downloading. Cancel subsequently passes the **current** `modelRoot` plus filename to `cancel_download`, so after choosing a new destination it targets a different path. The boolean return is ignored and the UI always says it is stopping. The Promise has no catch in this handler. Frontend progress keys contain only repository+filename, excluding destination and revision. UI state therefore cannot distinguish the same file's jobs/evidence at different destinations/revisions. Changing a card's selected build also hides the former file's active Cancel control because the displayed key changes. Already-on-disk inference uses filename suffix matching across the existing inventory and a previous done event, regardless of chosen destination; backend verification remains the ultimate defense.

**Remedy:** capture immutable job IDs and source/revision/destination in job state; cancel by that ID; show all active transfers in a dedicated list independent of filters/build choices. Respect a false cancellation response and catch errors. Reconcile completion with an explicit inventory refresh or a clear action.

**Regression:** start in folder A, change to B, cancel; switch build/filter during transfer; same repo/file at different destinations; failed/false cancellation; completion after inventory selection changes.

### FE-12

**Model-folder and download-destination fields lose the keyboard-focus ring.**

**Priority: Medium.**

**Confirmed CSS:** [src/App.css:31](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.css#L31), `102`; affected controls at [src/App.tsx:1163](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L1163), `1216`.

Global `:focus-visible` has specificity (0,1,0). The later `.path-bar input` selector has greater specificity (0,1,1) and sets `outline:0`. It overrides the global outline even during keyboard focus. These model-folder and download-destination inputs also have `border:0` and no substitute focus style, leaving no visible focus indicator declared for them. This conflicts with `docs/DESIGN.md`'s instruction to retain the amber focus-visible ring. The generic `label input`/`label select` outline resets have lower specificity (0,0,2), so they do not establish the same defect in ordinary labeled fields; those fields retain the global focus-visible outline.

**Remedy:** remove the path-bar outline reset or add `.path-bar input:focus-visible` with the amber outline. Keep the existing global rule for other controls.

**Regression:** tab to the model-folder and download-destination inputs and assert a nonzero visible outline in computed styles; use an ordinary labeled input as a control to ensure its existing outline remains. Confirm the two affected fields in packaged Windows focus traversal. Static specificity is confirmed; final WebView2 rendering has not been observed here.

### FE-13

**Several controls expose incomplete names/structures to assistive technology.**

**Priority: Medium.**

**Confirmed markup:** [src/App.tsx:1032–1041](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L1032-L1041), `1167–1187`, `1803–1804`, `1907–1912`; [src/V03EvidencePanel.tsx:544](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx#L544), `703`, `782`.

* Inventory uses `role="table"` and one `role="row"` header, but header spans have no columnheader roles and body entries are buttons without row/cell semantics. It advertises a table without its expected accessible hierarchy/column associations.
* Paired N-gram input groups each wrap **two inputs in one label**. A label associates with one labelable control; the second fields do not have their own name identifying maximum vs minimum/map draft vs lookup size.
* Provider tabs have tab/tablist roles and aria-selected but lack roving tabIndex, Arrow-key/Home/End handling, matching tabpanel, and aria-controls relationships. The WAI tab pattern explicitly provides those navigation/association conventions. [W3C Tabs Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/tabs/).
* Primary navigation indicates active state only with a class, no `aria-current`/equivalent semantic. Content replacement does not manage heading focus or provide a skip-to-main control. Keyboard activation can leave users traversing all remaining navigation before reaching changed content.
* Evidence headings jump from h2 to h4. This is an organizational issue rather than a demonstrated blocker.

**Remedy:** native table with a clearly named action in each row, or an explicitly chosen list pattern; one label per input; complete the accessible tab pattern or use ordinary buttons in a labeled group. Add navigation current-state semantics and deliberate focus behavior.

**Regression:** component accessible-name/role queries, axe, and keyboard/screen-reader checks with Windows Narrator or NVDA. Native file dialogs are used; no custom modal focus trap exists to audit in the current UI, and native dialog behavior was not run.

### FE-14

**Small text colors fail 4.5:1 contrast; compact layouts leave evidence controls cramped.**

**Priority: Medium.**

**Contrast evidence:** CSS `134` hardware source `#6f7974` on hardware panel `#202622`: **3.424:1**; `167` runtime code `#77817d` on `#222627`: **3.798:1**; `232` field help `#7f8885` on `#222627`: **4.192:1**; `94/96` empty-log explanatory text `#737b78` on `#111514`: **4.235:1**. These are non-large text (typically 8–10px). The default muted token on the panel is **5.937:1**, a safer existing choice. Calculations used CSS-defined foreground/background colors and standard sRGB linearization. W3C's minimum contrast criterion specifies 4.5:1 for ordinary text, with separate exceptions for large text, inactive controls and decoration. [W3C Contrast Minimum](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html).

**Responsive source evidence:** [src/App.css:401–424](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.css#L401-L424), `438–504`. Evidence controls keep `repeat(3,minmax(0,1fr))` at every viewport; action headings stay flex rows; evidence grids require 205px columns and adapter grids 240px. At a 320px root, screen/panel/section padding leaves about 222px for inner content, so a 240px adapter grid cannot fit absent another rule. Eight bottom-nav cells share 302px of usable width at 320px (~37.75px each), less than the repository's ≥44px mobile-target rule. Path actions have 42px minimum height; several plain links/managed entries stay below 44px. This does not establish a blanket WCAG target-size failure (the WCAG 2.2 AA minimum differs); it establishes noncompliance with the project's own 44px requirement and a concrete reflow risk. Large-screen Windows is the primary target; narrow/zoomed support still needs verification.

**Remedy:** use compliant muted text tokens, raise base/help sizes where practical, add evidence-specific one-column breakpoints, constrain grids with `min(100%, ...)`, wrap action headings, and redesign narrow bottom navigation to retain useful targets. Test 320/375/680/980 widths and 200%/400% zoom in the packaged environment.

### FE-15

**Runtime capability and status labels overstate what was established.**

**Priority: Medium.**

**Confirmed paths:** [src/App.tsx:1086](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L1086), `1121–1123`, `1341–1343`, `1674`, `1711–1713`, `1871–1875`; [src/App.css:409](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.css#L409).

The `VALID` loaded-profile tag is derived solely from `selected.complete`, not launch validation. First-run readiness marks Runtime configured and Serve ready from nonempty path strings. Speculation choices show a hard-coded fallback list when no runtime has been inspected, while adjacent copy says only this executable's advertised methods are offered. Editing a runtime path leaves old runtime capabilities/identity visible until inspection completes. Most controls are exposed without flag-specific capability labels; Rust may reject unsupported settings at preview/start, so UI availability is not support evidence. The evidence panel styles **every** strong result green, including `Unknown`, `Blocked`, rejected ranking labels and non-measured result classes, conflicting with the normative green=running/valid rule. Textual labels remain, so this is misleading emphasis rather than color-only encoding.

**Remedy:** use `shards complete`, `path selected`, `not inspected`, `validation pending`, and `validated` precisely; clear stale capability display on committed runtime identity changes. Offer known methods only after inspection or mark fallback values explicitly provisional. Map evidence tone to its actual status; unknown/incomplete states should use neutral/amber.

**Regression:** nonexistent runtime path, incomplete inspection, complete shards with invalid profile, unsupported flags and unknown/blocked evidence all render accurate words and tones.

### FE-16

**Legacy controls and new evidence tools have inconsistent active-operation/result ownership.**

**Priority: Medium.**

**Confirmed paths:** [src/App.tsx:159](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L159), `811–855`, `984–995`, `1081–1086`, `1105–1112`, `1144`, `1661`, `2093`; [src/V03EvidencePanel.tsx:164](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx#L164), `317–395`.

The App's single `busy` string represents scan, inspect, start, stop, cloud and legacy benchmark work, while the evidence panel maintains another busy string and tuning/install/health have still more flags. Buttons usually check only one operation name, not the actual conflicting activity. Completion of one task clears the shared string even if another is still active. Profile Start remains enabled while a server is running, depending on the backend to reject it. Legacy and v2 benchmark controls are on the same screen but do not share their busy state. Global status polling uses an async `setInterval` without in-flight serialization or request ownership: if a call takes longer than two seconds it queues more calls and later stale status can replace the result of an explicit action. It continues every two seconds in browser preview despite expected errors, and polls/re-renders App across every view.

The dashboard strategy uses the current editable profile, not the running server snapshot; its legacy LAST TEST is not keyed or cleared by actual run identity. Stored legacy results are written but never read. The server log panel disappears when `status.running` becomes false (`1144`), hiding the last output at exactly the time an exited process needs diagnosis. Rust returns a bounded log tail, which is good; this review does **not** claim unbounded log rendering.

**Remedy:** explicit operation records/compatibility rules and a server snapshot separate from drafts; single-flight polling or events; preserve last logs and failure state after exit; label legacy result with model/runtime/workload/time; consolidate legacy/v2 measurement UX or explain their distinct scope.

**Regression:** overlapping scan/cloud/start completions, legacy vs v2 benchmark overlap, slow poll resolving after stop/start, unexpected process exit, and draft profile editing while an existing server runs. IPC-01 covers native startup blocking; MT-05 covers backend operation ownership. This item covers the distinct UI status/result presentation.

### FE-17

**Tested workload validators are disconnected from production UI; domain defaults/rules are duplicated.**

**Priority: Low.**

**Confirmed paths:** [src/model.ts:475–514](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.ts#L475-L514), `1590–1629`; [src/model.test.ts:263–282](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.test.ts#L263-L282); [src/V03EvidencePanel.tsx:75–86](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx#L75-L86), `126–129`, `317–325`, `723–728`.

`defaultWorkload()` and `validateWorkload()` are imported by tests only; the evidence panel has a separate literal default and dispatches directly. The test named “rejects invalid workload limits before IPC” proves the pure function, not that actual UI rejects anything before IPC. Number inputs define some minima but no complete maxima and there is no form submission validity gate. `Number` accepts fractions; typed Rust integer deserialization rejects them. Rust remains authoritative and rejects invalid work, so this is UX/test-assurance debt rather than validation bypass.

TS mirrors Rust rules for fit budget/filtering and handwritten IPC types/default profiles; regression fixtures help, but there is no generated schema/contract harness ensuring parity. `suggestedProfile` also decides companion choice and substantial launch defaults in the frontend despite the architecture's “Rust owns truth” principle. Prioritize shared contracts on correctness-sensitive fields rather than adding abstractions indiscriminately.

**Remedy:** import the tested workload factory, validate form drafts with a production-used helper or backend validation endpoint and display field-level errors. Generate/verify Rust↔TS contracts or add representative round-trip fixtures. Use integer validation where Rust requires integer types.

**Regression:** default parity, fractional/negative/out-of-range values, blank numeric edits, and asserting no native run is dispatched while invalid; then assert the backend still rejects malformed direct calls.

### Additional product and maintenance observations

* **Cloud data disclosure:** README `224–243` explains that tuning sends hardware/model/runtime/companion paths and measurements. The tuning UI names a cloud advisor and costs but never shows a concise data-sent list before the action; its “paths ... never touched” sentence ([App.tsx:1972](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L1972)) describes mutability, not disclosure. Add that README fact inline near the action and preserve the distinction between local privacy-reviewed exports and cloud tuning. Do not claim a secret leak from this frontend; credential inputs are password fields and only masked statuses are displayed. Cloud data flow and export privacy are assessed separately in CLD-01 and MT-02.
* **Error handling consistency:** several filesystem dialogs, `openUrl` calls and cancellation promises lack catch/recovery messages, while most primary operations do catch. `errorText` understands structured JSON failures but App predominantly calls `String(error)`, so structured string errors can appear as raw JSON. Use consistent actionable formatting, retaining full diagnostic data in a disclosure/copy action. Native app/platform failure was not reproduced here.
* **First-run/empty inventory:** Profile view is conditional on `profile` existing (`1652`), producing no screen content when clicked before a model is loaded. Inventory has a header/table but no dedicated zero-model empty state/recovery action. The global notice gives some context; put it next to the relevant workflow.
* **Paths and auditability:** inventory directories ([App.css:111](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.css#L111)), log path (`99`) and catalog filenames (`295`) are truncated on desktop, despite the design document's full-path rule. Model paths elsewhere wrap correctly. Provide full selectable values or an explicit expansion/copy control without hiding necessary identity.
* **Design drift:** `.signal` and `.signal.live` use box shadows ([App.css:43–44](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.css#L43-L44)) despite normative no-shadow wording. Several unrelated evidence actions and all catalog downloads are green primary buttons, exceeding the “one primary per screen” design rule. Treat these as minor design consistency, not security findings.
* **Performance beyond preview:** catalog search sends the full snapshot model list across IPC on every keystroke ([App.tsx:873–900](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L873-L900)) without debounce; all catalog cards render eagerly. For the current finite catalog this is a measurable risk rather than proven unacceptable latency. Download progress updates global App state for every event. Extracting independent screens and throttling presentation updates can reduce work after correctness ownership is repaired. Avoid recommending virtualization without measuring actual inventory/catalog size and WebView2 frame time.
* **Accessibility motion:** CSS reduces animation duration and sets `scroll-behavior:auto` (`507–509`), a positive foundation. The “Jump to Advanced” action explicitly passes `{behavior:"smooth"}` ([App.tsx:1669](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L1669)); respect reduced-motion preference in that JS call too, and move keyboard focus to the expanded region if appropriate.
* **No source-injection sink found in reviewed frontend:** no `dangerouslySetInnerHTML`, `eval`, or raw HTML interpolation appeared; text/logs use React text nodes/pre. This is a scoped static observation, not proof the complete app is XSS-free.

### Strengths to retain

1. Native semantic buttons, inputs, selects, fieldsets and details/summary are used extensively. Most fields have enclosing labels, main navigation has a named nav landmark, and operation notices use live status roles. There are meaningful words beside status colors.
2. Secrets remain typed into password inputs and are passed directly to native credential commands; successful save clears drafts, provider switch clears the API-key draft, and UI status uses masked suffixes. No frontend localStorage writes of cloud/HF tokens were found ([App.tsx:503–528](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L503-L528), `640–667`, `1312`, `1921`).
3. Runtime and catalog loads use request sequencing; delayed Tauri listener registration is disposed safely after unmount. The runtime-install and managed-health progress listeners filter events by the active install key ([App.tsx:913–937](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L913-L937)); download events are maintained by their payload key (`902–911`). The listener helper has explicit tests ([model.test.ts:520–533](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.test.ts#L520-L533)). Preserve and extend this pattern.
4. Native backend remains authoritative for trusted command construction/launch and catalog filtering. The UI explicitly calls browser-preview limitations unknown rather than inventing hardware measurements.
5. Evidence types distinguish measured/estimated/unknown, include compatibility keys, preserve raw failure records, and privacy export has an explicit reviewed local-file action. These concepts are sound even where the component lifecycle currently undermines them.
6. Runtime catalog has explicit loading, empty, error and retry states, supports inspection/managed install provenance, and avoids claiming accelerator success from an install button alone.
7. The design is coherent and deliberately dense for a local technical control plane. Existing breakpoints, wrapping exact commands, native disclosure sections, reduced-motion CSS, and 110px bottom screen clearance provide a useful foundation.
8. Pure model helpers have targeted behavior tests covering runtime trust/status, cancellation labels, capacity evidence, bounded progress, migration and compatibility. The next test investment should exercise these helpers through actual UI workflows rather than add more source-pattern assertions.

### Suggested acceptance matrix

| Area | Packaged-app acceptance scenario | Required evidence |
|---|---|---|
| Identity | Select B, rescan A/B, type new runtime, inspect, start | Visible model/runtime equal exact start IPC profile and server snapshot |
| Tuning | Start A, navigate/select B, receive late progress/report, adopt | Report remains associated with A; no save under B; runtime adoption policy explicit |
| Cloud | Switch providers while key/model/probe reads complete out of order | Active tab, credential provider, model list and tuning request agree |
| Preview | Type rapidly while runtime probes are slow | Bounded probe count; responsive form; latest settings only in exact command |
| Evidence lifecycle | Run, navigate away/back, cancel, change profile, compare runs | Run/cancel handle survives; results/history persist with distinct provenance |
| Preflight | Change adapter/manual budget after a result | Old evidence is marked stale; deliberate empty selection remains possible |
| Downloads | Start in A; change folder/build/filter; stop | Correct original job is cancelled; outcome reflected truthfully |
| Persistence | Corrupt records and fail storage after a measured result | UI recovers; valid native result remains visible; failed save is separately explained |
| Keyboard | Tab through paths, paired controls, provider tabs, navigation | Visible focus, unique names, expected tab keys, coherent focus after navigation |
| Display | Narrow/zoomed windows and high-contrast preference | No clipped evidence controls; usable targets and ≥4.5:1 regular text |
| Failures | Runtime exits, probes fail, opener/dialog reject | Last logs/failure evidence and actionable recovery remain visible |

These scenarios are proposed acceptance gates, not tests claimed as executed by this review.


## Cloud integration, IPC and operational controls

This section covers the command surface and cloud integration independently of the subsystem findings below. Source references use the audited commit `e530371b056cd8e049c2246dbb151aa407bf359f`. An IPC command being callable by the main WebView is not, by itself, a vulnerability: the application deliberately provides privileged local operations. The concern is whether every route enforces the same rules and whether untrusted data can exhaust resources or cross an unintended trust boundary.

### IPC-01

**Normal server startup blocks the main thread and prevents stop/status during startup.**

**Priority: High.**

**Priority: High. Evidence: confirmed source behavior plus the framework's documented execution model; Windows freeze duration was not measured in this audit.**

`start_server` is an ordinary synchronous Tauri command. It takes `state.server` at the beginning, prepares and starts the executable, and then waits up to 600 seconds for health while retaining that mutex. Only after successful health and another effective-context request does it place the child in `ManagedServer`. `stop_server` and `server_status` need the same mutex. A slow or failed model load therefore cannot be stopped through the normal Stop command while startup is in progress. There is also no startup cancellation flag. See [lib.rs:1415–1496](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L1415-L1496).

This is more than contention between worker threads. Tauri documents that commands without `async` run on the main thread unless explicitly marked `#[tauri::command(async)]`; neither mechanism is present here. Its guidance recommends asynchronous commands for heavy work to avoid UI freezes. [Tauri command documentation](https://v2.tauri.app/develop/calling-rust/#async-commands).

**User consequence.** A model that requires substantial loading time, an incompatible runtime, or a server that never becomes healthy can make the control application unresponsive precisely when a user needs cancellation and diagnostic feedback. A status poll cannot expose an intermediate starting state because the slot remains unavailable until the wait ends. The existing timeout eventually limits one health attempt, but it does not make startup interactive.

**Related surfaces.** Synchronous `preview_command`, `inspect_runtime`, `check_runtime_health`, `scan_models`, `read_gguf_summary`, `preflight_model`, legacy `benchmark_server`, and `replay_benchmark_manifest` also perform filesystem, subprocess, socket, or hashing work. Some of these are much shorter, but the command boundary should classify them by actual cost. The frontend section details why command preview can invoke expensive work for every edit. The runtime section details repeated hashing during runtime enumeration.

**Fix.** Make starting an explicit backend operation with a unique operation ID and `starting/running/stopping/failed/cancelled` states. Reserve the operation under a short lock, release the lock, perform blocking work in `spawn_blocking`, and retain a cancellation flag and contained child handle reachable by Stop. Publish progress through correlated events. Reacquire the state lock only to commit a result if the operation ID is still current. Ensure failure and cancellation terminate and reap the process tree before clearing state. Do not merely add `async` around blocking work: move that work off the async executor too.

**Acceptance test.** Start a benign fixture process that opens the expected port but never returns ready. Verify the UI remains responsive, status reports `starting`, Stop cancels within a specified small deadline, the contained process tree exits, and another start succeeds. Repeat with slow `--help`, unreadable GGUF, early process exit, and window close during startup. Exercise the packaged Windows binary; a pure unit test cannot prove WebView responsiveness.

### IPC-02

**Content Security Policy is disabled.**

**Priority: Medium.**

**Priority: Medium hardening gap. Evidence: confirmed configuration; no exploitable XSS chain was demonstrated.**

The packaged application sets `app.security.csp` to `null` in [tauri.conf.json:24–26](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tauri.conf.json#L24-L26). Tauri's CSP protection is enabled only when configured, and the framework recommends restricting allowed sources to the application's needs. [Tauri CSP documentation](https://v2.tauri.app/security/csp/).

The main WebView can invoke commands that launch executables, write exports, download files, and use stored cloud credentials. A future injection bug in that renderer would therefore have meaningful consequences. The targeted source search found no `dangerouslySetInnerHTML`, `eval`, `new Function`, `innerHTML`, or `document.write` use in production `src`; React's ordinary text rendering is a positive control. CSP absence must not be reported as proof that arbitrary websites can currently execute these commands.

**Fix.** Establish a production CSP that allows bundled assets and the necessary Tauri IPC origins, with narrowly enumerated image/style/font exceptions justified by actual usage. Keep production and development policies distinct. Review the default `opener` permission and custom command reachability alongside the policy. The exact policy should be derived from a packaged build's resource needs rather than copied blindly from a generic web application.

**Acceptance test.** In the packaged app, exercise all screens, dialogs, icons, external URL opening, and inference WebUI navigation with CSP enabled. Verify unexpected inline scripts and remote scripts are rejected, with no broad `unsafe-eval` workaround. Include a hostile model/catalog text fixture to confirm it renders as text. Record any necessary policy exceptions.

### CLD-01

**OAuth callback parsing lacks a complete request and resource budget.**

**Priority: Medium.**

**Priority: Medium robustness issue, with a local denial-of-service prerequisite. Evidence: source-confirmed; no live sign-in or credential exchange was performed.**

The OpenRouter listener binds to loopback on an ephemeral port and uses a strong random PKCE verifier with S256. Those are appropriate protections. However, [cloud.rs:302–309](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/cloud.rs#L302-L309) accepts any request target containing a nonempty `code=` query parameter: it does not require GET, the `/callback` path, or one unambiguous parameter, and it does not percent-decode the code. [cloud.rs:325–378](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/cloud.rs#L325-L378) reads the first line into an unbounded `String`. The overall deadline is checked only when `accept` returns `WouldBlock`, not while a stream is being read or when successive connections are accepted.

The five-second socket read timeout limits an idle read; it is not a total byte budget or a total sign-in deadline. A local process that knows or discovers the callback port can send a very long request line, repeatedly supply stray requests, or send an invalid code before the real callback. An invalid code can prematurely end the listener and cause key exchange to fail. A read timeout currently aborts the entire flow rather than ignoring a malformed probe. These observations do not establish account takeover: the server-side exchange still receives the generated PKCE verifier, and the audit did not demonstrate a way to obtain a valid key without satisfying the provider's exchange rules.

The browser page says `OPENROUTER CONNECTED` as soon as a code is received, before the token exchange and Credential Manager write complete. A network or storage failure can therefore leave the browser claiming success while the application reports failure. [cloud.rs:353–367](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/cloud.rs#L353-L367), [lib.rs:2433–2450](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2433-L2450).

**Fix.** Bound request-line bytes and code length; enforce a monotonic end-to-end deadline on every path; accept only the intended callback method/path; parse and decode query parameters using a URL parser; reject duplicates and malformed encodings; ignore harmless probes without aborting the login. Use a state value if the provider's documented flow supports it, while retaining PKCE. Make the browser response say the callback was received and the user should return to the application, unless exchange and secure storage have actually succeeded. Prevent unlimited simultaneous login attempts.

**Acceptance test.** Valid callback; percent-encoded code; favicon; wrong path/method; duplicate code; empty code; malformed percent encoding; overlong request line; half-open connection; steady trickle of bytes; successive stray connections until deadline; failed exchange; failed Credential Manager write. Test resources locally and stub the remote exchange to avoid real keys or paid requests.

### OPS-01

**Server log writes are unbounded and reuse the same filename.**

**Priority: Medium.**

**Priority: Medium operational risk. Evidence: confirmed write/read design; disk exhaustion was not induced.**

`spawn_server` writes stdout/stderr directly to a temp file with `File::create`, using `server-{port}.log` for normal runs and `tuning.log` for tuning. The former content is truncated on the next start. The displayed log is safely bounded to the last 16 KiB and 12 lines, but the underlying write has no rotation or total size limit. [lib.rs:530–574](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L530-L574), [lib.rs:920–948](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L920-L948).

**Consequence.** A long-running or repeatedly failing runtime can consume disk space despite the small UI log view. Starting again can destroy the previous failure's diagnostic log. Different application instances using the same port/log naming convention can also collide. Local paths and runtime-emitted text remain in the OS temp directory; the README accurately discloses that location but there is no visible retention or export lifecycle here.

**Fix.** Use per-run IDs, a bounded log sink or rotation, a retention policy, and explicit diagnostic export. Preserve the final failure tail in structured evidence before cleanup. Apply user-only storage permissions and safe file creation; do not follow unexpected existing links. Keep the UI tail bounded.

**Acceptance test.** A fixture process emits more than the configured quota; retained disk usage stays bounded, newest diagnostic lines remain available, rotation does not block the child, and a second run preserves the first run's failure identity without overwriting its evidence.

### Cloud integration: positive controls and remaining validation

Credential storage is substantially better than plaintext application settings. `KeyringStore` uses the current and legacy Windows Credential Manager services, migrates old entries, and deletes the legacy copy after a successful write. The frontend receives `CredentialStatus` with a masked suffix rather than the retrieved stored key. A manually pasted key necessarily passes through frontend memory before being sent to Rust, but the stored secret is not read back into the renderer. Provider IDs resolve against a fixed HTTPS allowlist. Secrets are placed in Authorization headers, and ordinary key rejection messages do not echo their values. Relevant implementation: [cloud.rs:102–248](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/cloud.rs#L102-L248), [cloud.rs:414–435](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/cloud.rs#L414-L435).

The cloud advisor receives the entire serialized baseline profile, companion paths, metadata, trial commands, and errors. That data can include user directory names and local project/model names. This is **disclosed behavior**, not an undisclosed-upload finding: the [README:225–247](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/README.md#L225-L247) explicitly lists the fields and tells users not to use AI Tune if they must remain local. A useful product improvement is an in-app first-use data preview and an optional minimal/redacted brief, while retaining enough model facts for useful advice. Do not claim that local inference means the optional tuner is offline.

All six providers share a fixed request shape: chat completions, temperature 0.2, and max_tokens 1200; model lists and responses are read into strings without a response byte cap. Timeouts exist (20-second connect, 30-second listing/exchange, 180-second chat), but there is no provider-specific Retry-After/backoff handling or bounded aggregate cloud budget in this layer. The tuning section describes the stronger, concrete unbounded no-op loop. A payload-size cap, an end-to-end budget, and contract tests for each provider would make network behavior more predictable. See [cloud.rs:469–602](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/cloud.rs#L469-L602).

Do not infer that Anthropic is broken merely because its native API differs: its official documentation supports an OpenAI compatibility layer, including Authorization and the request fields used here. That documentation also describes compatibility limitations. [Anthropic compatibility documentation](https://platform.claude.com/docs/en/cli-sdks-libraries/libraries/openai-sdk). This audit did not use real credentials to verify model listing, quota failures, or inference against any provider. Default model availability, reasoning-model parameter compatibility, and provider retention behavior remain external contract checks, not certified functionality.

Targeted current-tree review found no confirmed embedded production API key. This is not a full-history secret scan, credential rotation review, Windows Credential Manager penetration test, or proof that third-party runtime logs never contain sensitive data.


## GitHub delivery, releases, governance and project health

Audit target: `sato942/localmotive`, main commit `e530371b056cd8e049c2246dbb151aa407bf359f`.
Read-only GitHub plugin API inspection on 2026-09-10. No GitHub mutation, workflow rerun, release change, or source modification was performed.

### Evidence and methodology

- Read root AGENTS.md, all four workflow files, action-pin manifest, gate manifest, packaged verifier, candidate inventory verifier, Windows Sandbox lifecycle scripts, README, historical release tracker and corrective note.
- Read all 156 workflow runs in two API pages (100 + 56), covering 2026-09-02 07:12:43 UTC through 2026-09-10 19:01:02 UTC.
- Read all 110 commits on main in two pages (100 + 10). All 110 map to the GitHub login sato942. This is the inspected main history, not proof that no other contributor ever existed on an unmerged branch.
- Read repository metadata, all three releases, latest release, annotated tag objects, main branch summary, repository rulesets, all four PR records and reviews on the two merged PRs.
- Read full job step summaries for pinned head CI, release-candidate CI, v0.5.0 Release and v0.5.0 Hardware qualify. Read raw logs for pinned head check and Rust audit, v0.5.0 packaged verifier, and failed v0.5.0 Sandbox job.
- Independently extracted the verifier's JSON printed into the Release package job log. It is stored as `release-0.5.0-packaged-from-job-log.json`. This is log-derived evidence, not a fresh local execution or direct download of the public JSON asset.
- Release asset API metadata, sizes and GitHub SHA-256 digests were inspected. The portable digest exactly equals the packaged verifier's recorded digest. Direct GitHub connector fetch of release-download URL returned 404; do not call this a missing asset: API metadata and successful publish/readback job prove the asset exists. No claim is made that this audit downloaded and rehashed all binaries or executed Windows installers.
- Detailed branch protection API GET returned 403 “Resource not accessible by integration”. This prevents inspection of privileged administration settings. The accessible branch summary independently reports protection disabled, and rulesets list returns [].

Evidence files: `github-health-evidence.json` (~API snapshots, all run summaries, job summaries, checks, releases); `release-0.5.0-packaged-from-job-log.json`; `github-health-log-excerpts.txt`.

### Executive judgment

The current source and shipped candidate have genuine successful CI and substantial packaged CPU runtime checks. Delivery has markedly improved beyond v0.4.0: actual portable launch, tamper rejection, repair, seven-stage health, cancellation/restart, artifact inventory, checksum and public-readback evidence are present. Those strengths must not be described as comprehensive installer, GPU, Windows-version, or v0.5 catalog UI qualification.

The most consequential delivery gaps are (1) no premerge CI or branch/ruleset enforcement; (2) demonstrated historical release-tag mutation plus per-job mutable-ref checkouts; (3) an independent hardware workflow that waits for a release while occupying the same runner needed to produce it; (4) Sandbox PASS criteria that allow an MSI uninstall failure; (5) no packaged behavioral checks for the new v0.5 SQLite/catalog/filter/refresh feature set.

Current unsigned distribution is expressly disclosed and accepted in the project's recorded policy. Record it as an accepted residual supply-chain limitation, not a hidden signing failure or a newly imposed publication veto.

### Current repository and release inventory

Repository created 2026-09-02 06:44:27 UTC; public; MIT license; default branch main; not archived; issues enabled; discussions disabled; observed stars 0, forks 0, open issues/PR count 0. Young single-maintainer project; low activity counts are maturity context, not evidence of bad code.

There are four historical PRs: #1 and #4 merged, #2 and #3 closed as duplicate-topic submissions. All authored by sato942. Reviews API returns [] for both merged PRs. #4 was created 2026-09-08 01:46:02 and merged 01:46:30, 28 seconds later; associated historical PR CI runs were ultimately cancelled. No standalone issues were returned by the full issues collection (all four records were PRs). Do not present an empty issue tracker as absence of defects.

Current main: e530371b056cd8e049c2246dbb151aa407bf359f (documentation ship ledger).
Current v0.5.0 annotated tag object: eb52220c50627b964ab3bc69806098ff86c2ada3.
v0.5.0 release source: a4b7127f739f7420232d9b6f63da693d39128d0b.
The one-commit difference between release and main is the post-ship documentation ledger, not an unexplained product-build mismatch.
Tag and main commit GitHub verification fields are unsigned. Authenticode, Git tag signing, catalog Ed25519 signing and build provenance are distinct controls.

Latest release: Localmotive 0.5.0 (unsigned), published 2026-09-10 18:56:46 UTC, draft false, prerelease false, six assets.

| Asset | Bytes | GitHub SHA-256 |
|---|---:|---|
| Localmotive_0.5.0_x64-portable.exe | 19,668,480 | e25539f18be0cc9d2d9d5cca480747b805b9887f7cfa4f3dc389f51bf81b0036 |
| Localmotive_0.5.0_x64-setup.exe | 5,025,586 | 22a7ef75d15900fd0ef12344fed9665fada2d734fc3ce5992296db06899c4545 |
| Localmotive_0.5.0_x64.msi | 7,057,408 | 581ae3d093423bcbbd94034ff6f3be469605ecaaae12087b37ade605d4fc5ee5 |
| SHA256SUMS-0.5.0.txt | 291 | 3717e7efe6b3c5ed9ce458b81b8fa2cdce26a00c50aa9c734e8576e87af278e7 |
| packaged-verification-0.4.1.json | 14,228 | c40ca759b17476ddc5e7f8643702756dd9ed53754ab1399db4ecf2016610a10c |
| candidate-inventory-0.5.0.json | 736 | ee4577e9d69efcc452be657cdb0f5a563456f100e21b854f986d60d6231d3eb3 |

Release link: [GitHub release evidence](https://github.com/sato942/localmotive/releases/tag/v0.5.0)
API source: [GitHub release evidence](https://api.github.com/repos/sato942/localmotive/releases/tags/v0.5.0)

### Actual verification status

| Evidence | Source revision | Observed result |
|---|---|---|
| Main CI #102, run 34517849213 | e530371... | success; rust-audit, check, package-smoke, Security audit all green |
| Release candidate CI, run 34512236377 | a4b7127... | success |
| Release, run 34514283693 | a4b7127... | rust-audit, quality, package, publish all success |
| Hardware qualify, run 34514283563 | a4b7127... | host proof success; clean-account lifecycle failure |
| Packaged behavior record printed in job 103001477265 | a4b7127... | 29 PASS, exact portable digest, source clean |
| Main Rust tests | e530371... | 403 passed, 0 failed, 2 ignored |
| Main Vitest | e530371... | 52 passed, 1 test file |
| Main Node release-gate tests | e530371... | 80 passed; run twice due explicit gate + npm test |
| Main Rust documentation tests | e530371... | 0 tests, passes warning/build gate |
| Main npm audit | e530371... | 0 vulnerabilities |
| Main Rust audit | e530371... | 0 vulnerability advisories; 6 unmaintained warnings, 1 unsound warning |

Main Rust toolchain in logs: rustc 1.98.1 (48a229cea 2026-09-01). No rust-toolchain pin file exists; pinned action commit still installs floating stable toolchain. Node configured major 20, not exact patch. All external GitHub Actions themselves are pinned to 40-hex commit SHAs and covered by an explicit pin manifest and verifier.

Ignored Rust tests:

- core::tests::real_model_tree_companion_order.
- health::tests::qualify_managed_runtime_on_current_host (downloads artifacts and executes on real hardware).

Links:

- [GitHub workflow evidence 34517849213](https://github.com/sato942/localmotive/actions/runs/34517849213)
- [GitHub workflow evidence 34512236377](https://github.com/sato942/localmotive/actions/runs/34512236377)
- [GitHub workflow evidence 34514283693](https://github.com/sato942/localmotive/actions/runs/34514283693)
- [GitHub workflow evidence 34514283563](https://github.com/sato942/localmotive/actions/runs/34514283563)
- [GitHub workflow evidence 34517849213](https://github.com/sato942/localmotive/actions/runs/34517849213/job/103007676318)
- [GitHub workflow evidence 34517849213](https://github.com/sato942/localmotive/actions/runs/34517849213/job/103007676100)

#### What the 29 packaged checks actually prove

Host: Windows build 10.0.26100 x64; AMD Ryzen 9 9950X3D; 32 logical CPUs; NVIDIA GeForce RTX 5090 driver 610.74. Run 2026-09-10 18:53:44.321 to 18:54:14.450 UTC. Host GPU presence does not mean GPU inference was qualified. Selected runtime and repair/health path were CPU; backend recommendation expressly returned CPU because no exact L4 accelerator compatibility record existed.

Checks:

1. Exact artifact digest, source revision, clean checkout and packaged WebView CDP connect.
2. Branding.
3. Backend runtime setup, conservative recommendation, unknown-adapter rejection.
4. Runtime-catalog UI loading/empty/error/rate-limit/scope/blocked-backend states, driven by verifier-injected React state fixtures.
5. Installation IPC rejects frontend-controlled browserDownloadUrl/digest/size/backend/tag and legacy option.
6. Isolated application-data root.
7. CPU runtime installation, intentional tamper, health trust rejection, reinstall repair.
8. Seven-stage CPU health, external cancellation and subsequent restart.

The old filename “packaged-verification-0.4.1.json” does NOT mean the evidence belongs to an old binary: digest and source match v0.5.0. Naming is confusing and the tested scope is still largely the prior release's runtime work.

### Findings

### GH-01

**default branch has no enforced premerge verification.**

**Priority: High.**

Evidence: /branches/main reports protected=false, protection.enabled=false, required_status_checks.enforcement_level=off and empty contexts/checks. /rulesets?includes_parents=true returns []. Current ci.yml:8-10 only triggers on push to main. AGENTS.md states “CI runs exactly this on every push and PR,” and ci.yml:3 says every push and PR; these claims are stale.

Consequence: a PR has no current CI path; code can enter main before quality checks run. Fail-fast dependencies protect package execution inside a run but do not prevent an unchecked commit becoming main. A rule requiring the current push-only checks without adding a PR trigger would deadlock PRs, so change both sides together.

Recommendation: add safe PR checks on GitHub-hosted Windows or an ephemeral isolated runner; require them through a main ruleset; restrict direct pushes and force pushes; apply reasonable review policy for a solo maintainer and document emergency bypass. Keep fork/untrusted PRs away from the owner interactive runner. Confirm by opening a benign test PR and observing required checks run and merge is blocked on failure.

Confidence: high for current branch summary, empty rulesets and trigger; privileged branch-protection details remain inaccessible.

### GH-02

**mutable release tags undermine stable source and evidence identity.**

**Priority: High.**

All-run history shows ELEVEN distinct source SHAs for push-triggered Release runs named v0.4.1 between 2026-09-06 and 2026-09-10. Examples:

- c955bfb3f21dc36a12fcef76669d94be2df16b3c, run 34052145060, Sep 6 18:34.
- af652cf244a56687c2f26055e414e25f63c89dae, run 34066696890, Sep 6 23:22.
- bc8c00fe45b1f30c71c5cf9257ac59a8c1ae7921, run 34124950617, Sep 7 13:00.
- 00d5c7ff20e9ef9c20fded5e69f2943a3c885523, run 34350404682, Sep 9 12:20.
- 2c55b3fd088a81f42b9746b1d94521d6e5547068, run 34415696465, Sep 9 23:10.
- 28b2ee6044118b4740e3e27d613ac1d03655e30c, run 34420337541, Sep 10 00:13.
- 96283b99d62b800d4e86bee471355ffe4ac0e2e0, run 34430058475, Sep 10 02:35.

The same tag ref therefore identified different sources over time, whether moved directly or deleted and recreated. Current v0.4.1 tag object a0ef02... peels to 96283... and matches the final successful run. No equivalent mutation was observed for v0.5.0.

Release checkout uses ${{ github.event.inputs.tag || github.ref }} at release.yml:43,69,155,290. Quality, package and publish independently resolve mutable refs. No job passes one immutable source SHA forward. Candidate inventory verification at publish regenerates the JSON from that job's HEAD; verify_candidate_inventory.mjs:64-72 writes a new record instead of comparing the producer's existing inventory/source identity. Main-ancestry check occurs only in quality. This creates a possible source/evidence mismatch if a ref changes between jobs; this audit did not demonstrate such a mismatch in v0.5.0.

Recommendation: create immutable version tags once; use new prerelease/patch names for retries on changed source; protect tag updates/deletions; resolve tag to commit once, pass SHA to every checkout, and assert equality with event and packaged evidence before publication. Verify rather than overwrite the original signed/digested candidate manifest. Preserve historical corrections with new evidence, not replaced source names.

Links: [GitHub workflow evidence 34415696465](https://github.com/sato942/localmotive/actions/runs/34415696465) and [GitHub workflow evidence 34430058475](https://github.com/sato942/localmotive/actions/runs/34430058475) .
Source: [Source: github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/release.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/release.yml)

### GH-03

**v0.5.0 clean-account verification fails because its scheduling waits on itself.**

**Priority: High.**

hardware-qualify.yml triggers independently on the same v* push as Release; its clean-account job depends only on host-proof, not the package job. Both workflows target the same self-hosted runner. host-run-lifecycle.ps1:48-65 waits 5 minutes for public installers.

Observed exact v0.5 sequence:

- Host proof succeeds.
- Sandbox availability check succeeds.
- Lifecycle job starts waiting at 18:28:15; “release not found” repeats.
- 18:33:20: “Timed out after 5 minute(s) waiting for GitHub Release v0.5.0 assets”.
- Release quality does not start until 18:37:37; package starts 18:42:53; assets publish 18:56:46.
- Hardware run concludes failure and uploads only the host attestation, no Sandbox lifecycle evidence.

This is an orchestration defect, not evidence that the v0.5 installer itself fails. On the one observed runner, the wait occupies capacity needed by the producer. Merely extending timeout can worsen the stall.

Recommendation: run lifecycle against exact packaged candidate artifacts via -CandidateDir as a dependent job before publish, or trigger a non-gating post-release check only after completed publication. Pin binary checksums/source. Make the status explicit if release policy still permits missing lifecycle evidence. Correct TODO-0.5.md:361: “Sandbox unavailable” is contradicted by the successful availability step; the demonstrated failure is missing release assets during polling.

Link: [GitHub workflow evidence 34514283563](https://github.com/sato942/localmotive/actions/runs/34514283563/job/102996152596)
Sources: hardware-qualify.yml:122-160; scripts/sandbox/host-run-lifecycle.ps1:48-69.

### GH-04

**Sandbox lifecycle PASS can conceal incomplete uninstall or wrong-version upgrade.**

**Priority: High.**

scripts/sandbox/run-lifecycle-in-sandbox.ps1:119 explicitly logs a warning and continues when an executable remains after MSI uninstall; :120 immediately emits msi-fresh-install-launch-uninstall PASS. This conflicts with the test's name and prevents that green record from proving removal.

Upgrade checks :124-132 only find an executable and keep it running for eight seconds. They do not assert executable version/hash changed from old to current, preserve settings/profiles, verify catalog DB migration, or test MSI upgrade. The fixed previous baseline is v0.4.0 (hardware-qualify.yml:160), so the ordinary 0.4.1→0.5.0 user transition is absent.

Recommendation: fail if the expected install scope remains, verify registry/product identity and targeted uninstall outcomes, compare exact current executable version/digest after upgrade, exercise realistic persisted settings and SQLite data, cover previous supported release(s), and distinguish startup-smoke from app-functional verification. Add negative tests proving leftover-executable and unchanged-version scenarios cannot return PASS.

This is a confirmed verifier defect; no claim is made that MSI uninstall actually leaves the file in a normal run.

### GH-05

**new v0.5 catalog and SQLite features lack packaged end-to-end release coverage.**

**Priority: Medium.**

Release invokes scripts/verify_041.mjs only. The 29 observed checks concern runtime management and prior trust boundary work, not the new HF model catalog, SQLite mirror, rich filters, hardware-fit controls, user override persistence, signed refresh fallback, cooldown, or bounds visible through their new UI.

Rust and unit tests for many of these features are present and pass; this finding is the gap between unit tests and released WebView/IPC/database interaction. AGENTS.md asks for a release-version verifier covering that release's changes. No verify_05* file was found.

Recommendation: add v0.5 packaged scenarios in a temporary profile for first fill, offline cache, valid signed refresh and invalid signature fallback, SQLite corruption recovery, filters/facets/pagination, user overrides persistence and removal, refresh/cooldown, and UI-visible error states. Bind evidence to source and installer/portable digests. Keep injected runtime-state UI checks but label their scope.

### GH-06

**failure evidence is lost by the Sandbox host harness.**

**Priority: Medium.**

host-run-lifecycle.ps1:122-124 throws on a FAIL result before :126-130 copies JSON/log to the release-evidence output. Timeout also throws without copying. hardware-qualify.yml:181-187 uploads only the output directory and ignores missing files. Thus a green “Upload sandbox evidence” step can mean no evidence artifact.

Observed v0.5.0 hardware run lists one hardware-attestation artifact and no sandbox-lifecycle artifact, even though its upload step says success.

Recommendation: copy failed result/log and host diagnostics in finally before rethrowing; always write a structured failure/timeout record with version/source/digests; set upload behavior to surface genuinely missing expected evidence. Separate “upload step successful” from “artifact exists” in dashboards.

### GH-07

**support statements and historical corrections remain inconsistent with public evidence.**

**Priority: Medium.**

Public v0.4.0 release notes still label three L2/PARTIAL rows Supported. Repository corrective note explicitly says “Proposed correction,” “This draft does not modify the published release,” and asks review before editing it. v0.4.1 notes explain the correction, but a person downloading v0.4.0 directly still sees its original false support label. No release edit was performed during this audit.

README:299 broadly says Windows x64 qualification tested on AMD Zen 5/NVIDIA Blackwell “including clean-account ... when release evidence is present for that version”; :303 qualifies by version. The actual v0.5.0 health evidence is CPU and clean-account failed before installation, so readers should not infer CUDA/Vulkan or v0.5 lifecycle qualification from the header. v0.5.0 release notes correctly disclose broad L4 limitations, which is a positive.

README:305 and hardware workflow summary still say 01:00–06:00 Dubai and GitHub-hosted main release; current CI/release run on the same self-hosted host outside that historical schedule. AGENTS.md still claims every push/PR CI.

Recommendation: place explicit factual correction on the affected v0.4.0 release, keep immutable binaries, publish a compact version→host/backend/lifecycle evidence matrix, and replace stale operational text. State “CPU packaged checks on this host” separately from accelerator support.

### GH-08

**supply-chain provenance is incomplete beyond checksum integrity.**

**Priority: Medium.**

Positive: actions SHA-pinned; npm ci and Cargo --locked; least-privilege contents:read by default; release-only contents:write; exact three-file checksum set; nonempty artifacts; package checks; published readback downloads and checksum validation; catalog signing private key isolated to the hosted sign job; no current fork-PR route to owner runner.

Limitations:

- Authenticode intentionally absent, unsigned label everywhere; no signer identity assurance from checksums alone.
- No observed GitHub build-provenance attestation or SBOM asset/workflow step. Local “attestation” JSON records are not cryptographically signed SLSA/in-toto build attestations.
- Rust stable and Node major float; no exact Rust toolchain file; Windows runner image/package manager environment mutable.
- Build, hardware test and publication share one owner interactive persistent Windows machine. CARGO_HOME/RUSTUP_HOME isolation protects toolchain directories, not a full security boundary or hermetic environment.
- Release candidates retained 14 days, public readback 90 days; public evidence JSON persists as release assets, but deeper ephemeral logs/artifacts need a long-term retention policy.
- Checksum publication provides integrity against accidental or out-of-band corruption, not independent publisher authenticity if release upload privileges are compromised.

Recommendation: preserve the accepted unsigned policy as disclosed; add signed build provenance and SBOM if feasible without paid Authenticode, pin exact toolchains and build image/tool versions, isolate release build/signing privilege from normal owner credentials, document key rotation/recovery and artifact retention. Do not pretend catalog Ed25519 signing authenticates application installers.

### GH-09

**governance and ongoing maintenance processes are not yet established.**

**Priority: Low.**

No SECURITY.md, CONTRIBUTING.md, CODEOWNERS, CODE_OF_CONDUCT, issue/PR templates, Dependabot or Renovate configuration was found in the inspected tree. There are no CodeQL/SAST, license policy, SBOM or scheduled dependency audit workflows among the four workflow files. GitHub secret-scanning/Dependabot settings and alerts could not be assessed via allowed API access, so absence of those services must not be claimed.

110 main-history commits map to one login; two merged PRs have no reviews; no standalone issue backlog. Good detailed source policy exists in AGENTS.md, but operational bottlenecks and unresolved known qualification work are spread through long historical ledgers.

Recommendation: add a security reporting/contact policy, contributor setup/support guidance, lightweight issue templates, automatic dependency-update PRs and scheduled audits, explicit owner/severity/acceptance criteria for outstanding findings, and backup/recovery instructions for runner and signing assets. Avoid bureaucracy unrelated to this small project.

### GH-10

**hardware “attestation” stub can declare HOST_MATCH despite a mismatched host.**

**Priority: Medium.**

hardware-qualify.yml:53-57 merely warns if CPU/GPU names do not match expected models. :85-87 unconditionally writes three rows with HOST_MATCH (Zen5 CPU, Blackwell CUDA, Blackwell Vulkan). The explicit supportClaimPolicy correctly says full packaged L4 still required, but HOST_MATCH itself would be false when runner labels drift or hardware is replaced.

Recommendation: derive each row from actual detected hardware, emit MATCH/NO_MATCH/UNKNOWN, include exact source/runtime/driver identity, and fail the intended host-proof job on mismatch. Do not hardcode qualification statuses.

### CI history and reliability context

All accessible 156 runs:

| Workflow | Success | Failure | Cancelled | Total |
|---|---:|---:|---:|---:|
| CI | 42 | 19 | 41 | 102 |
| Release | 12 | 17 | 1 | 30 |
| Hardware qualify | 11 | 4 | 4 | 19 |
| Catalog build/sign | 3 | 2 | 0 | 5 |
| Total | 68 | 42 | 46 | 156 |

This is all short project history, not a production availability/SLO metric. Cancellation is configured deliberately for CI and release concurrency and should not be counted as test failures. Development iteration, retags and earlier workflow versions account for much of the history; current head and current release are green. Nevertheless repeated long-running/cancelled jobs and many release retries show the cost of a shared runner and shipping-by-retrying mutable tags.

Catalog workflow is manual only. It builds/validates a candidate in one hosted job and signs/validates in a dependent hosted job, then retains candidate/signature artifacts for seven days. Despite workflow name “Publish curated catalog,” it does not commit/publish changes to main; final promotion is manual. This is a controlled separation, but documentation should explain the handoff and recovery before artifacts expire.

### Rust audit interpretation

Pinned-head audit log reports database commit b50980aad8b8f14f77e25a97b32dd94bf008b0af, advisory count 1243, lockfile dependency count 566, no ignored advisories, 0 vulnerability advisories, 7 informational warnings.
Warnings:

- proc-macro-error 1.0.4, RUSTSEC-2024-0370, unmaintained.
- unic-char-property 0.9.0, RUSTSEC-2025-0081, unmaintained.
- unic-char-range 0.9.0, RUSTSEC-2025-0075, unmaintained.
- unic-common 0.9.0, RUSTSEC-2025-0080, unmaintained.
- unic-ucd-ident 0.9.0, RUSTSEC-2025-0100, unmaintained.
- unic-ucd-version 0.9.0, RUSTSEC-2025-0098, unmaintained.
- glib 0.18.5, RUSTSEC-2024-0429, unsound VariantStrIter; patched >=0.20.0.

Green “Security audit” therefore means no blocking vulnerability advisories according to that action's policy, not zero security/maintenance findings. GTK/glib is ordinarily the Linux-side Tauri graph, so do not label this a demonstrated Windows exploitable vulnerability without target dependency/reachability proof. Target relevance remains a Windows dependency-graph verification task.

### Suggested action order

1. Add safe premerge CI and enforce main/tag rules.
2. Stop tag reuse; pin one resolved source through quality/package/publish.
3. Fix lifecycle orchestration to consume candidate artifacts and record all outcomes.
4. Make MSI uninstall/upgrade assertions honest; add catalog/SQLite packaged acceptance.
5. Correct public v0.4.0 claims and publish explicit version-scoped evidence table.
6. Pin toolchains, add artifact provenance/retention, dependency monitoring and security reporting.


## Tests, dependencies, maintainability and documentation

Pinned repository: `sato942/localmotive` at `e530371b056cd8e049c2246dbb151aa407bf359f` (version 0.5.0). Read-only source review. Line references below apply to this exact revision. Findings are recommendations, not source modifications.

### Evidence and limitations

- Read `AGENTS.md`, README, current product/runtime/option/catalog documents, qualification guide, v0.5 tracker, manifests/lockfiles, CI definition/gate policy, test inventory, substantial release-gate and packaged harness code, and the SQLite/builder paths underlying new 0.5 features.
- The audit ran `npm ci --ignore-scripts --no-audit --no-fund`, `npm run check`, and `npm audit --json`. Reported observed outcomes: installation exit 0; full frontend/check pipeline exit 0; Vitest 52 passed and Node release-gate suite 80 passed; catalog, branding, research anchor, qualification, icon checks and production build passed; npm advisory query exit 0 and zero advisories. Vite 7.3.6 built 323.29 KB JavaScript (95.40 KB gzip), 40.43 KB CSS (8.41 KB gzip). These results are current audit evidence, unlike historical claims in docs.
- No local Rust suite, Clippy, compiled Windows app, Windows installer, GPU/runtime qualification, accessibility screen-reader testing, or authentic provider API interactions were performed by this review. Remote CI and other source areas are covered elsewhere in this report. Static Rust test annotations are an inventory, not newly observed passes or coverage.
- Dependency inventory includes all lockfile platforms and build/test dependencies. It does not imply that every locked crate enters the Windows executable. No new Rust CVE/advisory assertion is made. A clean npm advisory scan is evidence about the registry advisory response at audit time, not a proof of absence of malicious or unknown defects.

### Confirmed findings owned here

### QD-01

**Catalog builder never advances its 90-day cutoff.**

**Priority: Medium.**

**Evidence:** [scripts/build_catalog.mjs:13–17](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/build_catalog.mjs#L13-L17) initializes the cutoff from literal `new Date("2026-09-10T00:00:00Z")`, then subtracts configured days. `:69–70` filters against this cutoff. In contrast `:209–218` assigns the emitted catalog `updated` from the actual current date and claims `cutoffDays: 90`. README `:151–152` and catalog README `:119–122` describe recent/last-90-days filtering.

**Impact:** Every future build retains the June 12, 2026 threshold instead of a rolling window. Increasingly old repositories qualify while the emitted document carries a fresh update date and 90-day policy. This is a future deterministic correctness defect, not evidence that the September 10 artifact already violated its own 90-day cutoff. Popularity sorting/caps may reduce which stale entries happen to survive, but do not repair the threshold.

**Targeted reproduction:** Created scratch `audit-notes/catalog-builder-repro/catalog/providers.json` with one explicit fixture author and a Node import module that overrides only `globalThis.fetch` and zero-argument `Date`/`Date.now`. Executed the repository's actual unmodified builder:

```sh
node --import ./mock.mjs /workspace/scratch/3f794cee911c/localmotive-source/scripts/build_catalog.mjs --stdout
```

Mock date: December 10, 2026. Fake Hub model metadata: July 1, 2026. Observed exit 0, `resolved 1 repos, 1 files`, emitted `updated: "2026-12-10"`, `cutoffDays: 90`, and retained that July 1 model (162 days old). No live network request and no repository files were written.

**Fix:** Derive cutoff from current UTC date or a named, explicit `--as-of` option for reproducible snapshots. Include actual `asOf` and cutoff date in publish evidence. Extract date selection and discovery/deduplication rules into importable pure helpers and test boundary dates, future dates, leap days, malformed `lastModified`, and deterministic snapshots.

**Related bounded-discovery limitation:** Builder `:13` defaults to `providers.maxReposPerAuthor` or 20; `--full` only increases to 100. Request `:62` has `limit=100`, `:70` slices the first page, and there is no continuation handling. The v0.5 tracker `:126` says full publish scans without the dry-run's 20 cap. Treat this as documentation/completeness debt until the workflow invocation and desired curation cap are considered: a curated intentional cap is acceptable, an implied exhaustive allowlist scan is not. Document the cap and whether pagination is intentionally omitted.

### QD-02

**New product flows lack automated component and orchestration contract tests.**

**Priority: Medium.**

**Evidence:** [vite.config.ts:13–17](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/vite.config.ts#L13-L17) only includes `src/**/*.test.{ts,tsx}`; the sole matching file is `src/model.test.ts` (52 cases). There are no App/V03EvidencePanel component tests, DOM environment, browser test runner configuration, or coverage configuration in the supplied project. `scripts/verify_041.mjs` is the retained release harness and targets runtime installation/recommendation/tampering/health/UI. Its “catalog” paths are the **runtime catalog**, not the new HF catalog/SQLite feature. [scripts/tests/release-gates.test.mjs:1068–1087](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/tests/release-gates.test.mjs#L1068-L1087) validates new mirror/user-row work by checking that function names and labels occur in files.

**Why this matters now:** The isolated SQLite reproductions and source review confirm defects in the actual orchestration and collision semantics despite six passing SQLite helper tests. [catalog_db.rs:522–637](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog_db.rs#L522-L637) covers in-memory mirror roundtrip, version stamping, direct corrupt-database rebuild, preservation on direct `mirror_verified_catalog`, basic invalid overrides and refusal to remove a curated row. None drives `fetch_model_catalog` → disk mirror → new application start → UI load. The frontend's `loadModelCatalog` at [src/App.tsx:463–500](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx#L463-L500) chains backend fetch, local rows, facets and token state; a failure in those boundaries is not tested by pure `model.ts` helpers.

**Contracts to add first:**

1. First HF catalog load succeeds, app restarts within the cooldown, cached data and download authorization remain usable.
2. Offline/corrupt-cache fallback does not stamp unsuccessful network retrieval as successful freshness; explicit refresh cooldown does not block reading existing data.
3. Healthy mirror refresh preserves local user rows; corrupt mirror rebuild takes the actual failing migration path and recovers.
4. Override ID and filename collisions cannot relabel, transfer or delete curated data; replacing a row removes obsolete files atomically.
5. UI applies filters after load/reload and hardware updates; older asynchronous response cannot replace a newer query; failures render useful retained data.
6. App load tolerates malformed saved profiles/denied browser storage; sensitive operation errors preserve recoverability.
7. Accessibility smoke checks keyboard navigation, labels, focus movement and status announcements on the actual catalog/profile workflows.

Use DOM-level tests with IPC mocked only at the boundary for frontend orchestration, and real temporary SQLite databases plus injected fetch boundaries for backend operations. Keep packaged Windows tests for actual IPC serialization, filesystem/process behavior and representative full flows. A coverage percentage alone is not the target; failing regression tests at known broken boundaries are.

### QD-03

**Packaged UI harness couples to private React internals and injects states without exercising their real transition.**

**Priority: Medium.**

**Evidence:** [scripts/verify_041.mjs:390–457](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_041.mjs#L390-L457) polls the private React fiber tree, searches `__reactFiber$`, walks `memoizedState`/`next`, guesses hooks by object shapes and boolean position, and obtains `queue.dispatch`. `:476–484` directly dispatches catalog, error and loading states. `setScenarioAndRefresh` at `:496–498` only calls state injection; it does not click Refresh or await a real request. Loading check `:680–702` proves rendering of manually supplied state and manually resolves it. The Node test [scripts/tests/release-gates.test.mjs:854–868](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/tests/release-gates.test.mjs#L854-L868) actively requires this mechanism's source strings.

**Impact:** A React implementation change or new similarly shaped hook can break the release gate despite correct product behavior. Conversely, injected loading/error states can still render correctly when the actual request lifecycle is broken. The harness itself documents a previous hardcoded hook-offset bug at `:407–410`, concrete evidence this coupling has already been fragile.

**Scope:** This does not invalidate genuine IPC/install/tamper/health checks in the same harness. Label synthetic presentation checks as such and do not equate them with end-to-end async request correctness.

**Fix:** Extract screens/components and use public React/DOM testing interfaces for synthetic presentation cases, with backend/network behavior injected at a stable boundary. Use the packaged harness to interact through controls and public IPC and await observed state, not to reach into React memory. Replace fixed cancellation delay `:1000` (250 ms) with an observed backend progress/active-run condition where feasible, so very fast or slow hosts do not create timing-only failures.

### QD-04

**Active-looking docs disagree on shipping, persistence, verification status and architecture.**

**Priority: Low.**

| Document evidence | Current implementation/evidence | Required correction |
|---|---|---|
| [docs/RUNTIME_MANAGER.md:104–110](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/RUNTIME_MANAGER.md#L104-L110) says release candidate **must** use signed build/certificate/timestamp | README `:311–329`, v0.5 tracker `:7`, CHANGELOG `:15–18` explicitly defer signing and ship disclosed unsigned full releases | Mark signing procedure optional/deferred and point to the current release policy. Keep old history frozen rather than rewriting historical evidence. |
| [docs/PRODUCT.md:7](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/PRODUCT.md#L7) says platform `web`; `:11` says SQLite-ready; `:27,35` describe 0.4.1 | Tauri desktop app is 0.5.0; `catalog_db.rs` implements SQLite and [lib.rs:2739–2758,2767–2788](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2739-L2758) uses it | Clarify schema metadata vs deployment platform; describe actual desktop platform and active local mirror. |
| [docs/RUNTIME_MANAGER.md:7,52–60](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/RUNTIME_MANAGER.md#L7) frames 0.4.1 contract | App manifests are 0.5.0, runtime pin still b10816 | Label which details are unchanged cross-version runtime policy and which are release-specific evidence. |
| README `:259–276` lists local storage and credentials/logs but does not identify catalog SQLite/cache/stamp | [catalog_db.rs:32–34](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog_db.rs#L32-L34) writes `catalog-mirror.sqlite`; [lib.rs:2680–2685](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs#L2680-L2685) uses app cache directory (temp fallback); signed JSON cache and refresh stamp also persist | Add storage matrix for settings, signed cache, SQLite user rows, cooldown, runtime downloads and evidence stores with reset/export/back-up guidance. User-added data living in a rebuildable cache deserves explicit lifecycle guidance. |
| AGENTS “CI runs exactly this on every push and PR”; CI header `:3–4` makes similar claim | [.github/workflows/ci.yml:8–10](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/ci.yml#L8-L10) only triggers push/main; no pull_request trigger | GH-01 covers premerge enforcement. Update contributor instructions and add safe hosted PR checks rather than running untrusted PR code on personal hardware. |
| [docs/qualification-tests.md:63–91](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/qualification-tests.md#L63-L91) is a September 6 snapshot with 0.4.1 counts 48/50, FAIL/PENDING/BLOCKED claims, zero hardware runs | Current local check pipeline passes; selected runtime/release status must be determined from fresh remote evidence | Move into dated history or add prominent frozen snapshot notice plus current evidence index. Its date qualifier means these old counts are not inherently fabricated; presentation as the current “have and need” guide is misleading. |
| [docs/qualification-tests.md:157](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/qualification-tests.md#L157) instructs `verify_versions.mjs 0.4.1`; `:197` points to `history/TODO-0.4.1.md` | Manifests are 0.5.0 and actual path is `docs/history/TODO-0.4.1.md` | Generate/currentize runnable commands and fix path. |
| [catalog/README.md:119–122](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/catalog/README.md#L119-L122) says unresolved repo is skipped without replacing last catalog | Builder `:65–67,145–147,226–228` accumulates problems and aborts publication unless `--allow-empty` | State fail-the-whole-build policy accurately and explain exceptional override. |
| `AGENTS.md` catalog schema rule requires previous-version backward compatibility | [catalog/README.md:95–96](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/catalog/README.md#L95-L96) intentionally rejects schema v1 and current loader supports schema 2 | Record approved compatibility decision explicitly; document old-client cache/bundled fallback expectations and update contributor contract if intentional. |
| Architecture maps in AGENTS/README omit new evidence/calibration/measurement/health/sharing/recommend/preflight/artifact/catalog_db modules | These are substantial active source modules | Update module ownership map and data-flow boundaries rather than rely on a 0.2/0.3-era map. |

`docs/history/TODO-0.5.md` is unusually detailed about red/green commands and decisions, which is a strength. Its `:194–199` records SQLite closeout before `:201–226` older “mirror is next/open work” notes. Preserve both as history but present dated/commit-ordered decision records or one authoritative current-status table so a reader can distinguish superseded decisions. Do not treat a checkbox as proof; the tracker itself correctly says this.

### QD-05

**Build instructions understate the Node minimum and toolchain is not fully reproducible.**

**Priority: Low.**

**Evidence:** README `:363` says Node.js 20. [package-lock.json:2400–2418](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/package-lock.json#L2400-L2418) locks Vite 7.3.6 requiring Node `^20.19.0 || >=22.12.0`. `package.json` has no `engines`, `packageManager`, or local Node-version file. CI uses `node-version: 20` ([.github/workflows/ci.yml:31–34](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/ci.yml#L31-L34)) so it resolves a supported recent Node 20 at the time, but the broad README admits older 20.x versions unsupported by locked Vite. Rust toolchain action is SHA-pinned but tracks stable, and there is no `rust-toolchain.toml` or `rust-version` in Cargo manifest.

**Impact:** A new contributor can follow instructions with an older Node 20 and hit engine/build failures. Lockfiles protect dependency selection, but source/compiler/npm changes can still change builds and lint gates over time.

**Fix:** Declare the actual minimum in package engines and README, add project toolchain files and an explicit update procedure; record Node/npm/Rust versions in release evidence. Exact toolchain pinning is an engineering choice, not a security defect by itself. Continue using `npm ci` and `cargo --locked` in CI.

### QD-06

**Imported llama-server reference has missing provenance and broken relative links.**

**Priority: Low.**

**Evidence:** `docs/LLAMA-SERVER-README.md` begins as an imported upstream README with no source commit/date identified in its opening metadata. References still target the upstream directory layout. Local-link scan found 10 missing relative targets at lines 14, 18, 347, 573, 748, 1395, 1649, 2089, 2096 and 2140: e.g. `../../docs/multimodal.md`, `../../docs/function-calling.md`, `../../tests/test-json-schema-to-grammar.cpp`, `../embedding`, `README-dev.md`, `chat.mjs`, `chat.sh`, and UI constants. [docs/OPTION_MAP.md:3](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/OPTION_MAP.md#L3) calls this its source.

**Impact:** Contributors following option/multimodal/tooling documentation encounter broken links or mistake a historical server option table for the capabilities of the pinned runtime. Runtime `--help` remains correctly authoritative by repository policy, reducing launch risk.

**Fix:** Add exact upstream repository path/commit/hash and import date, resolve internal links to immutable upstream URLs, and add a small local-link validator or exclude clearly labeled upstream vendored references intentionally. Separate app option mapping from upstream-generated option reference.

### Testing strengths and measured scope

#### Static Rust inventory (not executed in this review)

405 `#[test]` annotations across Rust modules, including 2 explicitly ignored tests. Target `#[cfg]` can change actual test count. The current tracker reports 403 passed/2 ignored on the Windows run; use remote observed logs rather than this static count to establish current result.

| Module | Test annotations | Ignored annotations | Total source lines (includes tests) |
|---|---:|---:|---:|
| runtime.rs | 92 | 0 | 7069 |
| core.rs | 52 | 1 | 2996 |
| lib.rs | 45 | 0 | 4226 |
| download.rs | 38 | 0 | 2383 |
| catalog.rs | 34 | 0 | 2004 |
| measurement.rs | 24 | 0 | 1455 |
| evidence.rs | 21 | 0 | 1196 |
| health.rs | 15 | 1 | 1739 |
| preflight.rs | 15 | 0 | 1170 |
| artifact.rs | 11 | 0 | 745 |
| tune.rs | 11 | 0 | 1002 |
| calibration.rs | 9 | 0 | 778 |
| cloud.rs | 8 | 0 | 774 |
| gguf.rs | 7 | 0 | 601 |
| proc.rs | 7 | 0 | 528 |
| catalog_db.rs | 6 | 0 | 637 |
| recommend.rs | 5 | 0 | 616 |
| sharing.rs | 5 | 0 | 715 |

- Tests are extensive in the high-risk runtime/download/parser/process areas, use real temporary files/sockets, and cover malformed data, digest/identity mismatch, archive limits, cancellation, output evidence, and qualification boundaries. Pure TypeScript tests exercise profile normalization, request sequence guards, runtime/install status, download readiness, evidence provenance, hardware budget and metrics decisions.
- Rust test modules reside alongside implementation, making test ownership clear. `AGENTS.md` states behavioral/regression tests are required and distinguishes fixture/runtime evidence from product support, a strong engineering policy.
- [vite.config.ts:13–17](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/vite.config.ts#L13-L17) intentionally prevents ignored research/third-party tests from contaminating the application suite.
- Two ignored probes are disclosed: [core.rs:2542–2566](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs#L2542-L2566) is an operator local-model-tree diagnostic which returns early if `C:\models` is absent and otherwise prints relationships without assertions. Do not count it as a robust acceptance test even when invoked with `--ignored`. [health.rs:1498+](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/health.rs#L1498) is a costly opt-in actual approved-runtime/hardware probe with environment prerequisites; ignoring this by default is appropriate, but requires a separately run hardware qualification job.
- Node release suite has 80 tests; 57 blocks call `readFile`. Some legitimately validate schemas/configs/artifact bytes. Many assert source regexes rather than behavior, especially frontend rules and the new SQLite features. Avoid describing all 80 as integration tests. [scripts/tests/release-gates.test.mjs:550–588](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/tests/release-gates.test.mjs#L550-L588) checks loading/cancellation race-related source strings, while `:1068–1087` checks SQLite symbols. A no-op helper retaining these names can pass these checks.
- Gate unit tests that call verifier functions on bad fixtures are stronger: e.g. qualification invalid support and mismatched attestation, research hash/review binding, checksum mismatch, and false-green/false-red schema records. Keep these tests; replace policy implementation-string tests where an observable contract exists.
- No enforced line/branch coverage or mutation-test command is configured. The absence of such a metric does not establish zero coverage. Priority is targeted regression coverage for the confirmed defects, then report module-level coverage to guide the next tests.
- [package.json:9,19](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/package.json#L9) runs release-gate tests in `npm test`; CI `:55–59` runs that test file explicitly and then via `npm run check` again. This redundant execution is minor CI latency/maintenance debt, not a correctness defect. Document the purpose or remove the duplicated invocation while preserving fail-fast gates.

### Maintainability assessment

The boundaries between Rust truth, React presentation, process construction, download verification, typed evidence and model metadata are coherent and appropriate for a control plane. The suite and exact manifests show intentional attention to integrity. The next constraint is orchestration complexity.

- `src/App.tsx`: 2136 lines, approximately 73 `useState` calls (74 lexical occurrences including import), 36 named `function` declarations; one top-level component owns inventory, runtime, downloads, tuning, launch profiles, settings and multiple async flows. UI regressions increasingly cross state domains and are poorly exercised by pure tests.
- `src/V03EvidencePanel.tsx`: 863 lines, around 22 state-hook calls, 23 function declarations. Separate component exists but still mixes data acquisition and presentation across several tools.
- `src/model.ts`: 1730 lines, broad manually mirrored IPC schemas and decisions. TypeScript strict/no-unused settings are enabled ([tsconfig.json:18–21](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/tsconfig.json#L18-L21)), useful baseline quality. There is no generated Rust↔TypeScript schema contract: a rename may compile on both sides yet fail at runtime unless an IPC test covers it. The schema-v2 snake-case/camelCase regression history in [catalog/README.md:74–82](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/catalog/README.md#L74-L82) illustrates this risk.
- `src-tauri/src/lib.rs`: 4226 lines with about 3200 production lines before test module. It combines application state, job orchestration, launch/workload lifecycles, catalog/filter IPC and persistence boundaries. `runtime.rs` is 7069 lines, with tests beginning around 3990; length includes valuable tests and is not itself proof of bad design.
- Refactor by ownership and behavior: catalog repository/cache service; supervised-job lifecycle; runtime command/installer facade; hooks for independent frontend operations; presentational screens receiving typed state and events. Establish tests at the current boundary before moving logic. Avoid a broad rewrite while correctness bugs remain.
- Prefer typed request/result contracts generated from Rust or schema-validated representative IPC fixtures; keep the Rust side authoritative. Separate saved-state versioning/validation from rendering.
- Consider bounded event/job state models to reduce unrelated booleans and sequence-guard repetition. Verify start/cancel/finish/rerun transitions rather than snapshotting implementation order.

### Dependency and supply-chain inventory

#### JavaScript

`package-lock.json` lockfile inventory contains **168 package records excluding the root**, of which 161 are marked `dev` and 7 are non-dev. All 168 have SHA-512 integrity and `https://registry.npmjs.org/` resolutions. No git/URL/path package dependency source appears. npm's actual installation on this platform can select fewer optional platform binaries than this cross-platform lockfile inventory.

| Direct package | Declared range | Locked version | Role |
|---|---|---|---|
| @tauri-apps/api | ^2 | 2.11.1 | IPC/runtime client |
| @tauri-apps/plugin-dialog | ^2.7.3 | 2.7.3 | Native picker client |
| @tauri-apps/plugin-opener | ^2 | 2.5.5 | Opener client |
| lucide-react | ^1.38.0 | 1.38.0 | Icons |
| react | ^19.1.0 | 19.2.8 | UI runtime |
| react-dom | ^19.1.0 | 19.2.8 | UI DOM renderer |
| @tauri-apps/cli | ^2 | 2.11.4 | Build tooling |
| @types/react | ^19.1.8 | 19.2.18 | Types |
| @types/react-dom | ^19.1.6 | 19.2.5 | Types |
| @vitejs/plugin-react | ^4.6.0 | 4.7.0 | Build plugin |
| ajv | 8.18.0 | 8.18.0 | Gate schemas |
| typescript | ~5.8.3 | 5.8.3 | Type checker |
| vite | ^7.0.4 | 7.3.6 | Bundler |
| vitest | ^4.1.11 | 4.1.11 | Tests |
| ws | 8.21.3 | 8.21.3 | CDP verifier |
| yaml | 2.9.0 | 2.9.0 | Workflow validation |

Lock metadata license counts: 139 MIT, 13 Apache-2.0 OR MIT, 2 MIT OR Apache-2.0, 3 Apache-2.0, 8 ISC, 2 BSD-3-Clause, 1 CC-BY-4.0. The CC-BY record is `caniuse-lite 1.0.30001810` (build data), not evidence of a nonpermissive license being embedded in the shipped app. Package metadata is an inventory input, not a legal compliance determination or a verified copy of all distribution notices.

#### Rust

`src-tauri/Cargo.lock` contains **566 package records including Localmotive**, all external sources on crates.io with checksums. No git dependency source found. Current root direct resolution:

| Dependency | Locked version | Important configuration |
|---|---|---|
| tauri | 2.11.5 | Desktop shell |
| tauri-build | 2.6.3 | Major 2 declared |
| tauri-plugin-dialog | 2.7.3 | Native picker |
| tauri-plugin-opener | 2.5.5 | External opener |
| serde | 1.0.229 | derive |
| serde_json | 1.0.151 | JSON |
| reqwest (direct) | 0.12.28 | default features off; blocking/json/rustls/gzip/brotli/deflate/zstd |
| zip | 4.6.1 | default features off; deflate |
| sha2 | 0.10.9 | SHA-256 |
| hex | 0.4.3 | Digests |
| keyring | 3.6.3 | windows-native |
| base64 (direct) | 0.22.1 | Encoding |
| rand (direct) | 0.8.8 | Randomness |
| cap-std | 4.0.3 | Capability filesystem APIs |
| ed25519-dalek | 2.2.0 | std signature validation |
| windows-sys (direct) | 0.61.2 | Win32 filesystem/security/process/network features |
| rusqlite | 0.40.2 | Bundled SQLite |
| windows (Windows only) | 0.62.2 | Exact pin; DXGI/Threading |
| windows-version (Windows only) | 0.1.7 | Exact pin |
| process-wrap (Windows only) | 10.0.0 | Exact pin; std/job-object/creation-flags |

There are 44 crate names with multiple locked versions. Relevant examples: reqwest 0.12.28 and 0.13.4; base64 0.21.7 and 0.22.1; rand 0.8.8 and 0.10.2; windows 0.61.3 and 0.62.2; windows-sys 0.45/0.52/0.59/0.60/0.61; thiserror 1.0.69 and 2.0.20; syn 1.0.109/2.0.119/3.0.4; multiple TOML/schema/indexmap/hashbrown generations. Many Windows target-package duplicates derive from upstream dependencies. Do not force incompatible major unification. Use `cargo tree --target x86_64-pc-windows-msvc -d` and release binary sizing to identify actual cost before changing anything.

Cargo.lock does not contain dependency license text/metadata, so a complete Rust license inventory needs `cargo metadata` with resolved package sources or generated SBOM/license tooling in the build environment. Do not infer every transitive license from Localmotive's own MIT declaration.

#### Controls present and recommended gaps

**Present:** committed npm/Cargo lockfiles; registry digests; `npm ci`; locked Rust CI commands; SHA-pinned GitHub Actions verified against committed policy; npm moderate-and-higher advisory gate; separate RustSec action; signed catalog document and compiled approved runtime hashes/content manifests; candidate checksum/inventory; source revision recorded in packaged evidence. These are material supply-chain strengths.

**Not found in supplied tracked files:** SECURITY.md/private disclosure instructions, Dependabot/Renovate configuration, cargo-deny/license policy, repository-generated SBOM/third-party NOTICE bundle, Rust toolchain pin, or automated component/coverage policy. Absence of a configuration file does not prove GitHub account-level features are disabled; available settings are discussed in the delivery section.

**Recommendations:** add a short SECURITY.md directing private vulnerability reporting; review dependency updates on a cadence and run advisory checks for shipped versions, not only new pushes; produce target-specific SBOM and notices alongside release artifacts; pin/record build toolchain and base OS image where reproducibility matters; retain action-pin updates with provenance; test release/install flows with least-privileged isolated runners. Do not equate a SHA256SUMS file fetched from the same unauthenticated release channel with independent publisher identity; current unsigned status is correctly disclosed and is an owner tradeoff, not a falsely claimed signature.

### Suggested verification additions, in order

1. Failing regression cases for current catalog startup/cooldown and mirror/override data integrity defects.
2. Builder clock/metadata/discovery tests reproducing QD-01 without any network or repository writes.
3. Component tests for real load/refresh/search/cancel transitions with delayed/rejected IPC responses.
4. New version-aware packaged HF catalog contract covering first run, restart, offline fallback, corrupt mirror, persistence and filtering, as well as actual IPC serialization.
5. Simplify private React-hook harness coupling and explicitly separate synthetic presentation evidence from real network/process evidence.
6. Current documentation/evidence index and supported toolchain declaration; archive stale qualification snapshot; source-pin upstream reference.
7. Target-specific dependency/license/SBOM inventory and periodic advisories; coverage reporting after the known correctness gaps are closed.


## Remediation plan and release acceptance

The following plan groups related findings into work that can be reviewed and verified. It is an engineering recommendation, not an estimate that all fixes take the same effort or that one person can implement them simultaneously. Keep each behavioral change paired with a failing regression, as the repository already requires. Close a finding only against a specified commit and observed test result.

| Order | Work package | Findings addressed | Exit condition |
|---|---|---|---|
| 1 | Unify managed execution authorization | RT-02, RT-04, RT-07 | Every EXE/CLI/bench probe and launch rejects tampered managed EXE/DLL content before execution; production-path tests exercise the actual guard |
| 2 | Restore catalog loading across restart/offline/cooldown | DC-01, DC-03, DC-07 | Valid local/bundled data loads in a fresh app state without network; refresh throttling never blocks read access; corrupt DB/cache recovery is explicit |
| 3 | Repair override ownership and persistence | DC-04–DC-06 | Curated and user identities cannot overwrite one another accidentally; edits/removal are transactional; browse/filter/download agree on provenance |
| 4 | Bound AI work and stop it reliably | MT-03, MT-04, RT-03 | Hard advisor-call/time/trial budgets; no further paid calls after cancellation; active requests/children stop within a documented deadline |
| 5 | Restore default benchmark correctness and public export privacy | MT-01, MT-02, MT-12 | Default warm workload works on b10816; full serialized export contains no redaction canaries; failures retain valid evidence |
| 6 | Preserve actual model/runtime/run identity | FE-01–FE-03, FE-05, FE-07, FE-16, MT-05 | UI draft, selected asset, running server and evidence records cannot silently substitute for one another; one backend operation owner; navigation preserves results/cancel control |
| 7 | Fix high-cost blocking paths and parser/download limits | IPC-01, FE-04, RT-05, RT-06, DC-02, DC-08, DC-12 | Responsive startup/preview; bounded/coalesced hash/probe work; >8 MiB no-range transfer works; parser budgets bound allocations and work |
| 8 | Repair release sequencing and verifier truthfulness | GH-01–GH-06, GH-10, QD-02, QD-03 | Safe premerge checks; protected immutable source/tag identity; candidate-based lifecycle; failed uninstall cannot PASS; every failure yields evidence |
| 9 | Repair legacy upgrades and secured local transport | RT-01, MT-06 | New installs always produce a launchable primary-root runtime; accepted TLS/auth profiles work with internal clients or are explicitly rejected before launch |
| 10 | Make evidence comparison and calibration defensible | MT-07–MT-11, MT-13–MT-15, FE-06, FE-17 | Full content/configuration identity; unique source runs for anchors; no cyclic dominance; explicit quality/workload scope and consistent validators |
| 11 | Harden recovery, privacy, and accessibility | IPC-02, CLD-01, OPS-01, FE-08–FE-09, FE-11–FE-15, DC-11, RT-08–RT-09 | Configured CSP; bounded callback/logs; recoverable saved-state corruption; durable transfer IDs; tested focus/names/contrast; conditional Windows races covered |
| 12 | Correct curation metadata and engineering documentation | DC-09–DC-10, QD-01, QD-04–QD-06, GH-07–GH-09 | Rolling/as-of date correct; immutable HF revisions; accurate quant labels; current evidence/storage/support docs; pinned/recorded toolchain and dependency follow-up |

Orders 1–8 deserve the earliest stabilization attention. Small independent fixes, such as the no-op budget, quant parser, typed runtime activation, or invalid PASS assertion, can land while the larger operation-state work is designed. Avoid coupling all findings into one enormous refactor or one release PR that reviewers cannot isolate.

### Concrete minimum regression pack

1. **Catalog restart:** start with a signed cached catalog and a fresh 26-hour stamp; create a new app state; load and browse successfully without network.
2. **Mirror preservation:** save local user rows; ordinary signed refresh preserves them; confirmed corruption quarantines/rebuilds predictably and reports any unrecoverable user data.
3. **Override collisions:** curated ID collision and case-insensitive filename collision cannot transfer or relabel file ownership; edit removes omitted files atomically.
4. **No-range download:** a legitimate HTTP 200 full response larger than 8 MiB completes and hashes correctly; interruption restarts safely.
5. **Managed probe trust:** replace each callable managed executable or DLL with an inert sentinel fixture; all public probes/tuning/health routes reject before the sentinel runs.
6. **Legacy runtime upgrade:** legacy directory exists and primary is absent; install returns a path that passes the same launch trust gate.
7. **Warm benchmark protocol:** cache_n plus prompt_n reflect the intended prompt workload; one warmup and five trials succeed against approved b10816 with counts retained.
8. **Tuning budget:** an advisor that repeats a value already in the baseline terminates after bounded attempts and makes no post-cancel calls.
9. **Operation ownership:** barriers force stop/start/quality/benchmark/tuning interleavings; evidence cannot be attributed to a replacement process or concurrent private cold-run server.
10. **Identity/navigation:** rescan/typed runtime changes/adopt tuning report preserve intended identity; leaving Benchmark and returning does not lose the run or cancel handle.
11. **Export redaction:** mixed successful/failed results with canary paths/secrets/notes serialize to a public bundle without those canaries, while retaining honest failure counts.
12. **Calibration uniqueness:** three clicks on one run remain one anchor; three valid independent runs are accepted; changed effective configuration or model content invalidates incompatible reuse.
13. **Release negative controls:** leftover MSI executable, unchanged upgrade version, wrong source SHA, missing candidate artifact, failed lifecycle, or absent evidence cannot yield a misleading PASS.
14. **Accessibility/recovery:** all interactive paths have visible keyboard focus and unique names; malformed persisted JSON and failed writes preserve a usable UI and completed results.

Run these in layers. Pure functions/SQL/protocol fixtures establish deterministic behavior cheaply. Component tests prove actual event/IPC sequencing. Packaged Windows scenarios prove Tauri scheduling, WebView2, NTFS, Job Objects, Credential Manager, installers, and runtime behavior. Passing one layer does not substitute for the others.

### Suggested exit criteria for a stabilization release

- Every chosen release-blocking finding has a regression that failed before the fix and passes afterward, including real integration scenarios rather than only source-text matches.
- TypeScript/tests/frontend build, Rust formatting/Clippy/tests, catalog signatures/schema, approval manifests, action pins and release gates all pass at the **same immutable source SHA**.
- Candidate portable and installer assets are built once, checksummed, and passed to packaged/lifecycle tests. Publication promotes those exact bytes and preserves the original candidate evidence.
- The packaged report explicitly distinguishes runtime UI fixture rendering, CPU inference, each accelerator tested, clean-account installation, upgrade and uninstall. Missing hardware is labeled untested and does not receive a supported label.
- A clean-account v0.4.1→new-version upgrade preserves profiles; a separate v0.5.0→new-version upgrade preserves the SQLite/user-override data introduced in v0.5.0. Both supported installer types receive appropriate tests.
- Cancellation timing and child/listener cleanup are measured in the packaged app during active work, not only between stages.
- Public release documentation states known remaining findings and accepted limitations, including unsigned distribution if that policy continues.

### What to measure after correctness is restored

| Question | Measurement to collect | Why it matters |
|---|---|---|
| Does UI work stay responsive? | Main-thread stalls and input/frame timing during typing, scanning, hashing and startup | Source analysis establishes expensive work; elapsed impact needs real WebView2 measurement |
| Is runtime verification proportional? | Bytes hashed, duplicate jobs, cold/warm duration, cancellation latency per action | Root lookup/listing should not unexpectedly rescan every installed backend |
| Does the downloader scale safely? | Throughput/CPU/disk queue at 1/4/8 connections; resume loss after injected failures | More connections do not guarantee faster serialized disk writes |
| Are measurements repeatable? | Independent run distributions, baseline drift, failure rate, sample count | A five-trial maximum/p95 and one noisy mean are weak signals for selecting a winner |
| Is calibration useful? | Held-out prediction error and interval coverage by model/runtime/hardware/workload class | A fitted multiplier with three entries does not itself establish predictive accuracy |
| Is the catalog usable at its current size? | Search/filter latency, IPC payload size, render time, missing metadata rate | Optimize measured bottlenecks before adding pagination/virtualization abstractions |
| Are releases stable to operate? | Queue time, required-check availability, gate failure categories, evidence retention | Historical retries and cancellations show friction but are not a production availability metric |

### Remaining verification requiring the target environment

This audit cannot honestly certify Windows packaged behavior from Linux source inspection. The principal outstanding target checks are: actual tamper rejection through every probe route; NTFS rename/open-handle races; runtime legacy migration; in-flight cancellation and Job Object cleanup; authenticated/TLS llama-server interaction; real default warm benchmarks; SQLite deletion/locking/recovery under WebView2; clean-account install/upgrade/uninstall; GPU identity and memory evidence on multiple identical adapters; Narrator/NVDA and high-DPI/high-contrast behavior; and actual cloud-provider login/model/request compatibility with authorized credentials.

The missing ignored `research/measuring` materials, private repository security settings/alerts, owner runner configuration, full transitive Rust license texts, and historical secrets were not available or not independently evaluated here. Their absence is a scope limit, not proof of a defect. The report's confirmed source and recorded-evidence findings remain actionable without those inputs.

## Evidence bundle guide

The companion evidence archive contains the explicit successful local check log, npm audit JSON, GitHub metadata/run/job/release snapshots, selected remote log excerpts, the packaged verification record extracted from the release job log, the controlled-date catalog-builder reproduction and its output, the exact-SQL reproduction and its output, a repository file inventory, a machine-readable findings register, and SHA-256 checksums for the bundle contents. It intentionally excludes credentials, downloaded model/runtime payloads, installed dependency trees, and a duplicate repository checkout.

To reproduce source-dependent checks, use the pinned commit above. The reproduction scripts use synthetic test values; those names and dates are fixtures, not additions to the real model catalog. The recorded Windows logs are historical observations. Running the report's proposed Windows acceptance scenarios is a separate verification step after fixes, not something this audit claims to have completed.


## Appendix: repository file inventory

This inventory establishes inclusion in the repository scope. “Source review” does not mean every line was dynamically executed; fixture and binary inventory does not imply visual or hardware testing. Native payloads referenced by manifests are external and were not downloaded.

| Tracked path | Bytes | Audit treatment |
|---|---:|---|
| [.gitattributes](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.gitattributes) | 755 | Project configuration, dependency, license or documentation review |
| [.github/workflow-actions.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflow-actions.json) | 2,041 | Workflow/gate source review plus live GitHub evidence |
| [.github/workflow-gates.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflow-gates.json) | 2,864 | Workflow/gate source review plus live GitHub evidence |
| [.github/workflows/catalog.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/catalog.yml) | 2,822 | Workflow/gate source review plus live GitHub evidence |
| [.github/workflows/ci.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/ci.yml) | 4,886 | Workflow/gate source review plus live GitHub evidence |
| [.github/workflows/hardware-qualify.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/hardware-qualify.yml) | 7,570 | Workflow/gate source review plus live GitHub evidence |
| [.github/workflows/release.yml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.github/workflows/release.yml) | 17,329 | Workflow/gate source review plus live GitHub evidence |
| [.gitignore](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.gitignore) | 412 | Project configuration, dependency, license or documentation review |
| [.impeccable/config.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.impeccable/config.json) | 26 | Design/configuration inventory and branding/icon gate context |
| [.impeccable/design.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/.impeccable/design.json) | 30,940 | Design/configuration inventory and branding/icon gate context |
| [AGENTS.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/AGENTS.md) | 16,168 | Project configuration, dependency, license or documentation review |
| [CHANGELOG.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/CHANGELOG.md) | 25,241 | Project configuration, dependency, license or documentation review |
| [LICENSE](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/LICENSE) | 1,081 | Project configuration, dependency, license or documentation review |
| [README.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/README.md) | 18,252 | Project configuration, dependency, license or documentation review |
| [assets/app-icon.svg](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/assets/app-icon.svg) | 495 | Design/configuration inventory and branding/icon gate context |
| [catalog/README.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/catalog/README.md) | 6,281 | Catalog/schema/builder/signature policy and inventory review |
| [catalog/catalog.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/catalog/catalog.json) | 630,182 | Catalog/schema/builder/signature policy and inventory review |
| [catalog/catalog.json.sig](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/catalog/catalog.json.sig) | 89 | Catalog/schema/builder/signature policy and inventory review |
| [catalog/providers.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/catalog/providers.json) | 630 | Catalog/schema/builder/signature policy and inventory review |
| [docs/DESIGN.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/DESIGN.md) | 33,423 | Documentation/design/reference review; no full visual design certification |
| [docs/Future_branding.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/Future_branding.md) | 5,230 | Documentation/design/reference review; no full visual design certification |
| [docs/LLAMA-SERVER-README.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/LLAMA-SERVER-README.md) | 106,546 | Documentation/design/reference review; no full visual design certification |
| [docs/OPTION_MAP.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/OPTION_MAP.md) | 3,406 | Documentation/design/reference review; no full visual design certification |
| [docs/PRODUCT.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/PRODUCT.md) | 4,071 | Documentation/design/reference review; no full visual design certification |
| [docs/RUNTIME_MANAGER.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/RUNTIME_MANAGER.md) | 4,622 | Documentation/design/reference review; no full visual design certification |
| [docs/history/TODO-0.4.1.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/history/TODO-0.4.1.md) | 109,207 | Historical context and evidence reconciliation; not accepted as standalone proof |
| [docs/history/TODO-0.4.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/history/TODO-0.4.md) | 44,835 | Historical context and evidence reconciliation; not accepted as standalone proof |
| [docs/history/TODO-0.5.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/history/TODO-0.5.md) | 38,522 | Historical context and evidence reconciliation; not accepted as standalone proof |
| [docs/history/TODO.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/history/TODO.md) | 14,747 | Historical context and evidence reconciliation; not accepted as standalone proof |
| [docs/qualification-tests.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/qualification-tests.md) | 15,815 | Documentation/design/reference review; no full visual design certification |
| [docs/theme.css](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/theme.css) | 3,760 | Documentation/design/reference review; no full visual design certification |
| [docs/tokens.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/docs/tokens.json) | 11,378 | Documentation/design/reference review; no full visual design certification |
| [index.html](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/index.html) | 1,045 | Project configuration, dependency, license or documentation review |
| [package-lock.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/package-lock.json) | 87,103 | Project configuration, dependency, license or documentation review |
| [package.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/package.json) | 1,565 | Project configuration, dependency, license or documentation review |
| [release-evidence/0.4.1/approvals.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/release-evidence/0.4.1/approvals.json) | 6,612 | Schema/evidence consistency and qualification-scope review |
| [release-evidence/0.4.1/attestations/clean-account-lifecycle-0.4.1.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/release-evidence/0.4.1/attestations/clean-account-lifecycle-0.4.1.json) | 1,787 | Schema/evidence consistency and qualification-scope review |
| [release-evidence/0.4.1/attestations/sandbox-clean-account-lifecycle.log](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/release-evidence/0.4.1/attestations/sandbox-clean-account-lifecycle.log) | 1,570 | Schema/evidence consistency and qualification-scope review |
| [release-evidence/0.4.1/catalog-signing.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/release-evidence/0.4.1/catalog-signing.json) | 1,304 | Schema/evidence consistency and qualification-scope review |
| [release-evidence/0.4.1/phase0a-independent-review.txt](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/release-evidence/0.4.1/phase0a-independent-review.txt) | 1,015 | Schema/evidence consistency and qualification-scope review |
| [release-evidence/0.4.1/qualification-matrix.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/release-evidence/0.4.1/qualification-matrix.json) | 8,382 | Schema/evidence consistency and qualification-scope review |
| [release-evidence/0.4.1/research-freeze-anchor.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/release-evidence/0.4.1/research-freeze-anchor.json) | 825 | Schema/evidence consistency and qualification-scope review |
| [release-evidence/0.4.1/research-freeze-manifest.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/release-evidence/0.4.1/research-freeze-manifest.json) | 768,512 | Schema/evidence consistency and qualification-scope review |
| [release-evidence/0.4.1/research-verification.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/release-evidence/0.4.1/research-verification.json) | 1,934 | Schema/evidence consistency and qualification-scope review |
| [release-evidence/0.4.1/v0.4.0-corrective-note.md](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/release-evidence/0.4.1/v0.4.0-corrective-note.md) | 1,284 | Schema/evidence consistency and qualification-scope review |
| [release-evidence/packaged-verification.schema.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/release-evidence/packaged-verification.schema.json) | 3,335 | Schema/evidence consistency and qualification-scope review |
| [release-evidence/qualification-attestation.schema.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/release-evidence/qualification-attestation.schema.json) | 2,981 | Schema/evidence consistency and qualification-scope review |
| [release-evidence/qualification-matrix.schema.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/release-evidence/qualification-matrix.schema.json) | 1,816 | Schema/evidence consistency and qualification-scope review |
| [scripts/build_catalog.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/build_catalog.mjs) | 9,365 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/capture_022.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/capture_022.mjs) | 6,935 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/drive_console_check.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/drive_console_check.mjs) | 4,598 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/dryrun_catalog.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/dryrun_catalog.mjs) | 3,102 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/live_verify_022.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/live_verify_022.mjs) | 9,262 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/negative_control_console.py](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/negative_control_console.py) | 1,381 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/sandbox/host-run-lifecycle.ps1](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/sandbox/host-run-lifecycle.ps1) | 6,151 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/sandbox/run-lifecycle-in-sandbox.ps1](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/sandbox/run-lifecycle-in-sandbox.ps1) | 6,103 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/sign-windows.ps1](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/sign-windows.ps1) | 2,061 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/sign_catalog_candidate.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/sign_catalog_candidate.mjs) | 878 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/tests/release-gates.test.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/tests/release-gates.test.mjs) | 51,181 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/validate_catalog.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/validate_catalog.mjs) | 4,322 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/verify_024.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_024.mjs) | 7,784 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/verify_025.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_025.mjs) | 6,926 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/verify_030.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_030.mjs) | 5,761 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/verify_040.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_040.mjs) | 9,078 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/verify_041.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_041.mjs) | 46,820 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/verify_branding.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_branding.mjs) | 3,432 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/verify_candidate_inventory.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_candidate_inventory.mjs) | 3,047 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/verify_icons.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_icons.mjs) | 3,062 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/verify_qualification.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_qualification.mjs) | 9,911 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/verify_research_anchor.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_research_anchor.mjs) | 11,423 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/verify_versions.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_versions.mjs) | 2,622 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/verify_workflow_gates.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_workflow_gates.mjs) | 4,738 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/verify_workflow_pins.mjs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/verify_workflow_pins.mjs) | 3,320 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [scripts/watch_console_windows.py](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/scripts/watch_console_windows.py) | 2,131 | Script/gate/harness review; local check gates or targeted reproduction where stated |
| [src-tauri/.cargo/config.toml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/.cargo/config.toml) | 55 | Project configuration, dependency, license or documentation review |
| [src-tauri/.gitignore](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/.gitignore) | 166 | Project configuration, dependency, license or documentation review |
| [src-tauri/Cargo.lock](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/Cargo.lock) | 146,681 | Project configuration, dependency, license or documentation review |
| [src-tauri/Cargo.toml](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/Cargo.toml) | 1,985 | Project configuration, dependency, license or documentation review |
| [src-tauri/approved_runtimes.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/approved_runtimes.json) | 12,106 | Runtime approval/manifest structural and hash-anchor checks |
| [src-tauri/build.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/build.rs) | 39 | Project configuration, dependency, license or documentation review |
| [src-tauri/capabilities/default.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/capabilities/default.json) | 245 | Project configuration, dependency, license or documentation review |
| [src-tauri/icons/128x128.png](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/128x128.png) | 999 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/icons/128x128@2x.png](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/128x128@2x.png) | 1,562 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/icons/32x32.png](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/32x32.png) | 491 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/icons/64x64.png](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/64x64.png) | 657 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/icons/Square107x107Logo.png](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/Square107x107Logo.png) | 832 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/icons/Square142x142Logo.png](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/Square142x142Logo.png) | 1,120 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/icons/Square150x150Logo.png](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/Square150x150Logo.png) | 1,205 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/icons/Square284x284Logo.png](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/Square284x284Logo.png) | 1,877 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/icons/Square30x30Logo.png](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/Square30x30Logo.png) | 427 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/icons/Square310x310Logo.png](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/Square310x310Logo.png) | 1,993 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/icons/Square44x44Logo.png](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/Square44x44Logo.png) | 544 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/icons/Square71x71Logo.png](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/Square71x71Logo.png) | 708 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/icons/Square89x89Logo.png](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/Square89x89Logo.png) | 800 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/icons/StoreLogo.png](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/StoreLogo.png) | 579 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/icons/icon.icns](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/icon.icns) | 24,266 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/icons/icon.ico](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/icon.ico) | 5,376 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/icons/icon.png](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/icons/icon.png) | 3,269 | Binary inventory; icon verifier passed; no independent visual icon QA |
| [src-tauri/runtime-content-manifests/cpu.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/runtime-content-manifests/cpu.json) | 8,298 | Runtime approval/manifest structural and hash-anchor checks |
| [src-tauri/runtime-content-manifests/cuda-12.4.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/runtime-content-manifests/cuda-12.4.json) | 9,107 | Runtime approval/manifest structural and hash-anchor checks |
| [src-tauri/runtime-content-manifests/cuda-13.3.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/runtime-content-manifests/cuda-13.3.json) | 9,106 | Runtime approval/manifest structural and hash-anchor checks |
| [src-tauri/runtime-content-manifests/openvino.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/runtime-content-manifests/openvino.json) | 12,976 | Runtime approval/manifest structural and hash-anchor checks |
| [src-tauri/runtime-content-manifests/rocm.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/runtime-content-manifests/rocm.json) | 8,916 | Runtime approval/manifest structural and hash-anchor checks |
| [src-tauri/runtime-content-manifests/sycl.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/runtime-content-manifests/sycl.json) | 11,529 | Runtime approval/manifest structural and hash-anchor checks |
| [src-tauri/runtime-content-manifests/vulkan.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/runtime-content-manifests/vulkan.json) | 8,461 | Runtime approval/manifest structural and hash-anchor checks |
| [src-tauri/src/artifact.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/artifact.rs) | 26,539 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/calibration.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/calibration.rs) | 28,181 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/catalog.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog.rs) | 79,781 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/catalog_db.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/catalog_db.rs) | 26,857 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/cloud.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/cloud.rs) | 28,635 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/core.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/core.rs) | 108,462 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/download.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/download.rs) | 91,937 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/evidence.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/evidence.rs) | 37,464 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/gguf.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/gguf.rs) | 20,396 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/health.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/health.rs) | 59,909 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/lib.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/lib.rs) | 149,834 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/main.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/main.rs) | 186 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/measurement.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/measurement.rs) | 51,955 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/preflight.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/preflight.rs) | 39,588 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/proc.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/proc.rs) | 19,128 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/recommend.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/recommend.rs) | 20,299 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/runtime.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/runtime.rs) | 270,079 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/sharing.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/sharing.rs) | 25,257 | Rust source and call-path review; related tests inspected |
| [src-tauri/src/tune.rs](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/src/tune.rs) | 39,222 | Rust source and call-path review; related tests inspected |
| [src-tauri/tauri.conf.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tauri.conf.json) | 1,016 | Project configuration, dependency, license or documentation review |
| [src-tauri/tauri.signing.conf.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tauri.signing.conf.json) | 189 | Project configuration, dependency, license or documentation review |
| [src-tauri/tests/fixtures/health/backend-ops.txt](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tests/fixtures/health/backend-ops.txt) | 58 | Fixture/schema inventory and relevant test context |
| [src-tauri/tests/fixtures/health/benchmark.jsonl](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tests/fixtures/health/benchmark.jsonl) | 94 | Fixture/schema inventory and relevant test context |
| [src-tauri/tests/fixtures/health/completion.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tests/fixtures/health/completion.json) | 115 | Fixture/schema inventory and relevant test context |
| [src-tauri/tests/fixtures/health/cpu.list-devices.stderr.txt](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tests/fixtures/health/cpu.list-devices.stderr.txt) | 0 | Fixture/schema inventory and relevant test context |
| [src-tauri/tests/fixtures/health/cpu.list-devices.stdout.txt](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tests/fixtures/health/cpu.list-devices.stdout.txt) | 28 | Fixture/schema inventory and relevant test context |
| [src-tauri/tests/fixtures/health/cuda.list-devices.stderr.txt](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tests/fixtures/health/cuda.list-devices.stderr.txt) | 0 | Fixture/schema inventory and relevant test context |
| [src-tauri/tests/fixtures/health/cuda.list-devices.stdout.txt](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tests/fixtures/health/cuda.list-devices.stdout.txt) | 80 | Fixture/schema inventory and relevant test context |
| [src-tauri/tests/fixtures/health/decision-records.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tests/fixtures/health/decision-records.json) | 1,439 | Fixture/schema inventory and relevant test context |
| [src-tauri/tests/fixtures/health/devices.txt](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tests/fixtures/health/devices.txt) | 73 | Fixture/schema inventory and relevant test context |
| [src-tauri/tests/fixtures/health/ready.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tests/fixtures/health/ready.json) | 16 | Fixture/schema inventory and relevant test context |
| [src-tauri/tests/fixtures/health/smoke-model.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tests/fixtures/health/smoke-model.json) | 570 | Fixture/schema inventory and relevant test context |
| [src-tauri/tests/fixtures/health/vulkan.list-devices.stderr.txt](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tests/fixtures/health/vulkan.list-devices.stderr.txt) | 0 | Fixture/schema inventory and relevant test context |
| [src-tauri/tests/fixtures/health/vulkan.list-devices.stdout.txt](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tests/fixtures/health/vulkan.list-devices.stdout.txt) | 82 | Fixture/schema inventory and relevant test context |
| [src-tauri/tests/fixtures/runtime/b10816-release.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tests/fixtures/runtime/b10816-release.json) | 9,231 | Fixture/schema inventory and relevant test context |
| [src-tauri/tests/fixtures/runtime/b10816-required-jobs.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tests/fixtures/runtime/b10816-required-jobs.json) | 7,840 | Fixture/schema inventory and relevant test context |
| [src-tauri/tests/fixtures/runtime/blocked-jobs.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src-tauri/tests/fixtures/runtime/blocked-jobs.json) | 746 | Fixture/schema inventory and relevant test context |
| [src/App.css](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.css) | 47,048 | Frontend source/state/UX review; model tests executed |
| [src/App.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/App.tsx) | 131,691 | Frontend source/state/UX review; model tests executed |
| [src/V03EvidencePanel.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/V03EvidencePanel.tsx) | 35,889 | Frontend source/state/UX review; model tests executed |
| [src/main.tsx](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/main.tsx) | 229 | Frontend source/state/UX review; model tests executed |
| [src/model.test.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.test.ts) | 28,535 | Frontend source/state/UX review; model tests executed |
| [src/model.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/model.ts) | 44,448 | Frontend source/state/UX review; model tests executed |
| [src/vite-env.d.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/src/vite-env.d.ts) | 38 | Frontend source/state/UX review; model tests executed |
| [tsconfig.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/tsconfig.json) | 605 | Project configuration, dependency, license or documentation review |
| [tsconfig.node.json](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/tsconfig.node.json) | 213 | Project configuration, dependency, license or documentation review |
| [vite.config.ts](https://github.com/sato942/localmotive/blob/e530371b056cd8e049c2246dbb151aa407bf359f/vite.config.ts) | 1,070 | Project configuration, dependency, license or documentation review |
