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
