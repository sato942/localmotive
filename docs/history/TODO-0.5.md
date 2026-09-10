# Localmotive 0.5 implementation tracker

- **Status:** PHASES 0–7 DONE 2026-09-10. PHASE 8 SHIP GATE OPEN (owner approval).
- **Target:** `0.5.0`
- **Tree at creation:** `30c28bfb2b4db1244f19424dc67326a074965535`
- **Tag at creation:** `v0.4.1` @ `a0ef02cf4b35a31ff10af1bd8c0a58064f86ab2e`
- **Signing:** DEFERRED_BY_OWNER. Builds stay honestly unsigned. Release notes disclose SmartScreen and checksums. Do not claim Authenticode.
- **Tracker rule:** Update a box only after its acceptance check runs. Record exact commands and observed results in the verification ledger. Do not treat a checkbox as evidence.
- **Frozen history:** Keep `docs/history/TODO-0.4.1.md` frozen. Add only a pointer to this file. Do not rewrite closeout evidence.

## 1. Scope

Localmotive 0.5 is a Windows x64 catalog and honesty release.

Do these jobs in 0.5:

- Make the current ship tip the GitHub Latest release while it stays honestly unsigned.
- Rewrite `README.md` for the current product and the 0.5 catalog story.
- Build catalog v2 from an allowlist with a signed publish artifact.
- Store the catalog in local SQLite with refresh cooldown.
- Add smart filters with hardware auto-defaults.
- Bound every user-facing input in Rust.
- Document real changes from 0.3 to 0.5 from git evidence.

Keep the control-plane scope. Do not reimplement inference. All inference stays in `llama-server`.

### Baseline pins at Phase 0 start

- Tree: `30c28bfb2b4db1244f19424dc67326a074965535`
- Tag `v0.4.1`: `a0ef02cf4b35a31ff10af1bd8c0a58064f86ab2e`
- Versions: `0.4.1` in `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`
- Release `v0.4.1`: `prerelease: true`, name `Localmotive 0.4.1 (unsigned prerelease)`, 6 assets
- Release `v0.4.0`: `prerelease: false`, Latest before the fix
- `GET /releases/latest` before the fix: `v0.4.0`
- Catalog schema: 1
- Catalog file: `catalog/catalog.json`, 6814 bytes, 8 models, 16 files
- Catalog total model bytes: 204498100288
- Catalog body limit: 4 MiB
- Catalog cache limit: 5 MiB
- Catalog signing: Ed25519 detached `catalog/catalog.json.sig`
- Runtime pin: `b10816` at `427291b5b34cd914a31b3fd3b61a68f6184f4b9f`

## 2. Non-goals

- Do not build a hosted catalog database or API.
- Do not use client-side live HF scrape as the default refresh.
- Do not hardcode the allowlist in the app binary.
- Do not weaken catalog signature verification.
- Do not use GitHub `prerelease: true` to mean unsigned.
- Do not claim Authenticode.
- Do not rewrite frozen `docs/history/TODO-0.4.1.md` evidence.
- Do not revive root `TODO.md`.
- Do not tag or publish `0.5.0` until the owner says ship.
- Do not touch personal `~/.cargo`. Keep runner Rust isolated via `start-runner.ps1`.

## 3. Phase checklist

### Phase 0 — Tracker and Latest repair (foundation, this phase)

- [x] Create `docs/history/TODO-0.5.md` with phases, boxes, and ledger.
- [x] Point `AGENTS.md` to `docs/history/TODO-0.5.md` for v0.5 work.
- [x] Edit GitHub release `v0.4.1`: set `prerelease: false`.
- [x] Rename `v0.4.1` to `Localmotive 0.4.1 (unsigned)`.
- [x] Confirm `GET /releases/latest` returns `v0.4.1`.
- [x] Confirm Releases UI shows Latest on 0.4.1.
- [x] Set `prerelease: false` in `.github/workflows/release.yml` publish path.
- [x] Set release name to `Localmotive ${{ version }} (unsigned)`.
- [x] Update read-back asserts to `isPrerelease === false` and unsigned name.
- [x] Update step titles and comments to honestly unsigned full release.
- [x] Update release-gates tests RED-green for the new asserts.
- [x] Remove pre-release default language from README and CHANGELOG current sections.

Phase 0 evidence (2026-09-10, HEAD `30c28bf` + working tree):

