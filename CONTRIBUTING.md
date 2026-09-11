# Contributing

Read `AGENTS.md` first: it is the authoritative project policy (tests are
mandatory, the runtime's `--help` is the source of truth, secrets never touch
disk in the clear, and no number is invented).

## Setup

1. Install Node `^20.19.0 || >=22.12.0` and the Rust toolchain pinned in
   `rust-toolchain.toml` (1.98.1).
2. `npm install`, then `npm run tauri dev` for the hot-reloading app.

## The check suite

Run all of it before you open a pull request:

```bash
npx tsc --noEmit -p tsconfig.json
npm test
npm run build
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Behavioural changes ship with a regression test, written failing first. UI
changes run the design detector and keep `docs/DESIGN.md` at zero lint
errors. `scripts/tests/release-gates.test.mjs` guards the release contracts;
extend it when you touch workflows, gates, or release documentation.

## Pull requests

Pull requests run `pr-check` on hosted runners (types, tests, build, gates).
They never receive secrets, and untrusted code never runs on the maintainer's
self-hosted machine. Keep changes focused; one idea per pull request; say what
you ran and what it returned.

## Maintenance and recovery (maintainer notes)

- **Runner isolation**: start the self-hosted runner only through
  `localmotive-control\start-runner.ps1` (or its watchdog) so the dedicated
  `CARGO_HOME`/`RUSTUP_HOME` directories apply. Never clean the owner's
  personal `~/.cargo`.
- **Catalog signing key**: the Ed25519 private key lives only in the
  `CATALOG_SIGNING_KEY_PEM` repository secret used by the catalog sign job.
  To rotate, generate a new keypair, update the embedded public key and the
  secret together in one change, run the catalog workflow, and verify the
  served `.sig` against the committed catalog. Signing is limited to the
  catalog document and never authenticates installers.
- **Release artifacts**: candidates are retained for 14 days; the public
  readback evidence is retained for 90 days; released checksums and evidence
  JSON persist as release assets. Keep release evidence for the lifetime of
  the supported version line and never rewrite published tags.
- **Backups**: back up `%LOCALAPPDATA%\Localmotive` (installs and SQLite
  mirror), the runner's `cargo-home`/`rustup-home`, and the signing secret
  out of band before moving machines.
