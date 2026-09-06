import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import Ajv from "ajv";
import { verifyVersions } from "../verify_versions.mjs";
import { validatePinnedUses } from "../verify_workflow_pins.mjs";
import { validateWorkflowGates } from "../verify_workflow_gates.mjs";
import { inspectIcoSizes, validateIconConfiguration } from "../verify_icons.mjs";
import { validateBrandingEntries } from "../verify_branding.mjs";
import { validateQualification } from "../verify_qualification.mjs";
import { validateResearchAnchor } from "../verify_research_anchor.mjs";
import { verifyCandidateInventory } from "../verify_candidate_inventory.mjs";

async function versionFixture(overrides = {}) {
  const root = await mkdtemp(join(tmpdir(), "localmotive-version-gate-"));
  await mkdir(join(root, "src-tauri"), { recursive: true });
  await mkdir(join(root, "src"), { recursive: true });
  const version = overrides.version ?? "0.4.1";
  await writeFile(join(root, "package.json"), JSON.stringify({ version }));
  await writeFile(join(root, "package-lock.json"), JSON.stringify({ version, packages: { "": { version: overrides.lockRoot ?? version } } }));
  await writeFile(join(root, "src-tauri", "tauri.conf.json"), JSON.stringify({ version: overrides.tauri ?? version }));
  await writeFile(join(root, "src-tauri", "Cargo.toml"), `[package]\nname = "localmotive"\nversion = "${overrides.cargo ?? version}"\n`);
  await writeFile(join(root, "src-tauri", "Cargo.lock"), `[[package]]\nname = "localmotive"\nversion = "${overrides.cargoLock ?? version}"\n`);
  const aboutSource = overrides.aboutFallback === null
    ? "setAbout(null);\n"
    : `setAbout({ name: "Localmotive", version: "${overrides.aboutFallback ?? version}" });\n`;
  await writeFile(join(root, "src", "App.tsx"), aboutSource);
  return root;
}

test("version gate checks every release version field", async () => {
  const root = await versionFixture();
  const result = await verifyVersions(root, "0.4.1");
  assert.equal(result.ok, true);
  assert.equal(result.fields.length, 6);
});

test("version gate rejects a mismatched Cargo.lock package entry", async () => {
  const root = await versionFixture({ cargoLock: "0.4.0" });
  const result = await verifyVersions(root, "0.4.1");
  assert.equal(result.ok, false);
  assert.match(result.failures.join("\n"), /Cargo\.lock/);
});

test("version gate accepts no fabricated browser About version", async () => {
  const root = await versionFixture({ aboutFallback: null });
  const result = await verifyVersions(root, "0.4.1");
  assert.equal(result.ok, true);
  assert.equal(result.fields.length, 6);
});

test("workflow pin gate rejects mutable action references", () => {
  const result = validatePinnedUses([
    { file: "ci.yml", job: "test", step: 0, uses: "actions/checkout@v4" },
  ], []);
  assert.equal(result.ok, false);
  assert.match(result.failures.join("\n"), /full 40-character commit SHA/);
});

test("workflow pin gate rejects an annotated tag object as the executable pin", () => {
  const tagObjectSha = "49a0bdc70d2e1b713ca9e2869b211fcce03d3c1c";
  const result = validatePinnedUses([
    { file: "ci.yml", job: "test", step: 0, uses: `swatinem/rust-cache@${tagObjectSha}` },
  ], [{
    action: "swatinem/rust-cache",
    reviewedRef: "v2",
    sha: tagObjectSha,
    tagObjectSha,
    sourceUrl: `https://github.com/swatinem/rust-cache/tree/${tagObjectSha}`,
  }]);
  assert.equal(result.ok, false);
  assert.match(result.failures.join("\n"), /annotated tag object/);
});

test("workflow pin gate accepts the peeled commit and records its tag object", () => {
  const commitSha = "6323deb102c322ba6fcbdcafc7e3dddab59af2b6";
  const tagObjectSha = "49a0bdc70d2e1b713ca9e2869b211fcce03d3c1c";
  const result = validatePinnedUses([
    { file: "ci.yml", job: "test", step: 0, uses: `swatinem/rust-cache@${commitSha}` },
  ], [{
    action: "swatinem/rust-cache",
    reviewedRef: "v2",
    sha: commitSha,
    tagObjectSha,
    sourceUrl: `https://github.com/swatinem/rust-cache/commit/${commitSha}`,
  }]);
  assert.equal(result.ok, true, result.failures.join("\n"));
});

