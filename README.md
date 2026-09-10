# Localmotive

Localmotive is a Windows desktop control plane for local GGUF inference.

Use Localmotive to manage `llama.cpp` runtimes, organize GGUF models,
build launch profiles, run `llama-server`, and measure generation throughput.

Localmotive is not an inference engine.

The selected `llama-server.exe` performs model loading and inference.
The same executable provides the local API and optional WebUI.

## Download

Download the current release from [GitHub Releases][releases].

Release 0.3.0 provides these Windows x64 files:

- `Localmotive_0.3.0_x64-setup.exe` is the NSIS installer.
- `Localmotive_0.3.0_x64.msi` is the MSI installer.
- `Localmotive_0.3.0_x64-portable.exe` runs without installation.
- `SHA256SUMS-0.3.0.txt` covers the three application files.

The 0.3.0 application files do not have Authenticode signatures.

Windows SmartScreen can show a warning when you start an unsigned file.

Verify a downloaded file before use:

```powershell
Get-FileHash .\Localmotive_0.3.0_x64-setup.exe -Algorithm SHA256
Get-Content .\SHA256SUMS-0.3.0.txt
```

Compare the two SHA-256 values.

## Requirements

The current release requires a Windows x64 computer.

Select one of these runtime sources:

- An official Windows `llama.cpp` build installed by Localmotive.
- An existing compatible `llama-server.exe` selected by the user.

Provide an existing GGUF model or use **HF Catalog** to download one.

Runtime downloads, catalog refreshes, and model downloads need network access.

Catalog browsing can use the last valid signed cache or embedded catalog
without network access.

AI Tune also needs credentials and network access for one configured provider.

Check model size and hardware requirements before each download.

The bundled 0.3.0 catalog includes individual files larger than 25 GB.
Memory needs and speed depend on the model, quantization, context, backend,
and computer hardware.

## Quick start

1. Start Localmotive.
2. Open **Runtime**.
3. Review the evidence label, then install the recommended `llama.cpp` build or select an existing runtime.
4. Open **Inventory**.
5. Select the folder that contains the GGUF files.
6. Select a complete model.
7. Review the generated profile.
8. Start the server.
9. Select **Open chat** to open the `llama.cpp` WebUI.

You can also select a model folder and download a model from **HF Catalog**.

New profiles use these defaults:

- Host: `127.0.0.1`
- Preferred port: `8080`
- Context: `8192`
- Parallel slots: `1`
- GPU layers: `all`

If the preferred port is unavailable, Localmotive scans a bounded port range.

## Features

### Runtime management

- Detect the Windows architecture and graphics adapters.
- Read the exact approved `ggml-org/llama.cpp` release from GitHub.
- Recommend an accelerator only when an exact L4 compatibility record matches.
- Recommend CPU with an explicit reason when no exact accelerator record matches.
- Show the other release asset choices.
- Pair CUDA runtime archives with the matching `cudart` archive.
- Require each approved asset's exact size and SHA-256.
- Reject traversal, links, junction escapes, duplicate paths, and archive resource-limit violations.
- Verify every extracted file against a compiled exact-content manifest.
- Remove a failed installation's staging directory.
- Install managed runtimes under `%LOCALAPPDATA%\Localmotive\runtimes`.
- Keep runtime versions and backend variants in separate directories.
- Register and inspect an existing `llama-server.exe`.
- Read supported options from the selected runtime's `--help` output.
- Omit generated options that the selected runtime does not advertise.

The approved release contains x64 CPU, CUDA, ROCm, SYCL, OpenVINO, and Vulkan assets.

Localmotive shows only assets that match the running application architecture.

The 0.4.1 ARM64 assets remain dormant and do not create install options.

### Model inventory and profiles

- Scan a selected folder recursively for `.gguf` files.
- Group split GGUF shards into one logical model.
- Block the Start button when a logical model has missing shards.
- Read filenames and file sizes without reading model tensor data.
- Detect `mmproj`, MTP, DSpark, DFlash, and EAGLE-3 files by filename.
- Associate companions in the same top-level model family folder.
- Rank companions by role, quantization proximity, and name.
- Configure model, cache, sampling, network, and placement options.
- Preserve advanced extra arguments as separate process arguments.
- Show the exact filtered command before launch.

### Server control and benchmarking

- Start one `llama-server` child process at a time.
- Redirect server output to a log file.
- Show recent server log lines in the application.
- Report the owned process, endpoint, profile, command, and exit state.
- Stop only the child process stored in the application state.
- Open the built-in `llama.cpp` WebUI when the profile enables it.
- Benchmark `/completion` with a fixed deterministic workload.
- Run one warmup before the measured repetitions.
- Report mean, median, minimum, and maximum generation throughput.

### Curated Hugging Face catalog

