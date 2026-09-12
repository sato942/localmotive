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
import { buildFixture, verify, withFixture } from "./lib/manifest_fixture.mjs";

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
