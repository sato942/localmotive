# Runtime Manager

Localmotive does not bundle llama.cpp. It obtains official binaries at first run or accepts a user-supplied executable.

## Release discovery

The app queries:

`https://api.github.com/repos/ggml-org/llama.cpp/releases?per_page=20`

Stable semantic releases may contain only release notes. Localmotive therefore chooses the newest non-draft release in that response that actually publishes Windows runtime ZIP assets.

## Windows recommendation policy

| Detected platform | Recommendation | Visible fallbacks |
|---|---|---|
| NVIDIA with driver CUDA 13+ | Newest compatible CUDA 13 asset | CUDA 12, Vulkan, CPU |
| NVIDIA with older compatible driver | Newest CUDA asset not exceeding driver CUDA major | Vulkan, CPU |
| AMD Radeon | ROCm/HIP Windows asset | Vulkan, CPU |
| Intel GPU | SYCL Windows asset | OpenVINO, Vulkan, CPU |
| Other Windows GPU | Vulkan | CPU |
| No supported GPU | CPU | Vulkan when published |
| Windows ARM64 | Matching ARM64 assets only | CPU and vendor assets when published |

CUDA installation always pairs the main `llama-...-win-cuda-...zip` archive with the matching `cudart-llama-...zip` archive.

## Installation location

`%LOCALAPPDATA%\Localmotive\runtimes\<release-tag>\<backend-variant>`

CUDA variants include their toolkit version in the folder name (for example `cuda-13.3`), so CUDA 12 and CUDA 13 builds can coexist.

Each install has `runtime.json` recording the release, backend, source asset, companion asset, and relative path to `llama-server.exe`.

## Integrity and failure handling

1. Download over HTTPS from the official GitHub asset URL.
2. Verify exact byte size when GitHub publishes it.
3. Verify SHA-256 when the API publishes a `sha256:` digest.
4. Extract into a staging directory.
5. Reject ZIP entries whose enclosed path would escape staging.
6. Require `llama-server.exe` to exist after extraction.
7. Write the manifest.
8. Atomically rename staging into the versioned runtime folder.
9. Remove staging on any failure.

Existing versioned installs are reused rather than downloaded again. Installing a newer release does not delete older runtimes.

## User-supplied runtimes

The native file picker accepts an existing `llama-server.exe`. Localmotive executes `--version` and `--help`, records its capabilities, labels it as user-supplied, and filters profile arguments to flags advertised by that build.

A user-supplied runtime is not claimed to be downloaded or hash-verified by Localmotive.
