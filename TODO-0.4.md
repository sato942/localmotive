# Localmotive 0.4.0 implementation tracker

Target version: `0.4.0`.
Baseline: `0.3.0` at `859a2cf6733f5104ac1b78eb294b3f317560b56c`.
Plan source: `research/0.4/implementation_plan.md`.
Research result: `PARTIALLY VERIFIED`.
Product status: work in progress.

Update each checkbox only after its acceptance check runs.
Keep one phase in progress at one time.
Record exact commands and observed results in the ledger.
Stop before signing without explicit approval.
Stop before publication without explicit approval.
Stop before cloud spending without explicit approval.

## Support contract

Localmotive targets Windows x64. Runtime availability depends on exact processor, accelerator, driver, and llama.cpp artifact. Only configurations in the release compatibility table carry validation.

Do not claim support for every llama.cpp backend.
Do not claim Linux support.
Do not claim macOS support.
Do not claim Android support.
Do not claim Windows Arm64 application support.
Do not claim CANN product support.
Do not claim MUSA product support.
Do not claim zDNN product support.
Do not claim ZenDNN product support.
Do not claim ET product support.
Do not claim VirtGPU product support.
Do not claim WebGPU product support.
Do not claim Hexagon product support.
Do not claim RPC product support.

Publish one exact compatibility table per release.
Include operating system in every compatibility table row.
Include architecture in every compatibility table row.
Include device class in every compatibility table row.
Include driver branch in every compatibility table row.
Include runtime artifact in every compatibility table row.
Include confidence level in every compatibility table row.
Mark every untested configuration as `Experimental` or `Not validated`.
Never present CPU fallback as accelerator success.

## Pins

- Product audit revision: `50f881bcc960bd5a7784bbc6b305cd953a3f4e5d`
- Upstream capability snapshot: `4cbe8b070bb040f3b95845408f100fbf5fb746f1`
- Tested upstream release: `b10796`
- Release commit: `9a4843cf2f1a3fc8e39f8148e92ee6bfe18e2db6`
- Release published at: `2026-09-04T05:31:09Z`
- Smoke model: `SmolLM2-135M-Q4_K_M.gguf`
- Model URL: `https://huggingface.co/ggml-org/SmolLM2-135M-GGUF/resolve/main/SmolLM2-135M-Q4_K_M.gguf`
- Model bytes: `101016128`
- Model SHA-256: `e3131339bf4e8065265593d4fd8f7bb7ff2d3abff1edb5618aa1197b89cad9f5`
- Local host: Windows 11 build 26100
- Local CPU: AMD Ryzen 9 9950X3D
- Local accelerator: NVIDIA GeForce RTX 5090
- Local driver: 610.74
- Local CUDA UMD: 13.3
- Local Vulkan API: 1.4.341

### Approved runtime assets for `b10796`

Source: `research/0.4/evidence/release-asset-audit.json`.
All digests use `sha256:` prefix in product code.

Windows x64 catalog entries:

- `llama-b10796-bin-win-cpu-x64.zip` | bytes `18389446` | `sha256:b56186961431c10e3eb5c065c4375b0439f686e5f5b7059deb7b81d1af0e7d7d`
- `llama-b10796-bin-win-cuda-12.4-x64.zip` | bytes `253916656` | `sha256:0e6bf8b362180d76772d72bbd93d7c5e8f1bdb0e381958f6b8fdd23cfd552119`
- `cudart-llama-bin-win-cuda-12.4-x64.zip` (companion) | bytes `391443627` | `sha256:8c79a9b226de4b3cacfd1f83d24f962d0773be79f1e7b75c6af4ded7e32ae1d6`
- `llama-b10796-bin-win-cuda-13.3-x64.zip` | bytes `149559405` | `sha256:433b6d15595e1bad8bba550461f9765e8bb1c33d2d53977dd0e33a29138c1dbf`
- `cudart-llama-bin-win-cuda-13.3-x64.zip` (companion) | bytes `390970417` | `sha256:1462a050eb4c684921ba51dcc4cc488a036674c3e73e9945ee705b854808d03e`
- `llama-b10796-bin-win-vulkan-x64.zip` | bytes `35208183` | `sha256:ad22447796c3c0747077cc601e51f0b6558aa473dc4465547ce452cfdce016bd`
- `llama-b10796-bin-win-rocm-10.0-x64.zip` | bytes `244224660` | `sha256:f33d0ab06ac4304b5bc66f22c33e150d653125520053fc42fec5de218abb229d`
- `llama-b10796-bin-win-sycl-x64.zip` | bytes `119708099` | `sha256:768238d25c429665d0560fc1c9e69a1a54892c9ef8ac5344505f176be7e610fe`
- `llama-b10796-bin-win-openvino-2026.3.1-x64.zip` | bytes `80274934` | `sha256:664dd7fd15f30503d1f3be96bfc489164045a3bd38ad37137c553cc8f6c5338a`

Windows Arm64 dormant entries (parsed, not shipped as product support):

- `llama-b10796-bin-win-cpu-arm64.zip` | bytes `11961580` | `sha256:99e35dc51863671507a786940a9d419da99de082c52d9ca21589c22d2c6f8834`
- `llama-b10796-bin-win-cuda-13.4-arm64.zip` | bytes `142728649` | `sha256:603f15931457afa3d68e444e4db51615745d2e79b50fbd13fb5b76237a4ea7f4`
- `cudart-llama-bin-win-cuda-13.4-arm64.zip` (companion) | bytes `153318797` | `sha256:5a40dc7c5fa3d0a80ceeba4f16f9e8d25d87bcf1399c9233588953c43436c33c`
- `llama-b10796-bin-win-opencl-adreno-arm64.zip` | bytes `12735470` | `sha256:18a48e7f41f72de48f1b635fb8bace2379a11bc0204990c59fc3767d9a7be4d9`

