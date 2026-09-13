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
import { join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const ATTESTATION_RECORDS = {
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
  // Record repository-relative paths: a fresh checkout validates the same
  // manifest, and the verifier refuses scratch paths outright.
  const rel = relative(root, path).replaceAll("\\", "/");
  const entry = { path: rel.startsWith("..") ? path.replaceAll("\\", "/") : rel, exists: existsSync(path) };
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

/// Mark lifecycle records as carried forward from a superseded candidate.
/// The mandate allows this only with an explicit source-delta justification,
/// and the manifest must name the exact prior inventory the records are bound
/// to; the verifier refuses a carry-forward whose cited inventory does not
/// match the record's own candidate digests.
export function carryForwardEntries({
  keys,
  evidenceFrom,
  reason,
  root = process.cwd(),
}) {
  if (!evidenceFrom || !reason) throw new Error("a carry-forward needs evidenceFrom and a reason");
  const from = resolve(root, evidenceFrom);
  const inventory = readJson(from);
  return {
    keys,
    entry: {
      evidenceFrom: evidenceFrom.replaceAll("\\", "/"),
      evidenceFromSha256: sha256Of(readFileSync(from)),
      evidenceFromSha256Lf: sha256LfOf(readFileSync(from)),
      evidenceFromSource: inventory.sourceRevision,
      reason,
    },
  };
}

export function buildQualificationManifest({
  inventoryPath,
  attestationsDir,
  outPath,
  release,
  supersededPath = null,
  carryForward = null,
  carryForwardGroups = null,
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
  // A second freeze can carry records from two different superseded
  // candidates, so the ledger accepts one group per citation. The single
  // `carryForward` form stays supported (one group).
  const groups = carryForwardGroups ?? (carryForward ? [carryForward] : []);
  const entryFor = new Map();
  for (const group of groups) {
    for (const key of group.keys ?? []) entryFor.set(key, group.entry);
  }
  for (const [key, nameFor] of Object.entries(ATTESTATION_RECORDS)) {
    const file = join(resolve(root, attestationsDir), nameFor(release));
    manifest.records[key] = recordEntry(root, file);
    const carried = entryFor.get(key);
    if (carried) {
      manifest.records[key].carriedForward = carried;
    }
  }
  const noteGroups = carryForwardGroups ?? (carryForward ? [carryForward] : []);
  if (noteGroups.length > 0) {
    manifest.carryForwardNote =
      "These lifecycle records were produced against the candidate named by evidenceFrom, not this candidate: " +
      noteGroups
        .map((group) => `${(group.keys ?? []).join(", ")} -> ${group.entry?.evidenceFrom}: ${group.entry?.reason}`)
        .join(" | ");
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
  const carriedKeys = (option("carry-forward-lifecycle") ?? "")
    .split(",")
    .map((key) => key.trim())
    .filter(Boolean);
  const carryForward =
    carriedKeys.length > 0
      ? carryForwardEntries({
          keys: carriedKeys,
          evidenceFrom: option("carry-forward-evidence"),
          reason: option("carry-forward-reason"),
        })
      : null;
  // --carry-forward-groups <file>: {"groups":[{keys,evidenceFrom,reason}, ...]}
  // lets one freeze carry records from two superseded candidates at once.
  let carryForwardGroups = null;
  const groupsPath = option("carry-forward-groups");
  if (groupsPath) {
    const parsed = readJson(resolve(process.cwd(), groupsPath));
    carryForwardGroups = (parsed.groups ?? []).map((group) =>
      carryForwardEntries({ keys: group.keys, evidenceFrom: group.evidenceFrom, reason: group.reason }),
    );
  }
  const { manifest, missing } = buildQualificationManifest({
    inventoryPath,
    attestationsDir,
    outPath,
    release,
    supersededPath: option("superseded"),
    carryForward,
    carryForwardGroups,
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
