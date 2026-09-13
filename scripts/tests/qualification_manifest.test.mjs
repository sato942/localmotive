// Qualification-manifest verifier negative controls: a fresh checkout must
// validate the real manifest, and a contradictory one must be refused even
// when every field is syntactically valid. The repository fixture lives in
// ./lib/manifest_fixture.mjs; each test mutates one relationship at a time:
// source-only, digest-only, missing record, incorrect outcome, path escape,
// and text-hash policy.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { DIGESTS, buildFixture, verify, withFixture } from "./lib/manifest_fixture.mjs";

test("a coherent manifest fixture validates", () => {
  withFixture({}, (fixture) => {
    const { failures, lines } = verify(fixture);
    assert.deepEqual(failures, [], failures.join("\n"));
    assert.ok(lines.some((line) => line.includes("producer inventory ok")));
  });
});

test("the expected reviewed revision is enforced when requested", () => {
  withFixture({}, (fixture) => {
    const { failures } = verify(fixture, { expectedSourceRevision: "d".repeat(40) });
    assert.ok(
      failures.some((failure) => failure.includes("does not match the expected reviewed revision")),
      failures.join("\n"),
    );
  });
});

test("a source-only mismatch between inventory and manifest is refused", () => {
  withFixture({ inventorySourceRevision: "e".repeat(40) }, (fixture) => {
    const { failures } = verify(fixture);
    assert.ok(failures.some((failure) => failure.includes("inventory source")), failures.join("\n"));
  });
});

test("a digest-only mismatch between inventory and manifest is refused", () => {
  withFixture({ inventoryDigest: "f".repeat(64) }, (fixture) => {
    const { failures } = verify(fixture);
    assert.ok(
      failures.some((failure) => failure.includes("contradicts the inventory digest")),
      failures.join("\n"),
    );
  });
});

test("a syntactically valid but foreign artifact digest in the manifest is refused", () => {
  withFixture({}, (fixture) => {
    const manifestPath = join(fixture.root, fixture.manifestFile);
    const doc = JSON.parse(readFileSync(manifestPath, "utf8"));
    doc.artifacts[0].sha256 = "9".repeat(64);
    writeFileSync(manifestPath, JSON.stringify(doc, null, 2));
    const { failures } = verify(fixture);
    assert.ok(
      failures.some((failure) => failure.includes("contradicts the inventory digest")),
      failures.join("\n"),
    );
  });
});

test("a missing required record is refused", () => {
  withFixture({ missingRecord: "lifecycle_preservation_v0.5.0" }, (fixture) => {
    const { failures } = verify(fixture);
    assert.ok(
      failures.some((failure) =>
        failure.includes("required record lifecycle_preservation_v0.5.0 is missing"),
      ),
      failures.join("\n"),
    );
  });
});

test("an incorrect outcome inside a record is refused", () => {
  withFixture({ lifecycleStatus: { key: "lifecycle_upgrade_v0.4.0", status: "FAIL" } }, (fixture) => {
    const { failures } = verify(fixture);
    assert.ok(
      failures.some((failure) => failure.includes("status FAIL is not PASS")),
      failures.join("\n"),
    );
  });
});

test("a witness that lost its required outcome is refused", () => {
  withFixture({ witnessOutcome: { key: "witness_timeout", status: "PASS" } }, (fixture) => {
    const { failures } = verify(fixture);
    assert.ok(
      failures.some((failure) =>
        failure.includes("witness_timeout: status PASS != required witness outcome TIMEOUT"),
      ),
      failures.join("\n"),
    );
  });
});

test("a lifecycle record whose staged digests contradict the inventory is refused", () => {
  withFixture({ lifecycleWrongSetupDigest: { key: "lifecycle_preservation_v0.4.1" } }, (fixture) => {
    const { failures } = verify(fixture);
    assert.ok(
      failures.some((failure) => failure.includes("staged setup digest contradicts the inventory")),
      failures.join("\n"),
    );
  });
});