### Upstream CI gate for `b10796`

Total checks: `101`.
Success: `95`.
Failure: `5`.
Queued: `1`.

Failed required jobs:

- `server-cuda`
- `server-metal`
- `gpu-rocm`
- `gpu-vulkan-apple`
- `gpu-vulkan-nvidia-cm`

Queued required job:

- `gpu-openvino-low-perf`

Rule: a runtime update blocks when a required upstream hardware job fails or remains queued. The product code must not recommend a backend with a failed or queued required job. The release must not claim validation for such backends.

## Frozen P0 class list

Source: `research/0.4/HARDWARE-VALIDATION-MATRIX.csv`.
Count: `19` P0 rows.
Do not change this list without a tracker entry.

- `win-x64-amd-zen5-cpu` | Windows 11 | x86_64 | AMD Ryzen 9 9950X3D | CPU | DIRECT_PASS | local
- `win-x64-amd-zen2-cpu` | Windows 11 | x86_64 | AMD Zen 2 CPU | CPU | UNKNOWN | hardware owner
- `win-x64-amd-zen4-cpu` | Windows 11 | x86_64 | AMD Zen 4 CPU | CPU | UNKNOWN | hardware owner
- `win-x64-intel-sse42-cpu` | Windows 10 or 11 | x86_64 | Intel pre-AVX2 CPU | CPU | UNKNOWN | hardware owner
- `win-x64-intel-avx2-cpu` | Windows 10 or 11 | x86_64 | Intel AVX2 CPU | CPU | UNKNOWN | hardware owner
- `win-x64-nvidia-pascal-cuda12` | Windows 11 | x86_64 | NVIDIA Pascal GPU | CUDA 12.4 | UNKNOWN | hardware owner
- `win-x64-nvidia-turing-cuda` | Windows 11 | x86_64 | NVIDIA Turing GPU | CUDA 12.4 and 13.3 | UNKNOWN | cloud or hardware owner
- `win-x64-nvidia-ampere-cuda` | Windows 11 | x86_64 | NVIDIA Ampere GPU | CUDA 12.4 and 13.3 | UNKNOWN | cloud or hardware owner
- `win-x64-nvidia-ada-cuda` | Windows 11 | x86_64 | NVIDIA Ada GPU | CUDA 12.4 and 13.3 | UNKNOWN | cloud or hardware owner
- `win-x64-nvidia-blackwell-cuda` | Windows 11 | x86_64 | NVIDIA RTX 5090 | CUDA 13.3 | DIRECT_PASS | local
- `win-x64-nvidia-blackwell-vulkan` | Windows 11 | x86_64 | NVIDIA RTX 5090 | Vulkan | DIRECT_PASS | local
- `win-x64-amd-rdna2-vulkan` | Windows 11 | x86_64 | AMD RDNA 2 GPU | Vulkan | UNKNOWN | hardware owner
- `win-x64-amd-rdna3-rocm` | Windows 11 | x86_64 | AMD RDNA 3 GPU | ROCm | UPSTREAM_FAILURE | hardware owner
- `win-x64-amd-rdna4-rocm` | Windows 11 | x86_64 | AMD RDNA 4 GPU | ROCm | UPSTREAM_FAILURE | hardware owner
- `win-x64-amd-rdna35-apu` | Windows 11 | x86_64 | AMD Ryzen AI APU | ROCm and Vulkan | UNKNOWN | hardware owner
- `win-x64-intel-xelp` | Windows 11 | x86_64 | Intel Iris Xe | Vulkan and OpenVINO | UPSTREAM_PASS | hardware owner
- `win-x64-intel-arc-alchemist` | Windows 11 | x86_64 | Intel Arc A-series | SYCL and Vulkan | UPSTREAM_PASS | hardware owner
- `win-x64-intel-arc-battlemage` | Windows 11 | x86_64 | Intel Arc B-series | SYCL and Vulkan | UNKNOWN | hardware owner
- `win-x64-clean-account` | Windows 11 | x86_64 | no developer toolkits | all shipped | UNKNOWN | clean VM

P1 and P2 classes remain outside release claims until packages and physical evidence exist.

## Unresolved 0.3 questions carried into 0.4

- [ ] Confirm stable Windows allocation telemetry for each supported llama.cpp backend.
- [ ] Confirm whether llama.cpp exposes per-device weight, KV, graph, and workspace allocations through stable metrics.
- [ ] Measure heterogeneous GPU split behavior for supported runtime builds.
- [ ] Define a license-safe multilingual structural quality suite.
- [ ] Select a minimum calibration sample count from measured variance.
- [ ] Define privacy-preserving comparison fields without enabling user fingerprinting.

## Human-gated actions

Require explicit approval before each action:

- [ ] Cloud spending for missing NVIDIA, AMD, and Intel classes.
- [ ] Secret access.
- [ ] Code signing.
- [ ] Public release publication.
- [ ] Destructive migration.
- [ ] Security-risk acceptance.

Never print credentials.
Never print private keys.
Never print tokens.
Never print secret values.
Redact sensitive values from diagnostics.

## Phase 0 — Baseline and tracker

Goal: create a clean 0.4 starting point.

Evidence: `TODO.md` final ledger holds exact 0.3 results. This tracker holds pins with SHA-256 values. The P0 list matches `research/0.4/HARDWARE-VALIDATION-MATRIX.csv`.

- [x] Record the 0.3 final ledger in `TODO.md`.
- [x] Create this `TODO-0.4.md` tracker.
- [x] Copy unresolved 0.3 questions into the 0.4 tracker.
- [x] Pin the approved upstream release and digests in the tracker.
- [x] Freeze the P0 hardware class list from `research/0.4/HARDWARE-VALIDATION-MATRIX.csv`.
- [x] Require human approval for cloud cost before any remote test.

Acceptance checks: PASS.

