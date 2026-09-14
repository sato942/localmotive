// F9-04 integration check: the release plumbing must connect its producers to
// its consumers. This test assembles the directory layout the release workflow
// produces (the packaged artifacts plus the lifecycle records of the same run),
// runs the real manifest generator over it, and then runs the real
// qualification and promotion validators. The negative controls are the
// failure modes the reviewed plumbing could hide: a missing lifecycle record,
// a lifecycle record from an unrelated run with no permitted carry-forward,
// a legacy evidence filename, and a source/artifact contradiction.
import test from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { buildFixture } from "./lib/manifest_fixture.mjs";
import { ATTESTATION_RECORDS, buildQualificationManifest } from "../build_qualification_manifest.mjs";
import { verifyReleasePromotion, promisedAssetNames } from "../verify_release_promotion.mjs";

const RELEASE = "0.6.0";
const SOURCE = "a".repeat(40);
const TAG = `v${RELEASE}`;

/// The generator records the repository head; give the fixture a real (tiny)
/// repository so the same generator the workflow runs is exercised here.
function initRepository(root) {
  const run = (args) =>
    execFileSync("git", args, { cwd: root, encoding: "utf8", windowsHide: true });
  run(["init", "-q"]);
  run(["add", "-A"]);
  run([
    "-c",
    "user.email=fixture@localmotive.test",
    "-c",
    "user.name=fixture",
    "commit",
    "-q",
    "-m",
    "fixture",
  ]);
}

async function withRepository(options, body) {
  const fixture = buildFixture(options);
  try {
    initRepository(fixture.root);
    return await body(fixture);
  } finally {
    rmSync(fixture.root, { recursive: true, force: true });
  }
}

function generateManifest(fixture, options = {}) {
  // The builder writes outPath relative to the process cwd; keep every path
  // inside the fixture so a test can never clobber the committed evidence.
  const manifestFile = join(
    fixture.root,
    "release-evidence",
    RELEASE,
    `qualification-manifest-${RELEASE}.json`,
  );
  buildQualificationManifest({
    inventoryPath: `release-evidence/${RELEASE}/candidate-inventory-${RELEASE}.json`,
    attestationsDir: `release-evidence/${RELEASE}/attestations`,
    outPath: manifestFile,
    release: RELEASE,
    root: fixture.root,
    ...options,
  });
  assert.ok(
    existsSync(manifestFile),
    "the generated manifest must land inside the fixture, never the repository",
  );
  return manifestFile;
}

function promote(fixture) {
  return verifyReleasePromotion({
    qualifiedDirectory: fixture.root,
    root: fixture.root,
    tag: TAG,
    release: RELEASE,
    expectedSourceRevision: SOURCE,
    verifyRunId: "123456789",
  });
}

function attestationPath(fixture, name) {
  return join(fixture.root, "release-evidence", RELEASE, "attestations", name);
}

test("the run's own lifecycle records qualify a generated manifest and publish set", async () => {
  await withRepository({ withQualifiedSet: true }, async (fixture) => {
    generateManifest(fixture);
    const promotion = await promote(fixture);
    assert.deepEqual(promotion.failures, [], "a workflow-shaped set must pass");
    assert.equal(promotion.ok, true);
  });
});

