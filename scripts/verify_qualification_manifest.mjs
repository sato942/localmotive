// Qualification-manifest verifier: validates that a checkout (fresh or warm)
// carries the qualification manifest, the producer inventory it names, and
// every record it references, with the recorded outcomes and identities still
// matching the record contents. A validator must reject CONTRADICTORY
// evidence, not merely malformed fields: a value can be syntactically perfect
// and still disagree with its producer inventory, its candidate digests, or
// the artifact set the release would publish.
//
// Path and newline policy (fresh-checkout reproducibility):
//   - Recorded paths are repository-relative and may use either separator;
//     the verifier normalizes '\' to '/', refuses absolute paths, drive
//     letters, '..' segments, and scratch references (`.hermes-0.6`).
//   - Text records are hashed over LF-normalized bytes (`sha256_lf`); every
//     `.log` record must define that digest, and a checkout whose line
//     endings were smudged still validates when the LF digest matches.
//
// Checks per record:
//   - the referenced path exists (no gitignored references allowed)
//   - JSON records parse; status / pass-count match the manifest entry
//   - sha256 matches exactly, or matches after CRLF->LF normalization
//   - the record's own schema, scenario, versions, required steps, installed
//     identities and preservation outcome match its required contract
//   - candidate-bearing records' source revision and digests match the
//     producer inventory and the artifact list
// Top level:
//   - sourceRevision is a full 40-hex commit and (when requested) equals the
//     expected reviewed revision
//   - the artifact list is the canonical set with sizes and 64-hex digests
//   - the recorded inventory exists, parses, and agrees with the manifest on
//     release, source revision and every artifact size and digest
//   - the recorded release-workflow file digest matches the checkout
//
// Usage: node scripts/verify_qualification_manifest.mjs [manifestPath] [--expect-source <sha>]
// Exit 0 = all checks pass; 1 = at least one failure (each printed).

import { readFileSync, existsSync } from "node:fs";
import { createHash } from "node:crypto";
import { join, resolve } from "node:path";

const DEFAULT_MANIFEST = "release-evidence/0.6.0/qualification-manifest-0.6.0.json";

/// Scenario contract for every lifecycle record: the evidence name, the
/// baseline it must exercise, and the exactly-one preservation step it must
/// carry for that baseline.
const LIFECYCLE_RECORDS = {
  "lifecycle_upgrade_v0.4.0": { scenario: "sandbox-clean-account-lifecycle-upgrade-v0.4.0", previousTag: "v0.4.0" },
  "lifecycle_upgrade_v0.5.0": { scenario: "sandbox-clean-account-lifecycle-upgrade-v0.5.0", previousTag: "v0.5.0" },
  "lifecycle_preservation_v0.4.1": { scenario: "sandbox-clean-account-lifecycle-preservation-v0.4.1", previousTag: "v0.4.1" },
  "lifecycle_preservation_v0.5.0": { scenario: "sandbox-clean-account-lifecycle-preservation-v0.5.0", previousTag: "v0.5.0" },
};

const LIFECYCLE_BASE_STEPS = [
  "nsis-fresh-install-launch-uninstall",
  "msi-fresh-install-launch-uninstall",
  "nsis-update-from-previous-launch-uninstall",
];

/// Witness legs: each must carry its own bounded outcome at the documented
/// stage. A missing or PASS witness is a failure, never a skip.
const WITNESS_RECORDS = {
  witness_missing_assets: { status: "FAIL", stage: "resolve-installers" },
  witness_timeout: { status: "TIMEOUT", stage: "sandbox-timeout" },
  witness_malformed_result: { status: "FAIL", stage: "sandbox-run" },
  witness_preservation_missing: { status: "FAIL", stage: "preservation-verification", preservation: "missing-files" },
  witness_stale_lock: { status: "FAIL", stage: "resolve-installers" },
  witness_live_lock: { status: "FAIL", stage: "resolve-installers" },
};

