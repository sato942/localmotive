# Localmotive handover

Start here. This file is the entry point for the dev team.

| Item | Value |
|---|---|
| Date | 2026-09-24 (Asia/Dubai) |
| From | tauridev (review, cleanup, and handover) |
| Base | `main` at `0bd39fb`. The lane branch `fix/u06-stabilization` is at `9b09857`, and `main` contains it. |
| Rules | [AGENTS.md](AGENTS.md). If this file and `AGENTS.md` disagree, `AGENTS.md` wins. |
| Work | [TODO.md](TODO.md): decisions, work items, and the evidence ledger. |
| Findings | [REVIEW.md](REVIEW.md): 137 findings with IDs. |

## 1. Your authority

On 2026-09-24, the owner (CEO) gave the dev team release authority. The
owner's words: "i want them to have authority to publish and not wait for me
(ceo) to interfere".

The team decides these items. No item waits for owner approval:

- the version number and each version bump;
- new tags;
- when to run release qualification;
- publication through `release-promote.yml`;
- the release scope, the disclosures, and the deferrals;
- every decision in `TODO.md` section 3.

The binding text is in `AGENTS.md`, section "CI and publication". The
tauridev agent profile has the same authority.

The delegation does not change these rules. They protect users:

- Keep secrets in Windows Credential Manager. Never print, log, or commit a
  secret.
- Publish only the bytes that the release gate qualified. Never rebuild
  during promotion.
- Never run untrusted pull-request code on the self-hosted runner.
- Ship unsigned. Disclose the unsigned files and the SmartScreen limits in
  each release.
- Claim support only for the scope that the evidence covers.
- Never move or reuse a tag.

## 2. One-time admin actions

The team cannot use its authority until the repository admin (`sato942`)
makes the changes A1 and A2. After that, no release needs the owner. GitHub
state, read on 2026-09-24:

| Setting | State | Action |
|---|---|---|
| Collaborators | `sato942` (admin) only. | A1: give each developer the Write role. Give the release lead Maintain or Admin. |
| Self-hosted runners | One: `DESKTOP-HPTF57N-zen5-blackwell`, on the owner's PC. Both release workflows need all six of its labels. | A2: register a runner that the team controls, or give the release lead Admin. Then do P1-12. |
| Environments | None. `release-promote.yml` asks only for the phrase `PUBLISH <tag>`. | None. |
| Ruleset `main-pr-check` (23218749) | Pull request required, strict `pr-check`, 0 approvals, no force push, no deletion. Bypass: owner only. | None. A Write member merges after `pr-check` passes. |
| Ruleset `immutable-release-tags` (23218751) | `refs/tags/v*`: no update, no deletion. Creation is open to Write. | None. A Write member creates tags. |

Before the first Write grant takes effect, run the direct-push refusal test
with that identity (`TODO.md` section 8, V06-GH-01.V3).

Only the owner can bypass `main-pr-check`, and a rewrite of public history
needs that bypass. Do not plan work that needs a history rewrite.

## 3. State at handover

| Item | State | Evidence |
|---|---|---|
| Last public release | `v0.5.0`, 2026-09-10 | L-02 |
| `v0.6.0` and `v0.6.1` | Tagged and never published. Both tags are burned. | L-02, L-04 |
| Release runs after `v0.5.0` | 14 started, 0 passed. Promotion never ran. | L-02 |
| Local gates | PASS: node tests 275, Vitest 282, Rust 628 passed and 7 ignored. | L-05 |
| Version in the tree | 0.6.1. The next release is 0.6.2. | `package.json` |

Why nothing shipped (details in `REVIEW.md` section 2):

1. Runtime setup needs a live, unauthenticated `api.github.com` call. The
   packaged matrix failed on it (RT-05, L-03).
2. A second concurrent catalog load returns `Busy` (RT-06). The harness
   recorded `[object Object]` instead of the error kind (LAB-01).
3. The gate needed `.hermes-0.6/`, a developer-only directory that a clean
   runner does not have (REL-04).
4. Promotion rejected the dispatch run that was chosen as the producer
   (REL-01).
5. Owner stops closed every other exit. The owner lifted them on 2026-09-24.

The change set of 2026-09-24 (section 4, step 0 lands it):