- RED: updated the two release-gates tests to require `prerelease: false` and reject `(unsigned prerelease)`. Ran `node --test scripts/tests/release-gates.test.mjs`. Observed 71 pass, 2 fail on the two renamed tests.
- GREEN: edited `.github/workflows/release.yml` publish path to `prerelease: false`, name `(unsigned)`, read-back `isPrerelease,false`, honestly unsigned full-release language. Ran `node --test scripts/tests/release-gates.test.mjs`. Observed 73 pass, 0 fail.
- Gates: `node scripts/verify_workflow_pins.mjs` PASS. `node scripts/verify_workflow_gates.mjs` PASS with zero failures.
- Public repair: `gh release edit v0.4.1 --prerelease=false --title "Localmotive 0.4.1 (unsigned)"` exit 0. Read-back `releases/tags/v0.4.1` is `prerelease:false`, name `Localmotive 0.4.1 (unsigned)`. `releases/latest` is `tag v0.4.1`. `gh release list` shows 0.4.1 Latest, 0.4.0 unmarked.
- README: replaced `Unsigned artifacts stay testing-only or pre-release.` with full-release Latest language. Full README rewrite stays in Phase 1.
- CHANGELOG: checked for pre-release default language. Found none. No change needed.
- Full gate proof on working tree: `npm test` gives Vitest 50 pass plus release-gates 73 pass 0 fail. `node scripts/verify_versions.mjs 0.4.1` PASS. `npm run branding:verify` PASS. `npm run catalog:validate` gives `valid (8 models, 16 files)`. `npx tsc --noEmit -p tsconfig.json` exit 0. No Rust file changed, so Rust suite stays for the next code change.
- CI on pushed commit `5d6cb0d`: run `34471288882` completed success on 2026-09-10. Jobs: rust-audit success, check success, Security audit success, package-smoke success.
- HF probe: `author=unsloth&filter=gguf&sort=lastModified&direction=-1&limit=5` returns live JSON with `lastModified`, `likes`, `downloads`, `tags`, `pipeline_tag`, `library_name`, `createdAt`. This supports author-scoped discovery with last-90-days filter for Phase 2 dry-run.

Acceptance:

- Latest API and UI point at the current tip.
- Workflow cannot mark the ship release as prerelease.
- Gates pass.

### Phase 1 — README rewrite

- [x] Rewrite root `README.md` for the current product.
- [x] Document download from Releases Latest as current unsigned.
- [x] Document SmartScreen and SHA-256 verification.
- [x] Document quick start, HF Catalog, refresh, filters, hardware-fit defaults.
- [x] Document network needs.
- [x] Remove stale 0.3.0 download section.
- [x] Add no invented benchmarks.
- [x] Add no Win10 claim without L4 evidence.

Phase 1 evidence (2026-09-10, commit `921df5e`):

- RED: new release-gates test required no `0.3.0` download names, no `(unsigned prerelease)`, plus Latest, `(unsigned)`, SmartScreen, SHA256SUMS, schema 2, providers.json, Hardware fit, raw.githubusercontent.com. Ran gates. Observed 76 pass, 1 fail on README.
- GREEN: rewrote `README.md` Download section with versioned file names, Latest policy, `(unsigned)` naming, SmartScreen, SHA-256 proof with 0.4.1 example. Catalog section covers providers.json allowlist, 90-day recency, schemaVersion 2, Hardware-fit defaults with disable/widen. Security section discloses Rust-boundary bounds. Ran gates. Observed 77 pass, 0 fail. Branding PASS.
- `catalog/README.md` rewritten in the same commit: schema 2 contract, snake-key signature rule, builder + signed publish flow, SQLite-local-only, dedupe and exclusion rules.
- No benchmarks invented. No Win10 claim beyond the existing qualification-limited language, which stays unchanged.

Acceptance:

- README matches product and Latest unsigned honesty.
- Docs and branding gates pass.

### Phase 2 — Catalog dry-run and format decision

- [x] Run allowlist dry-run against HF.
- [x] Record repo count, file count, artifact bytes.
- [x] Record decision on size limit and format in this file.
- [x] Decide gzip JSON versus signed SQLite network drop.
- [x] Keep current 4 MiB body and 5 MiB cache limits until the decision lands.

Phase 2 evidence (2026-09-10, HEAD `5d6cb0d`):

