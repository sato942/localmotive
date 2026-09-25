# Localmotive Support Matrix

Claim only what the recorded evidence establishes (audit G-08.I2, GH-07).
Every row names the evidence class: **MEASURED** on this host against a
packaged candidate; **EXERCISED** through automated tests or scripted
fixtures; **REVIEWED** from upstream documentation; **UNTESTED** - no claim.

Machine-readable evidence lives in `release-evidence/<version>/`. Current findings are in `REVIEW.md`. The 0.6
per-finding records are in `docs/history/TODO-0.6.md` in git history (last
tree `9b09857`).

Out of 0.6 scope (owner 2026-09-21, not an open 0.6 gate): multi-vendor GPU
hosts, a second GPU adapter, HDD/SATA storage targets, and OS-crash /
power-loss facilities. Those environments stay UNTESTED below; 0.6 will not
obtain that hardware. A deferral is not substitute evidence that a missing
configuration works.

## Platform and lifecycle

| Claim | Status | Evidence |
|---|---|---|
| Windows 11 x64 (build 26100 observed) lifecycle: install, upgrade with user-data preservation, uninstall | MEASURED (0.6.5 published release) | Windows Sandbox clean-account leg PASS for the 0.6.5 release (clean install, upgrade from v0.5.0 with preservation, uninstall), bound to source revision and candidate digests (published `packaged-verification-0.6.5.json` + `candidate-inventory-0.6.5.json`). Earlier 0.6.x lines were never published. |
| Windows 10 | UNTESTED | No packaged lifecycle evidence; do not claim support. |
| macOS and Linux | Out of scope | No artifacts are published. |

## Inference backends

| Claim | Status | Evidence |
|---|---|---|
| CUDA inference on the RTX 5090 (driver 610.74) via the approved managed runtime b10816 | MEASURED (0.6 development line, unpublished) | Packaged health 7/7 PASS and decode benchmarks: warm means 999.00 / 995.11 tok/s; default workload 1002.60 tok/s (n=5, p95 1007.57); TLS profile 1004.76 tok/s. Model: SmolLM2-135M Q4_K_M. These figures are historical 0.6-development lab observations, not version-bound release evidence: the laboratory legs moved out of the release gate, and no 0.6.x release publishes CUDA inference figures. |
| Other NVIDIA GPUs, AMD GPUs, Intel GPUs | UNTESTED | Single-GPU host. The capability list comes from each runtime's own `--help`; hardware qualification does not follow from it. |
| Vulkan, OpenVINO, SYCL, CPU managed runtimes | UNTESTED | The catalog can offer these builds; no managed install was exercised. OpenVINO was refused on this NVIDIA host by adapter policy. |
| CPU inference through a verified legacy runtime | MEASURED (single observation) | One legacy CPU runtime was adopted by discovery and served health; no managed CPU install was exercised. |
| Identical-GPU row (same model, two adapters) | UNTESTED | Needs a two-GPU host. |
| Multi-GPU selection beyond the listed adapter | UNTESTED | Rows fall back per item. |

## Serving, network, and downloads

| Claim | Status | Evidence |
|---|---|---|
| TLS serving with a caller-provided certificate and API-key file; API key enforced on inference endpoints | MEASURED | Packaged run: `https://127.0.0.1:8080` reached LIVE, `/props` returns 401 without the key and 200 with it; benchmark over TLS recorded. |
| Managed-runtime tamper rejection (renamed executable, swapped DLL, byte-identical hard-link alias) before any spawn | MEASURED | Packaged refusals with `trust_failure` and no child process; marker never executed. |
| Resumable downloads with partial-byte retention across cancel | MEASURED | Packaged cancel kept the preallocated `.part` file and sidecar stable; resume continued from the recorded ranges (sidecar delta +7 607 420 bytes). |
| Redirect and proxy policy (HTTPS-only hosts, bounded hops) | EXERCISED | 12-case matrix plus refused-redirect fixtures in the automated suite. |
| HTTP validator semantics (strong ETag gating, bounded `Retry-After`) | EXERCISED | Fixture tests plus a stalled-fetch abort case. |
| HDD and SATA SSD storage targets | UNTESTED | NVMe only on this host; other media are out of 0.6 scope (owner 2026-09-21), not an open 0.6 gate. |

## Catalog, credentials, and providers

