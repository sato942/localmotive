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

## Product support contract (0.4)

Localmotive targets Windows x64. Runtime availability depends on exact processor, accelerator, driver, and llama.cpp artifact. Only configurations in the release compatibility table carry validation.

| Level | Scope (OS, arch, device, driver, backend, revision) | Evidence |
|---|---|---|
| Supported | Windows 11, x64, AMD Ryzen 9 9950X3D, CPU, b10796 | research/0.4/evidence/attestations/local-windows-x64-cpu.json |
| Supported | Windows 11, x64, NVIDIA GeForce RTX 5090, driver 610.74, CUDA, b10796 | research/0.4/evidence/attestations/local-windows-x64-cuda.json |
| Supported | Windows 11, x64, NVIDIA GeForce RTX 5090, driver 610.74, Vulkan, b10796 | research/0.4/evidence/attestations/local-windows-x64-vulkan.json |
| Not validated | Every other catalog entry (ROCm, SYCL, OpenVINO, CUDA 12.4, Arm64) | None, untested configuration |

Upstream capability is not product support: ROCm, SYCL, OpenVINO, CUDA 12.4, and Arm64 archives parse as catalog entries but ship no automatic preference and no Supported label until a compatibility-table row with Phase 3 evidence exists. The interface renders this through supportStatusForOption in src/model.ts (level plus full scope plus evidence path, or Not validated with null evidence). The managed runtime screen shows one SUPPORTED or NOT VALIDATED tag plus a scope line per option. A manifest-dll-mismatch identity shows MISMATCH and REINSTALL with a reinstall action before launch.

## Identity and completeness

describe_runtime in src-tauri/src/runtime.rs derives identity from evidence beside the executable, never from its filename: the managed runtime.json manifest when present, otherwise the shipped ggml and CUDA DLLs. A manifest that disagrees with the sibling DLLs (wrong backend, wrong CUDA major, or missing companion such as cudart64_<major>.dll) reports backend mismatch with source manifest-dll-mismatch. The completeness rule per backend (backend_dependencies_are_complete): CUDA needs ggml-cuda.dll plus the matching cudart64_<major>.dll; ROCm needs ggml-hip.dll or ggml-rocm.dll; SYCL needs ggml-sycl.dll; OpenVINO needs ggml-openvino.dll; Vulkan needs ggml-vulkan.dll; Arm64 OpenCL needs ggml-opencl.dll; CPU needs ggml-cpu or ggml.dll.

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