- Command: `node scripts/dryrun_catalog.mjs`
- Observed: exit 0. Authors 24. Repos scanned 204. Files 1505. Artifact model bytes 22837534248768 (21269 GiB of model weights, not download size). Skipped no-SHA 0. Skipped excluded shards/companions 2141. Cutoff 2026-06-12T00:00:00Z (90 days before 2026-09-10).
- HF field used for recency: `lastModified` from `GET /api/models?author=<name>&filter=gguf&sort=lastModified&direction=-1&limit=100`. Cap 20 repos per author for the dry-run. Full publish scans the same field without the 20 cap.
- Per-author recent90/scanned/files: `unsloth` 50/20/239, `ornith-ai` 5/5/24, `lmstudio-community` 23/20/43, `huihui-ai` 14/14/63, `DavidAU` 24/20/211, `orcarouter` 4/4/15, `AtomicChat` 49/20/138, `bartowski` 100/20/450, `LiquidAI` 31/20/119, `Qwen` 0/0/0, `deepseek-ai` 0/0/0, `google` 5/5/5, `zai-org` 0/0/0, `MiniMaxAI` 0/0/0, `tencent` 2/2/5, `trohrbaugh` 1/1/1, `IFM` 5/5/5, `nvidia` 7/7/7, `microsoft` 3/3/4, `poolside` 2/2/5, `heterodoxin` 8/8/46, `coder3101` 0/0/0, `llmfan46` 25/20/88, `Blackfrost-AI` 8/8/37.
- Ambiguous or empty names resolved live, not invented: `IFM` resolves and returns 5 recent GGUF repos. `deepseek-ai` exists but publishes 0 GGUF repos. `MiniMaxAI` exists but publishes 0 GGUF repos. `Qwen` publishes 54 GGUF repos but newest is 2026-02-04, outside 90 days. `zai-org` publishes 5 GGUF repos but newest is 2024-11-28, outside 90 days. `coder3101` publishes 3 GGUF repos but newest is 2026-01-18, outside 90 days. All six stay in the allowlist with zero recent rows rather than removal by guess.
- Metadata size estimate: 1505 files at about 900 bytes of rich v2 metadata each gives 1354500 bytes (1.29 MiB) raw JSON. Gzip ratio 0.18 gives about 243810 bytes (0.23 MiB).
- Format decision: gzip JSON. It fits the current 4 MiB body limit and 5 MiB cache limit with headroom for full 90-day growth. No signed SQLite network drop. SQLite stays local-only on the PC per Phase 4.
- Size decision: keep body 4 MiB and cache 5 MiB unchanged. Revisit only if a publish exceeds 3 MiB gzipped.
- Full uncapped publish will exceed 1505 files because `bartowski` (100 recent), `unsloth` (50), `AtomicChat` (49) were capped at 20 repos each. Even at 3x files the gzipped JSON stays under 1 MiB. Decision stands.

Acceptance:

- Dry-run numbers live in the ledger.
- Size and format decision is explicit.

### Phase 3 — Schema v2, builder, allowlist, signed publish

- [x] Create `catalog/providers.json` with the deduped allowlist.
- [x] Implement author-scoped discovery for the last 90 days.
- [x] Include all single-file `.gguf` files in those repos.
- [x] Exclude shards, mmproj, draft, DSpark, DFlash, EAGLE, MTP companions.
- [x] Skip a file if SHA-256 is missing. Never invent metadata.
- [x] Fail closed against the last good catalog.
- [x] Bump `schemaVersion` to 2.
- [x] Set `SUPPORTED_SCHEMA` to 2 with clear failure on older schema.
- [x] Update `catalog/README.md`, parser, validate script, Rust tests, Publish workflow.
- [x] Ship the allowlist inside the signed artifact or signed sidecar.
- [x] Allow user local overrides. Mark user-sourced entries. Keep network verification strict.
- [x] Publish the signed artifact through CI.

Phase 3 evidence (2026-09-10, HEAD `634426a` + working tree):

- RED: new release-gates test required schema 2 across catalog, backend, validator, builder. Ran `node --test scripts/tests/release-gates.test.mjs`. Observed 72 pass, 1 fail (`1 !== 2` on checked-in v1 catalog).
- GREEN: `src-tauri/src/catalog.rs` sets `SUPPORTED_SCHEMA = 2` with `MIN_SUPPORTED_SCHEMA = 2` fail-closed and rich v2 fields. `scripts/build_catalog.mjs` reads `catalog/providers.json`, filters `lastModified >= 2026-06-12`, excludes shards/companions/subdirs, dedupes filenames by downloads, requires SHA-256, embeds providers block. `scripts/validate_catalog.mjs` requires schema 2 + providers + ISO dates. Ran gates again. Observed 73 pass, 0 fail.
- Rust: `cargo test --locked` gives 387 pass, 0 fail, 2 ignored. `cargo fmt --check` clean. `cargo clippy --locked --all-targets -- -D warnings` clean.
- CI catalog run `34479565302` RED: build FAILURE because bare `build_catalog.mjs` printed JSON to stdout and validate ran against stale v1. Fixed: default writes unsigned `catalog.json`, `--stdout` previews, key stays in sign job only. Commit `8093098`.
- CI catalog run `34479960156` RED: build FAILURE `detached Ed25519 signature is invalid` because validate ran against stale v1 `.sig`. Fixed: `--no-signature` for build-job structure check, full check after sign. Commit `634426a`.
- CI catalog run `34480446250` GREEN: build success (`resolved 158 repos, 1417 files`, `catalog: valid v2 (158 models, 1417 files)`), sign success (signed with maintainer key, `catalog: valid v2 (158 models, 1417 files)`). Artifacts `localmotive-catalog-candidate` (105250 B) + `localmotive-catalog-signature` (105726 B).
- Signed v2 checked in from run artifacts: `catalog.json` 630182 B (0.60 MiB), gzip 105130 B, schema 2, 158 models, 1417 files, updated 2026-09-10. `npm run catalog:validate` gives `valid v2 (158 models, 1417 files)`.
- Builder preview defects found and fixed before signing: 23 slash filenames (MTP/imatrix/experiments subdirs) excluded; 61 cross-publisher filename collisions with different SHA-256 deduped by downloads (most-used publisher wins).
- v1 rejection: `parse_rejects_schema_1_without_rich_fields` proves v1 fails with `needs schema 2`. Old schema fails clearly.
- Author add = catalog publish only: builder reads `providers.json`, backend has no allowlist reference (gate asserts `doesNotMatch backend /providers.json/`).
- Remaining: user local overrides (deferred to Phase 4 SQLite work, where user-sourced rows get marked). Network verification stays strict.
- `catalog/README.md` update stays open until Phase 4 lands the SQLite story in one rewrite.

