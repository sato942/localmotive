# GGUF Pilot

A public Windows desktop control plane for local GGUF inference. GGUF Pilot manages official `llama.cpp` binaries, discovers and downloads curated GGUF models, builds exact launch profiles, supervises the server process, and measures generation throughput.

## Public features

- First-run setup with no machine-specific paths.
- Detects Windows architecture, graphics adapters, NVIDIA driver CUDA support, and CPU fallback.
- Reads current binary releases from the official `ggml-org/llama.cpp` GitHub repository.
- Recommends the appropriate Windows backend:
  - NVIDIA CUDA, with the matching `cudart` DLL archive
  - AMD ROCm, with Vulkan fallback
  - Intel SYCL, with Vulkan/OpenVINO alternatives
  - Vulkan for broad Windows GPU compatibility
  - CPU for systems without a supported accelerator
  - Windows ARM64 assets when running on ARM64
- Downloads into `%LOCALAPPDATA%\GGUF Pilot\runtimes\<release>\<backend>`.
- Validates published size and SHA-256 metadata before activating a runtime.
- Safely extracts archives and rejects path traversal.
- Keeps versioned runtimes so updates do not overwrite a working older build.
- Supports choosing an existing `llama-server.exe` instead of downloading one.
- Uses the selected executable's real `--help` output as the capability source of truth.
- Omits flags not supported by an older or custom runtime.

## Model and serving features

- Select any model folder; no `C:\models` assumption.
- Recursively presents logical GGUF targets rather than raw shard files.
- Validates split-shard completeness.
- Recognizes `mmproj`, MTP, DSpark, DFlash, and EAGLE-3 companions in parent, child, and sibling family folders.
- Keeps high-value controls visible: context, slots, GPU/CPU placement, flash attention, KV cache, load mode, memory fitting, model companions, reasoning, and prompt caching.
- Categorizes CPU/batching, multi-GPU, cache lifecycle, speculative tuning, sampling, vision, security, adapters, and overrides under Advanced.
- Preserves raw extra arguments for new or model-specific llama.cpp features.
- Enforces safe LAN profiles: non-loopback binding requires a key file and restricted CORS.
- Starts, monitors, logs, benchmarks, and stops only the process owned by the app.
- Opens llama.cpp's built-in WebUI.

## HF Catalog and downloads

- **HF Catalog** is a browsable, searchable and filterable list controlled by the GGUF Pilot maintainers. The catalog only names repositories and files; model bytes travel directly from `huggingface.co` to the selected model folder.
- Filter by text, task tag, quantization and maximum file size; sort by downloads, likes, name or size.
- Downloads use parallel HTTP range requests for large files, retry transient failures, persist a sidecar resume map, and continue verified chunks after a restart. Every completed file is checked against the exact Hugging Face LFS SHA-256 published in the validated catalog.
- An optional Hugging Face read token is stored in Windows Credential Manager under `GGUF Pilot HF`. It is still useful: authenticated requests receive account-based Hub rate limits and can access gated models the account has accepted. It does **not** guarantee higher raw bandwidth.
- The app ships an embedded catalog for first-run/offline use, caches the last signed network copy, and refreshes the public manifest with HTTP ETags. Network catalogs require a detached Ed25519 maintainer signature before they can authorize a download.
- There is no application database or model-file proxy. The static manifest is CDN-served from `raw.githubusercontent.com`; see `catalog/README.md` for the private-control/public-delivery rationale and the optional private-admin-repository pattern.

## AI tuning

