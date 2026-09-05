# Changelog

## 0.4.0

### Validated scope only: approved runtimes, device evidence, explicit selection

Localmotive targets Windows x64. Runtime availability depends on exact processor, accelerator, driver, and llama.cpp artifact. Only configurations in the release compatibility table carry validation. Upstream capability is not product support.

Compatibility table (`b10796`, `9a4843cf2f1a3fc8e39f8148e92ee6bfe18e2db6`):

- Supported: Windows 11, x64, AMD Ryzen 9 9950X3D, CPU — attested `research/0.4/evidence/attestations/local-windows-x64-cpu.json` (`L2`, `PARTIAL`).
- Supported: Windows 11, x64, NVIDIA GeForce RTX 5090, driver 610.74, CUDA 13.3 — attested `research/0.4/evidence/attestations/local-windows-x64-cuda.json` (`L2`, `PARTIAL`).
- Supported: Windows 11, x64, NVIDIA GeForce RTX 5090, driver 610.74, Vulkan — attested `research/0.4/evidence/attestations/local-windows-x64-vulkan.json` (`L2`, `PARTIAL`).
- Not validated: every other catalog entry (ROCm, SYCL, OpenVINO, CUDA 12.4, Arm64) — untested configuration, no automatic preference, no `Supported` label.

Known limitations: no `test-backend-ops` in any pinned `b10796` archive (no `L3 RUNTIME` reachable; 5/5 runnable attestation checks `PASS`, `gguf-pilot-end-to-end` and `performance-qualification` `UNKNOWN`); 16 of 19 P0 classes need hardware owners or approved cloud spend (no AMD/Intel/Pascal/Turing/Ampere/Ada hardware locally); no clean-account/VM run; no installer/launch/inference/cancellation/restart/uninstall evidence; research evidence is gitignored and cited by path, not carried in the tag.

- Runtime selection uses one approved manifest (`src-tauri/approved_runtimes.json` pins `b10796` with URLs, bytes, SHA-256) instead of the newest release with Windows archives. Updates block when a required upstream hardware job fails or remains queued (`server-cuda`, `server-metal`, `gpu-rocm`, `gpu-vulkan-apple`, `gpu-vulkan-nvidia-cm` failed; `gpu-openvino-low-perf` queued).
- Recommendation is device-evidence based: CUDA needs driver branch 580+ (13.x) or 525+ (12.x) plus matching CUDA major; ROCm needs an exact AMD matrix family; SYCL needs Arc/Xe device evidence; OpenVINO and OpenCL stay catalog entries without automatic preference; Arm64 assets stay dormant; Qualcomm maps to dormant OpenCL; empty adapter lists recommend CPU explicitly; mixed-vendor multi-adapter systems require explicit device selection (`require_explicit_device_selection`), never silent cross-backend fallback.
- Runtime health requires `--list-devices` evidence plus the pinned `SmolLM2-135M-Q4_K_M.gguf` model load (`e31313…f5`, 16-token deterministic completion), `127.0.0.1` loopback health, and cancellation cleanup. Version/help success alone never marks a runtime healthy; silent CPU fallback fails accelerator health.
- Installed identity is evidence-derived (`describe_runtime`): managed `runtime.json` manifest, otherwise shipped ggml/CUDA DLLs. A manifest that disagrees with sibling DLLs (wrong backend, wrong CUDA major, missing companion) reports `mismatch` / `manifest-dll-mismatch` with a reinstall action before launch. Completeness per backend: CUDA needs `ggml-cuda.dll` plus matching `cudart64_<major>.dll`; ROCm `ggml-hip.dll`/`ggml-rocm.dll`; SYCL `ggml-sycl.dll`; OpenVINO `ggml-openvino.dll`; Vulkan `ggml-vulkan.dll`; Arm64 OpenCL `ggml-opencl.dll`; CPU `ggml-cpu`/`ggml.dll`.
- Extraction refuses traversal, symlink bits, duplicate paths (canonicalized), entry-count, per-entry, total-byte, and path-length violations. Install paths sanitize spaces, Unicode, drive-root, UNC, and `..` forms to `Normal`-only relative paths; manifest runtimes with non-normal components fail.
- The managed runtime screen shows one `SUPPORTED` / `NOT VALIDATED` tag plus a scope line per option (`OS arch · device · driver · backend · revision`), hardware evidence per adapter with source, artifact digest/size before actions, and `MISMATCH · REINSTALL` before launch on identity mismatch. New decision functions `supportStatusForOption` and `runtimeIdentityMismatch` carry Vitest coverage (46 tests).
- Verification on the release candidate: `cargo fmt --check` passes; `cargo clippy --locked --all-targets -- -D warnings` passes; `cargo test --locked` passes (296 passed, 1 ignored); `npx tsc --noEmit -p tsconfig.json` passes; `npm test` passes (46); `npm run build` passes; `python3 research/0.4/scripts/run_local_smoke.py` passes 10/10 on CPU/CUDA 13.3/Vulkan; `scripts/verify_040.mjs` (`node --check` passes) asserts packaged values but is not run here — it needs `npm run tauri build` plus a human-gated release tag, signing, and publication.
- Tracker: `TODO-0.4.md` holds pins, frozen P0 list, phase gates, and the verification ledger. Architecture: `docs/RUNTIME_MANAGER.md` gains the product support contract and identity/completeness sections.

