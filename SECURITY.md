# Security policy

## Reporting a vulnerability

Use GitHub's **private vulnerability reporting** form on this repository
(Security → Report a vulnerability). If the form is unavailable, open a
minimal public issue that asks the maintainer to start a private channel —
do not put exploit details, credentials, local paths, or model data in a
public issue.

Include:

- the affected version (from `package.json`) and how it was installed;
- the smallest reproduction you can give;
- what an attacker gains, and which asset is affected (credentials, downloads,
  runtime installs, evidence files, the release pipeline).

## What the project already treats as a defect

These are security-relevant by this repository's rules (see `AGENTS.md`):

- a secret written to disk, logs, the frontend state, or a command line;
- a download accepted without its size and SHA-256 verification, or published
  over an existing file;
- managed-content execution that bypasses the compiled-content trust check;
- a Tauri command that accepts an unreviewed command line from the frontend;
- a release artifact whose checksum, provenance, or version identity does not
  match its source revision.

## Scope

Localmotive is a local Windows control plane. Reports about upstream
`llama.cpp` behavior, Hugging Face hosting, or cloud provider services should
go to those projects. The catalog's Ed25519 signature authenticates the
catalog document only; it does not authenticate application installers, and
reporting guidance must not treat it as doing so.

## Response

The maintainer responds on a best-effort basis. There is no bug bounty.
Valid reports are fixed on `main` with a regression test, and the fix is
credited in the changelog unless you ask otherwise.

## Verification mode and WebView2 remote debugging

The packaged verifier exercises the app with controlled fixtures. It sets
these variables only for the candidate process it starts, under an isolated
application-data profile:

- `LOCALMOTIVE_VERIFY_ISOLATED_ROOT`: the gate. Without it the app ignores
  every variable below and verifies against the shipped catalog key and the
  canonical download host.
- `LOCALMOTIVE_CATALOG_URL`: loopback HTTP fixture endpoint only
  (`http://127.0.0.1:` or `http://localhost:`); anything else is refused.
- `LOCALMOTIVE_CATALOG_PUBKEY`: 32-byte fixture signing key, hex-encoded.
- `LOCALMOTIVE_CATALOG_ROOT`: fixture cache root, required to sit inside the
  isolated root.
- `LOCALMOTIVE_HF_BASE`: loopback HTTP base for downloads; anything else
  falls back to `https://huggingface.co`.
- `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`: standard WebView2 behavior, not an
  application setting. The verifier sets
  `--remote-debugging-port=<port> --user-data-dir=<isolated profile>` so the
  packaged matrix can drive the candidate over CDP. A normal launch never
  sets this variable and must expose no debugger endpoint: do not set it
  outside the isolated verifier profile, and treat any unexpected remote
  debugging port on a production run as a defect.

While any authority override is active the UI shows a VERIFICATION MODE
banner (backed by the read-only `verification_mode` command). A run without
the banner verifies the catalog against the shipped key and downloads from
the canonical host.

## Release integrity

Public releases ship **unsigned with an explicit disclosure**; verify the
published SHA-256 checksums before use. Signing is deferred, tracked in the
repository, and never claimed where it did not happen.

## Review limits

The residual boundaries of the secret and runner reviews, with what could and
could not be inspected, are recorded in `docs/SECURITY-REVIEW-LIMITS.md`.
