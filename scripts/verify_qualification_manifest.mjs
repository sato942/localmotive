// Qualification-manifest verifier: validates that a checkout (fresh or warm)
// carries the qualification manifest and every record it references, with the
// recorded outcomes still matching the record contents.
//
// Checks per record:
//   - the referenced path exists (no gitignored references allowed)
//   - JSON records parse and their status / pass-count matches the manifest
//   - sha256 matches exactly, or matches after CRLF->LF normalization (git
//     may smudge line endings on checkout, so a fresh clone validates on any
//     platform)
// Top level:
//   - sourceRevision is a full 40-hex commit id
//   - the artifact list carries three named digests with sizes
//   - the recorded release-workflow file digest matches the checkout
//
// Usage: node scripts/verify_qualification_manifest.mjs [manifestPath]
// Exit 0 = all checks pass; 1 = at least one failure (each printed).

import { readFileSync, existsSync } from "node:fs";
import { createHash } from "node:crypto";
import { dirname, join, resolve } from "node:path";

const DEFAULT_MANIFEST = "release-evidence/0.6.0/qualification-manifest-0.6.0.json";

function sha256Of(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function sha256LfOf(bytes) {
  const normalized = Buffer.from(bytes.toString("latin1").replace(/\r\n/g, "\n"), "latin1");
  return sha256Of(normalized);
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
export function verifyQualificationManifest({ manifestPath, root = "." }) {
  const failures = [];
  const lines = [];
  const resolved = resolve(root, manifestPath);
  if (!existsSync(resolved)) {
    return { failures: [`manifest not found: ${manifestPath}`], lines };
  }
  let manifest;
  try {
    manifest = JSON.parse(readFileSync(resolved, "utf8").replace(/^\uFEFF/, ""));
  } catch (error) {
    return { failures: [`manifest is not valid JSON: ${error}`], lines };
  }

  if (!/^[0-9a-f]{40}$/.test(manifest.sourceRevision ?? "")) {
    failures.push(`sourceRevision is not a full 40-hex commit: ${manifest.sourceRevision}`);
  } else {
    lines.push(`sourceRevision ok: ${manifest.sourceRevision}`);
  }

  const artifacts = manifest.artifacts ?? [];
  if (artifacts.length !== 3) {
    failures.push(`expected 3 artifact entries, found ${artifacts.length}`);
  }
  for (const artifact of artifacts) {
    if (!/^[0-9a-f]{64}$/.test(artifact.sha256 ?? "")) {
      failures.push(`artifact ${artifact.name} has no 64-hex sha256`);
    }
    if (!(Number(artifact.sizeBytes ?? artifact.size ?? 0) > 0)) {
      failures.push(`artifact ${artifact.name} has no positive size`);
    }
  }
  lines.push(`artifacts ok: ${artifacts.map((a) => a.name).join(", ")}`);

  const workflow = manifest.workflowFile ?? null;
  if (workflow?.path && workflow?.sha256) {
    const live = readEntry(root, workflow.path);
    if (!live.exists) {
      failures.push(`workflow file missing: ${workflow.path}`);
    } else if (
      live.sha256 !== workflow.sha256 &&
      live.sha256_lf !== workflow.sha256
    ) {
      failures.push(`workflow file digest drifted: ${workflow.path}`);
    } else {
      lines.push(`workflow file ok: ${workflow.path}`);
    }
  } else {
    failures.push("manifest does not record the release workflow revision");
  }

  let recordCount = 0;
  for (const [name, entry] of Object.entries(manifest.records ?? {})) {
    recordCount += 1;
    if (!entry.path || /(^|[\\/])\.hermes-0\.6([\\/]|$)/.test(entry.path)) {
      failures.push(`${name}: record path must live in the repository, not scratch: ${entry.path}`);
      continue;
    }
    const live = readEntry(root, entry.path);
    if (!live.exists) {
      failures.push(`${name}: record missing at ${entry.path}`);
      continue;
    }
    let digestState = "exact";
    if (entry.sha256 && live.sha256 !== entry.sha256) {
      if (entry.sha256_lf && live.sha256_lf === entry.sha256_lf) {
        digestState = "eol-equivalent";
      } else {
        failures.push(`${name}: sha256 mismatch at ${entry.path}`);
        continue;
      }
    }
    if (entry.path.endsWith(".json")) {
      let doc;
      try {
        doc = JSON.parse(live.bytes.toString("utf8").replace(/^\uFEFF/, ""));
      } catch (error) {
        failures.push(`${name}: record is not valid JSON: ${error}`);
        continue;
      }
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
    }
    lines.push(`${name}: ok (${digestState}) ${entry.path}`);
  }
  if (recordCount === 0) failures.push("manifest references no records");
  lines.push(`records checked: ${recordCount}`);
  return { failures, lines };
}

function main() {
  const manifestPath = process.argv[2] ?? DEFAULT_MANIFEST;
  const { failures, lines } = verifyQualificationManifest({ manifestPath });
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