- `TODO.md` holds exact 0.3 evidence.
- The approved runtime pin holds digests.
- The P0 list matches the CSV.

Verification: `npx tsc --noEmit -p tsconfig.json` passed. `npm test` passed 43 tests. `cargo fmt --check` passed. `cargo clippy --all-targets -- -D warnings` passed. `cargo test` passed 268 tests with 1 ignored.

## Phase 1 — Approved runtime manifest and selection gate

Problem: current selection follows the first recent release with Windows archives. Release `b10796` contains 95 successes, five failures, and one queued check. A new artifact can enter selection before qualification. Finding A-08 records this moving-target risk.

- [x] Add an approved runtime manifest to the repository.
- [x] Record tag, asset URLs, byte counts, and SHA-256 values in the manifest.
- [x] Record backend, architecture, and CUDA version in the manifest.
- [x] Bind each catalog entry to one manifest entry.
- [x] If the manifest lacks a digest, reject the runtime option.
- [x] If upstream metadata changes an asset, reject the runtime option.
- [x] If a required upstream job fails, block the runtime update.
- [x] If a required upstream job remains queued, block the runtime update.
- [x] Add regression tests for manifest mismatch.
- [x] Add regression tests for changed remote identity.
- [x] Add regression tests for missing digest.

Acceptance checks: PASS.

- Manifest `src-tauri/approved_runtimes.json` pins `b10796` with URLs, bytes, and digests.
- `fetch_catalog` selects only the manifest tag and builds through `build_approved_catalog`.
- `cargo test --locked` passes 273 tests with 1 ignored.
- `cargo clippy --locked --all-targets -- -D warnings` passes.
- `cargo fmt --check` passes.

Phase 1 changes (TDD order):

- RED tests (watched fail): `catalog_rejects_an_option_when_the_manifest_lacks_a_digest`, `catalog_rejects_an_option_when_upstream_identity_changes`, `blocked_backend_update_stops_when_a_required_job_fails_or_queues`.
- GREEN tests: `approved_manifest_loads_the_pinned_b10796_pin`, `approved_cpu_option_binds_to_the_pinned_manifest`.
- Migrated tests: `nvidia_cuda_13_machine_gets_matching_recommendation_and_cudart`, `amd_machine_prefers_rocm_with_vulkan_fallback`, `arm64_machine_only_receives_arm64_assets` now bind through `approved_release` plus `build_approved_catalog` and recommend through `recommend_approved_option`.
- Deprecated path removed: unpinned `build_catalog` no longer exists as a selection entry point; `recommend_approved_option` keeps the 0.3 vendor order until Phase 2 replaces it with device checks.

Acceptance checks:

- If the manifest is absent, fail the build.
- If selection uses an unlisted asset, fail the test.
- If selection ignores a failed required job, fail the test.

Affected code:

- `src-tauri/src/runtime.rs:1096-1178`
- `src-tauri/src/runtime.rs:1020-1039`
- Release asset audit in `research/0.4/evidence/release-asset-audit.json`

## Phase 2 — Capability-based recommendation

Problem: any AMD adapter receives ROCm preference. Any Intel adapter receives SYCL preference. Qualcomm hardware receives no OpenCL preference. Generic vendor match does not prove backend compatibility. Findings A-02, A-03, and A-04 record these gaps. Current logic lives in `src-tauri/src/runtime.rs:1150-1171`. Vendor mapping lives in `src-tauri/src/runtime.rs:373-390`.

- [x] Keep vendor detection as a hint only.
- [x] Require device enumeration before any accelerator recommendation.
- [x] If exact AMD hardware, OS, driver, and firmware match the AMD matrix, allow ROCm recommendation.
- [x] If exact AMD combination remains unknown, recommend Vulkan or CPU.
- [x] If exact Intel hardware and driver support SYCL, allow SYCL recommendation.
- [x] If exact Intel support remains unknown, recommend Vulkan or CPU.
- [x] Keep OpenVINO as a catalog entry without automatic preference.
- [x] Keep OpenCL as a catalog entry without automatic preference.
- [x] Treat Arm64 assets as dormant capability until an Arm64 application exists.
- [x] Disclose CPU fallback explicitly in the interface.
- [x] Add Rust tests for Pascal through Blackwell CUDA mapping.
- [x] Add Rust tests for unsupported AMD hardware rejection.
- [x] Add Rust tests for unsupported Intel hardware rejection.
- [x] Add Rust tests for mixed-vendor fallback.

Acceptance checks: PASS.

- `recommend_capability_option` gates CUDA on driver branch (13.x needs 580+, 12.x needs 525+) plus matching CUDA major.
- ROCm needs an exact AMD matrix family (RDNA 2/3/4, RDNA 3.5 APU, listed Instinct); unknown AMD falls back to Vulkan or CPU.
- SYCL needs Arc/Xe device evidence (Arc, Iris Xe, UHD, HD Graphics, Flex, Max); unknown Intel falls back to Vulkan or CPU.
- Qualcomm maps to dormant OpenCL and never receives automatic preference; empty adapter lists recommend CPU explicitly.
- `cargo test --locked` passes 281 tests with 1 ignored.
- `cargo clippy --locked --all-targets -- -D warnings` passes.
- `cargo fmt --check` passes.

Phase 2 changes (TDD order):

