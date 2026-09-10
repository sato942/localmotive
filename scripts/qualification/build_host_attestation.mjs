// Derive host-attestation matrix rows from DETECTED hardware (audit GH-10).
//
// Every row status is justified by the recorded observation: MATCH when the
// observation satisfies the stated expectation, NO_MATCH when it definitely
// does not (the intended host-proof job must fail), UNKNOWN when the
// observation is missing or ambiguous. Host presence never establishes
// packaged L4 product qualification; the policy string says so and the rows
// carry only observation-level statuses.
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
import { writeFile } from "node:fs/promises";

export const EXPECTED = {
  cpuPattern: /9950X3D|Ryzen 9 9950X/i,
  gpuPattern: /5090/i,
  cpuLabel: "AMD Zen 5 (9950X3D class)",
  gpuLabel: "NVIDIA Blackwell (RTX 5090 class)",
};

function classify(observation, pattern, expectsGpu) {
  const value = (observation ?? "").trim();
  if (!value) return "UNKNOWN";
  // Ambiguous multi-GPU inventories cannot prove the intended adapter.
  if (expectsGpu && value.includes(";") && !pattern.test(value)) return "UNKNOWN";
  return pattern.test(value) ? "MATCH" : "NO_MATCH";
}

export function deriveHostRows({
  cpu,
  gpu,
  expected = EXPECTED,
} = {}) {
  const cpuStatus = classify(cpu, expected.cpuPattern, false);
  const gpuStatus = classify(gpu, expected.gpuPattern, true);
  const rows = [
    {
      id: "amd-zen5-cpu",
      status: statusLabel(cpuStatus),
      observation: cpu ?? "",
      note: `${expected.cpuLabel} expected. Packaged L4 checks still required.`,
    },
    {
      id: "nvidia-blackwell-cuda",
      status: statusLabel(gpuStatus),
      observation: gpu ?? "",
      note: `${expected.gpuLabel} expected. Packaged CUDA L4 checks still required.`,
    },
    {
      id: "nvidia-blackwell-vulkan",
      status: statusLabel(gpuStatus),
      observation: gpu ?? "",
      note: `${expected.gpuLabel} expected. Packaged Vulkan L4 checks still required.`,
    },
  ];
  const definiteMismatch = cpuStatus === "NO_MATCH" || gpuStatus === "NO_MATCH";
  const unknown = cpuStatus === "UNKNOWN" || gpuStatus === "UNKNOWN";
  return {
    rows,
    verdict: definiteMismatch ? "NO_MATCH" : unknown ? "UNKNOWN" : "MATCH",
  };
}

function statusLabel(status) {
  // Explicit three-state vocabulary; never a boolean.
  return status === "MATCH" ? "HOST_MATCH" : status === "NO_MATCH" ? "HOST_NO_MATCH" : "HOST_UNKNOWN";
}

export async function writeAttestation({
  cpu,
  gpu,
  os,
  driver,
  version,
  tag,
  sourceRevision,
  runUrl,
  runId,
  outPath,
}) {
  const { rows, verdict } = deriveHostRows({ cpu, gpu });
  const doc = {
    schema: "localmotive.attestation.v0",
    version,
    tag,
    recordedAtUtc: new Date().toISOString(),
    sourceRevision: sourceRevision ?? null,
    host: { cpu: cpu ?? "", gpu: gpu ?? "", driver: driver ?? "", os: os ?? "" },
    hostVerdict: verdict,
    matrixRows: rows,
    supportClaimPolicy:
      "Do not mark SUPPORTED / L4_PASS without a successful packaged run recorded in this attestation. Host presence proves only that the expected hardware was observed.",
    githubRun: runUrl ?? null,
    githubRunId: runId ?? null,
  };
  await writeFile(outPath, `${JSON.stringify(doc, null, 2)}\n`, "utf8");
  return doc;
}

function argValue(args, name) {
  const index = args.indexOf(`--${name}`);
  return index === -1 ? undefined : args[index + 1];
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const outPath = argValue(args, "out");
  if (!outPath) {
    console.error("usage: build_host_attestation.mjs --out <path> [--cpu ...] [--gpu ...] [--os ...] [--driver ...] [--version ...] [--tag ...] [--source-revision ...] [--run-url ...] [--run-id ...]");
    process.exit(2);
  }
  const doc = await writeAttestation({
    cpu: argValue(args, "cpu"),
    gpu: argValue(args, "gpu"),
    os: argValue(args, "os"),
    driver: argValue(args, "driver"),
    version: argValue(args, "version"),
    tag: argValue(args, "tag"),
    sourceRevision: argValue(args, "source-revision"),
    runUrl: argValue(args, "run-url"),
    runId: argValue(args, "run-id"),
    outPath,
  });
  console.log(`host attestation written: verdict ${doc.hostVerdict}`);
  if (doc.hostVerdict === "NO_MATCH") {
    console.error("Detected hardware does not match the intended host: this host-proof job fails.");
    process.exit(1);
  }
}
