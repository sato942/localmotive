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

## Release integrity

Public releases ship **unsigned with an explicit disclosure**; verify the
published SHA-256 checksums before use. Signing is deferred, tracked in the
repository, and never claimed where it did not happen.