- Browse a maintainer-curated list of GGUF files.
- Search by repository, family, publisher, parameters, summary, and tags.
- Filter by tag, quantization, gated status, and maximum file size.
- Sort by downloads, likes, repository name, or smallest file.
- Download selected model bytes from Hugging Face to the model folder.
- Use one connection when the server does not prove range support.
- Use bounded parallel requests after valid range responses.
- Keep partial bytes and per-chunk state after an interruption.
- Reject inconsistent remote identity, size, and range responses.
- Check the exact size and catalog SHA-256 before finalization.
- Delete a partial model when final SHA-256 verification fails.
- Reject symbolic links and Windows reparse points in the download path.
- Accept network catalog changes only after Ed25519 verification.
- Fall back to the signed cache or embedded catalog when necessary.

The optional Hugging Face token supports authenticated Hub requests.
The token also supports gated repositories accepted by the account.

Localmotive stores the token in Windows Credential Manager.
The service name is `Localmotive HF`.

Localmotive sends the token in an `Authorization` header.
Localmotive does not put the token in a download URL or process argument.

### AI Tune

AI Tune asks a cloud model to propose throughput settings.
Localmotive measures each accepted proposal on the local computer.

The user can replace the saved profile with the best measured profile.

The application includes configurations for these providers:

- OpenRouter
- Anthropic
- OpenAI
- Google Gemini
- DeepSeek
- xAI

OpenRouter supports browser sign-in with OAuth PKCE or a pasted API key.
The other provider interfaces accept pasted API keys.

Localmotive stores cloud credentials in Windows Credential Manager.
The service name is `Localmotive`.

The frontend receives only credential status and a masked suffix.

The cloud advisor can change only fields in the tuning whitelist.
The whitelist excludes identity, network, path, and security fields.

Each trial starts `llama-server` and waits for `/health`.
Each successful launch is measured through `/completion`.
Localmotive then stops the trial process.

## Network and data behavior

### Runtime network access

Localmotive reads release metadata from the GitHub API.
Localmotive downloads selected official runtime archives from GitHub.

### Model catalog

Localmotive reads the catalog and signature from `raw.githubusercontent.com`.
Localmotive downloads selected model bytes from Hugging Face redirects.

If configured, the Hugging Face authorization header starts at the Hub URL.

### AI Tune network access

Localmotive sends a structured tuning brief to the selected cloud provider.

The brief includes these values:

- Hardware facts and system RAM size
- GGUF metadata
- Runtime capabilities
- Companion paths
- The baseline profile
- Trial commands
- Trial results and errors

AI Tune does not upload GGUF model bytes.

AI Tune does send local runtime, model, and companion paths.
AI Tune also sends profile values and measured trial evidence.

Do not use AI Tune if this metadata must remain local.

The selected provider can apply usage fees, quotas, retention policies,
and service terms.

### OpenRouter sign-in

OpenRouter sign-in opens `openrouter.ai` in the browser.
The OAuth PKCE flow returns through a temporary loopback callback.

### Local inference

Localmotive sends health checks and benchmarks to the configured server.
The WebUI communicates with the local `llama-server` endpoint.

## Local storage

Localmotive stores these values in the application WebView's local storage:

- Model folder
- Selected runtime path
- Selected cloud provider and advisor model
- Saved launch profiles
- Saved benchmark results
- Saved tuning reports

Localmotive does not encrypt these local-storage values.

Server logs use the operating system temporary directory.
The server log subdirectory is `localmotive`.

Cloud credentials and the Hugging Face token do not use local storage.
They use Windows Credential Manager.

## Security boundaries

- New profiles bind to `127.0.0.1` by default.
- Non-loopback profiles require an API key file.
- Non-loopback profiles require restricted CORS origins.
- API secrets use `--api-key-file` instead of a literal key argument.
- Localmotive passes process arguments without a shell command string.
- Child processes use `CREATE_NO_WINDOW` on Windows.
- Runtime and model paths must identify files before server launch.
- Catalog downloads must match the validated catalog.

Advanced extra arguments remain an expert interface.

Localmotive does not provide a permissions workflow for shell, file,
or agent options supplied through extra arguments.

Review the generated command before launch.

## Supported platforms

Windows x64 has been qualification-tested on AMD Zen 5 and NVIDIA Blackwell (RTX 50-series class), including clean-account NSIS/MSI install, launch, uninstall, and update-from-0.4.0 checks in Windows Sandbox when release evidence is present for that version.

The validated host uses Ryzen 9 9950X3D and GeForce RTX 5090.

A support claim applies to one version only when release evidence is present for that version.

Night jobs run on self-hosted runner `DESKTOP-HPTF57N-zen5-blackwell` during 01:00-06:00 Asia/Dubai.

Other Windows hardware may work but remains untested and unsupported until packaged attestations exist.

macOS remains out of scope for this matrix.

Code signing status is DEFERRED_BY_OWNER: Authenticode is postponed until the project is more mature (owner order 2026-09-09). No paid cert. SignPath stays pending or ignored and blocks no gate.

Windows installers remain honestly unsigned with disclosure. Windows SmartScreen can show a warning when you start an unsigned file. Unsigned artifacts ship as full releases, not GitHub Pre-releases, so the current tip stays visible as Latest.

## Current limitations

