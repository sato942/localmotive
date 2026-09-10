# AGENTS.md

Instructions for AI agents working on Localmotive. Read this before touching the
repository. It describes what the project is, how to change it safely, and the
rules that keep releases from shipping bugs.

Human contributors: everything here applies to you too.

---

## 1. What this project is

Localmotive is a Windows desktop control plane for local GGUF inference. It does
five jobs, and nothing else:

1. **Runtime management** — download official `llama.cpp` Windows builds from
   GitHub releases, verify them, and keep versioned installs side by side.
2. **Model discovery** — scan a folder for GGUF targets, group split shards into
   one logical model, and attach companion files (draft/speculation models,
   multimodal projectors).
3. **Launch profiles** — build the exact `llama-server` command line, validated
   against the selected runtime's real `--help` output.
4. **Measurement** — benchmark generation throughput through `/completion`.
5. **AI tuning** — let a cloud model propose settings that this PC measures.

It is a **control plane, not an inference engine**. All inference is
`llama-server`. If a feature would require reimplementing something llama.cpp
already does, it does not belong here.

### Architecture

```
docs/history/TODO.md                 v0.3 implementation strategy, status, and evidence ledger
src/                    React + TypeScript frontend (one screen per nav item)
  App.tsx               All screens; state lives here
  App.css               The entire design system, hand-written
  model.ts              Shared types + pure functions (profiles, state machines)
  model.test.ts         Vitest tests for those pure functions
src-tauri/src/
  lib.rs                Tauri commands — the only bridge to the frontend
  core.rs               Model scanning, launch profiles, validation, benchmarks
  runtime.rs            Hardware detection, GitHub releases, runtime installs
  gguf.rs               GGUF header reader (metadata only, never tensor data)
  tune.rs               AI tuning: brief, whitelist, proposal parsing, loop
  cloud.rs              Cloud providers, credential storage, OAuth PKCE
  catalog.rs            Curated HF catalog fetch + filtering
  download.rs           Resumable parallel HTTP downloader
  proc.rs               Child-process construction (CREATE_NO_WINDOW)
catalog/catalog.json    The curated model catalog (see section 7)
docs/                   Option map, runtime manager design, token exports
scripts/                Verification harnesses driven over CDP
```

**Rust owns truth. The frontend owns presentation.** Any logic that decides
*what is correct* — which flags are valid, which companion matches, whether a
port is free, whether a download is complete — belongs in Rust with a test.
The frontend may only decide *how it looks*.

---

## 2. The rules that matter most

### Rule 1 — Tests are not optional

**Every behavioural change ships with a test.** Not "when convenient", not
"for complex parts". Every time.

- Fixing a bug? Write the failing test *first*, watch it fail for the right
  reason, then fix it. A bug without a regression test will come back.
- Adding a function that makes a decision? Test it.
- Adding a UI-only style? No test needed — but if it changes *behaviour*
  (what is enabled, what is shown when), extract that decision into
  `model.ts` and test it there.

When you find yourself thinking "this is too simple to test", that is exactly
the code that breaks silently. Write the test; it takes thirty seconds.

**Where tests go:**

| What you changed | Test goes in | Run with |
|---|---|---|
| Rust logic | `#[cfg(test)] mod tests` in the same file | `cargo test` |
| Pure TS function | `src/model.test.ts` | `npm test` |
| Whole-app behaviour | `scripts/verify_*.mjs` (CDP) | manual, before release |

### Rule 2 — Never claim something works without running it

Do not write "this should now work" or "the build passes". Run it, read the
output, and report what actually happened. If a command fails, say so and fix
it. Fabricating results is the single worst thing you can do here.

For anything user-visible, verify on the **packaged binary**
(`npm run tauri build`), not the dev server. Several bugs in this project's
history only appeared in the packaged app.

### Rule 3 — The runtime's `--help` is the source of truth

Never assume a `llama-server` flag exists. `core::inspect_runtime` parses the
selected executable's real `--help` and `--version`; unsupported flags are
filtered out before launch. If you add a profile field, it must be gated by
that capability list. Users run patched and older builds.

### Rule 4 — Never invent data

No placeholder model names in the catalog, no fabricated benchmark figures, no
example paths that look real. If you cannot obtain a value, show "unknown" or
omit the row. This application's entire value is that its numbers are real.

### Rule 5 — Secrets never touch disk in the clear

Cloud and Hugging Face tokens live in **Windows Credential Manager** via the
`keyring` crate, under service names `Localmotive` / `Localmotive HF`. They are
never written to local storage, settings files, logs, or command lines, and
the UI shows only a masked suffix. If you add a provider, follow that pattern
exactly.

---

## 3. Development workflow

### v0.5 tracker

Read `docs/history/TODO-0.5.md` before starting v0.5 work. Treat `docs/history/TODO-0.5.md` as the
authoritative phase tracker, but never treat a checkbox as verification evidence.

