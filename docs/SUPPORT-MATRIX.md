# Localmotive Support Matrix

Claim only what the recorded evidence establishes (audit G-08.I2, GH-07).
Every row names the evidence class: **MEASURED** on this host against a
packaged candidate; **EXERCISED** through automated tests or scripted
fixtures; **REVIEWED** from upstream documentation; **UNTESTED** - no claim.

Machine-readable evidence lives in `release-evidence/<version>/` and
`docs/EVIDENCE-MATRIX.md`; per-finding records live in
`docs/history/TODO-0.6.md`.

## Platform and lifecycle

| Claim | Status | Evidence |
|---|---|---|
| Windows 11 x64 (build 26100 observed) lifecycle: install (MSI and NSIS), launch, uninstall, update | MEASURED | Windows Sandbox clean-account lifecycle PASS for the 0.6.0 candidate and the v0.4.1 / v0.5.0 baseline upgrades, bound to source revision and candidate digests (`sandbox-preservation-*.json`). |
| Windows 10 | UNTESTED | No packaged lifecycle evidence; do not claim support. |
| macOS and Linux | Out of scope | No artifacts are published. |

## Inference backends

| Claim | Status | Evidence |
|---|---|---|
| CUDA inference on the RTX 5090 (driver 610.74) via the approved managed runtime b10816 | MEASURED | Packaged health 7/7 PASS and decode benchmarks: warm means 999.00 / 995.11 tok/s; default workload 1002.60 tok/s (n=5, p95 1007.57); TLS profile 1004.76 tok/s. Model: SmolLM2-135M Q4_K_M. |
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
| HDD and SATA SSD storage targets | UNTESTED | NVMe only on this host; connection-scale throughput on other media remains open (S-25.I3). |

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