test("workflow gate rejects package jobs without every material dependency", () => {
  const workflows = {
    "ci.yml": {
      jobs: {
        test: { steps: [{ run: "npm test" }] },
        package: { needs: [], steps: [{ run: "npm run tauri build" }] },
      },
    },
  };
  const policy = {
    workflows: {
      "ci.yml": {
        gates: { test: ["npm test"] },
        packageJobs: { package: ["test"] },
        publicationJobs: {},
      },
    },
  };
  const result = validateWorkflowGates(workflows, policy);
  assert.equal(result.ok, false);
  assert.match(result.failures.join("\n"), /does not depend on material gate test/);
});

test("workflow gate rejects package jobs that omit packaged verification", () => {
  const workflows = {
    "release.yml": {
      jobs: {
        package: { steps: [{ run: "npm run tauri build" }] },
      },
    },
  };
  const policy = {
    workflows: {
      "release.yml": {
        gates: {},
        packageJobs: {},
        publicationJobs: {},
        requiredCommands: {
          package: ["node scripts/verify_041.mjs 10041 $portable"],
        },
      },
    },
  };
  const result = validateWorkflowGates(workflows, policy);
  assert.equal(result.ok, false);
  assert.match(result.failures.join("\n"), /must execute.*verify_041/);
});

test("catalog workflow cannot expose signing keys or push to main", async () => {
  const workflow = await readFile(join(process.cwd(), ".github", "workflows", "catalog.yml"), "utf8");
  assert.doesNotMatch(workflow, /CATALOG_SIGNING_KEY_PEM/);
  assert.doesNotMatch(workflow, /contents:\s*write/);
  assert.doesNotMatch(workflow, /git push/);
  assert.doesNotMatch(workflow, /ref:\s*main/);
});

test("release evidence uses the checked-out tag revision", async () => {
  const workflow = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  assert.match(workflow, /LOCALMOTIVE_SOURCE_REVISION=\$\(git rev-parse HEAD\)/);
  assert.doesNotMatch(workflow, /LOCALMOTIVE_SOURCE_REVISION:\s*\$\{\{ github\.sha \}\}/);
});

test("release workflow reads published assets back", async () => {
  const workflow = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  assert.match(workflow, /gh release view/);
  assert.match(workflow, /gh release download/);
  assert.match(workflow, /sha256sum -c/);
});

test("a controlled nonzero native command remains nonzero", async () => {
  const { spawnSync } = await import("node:child_process");
  const child = spawnSync(process.execPath, ["-e", "process.exit(23)"], { stdio: "ignore" });
  assert.equal(child.status, 23);
});

test("icon gate requires every Windows icon surface and ICO layer", () => {
  const ico = Buffer.alloc(6 + (6 * 16));
  ico.writeUInt16LE(0, 0);
  ico.writeUInt16LE(1, 2);
  ico.writeUInt16LE(6, 4);
  [16, 24, 32, 48, 64, 256].forEach((size, index) => {
    ico[6 + (index * 16)] = size === 256 ? 0 : size;
    ico[6 + (index * 16) + 1] = size === 256 ? 0 : size;
  });
  assert.deepEqual(inspectIcoSizes(ico), [16, 24, 32, 48, 64, 256]);

  const result = validateIconConfiguration({
    config: {
      bundle: {
        publisher: "Localmotive contributors",
        icon: ["icons/icon.ico"],
        windows: {
          nsis: {
            installerIcon: "icons/icon.ico",
            uninstallerIcon: "icons/icon.ico",
          },
        },
      },
    },
    svg: '<rect fill="#1d2122"/><rect stroke="#d5d1c5"/><rect fill="#9edc72"/><path d="M0 0h1v1Z"/><path d="M1 0h1v1Z"/>',
    icoSizes: inspectIcoSizes(ico),
    expectedPublisher: "Localmotive contributors",
  });
  assert.equal(result.ok, true, result.failures.join("\n"));
});

test("Windows installers carry an offline WebView2 prerequisite", async () => {
  const config = JSON.parse(await readFile(join(process.cwd(), "src-tauri", "tauri.conf.json"), "utf8"));
  assert.equal(config.bundle?.windows?.webviewInstallMode?.type, "offlineInstaller");
});