Acceptance:

- v2 validates. Old schema fails clearly.
- Author add needs catalog publish only, not app release.

### Phase 4 — Local SQLite mirror, first start, refresh, cooldown (DONE 2026-09-10)

- [x] Fill from verified artifact, then last-good cache, then bundled fallback.
- [x] Show honest source in UI.
- [x] Implement refresh with the same pipeline.
- [x] Implement refresh cooldown with default 1560 min.
- [x] Implement in-flight lock.
- [x] Show last success and remaining cooldown.
- [x] Rebuild corrupt cache from verified artifact or fallback.
- [x] SQLite mirror fill from verified bytes.
- [x] Query browse, filter, and sort from SQLite.
- [x] Versioned migrations.
- [x] Corrupt-DB rebuild test.

Phase 4 evidence (2026-09-10, commit `39d3684`):

- RED: new release-gates test `catalog refresh honors cooldown, lock, and last-success display` with impl stashed. Observed 77 pass, 1 fail on the new test.
- GREEN: `catalog.rs` adds `CATALOG_REFRESH_COOLDOWN_MINUTES = 1560`, `refresh_cooldown_remaining_minutes` (missing/unparsable stamp never blocks first fill), `CatalogRefreshGuard` (one refresh at a time, releases on drop), stamp read/write beside the cache, snapshot carries `lastSuccessSecs` + `cooldownRemainingMinutes` on all four origin paths. `fetch_model_catalog` acquires the guard, enforces cooldown with remaining-minutes errors, stamps success. `model.ts`/`App.tsx` render LAST SUCCESS plus COOLDOWN minutes. Observed gates 78 pass 0 fail. Rust lib 397 pass 0 fail. tsc clean. Catalog v2 valid (158/1417). Pins plus workflow gates clean. Clippy plus fmt clean.
- First start always fills: no stamp means no cooldown. Clock skew into the past caps at the full wait, never more.

Phase 4 closeout (2026-09-10, SQLite mirror + user overrides land):

- RED: two new release-gates tests (`local catalog SQLite mirror stores verified models with migrations and rebuilds`, `user catalog overrides stay local, marked, and outside network verification`). Ran `node --test scripts/tests/release-gates.test.mjs`. Observed 78 pass, 2 fail on the new tests.
- GREEN: new `src-tauri/src/catalog_db.rs` (rusqlite, local-only): `catalog_db_path` beside the JSON cache, `CATALOG_DB_SCHEMA_VERSION = 1`, `migrate_catalog_db` in one transaction with `PRAGMA user_version`, `mirror_verified_catalog` replacing network rows while preserving `user_sourced = 1` rows, `read_catalog_db_models` feeding the existing `filter_models`/`facets`/`rich_facets` (SQLite is storage, not a second filter), `rebuild_catalog_db_from_verified` deleting garbage and rebuilding from verified bytes. `CatalogModel.user_sourced` added with serde default; network parse keeps false. `fetch_model_catalog` mirrors verified rows after each fetch; `catalog_local_models` serves mirror rows with snapshot/bundled fallback so the tab never goes empty on a DB problem. `save_user_catalog_override` / `remove_user_catalog_override` commands enforce `validate_user_override` (repo shape, filename guards, SHA-256, 512-byte text cap, 200-row cap); remove refuses curated rows with a curated message. UI reads `catalog_local_models`, marks rows `USER ADDED · LOCAL ONLY`, and notes the count in the load notice. Observed gates 80 pass 0 fail. Rust lib 403 pass 0 fail 2 ignored. tsc exit 0. Vitest 52 pass. Catalog v2 valid (158/1417). fmt clean. Clippy zero errors.
- User rows never touch the signed artifact, never pass signature checks, and never leave the PC. Downloads still resolve against the signed snapshot in state, so a user row cannot authorize a network file.
- Rust tests: `mirror_stores_verified_models_and_reads_them_back`, `migrations_stamp_the_schema_version`, `corrupt_database_rebuilds_from_verified_bytes`, `user_override_is_marked_and_survives_network_refresh`, `user_override_rejects_bad_repos_missing_digests_and_oversize_text`, `user_override_remove_never_touches_network_rows` (all PASS in the 403-pass lib run).

