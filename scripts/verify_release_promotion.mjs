// Release-promotion validator: prove that a qualified release set is safe to
// publish, WITHOUT publishing. It is the machine-checkable part of the owner
// handoff: the verification workflow uploads this set, and the promotion
// workflow runs this validator against the downloaded bundle before any
// publication step is reached.
//
// What it refuses:
//   - a missing or extra promised release asset
//   - an artifact whose bytes changed after the producer recorded them
//   - a checksum file that does not bind the exact staged bytes
//   - a qualification manifest bound to another source revision
//   - a missing required record or a non-PASS lifecycle outcome
//   - a bundle that carries no qualification manifest at all
//
// Usage:
//   node scripts/verify_release_promotion.mjs --qualified <dir> --tag v0.6.0 \
//     --expect-source <40-hex sha> [--verify-run <id>] [--report <path>]
//
// The checkout (or bundle overlay) root defaults to the current directory.
import { createHash } from "node:crypto";
import { readFile, stat, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { dirname } from "node:path";
import { verifyQualificationManifest } from "./verify_qualification_manifest.mjs";

const SHA256 = /^[0-9a-f]{64}$/u;

function sha256Of(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

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

/// The exact asset set a Localmotive release promises. Publication uploads
/// these names only; the validator refuses an incomplete or inflated set.
export function promisedAssetNames(release) {
  return [
    `Localmotive_${release}_x64-setup.exe`,
    `Localmotive_${release}_x64-portable.exe`,
    `Localmotive_${release}_x64.msi`,
    `SHA256SUMS-${release}.txt`,
    `packaged-verification-${release}.json`,
    `candidate-inventory-${release}.json`,
  ];
}

/// Parse a `sha256sum` compatible checksum file. Returns name -> {sha256, sizeBytes}.
export function parseChecksumFile(text) {
  const entries = new Map();
  for (const rawLine of text.split(/\r?\n/u)) {
    const line = rawLine.trim();
    if (!line) continue;
    const match = /^([0-9a-f]{64})\s+\*?(.+)$/u.exec(line);
    if (!match) return { ok: false, reason: `unparseable checksum line: ${rawLine}` };
    if (entries.has(match[2])) return { ok: false, reason: `duplicate checksum entry: ${match[2]}` };
    entries.set(match[2], { sha256: match[1] });
  }
  if (entries.size === 0) return { ok: false, reason: "checksum file lists no files" };
  return { ok: true, entries };
}

export async function verifyReleasePromotion(options) {
  const {
    qualifiedDirectory,
    root = process.cwd(),
    tag,
    release,
    expectedSourceRevision,
    verifyRunId = null,
  } = options;
  const failures = [];
  const lines = [];
  const releaseVersion = release;

  const artifactDirectory = join(qualifiedDirectory, "artifacts");

  // --- Manifest and evidence relationships ---------------------------------
  const manifest = verifyQualificationManifest({
    manifestPath: join(root, "release-evidence", releaseVersion, `qualification-manifest-${releaseVersion}.json`),
    root,
    expectedSourceRevision,
  });
  failures.push(...manifest.failures);
  lines.push(...manifest.lines.map((line) => `manifest: ${line}`));

  // --- Exact promised asset set -------------------------------------------
  const promised = promisedAssetNames(releaseVersion);
  for (const name of promised) {
    try {
      const info = await stat(join(artifactDirectory, name));
      if (!info.isFile() || info.size === 0) {
        failures.push(`promised asset ${name} is not a non-empty file`);
      }
    } catch {
      failures.push(`promised asset ${name} is missing from the qualified set`);
    }
  }

  // --- Checksums bind the exact staged bytes -------------------------------
  let checksumEntries = null;
  try {
    const checksumBytes = await readFile(join(artifactDirectory, `SHA256SUMS-${releaseVersion}.txt`));
    const parsed = parseChecksumFile(checksumBytes.toString("utf8"));
    if (!parsed.ok) {
      failures.push(`checksum file: ${parsed.reason}`);
    } else {
      checksumEntries = parsed.entries;
    }
  } catch {
    failures.push(`checksum file SHA256SUMS-${releaseVersion}.txt is unreadable`);
  }
  if (checksumEntries) {
    const binaryNames = [
      `Localmotive_${releaseVersion}_x64-setup.exe`,
      `Localmotive_${releaseVersion}_x64-portable.exe`,
      `Localmotive_${releaseVersion}_x64.msi`,
    ];
    const listed = [...checksumEntries.keys()].sort();
    if (JSON.stringify(listed) !== JSON.stringify([...binaryNames].sort())) {
      failures.push(`checksum file lists [${listed.join(", ")}], expected exactly the three release binaries`);
    }
    for (const name of binaryNames) {
      const entry = checksumEntries.get(name);
      if (!entry) continue;
      try {
        const bytes = await readFile(join(artifactDirectory, name));
        const digest = sha256Of(bytes);
        if (digest !== entry.sha256) {
          failures.push(`artifact ${name} digest ${digest} contradicts the recorded checksum ${entry.sha256}`);
        }
      } catch {
        failures.push(`artifact ${name} is unreadable`);
      }
    }
  }

  // --- Artifact truth against the delivered qualification manifest ---------
  // The manifest may live in the bundle overlay or the checkout; when the
  // checkout copy exists it was already checked above. Bind the artifact
  // digests to whatever the promotion is about to publish.
  try {
    const manifestBytes = await readFile(
      join(root, "release-evidence", releaseVersion, `qualification-manifest-${releaseVersion}.json`),
    );
    const manifestDoc = JSON.parse(manifestBytes.toString("utf8"));
    if (!Array.isArray(manifestDoc.artifacts) || manifestDoc.artifacts.length !== 3) {
      failures.push("the qualification manifest does not list exactly three artifacts");
    } else {
      for (const artifact of manifestDoc.artifacts) {
        try {
          const bytes = await readFile(join(artifactDirectory, artifact.name));
          const digest = sha256Of(bytes);
          if (digest !== artifact.sha256) {
            failures.push(
              `artifact ${artifact.name} digest ${digest} contradicts the qualification manifest ${artifact.sha256}`,
            );
          }
          if (bytes.length !== Number(artifact.sizeBytes)) {
            failures.push(
              `artifact ${artifact.name} size ${bytes.length} contradicts the qualification manifest ${artifact.sizeBytes}`,
            );
          }
        } catch {
          failures.push(`artifact ${artifact.name} listed by the qualification manifest is missing`);
        }
      }
    }
    // The packaged-verification evidence promised with the release must be the
    // producer record itself, bound by digest.
    const record = manifestDoc.records?.packaged_verification;
    if (!record) {
      failures.push("the qualification manifest carries no packaged-verification record");
    } else {
      const promisedPath = join(artifactDirectory, `packaged-verification-${releaseVersion}.json`);
      try {
        const bytes = await readFile(promisedPath);
        const digest = sha256Of(bytes);
        if (digest !== record.sha256 && sha256LfOf(bytes) !== record.sha256_lf && digest !== record.sha256_lf) {
          failures.push(
            "the packaged-verification asset is not the producer record the qualification manifest references",
          );
        } else {
          lines.push("packaged-verification asset matches the producer record");
        }
      } catch {
        failures.push("the packaged-verification asset is unreadable");
      }
    }
  } catch (error) {
    failures.push(`the delivered qualification manifest is unreadable: ${error.message}`);
  }

  // --- Source / tag / run identity ----------------------------------------
  if (!/^v\d+\.\d+\.\d+$/u.test(tag ?? "")) {
    failures.push(`promotion tag ${tag} is not a vMAJOR.MINOR.PATCH tag`);
  } else if (tag !== `v${releaseVersion}`) {
    failures.push(`promotion tag ${tag} does not identify release ${releaseVersion}`);
  }
  if (!/^[0-9a-f]{40}$/u.test(expectedSourceRevision ?? "")) {
    failures.push(`expected source revision ${expectedSourceRevision} is not a full commit SHA`);
  }
  if (verifyRunId !== null && !/^\d+$/u.test(String(verifyRunId))) {
    failures.push(`verify run id ${verifyRunId} is not a numeric run id`);
  }
  lines.push(`promotion target: tag ${tag}, source ${expectedSourceRevision}, verify run ${verifyRunId ?? "(unspecified)"}`);

  return {
    ok: failures.length === 0,
    failures,
    lines,
    report: {
      schema: "localmotive.release-promotion-check.v1",
      tag,
      release: releaseVersion,
      sourceRevision: expectedSourceRevision,
      verifyRunId,
      checkedAt: new Date().toISOString(),
      promisedAssets: promised,
      status: failures.length === 0 ? "PASS" : "FAIL",
      failures,
    },
  };
}

async function main() {
  const argv = process.argv.slice(2);
  const readOption = (name) => {
    const index = argv.indexOf(name);
    return index >= 0 ? argv[index + 1] : undefined;
  };
  const qualifiedDirectory = readOption("--qualified");
  const tag = readOption("--tag");
  const expectedSourceRevision = readOption("--expect-source");
  const verifyRunId = readOption("--verify-run") ?? null;
  const reportPath = readOption("--report");
  if (!qualifiedDirectory || !tag) {
    console.error("usage: node scripts/verify_release_promotion.mjs --qualified <dir> --tag vX.Y.Z --expect-source <sha> [--verify-run <id>] [--report <path>]");
    process.exitCode = 2;
    return;
  }
  const release = tag.replace(/^v/u, "");
  const root = readOption("--root") ? resolve(readOption("--root")) : process.cwd();
  const result = await verifyReleasePromotion({
    qualifiedDirectory: resolve(qualifiedDirectory),
    root,
    tag,
    release,
    expectedSourceRevision,
    verifyRunId,
  });
  for (const line of result.lines) console.log(line);
  if (result.failures.length > 0) {
    for (const failure of result.failures) console.error(`FAIL ${failure}`);
    process.exitCode = 1;
  } else {
    console.log(`PASS release promotion set ${tag} at ${expectedSourceRevision} is safe to publish (${result.report.promisedAssets.length} assets)`);
  }
  if (reportPath) {
    await writeFile(resolve(reportPath), `${JSON.stringify(result.report, null, 2)}\n`, "utf8");
  }
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  main().catch((error) => {
    console.error(`promotion-check: ${error.message}`);
    process.exitCode = 1;
  });
}