test("runtime catalog exposes an accessible refresh action in every terminal state", async () => {
  const source = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  assert.match(source, /aria-label="Refresh approved runtime catalog"/);
  assert.match(source, /className="runtime-role runtime-scope"/);
  assert.match(source, /entry\.blockingJobs\.map/);
  assert.match(source, /DIRECT RUNTIME · L2/);
  assert.doesNotMatch(source, /sampleAsset/);
  assert.match(source, /Browser preview cannot retrieve the approved runtime catalog/);
});

test("production frontend never fabricates model or runtime inspection results", async () => {
  const source = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  assert.doesNotMatch(source, /previewModels/);
  assert.doesNotMatch(source, /Example-8B-Instruct-Q4_K_M/);
  assert.doesNotMatch(source, /build: "10679"/);
  assert.doesNotMatch(source, /capabilities represented from the inspected local runtime/);
  assert.doesNotMatch(source, /setCommand\(`\\"\$\{profile\.runtime\}/);
  assert.match(source, /setHardware\(\{\s*architecture: "unknown"/);
  assert.match(source, /invoke<AboutInfo>\("about_info"\)\.then\(setAbout\)\.catch\(\(\) => setAbout\(null\)\)/);
});

test("frontend renders backend-owned managed runtime trust", async () => {
  const source = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  assert.doesNotMatch(source, /runtimePath\.startsWith\(runtimeRoot\)/);
  assert.match(source, /runtimeIdentity\?\.managedVerified/);
});

test("runtime inspection commits one latest atomic result", async () => {
  const source = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  assert.match(source, /const runtimeInspectSeq = useRef\(0\)/);
  assert.match(source, /Promise\.all\(\[/);
  assert.match(source, /keepLatestRequest\(sequence, runtimeInspectSeq\.current\)/);
});

test("runtime setup refresh preserves the selected adapter recommendation", async () => {
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  const backend = await readFile(join(process.cwd(), "src-tauri", "src", "lib.rs"), "utf8");
  assert.match(app, /load_runtime_setup",\s*\{\s*adapterId: selectedRuntimeAdapterId \|\| null/);
  assert.match(backend, /async fn load_runtime_setup\([\s\S]*adapter_id: Option<String>/);
});

test("runtime catalog text actions meet the 44 pixel target minimum", async () => {
  const css = await readFile(join(process.cwd(), "src", "App.css"), "utf8");
  const rule = css.match(/\.runtime-source\s*\{([^}]*)\}/)?.[1] ?? "";
  assert.match(rule, /min-height:\s*44px/);
  assert.match(rule, /padding:/);
});

test("frontend renders backend catalog facts without owning support classification", async () => {
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  const model = await readFile(join(process.cwd(), "src", "model.ts"), "utf8");
  assert.doesNotMatch(app, /supportStatusForOption/);
  assert.doesNotMatch(model, /supportStatusForOption/);
  assert.doesNotMatch(model, /runtimeRevision:\s*"b10816"/);
  assert.match(app, /runtimeCatalogState\.catalog\.tag/);
});

test("hardware panel does not present heuristic hardware advice as the catalog recommendation", async () => {
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  assert.doesNotMatch(app, /hardware\?\.recommendation/);
  assert.match(app, /runtimeCatalog\?\.recommendationReason/);
});

test("runtime setup ignores stale responses and owns a separate loading state", async () => {
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  assert.match(app, /\[runtimeCatalogLoading, setRuntimeCatalogLoading\]/);
  assert.match(app, /async function loadRuntimeSetup\(\)[\s\S]*?const sequence = \+\+runtimeCatalogSeq\.current/);
  assert.match(app, /if \(!keepLatestRequest\(sequence, runtimeCatalogSeq\.current\)\) return;/);
  assert.match(app, /disabled=\{runtimeCatalogLoading\}/);
});

test("runtime setup invalidates pending responses during unmount", async () => {
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  assert.match(
    app,
    /loadRuntimeSetup\(\);[\s\S]*?return \(\) => \{\s*runtimeCatalogSeq\.current \+= 1;/,
  );
});

test("runtime install cancellation is available before progress and ignores stale events", async () => {
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  assert.match(app, /const installingRef = useRef\(""\)/);
  assert.match(app, /event\.payload\.installKey === installingRef\.current/);
  assert.match(app, /\{installing && \(/);
  assert.doesNotMatch(app, /\{installing && runtimeInstallProgress && \(/);
});

test("managed health progress is correlated to the active install key", async () => {
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  const model = await readFile(join(process.cwd(), "src", "model.ts"), "utf8");
  const backend = await readFile(join(process.cwd(), "src-tauri", "src", "lib.rs"), "utf8");
  assert.match(model, /export type HealthModelProgress = \{\s*installKey: string;/);
  assert.match(app, /event\.payload\.installKey === healthRunningRef\.current/);
  assert.match(backend, /HealthModelProgress \{\s*install_key:/);
});

test("runtime and health cancellation expose a pending UI state", async () => {
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  assert.match(app, /\[runtimeInstallCancelling, setRuntimeInstallCancelling\]/);
  assert.match(app, /\[healthCancelling, setHealthCancelling\]/);
  assert.match(app, /runtimeInstallCancelling \? "Stopping…"/);
  assert.match(app, /healthCancelling \? "Stopping…"/);
});

test("adapter catalog IPC preserves structured retry metadata", async () => {
  const backend = await readFile(join(process.cwd(), "src-tauri", "src", "lib.rs"), "utf8");
  assert.match(
    backend,
    /async fn fetch_runtime_catalog\([\s\S]*?\) -> Result<runtime::RuntimeCatalog, runtime::RuntimeCatalogError>/,
  );
});

test("branding gate rejects legacy names outside documented history and migration code", () => {
  const legacy = ["GGUF", "Pilot"].join(" ");
  const result = validateBrandingEntries([
    { path: "src/App.tsx", content: `<h1>${legacy}</h1>` },
    { path: "CHANGELOG.md", content: `Renamed ${legacy}.` },
    { path: "src-tauri/src/runtime.rs", content: `runtime_data_dir("${legacy}")` },
  ]);
  assert.equal(result.ok, false);
  assert.deepEqual(result.failures, ["src/App.tsx:legacy-product-name"]);
});

test("qualification gate permits disclosed gaps but never promotes them to support", () => {
  const approvals = {
    decisions: [{
      id: "missing-p0-hardware-evidence",
      status: "RISK_ACCEPTED_WITH_DISCLOSURE",
    }],
  };
  const matrix = {
    rows: [
      { id: "hardware-a", status: "UNKNOWN", supportClaim: false, releaseNotesDisclosed: true, attestation: null },
      { id: "hardware-b", status: "DIRECT_L2", supportClaim: true, releaseNotesDisclosed: true, attestation: null },
    ],
  };
  const result = validateQualification({
    matrix,
    approvals,
    compatibilityRecords: [],
    attestations: new Map(),
    expectedIds: ["hardware-a", "hardware-b"],
  });
  assert.equal(result.ok, false);
  assert.deepEqual(result.failures, ["hardware-b:support-requires-L4_PASS"]);
});

test("qualification gate preserves each frozen P0 hardware and backend scope", () => {
  const result = validateQualification({
    matrix: {
      rows: [{
        id: "hardware-a",
        hardwareClass: "Different GPU",
        os: "Windows 11 x64",
        architecture: "x64",
        backend: "cpu",
        status: "UNKNOWN",
        supportClaim: false,
        releaseNotesDisclosed: true,
        attestation: null,
      }],
    },
    approvals: { decisions: [{ id: "missing-p0-hardware-evidence", status: "RISK_ACCEPTED_WITH_DISCLOSURE" }] },
    compatibilityRecords: [],
    attestations: new Map(),
    expectedIds: ["hardware-a"],
    expectedRows: new Map([["hardware-a", {
      hardwareClass: "Frozen GPU",
      os: "Windows 11 x64",
      architecture: "x64",
      backend: "cuda",
    }]]),
  });
  assert.equal(result.ok, false);
  assert.deepEqual(result.failures, ["hardware-a:frozen-scope-mismatch"]);
});

test("qualification gate rejects a failed attestation on an L4 row", () => {
  const row = {
    id: "hardware-a",
    hardwareClass: "Frozen GPU",
    os: "Windows 11 x64",
    architecture: "x64",
    backend: "cuda",
    status: "L4_PASS",
    supportClaim: true,
    releaseNotesDisclosed: false,
    attestation: "failed-attestation",
  };
  const result = validateQualification({
    matrix: { rows: [row] },
    approvals: { decisions: [] },
    compatibilityRecords: [],
    attestations: new Map([["failed-attestation", {
      id: "failed-attestation",
      p0Id: "hardware-a",
      result: "FAIL",
      evidenceLevel: "L4_PRODUCT",
      qualificationKey: {},
      expiryIdentity: "unchanged",
    }]]),
    expectedIds: ["hardware-a"],
    expectedRows: new Map([["hardware-a", {
      hardwareClass: "Frozen GPU",
      os: "Windows 11 x64",
      architecture: "x64",
      backend: "cuda",
    }]]),
  });
  assert.equal(result.ok, false);
  assert.ok(result.failures.includes("hardware-a:L4-attestation-not-pass"));
});

test("hardware waiver does not waive clean-account lifecycle evidence", () => {
  const result = validateQualification({
    matrix: { rows: [{
      id: "win-x64-clean-account",
      hardwareClass: "Clean Windows x64 account without developer toolkits",
      os: "Windows 11 x64",
      architecture: "x64",
      backend: "all shipped",
      status: "UNKNOWN",
      supportClaim: false,
      releaseNotesDisclosed: true,
      attestation: null,
    }] },
    approvals: { decisions: [{ id: "missing-p0-hardware-evidence", status: "RISK_ACCEPTED_WITH_DISCLOSURE" }] },
    compatibilityRecords: [],
    attestations: new Map(),
    expectedIds: ["win-x64-clean-account"],
    expectedRows: new Map([["win-x64-clean-account", {
      hardwareClass: "Clean Windows x64 account without developer toolkits",
      os: "Windows 11 x64",
      architecture: "x64",
      backend: "all shipped",
    }]]),
  });
  assert.equal(result.ok, false);
  assert.ok(result.failures.includes("win-x64-clean-account:lifecycle-not-waived"));
});

test("research anchor gate rejects changed tracked manifest bytes", () => {
  const anchor = {
    schemaVersion: 1,
    manifestPath: "research/0.4.1/research-freeze.json",
    trackedManifestPath: "release-evidence/0.4.1/research-freeze-manifest.json",
    manifestSha256: "0".repeat(64),
    baseCommit: "a".repeat(40),
    scope: ["research/0.4", "research/0.4.1"],
    excluded: [],
    tree: { algorithm: "sha256-canonical-file-manifest-v1", sha256: "0".repeat(64), files: 0, bytes: 0 },
    productScopeSha256: "0".repeat(64),
  };
  const result = validateResearchAnchor({
    anchor,
    manifestBytes: Buffer.from("{}\n"),
    approvals: { decisions: [{ id: "phase-0a-rebaseline", status: "APPROVED_WITH_CONDITIONS" }] },
  });
  assert.equal(result.ok, false);
  assert.ok(result.failures.includes("anchor:manifest-sha256"));
  const ledgerResult = validateResearchAnchor({
    anchor,
    manifestBytes: Buffer.from("{}\n"),
    approvals: { decisions: [] },
    verification: { freeze: { manifestSha256: "f".repeat(64) } },
  });
  assert.ok(ledgerResult.failures.includes("verification:manifest-sha256"));
});

test("research anchor gate rejects a missing verification ledger", () => {
  const result = validateResearchAnchor({
    anchor: {},
    manifestBytes: Buffer.from("{}\n"),
    approvals: { decisions: [] },
    verification: null,
  });
  assert.ok(result.failures.includes("verification:ledger"));
});

test("research anchor gate rejects fabricated verification counts and commands", () => {
  const result = validateResearchAnchor({
    anchor: {},
    manifestBytes: Buffer.from("{}\n"),
    approvals: { decisions: [] },
    verification: {
      schemaVersion: 1,
      release: "0.4.1",
      commands: ["false"],
      unitTests: { status: "PASS", passed: 1, failed: 0, skipped: 0 },
      researchVerifier: { status: "PASS", corePassed: 1, passed: 1, failed: 0, unknown: 0 },
      freeze: {},
      cleanCheckoutCoverage: {},
      independentReview: { status: "PENDING" },
    },
  });
  assert.ok(result.failures.includes("verification:commands"));
  assert.ok(result.failures.includes("verification:unit-tests"));
  assert.ok(result.failures.includes("verification:research-verifier"));
});

test("research anchor gate rejects an unbound independent-review pass", () => {
  const result = validateResearchAnchor({
    anchor: {},
    manifestBytes: Buffer.from("{}\n"),
    approvals: { decisions: [] },
    verification: {
      independentReview: {
        status: "PASS",
      },
    },
  });
  assert.ok(result.failures.includes("verification:independent-review-report"));
});

test("research anchor gate rejects an untracked independent-review report", () => {
  const report = Buffer.from("Delegation: deleg_1234abcd\nVerdict: PASS\n", "utf8");
  const result = validateResearchAnchor({
    anchor: {},
    manifestBytes: Buffer.from("{}\n"),
    approvals: { decisions: [] },
    verification: {
      independentReview: {
        status: "PASS",
        verdict: "PASS",
        delegationId: "deleg_1234abcd",
        reportPath: "release-evidence/0.4.1/phase0a-independent-review.txt",
        reportSha256: createHash("sha256").update(report).digest("hex"),
      },
    },
    independentReviewBytes: report,
    trackedPaths: new Set(),
  });
  assert.ok(result.failures.includes("verification:independent-review-not-tracked"));
});

test("research anchor gate rejects an independent review for another research tree", () => {
  const report = Buffer.from([
    "Delegation: deleg_1234abcd",
    "Verdict: PASS",
    `Manifest SHA-256: ${"0".repeat(64)}`,
    `Tree SHA-256: ${"0".repeat(64)}`,
    "",
  ].join("\n"), "utf8");
  const result = validateResearchAnchor({
    anchor: {
      manifestSha256: "a".repeat(64),
      tree: { sha256: "b".repeat(64) },
    },
    manifestBytes: Buffer.from("{}\n"),
    approvals: { decisions: [] },
    verification: {
      independentReview: {
        status: "PASS",
        verdict: "PASS",
        delegationId: "deleg_1234abcd",
        reportPath: "release-evidence/0.4.1/phase0a-independent-review.txt",
        reportSha256: createHash("sha256").update(report).digest("hex"),
        manifestSha256: "0".repeat(64),
        treeSha256: "0".repeat(64),
      },
    },
    independentReviewBytes: report,
    trackedPaths: new Set(["release-evidence/0.4.1/phase0a-independent-review.txt"]),
  });
  assert.ok(result.failures.includes("verification:independent-review-tree"));
});

test("packaged catalog harness drives React state without a production test hook", async () => {
  const source = await readFile(join(process.cwd(), "scripts", "verify_041.mjs"), "utf8");
  assert.match(source, /__reactFiber\$/);
  assert.match(source, /findIndex/);
  assert.match(source, /hardwareIndex \+ 2/);
  assert.match(source, /queue\.dispatch/);
  assert.doesNotMatch(source, /hooks\[5\]/);
  assert.doesNotMatch(source, /__LM_VERIFY_ORIGINAL_INVOKE/);
  assert.doesNotMatch(source, /Page\.addScriptToEvaluateOnNewDocument/);
});

test("packaged verifier requires successful health, cancellation, and restart", async () => {
  const source = await readFile(join(process.cwd(), "scripts", "verify_041.mjs"), "utf8");
  assert.match(source, /ipc\.runtime-recommendation/);
  assert.match(source, /ipc\.reject-unknown-adapter/);
  assert.match(source, /ui\.blocked-backend/);
  assert.match(source, /health\.seven-stage-pass/);
  assert.match(source, /health\.cancellation/);
  assert.match(source, /health\.restart/);
});

test("packaged verification schema rejects false-green overall status", async () => {
  const schema = JSON.parse(await readFile(join(process.cwd(), "release-evidence", "packaged-verification.schema.json"), "utf8"));
  const validate = new Ajv({ strict: true }).compile(schema);
  const result = {
    schema_version: "1.0.0",
    verifier: "scripts/verify_041.mjs",
    source_revision: "a".repeat(40),
    source_dirty: true,
    artifact: { name: "candidate.exe", size_bytes: 1, sha256: "0".repeat(64) },
    host_class: { platform: "win32", release: "10.0", architecture: "x64", cpu_model: "CPU", logical_cpus: 1, memory_bytes: 1, adapters: [] },
    started_at: "2026-09-06T00:00:00Z",
    finished_at: "2026-09-06T00:00:01Z",
    checks: [{ id: "candidate.clean-source", criterion: "clean", status: "FAIL", evidence: {}, reason: "dirty" }],
    overall_status: "PASS",
  };
  assert.equal(validate(result), false);
});

test("packaged verification schema rejects false-red overall status", async () => {
  const schema = JSON.parse(await readFile(join(process.cwd(), "release-evidence", "packaged-verification.schema.json"), "utf8"));
  const validate = new Ajv({ strict: true }).compile(schema);
  const result = {
    schema_version: "1.0.0",
    verifier: "scripts/verify_041.mjs",
    source_revision: "a".repeat(40),
    source_dirty: false,
    artifact: { name: "candidate.exe", size_bytes: 1, sha256: "0".repeat(64) },
    host_class: { platform: "win32", release: "10.0", architecture: "x64", cpu_model: "CPU", logical_cpus: 1, memory_bytes: 1, adapters: [] },
    started_at: "2026-09-06T00:00:00Z",
    finished_at: "2026-09-06T00:00:01Z",
    checks: [{ id: "candidate.clean-source", criterion: "clean", status: "PASS", evidence: {}, reason: "" }],
    overall_status: "FAIL",
  };
  assert.equal(validate(result), false);
});

test("candidate inventory rejects a checksum that does not bind the staged bytes", async () => {
  const root = await mkdtemp(join(tmpdir(), "localmotive-inventory-gate-"));
  const names = [
    "Localmotive_0.4.1_x64-setup.exe",
    "Localmotive_0.4.1_x64-portable.exe",
    "Localmotive_0.4.1_x64.msi",
  ];
  for (const name of names) await writeFile(join(root, name), name);
  await writeFile(
    join(root, "SHA256SUMS-0.4.1.txt"),
    names.map((name) => `${"0".repeat(64)}  ${name}`).join("\n") + "\n",
  );
  await assert.rejects(
    verifyCandidateInventory({
      artifactDirectory: root,
      releaseVersion: "0.4.1",
      outputPath: join(root, "inventory.json"),
      sourceRevision: "a".repeat(40),
    }),
    /does not match SHA256SUMS/,
  );
});

test("signed build configuration requires certificate-store identity and timestamp verification", async () => {
  const config = JSON.parse(await readFile(
    join(process.cwd(), "src-tauri", "tauri.signing.conf.json"),
    "utf8",
  ));
  const script = await readFile(join(process.cwd(), "scripts", "sign-windows.ps1"), "utf8");
  assert.match(config.bundle.windows.signCommand, /sign-windows\.ps1/);
  assert.match(script, /LOCALMOTIVE_SIGNING_THUMBPRINT/);
  assert.match(script, /LOCALMOTIVE_TIMESTAMP_URL/);
  assert.match(script, /verify \/pa \/all \/v/);
  assert.match(script, /TimeStamperCertificate/);
  assert.doesNotMatch(script, /Export-PfxCertificate|ConvertTo-SecureString/);
});

test("hardware qualify workflow pins every remote action and orders sandbox after host proof", async () => {
  const workflow = await readFile(join(process.cwd(), ".github", "workflows", "hardware-qualify.yml"), "utf8");
  assert.doesNotMatch(workflow, /uses:\s*actions\/checkout@v/);
  assert.doesNotMatch(workflow, /uses:\s*actions\/upload-artifact@v/);
  assert.match(workflow, /actions\/checkout@11d5960a326750d5838078e36cf38b85af677262/);
  assert.match(workflow, /actions\/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02/);
  assert.match(workflow, /needs:\s*hardware-qualify/);
  assert.match(workflow, /supportClaimPolicy = "Do not mark SUPPORTED \/ L4_PASS without a successful packaged run/);
});

test("hardware qualify workflow cannot claim L4 support from host match alone", async () => {
  const workflow = await readFile(join(process.cwd(), ".github", "workflows", "hardware-qualify.yml"), "utf8");
  assert.match(workflow, /Packaged L4 checks still required/);
  assert.match(workflow, /schema = "localmotive\.attestation\.v0"/);
  assert.match(workflow, /status = "HOST_MATCH"/);
  assert.doesNotMatch(workflow, /status = "L4_PASS"/);
  assert.doesNotMatch(workflow, /id = ".*"; status = "SUPPORTED"/);
});

test("workflow gate policy covers the hardware qualify night path", async () => {
  const policy = JSON.parse(await readFile(join(process.cwd(), ".github", "workflow-gates.json"), "utf8"));
  assert.ok(policy.workflows["hardware-qualify.yml"]);
  assert.deepEqual(Object.keys(policy.workflows["hardware-qualify.yml"].gates).sort(), ["clean-account-lifecycle", "hardware-qualify"]);
  assert.deepEqual(policy.workflows["hardware-qualify.yml"].packageJobs["clean-account-lifecycle"], ["hardware-qualify"]);
});
