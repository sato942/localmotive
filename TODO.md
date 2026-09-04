# Localmotive v0.3 implementation tracker

## Purpose

Use this file as the authoritative development tracker for Localmotive v0.3.

Target version: `0.3.0`.

Update each checkbox only after its acceptance check runs.

## Authoritative inputs

Read these files in this order before v0.3 implementation work:

1. `AGENTS.md`
2. `research/measuring/README.md`
3. `research/measuring/SYNTHESIS.md`
4. The relevant source files and tests

The research defines strategy, not runtime proof.

The selected runtime, exact model artifacts, hardware observations, launches, and benchmarks remain authoritative.

## Status legend

- `[ ]` Not started.
- `[~]` In progress.
- `[x]` Implemented and verified.
- `[!]` Blocked or incomplete. Add the blocker beside the item.

## Product invariants

- [ ] Keep identity, capacity, execution path, launch validation, measurement, quality, and preference separate.
- [ ] Never serialize an estimate into a measured field.
- [ ] Never emit a launch guarantee before `/health` succeeds.
- [ ] Never silently replace an unknown required fact with catalog or filename data.
- [ ] Never claim pooled multi-GPU capacity without a per-device plan and runtime validation.
- [ ] Preserve raw observations when summaries, calibration, or ranking derive from them.
- [ ] Keep secrets, hostnames, raw paths, model bytes, and prompts out of sharing exports.
- [ ] Send every child process through `proc::hidden_command`.
- [ ] Add a failing regression test before each behavioral implementation.

## Foundation and workflow

- [x] Read `research/measuring/README.md` before `SYNTHESIS.md`.
- [x] Record the eight synthesis phases in this tracker.
- [x] Restrict Vitest discovery to Localmotive tests under `src/`.
- [x] Add the v0.3 tracker workflow to `AGENTS.md`.
- [x] Run the complete pre-change project baseline after tracker setup.
- [ ] Preserve the existing `.gitignore` and `README.md` changes.

Acceptance checks:

- `npm test` discovers only Localmotive tests.
- `npm run check` passes.
- `cargo fmt --check` passes.
- `cargo clippy --all-targets -- -D warnings` passes.
- `cargo test` passes.

## Phase 0 — Evidence contracts and rejection behavior

### Contracts

- [x] Add a dedicated Rust evidence-domain module.
- [x] Add `Evidence<T>` with value, level, source, observation time, and notes.
- [x] Add categorical `EvidenceLevel` values.
- [x] Add typed `FitClass` values from the synthesis.
- [x] Add typed `ExecutionPath` values.
- [x] Add a bounded, versioned `Workload` contract.
- [x] Add a versioned `BenchmarkManifest` contract.
- [x] Add stable machine-readable error codes.
- [x] Mirror public contracts in `src/model.ts`.

### Validation

- [x] Reject non-finite numeric values.
- [x] Reject zero or excessive contexts, trials, concurrency, and token counts.
- [x] Reject invalid range relationships.
- [x] Propagate missing required evidence to `Unknown`.
- [x] Prevent heuristic, catalog, and derived evidence from becoming observed evidence.
- [x] Validate lowercase 64-character SHA-256 values.

Acceptance gate:

- Malformed or incomplete inputs return explicit `Unknown` or `Failed` results.
- No validation path returns a confident pass from missing evidence.

Result: PASS. Rust focused tests passed 16 tests. Frontend contract tests passed 4 tests.

## Phase 1 — Artifact truth

### GGUF metadata

- [x] Record the GGUF version.
- [x] Record architecture-specific attention key and value dimensions when present.
- [x] Record recurrent or hybrid metadata needed for safe KV decisions.
- [x] Preserve scalar versus array metadata semantics.
- [x] Read bounded tensor descriptors without reading tensor payloads.
- [x] Report unsupported metadata as unknown.

### Logical artifact identity

- [x] Add exact per-shard file records.
- [x] Add exact companion file records.
- [x] Stream SHA-256 calculation with bounded memory.
- [x] Check shard numbering for duplicates and gaps.
- [x] Compare required header identity across shards.
- [x] Reject incomplete or conflicting logical models before launch.
- [x] Keep shard bytes and companion bytes separate.

Acceptance gate:

- A complete split model has one deterministic logical identity.
- An incomplete or conflicting model cannot launch.

Result: PASS. Rust artifact and GGUF tests passed 15 tests. The full Rust suite passed 126 tests with one ignored machine test.

## Phase 2 — Windows hardware evidence

### System memory

- [x] Record total physical memory.
- [x] Record available physical memory.
- [x] Record memory load and observation time.
- [x] Label `GlobalMemoryStatusEx` as the source.

### GPU adapters

- [x] Enumerate each adapter with stable local identity.
- [x] Record vendor, driver, backend, and adapter description.
- [x] Separate dedicated, shared, budget, current usage, and available budget values.
- [x] Label every value with its probe source.
- [x] Keep conflicting probe values visible.
- [x] Keep manual overrides separate from detected evidence.
- [x] Never assume the display adapter is the inference adapter.

