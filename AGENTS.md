# Localmotive engineering rules

## Product and ownership

Localmotive is a Windows desktop control plane for GGUF inference.
Keep runtime management, model discovery/downloads, launch profiles, measurement,
and AI tuning. Use `llama-server` for inference and its WebUI for chat.
Do not reimplement the inference engine.

Rust owns validation, runtime capabilities, files, downloads, credentials, and
processes. React owns presentation. Put pure frontend decisions in `src/model.ts`.
Keep IPC payloads typed and bounded. Do not turn frontend text into a shell command.

Source map:

- `src/App.tsx`, `src/screens/`: application state and presentation.
- `src/model.ts`, `src/model.test.ts`: frontend types and pure decision tests.
- `src/App.css`, `docs/DESIGN.md`: design system and normative design rules.
- `src-tauri/src/lib.rs`, `*_service.rs`: Tauri commands and operation ownership.
- `core.rs`, `runtime.rs`, `artifact.rs`, `gguf.rs`: profiles, runtime and file facts.
- `catalog.rs`, `catalog_db.rs`, `download.rs`: signed catalog, local mirror, downloads.
- `local_client.rs`, `health.rs`, `proc.rs`, `log_sink.rs`: transport and process safety.
- `measurement.rs`, `evidence.rs`, `tune.rs`, `cloud.rs`: measurement and tuning.
- `scripts/`: maintained verifiers and release tooling.

Read the relevant source, callers, tests, and manifests before editing.
Prefer deletion and existing code over another helper, runner, or policy layer.
Preserve user files and compatibility unless the approved scope changes them.

## Non-negotiable protections

- Derive supported flags from the selected runtime's real `--help` output.
  Filter unsupported generated flags before launch. Do not infer support from names.
- Group split GGUF shards into one model. Pass only the first shard to `-m`.
  Keep companion selection deterministic by role, quantization, and name.
- Keep cloud and Hugging Face secrets in Windows Credential Manager through `keyring`.
  Preserve service names `Localmotive` and `Localmotive HF`.
  Never put secrets in files, logs, browser storage, command lines, or frontend state.
  Show only credential status and a masked suffix.
- Construct child processes through `proc::hidden_command`.
  Retain process-tree ownership, bounded output, cancellation, and cleanup.
  Never stop unrelated processes by name or port.
- Validate paths, reparse points, archive entries, download ranges, size, and digests.
  Retain atomic publication, partial-download recovery, and rollback.
  Lexical path normalization does not prove filesystem containment.
- Report measured facts. Show unknown when evidence is unavailable.
  Do not invent models, hardware support, benchmark results, or verification records.

## Tests and verification

Every behavioral change needs a regression test.
For a bug, write the test first and observe the intended failure before fixing it.
Explain the real failure case in the test. Verify that important tests can fail
with a controlled mutation, then restore the implementation and rerun.

Use real temporary files and sockets. Keep fixtures isolated under parallel runs.
Mock external boundaries through the existing interfaces.
Assert observable results, not incidental comments or implementation spelling.
Cover malformed input, rejection, interruption, and relevant boundary values.
Do not delete a failing safety test or suppress warnings to obtain a pass.
Remove obsolete tests only when the approved behavior is removed or retained
coverage demonstrably replaces them.
Restore a single file from a copied backup, never `git checkout -- <file>`
in a dirty tree. A checkout reverts unrelated baseline work and breaks the build.
Count a harness or reader failure as a tooling failure, never as a product
RED or GREEN. Fix the reader and rerun.

Rust tests belong beside their implementation. Pure TypeScript tests belong in
`src/model.test.ts`. Component tests belong beside their components.
Packaged checks use the existing CDP matrix. Extend that matrix instead of adding
another version-named verifier for every release.

Run from the repository root in Git Bash:

```bash
npm ci
npm run check < /dev/null
npm audit --audit-level=moderate < /dev/null
node scripts/verify_versions.mjs < /dev/null
node scripts/verify_workflow_pins.mjs < /dev/null
node scripts/verify_workflow_gates.mjs < /dev/null
node scripts/verify_workflow_syntax.mjs < /dev/null
pwsh -NoProfile -File scripts/tests/verify_cleanup_matrix.ps1
cd src-tauri
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo test --locked
```

`npm run check` includes script tests, Vitest, repository checks, TypeScript,
and the production frontend build. `cargo test` includes documentation tests.
Do not run the same suite twice merely to satisfy a workflow layout.
Keep independent checks after artifact transfer and publication.