- `REVIEW.md` added. `TODO.md` rewritten.
- 17 Markdown files (1,588,153 bytes) and 2 dead scripts deleted. Git history
  keeps them at `9b09857`.
- `AGENTS.md`: the release authority, the fix-forward rule for tags, the
  release gate list, and the rule against a release lock.
- `scripts/tests/release-gates.test.mjs`: the link test scans every active
  document.
- `.github/workflows/ci.yml:10`: comment fix.
- This file.

## 4. Where to start

Do the steps in this order. Each `TODO.md` item has its acceptance criteria
and its RED test.

0. Commit the 2026-09-24 change set on `fix/u06-stabilization`. Open a PR to
   `main`. Merge it after `pr-check` passes.
1. P0-1: build the runtime catalog from `src-tauri/approved_runtimes.json`,
   with no network call.
2. P0-2: share one catalog load between concurrent callers.
3. P0-3: keep the error kind in harness evidence.
4. P0-4: fix the catalog query wire keys.
5. P0-5 and P0-6: stage release files under `$RUNNER_TEMP`, and name the
   failed step correctly.
6. P0-7: cut `release.yml` to the release gate in `AGENTS.md`.
7. P0-8: implement or delete the promised extra-file check.
8. P0-9: correct `README.md` and `CHANGELOG.md`. `v0.5.0` is the current
   release. 0.6.0 and 0.6.1 were not published.
9. P0-10: release 0.6.2 with the procedure in section 5.
10. P0-11: revert the network change on the runner host.

Then do the P1 items in order. P1-11 checks the 31 unverified High findings.

D1 is decided: Option A. `release-promote.yml` accepts the push-event run of
a new tag, so option A needs no change to promotion.

## 5. How to release

This is the only release procedure.

1. On the lane branch, bump the version in `package.json`,
   `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and
   `src-tauri/tauri.conf.json`. Run
   `node scripts/verify_versions.mjs < /dev/null`.
2. Add a `## X.Y.Z` section to `CHANGELOG.md`. `release-promote.yml` copies
   that section into the release notes. State the unsigned files and the
   SmartScreen limits there.
3. Open a PR to `main`. Merge it after `pr-check` passes.
4. Tag the merge commit and push the tag:
   `git tag vX.Y.Z <merge-sha>` and `git push origin vX.Y.Z`.
   The tag push starts `release.yml`.
5. When `release.yml` passes, run `release-promote.yml` with these inputs:
   `tag` = `vX.Y.Z`, `verify_run_id` = the run ID,
   `confirm` = `PUBLISH vX.Y.Z`, and `dry_run` = `true`.
6. When the dry run passes, run it again with `dry_run` = `false`. The
   workflow publishes the qualified bytes and reads back the public assets.
7. Add one line to the `TODO.md` ledger: the version, the peel SHA, the run
   IDs, the SHA-256 of each asset, and the open risks.

Promotion accepts only a successful push-event `Release verify` run whose
head SHA equals the tag peel. It also needs a green `pr-check` or `check` on
that commit (`release-promote.yml:86-100`). Promote within 90 days, before
the qualified bundle expires.

If a step fails after the tag:

- If the cause is transient, such as a runner or network fault, run
  `release.yml` again on the same tag.
- If the cause is in the source or in a workflow, the tag is burned. Do not
  move it. Fix the cause on `main`, bump the patch version, and start again
  at step 1. A new version costs nothing. A stuck release costs every user.

## 6. Never again

Each rule comes from a failure in the 0.6 cycle. `AGENTS.md` holds the
binding text.

