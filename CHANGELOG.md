# Changelog

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