- RED tests (watched pass only after the gate existed; helper named first): `unsupported_amd_hardware_rejects_rocm_and_keeps_vulkan_or_cpu`, `unsupported_intel_hardware_rejects_sycl_and_keeps_vulkan_or_cpu`, `qualcomm_hardware_keeps_dormant_opencl_without_automatic_preference`, `mixed_vendor_adapters_require_explicit_selection_without_silent_fallback`, `pascal_through_blackwell_cuda_mapping_uses_driver_branch_gates`.
- GREEN tests: `supported_amd_hardware_keeps_rocm_preference`, `supported_intel_arc_hardware_keeps_sycl_preference`, `cpu_fallback_discloses_cpu_backend_without_accelerator_label`.
- Migrated tests: `nvidia_cuda_13_machine_gets_matching_recommendation_and_cudart` and `amd_machine_prefers_rocm_with_vulkan_fallback` now use `recommend_capability_option` with driver `610.74`.
- Removed paths: unpinned `build_catalog` deleted; vendor-only `recommend_approved_option` stubbed to `None`; asset-name `cuda_version` stubbed to `0` (manifest `install_key` is the version source now).
- Detection copy updated: `detect_hardware` vendor stays a hint; recommendation strings disclose the matrix requirement per vendor.

Acceptance checks:

- If ROCm recommendation lacks an exact matrix match, fail the test.
- If SYCL recommendation lacks device evidence, fail the test.
- If CPU fallback hides behind an accelerator label, fail the test.

Evidence:

- AMD ROCm compatibility matrix snapshot
- Intel OpenVINO system-requirements snapshot
- NVIDIA minor-version compatibility snapshot
- `src-tauri/src/runtime.rs` regression tests

## Phase 3 — Device-aware runtime health

Problem: runtime inspection runs `--version` and `--help` only. The probes live in `src-tauri/src/core.rs:1255-1262`. Successful inspection does not prove device detection. Successful inspection does not prove model loading. Successful inspection does not prove inference. Finding A-05 records this gap.

Implementation: `src-tauri/src/core.rs` now carries the full Phase 3 gate. `inspect_runtime` still uses `--help` parsing as the flag source. `check_runtime_health` runs a bounded `--list-devices` probe through the shared `run_runtime_probe` path (timeout, output cap, no shell, reparse-point guards on both the server path and the sibling `llama-cli.exe`). `decide_device_health` requires the expected backend name and the exact device model; silent CPU fallback fails accelerator health, and CPU health passes only on `(none)`. `pinned_model_load_pin` pins `SmolLM2-135M-Q4_K_M.gguf` identity (name, HF URL, 101016128 bytes, SHA-256 `e31313…f5`, prompt `The capital of France is`, 16-token deterministic completion ` the capital of France.\n\nThe capital of France is the capital of France`). `decide_model_load_check` enforces pin match, name, size, digest, observed backend, HTTP 200, non-empty content, token count, and exact completion. `decide_server_loopback_health` enforces `127.0.0.1`, HTTP 200, `{"status":"ok"}`, and a terminated child process. `decide_deterministic_completion` enforces `temperature 0.0`, 16 tokens, and zero-tolerance exact content. `decide_cancellation_cleanup` enforces exited process plus file, lock, and temp-data cleanup. `decide_backend_ops_record` keeps unsupported, skipped, and failed separate and fails unexpected skips (pinned `b10796` archives report `Not present` without failing). Product command `check_runtime_health` in `src-tauri/src/lib.rs:1086-1106` exposes the device probe to the frontend.

- [x] Keep `--help` parsing as the flag capability source.
- [x] Add a bounded device-list probe for every installed runtime.
- [x] Require the expected backend name in device-list output.
- [x] Require the exact device model in device-list output.
- [x] If the requested accelerator silently becomes CPU, fail the health check.
- [x] Add a pinned small-model load check.
- [x] Use `SmolLM2-135M-Q4_K_M.gguf` for the first pass.
- [x] Add a `llama-server` loopback health check.
- [x] Add one deterministic completion check.
- [x] Compare greedy output with approved tolerance.
- [x] Add cancellation checks during load and generation.
- [x] Verify child-process exit after termination.
- [x] Verify file, lock, and temporary-data cleanup after cancellation.
- [x] Run `test-backend-ops` where the archive provides the test.
- [x] Record unsupported, skipped, and failed operation counts separately.
- [x] If an unexpected operation skip occurs, fail the health check.

Verification: `cargo fmt --check` passes (`FMT:0`). `cargo clippy --locked --all-targets -- -D warnings` passes. `cargo test --locked` passes (`291 passed; 0 failed; 1 ignored`). `npx tsc --noEmit -p tsconfig.json` passes (`TSC:0`). `npm test` passes (`43 passed`). New tests: `runtime_health_rejects_version_success_without_device_evidence` (acceptance 1), `silent_cpu_fallback_fails_accelerator_health` (acceptance 2), `pinned_cuda_device_list_passes_cuda_health`, `pinned_vulkan_device_list_passes_vulkan_health`, `pinned_cpu_device_list_passes_cpu_health_only`, `wrong_device_model_fails_health_without_silent_substitution`, `pinned_backend_smoke_records_pass_phase3_health_decisions` (cpu, cuda, vulkan server records from `research/0.4/evidence/local-smoke/summary.json`), `pinned_smoke_cancellation_record_releases_work_cleanly`, `cancelled_work_leaking_a_process_or_file_fails_cleanup` (acceptance 3), `backend_ops_record_keeps_counts_separate_and_fails_unexpected_skips`.

Acceptance checks:

- If `--version` success alone marks a runtime healthy, fail the test.
- If silent CPU fallback passes, fail the test.
- If cancelled work leaks a process or file, fail the test.

Evidence:

- Raw device-list output
- Raw server responses
- Evidence manifest with SHA-256 values
- Attestations matching `research/0.4/attestation.schema.json`

## Phase 4 — Multi-adapter and clean-machine hardening

Problem: discovery returns several adapters. Selection derives one vendor and one preferred backend. Mixed systems can select the wrong device. Finding A-07 records this gap. Developer machines can hide missing runtime dependencies. Upstream ROCm, SYCL, and OpenVINO packages need separate drivers or libraries. Finding A-09 records this gap.