## 0.3.0

### Evidence-first model fit, measurement, and sharing

- Added typed v0.3 evidence contracts in `src-tauri/src/evidence.rs`: `Evidence<T>` with level, source, observation time, and notes; `FitClass`; `ExecutionPath`; bounded `Workload`; and a versioned `BenchmarkManifest`. The frontend mirrors these contracts in `src/model.ts`.
- Added artifact truth in `src-tauri/src/artifact.rs`: exact per-shard and companion records, streamed SHA-256, shard gap/duplicate/conflicting-header rejection, and shard/companion byte separation. GGUF parsing remains metadata-only in `src-tauri/src/gguf.rs` with bounded tensor descriptors.
- Added conservative preflight in `src-tauri/src/preflight.rs`: exact per-volume storage facts, volume-aware disk capacity, dense KV-cache arithmetic only from complete terms, named policy reserves, per-device allocation plans, and `Unknown` for unverified topology or companion compatibility.
- Launch validation uses one validator for preview, Start, benchmarks, quality runs, and tuning: current `--help`/`--version` capability parsing, argument filtering with preserved rejections, loopback/IP-literal host policy, port probing with post-`/health` Windows listener-ownership proof, effective `/props` context evidence, and structured exit/log-tail failure evidence.
- Benchmark v2 records schema-versioned manifests with runtime, hardware, model, launch, workload, warmup, trial, failure, timeout, and cancellation evidence; tokenizer-exact prompt-token preparation through `/tokenize`; fixed `/completion` protocol; raw prefill/decode observations; directly observed `firstTokenMs` only; derived TTFT kept separate; per-attempt `GetProcessMemoryInfo(PeakWorkingSetSize)` evidence; cold-cache fresh-runtime trials; and replay through typed inputs with compatibility-key comparison.
- Added a deterministic structural quality suite plus Pareto ranking over launch-validated or measured candidates with explicit constraints, preference components, dominance explanations, and rejected-candidate reasons.
- Added local calibration with compatibility-keyed anchors/models, TTL/expiry handling, bounded local persistence, plus imported external evidence that stays pending until explicit review. Imported evidence never upgrades automatically.
- Added privacy-reviewed local sharing exports: versioned JSON bundles with purpose-specific digests instead of paths/prompts/credentials/bytes, explicit user confirmation, bounded collections, and local-file writes that never overwrite.
- Added the v0.3 evidence workbench in `src/V03EvidencePanel.tsx`: artifact and preflight cards, launch proof with effective context, benchmark v2 controls and summaries, quality/Pareto ranking, calibration/imported evidence, and a confirmed local privacy export. 43 Vitest cases cover these decisions.
- Verification: Rust 267 passed with 1 ignored; frontend 43 passed; `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, TypeScript check, catalog validation, production frontend build, and research verification pass. Packaged Windows build verified: `GGUF Pilot_0.3.0_x64_en-US.msi` (`37503709e6ef51e92181a79969efd04039336d45b43bde63907a218b65315202`), `GGUF Pilot_0.3.0_x64-setup.exe` (`ec2139891d545d2c3ffbf673c2f3d2907380ef9edb68ae184cdafbc7d44b03e2`), portable `gguf-pilot.exe` (`ff8629d98770ff8bf671fbe9fe0dc28b73de8a66ba47c2ddbd0759e254ce7e41`), and packaged CDP verification (`scripts/verify_030.mjs`) passes: hardware 1 adapter, replay rejection, Measured-only ranking, calibration/external guards, share confirmation, DNS rejection, and evidence-panel render.

### Curated Hugging Face catalog

- Added an eighth destination, **HF Catalog**, with full-text search, task/quantization/size filters, sorting by downloads, likes, name or file size, exact GGUF selection, repository links and model-folder-aware download readiness.
- The catalog is maintainer-controlled but model bytes go directly from Hugging Face to the user's selected folder. A validated, Ed25519-signed static JSON manifest on `raw.githubusercontent.com` replaces a hosted database: it is CDN-served, ETag-cacheable, versioned and auditable. The app rejects unsigned network changes, embeds the release's catalog, and caches the last signed copy so first-run and offline browsing still work.
- Added an optional Hugging Face read token stored under `GGUF Pilot HF` in Windows Credential Manager. Authentication still matters for account-based Hub rate limits and gated repositories, but the interface no longer promises that a token increases raw transfer bandwidth.

### Resumable direct downloads

- Large GGUF files use bounded parallel HTTP range requests with retry/backoff, a persisted per-chunk resume map and cancellation. Restarting the app keeps completed byte ranges instead of restarting the whole file.
- Completed downloads verify exact byte length and the curator-published Hugging Face LFS SHA-256. Existing files are reused only after the same digest check; range responses are bound to the probed remote identity so changed objects cannot be spliced into one file.
- Hardened path handling, URL encoding, chunk boundaries, duplicate-download rejection and failure cleanup. A range response can never write into the next chunk, and an ETag appearing or disappearing forces a clean restart.

### Contributor and release gates

- Added `AGENTS.md` with architecture, development workflow, security rules, release procedure and a mandatory test-first regression policy.
- Added catalog schema validation plus focused Rust and TypeScript coverage for catalog parsing/filtering, token masking, download readiness/progress, range planning, resume identity, cancellation, checksums and path safety.
- CI now runs the catalog validator, production frontend, dependency audit, strict Clippy, Rust tests and warning-free rustdoc, then builds all Windows artifacts and launches the packaged executable. Release tags must point to `main`, match every manifest, contain non-empty release notes and pass the same packaged smoke test.

## 0.2.4

### The runtime you chose is the runtime that is used

- Fixed: opening a model whose profile was saved against an older executable silently re-pinned the app to that old path, so an installed runtime update appeared not to stick. `normalizeProfile` now takes the runtime currently in use and keeps every other saved setting (name, context, batching, speculation) untouched. Covered by two frontend tests.

### About tab

- New seventh destination, **About**: version, bundle identifier, platform, framework and licence; the runtime, backend, accelerator and model count on this machine; and exactly where things live — model folder, managed runtime root, server logs, Windows Credential Manager for cloud keys, local storage for settings.
- Credits llama.cpp, Tauri, React/TypeScript/Vite and Lucide with links, and states plainly that the project is not affiliated with ggml-org.

### Ports no longer collide

- The default 8080 was taken whenever another server was already listening. New `pick_free_port` scans upward from the requested port and the first free one is used, so a fresh profile lands on 8081, 8082, … instead of failing at launch. A saved profile keeps the port you chose; only unsaved suggestions move, and the app says so in the notice strip.

### Multiple DSpark companions explained

- Investigated the DeepSeek-V4-Flash-0731 report: the eight DSpark entries are real files — seven in `dspark/` (BF16, Q4_K_M-hybrid, Q5_K_M-hybrid, Q6_K, Q6_K-hybrid, Q6_K-from-Q8, Q6_K-from-BF16-hybrid) plus `dspark-…-Q8_0.gguf` at the family root. The scanner was right to find them; the interface was wrong to present them as an undifferentiated pile and to pick one arbitrarily.
- Companions are now ranked by role, then by how close their quantisation is to the target's, then by name — deterministic across scans. For the UD-IQ4_XS target the Q4_K_M draft leads and BF16 sorts last.
- Quantisation labels are read from the *first* label in the name, so `Q6_K-from-BF16` is a Q6_K file, not a BF16 one.
- Inventory collapses repeats into a counted chip (`DSPARK ×8`), and the Profile screen offers a picker listing every draft when a family ships more than one.

### Repository and releases

- Published to GitHub with MIT licence, `.gitignore`, and two workflows:
  - **CI** — type-check, frontend tests, production build, `cargo fmt --check`, strict Clippy and Rust tests on every push and PR to `main`.
  - **Release** — pushing a `v*` tag verifies the tag matches all three manifests, runs the full check suite, builds, and publishes a release titled with the version whose body is this changelog section, attaching the NSIS installer, the MSI, the portable executable, and a SHA256SUMS file.

## 0.2.2

### Runtime Manager knows what is installed

- Each official build in the Runtime tab now shows its true relationship to the runtime in use instead of a permanent **Install** button:
  - **Up to date** (disabled, green) when the active executable is this package at the current release.
  - **Update to bNNNNN** when the active runtime is this backend but an older build — shown for managed installs and for user-supplied executables alike.
  - **Use this build** when the package is already downloaded into the managed folder but is not the active runtime.
  - **Install** only for packages not present on this PC.
- Backend identity is read from the DLLs beside `llama-server.exe` (`ggml-cuda.dll` plus `cudart64_13.dll` → CUDA 13; `ggml-vulkan.dll` → Vulkan; `ggml-hip.dll` → ROCm; SYCL/OpenVINO likewise), falling back to the managed `runtime.json` manifest. The sidebar shows the detected backend and its source.
- The sidebar lists every installed managed build; clicking one activates it.
- New Rust surface: `describe_runtime`, `list_managed_runtimes`; new TypeScript state machine `runtimeOptionState` (8 Vitest cases).

### AI Tune tab

- New sixth destination, **AI Tune**: a cloud model proposes llama-server settings and this PC measures them; the best measured configuration wins.
- **Providers:** OpenRouter (API key or browser sign-in via OAuth PKCE, mirroring how Hermes Agent authenticates), Anthropic, OpenAI, Google Gemini, DeepSeek, and xAI through their OpenAI-compatible endpoints. Model lists are fetched live; a **Test connection** button proves the credential before spending anything.
- **Credentials** are stored in Windows Credential Manager under the `GGUF Pilot` service via the `keyring` crate — never in local storage, settings files, logs, or the command line. The UI shows only a masked suffix. **Forget key** removes the entry.
- **Context length** is chosen from powers of two up to the model's native maximum, read from the GGUF header; every trial runs at exactly that context.
- The advisor receives a structured brief (hardware, GGUF architecture facts, runtime capabilities, current profile, every prior trial with its measurement) and must answer in strict JSON. It may change only a whitelisted set of throughput-relevant fields; host, port, alias, paths, and security settings are never touched. Proposals outside the whitelist or repeating a measured configuration are rejected before launch.
- Each trial launches llama-server with the proposed flags, waits for `/health`, benchmarks through `/completion`, stops the server, and reports live through `tuning-progress` events. Failed launches are recorded and fed back to the advisor as evidence.
- Hardened against real advisor behaviour seen in live runs: replies that echo the JSON schema in prose are parsed with a balanced-brace scanner (last valid object wins); a garbled reply costs a retry, and only three in a row end the session; when the advisor selects a model-backed method (`draft-dspark`, `draft-dflash`, `draft-eagle3`, `draft-mtp`, `draft-simple`) the tuner attaches the matching companion itself — `draftModel` stays tuner-owned and echoed values are ignored rather than rejected.
- **Adopt best as profile** writes the winning configuration into the model's saved profile.
- New modules: `src-tauri/src/gguf.rs` (header-only GGUF reader), `src-tauri/src/tune.rs` (brief, whitelist, proposal parsing, tuning loop), `src-tauri/src/cloud.rs` (providers, Credential Manager store, chat, PKCE). 45 Rust tests.
- Verified live on the packaged build against OpenRouter (`anthropic/claude-sonnet-4.6`) with LFM2.5-2.6B-Q8_0 at 8,192 context: the advisor found DSpark speculation with `draftPMin=0.1` at 391.13 tok/s vs a 384.14 baseline (+1.8%) in one run and correctly declared the baseline unbeatable in another. The credential was removed from Windows Credential Manager afterwards.

### Other

- `BenchmarkSummary`, `HardwareInfo`, `RuntimeCapabilities`, and `LaunchProfile` are now round-trippable through serde so tuning reports can be stored and re-read.
- Bottom navigation on mobile is six equal columns.

## 0.2.1

### Design language solidified

- `DESIGN.md` rewritten in the DESIGN.md spec format: YAML token frontmatter (20 colors, 20 typography roles, radius, spacing, 24 component entries) followed by the eight canonical sections in order — Overview, Colors, Typography, Layout, Elevation & Depth, Shapes, Components, Do's and Don'ts.
- Creative North Star recorded as **"The Machine-Room Control Cabinet"**, with 13 named rules (The One Meaning Rule, The Flat-Cabinet Rule, The Zero-Radius Rule, The Evidence-Is-Mono Rule, and others) and explicit anti-references.
- `.impeccable/design.json` sidecar added: tonal ramps built from the palette actually in use, the two shadow tokens, motion and focus tokens, breakpoints, layout metrics, and 12 self-contained component HTML/CSS snippets.
- `docs/theme.css` (Tailwind v4 `@theme` block) and `docs/tokens.json` (W3C DTCG) exported from the spec.
- No visual change: the 0.2.0 appearance is the specification. Two CSS variables were reconciled with their real usage (`--red` now holds the lit label color with `--red-deep` for the field; `--paper-ink` is the value actually rendered) and one unused variable was removed.
- `designmd lint`: 0 errors, 0 warnings. Impeccable design detector: 0 findings across `src/App.tsx` and `src/App.css`.

### Console windows suppressed

- New `src-tauri/src/proc.rs` provides `hidden_command`, which sets `CREATE_NO_WINDOW` on Windows.
- Every child process now goes through it: `nvidia-smi.exe` (both calls), the PowerShell `Win32_VideoController` adapter query, `llama-server.exe --version` / `--help` capability inspection, and the served model process itself.
- A source-level test (`proc::tests::no_module_constructs_a_raw_command`) fails the build if any module constructs a raw `std::process::Command`, so the flash cannot regress.
- Server output continues to the log file and the in-app log panel; nothing is lost by hiding the console.

### Other

- Bundle identifier changed from a personal namespace to `io.github.ggufpilot.app`.