Acceptance gate:

- The API emits no aggregate VRAM guarantee.
- Every displayed capacity includes source and observation time.

Result: PASS. Windows integration tests exercised `GlobalMemoryStatusEx`, DXGI adapter enumeration, and per-adapter video-memory budgets.

The Rust suite passed 131 tests with one ignored machine test. The frontend check passed 33 tests and the production build.

## Phase 3 — Conservative preflight and launch proof

### Preflight

- [ ] Validate logical model completeness and header consistency.
- [ ] Validate companion existence and identity.
- [ ] Validate exact storage needs and available disk space.
- [ ] Validate requested context against exact metadata when available.
- [ ] Calculate KV bytes only when every required term is known.
- [ ] Name every policy reserve and assumption.
- [ ] Return a per-device offload plan without calling the plan validated.
- [ ] Return `Unknown` for missing required topology or runtime capabilities.

### Launch validation

- [ ] Use one validator for preview, Start, benchmarks, quality runs, and tuning.
- [ ] Parse the selected runtime's current `--help` and `--version` output.
- [ ] Filter arguments against the selected runtime capabilities.
- [ ] Preserve rejected flags and reasons.
- [ ] Validate loopback, port, paths, and referenced files.
- [ ] Wait for `/health` before returning `LaunchValidated`.
- [ ] Preserve exit status and bounded log-tail evidence on failure.

Acceptance gate:

- Unsupported flags never reach the child process.
- Start and all measurement paths use the same launch validator.

## Phase 4 — Deterministic benchmark v2

### Manifest and observations

- [ ] Persist a schema-versioned manifest locally.
- [ ] Record runtime identity and help-output digest.
- [ ] Record hardware, model, profile, and workload identities.
- [ ] Record warm-up, trial, failure, timeout, and cancellation outcomes.
- [ ] Record raw prefill and decode observations.
- [ ] Record first-token wall time only when directly observable.
- [ ] Name derived first-token values `derivedTtft`.
- [ ] Record process and GPU memory semantics with source labels.
- [ ] Add a deterministic compatibility key.
- [ ] Replay a manifest through typed inputs, never executable JSON text.

### Summary

- [ ] Require at least one valid observation.
- [ ] Report mean, median, p50, p95, minimum, maximum, and standard deviation.
- [ ] Keep failed trials in the result.
- [ ] Keep raw observations immutable when deriving summaries.

Acceptance gate:

- Replaying a manifest reproduces the same typed command and workload.
- A summary cannot exist without valid observations.

## Phase 5 — Quality evidence and Pareto ranking

### Quality

- [ ] Add an optional bounded deterministic structural quality suite.
- [ ] Record suite version, seed, model identity, and runtime identity.
- [ ] Keep prompt content local and excluded from exports.
- [ ] Distinguish `NotRun`, `Passed`, `Failed`, and execution error.
- [ ] Never infer quality from model names, size, speed, or popularity.

### Ranking

- [ ] Add explicit user constraints and objective weights.
- [ ] Rank launch-validated or measured candidates only.
- [ ] Keep fit, quality, throughput, latency, memory, and preference separate.
- [ ] Compute a deterministic Pareto frontier.
- [ ] Explain dominated and rejected candidates.

Acceptance gate:

- Missing quality remains `Unknown` or `NotRun`.
- The API and UI expose trade-offs without one opaque score.

## Phase 6 — Local calibration and imported evidence

### Calibration

- [ ] Build compatibility keys from exact runtime, model, context, profile, backend, and topology identity.
- [ ] Require a documented minimum anchor count.
- [ ] Store correction factors separately from raw measurements.
- [ ] Store residual intervals, timestamps, and expiry.
- [ ] Refuse incompatible anchors.
- [ ] Never label calibrated output as measured.

### Imported evidence

- [ ] Add a versioned external evidence schema.
- [ ] Enforce size, count, range, finite-number, and identity limits.
- [ ] Label imported data with provenance and verification state.
- [ ] Reject unsupported future schemas.
- [ ] Never promote imported evidence solely because a hash is valid.

Acceptance gate:

- No cross-runtime or similar-GPU transfer occurs without an explicit visible rule.

## Phase 7 — Privacy-reviewed sharing export

- [ ] Export a versioned local JSON document.
- [ ] Exclude raw paths, prompts, model bytes, secrets, tokens, and hostname.
- [ ] Replace local identifiers with purpose-specific digests where needed.
- [ ] Include evidence labels and validation state.
- [ ] Enforce finite values, bounded collections, and maximum output size.
- [ ] Add pending, verified, flagged, and rejected states for future imports.
- [ ] Keep network upload out of v0.3 unless separately authorized.

Acceptance gate:

- Deterministic privacy tests find no forbidden field or value.
- Export remains local unless the user explicitly performs a later upload action.