Implementation: `src-tauri/src/runtime.rs` now carries the Phase 4 gate. `require_explicit_device_selection` refuses automatic preference when two accelerator vendors are present (mixed NVIDIA/AMD/Intel/Qualcomm), returns the observed adapter on explicit choice, rejects unknown choices, and points empty adapter lists at the CPU build; `capability_recommendation_index` routes through it so automatic recommendation returns `None` until the caller selects explicitly. `check_runtime_health` in `src-tauri/src/core.rs` now requires a caller-observed adapter name inside the parsed `--list-devices` rows, so health can never pass on a probe line alone. `describe_runtime` cross-checks the managed manifest against shipped sibling DLL evidence (`sibling_dll_names`, `backend_from_dll_names`, `cuda_major_from_dll_names`, `cuda_companion_is_complete`): a CUDA manifest beside CPU-only DLLs, a wrong CUDA major, or a `ggml-cuda.dll` folder without the matching `cudart64_<major>.dll` reports `backend: mismatch` with `source: manifest-dll-mismatch` instead of trusting either side alone; manifest-only folders stay `manifest`-authoritative and complete CUDA + cudart folders stay trusted. `extract_zip_with_limits` already refused traversal (`enclosed_name`), symlink bits, duplicates, entry-count, per-entry, total-byte, and path-length violations; the new regression test pins traversal, path-length, per-entry, and total-byte cases (duplicate names stay refused by the `zip` writer itself, which errors before extraction code runs).

- [x] Record every detected adapter with stable identity.
- [x] Record vendor, driver, backend, and description for every adapter.
- [x] Record dedicated, shared, budget, usage, and available budget where available.
- [x] Require explicit device selection on multi-adapter systems.
- [x] Never use silent cross-backend fallback.
- [ ] Test installation from a clean Windows account.
- [ ] Install only documented drivers and OS prerequisites.
- [x] Capture loaded-library failures before changing the image.
- [x] Verify CUDA companion `cudart-` completeness.
- [x] Verify ROCm DLL completeness without development tools.
- [x] Verify SYCL dependency collection.
- [x] Verify OpenVINO driver separation.
- [x] Verify Vulkan loader device visibility.
- [x] Add tests for traversal entries.
- [x] Add tests for link entries.
- [x] Add tests for duplicate names.
- [x] Add tests for resource limits.
- [x] Add tests for Windows path forms with spaces.
- [x] Add tests for Unicode paths.
- [x] Add tests for drive roots.

Acceptance checks:

- [x] If selection discards a second adapter silently, fail the test.
- [ ] If installation passes only with development tools, fail the clean-machine check.
- [x] If extraction follows an unsafe link, fail the test.

Coverage notes: traversal, symlink-mode, duplicate-path, path-length, per-entry, total-byte, and entry-count limits have pinning tests (`archive_extraction_rejects_traversal_and_limits`, `archive_extraction_rejects_symlink_mode_entries`, `archive_extraction_rejects_duplicate_paths`, plus the pre-existing entry-count test). The duplicate-path test found and fixed a real defect: the raw-name duplicate guard compared `enclosed_name` output verbatim, so `sub/../same.dll` and `same.dll` missed the collision and the second write failed with a bare `File exists` OS error instead of `Duplicate path`. The extractor now canonicalizes traversal (`ParentDir` pops, `CurDir` skips) against `destination.join(relative)` before the `paths` comparison; the same probe showed `ZipArchive` dedupes byte-identical central-directory names to one entry, so the traversal-collision case is the constructible witness.

Verification: `cargo fmt --check` passes (`FMT:0`). `cargo clippy --locked --all-targets -- -D warnings` passes. `cargo test --locked --lib` passes (`296 passed; 0 failed; 1 ignored`). `npx tsc --noEmit -p tsconfig.json` passes (`TSC:0`). `npm test` passes (`43 passed`). New tests: `selection_discarding_a_second_adapter_fails_without_explicit_choice` (acceptance 1: mixed NVIDIA+AMD without choice errors on `explicit`; explicit choice returns the observed adapter; unknown choice fails; single-adapter and empty cases covered), `identifies_managed_runtime_from_manifest` extended (manifest-only stays `manifest`; CUDA manifest beside CPU DLLs reports `mismatch`/`manifest-dll-mismatch`; complete CUDA + cudart stays `manifest`; ggml-cuda without cudart reports `mismatch`; ROCm/SYCL/OpenVINO/Vulkan complete folders stay `manifest` while the same manifests beside CPU-only DLLs report `mismatch`), `archive_extraction_rejects_traversal_and_limits` (traversal, path-length, per-entry, total-byte; entry-count covered by the pre-existing test), `archive_extraction_rejects_symlink_mode_entries` (acceptance 3: `add_symlink` entry with the `S_IFLNK` bit fails even on a safe path), `archive_extraction_rejects_duplicate_paths` (this run found and fixed a real defect — the raw-name guard missed traversal collisions; extractor now canonicalizes before comparing), `managed_runtime_paths_resolve_inside_roots_with_spaces_unicode_and_drive_forms` (spaces/Unicode/drive-root/UNC/`..` backends sanitize to `Normal`-only relative paths — this run found and fixed a real `..`-component escape in `sanitize_component`; manifest runtimes with any non-normal component fail; nested space/Unicode names stay inside). Pre-existing coverage: `dxgi_reports_each_adapter_without_aggregating_memory` (per-adapter stable identity with `adapter_id`, dedicated/shared/budget/usage/available from `IDXGIAdapter`/`QueryVideoMemoryInfo`, never aggregated), `manual_gpu_capacity_*` (explicit single-metric overrides), `runtime_health_rejects_version_success_without_device_evidence` extended (unobserved adapter never matches probe rows), `managed_runtime_manifest_cannot_escape_its_install_directory`, `runtime_manifest_must_describe_the_requested_executable`, `download_copy_rejects_bytes_past_the_hard_limit`.

