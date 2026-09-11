# Security review limits (audit S-28)

This record states exactly what was inspected, with which tools and when, and
what could not be inspected. An unavailable scan is not a clean result.

## 1. Full-history secret review

- Scope: `git log --all -p` over all 246 commits of every ref, 8 922 814 bytes
  of unified diff, plus a private-key path listing over the whole history.
- Date: 2026-09-11. Environment: local clone `C:\Users\Mubarak\Documents\Projects\localmotive`.
- Tooling gap: no dedicated history scanner (`gitleaks`, `trufflehog`,
  `ggshield`) is installed on this machine. The review therefore used a
  bounded pattern pass (provider prefixes `AKIA`/`ASIA`, `ghp_`, `gho_`,
  `github_pat_`, `sk-`, `AIza`, `xox[baprs]-`, `hf_`, `Bearer` literals,
  private-key PEM headers, `password=` assignments). A pattern pass is weaker
  than an entropy-and-rule scanner; the tooling gap remains open.
- Matches (6 diff lines, sanitized):
  1. Two lines: a `hf_`-shaped 39-character string used as a `validate_hf_token`
     test fixture (added with S-07, commit `9757d8d`). Its provenance cannot be
     verified from the repository; it is shaped like a real Hugging Face token.
     The fixture has been replaced in the working tree with an obviously
     synthetic literal, the occurrence is public in history, and a private
     owner-advisory note was recorded outside this public document (see
     section 4). No value appears in this document or in the tracker.
  2. Two lines: the literal placeholder text `[REDACTED PRIVATE KEY]` used by
     sanitizer tests. Not key material.
  3. Two lines: test-only PEM private keys embedded in `local_client.rs` for
     the MT-06 TLS-pinning tests, generated locally for the test suite. They
     authenticate nothing outside the tests, and no production or third-party
     system accepts them.
- Path level: no `.pem`, `.key`, `.p12`, `.pfx`, `id_rsa` or `id_ed25519`
  file exists anywhere in the history.

## 2. Repository security settings review (read via GitHub API as the owner)

Snapshot of `GET /repos/sato942/localmotive` `security_and_analysis`,
2026-09-11:

| Setting | State |
|---|---|
| Secret scanning | enabled |
| Secret scanning push protection | enabled |
| Secret scanning non-provider patterns | disabled |
| Secret scanning validity checks | disabled |
| Dependabot security updates | disabled |

- Open secret-scanning alerts: 0 (read via `GET /repos/sato942/localmotive/secret-scanning/alerts`).
  Zero open alerts is a platform observation, not proof that history is clean:
  non-provider patterns and validity checks are off, and platform detection
  cannot classify a token the platform does not know.
- GitHub-side coverage is incomplete by configuration; enabling non-provider
  patterns and validity checks is an owner settings decision, not a code change.

## 3. Runner isolation assessment

- Reviewed artifact: `C:\actions-runner-localmotive\localmotive-control\start-runner.ps1`
  (plus `start-runner-hidden.vbs`, `stop-runner.ps1`).
- Finding: the script starts the interactive `Runner.Listener` hidden, checks
  for an already-running listener by executable path, and sets process-level
  `CARGO_HOME=C:\actions-runner-localmotive\cargo-home` and
  `RUSTUP_HOME=C:\actions-runner-localmotive\rustup-home`. The machine-level
  variables point at the same directories (belt and suspenders). Both
  directories are populated (cargo registry, advisory database, rustup
  settings), so the isolation is in use, not merely declared.
- Discrepancy recorded: a second, stale copy of a `start-runner.ps1` exists at
  `C:\Users\Mubarak\actions-runner\localmotive-control\`. That copy starts the
  Windows service `actions.runner.sato942-localmotive.DESKTOP-HPTF57N-zen5-blackwell`,
  which is currently disabled and stopped; its binary path is
  `C:\actions-runner-localmotive\bin\RunnerService.exe` and its account is
  `NT AUTHORITY\NETWORK SERVICE`. Running the stale script would attempt a
  different execution identity than the documented interactive listener. The
  stale copy was not modified (outside this repository).
- Live state at assessment time (2026-09-11T15:45Z): the runner service is
  stopped and disabled, and no `Runner.Listener`/`Runner.Worker` process was
  running. Remote execution during that window came from other agents of the
  same machine only if a listener had been started; this record notes the
  state instead of asserting continuous availability.
- Not assessed: the GitHub-side runner group permissions, and whether other
  accounts on this machine can write to the runner directories. Both remain
  unverified.

## 4. Private remediation record

The `hf_`-shaped fixture and the stale-runner-script discrepancy are recorded
privately (with the string's sha256 prefix and history commit) in the
git-ignored review log kept during this work. Guidance for the owner: if that
fixture string is ever discovered to have been a live Hugging Face token,
revoke it at `huggingface.co/settings/tokens` and treat it as burned in public
history - rewriting public history is an owner decision, not an automatic
step. No secret value is copied into this document, the tracker, or any
committed file.

## 5. Remaining access gaps

- No dedicated secret scanner is installed; the pattern pass is the weaker
  substitute and remains the review's main limitation.
- Non-provider secret patterns, validity checks and Dependabot security
  updates are disabled in repository settings (owner decision to change).
- Runner group permissions and local directory ACLs are not assessed.
- Third-party dependency and license review is not duplicated here; it is
  tracked under `docs/SUPPLY-CHAIN.md` and GH-08.
