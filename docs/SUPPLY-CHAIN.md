# Supply chain

What protects a release, and what does not. This documents the accepted
posture after the 0.6 remediation (audit GH-08); it makes no claim that
checksums provide publisher authenticity.

## Protections in place

- Actions are SHA-pinned and verified by `scripts/verify_workflow_pins.mjs`.
- `npm ci` and `cargo --locked` install from lockfiles only.
- The toolchain is pinned (`rust-toolchain.toml`, `rust-version`,
  `package.json` `engines`/`packageManager`).
- Release tags resolve once to a full commit; every later job checks out that
  exact revision (`resolve` job), and tag updates/deletions are blocked by a
  ruleset.
- The release workflow produces the MSI, NSIS installer, portable exe, and a
  SHA-256 manifest; published assets are read back and validated.
- The frontend dependency SBOM (`npm sbom --sbom-format cyclonedx`) is
  generated in the package job and retained 90 days as the `sbom-<version>`
  workflow artifact. The Rust side's exact dependency graph is the committed
  `Cargo.lock`.
- The catalog Ed25519 key is isolated to the catalog sign job and signs only
  the catalog document; it does not authenticate application installers.

## Accepted limitations

- **Unsigned installers.** Authenticode is deferred by owner decision; every
  artifact is labeled honestly unsigned. SHA-256 checksums prove integrity
  against corruption, not identity against a compromised uploader.
- **No SLSA/in-toto build attestation yet.** The repository's attestation
  JSON records are evidence documents, not cryptographically signed
  provenance. Adding `actions/attest-build-provenance` requires a release-run
  verification on the owner's machine; it is tracked as an owner decision in
  the 0.6 tracker rather than landed unverified in the release workflow.
- **Shared build host.** Build, hardware qualification, and publication run
  on the same self-hosted Windows machine. The `CARGO_HOME`/`RUSTUP_HOME`
  isolation protects toolchain directories, not the whole environment.
- **Retention.** Release candidates are retained 14 days, public readback 90
  days, and the SBOM artifact 90 days; released checksums and evidence JSON
  persist as release assets. Keep release evidence for the supported version
  line.

## Key rotation and recovery

See `CONTRIBUTING.md` (Maintenance and recovery): catalog key rotation is a
single paired change (embedded public key + repository secret) followed by
one catalog workflow run; runner toolchain directories and the signing secret
are backed up out of band before moving machines.