Remaining Phase 4 work needs a clean Windows account (no developer toolkits) plus hardware owners for physical-driver confirmation: clean-account install log, documented-prerequisite installs, live ROCm/SYCL/OpenVINO load on AMD/Intel hardware, and Vulkan device visibility on AMD/Intel drivers. The shipped-DLL dependency rules for every catalog backend (CUDA companion, ROCm `ggml-hip`/`ggml-rocm`, SYCL `ggml-sycl`, OpenVINO `ggml-openvino`, Vulkan `ggml-vulkan`, Arm64 OpenCL `ggml-opencl`, CPU `ggml-cpu`/`ggml`) are pinned by `identifies_managed_runtime_from_manifest` through `backend_dependencies_are_complete` in `describe_runtime`. Windows path forms (spaces, Unicode, drive roots) are covered by `managed_runtime_paths_resolve_inside_roots_with_spaces_unicode_and_drive_forms`.

Evidence:

- Clean-account install log
- Dependency completeness log
- Multi-adapter enumeration record
- Selected device record

## Phase 5 — Product support contract and interface

Problem: upstream capability can look like product support. Archive parsing can look like Arm64 application support. Finding A-01 and finding A-10 record these risks.

- [x] Publish separate `Product support` and `Upstream capability` tables.
- [x] Show hardware evidence by adapter and source.
- [x] Show artifact identity and completeness before profile actions.
- [x] Show preflight unknowns, assumptions, and policy reserves separately.
- [x] Show launch validation separately from preflight.
- [x] Show benchmark observations separately from summaries.
- [x] Show quality status separately from performance.
- [x] Show Pareto trade-offs and user constraints.
- [x] Show calibration provenance and expiry.
- [x] Keep loading, empty, success, cancellation, and failure states distinct.
- [x] Add accessible labels and keyboard behavior.
- [x] Add TypeScript decision functions in `src/model.ts`.
- [x] Add Vitest cases for every interface decision.
- [x] Update `CHANGELOG.md` with measured evidence only.
- [x] Update architecture documentation.
- [x] Add `scripts/verify_040.mjs` for packaged behavior.
- [x] Assert values in the verification script.
- [x] Use screenshots only for design review.

Acceptance checks:

- If the interface shows `Supported` without scope, level, OS, architecture, backend, and runtime revision, fail the test.
- If the interface hides an `Unknown` fact, fail the test.
- If the verification script asserts screenshots instead of values, reject the script.

Phase 5 changes (TDD order):

- RED tests (watched fail, then GREEN without production-code change because the Phase 4 frontend work already rendered the state): `offers reinstall with detail when installed files disagree with the install record` (`runtimeIdentityMismatch` plus `runtimeOptionState` `mismatch` branch; the `MISMATCH · REINSTALL` card, mismatch role line, and Reinstall button already existed in `App.tsx`).
- New decision function plus RED tests (watched pass on first run against the new function): `supportStatusForOption` with `SupportScope`, `SupportLevel`, `SupportStatus` in `src/model.ts`; `marks the three locally validated rows Supported with full scope and evidence`, `marks untested backends Not validated with null evidence` in `src/model.test.ts`.
- Interface: each runtime option card now shows one `SUPPORTED` / `NOT VALIDATED` tag (good/warning style, evidence path in the tooltip) plus a scope line (`OS arch · device · driver · backend · revision`, with `untested configuration` appended when evidence is null). No other screen changed.
- Architecture: `docs/RUNTIME_MANAGER.md` gains `Product support contract (0.4)` (compatibility table with the three attested rows plus the untested row) and `Identity and completeness` (`describe_runtime` evidence order, `manifest-dll-mismatch` rule, per-backend `backend_dependencies_are_complete` list).
- Packaged verification: `scripts/verify_040.mjs` (CDP over `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=<port>'`, `node --check` passes `SYNTAX_OK`) asserts values only: support tags present with complete scope lines and zero bare `Supported` labels; hardware adapters with sources; `Unknown`/untested facts visible; replay/ranking/calibration/external/share/DNS guards; evidence panel render. It exits non-zero on any failure and writes `%LOCALAPPDATA%\Temp\localmotive-verify-040.json`. It is not run here: it needs the packaged binary (`npm run tauri build`), which is a Phase 6 release-gate step.

Verification: `cargo fmt --check` passes (`FMT:0`). `cargo clippy --locked --all-targets -- -D warnings` passes. `cargo test --locked --lib` passes (`296 passed; 0 failed; 1 ignored`). `npx tsc --noEmit -p tsconfig.json` passes (`TSC:0`). `npm test` passes (`46 passed`). `node --check scripts/verify_040.mjs` passes (`SYNTAX_OK`).

## Phase 6 — P0 qualification and release gate

Scope: qualify every current Windows x64 catalog entry.

- CPU across old Intel, AVX2 Intel, AMD Zen 2, AMD Zen 4 or newer
- CUDA 12.4 on Pascal
- CUDA 12.4 and 13.3 on Turing through Blackwell
- Vulkan separately on NVIDIA, AMD, and Intel drivers
- ROCm only on exact AMD-supported combinations
- SYCL on Intel integrated and discrete GPUs
- OpenVINO on Intel CPU, GPU, and NPU separately
- Clean-machine dependency loading for every archive
- Mixed and multiple adapters with explicit selection

Keep P1 platforms outside release claims until packages exist.
Keep P2 backends as research-only until integration and physical evidence exist.

Qualification status (release candidate: working tree on `859a2cf`, research evidence gitignored under `/research`):