/// Records whose absence means the release is not qualified. The negative
/// controls (byte-doctored refusals) are retained as history but not required
/// per candidate; every entry here is mandatory.
const REQUIRED_RECORDS = [
  "packaged_verification",
  "mt06_cancellation",
  "rt06_all_backends",
  "rt06_full_run_log",
  "a11y_packaged_verification",
  "installer_payload_identity",
  "dc04_command_path",
  "rt04_delayed_download",
  ...Object.keys(LIFECYCLE_RECORDS),
  ...Object.keys(WITNESS_RECORDS),
];

function sha256Of(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function sha256LfOf(bytes) {
  const normalized = Buffer.from(bytes.toString("latin1").replace(/\r\n/g, "\n"), "latin1");
  return sha256Of(normalized);
}

function normalizeRecordPath(path) {
  if (typeof path !== "string" || path.length === 0) return null;
  const normalized = path.replaceAll("\\", "/");
  if (normalized.startsWith("/") || /^[a-zA-Z]:/.test(normalized)) return null;
  if (normalized.split("/").includes("..")) return null;
  return normalized;
}

function readEntry(root, relativePath) {
  const absolute = join(root, relativePath);
  if (!existsSync(absolute)) return { exists: false };
  const bytes = readFileSync(absolute);
  return {
    exists: true,
    bytes,
    sha256: sha256Of(bytes),
    sha256_lf: sha256LfOf(bytes),
  };
}

function parseJson(bytes) {
  return JSON.parse(bytes.toString("utf8").replace(/^\uFEFF/, ""));
}

function countChecks(doc) {
  if (!Array.isArray(doc.checks)) return null;
  const passed = doc.checks.filter(
    (check) => check.ok === true || check.status === "PASS",
  ).length;
  return `${passed}/${doc.checks.length}`;
}

/**
 * @returns {{failures: string[], lines: string[]}}
 */
export function verifyQualificationManifest({ manifestPath, root = ".", expectedSourceRevision = null }) {
  const failures = [];
  const lines = [];
  const resolved = resolve(root, manifestPath);
  if (!existsSync(resolved)) {
    return { failures: [`manifest not found: ${manifestPath}`], lines };
  }
  let manifest;
  try {
    manifest = parseJson(readFileSync(resolved));
  } catch (error) {
    return { failures: [`manifest is not valid JSON: ${error}`], lines };
  }
  const release = manifest.release ?? "";

  if (manifest.schema !== "localmotive.qualification-manifest.v1") {
    failures.push(`manifest schema is not localmotive.qualification-manifest.v1: ${manifest.schema}`);
  }
  if (!/^\d+\.\d+\.\d+$/u.test(release)) {
    failures.push(`manifest release is not a semantic version: ${release}`);
  }
  if (!/^[0-9a-f]{40}$/.test(manifest.sourceRevision ?? "")) {
    failures.push(`sourceRevision is not a full 40-hex commit: ${manifest.sourceRevision}`);
  } else {
    if (expectedSourceRevision && manifest.sourceRevision !== expectedSourceRevision) {
      failures.push(
        `manifest source revision ${manifest.sourceRevision} does not match the expected reviewed revision ${expectedSourceRevision}`,
      );
    }
    lines.push(`sourceRevision ok: ${manifest.sourceRevision}`);
  }

  // --- Artifact list -------------------------------------------------------
  const artifacts = manifest.artifacts ?? [];
  if (artifacts.length !== 3) {
    failures.push(`expected 3 artifact entries, found ${artifacts.length}`);
  }
  const canonical = [
    `Localmotive_${release}_x64-portable.exe`,
    `Localmotive_${release}_x64-setup.exe`,
    `Localmotive_${release}_x64.msi`,
  ].sort();
  const names = artifacts.map((artifact) => artifact.name);
  if (new Set(names).size !== names.length) {
    failures.push("artifact list contains a duplicate name");
  }
  if (JSON.stringify([...names].sort()) !== JSON.stringify(canonical)) {
    failures.push(`artifact list is not the canonical release set: ${names.join(", ")}`);
  }
  for (const artifact of artifacts) {
    if (!/^[0-9a-f]{64}$/.test(artifact.sha256 ?? "")) {
      failures.push(`artifact ${artifact.name} has no 64-hex sha256`);
    }
    if (!(Number(artifact.sizeBytes ?? 0) > 0)) {
      failures.push(`artifact ${artifact.name} has no positive size`);
    }
  }
  lines.push(`artifacts ok: ${names.join(", ")}`);

  // --- Producer inventory relationship -------------------------------------
  const inventory = manifest.inventory ?? null;
  let inventoryDoc = null;
  let inventoryDigest = null;
  if (!inventory?.path || !inventory?.sha256) {
    failures.push("manifest does not record the producer candidate inventory");
  } else {
    const inventoryPath = normalizeRecordPath(inventory.path);
    if (!inventoryPath) {
      failures.push(`inventory path is not a repository-relative path: ${inventory.path}`);
    } else {
      const live = readEntry(root, inventoryPath);
      if (!live.exists) {
        failures.push(`producer inventory missing at ${inventoryPath}`);
      } else {
        inventoryDigest = live.sha256;
        if (live.sha256 !== inventory.sha256 && live.sha256_lf !== inventory.sha256) {
          failures.push(`producer inventory digest drifted: ${inventoryPath}`);
        }
        try {
          inventoryDoc = parseJson(live.bytes);
        } catch (error) {
          failures.push(`producer inventory is not valid JSON: ${error}`);
        }
      }
    }
  }
  if (inventoryDoc) {
    if (inventoryDoc.schemaVersion !== 1) {
      failures.push(`producer inventory schema ${inventoryDoc.schemaVersion} is not supported`);
    }
    if (inventoryDoc.release !== release) {
      failures.push(
        `inventory release ${inventoryDoc.release} contradicts the manifest release ${release}`,
      );
    }
    if (inventoryDoc.sourceRevision !== manifest.sourceRevision) {
      failures.push(
        `inventory source ${inventoryDoc.sourceRevision} contradicts the manifest source ${manifest.sourceRevision}`,
      );
    }
    const inventoryArtifacts = Array.isArray(inventoryDoc.artifacts) ? inventoryDoc.artifacts : [];
    const byName = new Map(inventoryArtifacts.map((artifact) => [artifact.name, artifact]));
    for (const artifact of artifacts) {
      const row = byName.get(artifact.name);
      if (!row) {
        failures.push(`inventory has no entry for the manifest artifact ${artifact.name}`);
        continue;
      }
      if (row.sha256 !== artifact.sha256) {
        failures.push(
          `artifact ${artifact.name} digest ${artifact.sha256} contradicts the inventory digest ${row.sha256}`,
        );
      }
      if (Number(row.sizeBytes) !== Number(artifact.sizeBytes)) {
        failures.push(
          `artifact ${artifact.name} size ${artifact.sizeBytes} contradicts the inventory size ${row.sizeBytes}`,
        );
      }
    }
    if (inventoryArtifacts.length !== artifacts.length) {
      failures.push(
        `inventory lists ${inventoryArtifacts.length} artifacts; the manifest lists ${artifacts.length}`,
      );
    }
    lines.push(`producer inventory ok: ${inventory.path}`);
  }

  // --- Workflow file -------------------------------------------------------
  const workflow = manifest.workflowFile ?? null;
  if (workflow?.path && workflow?.sha256) {
    const workflowPath = normalizeRecordPath(workflow.path);
    if (!workflowPath) {
      failures.push(`workflow path is not a repository-relative path: ${workflow.path}`);
    } else {
      const live = readEntry(root, workflowPath);
      if (!live.exists) {
        failures.push(`workflow file missing: ${workflowPath}`);
      } else if (live.sha256 !== workflow.sha256 && live.sha256_lf !== workflow.sha256) {
        failures.push(`workflow file digest drifted: ${workflowPath}`);
      } else {
        lines.push(`workflow file ok: ${workflowPath}`);
      }
    }
  } else {
    failures.push("manifest does not record the release workflow revision");
  }

  // --- Records -------------------------------------------------------------
  const records = manifest.records ?? {};
  for (const required of REQUIRED_RECORDS) {
    if (!records[required]) {
      failures.push(`required record ${required} is missing from the manifest`);
    }
  }
  const docs = new Map();
  let recordCount = 0;
  for (const [name, entry] of Object.entries(records)) {
    recordCount += 1;
    const relative = normalizeRecordPath(entry.path);
    if (!relative || /(^|\/)\.hermes-0\.6(\/|$)/.test(entry.path ?? "")) {
      failures.push(`${name}: record path must live in the repository, not scratch: ${entry.path}`);
      continue;
    }
    const live = readEntry(root, relative);
    if (!live.exists) {
      failures.push(`${name}: record missing at ${relative}`);
      continue;
    }
    let digestState = "exact";
    if (entry.sha256 && live.sha256 !== entry.sha256) {
      if (entry.sha256_lf && live.sha256_lf === entry.sha256_lf) {
        digestState = "eol-equivalent";
      } else {
        failures.push(`${name}: sha256 mismatch at ${relative}`);
        continue;
      }
    }
    if (relative.endsWith(".log") && !entry.sha256_lf) {
      failures.push(`${name}: text record must record its newline-normalized sha256_lf`);
      continue;
    }
    if (relative.endsWith(".json")) {
      let doc;
      try {
        doc = parseJson(live.bytes);
      } catch (error) {
        failures.push(`${name}: record is not valid JSON: ${error}`);
        continue;
      }
      docs.set(name, doc);
      const liveStatus = doc.status ?? doc.overall_status ?? null;
      if (entry.status && liveStatus !== entry.status) {
        failures.push(`${name}: status drifted (${liveStatus} != ${entry.status})`);
        continue;
      }
      if (entry.checks) {
        const liveChecks = countChecks(doc);
        if (liveChecks && liveChecks !== entry.checks) {
          failures.push(`${name}: checks drifted (${liveChecks} != ${entry.checks})`);
          continue;
        }
      }
      if (Array.isArray(doc.checks) && doc.checks.some((check) => check.ok === false)) {
        failures.push(`${name}: record contains a failed check`);
        continue;
      }
    } else if (relative.endsWith(".log")) {
      docs.set(name, live.bytes.toString("utf8").replace(/^\uFEFF/, ""));
    }
    lines.push(`${name}: ok (${digestState}) ${relative}`);
  }
  if (recordCount === 0) failures.push("manifest references no records");

  // Carry-forward ledger: a lifecycle record may be carried from a superseded
  // candidate ONLY with a justification that names the prior inventory, and
  // that inventory must be the one the record's own digests are bound to.
  // The record is refused if the citation does not resolve, is not a history
  // path, or contradicts the recorded staged digests.
  const carriedForward = new Map();
    for (const [key, entry] of Object.entries(records)) {
      const carried = entry?.carriedForward;
      if (!carried) continue;
      const citePath = normalizeRecordPath(carried.evidenceFrom);
      if (!citePath || !/^release-evidence\/[^/]+\/history\//.test(citePath)) {
        failures.push(`${key}: a carried-forward record must cite a history-path inventory`);
        continue;
      }
      const citeBytes = readEntry(root, citePath);
      if (!citeBytes.exists) {
        failures.push(`${key}: carried-forward evidence inventory ${citePath} is missing`);
        continue;
      }
      const citeExact = carried.evidenceFromSha256 ? citeBytes.sha256 === carried.evidenceFromSha256 : true;
      const citeLf = carried.evidenceFromSha256Lf ? citeBytes.sha256_lf === carried.evidenceFromSha256Lf : citeExact;
      if (!citeExact && !citeLf) {
        failures.push(`${key}: carried-forward evidence inventory digest drifted from the recorded citation`);
        continue;
      }
      let prior = null;
      try {
        prior = parseJson(citeBytes.bytes);
      } catch (error) {
        failures.push(`${key}: carried-forward evidence inventory is not valid JSON: ${error}`);
        continue;
      }
      if (!carried.reason || String(carried.reason).trim().length < 20) {
        failures.push(`${key}: a carried-forward record needs a written source-delta justification`);
      }
      if (prior.sourceRevision === manifest.sourceRevision) {
        failures.push(`${key}: carried-forward evidence cites the CURRENT candidate; re-run it instead`);
        continue;
      }
      const priorArtifacts = new Map((prior.artifacts ?? []).map((artifact) => [artifact.name, artifact]));
      const priorSetup = [...priorArtifacts.entries()].find(([name]) => name.endsWith("-setup.exe"))?.[1];
      const priorMsi = [...priorArtifacts.entries()].find(([name]) => name.endsWith(".msi"))?.[1];
      const doc = docs.get(key);
      if (typeof doc !== "object" || doc === null) {
        failures.push(`${key}: carried-forward record is not a parsed document`);
        continue;
      }
      if (doc.candidateDigests?.currentSetup !== priorSetup?.sha256) {
        failures.push(`${key}: carried-forward record's staged setup digest does not match its cited inventory`);
        continue;
      }
      if (doc.candidateDigests?.currentMsi !== priorMsi?.sha256) {
        failures.push(`${key}: carried-forward record's staged MSI digest does not match its cited inventory`);
        continue;
      }
      carriedForward.set(key, { citePath, priorSource: prior.sourceRevision, reason: carried.reason });
      lines.push(
        `carried forward: ${key} from ${prior.sourceRevision.slice(0, 12)} (${citePath}); ${carried.reason}`,
      );
    }


  // --- Per-record contracts ------------------------------------------------
  const inventoryRows = new Map(
    (inventoryDoc?.artifacts ?? []).map((artifact) => [artifact.name, artifact]),
  );
  const setupRow = inventoryRows.get(`Localmotive_${release}_x64-setup.exe`);
  const msiRow = inventoryRows.get(`Localmotive_${release}_x64.msi`);
  const portableRow = inventoryRows.get(`Localmotive_${release}_x64-portable.exe`);

  for (const [key, scenario] of Object.entries(LIFECYCLE_RECORDS)) {
    const doc = docs.get(key);
    if (!doc) continue;
    const label = `${key}`;
    if (doc.schema !== "localmotive.sandbox-lifecycle.v0") {
      failures.push(`${label}: schema ${doc.schema} is not the lifecycle contract`);
    }
    if (doc.status !== "PASS") {
      failures.push(`${label}: status ${doc.status} is not PASS`);
    }
    if (doc.version !== release) {
      failures.push(`${label}: version ${doc.version} contradicts the release ${release}`);
    }
    if (doc.tag !== `v${release}`) {
      failures.push(`${label}: tag ${doc.tag} contradicts the release tag v${release}`);
    }
    if (doc.previousTag !== scenario.previousTag) {
      failures.push(`${label}: previousTag ${doc.previousTag} does not match the scenario baseline ${scenario.previousTag}`);
    }
    if (doc.preservation?.status !== "PASS") {
      failures.push(`${label}: preservation ${doc.preservation?.status ?? "absent"} is not PASS`);
    }
    for (const step of [...LIFECYCLE_BASE_STEPS, `nsis-preservation-from-${scenario.previousTag}`]) {
      const found = (doc.steps ?? []).find((entry) => entry.name === step);
      if (!found) {
        failures.push(`${label}: required step ${step} is missing`);
      } else if (found.status !== "PASS") {
        failures.push(`${label}: required step ${step} is ${found.status}`);
      }
    }
    const carried = carriedForward.get(key) ?? null;
    if (carried) {
      // The ledger checked the record against the inventory it cites; the
      // current-candidate digests are deliberately not required here.
    } else {
      if (doc.sourceRevision !== manifest.sourceRevision) {
        failures.push(
          `${label}: source revision ${doc.sourceRevision} contradicts the manifest source ${manifest.sourceRevision}`,
        );
      }
      if (setupRow && String(doc.candidateDigests?.currentSetup).toLowerCase() !== setupRow.sha256) {
        failures.push(`${label}: staged setup digest contradicts the inventory`);
      }
      if (msiRow && String(doc.candidateDigests?.currentMsi).toLowerCase() !== msiRow.sha256) {
        failures.push(`${label}: staged MSI digest contradicts the inventory`);
      }
      if (inventoryDigest && doc.candidateInventorySha256 !== inventoryDigest) {
        failures.push(`${label}: candidateInventorySha256 does not match the recorded producer inventory`);
      }
    }
    if (!/^[0-9a-f]{40}$/.test(doc.harnessRevision ?? "")) {
      failures.push(`${label}: harness revision is not recorded`);
    }
    for (const label2 of ["NSIS fresh install", "MSI fresh install", "Pre-update install", "Post-update install", "Preservation upgrade"]) {
      if (!doc.installedDigests?.[label2]) {
        failures.push(`${label}: installed identity '${label2}' is not recorded`);
      }
    }
    if (!doc.coverageNote) {
      failures.push(`${label}: coverage note is missing (the record must scope what it established)`);
    }
  }

  // The payload-identity record binds the installed digests to the staged
  // installers, so the lifecycle records and this document must agree.
  const payloadIdentity = docs.get("installer_payload_identity");
  if (payloadIdentity) {
    if (payloadIdentity.schema !== "localmotive.installer-payload-identity.v1") {
      failures.push(`installer_payload_identity: schema ${payloadIdentity.schema} is not the payload-identity contract`);
    }
    if (payloadIdentity.status !== "PASS") {
      failures.push(`installer_payload_identity: status ${payloadIdentity.status} is not PASS`);
    }
    if (payloadIdentity.sourceRevision !== manifest.sourceRevision) {
      failures.push(
        `installer_payload_identity: source ${payloadIdentity.sourceRevision} contradicts the manifest source`,
      );
    }
    if (inventoryDigest && payloadIdentity.inventorySha256 !== inventoryDigest) {
      failures.push("installer_payload_identity: inventory digest does not match the producer inventory");
    }
    if (portableRow && payloadIdentity.portable?.sha256 !== portableRow.sha256) {
      failures.push("installer_payload_identity: portable digest contradicts the inventory");
    }
  }

  // F9-03: validate the ACTUAL producer schema (scripts/verify_041.mjs writes
  // schema_version, verifier, source_revision, source_dirty, artifact {name,
  // size_bytes, sha256}, per-check status and overall_status). The earlier
  // branch read optional camelCase aliases that the producer never writes, so
  // a record with the real fields was only checked for its aggregate string
  // and every semantic contradiction (wrong source, wrong artifact identity,
  // a failed check under a passing aggregate) validated silently. No field is
  // optional here: absence is a failure, never a skipped check.
  const packaged = docs.get("packaged_verification");
  if (packaged) {
    if (packaged.schema_version !== "1.0.0") {
      failures.push(
        `packaged_verification: schema_version ${packaged.schema_version} is not the producer contract`,
      );
    }
    if (typeof packaged.verifier !== "string" || packaged.verifier.trim() === "") {
      failures.push("packaged_verification: the producing verifier is not recorded");
    }
    if (typeof packaged.source_dirty !== "boolean") {
      failures.push("packaged_verification: the source-dirty flag is not recorded");
    }
    if (!/^[0-9a-f]{40}$/.test(packaged.source_revision ?? "")) {
      failures.push("packaged_verification: source_revision is missing or is not a full commit SHA");
    } else if (packaged.source_revision !== manifest.sourceRevision) {
      failures.push(
        `packaged_verification: source_revision ${packaged.source_revision} contradicts the manifest source ${manifest.sourceRevision}`,
      );
    }
    const artifact = packaged.artifact;
    if (typeof artifact !== "object" || artifact === null) {
      failures.push("packaged_verification: the artifact identity is missing");
    } else {
      if (typeof artifact.name !== "string" || artifact.name.trim() === "") {
        failures.push("packaged_verification: the artifact name is missing");
      }
      if (!Number.isFinite(Number(artifact.size_bytes))) {
        failures.push("packaged_verification: the artifact size is missing");
      }
      if (!/^[0-9a-f]{64}$/.test(artifact.sha256 ?? "")) {
        failures.push("packaged_verification: the artifact digest is missing or malformed");
      }
      if (portableRow) {
        if (artifact.name !== portableRow.name) {
          failures.push(
            `packaged_verification: artifact name ${artifact.name} contradicts the inventory ${portableRow.name}`,
          );
        }
        if (Number(artifact.size_bytes) !== Number(portableRow.sizeBytes)) {
          failures.push(
            `packaged_verification: artifact size ${artifact.size_bytes} contradicts the inventory ${portableRow.sizeBytes}`,
          );
        }
        if (artifact.sha256 !== portableRow.sha256) {
          failures.push("packaged_verification: artifact digest contradicts the inventory");
        }
      }
    }
    if (!Array.isArray(packaged.checks) || packaged.checks.length === 0) {
      failures.push("packaged_verification: no per-check outcomes are recorded");
    } else {
      let recordedFailed = false;
      for (const [index, check] of packaged.checks.entries()) {
        const label = `packaged_verification: check ${check?.id ?? `#${index}`}`;
        if (typeof check?.id !== "string" || check.id.trim() === "") {
          failures.push(`${label}: the check id is missing`);
          continue;
        }
        if (check.status !== "PASS" && check.status !== "FAIL") {
          failures.push(`${label}: status ${check.status} is not a recorded outcome`);
          continue;
        }
        if (check.status === "FAIL") {
          recordedFailed = true;
          failures.push(`${label} is FAIL, so the aggregate cannot pass`);
        }
      }
      if (!recordedFailed && packaged.overall_status !== "PASS") {
        failures.push(
          `packaged_verification: overall_status ${packaged.overall_status} contradicts every PASS check`,
        );
      }
    }
    if (packaged.overall_status !== "PASS") {
      failures.push(`packaged_verification: overall_status ${packaged.overall_status} is not PASS`);
    }
  }

  const mt06 = docs.get("mt06_cancellation");
  if (mt06) {
    if (mt06.schema !== "localmotive.mt06-cancellation.v1") {
      failures.push(`mt06_cancellation: schema ${mt06.schema} is not the cancellation contract`);
    }
    if (mt06.sourceRevision !== manifest.sourceRevision) {
      failures.push("mt06_cancellation: source revision contradicts the manifest source");
    }
    if (!mt06.portableDigest) {
      failures.push("mt06_cancellation: the portable digest is not recorded");
    } else if (portableRow && mt06.portableDigest !== portableRow.sha256) {
      failures.push("mt06_cancellation: portable digest contradicts the inventory");
    }
    if (!Array.isArray(mt06.checks) || mt06.checks.length === 0) {
      failures.push("mt06_cancellation: no checks recorded");
    }
    if (!mt06.replacementRecordPath) {
      failures.push("mt06_cancellation: the replacement run's terminal record is not recorded");
    }
  }

  const rt06 = docs.get("rt06_all_backends");
  if (rt06) {
    if (rt06.schema !== "localmotive.rt06-all-backends.v1") {
      failures.push(`rt06_all_backends: schema ${rt06.schema} is not the backend-matrix contract`);
    }
    if (!rt06.sourceRevision) {
      failures.push("rt06_all_backends: the candidate source revision is not recorded");
    } else if (rt06.sourceRevision !== manifest.sourceRevision) {
      failures.push("rt06_all_backends: source revision contradicts the manifest source");
    }
    if (portableRow && rt06.portableDigest && rt06.portableDigest !== portableRow.sha256) {
      failures.push("rt06_all_backends: portable digest contradicts the inventory");
    }
    const scope = rt06.backendScope;
    if (!scope || !Array.isArray(scope.installed) || !Array.isArray(scope.refused)) {
      failures.push("rt06_all_backends: the installed/refused backend scope is not recorded");
    } else if (rt06.installedCount !== scope.installed.length || rt06.refusedCount !== scope.refused.length) {
      failures.push("rt06_all_backends: the declared backend counts do not match the scope lists");
    }
  }

  // The installed payload digests recorded by the lifecycle legs must match
  // the digests measured from the staged installers.
  if (payloadIdentity) {
    for (const [key, doc] of docs.entries()) {
      if (!LIFECYCLE_RECORDS[key]) continue;
      if (carriedForward.has(key)) continue; // bound to the prior candidate's payloads
      if (typeof doc !== "object" || doc === null) continue;
      if (payloadIdentity.nsisPayload?.sha256 && doc.nsisPayloadDigest !== payloadIdentity.nsisPayload.sha256) {
        failures.push(`${key}: installed NSIS payload digest contradicts the staged installer payload`);
      }
      if (payloadIdentity.msiPayload?.sha256 && doc.msiPayloadDigest !== payloadIdentity.msiPayload.sha256) {
        failures.push(`${key}: installed MSI payload digest contradicts the staged installer payload`);
      }
    }
  }

  for (const [key, expectations] of Object.entries(WITNESS_RECORDS)) {
    const doc = docs.get(key);
    if (!doc) continue;
    if (doc.status !== expectations.status) {
      failures.push(`${key}: status ${doc.status} != required witness outcome ${expectations.status}`);
    }
    if (doc.stage !== expectations.stage) {
      failures.push(`${key}: stage ${doc.stage} != required witness stage ${expectations.stage}`);
    }
    if (expectations.preservation && doc.preservation?.status !== expectations.preservation) {
      failures.push(`${key}: preservation ${doc.preservation?.status ?? "absent"} != ${expectations.preservation}`);
    }
  }

  // Records that carry a candidate binding must agree with the manifest.
  for (const [key, doc] of docs.entries()) {
    if (LIFECYCLE_RECORDS[key] || !doc || typeof doc !== "object") continue;
    if (typeof doc.sourceRevision === "string" && /^[0-9a-f]{40}$/.test(doc.sourceRevision)) {
      if (key.startsWith("negative_control")) continue;
      if (doc.sourceRevision !== manifest.sourceRevision) {
        failures.push(`${key}: source revision ${doc.sourceRevision} contradicts the manifest source`);
      }
    }
  }

  lines.push(`records checked: ${recordCount}`);
  return { failures, lines };
}

function main() {
  const args = process.argv.slice(2);
  const expectIndex = args.indexOf("--expect-source");
  const expectedSourceRevision = expectIndex === -1 ? null : args[expectIndex + 1];
  const manifestPath = args.find((arg, index) => !arg.startsWith("--") && index !== expectIndex + 1) ?? DEFAULT_MANIFEST;
  const { failures, lines } = verifyQualificationManifest({ manifestPath, expectedSourceRevision });
  for (const line of lines) console.log(`PASS ${line}`);
  for (const failure of failures) console.error(`FAIL ${failure}`);
  if (failures.length > 0) {
    console.error(`MANIFEST VERIFY FAILED (${failures.length} failure(s))`);
    process.exitCode = 1;
    return;
  }
  console.log(`MANIFEST VERIFY PASS (${manifestPath})`);
}

if (process.argv[1] && process.argv[1].endsWith("verify_qualification_manifest.mjs")) {
  main();
}