| Claim | Status | Evidence |
|---|---|---|
| Signed catalog with freshness/replay policy and stale-cache fallback | MEASURED | Packaged offline start showed `SHOWING LAST SAVED LIST` with 158 models and the real cooldown; a corrupted mirror was quarantined and rebuilt. |
| SQLite mirror recovery under corruption and NTFS sharing violations | MEASURED | Packaged run: quarantine + rebuild, visible lock failure with the in-memory catalog kept, success after release. |
| Cloud providers (OpenAI-compatible, Anthropic, and the other offered providers) | EXERCISED (contract fixtures only) | Six-provider stub fixture validates request/response handling and retry bounds; no live provider call was made. Live calls are untested and no provider is claimed as supported. |
| Hugging Face token flows (gated repositories, revocation responses) | EXERCISED (fixtures) | Token bounds and cleanup notices are tested; no live authenticated request was made. |
| Credential storage in Windows Credential Manager | EXERCISED | The keyring-backed store is used by tests; live interactive credential prompts were not exercised. |

## Accessibility and presentation

| Claim | Status | Evidence |
|---|---|---|
| Keyboard reachability, labels, focus order, and status announcements on representative flows | MEASURED | Packaged accessibility probe PASS (14 screens, zero violations). |
| Engine-level forced-colors and reduced-motion handling | MEASURED | Forced-colors emulation produces a legible palette; the Jump action respects `prefers-reduced-motion`. |
| Screen readers (Narrator, NVDA) | UNTESTED | No screen reader was available; do not claim assistive-technology support beyond the structural checks. |
| OS-level high-contrast themes applied interactively | UNTESTED | Engine emulation only. |
| Text reflow 320-980 px and 200% / 400% zoom | MEASURED | Packaged responsive probe PASS at 320 / 375 / 680 / 980 px and both zoom levels. |

## Release and distribution

| Claim | Status | Evidence |
|---|---|---|
| Honest unsigned distribution | POLICY RETAINED | Release name carries `(unsigned)`, notes disclose SmartScreen, `SHA256SUMS` is published and verifiable; no signature is claimed. Code signing stays deferred by owner order and blocks no gate. |
| Version-scoped support | POLICY | A claim applies to one version only with release evidence for that version. |
| Performance roofline program (independent distributions, drift, calibration error, queue metrics) | OPEN | S-25.I3 remains deferred; measured numbers are scoped to this host and the shipped workload. |

The Windows 10 note: Microsoft ended normal Windows 10 support on
2025-10-14; servicing or ESU policy applies if that platform is used.

## Version evidence (merged from the retired per-version matrix, P2-9)

Version maps to what was actually exercised. A cell says PASS only where release
evidence for that exact version exists; everything else stays UNKNOWN or NOT
RUN. The authoritative per-item evidence lives in the current tracker,
`release-evidence/`, and the release assets named below.

## Hosts

| Host | Role |
|---|---|
| Ryzen 9 9950X3D, GeForce RTX 5090, Windows 11 26100 | Primary validation and CI self-hosted runner `DESKTOP-HPTF57N-zen5-blackwell` |
| Windows Sandbox (clean account) | Installer lifecycle isolation |


## Versions

| Version | Channel | CPU packaged lifecycle | Accelerator (CUDA) packaged | Clean-account Sandbox | Notes / evidence |
|---|---|---|---|---|---|
| 0.4.0 | Public release | PASS (as published) | UNKNOWN | UNKNOWN | Its release notes labeled three L2/PARTIAL rows "Supported"; the correction is explained in the 0.4.1 notes and this repository. Binaries stay immutable. |
| 0.4.1 | Public release | PASS (as published) | UNKNOWN | UNKNOWN | The frozen record is `docs/history/TODO-0.4.1.md` in git history (last tree `9b09857`). |
| 0.5.0 | Public release | PASS (release workflow `package-smoke` + packaged matrix on this host) | UNKNOWN | FAIL (failed before installation; recorded in the release evidence) | Health evidence is CPU only. Release notes disclose broad L4 limitations. The ship ledger is `docs/history/TODO-0.5.md` in git history (last tree `9b09857`). |
| 0.6.0 | Superseded (never published) | Superseded; see 0.6.1 | NOT RUN | NOT RUN | Superseded by the 0.6.1 candidate; retained as history only. No support claim before the 0.6 evidence exists. |
| 0.6.1 | Superseded (never published) | PASS (local packaged matrices plus retained `release-evidence/0.6.1/` records, source-bound) | NOT RUN under the release gate (laboratory legs moved out; historical observations only) | PASS (baseline upgrade legs for the 0.6.1 candidate, retained unpublished) | Superseded by the published 0.6.5. Earlier 0.6.x tags were never published. |
| 0.6.5 | Public release | PASS (as published) | NOT RUN under the release gate | PASS (as published: one clean-account leg in the release gate) | Latest published release. Health evidence is CPU only; unsigned with SmartScreen disclosure. |