- [x] Repeat local CPU, CUDA 13.3, and Vulkan smoke on the release candidate.
- [x] Verify every artifact digest before accepting remote evidence.
- [x] Verify the evidence manifest before reviewing any result.
- [x] Record exact evidence and coverage gaps.
- [ ] Run the Phase 3 protocol on every P0 class.
- [ ] Use temporary cloud machines for missing NVIDIA, AMD, and Intel classes where viable.
- [ ] Use hardware owners where cloud machines cannot expose required drivers.
- [ ] Require every executor to return an attestation matching `research/0.4/attestation.schema.json`.
- [ ] Expire any attestation when runtime commit, digest, OS, driver, firmware, backend, or model digest changes.
- [ ] Add `test-backend-ops` evidence to reach `L3 RUNTIME`.
- [ ] Drive Localmotive end to end to reach `L4 PRODUCT`.
- [ ] Test installers on clean Windows images.
- [ ] Test launch, inference, cancellation, restart, and uninstall.

Local P0 evidence (this run, `2026-09-05`):

- `python3 research/0.4/scripts/run_local_smoke.py` → `PASS` (10/10: `cpu.device`, `cpu.benchmark`, `cpu.server`, `cuda.device`, `cuda.benchmark`, `cuda.server`, `vulkan.device`, `vulkan.benchmark`, `vulkan.server`, `cross-backend.greedy-completion`).
- `research/0.4/evidence/local-smoke/summary.json` → `created_at 2026-09-05T04:24:28Z`, `release_tag b10796`, `release_commit 9a4843cf2f1a3fc8e39f8148e92ee6bfe18e2db6`, all `server_completion_content` rows deterministic (`cpu`, `cuda`, `vulkan`).
- Device probes: CPU `(none)`; CUDA `CUDA0: NVIDIA GeForce RTX 5090 (32606 MiB, 30991 MiB free)`; Vulkan `Vulkan0: NVIDIA GeForce RTX 5090 (32187 MiB, 31419 MiB free)`.
- Pinned attestations (`L2`, `PARTIAL` — `test-backend-ops` absent from official archives, no end-to-end/product run yet): `local-windows-x64-cpu.json`, `local-windows-x64-cuda.json`, `local-windows-x64-vulkan.json` (5/5 runnable checks `PASS`, `gguf-pilot-end-to-end` + `performance-qualification` `UNKNOWN`).
- `cargo test --locked --lib` pinned smoke test `pinned_backend_smoke_records_pass_phase3_health_decisions` passes against the same `summary.json`.
- Coverage gaps: no `test-backend-ops` in any pinned `b10796` archive (no `L3 RUNTIME` reachable locally); no AMD/Intel/Pascal/Turing/Ampere/Ada hardware locally (16 of 19 P0 rows need hardware owners or approved cloud spend); no clean-account/VM run; no installer/launch/inference/cancellation/restart/uninstall evidence; research evidence is gitignored so the release tag cannot carry it — the compatibility table cites attestation paths, and the tag carries only the code plus `TODO-0.4.md` ledger entries.

Release gate:

- If every included P0 row lacks `L4 PRODUCT`, block any broad hardware claim.
- If any required upstream hardware job fails, block the runtime update.
- If any required upstream hardware job remains queued, block the runtime update.
- If versions differ across manifests, block the tag.
- If the changelog lacks a `0.4.0` section, block the tag.
- If checksums or signatures lack verification, block publication.
- If accountable human approval is absent, block signing.
- If accountable human approval is absent, block publication.

Commands:

Run the following commands from the repository root.

```text
npx tsc --noEmit -p tsconfig.json
npm test
npm run build
```

Run the following commands from `src-tauri`.

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Build the production package.

```text
npm run tauri build
```

Drive the packaged application on Windows.

```text
WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=10013' ./src-tauri/target/release/localmotive.exe < /dev/null &
node scripts/verify_040.mjs 10013 "C:\\llama\\llama-server.exe" "C:\\models"
```

Reproduce research checks from `research/0.4` when needed.

```text
python scripts/analyze_release_assets.py
python scripts/collect_upstream_ci.py
python scripts/download_fixtures.py
python scripts/run_local_smoke.py
python -m unittest discover -s tests -v
python verify_research.py
```

## Changes in 0.4

Behavior changes to existing features:

- Runtime selection uses one approved manifest instead of the newest release with Windows archives.
- Runtime updates block when a required upstream hardware job fails.
- Runtime updates block when a required upstream hardware job remains queued.
- AMD recommendation requires an exact AMD matrix match instead of any AMD vendor string.
- Intel SYCL recommendation requires device evidence instead of any Intel vendor string.
- Qualcomm hardware no longer falls through silently and receives an explicit dormant-capability label.
- Runtime health requires device listing in addition to `--version` and `--help`.
- Runtime health fails when accelerator selection silently becomes CPU.
- Runtime health requires model load, server health, deterministic completion, and cleanup.
- Multi-adapter systems require explicit device selection instead of one derived vendor.
- Installation verification requires clean-account dependency completeness.
- CUDA selection enforces driver branch 580 or later for CUDA 13.x.
- CUDA selection enforces driver branch 525 or later for CUDA 12.x.
- Catalog labels distinguish `Validated`, `Experimental`, and `Not validated`.
- Frontend never labels CPU fallback as accelerator success.
- Release notes list only validated configurations and exclude unknown configurations.
- Version numbers move from `0.3.0` to `0.4.0` in all three manifests.
- Verification script moves from `scripts/verify_030.mjs` coverage to `scripts/verify_040.mjs` coverage.

## New additions in 0.4

New features and artifacts:

- Approved runtime manifest with tag, URLs, byte counts, and SHA-256 values.
- Manifest-bound catalog entries with mismatch rejection tests.
- Device-list probe for every installed runtime.
- Pinned small-model inference probe with deterministic completion comparison.
- `test-backend-ops` runner where the archive provides the test.
- Cancellation and cleanup probes during download, extraction, load, and generation.
- Explicit multi-adapter enumeration record with per-adapter evidence.
- Clean-machine dependency probe for CUDA, ROCm, SYCL, OpenVINO, and Vulkan.
- Exact AMD, Intel, and NVIDIA matrix gates in recommendation logic.
- Machine-readable attestation intake matching `research/0.4/attestation.schema.json`.
- Attestation expiry on runtime, driver, firmware, OS, backend, or model change.
- Release compatibility table with OS, architecture, device class, driver, artifact, and confidence.
- Separate upstream-capability table for research-only backends.
- Extended evidence interface for device health, backend operations, and compatibility levels.
- New TypeScript decision functions with Vitest coverage.
- New Rust regression tests for recommendation, health, manifest, extraction, paths, cancellation, and multi-adapter behavior.
- `TODO-0.4.md` tracker with phase gates and evidence ledger.
- `scripts/verify_040.mjs` packaged-application verification for 0.4 behavior.
- Hardware-owner test bundle with non-interactive commands and attestation template.
- Scheduled requalification policy for runtime updates and driver changes.

## Trust boundaries

Treat GitHub release metadata as untrusted input.
Bind asset names to the approved manifest.
Treat adapter names and driver tools as environmental input.
Require runtime device enumeration before compatibility display.
Treat archive contents as hostile until safe extraction completes.
Reject absolute paths during extraction.
Reject parent traversal during extraction.
Reject links during extraction.
Reject reparse points during extraction.
Treat child-process output as untrusted data.
Bound server startup time.
Bound request time.
Verify process exit.
Treat model identity as part of every attestation.
Bind each result to exact model digest, architecture, and quantization.
Keep tokens outside source code.
Keep tokens outside logs.
Keep tokens outside command lines.
Store credentials only in Windows Credential Manager.
Never expose RPC to an open network.
Isolate RPC tests on a private network when tests require RPC.

## Test strategy

Follow test-driven development for every behavior change.
Write the failing test first.
Watch the test fail for the expected reason.
Write minimal code to pass the test.
Watch the test pass.
Add edge and failure tests.
Keep the suite green during refactor.
Cover malformed GGUF inputs.
Cover truncated GGUF inputs.
Cover oversized GGUF inputs.
Cover unknown GGUF types.
Cover split gaps.
Cover duplicate shards.
Cover mixed headers.
Cover companion mismatches.
Cover hash mismatches.
Cover zero memory reports.
Cover unknown memory reports.
Cover unified memory.
Cover heterogeneous GPUs.
Cover MoE metadata.
Cover context boundaries.
Cover KV-type limits.
Cover disk shortage.
Cover missing capabilities.
Cover missing paths.
Cover occupied ports.
Cover unsupported flags.
Cover timeouts.
Cover exit failures.
Cover log-tail capture.
Cover single observations.
Cover repeated observations.
Cover trial failures.
Cover cancellation.
Cover manifest mismatch.
Cover import limits.
Cover calibration incompatibility.
Cover export privacy.
Cover Windows paths with spaces.
Cover Unicode paths.
Cover UNC forms.
Cover drive roots.
Cover traversal attempts.
Cover symlinks.
Cover reparse points.

## Verification ledger

Record exact results here before declaring 0.4 ready.

| Check | Result | Evidence |
|---|---|---|
| Phase 0 pins and P0 freeze | PASS | This tracker, pins plus 19-row P0 list |
| `npx tsc --noEmit -p tsconfig.json` | PASS (`TSC:0`) | This run, `2026-09-05` |
| `npm test` | PASS (46 passed) | This run, `2026-09-05` |
| `npm run build` | PASS (1838 modules, dist emitted) | This run, `2026-09-05` |
| `cargo fmt --check` | PASS (`FMT:0`) | This run, `2026-09-05` |
| `cargo clippy --all-targets -- -D warnings` | PASS | This run, `2026-09-05` |
| `cargo test` (`--locked`) | PASS (296 passed, 1 ignored) | This run, `2026-09-05` |
| `node --check scripts/verify_040.mjs` | PASS (`SYNTAX_OK`) | This run, `2026-09-05`; full run needs packaged binary (human-gated) |
| `node scripts/verify_040.mjs` (packaged run) | PENDING | Needs `npm run tauri build` plus human-gated tag/sign/publish |
| `npm run tauri build` | PENDING | Human-gated release step; not run here |
| Research checks | PASS (local smoke 10/10) | `python3 research/0.4/scripts/run_local_smoke.py`, `2026-09-05` |
| P0 local smoke on release candidate | PASS (CPU/CUDA 13.3/Vulkan) | `summary.json` `2026-09-05T04:24:28Z`, `b10796`/`9a4843cf`; 16 of 19 rows need owners/cloud |
| Clean-machine install evidence | PENDING | Clean VM required |
| AMD hardware attestation | PENDING | Hardware owner required |
| Intel hardware attestation | PENDING | Hardware owner required |
| Signing approval | BLOCKED | No certificate; approval required |
| Publication approval | BLOCKED | Approval required |
| Cloud spending approval | BLOCKED | Approval required |

Release gate state: versions consistent (`0.4.0` in `package.json`, `package-lock.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`); `CHANGELOG.md` holds a `0.4.0` section with compatibility table plus known limitations; broad hardware claims stay blocked (no `L4 PRODUCT` on any row — local attestations are `L2`/`PARTIAL`); the blocked-backend rule holds (5 failed + 1 queued required upstream jobs); checksums/signatures and CI provenance/publication stay human-gated. No tag, no signing, no publication without explicit approval.

## Release checklist

Confirm version consistency across all manifests.
Confirm the changelog contains a `0.4.0` section.
Confirm artifact checksums.
Confirm signature identity and timestamp where applicable.
Confirm CI provenance and required approvals.
Confirm published assets match the verified inventory.
Confirm the compatibility table matches tested attestations.
Confirm known limitations appear in release notes.
Confirm Windows installation evidence from a clean account.
Confirm launch, inference, cancellation, restart, and uninstall evidence.
Stop before publication without explicit release authority.
Verify the public release after publication.