Phase 4 decision (2026-09-10, SQLite stays local-only, mirror is next):

- SQLite stays a local query cache only. It is NOT the network drop: the
  signed JSON stays the network contract per the Phase 2 format decision
  (gzip v2 0.10 MiB fits the 4 MiB body limit). No hosted DB or API exists.
- The existing `catalog-cache.json` record plus the bundled fallback already
  implement the full Phase 4 pipeline in production: first start fills from
  verified network artifact, then last-good signed cache, then bundled file;
  `CatalogSnapshot.origin` (`network`, `not-modified`, `cache`, `bundled`)
  shows the honest source in the UI via `originLabel`; ETag conditional
  refresh avoids re-download; the cache write lock is the in-flight guard;
  corrupt cache falls back to bundled (proven by
  `corrupt_cache_falls_back_to_the_bundled_catalog`).
- `rusqlite 0.40.2` with the bundled feature is added to `Cargo.toml` plus
  lockfile in commit `a75060b` as Phase 4 groundwork. It compiles
  (`cargo build` clean) and stays unused until the local mirror lands, so no
  behavior changes and no migration runs yet.
- Cooldown default 1560 min, versioned migrations, last-success display, and
  the SQLite mirror query path stay open work for the next increment. They
  are NOT claimed as done. No box is checked on intent: the checkmarks above
  cover the pipeline semantics the JSON cache already proves, and the tracker
  records the SQLite remainder explicitly here.
- Remaining before Phase 4 closes fully: SQLite mirror fill from verified
  bytes, refresh cooldown timestamp, in-flight DB lock, last-success UI,
  migration versioning, corrupt-DB rebuild test. rusqlite is vendored and
  compiling; the mirror is the next commit, not this one.

Acceptance:

- First start fills DB. Refresh respects cooldown and lock.
- Corrupt DB rebuilds.

### Phase 5 — Filters and hardware auto-defaults (DONE 2026-09-10, commits `4f96d0a` + `a75060b`)

- [x] Add rich adjustable filters and sort.
- [x] Support size, quant, params, author, gated, downloads, likes, recency, tags, license.
- [x] Enable auto filter by default from `detect_hardware`.
- [x] Prefer dedicated VRAM when known, else system memory evidence.
- [x] Do not combine dedicated and shared budgets if product rules forbid it.
- [x] Hide or deprioritize files above a tunable fraction of budget by default.
- [x] Allow user disable and widen. Explain why in UI.

Phase 5 evidence (verified 2026-09-10 on this tree):

- Backend owns truth: `filter_models` (text/tag/quant/author/license/pipeline/architecture/size/gated/sort), `model_hidden_by_fit_rule` (saturating `budget * per_mille / 1000`, zero disables), `hardware_fit_budget` (dedicated wins, else shared, else system, never summed), `rich_facets` + `catalog_fit_budget` commands.
- UI sends inputs only: search, use, quant, author, licence, pipeline, architecture, size, order, hide-gated, Hardware-fit toggle (default ON), fit-budget fraction (25/50/75/100%). Budget line explains source (`on dedicated budget` etc.) or `budget unknown`. Per-row WHY is the size label plus the fit line.
- Tests: `filter_applies_rich_author_license_pipeline_and_architecture_constraints`, `hardware_fit_rule_hides_models_above_a_tunable_budget_fraction`, `hardware_fit_budget_never_combines_dedicated_and_shared`, `rich_facets_list_authors_licenses_pipelines_and_architectures`, `filter_sorts_deterministically_on_every_key`, `facets_are_sorted_and_deduplicated` (all PASS in the 397-pass lib run); `modelHiddenByFitRule` + `hardwareFitBudget` Vitest pins (52 pass); release-gates `model catalog exposes rich filters with hardware auto-fit defaults` (PASS in the 78-pass run).

Acceptance:

- Auto defaults have tests.
- User can disable or widen the filter.

### Phase 6 — Input-limit audit and Rust enforcement (DONE 2026-09-10, commit `bd1de08`)

- [x] Inventory every user-editable field.
- [x] Define max length, numeric min-max, charset, path safety per field.
- [x] Enforce limits in Rust at the Tauri and domain boundary.
- [x] Keep UI limits as hints only.
- [x] Keep `llama-server` values within runtime `--help` capabilities.
- [x] Reject oversize, negative, NaN, `..` paths, huge JSON without panic or hang.

Phase 6 evidence (verified 2026-09-10 on this tree):

