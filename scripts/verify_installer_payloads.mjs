// Installer payload identity: extract the executable embedded in the staged
// NSIS and MSI installers, record the actual payload digests, and explain the
// difference from the portable executable.
//
// Investigation result (2026-09-13, freeze-8 candidate): the three binaries
// are the SAME build bytes apart from one 3-byte bundle-type marker that
// Tauri's bundlers patch in place - the portable carries "UNK", the NSIS
// payload "NSS", the MSI payload "MSI" (the tail of the runtime's
// TYPE_VAR_<bundle> identity string). Every other byte is identical, which is
// why the recorded digests differ while the executables are, byte for byte,
// the same program. The earlier note in the lifecycle harness attributed the
// difference to non-reproducible bundler builds; this script replaces that
// assumption with a measured, reproducible comparison.
//
// Usage:
//   node scripts/verify_installer_payloads.mjs <artifactDir> <inventoryPath> <evidencePath>
// Requires 7-Zip (LOCALMOTIVE_7ZIP overrides the default install path).
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SEVEN_ZIP = process.env.LOCALMOTIVE_7ZIP ?? "C:\\Program Files\\7-Zip\\7z.exe";

export function extractInstallerPayload(installerPath, { sevenZip = SEVEN_ZIP, sizeHint } = {}) {
  const workDir = mkdtempSync(join(tmpdir(), "localmotive-payload-"));
  try {
    execFileSync(sevenZip, ["x", "-y", `-o${workDir}`, installerPath], { windowsHide: true });
    const candidates = [];
    const walk = (directory) => {
      for (const entry of readdirSync(directory, { withFileTypes: true })) {
        const path = join(directory, entry.name);
        if (entry.isDirectory()) walk(path);
        else if (statSync(path).size === sizeHint) candidates.push(path);
      }
    };
    walk(workDir);
    if (candidates.length !== 1) {
      throw new Error(
        `${installerPath}: expected exactly one embedded payload of ${sizeHint} bytes, found ${candidates.length}`,
      );
    }
    return readFileSync(candidates[0]);
  } finally {
    rmSync(workDir, { recursive: true, force: true });
  }
}

/// The bundler patches one 3-byte marker in the same executable. Verify that
/// the payload and the portable executable are identical everywhere else.
export function compareBundlePayloads(portable, payload, label) {
  if (portable.length !== payload.length) {
    return {
      ok: false,
      reason: `${label}: payload size ${payload.length} differs from the portable size ${portable.length}`,
    };
  }
  const diffPositions = [];
  for (let index = 0; index < portable.length; index += 1) {
    if (portable[index] !== payload[index]) diffPositions.push(index);
  }
  if (diffPositions.length !== 3) {
    return {
      ok: false,
      reason: `${label}: expected exactly the 3-byte bundle-type marker to differ, found ${diffPositions.length} differing bytes`,
      diffPositions,
    };
  }
  const marker = (buffer) => buffer.subarray(diffPositions[0], diffPositions[0] + 3).toString("latin1");
  const portableMarker = marker(portable);
  const payloadMarker = marker(payload);
  const expected = { "NSS": "nsis", "MSI": "msi" }[payloadMarker];
  if (portableMarker !== "UNK" || !expected || expected !== label) {
    return {
      ok: false,
      reason: `${label}: unexpected marker bytes - portable '${portableMarker}', payload '${payloadMarker}'`,
      diffPositions,
    };
  }
  return { ok: true, diffPositions, portableMarker, payloadMarker };
}

export function verifyInstallerPayloads({ artifactDir, inventoryPath, evidencePath, sevenZip = SEVEN_ZIP }) {
  const inventory = JSON.parse(readFileSync(inventoryPath, "utf8"));
  const portableEntry = inventory.artifacts.find((artifact) => artifact.name.endsWith("-portable.exe"));
  const setupEntry = inventory.artifacts.find((artifact) => artifact.name.endsWith("-setup.exe"));
  const msiEntry = inventory.artifacts.find((artifact) => artifact.name.endsWith(".msi"));
  if (!portableEntry || !setupEntry || !msiEntry) {
    throw new Error("The candidate inventory does not carry the expected artifact set");
  }
  const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
  const portablePath = join(artifactDir, portableEntry.name);
  const portable = readFileSync(portablePath);
  if (portable.length !== portableEntry.sizeBytes || digest(portable) !== portableEntry.sha256) {
    throw new Error("The portable executable does not match the approved inventory");
  }
  const nsisPayload = extractInstallerPayload(join(artifactDir, setupEntry.name), {
    sevenZip,
    sizeHint: portable.length,
  });
  const msiPayload = extractInstallerPayload(join(artifactDir, msiEntry.name), {
    sevenZip,
    sizeHint: portable.length,
  });
  const nsisComparison = compareBundlePayloads(portable, nsisPayload, "nsis");
  const msiComparison = compareBundlePayloads(portable, msiPayload, "msi");
  const evidence = {
    schema: "localmotive.installer-payload-identity.v1",
    generatedAt: new Date().toISOString(),
    sourceRevision: inventory.sourceRevision,
    inventorySha256: digest(readFileSync(inventoryPath)),
    release: inventory.release,
    portable: { name: portableEntry.name, sizeBytes: portableEntry.sizeBytes, sha256: portableEntry.sha256 },
    nsisPayload: { sizeBytes: nsisPayload.length, sha256: digest(nsisPayload) },
    msiPayload: { sizeBytes: msiPayload.length, sha256: digest(msiPayload) },
    marker: {
      portable: nsisComparison.portableMarker ?? null,
      nsis: nsisComparison.payloadMarker ?? null,
      msi: msiComparison.payloadMarker ?? null,
      differingBytes: nsisComparison.diffPositions?.[0] ?? null,
      note:
        "Tauri's NSIS/MSI bundlers patch one 3-byte bundle-type marker ('UNK' -> 'NSS' / 'MSI') into their " +
        "embedded copy of the executable. Every other byte is identical to the portable executable, so the " +
        "recorded digests differ while the installed programs are the same build.",
    },
    status: nsisComparison.ok && msiComparison.ok ? "PASS" : "FAIL",
    detail: [nsisComparison, msiComparison].map((comparison) => comparison.reason ?? "ok").join("; "),
  };
  if (evidencePath) {
    writeFileSync(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`, "utf8");
  }
  return evidence;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [artifactDir, inventoryPath, evidencePath] = process.argv.slice(2);
  if (!artifactDir || !inventoryPath) {
    console.error(
      "usage: node scripts/verify_installer_payloads.mjs <artifactDir> <inventoryPath> [evidencePath]",
    );
    process.exit(2);
  }
  const evidence = verifyInstallerPayloads({
    artifactDir: resolve(artifactDir),
    inventoryPath: resolve(inventoryPath),
    evidencePath: evidencePath ? resolve(evidencePath) : null,
  });
  console.log(JSON.stringify(evidence, null, 2));
  if (evidence.status !== "PASS") {
    console.error(`INSTALLER PAYLOAD VERIFY FAILED: ${evidence.detail}`);
    process.exitCode = 1;
  }
}