- **AI Tune** tab: a cloud model proposes llama-server settings, this PC measures each one, and the best measured configuration wins. Pick the context length to tune for (powers of two up to the model's native maximum from the GGUF header), the number of AI trials, and the measurement size.
- Providers: OpenRouter (browser sign-in via OAuth PKCE, or an API key), Anthropic, OpenAI, Google Gemini, DeepSeek, xAI. Model lists are fetched live; **Test connection** proves the credential before any trial runs.
- Credentials live in Windows Credential Manager under the `GGUF Pilot` service — not in settings files, local storage, logs, or the command line. The app shows only a masked suffix; **Forget key** removes the entry.
- The advisor may change only throughput-relevant fields (offload, threads, batching, KV cache type, flash attention, speculative settings). Host, port, alias, paths, and security settings are never touched. When it selects a model-backed speculative method, the app attaches the matching companion file itself.
- Every trial launches llama-server with the proposed flags, waits for `/health`, benchmarks through `/completion`, and stops the server. Failures are recorded and fed back as evidence. **Adopt best as profile** writes the winner into the model's saved profile.

## Runtime states

Each official build in the Runtime tab reports its true relationship to the runtime in use: **Up to date**, **Update to b<tag>**, **Use this build** (downloaded but not active), or **Install**. Backend identity (CUDA 12/13, Vulkan, ROCm, SYCL, OpenVINO, CPU) is read from the DLLs beside `llama-server.exe`, or from the managed `runtime.json` manifest.

## First run

1. Open **Runtime**.
2. Review detected hardware and install the recommended official build, or browse to an existing `llama-server.exe`.
3. Open **Inventory** and choose the folder containing GGUF models.
4. Select a model, review its profile, and start the server.

Alternatively, choose the folder first and use **HF Catalog** to download a curated GGUF directly into it.

Defaults after setup:

- Host: `127.0.0.1`
- Port: `8080`
- Context: `8192`
- GPU layers: `all`

## Run from source

```bash
npm install
npm run tauri dev
```

## Test and verify

```bash
npm run check
cd src-tauri
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo test --doc
```

## Build installers

```bash
npm run tauri build
```

Artifacts currently produced on this workstation are Windows x64:

- Portable executable: `src-tauri/target/release/gguf-pilot.exe`
- MSI: `src-tauri/target/release/bundle/msi/GGUF Pilot_0.2.5_x64_en-US.msi`
- Setup executable: `src-tauri/target/release/bundle/nsis/GGUF Pilot_0.2.5_x64-setup.exe`

The source and runtime catalog support Windows ARM64, but producing the ARM64 application installer requires an ARM64 MSVC cross-toolchain or ARM64 CI runner. No ARM64 installer is claimed in this handoff.

## Security and integrity

- GitHub release downloads use HTTPS and the official repository API.
- Hugging Face model downloads use HTTPS directly to the Hub; GGUF Pilot never receives or proxies model bytes.
- Hugging Face tokens stay in Windows Credential Manager and are sent only as an authorization header to Hub requests, never in URLs, logs, catalog files, or local storage.
- Asset byte size is checked when supplied.
- Asset SHA-256 is checked when GitHub publishes a digest.
- ZIP entries must remain inside the staging directory.
- Failed installs are removed and never become the selected runtime.
- API secrets are referenced through a key file rather than placed directly in the process command.
- Built-in shell/file tools and agent mode remain raw advanced arguments pending a dedicated permissions workflow.
- The installers in this handoff are not Authenticode-signed. A public publisher should sign MSI/NSIS artifacts to avoid SmartScreen reputation warnings.

## Operating notes

1. Inventory scans do not open or load model weights.
2. The application supervises one model server at a time.
3. Model-backed speculation requires a compatible companion; MTP may be embedded or external.
4. Runtime names never invent flags: DSpark variants map to the actual method advertised by the executable.
5. N-gram methods need no companion and are workload-dependent.
6. Benchmark comparisons require the same prompt, output length, runtime build, context, and placement.
7. Stop terminates the directly owned child process. Windows Job Object containment and multi-server groups remain future hardening work.
8. Every child process — hardware probes, runtime capability inspection, and the model server itself — is created with `CREATE_NO_WINDOW`, so no console window ever flashes on screen. Server output goes to the log file and the in-app log panel.

## Project records

- `PRODUCT.md` — product truth and constraints
- `DESIGN.md` — the design language: token frontmatter plus the machine-room control cabinet spec
- `.impeccable/design.json` — design sidecar: tonal ramps, shadow/motion/focus tokens, component snippets
- `docs/theme.css`, `docs/tokens.json` — Tailwind v4 and W3C DTCG exports of the token layer
- `CHANGELOG.md` — release notes
- `AGENTS.md` — architecture, development rules and mandatory testing workflow for coding agents and contributors
- `catalog/catalog.json`, `catalog/README.md` — curated manifest and owner-only curation/publishing procedure
- `LLAMA-SERVER-README.md` — imported llama-server reference
- `docs/OPTION_MAP.md` — option categorization rationale
- `scripts/` — verification harness: console-window watcher and negative control, CDP drivers (`drive_console_check.mjs`, `live_verify_022.mjs`, `capture_022.mjs`)
- `.impeccable/review/` — desktop/mobile verification captures
