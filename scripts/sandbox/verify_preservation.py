#!/usr/bin/env python3
"""G-06 preservation verifier (R07, follow-up review db548c8; F9-05).

Proves that REAL application data created for the released baseline survives
the upgrade to 0.6, not merely that a marker file exists:

* mirror flavor (v0.5.0+): the collected catalog mirror must still carry the
  v0.5.0-schema user-override rows with their ownership flags and sentinel
  values. Network (non-user) rows may legitimately be refreshed by the 0.6
  app; user rows are device data and must be kept.
* cache flavor (v0.4.1): v0.4.1 predates the SQLite mirror. Its persisted
  catalog state is the cache record JSON and its SETTINGS/PROFILE state is
  WebView2 localStorage under the v0.4.1 keys. The cache flavor therefore
  requires BOTH:
  - the cache record must still exist and parse as a cache record whose
    schemaVersion is the released contract's version (1). A record whose
    schemaVersion is absent or null is NOT a v0.4.1 cache record;
  - the recovered settings/profile values must equal the values the fixture
    seeded. Cache survival alone does not satisfy the profile/settings
    requirement (F9-05), so the collected reads are compared against the
    fixture's expected values.

Both flavors also check the user-data canary file. Usage:
  python scripts/sandbox/verify_preservation.py <mirror.sqlite> <userdata.txt> <cache.json> --flavor mirror|cache
  python scripts/sandbox/verify_preservation.py <mirror.sqlite> <userdata.txt> <cache.json> --flavor cache \
      --settings-fixture scripts/sandbox/canary-settings.json --settings <collected-settings.json>
Exits 0 on PASS, 1 on FAIL, 2 on usage error.
"""

import json
import sqlite3
import sys

USER_SENTINEL = "USER-OVERRIDE-SENTINEL-0.6-preservation"
USER_SHA = "a" * 64
CACHE_SCHEMA_VERSION = 1


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
            failures.append(
                f"cache record shape is not a CacheRecord: keys={list(record) if isinstance(record, dict) else type(record)}"
            )
            return
        body = json.loads(record["body"])
        if not isinstance(body, dict):
            failures.append("cache record body is not a catalog document")
            return
        version = body.get("schemaVersion")
        # The released v0.4.1 build only ever wrote schemaVersion 1. A missing
        # or null value means the record is not the released contract's cache
        # record, so accepting it would prove nothing about recovery.
        if version != CACHE_SCHEMA_VERSION:
            failures.append(
                f"cache record schemaVersion {version!r} != {CACHE_SCHEMA_VERSION} (the v0.4.1 contract)"
            )
            return
        print(f"PRESERVE_PASS cache: record present and parseable (schemaVersion {version})")
    except Exception as error:  # noqa: BLE001 - report any read failure
        failures.append(f"cache record missing or unreadable: {error}")


def check_settings(fixture_path: str, collected_path: str, failures: list) -> None:
    """F9-05: the recovered profile/settings values must equal the seeded ones."""
    try:
        with open(fixture_path, encoding="utf-8") as handle:
            fixture = json.load(handle)
    except Exception as error:  # noqa: BLE001 - report any read failure
        failures.append(f"settings fixture unreadable: {error}")
        return
    try:
        with open(collected_path, encoding="utf-8") as handle:
            collected = json.load(handle)
    except Exception as error:  # noqa: BLE001 - report any read failure
        failures.append(f"collected settings unreadable (profile/settings not recovered): {error}")
        return
    if not isinstance(collected, dict) or not isinstance(collected.get("reads"), dict):
        failures.append("collected settings do not carry a reads map")
        return
    reads = collected["reads"]
    expect = fixture.get("expect") or {}
    expected_settings = expect.get("settings") or {}
    expected_profile = expect.get("profile") or {}
    if not expected_settings or not expected_profile:
        failures.append("settings fixture carries no expected values to compare against")
        return
    for name, value in expected_settings.items():
        key = f"localmotive:{name}"
        actual = reads.get(key)
        if actual != value:
            failures.append(f"setting {key} not recovered: {actual!r} != {value!r}")
        else:
            print(f"PRESERVE_PASS settings: {key} recovered")
    profile_key = "localmotive:profile:fixture/v041-legacy-model"
    raw = reads.get(profile_key)
    if not isinstance(raw, str):
        failures.append(f"profile record {profile_key} lost: {raw!r}")
        return
    try:
        profile = json.loads(raw)
    except Exception as error:  # noqa: BLE001 - report the parse failure
        failures.append(f"profile record {profile_key} unparseable: {error}")
        return
    if not isinstance(profile, dict):
        failures.append(f"profile record {profile_key} is not a profile object")
        return
    for field, value in expected_profile.items():
        actual = profile.get(field)
        if actual != value:
            failures.append(f"profile field {field} not recovered: {actual!r} != {value!r}")
        else:
            print(f"PRESERVE_PASS profile: {field} recovered")
    # The tuning record belongs to the same released schema; losing it means
    # the upgrade dropped user-owned tuning state.
    tuning_key = "localmotive:tuning:fixture/v041-legacy-model"
    if not isinstance(reads.get(tuning_key), str):
        failures.append(f"tuning record {tuning_key} lost: {reads.get(tuning_key)!r}")


def main() -> int:
    args = sys.argv[1:]
    flavor = "mirror"
    if "--flavor" in args:
        index = args.index("--flavor")
        flavor = args[index + 1]
        del args[index:index + 2]
    settings_fixture = None
    if "--settings-fixture" in args:
        index = args.index("--settings-fixture")
        settings_fixture = args[index + 1]
        del args[index:index + 2]
    settings_collected = None
    if "--settings" in args:
        index = args.index("--settings")
        settings_collected = args[index + 1]
        del args[index:index + 2]
    if len(args) != 3:
        print(
            "usage: verify_preservation.py <mirror.sqlite> <userdata.txt> <cache.json> --flavor mirror|cache "
            "[--settings-fixture <fixture.json> --settings <collected.json>]",
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
        if not settings_fixture or not settings_collected:
            failures.append(
                "cache flavor requires --settings-fixture and --settings: the v0.4.1 profile/settings "
                "recovery must be asserted, not just cache survival"
            )
        else:
            check_settings(settings_fixture, settings_collected, failures)
    check_userdata(userdata, failures)
    for failure in failures:
        print(f"PRESERVE_FAIL {failure}", file=sys.stderr)
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
