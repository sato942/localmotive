# Superseded candidate evidence: freeze-8 (source da091a4)

This directory preserves the freeze-8 candidate evidence exactly as it was
committed at `da091a4`, before the freeze-9 re-cut at `eb01bc9`.

- `qualification-manifest-0.6.0.json` and `candidate-inventory-0.6.0.json`
  describe the freeze-8 staged artifacts (portable digest `f46ed292...`,
  setup `fd33cb12...`, MSI `c1679979...`).
- `attestations/` holds every record that manifest referenced.

Freeze-8 was superseded because the R16 remediation changed product code
(benchmark ownership through cleanup, local-client transports) and the
release path (verify-only tag flow plus an authorized promotion workflow),
so the candidate bytes, the harness revisions and the release workflow all
moved. The freeze-9 evidence replaces the canonical paths in
`release-evidence/0.6.0/`; nothing here may be relabelled as a measurement
of the freeze-9 bytes.