- Catalog boundary: `MAX_QUERY_TEXT_LEN = 512`, `MAX_QUERY_TERM_LEN = 128`, `MAX_FILTER_VALUE_LEN = 128`, `MAX_FILTER_MODELS = 10_000`, `MAX_FACET_MODELS = 10_000`, `MAX_BUDGET_ENTRIES = 64`; `validate_catalog_query` / `validate_facet_models` / `validate_budget_inputs` run in `filter_catalog`, `catalog_facets`, `catalog_rich_facets`, `catalog_fit_budget` before any work.
- Profile boundary: `MAX_PROFILE_TEXT_LEN = 1024`, `MAX_PROFILE_PATH_LEN = 32767`, `MAX_TENSOR_SPLIT_ENTRIES = 64`, `MAX_TENSOR_SPLIT_CHARS = 1024`; `validate_profile_input_bounds` runs first in `build_args`, and `validate_launch_arguments` keeps values inside runtime `--help` capabilities. HF token keeps charset/shape checks; download target keeps repo/filename/`..`/reparse guards.
- Tests: `catalog_query_limits_reject_oversize_text_filters_and_model_lists`, `launch_profile_input_bounds_reject_oversize_strings_and_splits`, `raw_extra_argument_count_is_bounded`, `raw_extra_argument_length_is_bounded`, plus the pre-existing `..`/negative/NaN/malformed suites (all PASS in the 397-pass lib run); release-gates `catalog commands reject oversize input` + `launch profiles reject oversize input` (PASS in the 78-pass run).

Acceptance:

- All listed inputs reject bad values with clear errors.
- Tests cover oversize and malformed cases.

### Phase 7 — Real 0.3 to 0.5 change documentation (DONE 2026-09-10, no `## 0.5.0` yet: bump waits for owner ship gate)

- [x] Verify history with `git log v0.3.0..HEAD`, tags, release notes, TODO closeouts.
- [x] Write evidence-backed 0.3.0 to 0.4.0 to 0.4.1 to 0.5.0 narrative.
- [ ] Add `## 0.5.0` to `CHANGELOG.md` at bump.
- [x] Add short Whats new since 0.3 in README or `docs/` if useful.
- [x] Disclose unsigned, SmartScreen, Latest fix, self-hosted CI, catalog signing, evidence, package verify, deferred Authenticode.
- [x] Invent no features, signing, Win10 support, or perf numbers.

Phase 7 evidence (2026-09-10, verified on this tree, counts from git):

- `git log v0.3.0..HEAD --oneline | wc -l` gives 99. Split: `v0.3.0..v0.4.0` 2 commits (rename `859a2cf` + release `b0435b3`), `v0.4.0..v0.4.1` 83 commits, `v0.4.1..HEAD` 14 commits. Tags present: `v0.3.0`, `v0.4.0`, `v0.4.1`. No `v0.5.0` tag exists.
- 0.3.0 to 0.4.0 (`b0435b3`): `CHANGELOG.md ## 0.4.0` is the narrative — approved-runtime manifest, device-evidence recommendation, health contract, extraction guards, scope lines, packaged checks (Rust 296+1, frontend 46). No invented additions here.
- 0.4.0 to 0.4.1 (83 commits): `3404206` product surface (UI honesty, icons, catalog tab, narrow claims) + `cc8a1a6` backend hardening (managed trust, health contract, bounded catalog); `7c76831` self-hosted runner for check jobs; `bdd9934`+`fdcf81c`+`877206a` Sandbox clean-account lifecycle; `bc8c00f` L4 attestation; `c8f5c11` sourceRevision rebind; F-041 finding pins; `1205484` unsigned disclosure + `00d5c7f` unsigned publish path (checksum+inventory replace Authenticode) + `f5d1c2f` self-hosted release runs; `CHANGELOG.md ## 0.4.1` corrective note + deferred-signing exception (`Authenticode is deferred by owner order 2026-09-09`, verify SHA-256).
- 0.4.1 to 0.5 work-in-progress (14 commits, unreleased, no tag): `5d6cb0d` Latest repair (`v0.4.1` to `prerelease:false`, `(unsigned)` name); `90147ff`→`8093098`→`634426a` schema v2 contract + allowlist builder + signed publish (CI runs `34479565302`/`34479960156` RED then `34480446250` GREEN); `c83811a` signed v2 lands (158 models, 1417 files, 630182 B, gzip 105130 B); `a75060b` filters/fit/budget + rusqlite groundwork; `4f96d0a` UI filters + auto-fit; `bd1de08` input bounds; `921df5e` README rewrite; `39d3684` cooldown/lock/stamp.
- Whats new since 0.3 lives in `README.md`: catalog v2 story (providers.json, 90-day recency, schema 2, hardware fit, signed artifact, refresh guard + 1560 min cooldown + last-success display), input-boundary disclosure, Latest/unsigned honesty. `CHANGELOG.md ## 0.5.0` waits for the owner-gated bump — writing it now would claim an unshipped version.
- Every disclosure above traces to a commit, tag, CI run, or file named here. No benchmarks, no signing, no Win10 support, no perf numbers invented.

Acceptance:

- Every claim traces to git or release evidence.

### Phase 8 — Version bump and ship gates (owner gate)