## What each column means

- **CPU packaged lifecycle**: the packaged binary built by the release
  workflow, the MSI/NSIS install, launch, uninstall, and packaged screens
  exercised over CDP on this host.
- **Accelerator (CUDA) packaged**: a lifecycle run where a CUDA runtime
  serves a model end to end. CPU health runs, flag inspection, or "the UI
  offers CUDA" are not evidence for this column.
- **Clean-account Sandbox**: the same lifecycle inside Windows Sandbox with a
  clean account, proving no dependency on the maintainer's machine state.

## Reading rules

- A green row here never upgrades to a broader claim: one exact attestation
  matches one qualification key.
- The catalog signature authenticates the catalog document; it does not
  authenticate application installers (see `SECURITY.md`).
- Releases ship unsigned with disclosure; verify published SHA-256 checksums.

## Copied commands and manifest paths (audit S-14)

- The provisional command preview names its shell: a paste-ready PowerShell
  line (single-quoted, so every metacharacter and Unicode value is literal)
  and, when every value is expressible, a cmd.exe line; a value containing
  `%` refuses the cmd.exe form with an explanatory notice. A JSON `argv`
  array accompanies both and is lossless for any wrapper. The application
  itself always launches llama-server through an argument array, never a
  shell string.
- Raw local run manifests are working files for this machine, not publication
  artifacts. Some command fields are fingerprinted, but draft and LoRA paths
  can remain. Do not publish these files without a separate privacy review.
  The 0.6 source build has no share/export redaction feature.

## Historical calibration lifecycle before 0.6 (audit S-16)

The following records the retired implementation. The 0.6 source build does
not load, prune, export or clear calibration history. Existing files remain
untouched; no current UI control provides this lifecycle.

- **Storage**: one JSON record per file under `<app data>/calibration/anchors`
  and `.../models`; each record is size-bounded (64 KiB) and the directory
  enumeration refuses more than 10 000 records.
- **Retention**: persisting prunes the oldest recognized records beyond 4 000
  per category; only `.json` records in those directories are candidates, and
  everything else is untouched. Pruned records are raw local history: exports
  and share bundles that reference a measurement carry their own copy, so
  retention never silently invalidates an exported artifact.
- **Cleanup**: "Clear local history" in the evidence panel removes every
  stored calibration record and reports the count; quarantined files are kept
  because they are the diagnostic evidence of earlier failures.
- **Corruption**: a corrupt, oversized, schema-invalid or unreadable record is
  moved to `.../calibration/quarantine/<name>.corrupt-<stamp>` and reported as
  a bounded load problem while the remaining compatible history keeps
  loading; one bad file never hides the rest.

## Rust–TypeScript and persisted-format contracts (audit S-18)

- **Wire contract**: `scripts/tests/fixtures/ipc-contract.json` is the shared
  authority for representative IPC payloads. The Rust test
  `s18_shared_ipc_contract_fixture_matches_rust_serialization` asserts its
  serialized values equal the fixture entries exactly (camelCase keys, enum
  spellings, nullable fields serialized as `null`), and
  `scripts/tests/ipc_contract.test.mjs` asserts the same entries match the
  `model.ts` consumer expectations (exact key sets, JS types, camelCase only,
  documented enum spellings). Errors cross the IPC boundary as plain strings
  by contract — never as objects.
- **Persisted formats**: catalog documents carry `schemaVersion`, and benchmark
  manifests carry `schema` plus an execution-snapshot schema in their keys.
  Profiles use `normalizeProfile` on read; frontend checks do not replace Rust
  launch validation. The retired calibration record schema and its migration
  policy are historical, not an active 0.6 storage service.

## Bounded property campaigns (audit S-19)

`src-tauri/src/test_support.rs` provides a deterministic splitmix64 generator;
`src-tauri/src/property_tests.rs` runs five campaigns (400-600 seeds each,
inputs capped at 512 bytes / 256 characters): GGUF reader versus random and
truncated bytes, proposal parser versus noisy and nested-brace text, shard
name parse/display round-trip versus noise, effective-argument sanitizer
idempotence plus a canary path, and numeric summaries versus extreme/NaN
inputs — all asserted finite where the contract promises finite. The frontend
campaign in `src/model.test.ts` runs 300 generated garbage stored profiles
through `safeJsonParse`/`normalizeProfile` against a valid model.

These are BOUNDED campaigns with deterministic seeds, not exhaustive proof and
not coverage-guided fuzzing: a violation reproduces from its reported seed,
resource use is capped by iteration and input-size limits, and passing this
campaign says nothing about inputs outside the generated classes.
