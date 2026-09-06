import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const EXPECTED_SCOPE = ["research/0.4", "research/0.4.1"];
const EXPECTED_EXCLUDED = [
  "**/.git/**",
  "**/__pycache__/**",
  "**/*.pyc",
  "research/0.4.1/research-freeze.json",
  "research/0.4.1/research-freeze.sha256",
];
const EXPECTED_UNIT_TESTS = { status: "PASS", passed: 101, failed: 0, skipped: 0 };
const EXPECTED_RESEARCH_VERIFIER = {
  status: "PASS",
  corePassed: 63,
  passed: 65,
  failed: 0,
  unknown: 0,
};

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function stableValue(value) {
  if (Array.isArray(value)) return value.map(stableValue);
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.keys(value).sort().map((key) => [key, stableValue(value[key])]),
    );
  }
  return value;
}

function exactJson(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export function validateResearchAnchor({
  anchor,
  manifestBytes,
  approvals,
  verification = null,
  independentReviewBytes = null,
  trackedPaths = null,
}) {
  const failures = [];
  let manifest;
  try {
    manifest = JSON.parse(manifestBytes.toString("utf8"));
  } catch {
    return { ok: false, failures: ["manifest:json"] };
  }
  const expectedAnchorKeys = [
    "baseCommit",
    "excluded",
    "manifestPath",
    "manifestSha256",
    "productScopeSha256",
    "schemaVersion",
    "scope",
    "trackedManifestPath",
    "tree",
  ].sort();
  if (!anchor || !exactJson(Object.keys(anchor).sort(), expectedAnchorKeys)) failures.push("anchor:fields");
  if (anchor?.schemaVersion !== 1) failures.push("anchor:schema-version");
  if (anchor?.manifestPath !== "research/0.4.1/research-freeze.json") failures.push("anchor:manifest-path");
  if (anchor?.trackedManifestPath !== "release-evidence/0.4.1/research-freeze-manifest.json") failures.push("anchor:tracked-manifest-path");
  if (anchor?.manifestSha256 !== sha256(manifestBytes)) failures.push("anchor:manifest-sha256");
  if (!/^[0-9a-f]{40}$/u.test(anchor?.baseCommit ?? "") || anchor?.baseCommit !== manifest.base_commit) failures.push("anchor:base-commit");
  if (!exactJson(anchor?.scope, EXPECTED_SCOPE) || !exactJson(manifest.scope, EXPECTED_SCOPE)) failures.push("anchor:scope");
  if (!exactJson(anchor?.excluded, EXPECTED_EXCLUDED) || !exactJson(manifest.excluded, EXPECTED_EXCLUDED)) failures.push("anchor:excluded");
  if (manifest.schema_version !== 2) failures.push("manifest:schema-version");
  if (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z$/u.test(manifest.created_at ?? "")) failures.push("manifest:created-at");

  const files = Array.isArray(manifest.files) ? manifest.files : [];
  if (!Array.isArray(manifest.files)) failures.push("manifest:files");
  const paths = files.map((entry) => entry?.path);
  if (!exactJson(paths, [...paths].sort())) failures.push("manifest:path-order");
  if (new Set(paths).size !== paths.length) failures.push("manifest:duplicate-path");
  if (new Set(paths.map((path) => String(path).toLowerCase())).size !== paths.length) failures.push("manifest:casefold-path");
  for (const entry of files) {
    const validPath = typeof entry?.path === "string"
      && !entry.path.includes("\\")
      && !entry.path.includes(":")
      && !entry.path.startsWith("/")
      && !entry.path.split("/").some((part) => part === "" || part === "." || part === "..")
      && (entry.path.startsWith("research/0.4/") || entry.path.startsWith("research/0.4.1/"));
    if (!validPath || !Number.isSafeInteger(entry?.bytes) || entry.bytes < 0 || !/^[0-9a-f]{64}$/u.test(entry?.sha256 ?? "")) {
      failures.push(`manifest:invalid-entry:${entry?.path ?? "unknown"}`);
    }
  }
  const canonicalFiles = Buffer.from(JSON.stringify(stableValue(files)), "utf8");
  const calculatedTree = {
    algorithm: "sha256-canonical-file-manifest-v1",
    sha256: sha256(canonicalFiles),
    files: files.length,
    bytes: files.reduce((sum, entry) => sum + (Number.isSafeInteger(entry?.bytes) ? entry.bytes : 0), 0),
  };
  if (!exactJson(manifest.tree, calculatedTree) || !exactJson(anchor?.tree, calculatedTree)) failures.push("anchor:tree");
  const productScope = files.find((entry) => entry.path === "research/0.4.1/product-scope.json");
  if (!productScope || anchor?.productScopeSha256 !== productScope.sha256) failures.push("anchor:product-scope-sha256");
  const approved = approvals?.decisions?.some((decision) =>
    decision.id === "phase-0a-rebaseline"
    && decision.status === "APPROVED_WITH_CONDITIONS");
  if (!approved) failures.push("anchor:rebaseline-not-approved");
  if (!isPlainObject(verification)) {
    failures.push("verification:ledger");
  } else {
    const expectedVerificationKeys = [
      "baseCommit",
      "cleanCheckoutCoverage",
      "commands",
      "freeze",
      "independentReview",
      "observedAt",
      "release",
      "researchVerifier",
      "schemaVersion",
      "unitTests",
    ].sort();
    if (
      !exactJson(Object.keys(verification).sort(), expectedVerificationKeys)
      || verification.schemaVersion !== 1
      || verification.release !== "0.4.1"
      || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z$/u.test(verification.observedAt ?? "")
    ) {
      failures.push("verification:ledger");
    }
    const expectedCommands = [
      "python -m unittest discover -s research/0.4/tests -v",
      "python research/0.4/verify_research.py --core-evidence",
      `python research/0.4.1/freeze_research.py check --allow-product-changes --expected-manifest-sha256 ${anchor?.manifestSha256}`,
      "node scripts/verify_research_anchor.mjs",
      "python research/0.4/verify_research.py",
    ];
    if (!exactJson(verification.commands, expectedCommands)) failures.push("verification:commands");
    if (verification.baseCommit !== anchor?.baseCommit) failures.push("verification:base-commit");
    if (verification.freeze?.manifestSha256 !== anchor?.manifestSha256) failures.push("verification:manifest-sha256");
    if (verification.freeze?.treeSha256 !== anchor?.tree?.sha256) failures.push("verification:tree-sha256");
    if (verification.freeze?.productScopeSha256 !== anchor?.productScopeSha256) failures.push("verification:product-scope-sha256");
    if (!exactJson(verification.unitTests, EXPECTED_UNIT_TESTS)) {
      failures.push("verification:unit-tests");
    }
    if (!exactJson(verification.researchVerifier, EXPECTED_RESEARCH_VERIFIER)) {
      failures.push("verification:research-verifier");
    }
    if (
      verification.freeze?.status !== "PASS"
      || verification.freeze?.files !== anchor?.tree?.files
      || verification.freeze?.bytes !== anchor?.tree?.bytes
    ) {
      failures.push("verification:freeze");
    }
    if (
      verification.cleanCheckoutCoverage?.researchDirectoryIntentionallyIgnored !== true
      || verification.cleanCheckoutCoverage?.trackedManifestAndAnchorGate !== "PASS"
      || verification.cleanCheckoutCoverage?.fullResearchReexecutionInGitHubActions !== "UNAVAILABLE"
      || typeof verification.cleanCheckoutCoverage?.reason !== "string"
      || verification.cleanCheckoutCoverage.reason.length === 0
    ) {
      failures.push("verification:clean-checkout-coverage");
    }
    const review = verification.independentReview;
    if (review?.status !== "PASS") {
      failures.push("verification:independent-review");
    } else {
      const expectedPath = "release-evidence/0.4.1/phase0a-independent-review.txt";
      const reportText = Buffer.isBuffer(independentReviewBytes)
        ? independentReviewBytes.toString("utf8")
        : "";
      if (
        review.verdict !== "PASS"
        || review.reportPath !== expectedPath
        || !/^deleg_[0-9a-f]{8}$/u.test(review.delegationId ?? "")
        || !/^[0-9a-f]{64}$/u.test(review.reportSha256 ?? "")
        || !Buffer.isBuffer(independentReviewBytes)
        || sha256(independentReviewBytes ?? Buffer.alloc(0)) !== review.reportSha256
        || !reportText.includes(`Delegation: ${review.delegationId}`)
        || !/^Verdict: PASS$/mu.test(reportText)
      ) {
        failures.push("verification:independent-review-report");
      }
      if (
        review.manifestSha256 !== anchor?.manifestSha256
        || review.treeSha256 !== anchor?.tree?.sha256
        || !reportText.includes(`Manifest SHA-256: ${review.manifestSha256}`)
        || !reportText.includes(`Tree SHA-256: ${review.treeSha256}`)
      ) {
        failures.push("verification:independent-review-tree");
      }
      if (!(trackedPaths instanceof Set) || !trackedPaths.has(expectedPath)) {
        failures.push("verification:independent-review-not-tracked");
      }
    }
  }
  return { ok: failures.length === 0, failures };
}

function trackedByGit(root, path) {
  return spawnSync("git", ["ls-files", "--error-unmatch", "--", path], {
    cwd: root,
    encoding: "utf8",
    windowsHide: true,
  }).status === 0;
}

export async function verifyResearchAnchor(root = process.cwd()) {
  const absolute = resolve(root);
  const anchorPath = "release-evidence/0.4.1/research-freeze-anchor.json";
  const manifestPath = "release-evidence/0.4.1/research-freeze-manifest.json";
  const anchor = JSON.parse(await readFile(join(absolute, anchorPath), "utf8"));
  const manifestBytes = await readFile(join(absolute, manifestPath));
  const approvals = JSON.parse(await readFile(join(absolute, "release-evidence/0.4.1/approvals.json"), "utf8"));
  const verification = JSON.parse(await readFile(
    join(absolute, "release-evidence/0.4.1/research-verification.json"),
    "utf8",
  ));
  let independentReviewBytes = null;
  const independentReviewPath = "release-evidence/0.4.1/phase0a-independent-review.txt";
  try {
    independentReviewBytes = await readFile(join(
      absolute,
      independentReviewPath,
    ));
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }
  const result = validateResearchAnchor({
    anchor,
    manifestBytes,
    approvals,
    verification,
    independentReviewBytes,
    trackedPaths: new Set(
      [anchorPath, manifestPath, independentReviewPath]
        .filter((path) => trackedByGit(absolute, path)),
    ),
  });
  const failures = [...result.failures];
  if (!trackedByGit(absolute, anchorPath)) failures.push("anchor:not-tracked");
  if (!trackedByGit(absolute, manifestPath)) failures.push("manifest:not-tracked");
  try {
    const localManifest = await readFile(join(absolute, "research/0.4.1/research-freeze.json"));
    if (!localManifest.equals(manifestBytes)) failures.push("manifest:local-copy-mismatch");
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }
  return { ok: failures.length === 0, failures };
}

const invokedPath = process.argv[1] ? resolve(process.argv[1]) : null;
if (invokedPath === fileURLToPath(import.meta.url)) {
  const result = await verifyResearchAnchor(process.cwd());
  if (!result.ok) {
    for (const failure of result.failures) console.error(`FAIL ${failure}`);
    process.exitCode = 1;
  } else {
    console.log("PASS research freeze: tracked manifest and independent anchor agree");
  }
}