- [ ] Bump `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` to `0.5.0` together.
- [ ] Add CHANGELOG `## 0.5.0`.
- [ ] Update README asset names and `SHA256SUMS` examples.
- [ ] Get CI green.
- [ ] Tag-push Release with `prerelease: false` and name `(unsigned)`.
- [ ] Confirm read-back PASS and Latest = `v0.5.0`.
- [ ] Wait for explicit owner ship approval before tag and publish.

Acceptance:

- All ship gates pass. Owner approves publish.

## 4. Pointers

- Allowlist config: `catalog/providers.json` (live dry-run config, schemaVersion 1, cutoffDays 90, maxReposPerAuthor 20 dry-run cap).
- Dry-run script: `scripts/dryrun_catalog.mjs` (read-only, no signing key, no catalog write).
- Current allowlist source: this section until `catalog/providers.json` lands.
- Planned allowlist names: `unsloth`, `ornith-ai`, `lmstudio-community`, `huihui-ai`, `DavidAU`, `orcarouter`, `AtomicChat`, `bartowski`, `LiquidAI`, `Qwen`, `deepseek-ai`, `google`, `zai-org`, `MiniMaxAI`, `tencent`, `trohrbaugh`, `IFM`, `nvidia`, `microsoft`, `poolside`, `heterodoxin`, `coder3101`, `llmfan46`, `Blackfrost-AI`.
- Ambiguous names resolved by dry-run 2026-09-10: `IFM` resolves with 5 recent GGUF repos. `deepseek-ai`, `MiniMaxAI` exist with 0 GGUF repos. `Qwen`, `zai-org`, `coder3101` exist but have no GGUF repo newer than 90 days. Do not invent.
- Size limits now: body 4 MiB, cache 5 MiB. Size decision from dry-run: KEEP. Gzip v2 estimate 0.23 MiB.
- Schema version: 2 live. `SUPPORTED_SCHEMA` in `src-tauri/src/catalog.rs` is 2 with `MIN_SUPPORTED_SCHEMA = 2` fail-closed. Checked-in catalog: schema 2, 158 models, 1417 files, updated 2026-09-10.
- Cooldown default: 1560 min live in `CATALOG_REFRESH_COOLDOWN_MINUTES`, enforced in `fetch_model_catalog` with remaining-minutes errors; last success shown in UI.
- Format decision: gzip JSON (decided 2026-09-10). No SQLite network drop. SQLite stays local-only.
- Latest policy: honestly unsigned full release with `prerelease: false`. Honesty lives in name, body, and notes. Never hide the ship tip as Pre-release.
- Signing: DEFERRED_BY_OWNER. No Authenticode claim.

## 5. Verification ledger

Append every check with exact command and observed result. Never write should work.

### 2026-09-10 Phase 0 start

- Command: `git rev-parse HEAD`
- Observed: `30c28bfb2b4db1244f19424dc67326a074965535`

- Command: `git rev-parse v0.4.1`
- Observed: `a0ef02cf4b35a31ff10af1bd8c0a58064f86ab2e`

- Command: `gh api repos/sato942/localmotive/releases/latest --jq '{tag: .tag_name, name: .name, prerelease: .prerelease, draft: .draft}'`
- Observed before fix: `{"draft":false,"name":"Localmotive 0.4.0","prerelease":false,"tag":"v0.4.0"}`

- Command: `gh api repos/sato942/localmotive/releases/tags/v0.4.1 --jq '{tag: .tag_name, name: .name, prerelease: .prerelease, draft: .draft}'`
- Observed before fix: `{"draft":false,"name":"Localmotive 0.4.1 (unsigned prerelease)","prerelease":true,"tag":"v0.4.1"}`

- Command: `node -e "const c=require('./catalog/catalog.json'); console.log('schema',c.schemaVersion,'models',c.models.length,'files',c.models.reduce((n,m)=>n+m.files.length,0),'bytes',c.models.reduce((n,m)=>n+m.files.reduce((a,f)=>a+f.sizeBytes,0),0))"`
- Observed: `schema 1 models 8 files 16 bytes 204498100288`

- Pending: Latest repair commands and output. Done 2026-09-10. See Phase 0 evidence.
- Pending: release-gates RED and GREEN runs. Done 2026-09-10. RED 71 pass 2 fail. GREEN 73 pass 0 fail. Pins PASS. Workflow gates PASS.
- Pending: catalog dry-run numbers. Done 2026-09-10. See Phase 2 evidence.

### 2026-09-10 Final gate suite (HEAD `d388471`, before tracker closeout)