| Rule | What happened |
|---|---|
| Never move or reuse a tag. Fix forward with a new patch version. | `v0.6.0` ran on 6 different SHAs in about 4 hours (L-04). After tags became immutable, a rule against a new version left `v0.6.1` with no exit. |
| No release waits for one person. | The owner stops of 2026-09-23 closed every exit from the deadlock. |
| The gate reads only the tagged tree, the outputs of its own run, and pinned fixtures. | The gate needed `.hermes-0.6/`, a developer-only directory (REL-04). |
| The gate makes no live third-party API call. | Runtime setup needed a live `api.github.com` call, and the packaged matrix failed on it (RT-05, L-03). |
| Evidence comes from the run that built the candidate. Never commit evidence as a release precondition. | Three builds of one source gave three different digests. Attempt 6 of `v0.6.1` had 12 of 18 records missing (L-04). |
| Hardware campaigns, fault injection, soak cycles, and lab witnesses never block a release. | The 18-record campaign made the gate depend on lab evidence that a clean runner cannot produce (P0-7). |
| Promotion accepts exactly what the release procedure produces. Change both in one PR, and test the promote validator on a real verify bundle. | Promotion needed a push-event run, but the chosen producer was a dispatch run (REL-01). |
| Run a gate change end to end on the runner before a release depends on it. | Release steps #24 to #27 did not run in the 0.6.1 dispatch runs (L-03). |
| Keep one packaged matrix and one CDP client (`scripts/lib/cdp_client.mjs`). Add no version-named verifier or campaign script. | `verify_041.mjs` and the `g05_*` scripts grew beside the matrix. `scripts/` holds 89 scripts (LAB-03, LAB-05). |
| Test behavior. Do not assert prose or source text. | `release-gates.test.mjs` grew to 3,230 lines. Its prose pins had to be cut before 17 documents could be deleted (L-05). |
| Keep audit ticket IDs out of source, test names, and function names. | Rust source holds 399 ticket labels (P2-5). |
| Keep one tracker (`TODO.md`), one review (`REVIEW.md`), and this handover. Add no reports, maps, or snapshots. | 17 Markdown files were deleted on 2026-09-24. |
| Never commit raw captures. Upload them as CI artifacts. | `artifacts/` holds 5,494 tracked files, including 33 WebView2 profiles (DOC-01). |
| Record the exact error before a change. Diagnose from the error kind. | The harness recorded `[object Object]` (LAB-01). The old tracker called the `Busy` failure environmental, but a second concurrent call reproduces it (RT-06). |
| Keep the verify job at 60 minutes or less. If the gate needs more time, cut the gate. | The verify job allows 360 minutes (`release.yml:96`). |
| If rules together block every path to a release, change a blocking rule in the same PR as the fix, and record the reason in `TODO.md`. | See the first two rows. |

## 7. Where things are

| Path | Content |
|---|---|
| `AGENTS.md` | Binding rules: product boundaries, protections, tests, CI, release authority, and the release gate. |
| `TODO.md` | Decisions, work items, risks, deferrals, and the evidence ledger. The only tracker. |
| `REVIEW.md` | Findings by ID, the causes of failure, the release gate, and the Markdown disposition. |
| `CHANGELOG.md` | Release notes. Promotion reads the version section. |
| `docs/` | Product, runtime manager, design, support and evidence matrices, supply chain, security limits, artifact identity, and catalog promotion. |
| `catalog/README.md` | Catalog schema, signing, and curation. |
| `.github/workflows/ci.yml` | `pr-check` on `windows-latest` for untrusted PRs. Trusted jobs on the self-hosted runner. |
| `.github/workflows/release.yml` | Tag push: build, verify, and retain the candidate. It does not publish. |
| `.github/workflows/release-promote.yml` | Dispatch: validate the qualified bundle and publish the same bytes. |
| `.github/workflows/hardware-qualify.yml` | Hardware runs on the GPU host. They never block a release. |
| `.github/workflows/catalog.yml` | Dispatch: build or re-sign the curated catalog, on `ubuntu-latest`. |
| `scripts/verify_packaged_matrix.ps1` | The packaged matrix. Extend it. Do not add another verifier. |

## 8. Host and tool notes

- Run commands from the repository root in Git Bash. `AGENTS.md`, section
  "Tests and verification", lists the full check set.
- On the runner host, run Node with `< /dev/null`. Without it, some scripts
  fail with `stdin is not a tty`. That is a tooling failure, not a product
  failure.
- Use forward-slash native paths, such as `C:/...`, for native tools.
- The runner directory is `C:/actions-runner-localmotive`. `AGENTS.md` names
  `localmotive-control/start-runner.ps1`. That script is not in this
  repository, and it was not found at `C:/localmotive-control`. Its location
  is UNKNOWN.
- Keep the runner's `CARGO_HOME` and `RUSTUP_HOME` separate from the owner's.
  Never clean the owner's toolchains.
- The runner host has a temporary network change from 2026-09-22 (P0-11).
- `research/` is ignored. Do not modify it.
- The open risks are in `TODO.md` section 7.