test("a lifecycle record bound to a different source is refused", () => {
  withFixture({ lifecycleWrongSource: { key: "lifecycle_upgrade_v0.5.0" } }, (fixture) => {
    const { failures } = verify(fixture);
    assert.ok(
      failures.some(
        (failure) => failure.includes("source revision") && failure.includes("contradicts"),
      ),
      failures.join("\n"),
    );
  });
});

test("a lifecycle record missing a required step is refused", () => {
  withFixture({ lifecycleMissingStep: { key: "lifecycle_preservation_v0.5.0" } }, (fixture) => {
    const { failures } = verify(fixture);
    assert.ok(
      failures.some((failure) =>
        failure.includes("required step nsis-preservation-from-v0.5.0 is missing"),
      ),
      failures.join("\n"),
    );
  });
});

test("installed payload digests that contradict the staged installers are refused", () => {
  withFixture({ nsisPayloadDigest: "8".repeat(64) }, (fixture) => {
    const { failures } = verify(fixture);
    assert.ok(
      failures.some((failure) => failure.includes("installed NSIS payload digest contradicts")),
      failures.join("\n"),
    );
  });
});

test("a record path that escapes the repository is refused", () => {
  withFixture({ escapePath: "witness_stale_lock" }, (fixture) => {
    const { failures } = verify(fixture);
    assert.ok(
      failures.some(
        (failure) =>
          failure.includes("must live in the repository") || failure.includes("record missing"),
      ),
      failures.join("\n"),
    );
  });
});

test("a text record without its newline-normalized digest is refused", () => {
  withFixture({ logWithoutLf: "rt06_full_run_log" }, (fixture) => {
    const { failures } = verify(fixture);
    assert.ok(
      failures.some((failure) => failure.includes("newline-normalized sha256_lf")),
      failures.join("\n"),
    );
  });
});

test("a lifecycle record carried forward from a superseded candidate validates when its citation matches", () =>
  withFixture(
    { withQualifiedSet: true, carryForward: { key: "lifecycle_upgrade_v0.4.0" } },
    async (fixture) => {
      const result = await verify(fixture);
      assert.deepEqual(result.failures, [], result.failures.join("\n"));
      assert.ok(
        result.lines.some((line) => line.startsWith("carried forward: lifecycle_upgrade_v0.4.0 from")),
        result.lines.join("\n"),
      );
    },
  ));

test("a carried-forward record whose citation digest drifted is refused", () =>
  withFixture(
    {
      carryForward: {
        key: "lifecycle_upgrade_v0.4.0",
        citeShaOverride: "9".repeat(64),
      },
    },
    (fixture) => {
      const result = verify(fixture);
      assert.ok(result.failures.length > 0, "expected a refusal");
      assert.match(result.failures.join("\n"), /carried-forward evidence inventory digest drifted/);
    },
  ));

test("a carried-forward record whose staged digests contradict its cited inventory is refused", () =>
  withFixture(
    {
      carryForward: {
        key: "lifecycle_upgrade_v0.4.0",
        recordMsi: "0".repeat(64),
      },
    },
    (fixture) => {
      const result = verify(fixture);
      assert.ok(result.failures.length > 0, "expected a refusal");
      assert.match(result.failures.join("\n"), /does not match its cited inventory/);
    },
  ));

test("a carry-forward that cites a path outside the history tree is refused", () =>
  withFixture(
    {
      carryForward: {
        key: "lifecycle_upgrade_v0.4.0",
        citePath: "release-evidence/0.6.0/candidate-inventory-0.6.0.json",
      },
    },
    (fixture) => {
      const result = verify(fixture);
      assert.ok(result.failures.length > 0, "expected a refusal");
      assert.match(result.failures.join("\n"), /must cite a history-path inventory/);
    },
  ));

