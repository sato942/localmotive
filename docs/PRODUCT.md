# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

delegated: Tauri 2 desktop shell, Rust control plane, React and TypeScript interface, SQLite-ready local persistence. The user delegated implementation choices.

## Users

Windows users running local GGUF language models on CPU, NVIDIA, AMD, or Intel hardware. Their primary job is to select a logical model, obtain a compatible llama.cpp runtime, choose an acceleration strategy, start or stop an OpenAI-compatible server, and compare throughput without reconstructing commands by hand.

## Product Purpose

Localmotive turns a heterogeneous GGUF collection into validated logical models and repeatable launch profiles. Success means specialized paths—baseline, n-gram, MTP, DFlash, DSpark, EAGLE-3 and multimodal companions—remain explicit, inspectable, and benchmarkable rather than hidden behind generic presets.

## Positioning

Unlike chat-first local-model apps, Localmotive treats runtime build capability, target/companion compatibility, exact command provenance, and measured profile performance as first-class product data.

## Operating Context

Localmotive 0.4.1 targets Windows 10 and Windows 11 x64.

The target scope is not a tested compatibility claim.

The approved catalog exposes reviewed x64 CPU, CUDA, ROCm, SYCL, Vulkan, and OpenVINO archives.

Each row remains unvalidated until one exact L4 product attestation matches its complete qualification key.

ARM64 assets remain dormant and are not part of the 0.4.1 product scope.

The app preserves support for user-supplied executables required by experimental model families.

The built-in llama-server WebUI remains the chat surface after the control plane starts a server.

## Capabilities and Constraints

- GGUF only.
- Detect Windows architecture and each GPU adapter without treating vendor names as compatibility proof.
- Recommend an accelerator only after one exact L4 compatibility record matches.
- Recommend CPU with an explicit reason when no exact accelerator record matches.
- Download and verify official runtimes into versioned application-managed storage; pair CUDA binaries with matching cudart archives.
- Scan a user-selected folder recursively while grouping split shards and recognizing draft, MTP, DSpark, DFlash, EAGLE-3 and mmproj companions.
- Register multiple llama-server executables and detect capabilities from `--version`, `--help`, and `--list-devices`.
- Save profiles containing target, companions, network identity, memory/offload settings, speculative strategy, and raw extra arguments.
- Validate paths, shard completeness, ports, aliases, and runtime flags before launch.
- Supervise only processes started by the app; never kill unrelated processes.
- Bind to loopback by default and support API-key configuration.
- Benchmark deterministic workloads with warmup and repeated measurements; retain exact commands and distinguish near ties.
- Never infer an unsupported runtime flag from a filename; runtime `--help` is authoritative and unsupported profile flags are omitted from the exact command.

## Evidence on Hand

- `LLAMA-SERVER-README.md` is the local option and endpoint reference.
- The official GitHub releases API publishes Windows runtime asset names, sizes, and SHA-256 digests where available.
- The test suite includes representative NVIDIA, AMD, Intel, x64, and dormant ARM64 catalog cases.
- Test fixtures and upstream checks do not establish product support.
- No commercial claims, external customers, or production-availability claims exist and none should be invented.

## Product Principles

1. Exact commands over hidden magic.
2. Capabilities come from the selected runtime, not assumptions.
3. Model companions are explicit relationships, not loose filename guesses.
4. Baselines and repeated measurements precede optimization claims.
5. Safe local defaults and owned process lifecycles.

## Accessibility & Inclusion

Keyboard-operable controls, visible focus, non-color status labels, reduced-motion support, and readable data density are required.