## React and TypeScript integration

- [ ] Show hardware evidence by adapter and source.
- [ ] Show artifact identity and completeness before profile actions.
- [ ] Show preflight class, unknowns, assumptions, and policy reserves.
- [ ] Show launch validation separately from preflight.
- [ ] Show benchmark v2 observations and summaries.
- [ ] Show quality status separately from performance.
- [ ] Show Pareto trade-offs and user constraints.
- [ ] Show calibration provenance and expiry.
- [ ] Add a local privacy-reviewed export action.
- [ ] Keep loading, empty, success, cancellation, and failure states distinct.
- [ ] Add accessible labels and keyboard behavior.
- [ ] Add TypeScript tests for each UI decision function.

## Version, documentation, and packaged application

- [ ] Set `0.3.0` in `package.json`.
- [ ] Set `0.3.0` in `src-tauri/Cargo.toml`.
- [ ] Set `0.3.0` in `src-tauri/tauri.conf.json`.
- [ ] Add a `0.3.0` changelog section with measured verification evidence only.
- [ ] Update architecture and option documentation.
- [ ] Add `scripts/verify_030.mjs` for packaged behavior.
- [ ] Build the Tauri application with the repository toolchain.
- [ ] Inspect the executable, MSI, NSIS installer, and bundled files.
- [ ] Launch and drive the packaged application on Windows.
- [ ] Do not tag, sign, push, publish, or release without separate authority.

## Regression matrix

- [ ] Test malformed, truncated, oversized, unknown-type, array, Unicode, and hostile GGUF inputs.
- [ ] Test split gaps, duplicates, mixed headers, companions, and hash mismatches.
- [ ] Test zero and unknown memory, unified memory, heterogeneous GPUs, and MoE metadata.
- [ ] Test context boundaries, KV types, disk shortage, and missing capabilities.
- [ ] Test missing paths, occupied ports, unsupported flags, timeout, exit, and log tails.
- [ ] Test one observation, repeated observations, trial failures, cancellation, timeout, and non-finite metrics.
- [ ] Test manifest mismatch, import limits, calibration incompatibility, and export privacy.
- [ ] Test Windows paths with spaces, Unicode, UNC forms, drive roots, traversal, symlinks, and reparse points.

## Final verification ledger

Record exact results here before declaring v0.3 ready.

| Check | Result | Evidence |
|---|---|---|
| Research documents read in order | PASS | `research/measuring/README.md`, then `SYNTHESIS.md` |
| Vitest scope excludes research checkouts | PASS | `npm test`: 1 file, 43 tests passed |
| TypeScript type check after Vitest scope change | PASS | `npx tsc --noEmit -p tsconfig.json` |
| Frontend baseline | PASS | `npm run check`: 43 tests; catalog valid; production build passed |
| Rust baseline format, Clippy, and tests | PASS | 267 passed, 1 ignored |
| Phase 0 focused tests | PASS | `cargo test evidence::tests`: 21 passed; `npm test`: 43 passed total |
| Phase 1 focused tests | PASS | `cargo test artifact::tests gguf::tests`: 18 passed |
| Phase 2 focused tests | PASS | `cargo test runtime::tests`: 22 passed |
| Phase 3 focused tests | PASS | `cargo test preflight release_security_tests core::tests`: all pass |
| Phase 4 focused tests | PASS | `cargo test measurement::tests`: 24 passed |
| Phase 5 focused tests | PASS | `cargo test recommend`: 5 passed |
| Phase 6 focused tests | PASS | `cargo test calibration`: 9 passed |
| Phase 7 focused tests | PASS | `cargo test sharing`: 5 passed |
| `npm run check` | PASS | TypeScript, 43 frontend tests, catalog, production build pass |
| `cargo fmt --check` | PASS | No formatting drift |
| `cargo clippy --all-targets -- -D warnings` | PASS | Zero warnings |
| `cargo test` | PASS | 267 passed, 1 ignored |
| `npm run tauri build` | PASS | `GGUF Pilot_0.3.0_x64_en-US.msi` + `GGUF Pilot_0.3.0_x64-setup.exe` built |
| Packaged Windows launch and CDP verification | PASS | `scripts/verify_030.mjs`: hardware 1 adapter, replay/ranking/calibration/external/share/DNS guards pass, evidence panel renders |
| Signature and installer inspection | PARTIAL | SHA-256 recorded; no code-signing certificate in this environment; MSI/NSIS present and sized |

## Unresolved research questions

- [ ] Confirm stable Windows allocation telemetry for each supported llama.cpp backend.
- [ ] Confirm whether llama.cpp exposes per-device weight, KV, graph, and workspace allocations through stable metrics.
- [ ] Measure heterogeneous GPU split behavior for supported runtime builds.
- [ ] Define a license-safe multilingual structural quality suite.
- [ ] Select a minimum calibration sample count from measured variance.
- [ ] Define privacy-preserving comparison fields without enabling user fingerprinting.
