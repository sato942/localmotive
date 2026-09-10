# Localmotive 0.5 implementation tracker

- **Status:** PHASE 0 FOUNDATION IN PROGRESS
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

- [ ] Rewrite root `README.md` for the current product.
- [ ] Document download from Releases Latest as current unsigned.
- [ ] Document SmartScreen and SHA-256 verification.
- [ ] Document quick start, HF Catalog, refresh, filters, hardware-fit defaults.
- [ ] Document network needs.
- [ ] Remove stale 0.3.0 download section.
- [ ] Add no invented benchmarks.
- [ ] Add no Win10 claim without L4 evidence.

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

- [ ] Create `catalog/providers.json` with the deduped allowlist.
- [ ] Implement author-scoped discovery for the last 90 days.
- [ ] Include all single-file `.gguf` files in those repos.
- [ ] Exclude shards, mmproj, draft, DSpark, DFlash, EAGLE, MTP companions.
- [ ] Skip a file if SHA-256 is missing. Never invent metadata.
- [ ] Fail closed against the last good catalog.
- [ ] Bump `schemaVersion` to 2.
- [ ] Set `SUPPORTED_SCHEMA` to 2 with clear failure on older schema.
- [ ] Update `catalog/README.md`, parser, validate script, Rust tests, Publish workflow.
- [ ] Ship the allowlist inside the signed artifact or signed sidecar.
- [ ] Allow user local overrides. Mark user-sourced entries. Keep network verification strict.
- [ ] Publish the signed artifact through CI.

Acceptance:

- v2 validates. Old schema fails clearly.
- Author add needs catalog publish only, not app release.

### Phase 4 — Local SQLite, first start, refresh, cooldown

- [ ] Create local SQLite on first start or missing DB.
- [ ] Fill DB from verified artifact, then last-good cache, then bundled fallback.
- [ ] Show honest source in UI.
- [ ] Implement refresh with the same pipeline.
- [ ] Implement cooldown with default 1560 min.
- [ ] Implement in-flight lock.
- [ ] Show last success and remaining cooldown.
- [ ] Query browse, filter, and sort from SQLite.
- [ ] Add versioned migrations.
- [ ] Rebuild corrupt DB from verified artifact or fallback.

Acceptance:

- First start fills DB. Refresh respects cooldown and lock.
- Corrupt DB rebuilds.

### Phase 5 — Filters and hardware auto-defaults

- [ ] Add rich adjustable filters and sort.
- [ ] Support size, quant, params, author, gated, downloads, likes, recency, tags, license.
- [ ] Enable auto filter by default from `detect_hardware`.
- [ ] Prefer dedicated VRAM when known, else system memory evidence.
- [ ] Do not combine dedicated and shared budgets if product rules forbid it.
- [ ] Hide or deprioritize files above a tunable fraction of budget by default.
- [ ] Allow user disable and widen. Explain why in UI.

Acceptance:

- Auto defaults have tests.
- User can disable or widen the filter.

### Phase 6 — Input-limit audit and Rust enforcement

- [ ] Inventory every user-editable field.
- [ ] Define max length, numeric min-max, charset, path safety per field.
- [ ] Enforce limits in Rust at the Tauri and domain boundary.
- [ ] Keep UI limits as hints only.
- [ ] Keep `llama-server` values within runtime `--help` capabilities.
- [ ] Reject oversize, negative, NaN, `..` paths, huge JSON without panic or hang.

Acceptance:

- All listed inputs reject bad values with clear errors.
- Tests cover oversize and malformed cases.

### Phase 7 — Real 0.3 to 0.5 change documentation

- [ ] Verify history with `git log v0.3.0..HEAD`, tags, release notes, TODO closeouts.
- [ ] Write evidence-backed 0.3.0 to 0.4.0 to 0.4.1 to 0.5.0 narrative.
- [ ] Add `## 0.5.0` to `CHANGELOG.md` at bump.
- [ ] Add short Whats new since 0.3 in README or `docs/` if useful.
- [ ] Disclose unsigned, SmartScreen, Latest fix, self-hosted CI, catalog signing, evidence, package verify, deferred Authenticode.
- [ ] Invent no features, signing, Win10 support, or perf numbers.

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
- Schema version: 1 now, 2 planned. `SUPPORTED_SCHEMA` in `src-tauri/src/catalog.rs` is 1 now.
- Cooldown default: 1560 min (planned, tunable).
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
- Pending: catalog dry-run numbers. Append after run.

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
- Planned allowlist: `catalog/providers.json`
- Catalog core: `src-tauri/src/catalog.rs`
- Published releases: <https://github.com/sato942/localmotive/releases>
- Latest API: <https://api.github.com/repos/sato942/localmotive/releases/latest>

## 8. Current acceptance status

| Material criterion | Result | Reason |
|---|---|---|
| 0.5 tracker exists and covers the mandate | PASS | File exists with phases, pointers, ledger, acceptance table. |
| Latest points at current tip while unsigned | PASS | `releases/latest` is `v0.4.1`. `v0.4.1` is `prerelease:false` with name `(unsigned)`. `gh release list` shows 0.4.1 Latest. |
| Workflow cannot mark ship as prerelease | PASS | `release.yml` sets `prerelease: false` and asserts `isPrerelease:false`. Release-gates 73 pass 0 fail. |
| README matches product and unsigned Latest | FAIL | Root `README.md` still describes 0.3.0 assets. |
| Catalog v2, signed publish, SQLite, filters | FAIL | Schema 1, hand-picked builder, no SQLite, no auto defaults. |
| Input limits enforced in Rust | UNKNOWN | Audit not started. |
| Real 0.3 to 0.5 docs | FAIL | No 0.5 narrative yet. |
| 0.5.0 ship gates | BLOCKED | Owner ship approval pending. Do not tag. |

## Conclusion

Phase 0 starts with tracker creation and Latest repair.

Complete Latest repair before catalog work.

Do not publish 0.5.0 until the owner says ship.