Keep `docs/history/TODO-0.4.1.md` frozen. Add only a pointer to the 0.5 tracker. Do not rewrite closeout evidence.

Update `docs/history/TODO-0.5.md` after each acceptance check. Keep only one implementation phase
in progress. Record exact commands and observed results in its verification ledger.

### v0.3 tracker and research inputs (archived)

Read `docs/history/TODO.md` after this file before starting v0.3 work. Treat `docs/history/TODO.md` as the
authoritative phase tracker, but never treat a checkbox as verification evidence.

For model-fit or measurement work, read these research files in this order:

1. `research/measuring/README.md`
2. `research/measuring/SYNTHESIS.md`
3. Relevant project notes, source files, and tests

Update `docs/history/TODO.md` after each acceptance check. Keep only one implementation phase
in progress. Record exact commands and observed results in its verification ledger.

Keep `research/` ignored and unmodified during product implementation. Vitest is
restricted to `src/**/*.test.{ts,tsx}` so third-party checkout tests cannot enter
the Localmotive suite.

### Setup

```bash
npm install
npm run tauri dev          # hot-reloading dev app
```

### The check suite — run all of it before you claim to be done

```bash
npx tsc --noEmit -p tsconfig.json    # types
npm test                             # Vitest
npm run build                        # production frontend
cd src-tauri
cargo fmt --check                    # formatting (CI fails on drift)
cargo clippy --all-targets -- -D warnings   # zero warnings allowed
cargo test                           # Rust tests
```

CI runs exactly this on every push and PR. If it fails locally it will fail
there; do not push hoping otherwise.

### Self-hosted runner Rust isolation

The runner (`C:\actions-runner-localmotive`, Listener started hidden via
`localmotive-control\start-runner.ps1`) runs as the logged-in user, so its
Rust toolchain must not share the owner's `~/.cargo` / `~/.rustup`.
`start-runner.ps1` sets process-level `CARGO_HOME` / `RUSTUP_HOME` to the
dedicated `cargo-home` / `rustup-home` dirs before spawning the Listener.
Always start the runner via that script (or the watchdog that calls it) so
the isolation applies. Machine-level `CARGO_HOME` / `RUSTUP_HOME` may also
be set when elevation is available (belt and suspenders). Never clean the
owner's personal `~/.cargo`.

### Packaging

```bash
npm run tauri build
```

Produces `src-tauri/target/release/localmotive.exe`, an MSI, and an NSIS
installer.

### Verifying the packaged app

The `scripts/` directory drives the real binary over the Chrome DevTools
Protocol. This is how every release is checked:

```bash
# 1. Launch with the debugger port open
WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=10013' \
  ./src-tauri/target/release/localmotive.exe < /dev/null &

# 2. Drive it
node scripts/verify_024.mjs 10013 "C:\\llama\\llama-server.exe" "C:\\models"
```

Write a new `verify_<version>.mjs` for each release covering that release's
changes. Assert on values, not screenshots; use screenshots only for design
review.

> On this Windows host, always run Node with `< /dev/null`. Without it the
> process inherits a non-tty stdin and exits immediately with
> "stdin is not a tty".

---

## 4. Testing standards

### What a good test looks like

```rust
#[test]
fn free_port_prefers_the_requested_one_then_walks_upward() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let taken = listener.local_addr().unwrap().port();
    drop(listener);
    assert_eq!(pick_free_port("127.0.0.1", taken).unwrap(), taken);

    let held = TcpListener::bind(("127.0.0.1", taken)).unwrap();
    let next = pick_free_port("127.0.0.1", taken).unwrap();
    assert!(next > taken, "expected a port above {taken}, got {next}");
    drop(held);
}
```

It names the behaviour, uses real resources rather than mocks, and asserts the
consequence a user would notice.

### Requirements

- **Name the behaviour, not the function.** `scan_groups_complete_shards_and_marks_companions`, not `test_scan`.
- **Real code over mocks.** Use `tempdir`-style real files and real sockets.
  Mock only what crosses the network, and mock it at a trait boundary
  (`Bench`, `Advisor`, `SecretStore` are the existing examples).
- **Assert the observable consequence,** not internal call order.
- **Cover the failure path.** Every parser needs a malformed input test; every
  network call needs a failure test; every validator needs a rejection test.
- **Test the edges you actually hit:** empty input, one item, many items,
  duplicates, unicode, paths with spaces, values at and past a limit.

### Regression tests are mandatory

When a bug is found — by you, by CI, or by the user — the fix has three parts:

1. A test that reproduces it and **fails** on the current code.
2. The fix.
3. A comment on the test explaining the real-world case, so nobody "simplifies"
   it away later.

Examples in this repo, all written after a real failure:

- `parses_proposal_when_prose_contains_other_braces` — a cloud model echoed the
  JSON schema in prose and broke the parser.
- `quant_weight_reads_the_primary_label_not_a_provenance_suffix` —
  `Q6_K-from-BF16` was being read as BF16.
