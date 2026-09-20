# Agent feedback log

Compressed 2026-09-20. Full history lives in git (commits 6099c6f through
3e79097). Lessons rated 8+/10 for trouble prevention now live in AGENTS.md:
restore single files from a copied backup, and count harness failures as
tooling failures. Lesser tool quirks were discarded. Kanban is banned in
AGENTS.md; the old kanban-environment entries below are void.

## Latest verification snapshot (2026-09-20, head 9436aec)

- cargo fmt clean, clippy `-D warnings` clean, 622 Rust tests pass (7 ignored).
- release-gates script tests 164/164 pass. tsc and diff-check clean.
- Releases ship unsigned as standing policy. No signing step exists.

## Standing open risks

- Startup cleanup: the post-spawn local-client constructor can return before
  `clear_starting` runs. Needs a controlled reproduction.
- Upstream LoRA parsing: pinned runtime splits scaled entries on every colon,
  so a Windows drive path stays unproven. Reproduce with qualified runtime
  bytes before proposing a correction.
- Legacy ownership corrections need an independent re-review.
- A file once vanished from this checkout without an owning command and was
  restored from HEAD. Watch for another worker sharing this checkout.

## Closed this session

- Legacy benchmark ownership review: final cancellation gate, early slot
  publication, fail-closed slot guard. RED-before-GREEN with mutation proof.
- Release signing pursuit dropped. Signing capability deleted from the tree.
- Branches trimmed to main plus the active lane. Dependabot PRs closed.
- Trackers merged into TODO.md. Freeze-9 handoff removed by owner order.

## Investigation 2026-09-20: isolated Windows environment blocker

- Host reports Windows 10 IoT Enterprise, build 26100. Sandbox binaries
  exist. A hypervisor runs on this host.
- Sandbox booted here before (2026-09-15 evidence). The recorded failure was
  CO_E_APPSINGLEUSE: a second instance cannot start while one runs.
- A Sandbox server held the single slot since 2026-09-15 09:20 (PID 62780),
  an orphan from the September test campaign. Terminated it by PID with
  MSYS_NO_PATHCONV=1 taskkill. Zero Sandbox processes remain.
- Blocker removed. Next step: run the U06-04 lifecycle legs while the slot
  is free. Note: taskkill needs MSYS_NO_PATHCONV=1 under Git Bash, else
  /PID mangles into a path.