// ---------------------------------------------------------------------------
// F9-03: the packaged-verification branch validates the REAL producer schema
// (scripts/verify_041.mjs). Each case below mutates one semantic relationship
// while the fixture recomputes the record's own hash metadata, so a rejection
// demonstrates the relationship check rather than an incidental stale hash.
// ---------------------------------------------------------------------------

test("F9-03 a genuine producer-shaped packaged record validates unchanged", () =>
  withFixture({}, (fixture) => {
    const result = verify(fixture);
    assert.deepEqual(result.failures, [], result.failures.join("\n"));
  }));

test("F9-03 a producer record whose source_revision contradicts the manifest is refused", () =>
  withFixture(
    { packaged: { source_revision: "d".repeat(40) } },
    (fixture) => {
      const result = verify(fixture);
      assert.ok(result.failures.length > 0, "expected a refusal");
      assert.match(result.failures.join("\n"), /source_revision .* contradicts the manifest source/);
    },
  ));

test("F9-03 a producer record whose artifact digest contradicts the inventory is refused", () =>
  withFixture(
    {
      packaged: {
        artifact: {
          name: "Localmotive_0.6.0_x64-portable.exe",
          size_bytes: 11,
          sha256: "e".repeat(64),
        },
      },
    },
    (fixture) => {
      const result = verify(fixture);
      assert.ok(result.failures.length > 0, "expected a refusal");
      assert.match(result.failures.join("\n"), /artifact digest contradicts the inventory/);
    },
  ));

test("F9-03 a producer record whose artifact size contradicts the inventory is refused", () =>
  withFixture(
    {
      packaged: {
        artifact: {
          name: "Localmotive_0.6.0_x64-portable.exe",
          size_bytes: 4096,
          sha256: DIGESTS.portable,
        },
      },
    },
    (fixture) => {
      const result = verify(fixture);
      assert.ok(result.failures.length > 0, "expected a refusal");
      assert.match(result.failures.join("\n"), /artifact size .* contradicts the inventory/);
    },
  ));

test("F9-03 a producer record whose artifact name contradicts the inventory is refused", () =>
  withFixture(
    {
      packaged: {
        artifact: {
          name: "Localmotive_0.6.0_x64-something-else.exe",
          size_bytes: 11,
          sha256: DIGESTS.portable,
        },
      },
    },
    (fixture) => {
      const result = verify(fixture);
      assert.ok(result.failures.length > 0, "expected a refusal");
      assert.match(result.failures.join("\n"), /artifact name .* contradicts the inventory/);
    },
  ));

test("F9-03 a producer record with no artifact identity is refused instead of skipping", () =>
  withFixture({ packaged: { artifact: undefined } }, (fixture) => {
    const result = verify(fixture);
    assert.ok(result.failures.length > 0, "expected a refusal");
    assert.match(result.failures.join("\n"), /artifact identity is missing/);
  }));

test("F9-03 a producer record without a source revision is refused instead of skipping", () =>
  withFixture({ packaged: { source_revision: undefined } }, (fixture) => {
    const result = verify(fixture);
    assert.ok(result.failures.length > 0, "expected a refusal");
    assert.match(result.failures.join("\n"), /source_revision is missing/);
  }));

test("F9-03 a failed required check under a passing aggregate is refused", () =>
  withFixture(
    {
      packaged: {
        checks: [
          { id: "candidate.artifact", status: "PASS" },
          { id: "health.seven-stage-pass", status: "FAIL" },
        ],
      },
    },
    (fixture) => {
      const result = verify(fixture);
      assert.ok(result.failures.length > 0, "expected a refusal");
      assert.match(result.failures.join("\n"), /check health\.seven-stage-pass is FAIL/);
    },
  ));

test("F9-03 an aggregate failure with every recorded check passing is refused", () =>
  withFixture({ packaged: { overall_status: "FAIL" } }, (fixture) => {
    const result = verify(fixture);
    assert.ok(result.failures.length > 0, "expected a refusal");
    assert.match(result.failures.join("\n"), /overall_status FAIL contradicts every PASS check/);
  }));