- `no_module_constructs_a_raw_command` — reads the source of every module and
  fails if any bypasses `proc::hidden_command`, which would flash a console.
- `adopts_the_current_runtime_instead_of_the_one_saved_with_the_profile` — a
  saved profile re-pinned an old runtime after an update.

### Verify your test can fail

After writing a test that passes, break the implementation deliberately and
confirm the test catches it. A test that cannot fail is worse than no test: it
provides false confidence. For critical invariants, keep a mutation check in
your process even if it is not committed.

---

## 5. Code conventions

### Rust

- `cargo fmt` output, no exceptions. Clippy at `-D warnings`.
- Errors are `Result<T, String>` at the Tauri boundary, with messages written
  for the user: name the problem *and* the recovery.
  `"Port 8080 is already in use on 127.0.0.1"`, not `"bind failed"`.
- No `unwrap()` on anything derived from user input, the filesystem, or the
  network. `unwrap()` in tests is fine.
- Child processes go through `proc::hidden_command`. A test enforces this.
- Public functions that encode a rule get a doc comment explaining *why*, not
  what.

### TypeScript

- Types in `model.ts`, mirroring the Rust structs with camelCase serde names.
- Pure decision logic goes in `model.ts` so it can be tested without a DOM.
- No `any`. No new dependencies without a reason that survives "could we do
  this in twenty lines?"

### CSS and design

`docs/DESIGN.md` is normative — token frontmatter plus the "machine-room control
cabinet" spec. Before touching `App.css`, read it. The load-bearing rules:

- `border-radius: 0` everywhere except indicator lamps.
- **No `box-shadow`.** Depth comes from the six graphite tones.
- Green means running/valid, amber means incomplete/unverified, red means stop.
  Never borrow them for emphasis. Signal colours stay under ~10% of a screen.
- Every state tag contains words; colour is a second channel, never the only one.
- Mobile targets ≥44 px, with 110 px bottom clearance above the fixed nav bar.

After UI changes, run the design detector and fix everything it reports:

```bash
node <impeccable-skill>/scripts/detect.mjs --json src/App.tsx src/App.css
```

New components need an entry in `docs/DESIGN.md` (frontmatter token + a prose
section) and must keep `designmd lint docs/DESIGN.md` at zero errors.

---

## 6. Releasing

Versions live in **three** files and must match, or CI rejects the tag:

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

Steps:

1. Bump all three.
2. Add a `## <version>` section to `CHANGELOG.md`. The release workflow extracts
   it verbatim as the release body, so write it for users: what changed, why it
   matters, what was verified.
3. Run the full check suite, then package and verify on the real binary.
4. Commit, push, then `git tag -a vX.Y.Z -m "..."` and push the tag.
5. The Release workflow builds and publishes the MSI, NSIS installer, portable
   exe, and a SHA256SUMS file.
6. **Confirm the release actually has its assets** before telling anyone it
   shipped.

Never tag a commit whose checks you have not run.

---

## 7. The curated catalog

`catalog/catalog.json` is a hand-curated list of GGUF models offered in the
**HF Catalog** tab. There is deliberately **no database and no server**:

- The app fetches it from `raw.githubusercontent.com`, which is CDN-served and
  supports `ETag` / `304 Not Modified`, so refreshes are nearly free.
- Editing is a normal commit (or the GitHub web editor) — versioned, reviewable,
  revertible.
- Nothing to host, pay for, secure, or keep running.

Downloads go **directly from Hugging Face to the user**; the catalog only says
which repos and files are offered. Adding an entry costs nothing and serves no
bytes.

Schema and curation rules: `catalog/README.md`. When you change the schema,
bump `schemaVersion`, keep the loader backward-compatible with the previous
version, and test both.

---

## 8. Things that will bite you

- **Console windows.** Any new `std::process::Command` flashes a console in a
  windowed app. Use `proc::hidden_command`; the test will catch you.
- **Split shards.** Models arrive as `-00001-of-00004.gguf`. Only the first
  shard is passed to `-m`. Never treat shards as separate models.
- **Companion ordering.** A family may ship many drafts (one real family ships
  eight DSpark variants). They are ranked by quantisation closeness; keep that
  deterministic or the UI and the suggested profile will disagree.
- **The per-file linter lies about Rust editions.** It reports
  "`async fn` is not permitted in Rust 2015" for edition-2021 code. Trust
  `cargo build`, not that message.
- **Windows paths.** Backslashes need escaping in JSON, JS string literals, and
  test fixtures. Prefer forward slashes when passing paths to native tools.
- **`gh` account labels can be stale.** Confirm with `gh api user --jq .login`
  before creating repositories.

---

## 9. Working with the user

- Report what you ran and what it returned. Show real numbers.
- Flag genuine uncertainty; do not pad with hedges.
- If asked to do something that would break a rule here, say so and propose the
  alternative rather than quietly doing it.
- Prefer finishing one item completely — code, tests, verification, docs — over
  starting several.