Use `npm run tauri dev` for development. Verify user-visible changes on the
packaged executable, not only the development server:

```bash
npm run tauri build < /dev/null
```

The build produces `src-tauri/target/release/localmotive.exe`, MSI, and NSIS files.
For the full packaged matrix, use a clean checkout and its newly built candidate.
Choose an unused debugger port. Run this PowerShell command from the repository root:

```powershell
$env:RUNNER_TEMP = $env:TEMP
$version = (Get-Content package.json | ConvertFrom-Json).version
$revision = git rev-parse HEAD
pwsh -NoProfile -File scripts/verify_packaged_matrix.ps1 `
  -Portable src-tauri/target/release/localmotive.exe `
  -Version $version -ResolvedSha $revision -CdpPort 10013
```

The matrix owns its application process, isolated data, fixtures, and cleanup.
Assert values. Use screenshots for design review only.
Report exact commands and observed results. Keep failed or unexecuted checks visible.
A successful build is not installer-upgrade evidence or release approval.

## Code and design conventions

- Follow `cargo fmt` and warning-free Clippy. Avoid `unwrap()` on user, file,
  or network input. Tests may use it.
- Return `Result<T, String>` at Tauri boundaries. Name the problem and recovery.
  Document why public rule functions exist.
- Mirror Rust IPC structs in TypeScript with the same camelCase field names.
  Do not use `any`. Justify new dependencies.
- Read `docs/DESIGN.md` before editing CSS. Keep square corners except lamps.
  Do not use `box-shadow`. Reserve green for valid/running, amber for incomplete,
  and red for stop. Keep signal colors below about 10% of a screen.
- Keep keyboard access, visible focus, labels, reduced motion, and words on status tags.
  Keep mobile targets at least 44 px and 110 px clearance above fixed navigation.
- After UI changes, run the design detector and fix its findings.
  Add new components to `docs/DESIGN.md` and keep `designmd lint` error-free.

## CI and publication

Untrusted pull requests run only on the ephemeral `windows-latest` runner with
read-only permissions and no secrets. Trusted main pushes use the self-hosted
runner. Preserve the required `pr-check` name, main-branch protection, and
force-push/deletion restrictions. Workflow `needs` does not replace branch rules.

Start the self-hosted runner through `localmotive-control/start-runner.ps1`.
Keep its `CARGO_HOME` and `RUSTUP_HOME` separate from the owner's toolchains.
Never clean the owner's personal Cargo or Rustup directories.

Resolve a release tag to one full SHA. Build and qualify that source only.
Do not retarget used version tags. Keep version fields and lockfiles consistent
through `scripts/verify_versions.mjs`. Add user-facing changes to `CHANGELOG.md`.

`release.yml` verifies and retains candidate bytes; it does not publish.
`release-promote.yml` requires explicit authorization and validates the downloaded
qualified bundle before publishing those same bytes. Never rebuild during promotion.
Retain artifact inventory, checksums, native lifecycle/preservation evidence,
and public asset readback. Do not replace current-candidate evidence with old records.
Disclose unsigned artifacts and SmartScreen limitations in every release.
No signing step exists and none is pursued. Do not publish without explicit
authority for that action.

## Catalog and project records

The signed catalog comes from repository-hosted JSON. Model downloads come from
Hugging Face. The application also keeps a local SQLite mirror and user overrides;
that mirror does not authorize downloads. Follow `catalog/README.md` for schema
changes, signing, backward compatibility, and curation.

Use `TODO.md` as the current tracker. Use one tracker only and do not version
its name. Never use kanban boards, kanban tools, or kanban task protocols for
this project. Consult
`docs/history/localmotive-comprehensive-audit.md` for the traced findings.
Record actual regression and verification evidence in the current ledger.
Do not treat checkboxes as proof. Do not create another release tracker.
Keep historical trackers and accepted evidence frozen.

For model-fit or measurement work, read `research/measuring/README.md`, then
`research/measuring/SYNTHESIS.md`, then the relevant source and tests.
Keep `research/` ignored and unmodified. Vitest must stay restricted to
`src/**/*.test.{ts,tsx}`.

## Windows tool notes

Use forward-slash native paths such as `C:/...` for native tools from Git Bash.
Run Node with `< /dev/null` on this host to avoid `stdin is not a tty`.
The per-file Rust linter can assume Rust 2015; use the edition-aware Cargo checks.
Before creating a repository, confirm the GitHub identity with
`gh api user --jq .login`. Account labels can be stale.
