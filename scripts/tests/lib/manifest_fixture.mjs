// Shared miniature-repository fixture for the qualification-manifest and
// release-promotion verifiers: manifest + inventory + records (+ optionally a
// real qualified artifact set), built so a single relationship can be mutated
// at a time by the calling test file.
import { createHash } from "node:crypto";
import { copyFileSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { verifyQualificationManifest } from "../../verify_qualification_manifest.mjs";

const RELEASE = "0.6.0";
const SOURCE = "a".repeat(40);
const DIGESTS = {
  portable: "1".repeat(64),
  setup: "2".repeat(64),
  msi: "3".repeat(64),
};

/// Real (tiny) artifact bodies for fixtures that must carry actual bytes:
/// a qualified-set fixture binds its digests to files on disk.
export function artifactBodies(release) {
  return {
    setup: `setup-${release}-bytes
`,
    portable: `portable-${release}-bytes
`,
    msi: `msi-${release}-bytes
`,
  };
}

function sha256Of(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

/// CRLF -> LF, byte-scan (no regex): a checkout whose line endings were
/// smudged must still match a recorded LF digest.
function sha256LfOf(bytes) {
  const out = Buffer.alloc(bytes.length);
  let length = 0;
  for (let index = 0; index < bytes.length; index += 1) {
    if (bytes[index] === 0x0d && bytes[index + 1] === 0x0a) continue;
    out[length] = bytes[index];
    length += 1;
  }
  return sha256Of(out.subarray(0, length));
}

function write(root, relative, contents) {
  const path = join(root, relative);
  mkdirSync(dirname(path), { recursive: true });
  const bytes =
    typeof contents === "string"
      ? Buffer.from(contents, "utf8")
      : Buffer.from(JSON.stringify(contents, null, 2));
  writeFileSync(path, bytes);
  const entry = {
    path,
    relative: relative.split("\\").join("/"),
    sha256: sha256Of(bytes),
    sha256_lf: sha256LfOf(bytes),
  };
  // Mirror the generator: a carried-forward record carries its citation on the
  // manifest entry as well as in the document.
  if (contents && typeof contents === "object" && contents.carriedForward) {
    entry.carriedForward = contents.carriedForward;
  }
  return entry;
}

function lifecycleRecord(digests, previousTag, inventorySha, overrides = {}) {
  return {
    schema: "localmotive.sandbox-lifecycle.v0",
    status: "PASS",
    stage: "preservation-verification",
    tag: `v${RELEASE}`,
    version: RELEASE,
    previousTag,
    sourceRevision: SOURCE,
    harnessRevision: "b".repeat(40),
    candidateDigests: { currentSetup: digests.setup, currentMsi: digests.msi },
    candidateInventorySha256: inventorySha,
    preservation: { status: "PASS", output: "PRESERVE_PASS" },
    steps: [
      { name: "nsis-fresh-install-launch-uninstall", status: "PASS", detail: "" },
      { name: "msi-fresh-install-launch-uninstall", status: "PASS", detail: "" },
      { name: "nsis-update-from-previous-launch-uninstall", status: "PASS", detail: "" },
      { name: `nsis-preservation-from-${previousTag}`, status: "PASS", detail: "" },
    ],
    installedDigests: {
      "NSIS fresh install": "4".repeat(64),
      "MSI fresh install": "5".repeat(64),
      "Pre-update install": "6".repeat(64),
      "Post-update install": "4".repeat(64),
      "Preservation upgrade": "4".repeat(64),
    },
    nsisPayloadDigest: "4".repeat(64),
    msiPayloadDigest: "5".repeat(64),
    coverageNote: "install/launch/uninstall, launch probe and preservation scope",
    ...overrides,
  };
}

export function buildFixture(options = {}) {
  const root = mkdtempSync(join(tmpdir(), "lm-manifest-"));
  const bodies = options.withQualifiedSet ? artifactBodies(RELEASE) : null;
  const digests = bodies
    ? {
        setup: sha256Of(Buffer.from(bodies.setup, "utf8")),
        portable: sha256Of(Buffer.from(bodies.portable, "utf8")),
        msi: sha256Of(Buffer.from(bodies.msi, "utf8")),
      }
    : DIGESTS;
  // The approved artifact digests, independent of any inventory-side mutation:
  // a digest-only contradiction needs the two sides to be able to disagree.
  const approvedArtifacts = () => [
    {
      name: `Localmotive_${RELEASE}_x64-portable.exe`,
      sizeBytes: bodies ? Buffer.byteLength(bodies.portable) : 11,
      sha256: digests.portable,
    },
    {
      name: `Localmotive_${RELEASE}_x64-setup.exe`,
      sizeBytes: bodies ? Buffer.byteLength(bodies.setup) : 12,
      sha256: digests.setup,
    },
    {
      name: `Localmotive_${RELEASE}_x64.msi`,
      sizeBytes: bodies ? Buffer.byteLength(bodies.msi) : 13,
      sha256: digests.msi,
    },
  ];
  const inventory = {
    schemaVersion: 1,
    release: RELEASE,
    sourceRevision: options.inventorySourceRevision ?? SOURCE,
    generatedAt: "2026-09-13T00:00:00Z",
    checksumFile: `SHA256SUMS-${RELEASE}.txt`,
    artifacts: approvedArtifacts(),
  };
  if (options.inventoryDigest) {
    inventory.artifacts[2].sha256 = options.inventoryDigest;
  }
  const inventoryFile = write(
    root,
    `release-evidence/${RELEASE}/candidate-inventory-${RELEASE}.json`,
    inventory,
  );
  const workflowFile = write(root, ".github/workflows/release.yml", "name: Release\n");
  const inventorySha = inventoryFile.sha256;

  const records = {};
  records.packaged_verification = write(
    root,
    `release-evidence/${RELEASE}/attestations/packaged-verification-${RELEASE}.json`,
    {
      overall_status: "PASS",
      sourceRevision: SOURCE,
      candidatePortableSha256: digests.portable,
      checks: [{ status: "PASS" }, { status: "PASS" }],
    },
  );
  const scenarios = [
    ["lifecycle_upgrade_v0.4.0", "sandbox-clean-account-lifecycle-upgrade-v0.4.0", "v0.4.0"],
    ["lifecycle_upgrade_v0.5.0", "sandbox-clean-account-lifecycle-upgrade-v0.5.0", "v0.5.0"],
    ["lifecycle_preservation_v0.4.1", "sandbox-clean-account-lifecycle-preservation-v0.4.1", "v0.4.1"],
    ["lifecycle_preservation_v0.5.0", "sandbox-clean-account-lifecycle-preservation-v0.5.0", "v0.5.0"],
  ];
  for (const [key, scenario, previousTag] of scenarios) {
    const overrides = {};
    if (options.lifecycleStatus?.key === key) {
      overrides.status = options.lifecycleStatus.status;
    }
    if (options.lifecyclePreservation?.key === key) {
      overrides.preservation = { status: options.lifecyclePreservation.status };
    }
    if (options.lifecycleWrongSource?.key === key) {
      overrides.sourceRevision = "c".repeat(40);
    }
    if (options.carryForward?.key === key) {
      const carried = carryForwardFixture(root, options.carryForward);
      overrides.candidateDigests = carried.recordDigests;
      overrides.carriedForward = carried.entry;
    }
    if (options.lifecycleMissingStep?.key === key) {
      const record = lifecycleRecord(digests, previousTag, inventorySha, overrides);
      record.steps = record.steps.slice(0, 3);
      records[key] = write(
        root,
        `release-evidence/${RELEASE}/attestations/${scenario}.json`,
        record,
      );
      continue;
    }
    if (options.lifecycleWrongSetupDigest?.key === key) {
      const record = lifecycleRecord(digests, previousTag, inventorySha, overrides);
      record.candidateDigests.currentSetup = "9".repeat(64);
      records[key] = write(
        root,
        `release-evidence/${RELEASE}/attestations/${scenario}.json`,
        record,
      );
      continue;
    }
    records[key] = write(
      root,
      `release-evidence/${RELEASE}/attestations/${scenario}.json`,
      lifecycleRecord(digests, previousTag, inventorySha, overrides),
    );
  }

  const witnesses = {
    witness_missing_assets: { status: "FAIL", stage: "resolve-installers" },
    witness_timeout: { status: "TIMEOUT", stage: "sandbox-timeout" },
    witness_malformed_result: { status: "FAIL", stage: "sandbox-run" },
    witness_preservation_missing: {
      status: "FAIL",
      stage: "preservation-verification",
      preservation: { status: "missing-files" },
    },
    witness_stale_lock: { status: "FAIL", stage: "resolve-installers" },
    witness_live_lock: { status: "FAIL", stage: "resolve-installers" },
  };
  for (const [key, doc] of Object.entries(witnesses)) {
    const file = key.split("_").join("-");
    const record = { schema: "localmotive.sandbox-lifecycle.v0", ...doc };
    if (options.witnessOutcome?.key === key) {
      record.status = options.witnessOutcome.status;
    }
    records[key] = write(
      root,
      `release-evidence/${RELEASE}/attestations/${file}.json`,
      record,
    );
  }

  records.mt06_cancellation = write(
    root,
    `release-evidence/${RELEASE}/attestations/mt06-cycles-result.json`,
    {
      schema: "localmotive.mt06-cancellation.v1",
      sourceRevision: options.mt06Source ?? SOURCE,
      portableDigest: digests.portable,
      checks: [{ ok: true }, { ok: true }],
      replacementRecordPath: "C:/benchmarks/replacement.json",
      summary: "6/6 checks PASS",
    },
  );
  records.installer_payload_identity = write(
    root,
    `release-evidence/${RELEASE}/attestations/installer-payload-identity.json`,
    {
      schema: "localmotive.installer-payload-identity.v1",
      status: "PASS",
      sourceRevision: SOURCE,
      inventorySha256: inventorySha,
      portable: { sha256: digests.portable },
      nsisPayload: { sha256: options.nsisPayloadDigest ?? "4".repeat(64) },
      msiPayload: { sha256: options.msiPayloadDigest ?? "5".repeat(64) },
    },
  );
  records.rt06_all_backends = write(
    root,
    `release-evidence/${RELEASE}/attestations/rt06-all-backends.json`,
    {
      schema: "localmotive.rt06-all-backends.v1",
      sourceRevision: options.rt06Source ?? SOURCE,
      portableDigest: digests.portable,
      installedCount: 4,
      refusedCount: 3,
      backendScope: {
        installed: [
          { installKey: "cpu" },
          { installKey: "cuda-12.4" },
          { installKey: "cuda-13.3" },
          { installKey: "vulkan" },
        ],
        refused: [{ installKey: "openvino" }, { installKey: "rocm" }, { installKey: "sycl" }],
        selected: ["cpu", "cuda-12.4", "cuda-13.3", "vulkan"],
        launched: ["cuda-13.3"],
        sevenBackendCriterionSatisfied: false,
      },
      checks: [{ ok: true }, { ok: true }],
      summary: { total: 2, pass: 2, fail: 0 },
    },
  );
  records.rt06_full_run_log = write(
    root,
    `release-evidence/${RELEASE}/attestations/rt06-full-run.log`,
    "RT06 SUMMARY: 12/12\n",
  );
  records.a11y_packaged_verification = write(
    root,
    `release-evidence/${RELEASE}/attestations/g05-a11y.log`,
    "A11Y_PASS\n",
  );
  records.dc04_command_path = write(
    root,
    `release-evidence/${RELEASE}/attestations/dc04.log`,
    "13/13 PASS\n",
  );
  records.rt04_delayed_download = write(
    root,
    `release-evidence/${RELEASE}/attestations/rt04.log`,
    "17/17 PASS\n",
  );

  if (options.withQualifiedSet) {
    // A promotion-ready set: the three staged binaries, their checksum file,
    // the packaged-verification asset (byte-identical to the producer record)
    // and the producer inventory, all in the layout a release bundle uses.
    const artifactsDir = join(root, "artifacts");
    mkdirSync(artifactsDir, { recursive: true });
    writeFileSync(join(artifactsDir, `Localmotive_${RELEASE}_x64-setup.exe`), bodies.setup);
    writeFileSync(join(artifactsDir, `Localmotive_${RELEASE}_x64-portable.exe`), bodies.portable);
    writeFileSync(join(artifactsDir, `Localmotive_${RELEASE}_x64.msi`), bodies.msi);
    const checksumLines = [
      `${digests.setup}  Localmotive_${RELEASE}_x64-setup.exe`,
      `${digests.portable}  Localmotive_${RELEASE}_x64-portable.exe`,
      `${digests.msi}  Localmotive_${RELEASE}_x64.msi`,
    ];
    writeFileSync(
      join(artifactsDir, `SHA256SUMS-${RELEASE}.txt`),
      checksumLines.join(String.fromCharCode(10)) + String.fromCharCode(10),
    );
    copyFileSync(
      records.packaged_verification.path,
      join(artifactsDir, `packaged-verification-${RELEASE}.json`),
    );
    copyFileSync(inventoryFile.path, join(artifactsDir, `candidate-inventory-${RELEASE}.json`));
  }

  if (options.missingRecord) delete records[options.missingRecord];
  if (options.logWithoutLf) delete records[options.logWithoutLf].sha256_lf;
  if (options.escapePath) records[options.escapePath].relative = "../outside.json";

  const recordEntries = {};
  for (const [key, file] of Object.entries(records)) {
    recordEntries[key] = {
      path: file.relative,
      exists: true,
      sha256: file.sha256,
      sha256_lf: file.sha256_lf,
    };
    if (file.carriedForward) recordEntries[key].carriedForward = file.carriedForward;
  }

  const manifest = {
    schema: "localmotive.qualification-manifest.v1",
    release: RELEASE,
    sourceRevision: options.manifestSource ?? SOURCE,
    generatedAt: "2026-09-13T00:00:00Z",
    workflowFile: { path: workflowFile.relative, sha256: workflowFile.sha256 },
    inventory: { path: inventoryFile.relative, sha256: options.inventorySha ?? inventorySha },
    artifacts: options.artifactOverride ?? approvedArtifacts(),
    records: recordEntries,
  };
  const manifestFile = write(
    root,
    `release-evidence/${RELEASE}/qualification-manifest-${RELEASE}.json`,
    manifest,
  );
  return { root, manifestFile: manifestFile.relative };
}

export function verify(fixture, options = {}) {
  return verifyQualificationManifest({
    manifestPath: fixture.manifestFile,
    root: fixture.root,
    ...options,
  });
}

/// Build a carried-forward lifecycle record: the record stays bound to a
/// superseded candidate whose inventory is committed under a history path.
/// `options.carryForward` = { key, citePath, reason, priorSource, priorSetup,
/// priorMsi, recordSetup, recordMsi, citeShaOverride }.
export function carryForwardFixture(root, options) {
  const {
    key,
    citePath = "release-evidence/0.6.0/history/freeze8/prior-inventory.json",
    reason = "installer and migration code paths are unchanged between the two candidates",
    priorSource = "c".repeat(40),
    priorSetup = "5".repeat(64),
    priorMsi = "6".repeat(64),
    recordSetup = priorSetup,
    recordMsi = priorMsi,
    citeShaOverride = null,
  } = options;
  const absolute = join(root, citePath);
  mkdirSync(dirname(absolute), { recursive: true });
  const prior = {
    schema: "localmotive.candidate-inventory.v1",
    release: RELEASE,
    sourceRevision: priorSource,
    artifacts: [
      { name: `Localmotive_${RELEASE}_x64-portable.exe`, sizeBytes: 11, sha256: "7".repeat(64) },
      { name: `Localmotive_${RELEASE}_x64-setup.exe`, sizeBytes: 12, sha256: priorSetup },
      { name: `Localmotive_${RELEASE}_x64.msi`, sizeBytes: 13, sha256: priorMsi },
    ],
  };
  writeFileSync(absolute, JSON.stringify(prior, null, 2), "utf8");
  const citeSha = sha256Of(readFileSync(absolute));
  return {
    key,
    entry: {
      evidenceFrom: citePath,
      evidenceFromSha256: citeShaOverride ?? citeSha,
      reason,
    },
    recordDigests: { currentSetup: recordSetup, currentMsi: recordMsi },
  };
}

export function withFixture(options, run) {
  const fixture = buildFixture(options);
  try {
    run(fixture);
  } finally {
    rmSync(fixture.root, { recursive: true, force: true });
  }
}

