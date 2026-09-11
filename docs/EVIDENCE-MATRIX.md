# Evidence matrix

Version → what was actually exercised. A cell says PASS only where release
evidence for that exact version exists; everything else stays UNKNOWN or NOT
RUN. This file is the index for the README support statement and is updated
with each release. The authoritative per-item evidence lives in
`docs/history/TODO-0.6.md` and the release assets named below.

Legend: **PASS** evidence present · **FAIL** evidence present and failing ·
**UNKNOWN** not established for that version · **NOT RUN** deliberately not
executed.

## Hosts

| Host | Role |
|---|---|
| Ryzen 9 9950X3D, GeForce RTX 5090, Windows 11 26100 | Primary validation and CI self-hosted runner `DESKTOP-HPTF57N-zen5-blackwell` |
| Windows Sandbox (clean account) | Installer lifecycle isolation |

## Versions

| Version | Channel | CPU packaged lifecycle | Accelerator (CUDA) packaged | Clean-account Sandbox | Notes / evidence |
|---|---|---|---|---|---|
| 0.4.0 | Public release | PASS (as published) | UNKNOWN | UNKNOWN | Its release notes labeled three L2/PARTIAL rows "Supported"; the correction is explained in the 0.4.1 notes and this repository. Binaries stay immutable. |
| 0.4.1 | Public release | PASS (as published) | UNKNOWN | UNKNOWN | See `docs/history/TODO-0.4.1.md` for the frozen record. |
| 0.5.0 | Public release | PASS (release workflow `package-smoke` + packaged matrix on this host) | UNKNOWN | FAIL (failed before installation; recorded in the release evidence) | Health evidence is CPU only. Release notes disclose broad L4 limitations. See the ship ledger in `docs/history/TODO-0.5.md`. |
| 0.6.0 | In development | In progress (tracked in `docs/history/TODO-0.6.md`) | NOT RUN | NOT RUN | No support claim before the 0.6 evidence exists. |

## What each column means

- **CPU packaged lifecycle**: the packaged binary built by the release
  workflow, the MSI/NSIS install, launch, uninstall, and packaged screens
  exercised over CDP on this host.
- **Accelerator (CUDA) packaged**: a lifecycle run where a CUDA runtime
  serves a model end to end. CPU health runs, flag inspection, or "the UI
  offers CUDA" are not evidence for this column.
- **Clean-account Sandbox**: the same lifecycle inside Windows Sandbox with a
  clean account, proving no dependency on the maintainer's machine state.

## Reading rules

- A green row here never upgrades to a broader claim: one exact attestation
  matches one qualification key.
- The catalog signature authenticates the catalog document; it does not
  authenticate application installers (see `SECURITY.md`).
- Releases ship unsigned with disclosure; verify published SHA-256 checksums.

## Copied commands and manifest paths (audit S-14)

- The provisional command preview names its shell: a paste-ready PowerShell
  line (single-quoted, so every metacharacter and Unicode value is literal)
  and, when every value is expressible, a cmd.exe line; a value containing
  `%` refuses the cmd.exe form with an explanatory notice. A JSON `argv`
  array accompanies both and is lossless for any wrapper. The application
  itself always launches llama-server through an argument array, never a
  shell string.
- Raw local run manifests are working files for this machine and still
  include explicit local paths (model, projector, draft, LoRA, template and
  key/certificate files). They are **not** publication artifacts: share and
  export bundles run the separate redaction path, and the effective-argument
  identity used for calibration is sanitized (`[model]`, `[draft-model]`,
  `[lora]`, `[configured]`) for every path-bearing flag, including the short
  `-md` draft form.

## Local calibration history lifecycle (audit S-16)

- **Storage**: one JSON record per file under `<app data>/calibration/anchors`
  and `.../models`; each record is size-bounded (64 KiB) and the directory
  enumeration refuses more than 10 000 records.
- **Retention**: persisting prunes the oldest recognized records beyond 4 000
  per category; only `.json` records in those directories are candidates, and
  everything else is untouched. Pruned records are raw local history: exports
  and share bundles that reference a measurement carry their own copy, so
  retention never silently invalidates an exported artifact.
- **Cleanup**: "Clear local history" in the evidence panel removes every
  stored calibration record and reports the count; quarantined files are kept
  because they are the diagnostic evidence of earlier failures.
- **Corruption**: a corrupt, oversized, schema-invalid or unreadable record is
  moved to `.../calibration/quarantine/<name>.corrupt-<stamp>` and reported as
  a bounded load problem while the remaining compatible history keeps
  loading; one bad file never hides the rest.

## Rust–TypeScript and persisted-format contracts (audit S-18)

- **Wire contract**: `scripts/tests/fixtures/ipc-contract.json` is the shared
  authority for representative IPC payloads. The Rust test
  `s18_shared_ipc_contract_fixture_matches_rust_serialization` asserts its
  serialized values equal the fixture entries exactly (camelCase keys, enum
  spellings, nullable fields serialized as `null`), and
  `scripts/tests/ipc_contract.test.mjs` asserts the same entries match the
  `model.ts` consumer expectations (exact key sets, JS types, camelCase only,
  documented enum spellings). Errors cross the IPC boundary as plain strings
  by contract — never as objects.
- **Persisted formats**: catalog documents carry `schemaVersion` (verified by
  the shared catalog contract), benchmark manifests carry `schema` and the
  execution-snapshot schema inside their keys, calibration anchors and models
  carry `schemaVersion` (current `RECORD_SCHEMA_VERSION = 1`), and profile
  records are normalized on read (`normalizeProfile`) with quarantine for
  unreadable shapes. Records missing the field load as version 1; a NEWER
  version is rejected with "rebuild it from current measurements with a newer
  Localmotive" instead of being misread. Unknown fields are ignored on
  purpose for forward compatibility, and serialization validation lives in
  the model/loader layer, never in rendering.