test("the staged packaged record reaches the manifest without prepopulated identical copies", async () => {
  // F9-04: the workflow's producer/consumer layout. The package job writes the
  // fresh record to artifacts/; the committed attestation is a DIFFERENT
  // (stale) file. generateManifest must stage the fresh record over the
  // attestation input, so the manifest binds the fresh record and promotion
  // passes without anyone manually copying identical bytes into both places.
  await withRepository({ withQualifiedSet: true }, async (fixture) => {
    const staged = join(fixture.root, "artifacts", `packaged-verification-${RELEASE}.json`);
    const record = JSON.parse(readFileSync(staged, "utf8"));
    // A fresh producer run: new timestamps, identical source and artifact
    // identity (the owner's reproduction changes only the timestamps).
    record.started_at = "2026-09-14T00:00:00.000Z";
    record.finished_at = "2026-09-14T00:05:00.000Z";
    writeFileSync(staged, `${JSON.stringify(record, null, 2)}\n`);
    // The committed attestation is stale: same producer shape, other run.
    const stale = { ...record, started_at: "2026-01-01T00:00:00.000Z", finished_at: "2026-01-01T00:05:00.000Z" };
    writeFileSync(attestationPath(fixture, `packaged-verification-${RELEASE}.json`), `${JSON.stringify(stale, null, 2)}\n`);
    generateManifest(fixture, { stagePackagedVerification: `artifacts/packaged-verification-${RELEASE}.json` });
    // The manifest now references the staged bytes, not the stale copy.
    const manifest = JSON.parse(
      readFileSync(join(fixture.root, "release-evidence", RELEASE, `qualification-manifest-${RELEASE}.json`), "utf8"),
    );
    const stagedBytes = readFileSync(staged);
    const { createHash } = await import("node:crypto");
    assert.equal(
      manifest.records.packaged_verification.sha256,
      createHash("sha256").update(stagedBytes).digest("hex"),
      "the manifest must bind the freshly staged record",
    );
    // And the committed attestation now carries the staged bytes.
    assert.equal(
      readFileSync(attestationPath(fixture, `packaged-verification-${RELEASE}.json`), "utf8"),
      readFileSync(staged, "utf8"),
    );
    const promotion = await promote(fixture);
    assert.deepEqual(promotion.failures, [], `a freshly staged record must pass: ${promotion.failures.join("; ")}`);
    assert.equal(promotion.ok, true);
  });
});

test("a missing staged packaged record fails instead of binding a stale copy", async () => {
  // F9-04: without the fresh producer file the builder must fail loudly
  // rather than silently qualify whatever committed copy happens to exist.
  await withRepository({ withQualifiedSet: true }, async (fixture) => {
    assert.throws(
      () => generateManifest(fixture, { stagePackagedVerification: `artifacts/packaged-verification-missing.json` }),
      /staged packaged-verification producer record is missing/,
    );
  });
});

test("a missing lifecycle record fails qualification", async () => {
  await withRepository({ withQualifiedSet: true }, async (fixture) => {
    generateManifest(fixture);
    renameSync(
      attestationPath(fixture, "sandbox-clean-account-lifecycle-upgrade-v0.4.0.json"),
      attestationPath(fixture, "sandbox-clean-account-lifecycle-upgrade-v0.4.0.moved"),
    );
    const promotion = await promote(fixture);
    assert.equal(promotion.ok, false, "a vanished lifecycle record cannot qualify");
  });
});

test("an unrelated run's lifecycle record fails without a permitted carry-forward", async () => {
  await withRepository(
    { withQualifiedSet: true, lifecycleWrongSource: { key: "lifecycle_preservation_v0.4.1" } },
    async (fixture) => {
      generateManifest(fixture);
      const promotion = await promote(fixture);
      assert.equal(promotion.ok, false, "a record bound to another source cannot qualify");
    },
  );
});

test("a legacy evidence filename fails instead of passing silently", async () => {
  await withRepository({ withQualifiedSet: true }, async (fixture) => {
    generateManifest(fixture);
    renameSync(
      attestationPath(fixture, `packaged-verification-${RELEASE}.json`),
      attestationPath(fixture, "packaged-verification-0.4.1.json"),
    );
    const promotion = await promote(fixture);
    assert.equal(
      promotion.ok,
      false,
      "the producer's historical filename is not the contract's name",
    );
  });
});

test("an artifact contradiction fails even when every value is well formed", async () => {
  await withRepository({ withQualifiedSet: true }, async (fixture) => {
    // Contradict the portable row the packaged-verification record binds. Every
    // value stays a valid digest; the relationship is what is broken.
    const inventoryPath = join(
      fixture.root,
      "release-evidence",
      RELEASE,
      `candidate-inventory-${RELEASE}.json`,
    );
    const inventory = JSON.parse(readFileSync(inventoryPath, "utf8"));
    const portable = inventory.artifacts.find((row) => /portable/.test(row.name));
    assert.ok(portable, "the fixture advertises a portable artifact");
    portable.sha256 = "f".repeat(64);
    writeFileSync(inventoryPath, JSON.stringify(inventory, null, 2));
    generateManifest(fixture);
    const promotion = await promote(fixture);
    assert.equal(promotion.ok, false, "advertised bytes that do not exist cannot pass");
  });
});