- Command: `cargo test --locked --lib` in `src-tauri`
- Observed: 397 pass, 0 fail, 2 ignored.
- Command: `cargo fmt --check` / `cargo clippy --locked --all-targets -- -D warnings`
- Observed: both clean, zero errors.
- Command: `npm test`
- Observed: Vitest 52 pass; release-gates 78 pass 0 fail.
- Command: `npx tsc --noEmit -p tsconfig.json`
- Observed: exit 0.
- Command: `npm run catalog:validate`
- Observed: `valid v2 (158 models, 1417 files)`.
- Command: `node scripts/verify_workflow_pins.mjs` / `node scripts/verify_workflow_gates.mjs` / `node scripts/verify_versions.mjs 0.4.1` / `npm run branding:verify` / `npm run build`
- Observed: all PASS; production frontend builds.
- Command: `RUSTDOCFLAGS='-D warnings' cargo test --locked --doc`
- Observed: 0 pass, 0 fail, clean.
- Command: `gh api repos/sato942/localmotive/releases/latest --jq '{tag, name, prerelease}'`
- Observed: `v0.4.1`, `Localmotive 0.4.1 (unsigned)`, `prerelease:false`. No `v0.5.0` tag exists. Nothing published.
- CI on pushed HEAD `d388471`: run `34487771471` in progress at closeout (rust-audit, check, Security audit success; package-smoke running). Local suite above is the closeout evidence; CI read-back appends on completion.

## 6. Reproduction and verification commands

Run repository checks from the repository root:

```text
node scripts/verify_versions.mjs 0.4.1
node scripts/verify_workflow_pins.mjs
node scripts/verify_workflow_gates.mjs
node --test scripts/tests/release-gates.test.mjs
npm run check
cd src-tauri
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
RUSTDOCFLAGS='-D warnings' cargo test --locked --doc
```

Check Latest visibility:

```text
gh release list --limit 10
gh api repos/sato942/localmotive/releases/latest --jq '{tag: .tag_name, name: .name, prerelease: .prerelease, draft: .draft}'
gh api repos/sato942/localmotive/releases/tags/v0.4.1 --jq '{tag: .tag_name, name: .name, prerelease: .prerelease, draft: .draft}'
```

Check catalog:

```text
npm run catalog:validate
wc -c catalog/catalog.json
```

## 7. Evidence index

- Current tracker: `docs/history/TODO-0.5.md`
- Frozen tracker: `docs/history/TODO-0.4.1.md`
- Release workflow: `.github/workflows/release.yml`
- Release gates: `scripts/tests/release-gates.test.mjs`
- Workflow gates policy: `.github/workflow-gates.json`
- Catalog manifest: `catalog/catalog.json`
- Catalog signature: `catalog/catalog.json.sig`
- Catalog rules: `catalog/README.md`
- Catalog builder: `scripts/build_catalog.mjs`
- Catalog validator: `scripts/validate_catalog.mjs`
- Allowlist config: `catalog/providers.json` (live, 24 authors, schemaVersion 2, cutoffDays 90).
- Catalog core: `src-tauri/src/catalog.rs`
- Published releases: <https://github.com/sato942/localmotive/releases>
- Latest API: <https://api.github.com/repos/sato942/localmotive/releases/latest>

## 8. Current acceptance status

| Material criterion | Result | Reason |
|---|---|---|
| 0.5 tracker exists and covers the mandate | PASS | File exists with phases, pointers, ledger, acceptance table. |
| Latest points at current tip while unsigned | PASS | `releases/latest` is `v0.4.1`. `v0.4.1` is `prerelease:false` with name `(unsigned)`. |
| Workflow cannot mark ship as prerelease | PASS | `release.yml` sets `prerelease: false` and asserts `isPrerelease:false`. Release-gates 78 pass 0 fail. |
| README matches product and unsigned Latest | PASS | Rewritten in `921df5e`. Gates README test PASS. Branding PASS. |
| Catalog v2 + signed publish | PASS | Schema 2 live, 158 models, 1417 files, CI-signed. Validator + gates PASS. |
| Refresh cooldown + lock + last-success | PASS | `39d3684`. Cooldown 1560 live, guard live, stamp display live. Gates 78/0. Rust 397/0. |
| Filters + hardware auto-defaults | PASS | Rich filters + fit rule + budget, UI toggle default ON. Rust + Vitest + gates PASS. |
| Input limits enforced in Rust | PASS | Catalog + profile bounds at Tauri boundary. Rust + gates PASS. |
| Real 0.3 to 0.5 docs | PASS | Phase 7 narrative in tracker, counts from git. No invented claims. `## 0.5.0` waits for bump. |
| SQLite mirror + migrations + DB rebuild test | PASS | `catalog_db.rs` local-only; mirror/rebuild/migration tests PASS; gates 80/0. |
| User local catalog overrides | PASS | Marked `user_sourced`, local-only, strict verification; remove refuses curated rows. |
| 0.5.0 ship gates | BLOCKED | Owner ship approval pending. No tag. No publish. |

## Conclusion

Phases 0–7 are DONE. The SQLite mirror (fill, query path, migrations,
corrupt-DB rebuild) and user local overrides (marked, local-only, strict
verification) are implemented with RED-first tests and gate proof above.

Do not tag or publish 0.5.0 until the owner says ship. Phase 8 stays [ ].
