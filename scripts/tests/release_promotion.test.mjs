// Release-promotion validator negative controls: the promotion path must
// refuse a set that no longer matches what the qualification manifest,
// inventory, checksums and records describe — before any publication step.
// These tests run the real validator against a miniature promotion bundle;
// nothing is published or tagged.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync, rmSync, unlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { buildFixture } from "./lib/manifest_fixture.mjs";
import { parseChecksumFile, promisedAssetNames, verifyReleasePromotion } from "../verify_release_promotion.mjs";

const RELEASE = "0.6.0";
const TAG = `v${RELEASE}`;
const SOURCE = "a".repeat(40);

async function run(fixture, overrides = {}) {
  return verifyReleasePromotion({
    qualifiedDirectory: fixture.root,
    root: fixture.root,
    tag: TAG,
    release: RELEASE,
    expectedSourceRevision: SOURCE,
    verifyRunId: "123456789",
    ...overrides,
  });
}

async function withFixture(options, body) {
  const fixture = buildFixture(options);
  try {
    await body(fixture);
  } finally {
    rmSync(fixture.root, { recursive: true, force: true });
  }
}

test("a coherent qualified set is safe to publish", async () => {
  await withFixture({ withQualifiedSet: true }, async (fixture) => {
    const result = await run(fixture);
    assert.deepEqual(result.failures, [], result.failures.join("\n"));
    assert.equal(result.report.status, "PASS");
    assert.deepEqual([...result.report.promisedAssets].sort(), [...promisedAssetNames(RELEASE)].sort());
  });
});

test("a wrong expected source revision is refused even when the set is intact", async () => {
  await withFixture({ withQualifiedSet: true }, async (fixture) => {
    const result = await run(fixture, { expectedSourceRevision: "d".repeat(40) });
    assert.equal(result.ok, false);
    assert.ok(
      result.failures.some((failure) => failure.includes("does not match the expected reviewed revision")),
      result.failures.join("\n"),
    );
  });
});

test("bytes that changed after qualification are refused", async () => {
  await withFixture({ withQualifiedSet: true }, async (fixture) => {
    writeFileSync(join(fixture.root, "artifacts", `Localmotive_${RELEASE}_x64-portable.exe`), "tampered\n");
    const result = await run(fixture);
    assert.equal(result.ok, false);
    assert.ok(
      result.failures.some((failure) => failure.includes("contradicts the recorded checksum")),
      result.failures.join("\n"),
    );
    assert.ok(
      result.failures.some((failure) => failure.includes("contradicts the qualification manifest")),
      result.failures.join("\n"),
    );
  });
});

test("a missing promised asset is refused", async () => {
  await withFixture({ withQualifiedSet: true }, async (fixture) => {
    unlinkSync(join(fixture.root, "artifacts", `Localmotive_${RELEASE}_x64.msi`));
    const result = await run(fixture);
    assert.equal(result.ok, false);
    assert.ok(
      result.failures.some((failure) => failure.includes(`promised asset Localmotive_${RELEASE}_x64.msi is missing`)),
      result.failures.join("\n"),
    );
  });
});

test("missing lifecycle evidence is refused", async () => {
  await withFixture({ withQualifiedSet: true }, async (fixture) => {
    unlinkSync(
      join(
        fixture.root,
        "release-evidence",
        RELEASE,
        "attestations",
        "sandbox-clean-account-lifecycle-preservation-v0.5.0.json",
      ),
    );
    const result = await run(fixture);
    assert.equal(result.ok, false);
    assert.ok(
      result.failures.some((failure) => failure.includes("lifecycle_preservation_v0.5.0: record missing")),
      result.failures.join("\n"),
    );
  });
});

test("a moved or relabelled tag is refused", async () => {
  await withFixture({ withQualifiedSet: true }, async (fixture) => {
    const moved = await run(fixture, { tag: "v0.6.1", release: RELEASE });
    assert.equal(moved.ok, false);
    assert.ok(
      moved.failures.some((failure) => failure.includes("does not identify release")),
      moved.failures.join("\n"),
    );
    const malformed = await run(fixture, { tag: "0.6.0" });
    assert.equal(malformed.ok, false);
    assert.ok(
      malformed.failures.some((failure) => failure.includes("is not a vMAJOR.MINOR.PATCH tag")),
      malformed.failures.join("\n"),
    );
  });
});

test("a packaged-verification asset that is not the producer record is refused", async () => {
  await withFixture({ withQualifiedSet: true }, async (fixture) => {
    writeFileSync(
      join(fixture.root, "artifacts", `packaged-verification-${RELEASE}.json`),
      JSON.stringify({ overall_status: "PASS", checks: [] }),
    );
    const result = await run(fixture);
    assert.equal(result.ok, false);
    assert.ok(
      result.failures.some((failure) => failure.includes("not the producer record")),
      result.failures.join("\n"),
    );
  });
});

test("a checksum file that lists the wrong file set is refused", async () => {
  await withFixture({ withQualifiedSet: true }, async (fixture) => {
    const path = join(fixture.root, "artifacts", `SHA256SUMS-${RELEASE}.txt`);
    const lines = readFileSync(path, "utf8").split("\n").filter(Boolean);
    writeFileSync(path, `${lines.slice(0, 2).join("\n")}\n`);
    const result = await run(fixture);
    assert.equal(result.ok, false);
    assert.ok(
      result.failures.some((failure) => failure.includes("expected exactly the three release binaries")),
      result.failures.join("\n"),
    );
  });
});

test("the checksum parser refuses duplicates and unparseable lines", () => {
  const line = `${"a".repeat(64)}  Localmotive_0.6.0_x64-portable.exe`;
  assert.equal(parseChecksumFile(`${line}\n${line}\n`).ok, false);
  assert.match(parseChecksumFile(`${line}\n${line}\n`).reason, /duplicate/);
  assert.equal(parseChecksumFile("not-a-checksum\n").ok, false);
  assert.equal(parseChecksumFile("\n").ok, false);
  const good = parseChecksumFile(`${line}\n`);
  assert.equal(good.ok, true);
  assert.equal(good.entries.size, 1);
});