test("the fixture writes every mandatory record under the generator's name", async () => {
  // The fixture and the generator must not drift: a renamed mandatory record
  // would make this integration check validate a set the workflow never builds.
  await withRepository({ withQualifiedSet: true }, async (fixture) => {
    const attestations = join(fixture.root, "release-evidence", RELEASE, "attestations");
    for (const [key, nameFor] of Object.entries(ATTESTATION_RECORDS)) {
      assert.ok(
        existsSync(join(attestations, nameFor(RELEASE))),
        `the fixture must write ${key} at ${nameFor(RELEASE)}`,
      );
    }
  });
});

test("a controlled package-stage failure reports every missing output", async () => {
  // F9-04: the job's failure report must name the evidence it did not produce
  // and keep the failing outcome. Both outcomes are exercised here with the
  // same script the workflow runs.
  const checker = join(process.cwd(), "scripts", "check_package_outputs.ps1");
  const root = mkdtempSync(join(tmpdir(), "lm-package-"));
  try {
    const failing = execFileSync(
      "powershell",
      ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", checker, "-Root", root, "-Version", RELEASE],
      { encoding: "utf8", stdio: "pipe" },
    );
    assert.fail(`expected the missing-evidence report to fail: ${failing}`);
  } catch (error) {
    assert.equal(error.status, 1, "the report must exit non-zero");
    const output = `${error.stdout ?? ""}${error.stderr ?? ""}`;
    for (const outputName of [
      `Localmotive_${RELEASE}_x64-portable.exe`,
      `Localmotive_${RELEASE}_x64-setup.exe`,
      `Localmotive_${RELEASE}_x64.msi`,
      `candidate-inventory-${RELEASE}.json`,
      `SHA256SUMS-${RELEASE}.txt`,
      `packaged-verification-${RELEASE}.json`,
    ]) {
      assert.match(output, new RegExp(`MISSING artifacts/${outputName.replace(/[.]/g, "\.")}`));
    }
  }
  try {
    // A complete set exits zero: the report must not fire on a healthy stage.
    const artifacts = join(root, "artifacts");
    mkdirSync(artifacts, { recursive: true });
    for (const outputName of [
      `Localmotive_${RELEASE}_x64-portable.exe`,
      `Localmotive_${RELEASE}_x64-setup.exe`,
      `Localmotive_${RELEASE}_x64.msi`,
      `candidate-inventory-${RELEASE}.json`,
      `SHA256SUMS-${RELEASE}.txt`,
      `packaged-verification-${RELEASE}.json`,
    ]) {
      writeFileSync(join(artifacts, outputName), "fixture");
    }
    const result = execFileSync(
      "powershell",
      ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", checker, "-Root", root, "-Version", RELEASE],
      { encoding: "utf8", stdio: "pipe" },
    );
    assert.match(result, /Every expected package output exists/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("the produced evidence name and the promotion contract agree", () => {
  const release = readFileSync(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  // The producer must write the versioned name the promotion contract promises.
  // The name is produced inside scripts/verify_packaged_impl.ps1 (invoked by
  // the workflow through the wrapper); assert on the union so the contract
  // survives the extraction.
  assert.match(release, /verify_packaged_matrix\.ps1/);
  const wrapper = readFileSync(join(process.cwd(), "scripts", "verify_packaged_matrix.ps1"), "utf8");
  assert.match(wrapper, /verify_packaged_impl\.ps1/);
  const script = readFileSync(join(process.cwd(), "scripts", "verify_packaged_impl.ps1"), "utf8");
  assert.match(script, /packaged-verification-\$Version\.json/);
  assert.doesNotMatch(
    release,
    /artifacts\/packaged-verification-0\.4\.1\.json/,
    "the historical verifier output name must not be the release asset name",
  );
  assert.ok(
    promisedAssetNames(RELEASE).includes(`packaged-verification-${RELEASE}.json`),
    "the promotion contract promises the versioned evidence name",
  );
  // F9-04: the qualification step must stage the freshly produced record into
  // the manifest; an unused --stage/--source pair leaves the stale committed
  // copy bound instead.
  assert.match(release, /--stage-packaged-verification "artifacts\/packaged-verification-/);
  assert.doesNotMatch(release, /build_qualification_manifest\.mjs --stage artifacts/);
  const builder = readFileSync(join(process.cwd(), "scripts", "build_qualification_manifest.mjs"), "utf8");
  assert.match(builder, /stagePackagedVerification/);
});
