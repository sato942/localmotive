#!/usr/bin/env python3
"""Build the lifecycle preservation fixtures (R07, follow-up review db548c8).

The old fixture was a marker-only SQLite file: any non-empty database with a
canary table satisfied the host verifier, so the preservation leg proved file
survival rather than recovery of real application data. This generator builds
fixtures that are derived from the RELEASED products themselves:

* `canary-mirror.sqlite.b64` - a catalog mirror that matches the v0.5.0
  schema exactly. The DDL below is copied verbatim from
  `git show v0.5.0:src-tauri/src/catalog_db.rs` (release source a4b7127f,
  `catalog-mirror.sqlite`, PRAGMA user_version = 1). It carries one verified
  (network) row and one USER OVERRIDE row (`user_sourced = 1`) whose summary
  is a sentinel. After the upgrade, the 0.6 application must have kept the
  user row and its ownership flag.

* `canary-catalog-cache.json` - a catalog cache record in the v0.4.1 format
  (`{"body": "<catalog JSON>", "etag": null}` with schemaVersion 1), copied
  from the v0.4.1 source's CacheRecord and Catalog shapes
  (`git show v0.4.1:src-tauri/src/catalog.rs`). v0.4.1 predates the SQLite
  mirror, so this is the data that version really persisted.

Regeneration is deterministic in content (ids, sentinels, digests); the exact
SQLite page bytes depend on the local sqlite3 version, so regenerate only
when the schema or sentinels must change, and commit the regenerated .b64.

Usage (from the repository root):
    python scripts/sandbox/build_preservation_fixture.py
"""

import base64
import io
import json
import pathlib
import sqlite3
import sys

SENTINEL = "USER-OVERRIDE-SENTINEL-0.6-preservation"
NODE_SENTINEL = "NODE-ROW-SENTINEL-0.6-preservation"
USER_SHA = "a" * 64
NODE_SHA = "b" * 64

# Verbatim from v0.5.0 src-tauri/src/catalog_db.rs (release a4b7127f). Keep in
# sync only by citing a new release source; never edit silently.
SCHEMA_V0_5_0 = """
CREATE TABLE catalog_model (
    id TEXT PRIMARY KEY,
    repo TEXT NOT NULL,
    family TEXT NOT NULL DEFAULT '',
    parameters TEXT NOT NULL DEFAULT '',
    publisher TEXT NOT NULL DEFAULT '',
    author TEXT NOT NULL DEFAULT '',
    summary TEXT NOT NULL DEFAULT '',
    tags_json TEXT NOT NULL DEFAULT '[]',
    gated INTEGER NOT NULL DEFAULT 0,
    downloads INTEGER NOT NULL DEFAULT 0,
    likes INTEGER NOT NULL DEFAULT 0,
    license TEXT NOT NULL DEFAULT '',
    pipeline_tag TEXT NOT NULL DEFAULT '',
    library_name TEXT NOT NULL DEFAULT '',
    architecture TEXT NOT NULL DEFAULT '',
    last_modified TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT '',
    user_sourced INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE catalog_file (
    filename_lower TEXT PRIMARY KEY,
    model_id TEXT NOT NULL REFERENCES catalog_model(id) ON DELETE CASCADE,
    quant TEXT NOT NULL DEFAULT '',
    filename TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    sha256 TEXT NOT NULL,
    revision TEXT NOT NULL DEFAULT 'main',
    last_modified TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT '',
    user_sourced INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX catalog_file_model_idx ON catalog_file(model_id);
PRAGMA user_version = 1;
"""


def build_mirror() -> bytes:
    path = pathlib.Path(".hermes-0.6/r07-fixture/catalog-mirror.sqlite")
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    connection = sqlite3.connect(path)
    connection.executescript(SCHEMA_V0_5_0)
    connection.execute(
        "INSERT INTO catalog_model (id, repo, family, parameters, publisher, summary,"
        " tags_json, downloads, likes, license, user_sourced)"
        " VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0)",
        (
            "fixture/verified-node",
            "fixture/verified-node",
            "fixture",
            "7B",
            "fixture",
            NODE_SENTINEL,
            json.dumps(["fixture"]),
            1,
            2,
            "apache-2.0",
        ),
    )
    connection.execute(
        "INSERT INTO catalog_file (filename_lower, model_id, quant, filename, size_bytes,"
        " sha256, revision, user_sourced)"
        " VALUES (?, ?, ?, ?, ?, ?, 'main', 0)",
        ("fixture-verified-q4.gguf", "fixture/verified-node", "Q4_K_M", "fixture-verified-Q4_K_M.gguf", 1024, NODE_SHA),
    )
    connection.execute(
        "INSERT INTO catalog_model (id, repo, family, parameters, publisher, summary,"
        " tags_json, downloads, likes, license, user_sourced)"
        " VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1)",
        (
            "fixture/user-override",
            "fixture/user-override",
            "fixture",
            "1.5B",
            "user",
            SENTINEL,
            json.dumps(["fixture", "user"]),
            0,
            0,
            "mit",
        ),
    )
    connection.execute(
        "INSERT INTO catalog_file (filename_lower, model_id, quant, filename, size_bytes,"
        " sha256, revision, user_sourced)"
        " VALUES (?, ?, ?, ?, ?, ?, 'main', 1)",
        ("fixture-user-q5.gguf", "fixture/user-override", "Q5_K_M", "fixture-user-Q5_K_M.gguf", 2048, USER_SHA),
    )
    connection.commit()
    connection.close()
    return path.read_bytes()


