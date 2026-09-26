# Team-controlled runner pack

This pack provisions a NEW Windows 11 x64 machine as the team-controlled
self-hosted runner for `release.yml` / `release-promote.yml` (P1-12, A2).
The owner runner (`DESKTOP-HPTF57N-zen5-blackwell`) stays untouched until
the team runner goes green; only then is its `localmotive-release` label
removed.

Files:

- `provision-team-runner.ps1` — idempotent provisioner. Run it in an
  elevated PowerShell on the NEW machine. It checks and installs Git,
  Node.js LTS, and Rustup (via winget), creates
  `C:\actions-runner-team`, points machine-level `CARGO_HOME` and
  `RUSTUP_HOME` at team-local directories (never the owner's toolchains),
  and downloads the latest GitHub Actions runner. It never asks for and
  never stores the registration token.
- `start-team-runner.ps1` — template that starts the runner interactively
  (hidden window, 24/7). Copy it next to the runner root and adjust the
  path if you changed the install directory.
- `REGISTER.md` — the human procedure: token, `config.cmd` line with the
  exact label set, first-run verification, the `release.yml` dispatch that
  closes P1-12, owner-runner label removal, and rollback.

Safety rules:

- The registration token comes from the GitHub UI
  (Settings → Actions → Runners → New self-hosted runner) and is typed by
  the human into `config.cmd`. It never enters a file, a log, or a chat.
- The team runner carries `localmotive-release`. While the owner runner
  still carries that label, jobs may land on either machine: treat the
  first green dispatch as provisional and remove the owner label the same
  day (REGISTER.md step 5).
- Never run the provisioner on the owner's PC. It refuses to run on a
  machine whose hostname is `DESKTOP-HPTF57N`.
