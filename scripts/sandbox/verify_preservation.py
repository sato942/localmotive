#!/usr/bin/env python3
"""G-06 preservation verifier: proves the collected catalog mirror survived an
installer upgrade with its canary marker intact, and that the user-data
canary file survived with matching content. Usage:
  python scripts/sandbox/verify_preservation.py <collected-mirror.sqlite> <collected-userdata.txt>
Exits 0 on PASS, 1 on FAIL, 2 on usage error."""
import sqlite3
import sys

def main() -> int:
    if len(sys.argv) != 3:
        print("usage: verify_preservation.py <mirror.sqlite> <userdata.txt>", file=sys.stderr)
        return 2
    mirror, userdata = sys.argv[1], sys.argv[2]
    failures = []
    try:
        con = sqlite3.connect(f"file:{mirror}?mode=ro", uri=True)
        rows = con.execute("SELECT label, planted_at FROM canary_marker").fetchall()
        con.close()
        if rows != [("g06-preservation-canary", "2026-09-11T00:00:00Z")]:
            failures.append(f"mirror marker rows unexpected: {rows!r}")
        else:
            print("PRESERVE_PASS mirror: canary_marker intact")
    except Exception as error:
        failures.append(f"mirror unreadable: {error}")
    try:
        with open(userdata, encoding="utf-8") as handle:
            content = handle.read()
        if content != "g06-preservation-userdata-canary\n":
            failures.append(f"userdata canary content mismatch: {content!r}")
        else:
            print("PRESERVE_PASS userdata: canary content intact")
    except Exception as error:
        failures.append(f"userdata canary unreadable: {error}")
    for failure in failures:
        print(f"PRESERVE_FAIL {failure}", file=sys.stderr)
    return 1 if failures else 0

if __name__ == "__main__":
    raise SystemExit(main())