def build_cache_record() -> bytes:
    # v0.4.1 CacheRecord { body, etag } with a schemaVersion-1 catalog body.
    body = {
        "schemaVersion": 1,
        "updated": "2024-12-01",
        "source": "https://github.com/sato942/localmotive/blob/main/catalog/catalog.json",
        "note": "Curated list of GGUF builds.",
        "models": [
            {
                "id": "fixture/v041-node",
                "repo": "fixture/v041-node",
                "family": "fixture",
                "parameters": "8B",
                "publisher": "fixture",
                "summary": "v0.4.1-era cache record staged for the upgrade",
                "tags": ["fixture"],
                "gated": False,
                "downloads": 0,
                "likes": 0,
                "files": [
                    {
                        "quant": "Q4_K_M",
                        "filename": "fixture-v041-Q4_K_M.gguf",
                        "sizeBytes": 1024,
                        "sha256": "c" * 64,
                        "revision": "main",
                    }
                ],
            }
        ],
    }
    record = {"body": json.dumps(body, indent=2), "etag": None}
    return json.dumps(record, indent=2).encode("utf-8")


def build_settings_record() -> dict:
    """The v0.4.1 SETTINGS/PROFILE fixture (F9-05).

    v0.4.1 persisted settings through WebView2 localStorage under these exact
    keys (`git show v0.4.1:src/App.tsx`, readRecord calls): model-root,
    runtime, cloud-provider, cloud-model, per-model `profile:<id>` and
    `tuning:<id>` records, and per-alias `benchmark:<alias>` records. The
    profile below carries a v0.4.1-shaped LaunchProfile subset
    (`git show v0.4.1:src/model.ts`). The values are sentinels: after the
    upgrade the application must recover THESE values, so the host verifier
    compares the collected reads against this fixture rather than accepting a
    surviving file or marker.
    """
    profile = {
        "name": "v041-fixture / recovered",
        "runtime": "C:/fixtures/v041/llama-server.exe",
        "model": "C:/fixtures/v041/models/legacy-Q4_K_M.gguf",
        "draftModel": None,
        "mmproj": None,
        "host": "127.0.0.1",
        "port": 8123,
        "alias": "v041-legacy",
        "context": 6144,
        "parallel": 1,
        "gpuLayers": "all",
        "threads": 6,
        "threadsBatch": 6,
        "batch": 448,
        "ubatch": 112,
        "flashAttention": "auto",
        "fit": False,
        "cacheTypeK": "f16",
        "cacheTypeV": "f16",
        "temperature": 0.73,
        "seed": 424242,
        "metrics": True,
        "slots": True,
        "reasoningBudget": 777,
        "extraArgs": [],
    }
    # v0.4.1 TuneReport shape (src/App.tsx: baselineTps/bestTps drive the
    # gain notice; the current app quarantines a record whose shape drifted).
    tuning = {
        "baselineTps": 61.5,
        "bestIndex": 0,
        "bestTps": 74.25,
        "bestProfile": profile,
        "trials": [],
        "stoppedReason": "Trial budget reached",
        "objective": "short-prompt decode throughput",
    }
    keys = {
        "localmotive:model-root": "C:/fixtures/v041/models",
        "localmotive:runtime": "C:/fixtures/v041/llama-server.exe",
        "localmotive:cloud-provider": "openrouter",
        "localmotive:cloud-model": "fixture/v041-cloud-model",
        "localmotive:profile:fixture/v041-legacy-model": json.dumps(profile, indent=2),
        "localmotive:tuning:fixture/v041-legacy-model": json.dumps(tuning, indent=2),
    }
    return {
        "schema": "localmotive.v041-settings.v1",
        "source": [
            "git show v0.4.1:src/App.tsx (readRecord/readSetting keys)",
            "git show v0.4.1:src/model.ts (LaunchProfile shape)",
        ],
        "keys": keys,
        "expect": {
            "settings": {
                "model-root": keys["localmotive:model-root"],
                "runtime": keys["localmotive:runtime"],
                "cloud-provider": "openrouter",
                "cloud-model": "fixture/v041-cloud-model",
            },
            "profile": {
                "name": profile["name"],
                "model": profile["model"],
                "port": profile["port"],
                "context": profile["context"],
                "seed": profile["seed"],
                "temperature": profile["temperature"],
                "threads": profile["threads"],
                "batch": profile["batch"],
            },
        },
    }


def main() -> int:
    here = pathlib.Path(__file__).resolve().parent
    mirror = build_mirror()
    cache = build_cache_record()
    settings = build_settings_record()
    mirror_b64 = base64.b64encode(mirror).decode("ascii")
    (here / "canary-mirror.sqlite.b64").write_text(mirror_b64 + "\n", encoding="ascii")
    (here / "canary-catalog-cache.json").write_bytes(cache)
    (here / "canary-settings.json").write_text(
        json.dumps(settings, indent=2) + "\n", encoding="utf-8"
    )
    print(f"mirror bytes: {len(mirror)}")
    print(f"cache bytes: {len(cache)}")
    print(f"settings keys: {len(settings['keys'])}")
    print(f"SENTINELS: {SENTINEL} | {NODE_SENTINEL}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
