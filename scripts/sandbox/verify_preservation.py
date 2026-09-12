#!/usr/bin/env python3
"""G-06 preservation verifier (R07, follow-up review db548c8).

Proves that REAL application data created for the released baseline survives
the upgrade to 0.6, not merely that a marker file exists:

* mirror flavor (v0.5.0+): the collected catalog mirror must still carry the
  v0.5.0-schema user-override rows with their ownership flags and sentinel
  values. Network (non-user) rows may legitimately be refreshed by the 0.6
  app; user rows are device data and must be kept.
* cache flavor (v0.4.1): v0.4.1 predates the SQLite mirror, so its persisted
  catalog state is the cache record JSON. The 0.6 application must still
  honor that location: the record must exist and parse as a cache record
  (refreshed content is acceptable; deletion or corruption is not).

Both flavors also check the user-data canary file. Usage:
  python scripts/sandbox/verify_preservation.py <mirror.sqlite> <userdata.txt> <cache.json> --flavor mirror|cache
Exits 0 on PASS, 1 on FAIL, 2 on usage error.
"""

import json
import sqlite3
import sys

USER_SENTINEL = "USER-OVERRIDE-SENTINEL-0.6-preservation"
USER_SHA = "a" * 64


def check_userdata(userdata: str, failures: list) -> None:
    try:
        with open(userdata, encoding="utf-8") as handle:
            content = handle.read()
        if content != "g06-preservation-userdata-canary\n":
            failures.append(f"userdata canary content mismatch: {content!r}")
        else:
            print("PRESERVE_PASS userdata: canary content intact")
    except Exception as error:  # noqa: BLE001 - report any read failure
        failures.append(f"userdata canary unreadable: {error}")


def check_mirror(mirror: str, failures: list) -> None:
    try:
        connection = sqlite3.connect(f"file:{mirror}?mode=ro", uri=True)
        version = connection.execute("PRAGMA user_version").fetchone()[0]
        if version != 1:
            failures.append(f"mirror schema version {version} != 1 (the v0.5.0 schema)")
        else:
            print("PRESERVE_PASS mirror: schema version 1 intact")
        model = connection.execute(
            "SELECT summary, user_sourced FROM catalog_model WHERE id = 'fixture/user-override'"
        ).fetchall()
        if model != [(USER_SENTINEL, 1)]:
            failures.append(f"user override model row lost or altered: {model!r}")
        else:
            print("PRESERVE_PASS mirror: user override model row recovered with user_sourced=1")
        files = connection.execute(
            "SELECT sha256, user_sourced FROM catalog_file WHERE model_id = 'fixture/user-override'"
        ).fetchall()
        if files != [(USER_SHA, 1)]:
            failures.append(f"user override file row lost or altered: {files!r}")
        else:
            print("PRESERVE_PASS mirror: user override file row recovered with user_sourced=1")
        owned = connection.execute(
            "SELECT COUNT(*) FROM catalog_model WHERE user_sourced = 1"
        ).fetchone()[0]
        if owned < 1:
            failures.append("no user-sourced model rows remain after the upgrade")
        connection.close()
    except Exception as error:  # noqa: BLE001 - report any read failure
        failures.append(f"mirror unreadable: {error}")


def check_cache(cache: str, failures: list) -> None:
    try:
        with open(cache, encoding="utf-8") as handle:
            record = json.load(handle)
        if not isinstance(record, dict) or "body" not in record:
            failures.append(f"cache record shape is not a CacheRecord: keys={list(record) if isinstance(record, dict) else type(record)}")
            return
        body = json.loads(record["body"])
        if not isinstance(body, dict) or "schemaVersion" not in body:
            failures.append("cache record body is not a catalog document")
            return
        print(
            f"PRESERVE_PASS cache: record present and parseable (schemaVersion {body['schemaVersion']})"
        )
    except Exception as error:  # noqa: BLE001 - report any read failure
        failures.append(f"cache record missing or unreadable: {error}")


def main() -> int:
    args = sys.argv[1:]
    flavor = "mirror"
    if "--flavor" in args:
        index = args.index("--flavor")
        flavor = args[index + 1]
        del args[index:index + 2]
    if len(args) != 3:
        print(
            "usage: verify_preservation.py <mirror.sqlite> <userdata.txt> <cache.json> --flavor mirror|cache",
            file=sys.stderr,
        )
        return 2
    if flavor not in ("mirror", "cache"):
        print(f"unknown flavor: {flavor}", file=sys.stderr)
        return 2
    mirror, userdata, cache = args
    failures: list = []
    if flavor == "mirror":
        check_mirror(mirror, failures)
    else:
        check_cache(cache, failures)
    check_userdata(userdata, failures)
    for failure in failures:
        print(f"PRESERVE_FAIL {failure}", file=sys.stderr)
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
