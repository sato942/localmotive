# Runtime Manager

> Scope: the runtime policy (pinned official builds, per-version installs, `--help` capability gating) is version-independent. Release-specific counts, dates and contract numbers quoted below are historical 0.4.1 evidence; current identities live in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`.

Localmotive does not bundle llama.cpp. It obtains official binaries at first run or accepts a user-supplied executable.

## Approved release identity

Localmotive 0.4.1 approves only `ggml-org/llama.cpp` tag `b10816`.

The approved commit is `427291b5b34cd914a31b3fd3b61a68f6184f4b9f`.

The app queries the exact release endpoint for that tag.

The app rejects another tag, commit, release identifier, asset name, size, digest, or URL.

The frontend sends only an immutable install key and an optional selected adapter identifier.

## Windows recommendation policy

Localmotive recommends an accelerator only after one exact L4 compatibility record matches.

The key contains:

- Windows build;
- architecture;
- stable adapter compatibility identifier;
- adapter driver;
- required firmware, or explicit `not-applicable`;
- backend;
- install key;
- runtime commit;
- asset name and SHA-256;
- attestation and expiry identity.

Vendor names are discovery hints only.

If no exact record matches, Localmotive recommends CPU and explains the fallback.

If several adapters exist, select one adapter before an accelerator can match.

CUDA installation always pairs the main `llama-...-win-cuda-...zip` archive with the matching `cudart-llama-...zip` archive.

## Installation location

`%LOCALAPPDATA%\Localmotive\runtimes\<release-tag>\<backend-variant>`

CUDA variants include their toolkit version in the folder name (for example `cuda-13.3`), so CUDA 12 and CUDA 13 builds can coexist.

Each install has `runtime.json` recording the backend-owned immutable identity.

Each active install key also binds one compiled exact-content manifest.

## Product support contract (0.4.1)

Localmotive targets Windows 10 and Windows 11 x64.

The target scope is not a tested compatibility claim.

Every 0.4.1 catalog row remains `Not validated` unless an exact L4 compatibility record exists.

The approved manifest currently contains no L4 compatibility records.

Direct `llama.cpp` evidence has an L2 ceiling.

Upstream CI evidence does not establish Localmotive product support.

The interface shows blocking upstream jobs and public evidence URLs without linking local research paths.

## Identity and completeness

Managed runtime acceptance compares every regular file against the compiled content manifest.

The comparison rejects missing files, additional files, changed sizes, changed SHA-256 values, links, and reparse-point ancestors.

Localmotive verifies fresh installs, reused installs, and managed runtimes before launch.

Every content manifest requires `llama-server.exe`, `llama-cli.exe`, and `llama-bench.exe`.

## Integrity and failure handling

1. Resolve every artifact field from the compiled approved manifest.
2. Probe with a bounded HTTP client.
3. Bind resume state to URL, size, SHA-256, ETag presence, and last-modified value.
4. Limit each request to eight MiB.
5. Retry at most twice after the initial request.
6. Restart from byte zero if a server ignores a valid range request.
7. Verify exact archive size and SHA-256 before extraction.
8. Extract into a unique staging directory.
9. Reject traversal, links, duplicate paths, and archive resource-limit violations.
10. Verify the complete extracted inventory against the compiled content manifest.
11. Write trusted runtime metadata.
12. Replace the destination atomically on one volume.
13. Remove staging on cancellation or failure.

Existing versioned installs are reused rather than downloaded again. Installing a newer release does not delete older runtimes.

On Windows, `process-wrap` 10.0.0 creates each managed child suspended.

The child enters a kill-on-close Job Object before its primary thread resumes.

Cancellation terminates and waits for the complete contained process tree.

## Windows signing

The ordinary `npm run tauri build` command produces unsigned test candidates.

> Policy update (0.5 and later): public releases ship **unsigned with an explicit disclosure** while code signing is deferred; see the release policy in `README.md` and the release notes in `CHANGELOG.md`. The signing requirements below document the procedure to use **if and when** signing is resumed; they are no longer a precondition for a release candidate. Historical evidence in `docs/history/` is frozen and is not rewritten.

The release candidate must use `npm run tauri:build:signed`.

The signed build requires an approved `CurrentUser\My` certificate thumbprint.

The signed build also requires an approved HTTPS timestamp endpoint.

`scripts/sign-windows.ps1` checks code-signing usage, private-key access, expiry, signing success, and Authenticode verification.

Do not record or export private-key material.

## User-supplied runtimes

The native file picker accepts an existing `llama-server.exe`. Localmotive executes `--version` and `--help`, records its capabilities, labels it as user-supplied, and filters profile arguments to flags advertised by that build.

A user-supplied runtime is not claimed to be downloaded or hash-verified by Localmotive.