- The current application release provides Windows x64 artifacts only.
- Windows 10 and most hardware classes do not have L4 product evidence.
- [Microsoft ended normal Windows 10 support on October 14, 2025](https://support.microsoft.com/en-us/help/3207828); use an applicable supported servicing or ESU policy.
- Missing P0 rows are disclosed and do not receive support claims.
- This repository does not publish an ARM64 Localmotive application or runtime option.
- Localmotive supervises one model server at a time.
- Windows child processes enter a kill-on-close Job Object before execution resumes.
- Stop and cancellation terminate the complete contained process tree.
- Catalog downloads support single-file GGUF entries only.
- Inventory scanning supports split GGUF files already on disk.
- Application updates require a newer manual installation or executable.
- Localmotive 0.4.1 ships unsigned under the deferred-signing exception: no Authenticode signatures exist, so no signature match can be claimed. Verify the published SHA-256 checksums before use. SmartScreen can warn on unsigned files.
- Signing boxes are deferred, not green, and block no gate.

## Troubleshooting

### SmartScreen shows a warning

Localmotive 0.4.1 ships unsigned under the deferred-signing exception: there is no Authenticode signer to verify. Verify the published SHA-256 checksums before use.

SmartScreen can warn on unsigned files.

### A model is incomplete

Place every numbered shard under the selected model folder.
Scan the inventory again.

### A gated model download fails

Accept the repository license on Hugging Face.
Add a Hugging Face read token with access to the repository.

### A download stops

Select **Download** again to reuse valid saved chunks.
Localmotive restarts when the saved remote identity no longer matches.

### The server does not start

Inspect the application notice and server log.
Confirm that the runtime, model, companion, and key-file paths still exist.

## Build from source

Install the [Tauri prerequisites for Windows][tauri-prerequisites].

Use Node.js 20 with npm and the stable Rust toolchain.

Install the Microsoft C++ Build Tools and WebView2 requirements described
by the linked Tauri prerequisites.

```bash
git clone https://github.com/sato942/localmotive.git
cd localmotive
npm ci
npm run tauri dev
```

Build the application and Windows bundles:

```bash
npm run tauri build
```

The Windows x64 build produces these local artifacts:

- `src-tauri/target/release/localmotive.exe`
- `src-tauri/target/release/bundle/msi/Localmotive_<version>_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/Localmotive_<version>_x64-setup.exe`

## Test and verify

Run the frontend, catalog, and production build checks:

```bash
npm run check
```

Run the Rust checks:

```bash
cd src-tauri
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
RUSTDOCFLAGS='-D warnings' cargo test --locked --doc
```

GitHub Actions builds the packaged application.
The workflow starts the portable executable for a startup smoke test.

The smoke test does not prove model or GPU compatibility.
The smoke test does not test installer behavior or all Windows versions.

## Architecture

```text
src/                      React and TypeScript interface
src-tauri/src/lib.rs      Tauri commands and application state
src-tauri/src/core.rs     Models, profiles, runtimes, and benchmarks
src-tauri/src/runtime.rs  Hardware and managed runtime installation
src-tauri/src/catalog.rs  Signed catalog and Hugging Face token
src-tauri/src/download.rs Resumable model downloader and path controls
src-tauri/src/gguf.rs     Bounded GGUF metadata reader
src-tauri/src/tune.rs     Tuning whitelist and measured loop
src-tauri/src/cloud.rs    Cloud providers, OAuth, and credentials
src-tauri/src/proc.rs     Windows child-process construction
catalog/                  Curated model catalog and signature
scripts/                  Catalog and packaged-application checks
```

Rust owns filesystem, network, credential, process, and measurement work.
React owns presentation and local interface state.

Tauri commands form the boundary between both parts.

## Contributing

Read [AGENTS.md](AGENTS.md) before changing the repository.

Add a regression test for each behavioral change.
Run the complete check suite before you open a pull request.

Use the [issue tracker][issues] for defects and feature proposals.

Do not include credentials or private model paths in a public issue.
Remove sensitive content from logs before you attach the logs.

See these project records for more detail:

- [CHANGELOG.md](CHANGELOG.md) contains the release history.
- [catalog/README.md](catalog/README.md) contains catalog rules.
- [docs/DESIGN.md](docs/DESIGN.md) contains the interface specification.
- [docs/OPTION_MAP.md](docs/OPTION_MAP.md) explains profile grouping.
- [docs/LLAMA-SERVER-README.md](docs/LLAMA-SERVER-README.md) is imported reference.

## License and affiliation

Localmotive is available under the [MIT License](LICENSE).

Localmotive is not affiliated with or endorsed by these organizations:

- `ggml-org`
- Hugging Face
- The listed cloud providers

`llama.cpp`, downloaded models, and cloud services have separate terms.

Review the applicable license and model card before use or redistribution.

[issues]: https://github.com/sato942/localmotive/issues
[releases]: https://github.com/sato942/localmotive/releases/latest
[tauri-prerequisites]: https://v2.tauri.app/start/prerequisites/
