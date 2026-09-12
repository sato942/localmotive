// Build the canonical candidate qualification manifest.
//
// One file that ties the frozen source revision, the producer candidate
// inventory, the exact artifact bytes, and every qualification record produced
// at that revision, with each record's own harness revision recorded
// separately. Records from earlier candidate cuts are labelled superseded
// rather than rewritten. Regenerate with:
//
//   node scripts/build_qualification_manifest.mjs \
//     --inventory release-evidence/0.6.0/candidate-inventory-0.6.0.json \
//     --attestations release-evidence/0.6.0/attestations \
//     --out release-evidence/0.6.0/qualification-manifest-0.6.0.json \
//     --release 0.6.0 [--superseded <path>]
//
// The verifier (`scripts/verify_qualification_manifest.mjs`) re-derives every
// relationship from a fresh checkout; this script only records them.
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ATTESTATION_RECORDS = {
  packaged_verification: (version) => `packaged-verification-${version}.json`,
  "lifecycle_upgrade_v0.4.0": () => "sandbox-clean-account-lifecycle-upgrade-v0.4.0.json",
  "lifecycle_upgrade_v0.5.0": () => "sandbox-clean-account-lifecycle-upgrade-v0.5.0.json",
  "lifecycle_preservation_v0.4.1": () => "sandbox-clean-account-lifecycle-preservation-v0.4.1.json",
  "lifecycle_preservation_v0.5.0": () => "sandbox-clean-account-lifecycle-preservation-v0.5.0.json",
  witness_missing_assets: () => "witness-missing-assets.json",
  witness_timeout: () => "witness-timeout.json",
  witness_malformed_result: () => "witness-malformed-result.json",
  witness_preservation_missing: () => "witness-preservation-missing.json",
  witness_stale_lock: () => "witness-stale-lock.json",
  witness_live_lock: () => "witness-live-lock.json",
  mt06_cancellation: () => "mt06-cycles-result.json",
  installer_payload_identity: () => "installer-payload-identity.json",
  dc04_command_path: () => "dc04-v2-command-path-verification.log",
  rt04_delayed_download: () => "rt04v2-delayed-download-verification.log",
  a11y_packaged_verification: () => "g05-a11y-packaged-verification.log",
  rt06_all_backends: () => "rt06-all-backends.json",
  rt06_full_run_log: () => "rt06-full-run.log",
};

function sha256Of(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function sha256LfOf(bytes) {
  const normalized = Buffer.from(bytes.toString("latin1").replace(/\r\n/g, "\n"), "latin1");
  return sha256Of(normalized);
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8").replace(/^\uFEFF/, ""));
}

function recordEntry(root, path) {
  const entry = { path: path.replaceAll("\\", "/"), exists: existsSync(path) };
  if (!entry.exists) return entry;
  const bytes = readFileSync(path);
  entry.sha256 = sha256Of(bytes);
  entry.sha256_lf = sha256LfOf(bytes);
  if (path.endsWith(".json")) {
    try {
      const doc = readJson(path);
      for (const key of ["status", "overall_status", "stage", "sourceRevision", "harnessRevision", "summary"]) {
        if (doc[key] !== undefined) entry[key] = doc[key];
      }
      if (Array.isArray(doc.checks)) {
        const passed = doc.checks.filter((check) => check.ok === true || check.status === "PASS").length;
        entry.checks = `${passed}/${doc.checks.length}`;
      }
    } catch (error) {
      entry.parse_error = String(error);
    }
  }
  return entry;
}

export function buildQualificationManifest({
  inventoryPath,
  attestationsDir,
  outPath,
  release,
  supersededPath = null,
  root = process.cwd(),
}) {
  const inventoryFile = resolve(root, inventoryPath);
  const inventory = readJson(inventoryFile);
  const manifest = {
    schema: "localmotive.qualification-manifest.v1",
    release,
    generatedAt: new Date().toISOString(),
    repositoryHeadAtGeneration: execFileSync("git", ["rev-parse", "HEAD"], {
      cwd: root,
      encoding: "utf8",
      windowsHide: true,
    }).trim(),
    sourceRevision: inventory.sourceRevision,
    workflowFile: {
      path: ".github/workflows/release.yml",
      sha256: sha256Of(readFileSync(resolve(root, ".github/workflows/release.yml"))),
    },
    inventory: {
      path: inventoryPath.replaceAll("\\", "/"),
      sha256: sha256Of(readFileSync(inventoryFile)),
    },
    artifacts: inventory.artifacts,
    records: {},
    supersededRecords: [],
  };
  for (const [key, nameFor] of Object.entries(ATTESTATION_RECORDS)) {
    const file = join(resolve(root, attestationsDir), nameFor(release));
    manifest.records[key] = recordEntry(root, file);
  }
  if (supersededPath) {
    manifest.supersededRecords.push(readJson(resolve(root, supersededPath)));
  }
  writeFileSync(outPath, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
  const missing = Object.entries(manifest.records)
    .filter(([, entry]) => !entry.exists)
    .map(([key]) => key);
  return { manifest, missing };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const option = (name, fallback = null) => {
    const index = args.indexOf(`--${name}`);
    return index === -1 ? fallback : args[index + 1];
  };
  const inventoryPath = option("inventory");
  const attestationsDir = option("attestations");
  const outPath = option("out");
  const release = option("release");
  if (!inventoryPath || !attestationsDir || !outPath || !release) {
    console.error(
      "usage: node scripts/build_qualification_manifest.mjs --inventory <path> --attestations <dir> --out <path> --release <version> [--superseded <path>]",
    );
    process.exit(2);
  }
  const { manifest, missing } = buildQualificationManifest({
    inventoryPath,
    attestationsDir,
    outPath,
    release,
    supersededPath: option("superseded"),
  });
  console.log(`manifest written: ${outPath}`);
  console.log(`sourceRevision: ${manifest.sourceRevision}`);
  console.log(`inventory: ${manifest.inventory.path}`);
  if (missing.length > 0) {
    console.error(`MISSING RECORDS: ${missing.join(", ")}`);
    process.exitCode = 1;
  } else {
    console.log(`all ${Object.keys(manifest.records).length} qualification records present`);
  }
}
