import { readdir, readFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import Ajv2020 from "ajv/dist/2020.js";

export const P0_IDS = [
  "win-x64-amd-zen5-cpu",
  "win-x64-amd-zen2-cpu",
  "win-x64-amd-zen4-cpu",
  "win-x64-intel-sse42-cpu",
  "win-x64-intel-avx2-cpu",
  "win-x64-nvidia-pascal-cuda12",
  "win-x64-nvidia-turing-cuda",
  "win-x64-nvidia-ampere-cuda",
  "win-x64-nvidia-ada-cuda",
  "win-x64-nvidia-blackwell-cuda",
  "win-x64-nvidia-blackwell-vulkan",
  "win-x64-amd-rdna2-vulkan",
  "win-x64-amd-rdna3-rocm",
  "win-x64-amd-rdna4-rocm",
  "win-x64-amd-rdna35-apu",
  "win-x64-intel-xelp",
  "win-x64-intel-arc-alchemist",
  "win-x64-intel-arc-battlemage",
  "win-x64-clean-account",
];

export const P0_ROWS = new Map([
  ["win-x64-amd-zen5-cpu", { hardwareClass: "AMD Ryzen 9 9950X3D", os: "Windows 11 x64", architecture: "x64", backend: "CPU" }],
  ["win-x64-amd-zen2-cpu", { hardwareClass: "AMD Zen 2 CPU", os: "Windows 11 x64", architecture: "x64", backend: "CPU" }],
  ["win-x64-amd-zen4-cpu", { hardwareClass: "AMD Zen 4 CPU", os: "Windows 11 x64", architecture: "x64", backend: "CPU" }],
  ["win-x64-intel-sse42-cpu", { hardwareClass: "Intel pre-AVX2 CPU", os: "Windows 10 or 11 x64", architecture: "x64", backend: "CPU" }],
  ["win-x64-intel-avx2-cpu", { hardwareClass: "Intel AVX2 CPU", os: "Windows 10 or 11 x64", architecture: "x64", backend: "CPU" }],
  ["win-x64-nvidia-pascal-cuda12", { hardwareClass: "NVIDIA Pascal GPU", os: "Windows 11 x64", architecture: "x64", backend: "CUDA 12.4" }],
  ["win-x64-nvidia-turing-cuda", { hardwareClass: "NVIDIA Turing GPU", os: "Windows 11 x64", architecture: "x64", backend: "CUDA 12.4 and CUDA 13.3" }],
  ["win-x64-nvidia-ampere-cuda", { hardwareClass: "NVIDIA Ampere GPU", os: "Windows 11 x64", architecture: "x64", backend: "CUDA 12.4 and CUDA 13.3" }],
  ["win-x64-nvidia-ada-cuda", { hardwareClass: "NVIDIA Ada GPU", os: "Windows 11 x64", architecture: "x64", backend: "CUDA 12.4 and CUDA 13.3" }],
  ["win-x64-nvidia-blackwell-cuda", { hardwareClass: "NVIDIA RTX 5090", os: "Windows 11 x64", architecture: "x64", backend: "CUDA 13.3" }],
  ["win-x64-nvidia-blackwell-vulkan", { hardwareClass: "NVIDIA RTX 5090", os: "Windows 11 x64", architecture: "x64", backend: "Vulkan" }],
  ["win-x64-amd-rdna2-vulkan", { hardwareClass: "AMD RDNA 2 GPU", os: "Windows 11 x64", architecture: "x64", backend: "Vulkan" }],
  ["win-x64-amd-rdna3-rocm", { hardwareClass: "AMD RDNA 3 GPU", os: "Windows 11 x64", architecture: "x64", backend: "ROCm" }],
  ["win-x64-amd-rdna4-rocm", { hardwareClass: "AMD RDNA 4 GPU", os: "Windows 11 x64", architecture: "x64", backend: "ROCm" }],
  ["win-x64-amd-rdna35-apu", { hardwareClass: "AMD Ryzen AI APU", os: "Windows 11 x64", architecture: "x64", backend: "ROCm and Vulkan" }],
  ["win-x64-intel-xelp", { hardwareClass: "Intel Iris Xe", os: "Windows 11 x64", architecture: "x64", backend: "Vulkan and OpenVINO" }],
  ["win-x64-intel-arc-alchemist", { hardwareClass: "Intel Arc A-series", os: "Windows 11 x64", architecture: "x64", backend: "SYCL and Vulkan" }],
  ["win-x64-intel-arc-battlemage", { hardwareClass: "Intel Arc B-series", os: "Windows 11 x64", architecture: "x64", backend: "SYCL and Vulkan" }],
  ["win-x64-clean-account", { hardwareClass: "no developer toolkits", os: "Windows 11 x64", architecture: "x64", backend: "all shipped" }],
]);

const ALLOWED_STATUS = new Set([
  "L4_PASS",
  "L4_FAIL",
  "DIRECT_L2",
  "UPSTREAM_ONLY",
  "UPSTREAM_FAILURE",
  "UNKNOWN",
]);

function exactJson(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

export function validateQualification({
  matrix,
  approvals,
  compatibilityRecords,
  attestations,
  expectedIds = P0_IDS,
  expectedRows = P0_ROWS,
}) {
  const failures = [];
  const rows = Array.isArray(matrix?.rows) ? matrix.rows : [];
  const expected = new Set(expectedIds);
  const ids = rows.map((row) => row?.id);
  const unique = new Set(ids);
  if (ids.length !== unique.size) failures.push("matrix:duplicate-row-id");
  for (const id of expected) {
    if (!unique.has(id)) failures.push(`${id}:missing-row`);
  }
  for (const id of unique) {
    if (!expected.has(id)) failures.push(`${id}:unexpected-row`);
  }

  const rowById = new Map(rows.map((row) => [row.id, row]));
  for (const [id, expectedRow] of expectedRows) {
    const row = rowById.get(id);
    if (!row) continue;
    const observed = {
      hardwareClass: row.hardwareClass,
      os: row.os,
      architecture: row.architecture,
      backend: row.backend,
    };
    if (!exactJson(observed, expectedRow)) failures.push(`${id}:frozen-scope-mismatch`);
  }

  const riskAccepted = approvals?.decisions?.some((decision) =>
    decision.id === "missing-p0-hardware-evidence"
    && decision.status === "RISK_ACCEPTED_WITH_DISCLOSURE");
  for (const row of rows) {
    if (!ALLOWED_STATUS.has(row.status)) {
      failures.push(`${row.id}:invalid-status`);
      continue;
    }
    if (row.supportClaim && row.status !== "L4_PASS") {
      failures.push(`${row.id}:support-requires-L4_PASS`);
    }
    if (row.status !== "L4_PASS") {
      if (row.id === "win-x64-clean-account") {
        failures.push(`${row.id}:lifecycle-not-waived`);
      } else if (!riskAccepted) {
        failures.push(`${row.id}:missing-P0-evidence-not-approved`);
      }
      if (row.releaseNotesDisclosed !== true) failures.push(`${row.id}:gap-not-disclosed`);
    }
    if (row.status === "L4_PASS") {
      if (typeof row.attestation !== "string" || !row.attestation) {
        failures.push(`${row.id}:L4-attestation-missing`);
      } else if (!attestations.has(row.attestation)) {
        failures.push(`${row.id}:L4-attestation-not-found`);
      } else {
        const attestation = attestations.get(row.attestation);
        if (
          attestation.p0Id !== row.id
          || attestation.result !== "PASS"
          || attestation.evidenceLevel !== "L4_PRODUCT"
        ) {
          failures.push(`${row.id}:L4-attestation-not-pass`);
        }
        const records = (compatibilityRecords ?? []).filter(
          (record) => record.attestationId === row.attestation,
        );
        if (records.length !== 1) failures.push(`${row.id}:L4-compatibility-record-count`);
      }
    }
  }

  for (const record of compatibilityRecords ?? []) {
    const attestation = attestations.get(record.attestationId);
    if (!attestation) {
      failures.push(`${record.attestationId}:compatibility-attestation-not-found`);
      continue;
    }
    const row = rowById.get(attestation.p0Id);
    if (!row || row.status !== "L4_PASS" || row.supportClaim !== true) {
      failures.push(`${record.attestationId}:compatibility-record-without-L4-support-row`);
    }
    if (attestation.result !== "PASS" || attestation.evidenceLevel !== "L4_PRODUCT") {
      failures.push(`${record.attestationId}:compatibility-record-without-L4-pass`);
    }
    if (!exactJson(record.key, attestation.qualificationKey)) {
      failures.push(`${record.attestationId}:qualification-key-mismatch`);
    }
    if (record.expiryIdentity !== attestation.expiryIdentity) {
      failures.push(`${record.attestationId}:expiry-identity-mismatch`);
    }
  }

  return { ok: failures.length === 0, failures };
}

async function readJson(path) {
  return JSON.parse(await readFile(path, "utf8"));
}

export async function verifyQualification(root = process.cwd()) {
  const absolute = resolve(root);
  const matrix = await readJson(join(absolute, "release-evidence", "0.4.1", "qualification-matrix.json"));
  const approvals = await readJson(join(absolute, "release-evidence", "0.4.1", "approvals.json"));
  const manifest = await readJson(join(absolute, "src-tauri", "approved_runtimes.json"));
  const matrixSchema = await readJson(join(absolute, "release-evidence", "qualification-matrix.schema.json"));
  const attestationSchema = await readJson(join(absolute, "release-evidence", "qualification-attestation.schema.json"));
  const ajv = new Ajv2020({ allErrors: true, strict: true });
  const validateMatrix = ajv.compile(matrixSchema);
  const validateAttestation = ajv.compile(attestationSchema);
  const failures = [];
  if (!validateMatrix(matrix)) {
    failures.push(...(validateMatrix.errors ?? []).map((error) => `matrix-schema:${error.instancePath || "/"}:${error.message}`));
  }

  const attestations = new Map();
  const attestationDirectory = join(absolute, "release-evidence", "0.4.1", "attestations");
  try {
    for (const entry of await readdir(attestationDirectory, { withFileTypes: true })) {
      if (!entry.isFile() || !entry.name.endsWith(".json")) continue;
      const value = await readJson(join(attestationDirectory, entry.name));
      if (!validateAttestation(value)) {
        failures.push(...(validateAttestation.errors ?? []).map((error) => `${entry.name}:${error.instancePath || "/"}:${error.message}`));
      }
      if (attestations.has(value.id)) failures.push(`${entry.name}:duplicate-attestation-id`);
      attestations.set(value.id, value);
    }
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }

  const result = validateQualification({
    matrix,
    approvals,
    compatibilityRecords: manifest.compatibilityRecords,
    attestations,
  });
  failures.push(...result.failures);
  return { ok: failures.length === 0, failures, rows: matrix.rows.length, attestations: attestations.size };
}

const invokedPath = process.argv[1] ? resolve(process.argv[1]) : null;
if (invokedPath === fileURLToPath(import.meta.url)) {
  const result = await verifyQualification(process.cwd());
  if (!result.ok) {
    for (const failure of result.failures) console.error(`FAIL ${failure}`);
    process.exitCode = 1;
  } else {
    console.log(`PASS qualification policy: ${result.rows} disclosed P0 rows, ${result.attestations} L4 attestations`);
  }
}
