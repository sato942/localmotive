import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, mkdir, readFile, readdir, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import Ajv from "ajv";
import { verifyVersions } from "../verify_versions.mjs";
import { validatePinnedUses } from "../verify_workflow_pins.mjs";
import { loadWorkflows, validateWorkflowGates } from "../verify_workflow_gates.mjs";
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

/// Concatenated frontend source: every screen module FIRST, then App.tsx.
/// The S-27.I2 extraction moves JSX into src/screens/*.tsx; guards that read
/// App.tsx alone silently stop matching. Splits find real component bodies
/// before any test-text mention because the screens come first.
async function frontendSources() {
  const screens = [
    "AboutScreen.tsx",
    "BenchmarkScreen.tsx",
    "CatalogScreen.tsx",
    "DashboardScreen.tsx",
    "InventoryScreen.tsx",
    "ProfileEmptyScreen.tsx",
    "ProfileScreen.tsx",
    "RuntimeScreen.tsx",
    "TuneScreen.tsx",
  ];
  const parts = [];
  for (const name of screens) {
    try {
      parts.push(await readFile(join(process.cwd(), "src", "screens", name), "utf8"));
    } catch {
      // A screen module that does not exist yet contributes nothing.
    }
  }
  parts.push(await readFile(join(process.cwd(), "src", "App.tsx"), "utf8"));
  return parts.join("\n");
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
    { file: "ci.yml", job: "test", step: 0, uses: "actions/checkout@v5.1.0" },
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

test("catalog v2 schema, builder, and signed publish wiring stay consistent", async () => {
  const catalog = JSON.parse(await readFile(join(process.cwd(), "catalog", "catalog.json"), "utf8"));
  const providers = JSON.parse(await readFile(join(process.cwd(), "catalog", "providers.json"), "utf8"));
  const backend = await readFile(join(process.cwd(), "src-tauri", "src", "catalog.rs"), "utf8");
  const builder = await readFile(join(process.cwd(), "scripts", "build_catalog.mjs"), "utf8");
  const validator = await readFile(join(process.cwd(), "scripts", "validate_catalog.mjs"), "utf8");
  const workflow = await readFile(join(process.cwd(), ".github", "workflows", "catalog.yml"), "utf8");
  const gates = JSON.parse(await readFile(join(process.cwd(), ".github", "workflow-gates.json"), "utf8"));
  // The checked-in catalog is schema 2 after the signed v2 publish lands.
  // This test pins the full contract so no half-migrated publish can ship.
  // Seen live: a v2 preview with slash filenames and 61 cross-publisher
  // collisions failed the client filename guard, so the builder now excludes
  // subdirectories and dedupes by filename before signing.
  assert.equal(catalog.schemaVersion, 2);
  assert.match(backend, /pub const SUPPORTED_SCHEMA: u32 = 2;/);
  assert.match(backend, /MIN_SUPPORTED_SCHEMA/);
  assert.match(validator, /schemaVersion must be 2/);
  assert.match(validator, /providers\.allowlist/);
  assert.match(builder, /schemaVersion: 2/);
  assert.match(builder, /providers\.json/);
  assert.match(builder, /seenFilenames/);
  // The builder reads the allowlist from repo config, not a hardcoded list in
  // the app binary. Adding an author is a catalog publish, not an app release.
  assert.ok(Array.isArray(providers.allowlist) && providers.allowlist.length > 0);
  assert.doesNotMatch(backend, /providers\.json/);
  assert.match(backend, /DEFAULT_CATALOG_URL/);
  // The signed publish path keeps the private key in the sign job only. The
  // build job rebuilds from the allowlist without secrets; the sign job signs
  // the exact candidate and verifies the detached signature.
  assert.match(workflow, /CATALOG_SIGNING_KEY_PEM/);
  assert.match(workflow, /sign_catalog_candidate/);
  assert.match(workflow, /validate_catalog\.mjs catalog\/catalog\.json --no-signature/);
  assert.match(workflow, /validate_catalog\.mjs catalog\/catalog\.json\n/);
  assert.ok(gates.workflows["catalog.yml"].gates.build);
  assert.ok(gates.workflows["catalog.yml"].gates.sign);
});

test("release evidence uses one resolved immutable revision", async () => {
  const workflow = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  // The producer records the resolver's SHA only after proving its checkout
  // matches it (audit GH-02 I1); workflow-generated context SHAs are never
  // accepted as the evidence identity.
  assert.match(workflow, /test "\$\(git rev-parse HEAD\)" = "\$RESOLVED_SHA"/);
  assert.match(workflow, /LOCALMOTIVE_SOURCE_REVISION=\$RESOLVED_SHA/);
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

test("Windows installers use the bootstrapper WebView2 mode until offline bundling is fixed", async () => {
  const config = JSON.parse(await readFile(join(process.cwd(), "src-tauri", "tauri.conf.json"), "utf8"));
  assert.equal(config.bundle?.windows?.webviewInstallMode?.type, "downloadBootstrapper");
});

test("README matches the shipped product and the unsigned Latest policy", async () => {
  const readme = await readFile(join(process.cwd(), "README.md"), "utf8");
  // Stale 0.3.0 download section hid the real ship state. Seen live: the
  // Download section still named 0.3.0 files while Latest served 0.4.1.
  assert.doesNotMatch(readme, /Release 0\.3\.0 provides/);
  assert.doesNotMatch(readme, /Localmotive_0\.3\.0_x64/);
  assert.doesNotMatch(readme, /SHA256SUMS-0\.3\.0/);
  assert.doesNotMatch(readme, /The bundled 0\.3\.0 catalog/);
  assert.doesNotMatch(readme, /\(unsigned prerelease\)/);
  assert.doesNotMatch(readme, /testing-only or pre-release/);
  // Current product: unsigned full release on Latest, SmartScreen honesty,
  // checksum proof, catalog v2 story, and network needs.
  assert.match(readme, /Latest/);
  assert.match(readme, /\(unsigned\)/);
  assert.match(readme, /SmartScreen/);
  assert.match(readme, /SHA256SUMS/);
  assert.match(readme, /schema 2|schemaVersion 2/i);
  assert.match(readme, /providers\.json/);
  assert.match(readme, /Hardware fit/i);
  assert.match(readme, /raw\.githubusercontent\.com/);
});

test("launch profiles reject oversize input at the Rust boundary", async () => {
  const core = await readFile(join(process.cwd(), "src-tauri", "src", "core.rs"), "utf8");
  // Profile strings flow from the UI into build_args. UI limits are hints
  // only; the Rust boundary owns truth. Seen live: no length check existed on
  // any profile string, so a hostile frontend could submit megabyte paths.
  assert.match(core, /MAX_PROFILE_TEXT_LEN/);
  assert.match(core, /MAX_TENSOR_SPLIT_ENTRIES/);
  assert.match(core, /validate_profile_input_bounds/);
});

test("catalog commands reject oversize input at the Rust boundary", async () => {
  const backend = await readFile(join(process.cwd(), "src-tauri", "src", "catalog.rs"), "utf8");
  // The catalog command family lives in catalog_service.rs after the S-27 I1
  // extraction; the boundary checks moved with it.
  const lib = await readFile(join(process.cwd(), "src-tauri", "src", "catalog_service.rs"), "utf8");
  // A compromised webview can send any JSON. UI limits are hints only; the
  // Rust boundary owns truth. Seen live: filter_catalog took Vec + query with
  // no length checks, so a hostile frontend could submit megabytes of text.
  assert.match(backend, /MAX_QUERY_TEXT_LEN/);
  assert.match(backend, /MAX_FILTER_VALUE_LEN/);
  assert.match(backend, /MAX_FILTER_MODELS/);
  assert.match(backend, /validate_catalog_query/);
  assert.match(backend, /validate_facet_models/);
  assert.match(backend, /validate_budget_inputs/);
  assert.match(lib, /validate_catalog_query\(&query, models\.len\(\)\)/);
  assert.match(lib, /validate_facet_models\(models\.len\(\)\)/);
  assert.match(lib, /validate_budget_inputs\(dedicated_bytes\.len\(\), shared_bytes\.len\(\)\)/);
});

test("catalog refresh honors cooldown, lock, and last-success display", async () => {
  const backend = await readFile(join(process.cwd(), "src-tauri", "src", "catalog.rs"), "utf8");
  // The fetch command moved to catalog_service.rs (S-27 I1); catalog.rs still
  // owns the cooldown/guard machinery it delegates to.
  const lib = await readFile(join(process.cwd(), "src-tauri", "src", "catalog_service.rs"), "utf8");
  const model = await readFile(join(process.cwd(), "src", "model.ts"), "utf8");
  const app = await frontendSources();
  // Seen live: fetch_catalog had no cooldown, so every Refresh click hit the
  // network, and two clicks started two fetches. The backend now owns a 1560
  // min cooldown, an in-flight guard, and a last-success stamp; the UI shows
  // last success plus the remaining wait.
  assert.match(backend, /CATALOG_REFRESH_COOLDOWN_MINUTES/);
  assert.match(backend, /refresh_cooldown_remaining_minutes/);
  assert.match(backend, /CatalogRefreshGuard/);
  assert.match(backend, /read_refresh_stamp/);
  assert.match(lib, /CatalogRefreshGuard::try_acquire/);
  assert.match(lib, /refresh_cooldown_remaining_minutes/);
  assert.match(lib, /CATALOG_REFRESH_COOLDOWN_MINUTES/);
  assert.match(model, /lastSuccessSecs/);
  assert.match(model, /cooldownRemainingMinutes/);
  assert.match(app, /LAST SUCCESS/);
  assert.match(app, /COOLDOWN/);
});

test("model catalog exposes rich filters with hardware auto-fit defaults", async () => {
  const app = await frontendSources();
  const model = await readFile(join(process.cwd(), "src", "model.ts"), "utf8");
  const backend = await readFile(join(process.cwd(), "src-tauri", "src", "lib.rs"), "utf8");
  // Rich adjustable filters come from the signed v2 artifact via backend
  // facets. The UI must not hardcode the author list: adding an author is a
  // catalog publish, not an app release.
  assert.match(app, /catalog_rich_facets/);
  assert.match(app, /Any author/);
  assert.match(app, /Any licence/);
  assert.match(app, /Any pipeline/);
  assert.match(app, /Any architecture/);
  assert.doesNotMatch(app, /lmstudio-community.*huihui-ai.*DavidAU/s);
  // Hardware auto-fit is on by default, explains its budget source, and can
  // be disabled or widened. Rust owns the rule; the UI only sends inputs.
  assert.match(app, /catalogFitEnabled/);
  assert.match(app, /Hardware fit/);
  assert.match(app, /Fit budget/);
  assert.match(app, /fit_per_mille/);
  assert.match(app, /budget_bytes/);
  assert.match(model, /hardwareFitBudget/);
  assert.match(model, /modelHiddenByFitRule/);
  assert.match(model, /DEFAULT_FIT_PER_MILLE/);
  assert.match(backend, /catalog_rich_facets/);
  assert.match(backend, /catalog_fit_budget/);
});

test("runtime catalog exposes an accessible refresh action in every terminal state", async () => {
  const source = await frontendSources();
  assert.match(source, /aria-label="Refresh approved runtime catalog"/);
  assert.match(source, /className="runtime-role runtime-scope"/);
  assert.match(source, /entry\.blockingJobs\.map/);
  assert.match(source, /DIRECT RUNTIME · L2/);
  assert.doesNotMatch(source, /sampleAsset/);
  assert.match(source, /Browser preview cannot retrieve the approved runtime catalog/);
});

test("production frontend never fabricates model or runtime inspection results", async () => {
  const source = await frontendSources();
  assert.doesNotMatch(source, /previewModels/);
  assert.doesNotMatch(source, /Example-8B-Instruct-Q4_K_M/);
  assert.doesNotMatch(source, /build: "10679"/);
  assert.doesNotMatch(source, /capabilities represented from the inspected local runtime/);
  assert.doesNotMatch(source, /setCommand\(`\\"\$\{profile\.runtime\}/);
  assert.match(source, /setHardware\(\{\s*architecture: "unknown"/);
  assert.match(source, /invoke<AboutInfo>\("about_info"\)\.then\(setAbout\)\.catch\(\(\) => setAbout\(null\)\)/);
});

test("frontend renders backend-owned managed runtime trust", async () => {
  const source = await frontendSources();
  assert.doesNotMatch(source, /runtimePath\.startsWith\(runtimeRoot\)/);
  assert.match(source, /runtimeIdentity\?\.managedVerified/);
});

test("runtime inspection commits one latest atomic result", async () => {
  const source = await frontendSources();
  assert.match(source, /const runtimeInspectSeq = useRef\(0\)/);
  assert.match(source, /Promise\.all\(\[/);
  assert.match(source, /keepLatestRequest\(sequence, runtimeInspectSeq\.current\)/);
});

test("runtime setup refresh preserves the selected adapter recommendation", async () => {
  const app = await frontendSources();
  // The runtime command family moved to runtime_service.rs (S-27 I1/I2).
  const backend = await readFile(join(process.cwd(), "src-tauri", "src", "runtime_service.rs"), "utf8");
  assert.match(app, /load_runtime_setup",\s*\{\s*adapterId: selectedRuntimeAdapterId \|\| null/);
  assert.match(backend, /async fn load_runtime_setup\([\s\S]*adapter_id: Option<String>/);
});

test("runtime catalog text actions meet the 44 pixel target minimum", async () => {
  const css = await readFile(join(process.cwd(), "src", "App.css"), "utf8");
  const rule = css.match(/\.runtime-source\s*\{([^}]*)\}/)?.[1] ?? "";
  assert.match(rule, /min-height:\s*44px/);
  assert.match(rule, /padding:/);
});

test("a rejected catalog invocation removes the spinner", async () => {
  const app = await frontendSources();
  const errorAt = app.indexOf('runtime-catalog-message error');
  assert.ok(errorAt > 0, "the catalog error block is missing");
  const errorBlock = app.slice(errorAt, errorAt + 2000);
  assert.doesNotMatch(errorBlock, /className="spin"/);
  assert.doesNotMatch(errorBlock, /className="runtime-loading"/);
  // The error branch is exclusive with the loading branch: the state
  // machine returns exactly one kind, so error can never co-render loading.
  const model = await readFile(join(process.cwd(), "src", "model.ts"), "utf8");
  assert.match(model, /if \(input\.loading\) return \{ kind: "loading" \};/);
  assert.match(model, /if \(input\.error\) return \{ kind: "error", error: input\.error \};/);
});

test("error state never renders a spinner", async () => {
  const app = await frontendSources();
  const errorAt = app.indexOf('runtime-catalog-message error');
  assert.ok(errorAt > 0, "the catalog error block is missing");
  const errorBlock = app.slice(errorAt, errorAt + 2000);
  assert.doesNotMatch(errorBlock, /className="spin"/);
  assert.doesNotMatch(errorBlock, /className="runtime-loading"/);
});

test("loading state has an accessible status label", async () => {
  const app = await frontendSources();
  assert.match(app, /className="runtime-loading" role="status" aria-label="Loading approved runtime catalog"/);
});

test("blocked CUDA shows job server-cuda and its evidence URL", async () => {
  const app = await frontendSources();
  assert.match(app, /entry\.blockingJobs\.map/);
  assert.match(app, /View \{jobName\} evidence/);
});

test("L2 rows render DIRECT RUNTIME · L2", async () => {
  const app = await frontendSources();
  assert.match(app, /DIRECT RUNTIME · L2/);
  assert.match(app, /L2 EVIDENCE CEILING/);
});

test("no row renders SUPPORTED without the required evidence", async () => {
  const app = await frontendSources();
  const model = await readFile(join(process.cwd(), "src", "model.ts"), "utf8");
  assert.doesNotMatch(app, /SUPPORTED/);
  assert.doesNotMatch(model, /SUPPORTED/);
});

test("public 0.4.1 documentation states the L2 evidence ceiling", async () => {
  const changelog = await readFile(join(process.cwd(), "CHANGELOG.md"), "utf8");
  const contract = await readFile(join(process.cwd(), "docs", "RUNTIME_MANAGER.md"), "utf8");
  assert.match(changelog, /L2 EVIDENCE CEILING/);
  assert.match(contract, /L2 ceiling/);
});

test("public links resolve without a gitignored local path", async () => {
  const app = await frontendSources();
  assert.doesNotMatch(app, /research\//);
  assert.doesNotMatch(app, /LOCALAPPDATA/);
  assert.doesNotMatch(app, /AppData/);
  assert.match(app, /https:\/\/github\.com\/ggml-org\/llama\.cpp\/releases/);
});

test("MSI manufacturer equals the approved publisher value", async () => {
  const { execFileSync } = await import("node:child_process");
  const config = JSON.parse(await readFile(join(process.cwd(), "src-tauri", "tauri.conf.json"), "utf8"));
  const cargo = await readFile(join(process.cwd(), "src-tauri", "Cargo.toml"), "utf8");
  const publisher = config.bundle?.publisher;
  assert.equal(publisher, "Localmotive contributors");
  assert.match(cargo, /authors = \["Localmotive contributors"\]/);
  // MSI Manufacturer falls back to bundle.publisher when no explicit
  // windows.wix fragment overrides it; fail loudly if a fragment appears
  // without carrying the approved value.
  let wixFragment = "";
  try {
    wixFragment = execFileSync("git", ["grep", "-l", "Manufacturer", "--", "src-tauri"], { encoding: "utf8" }).trim();
  } catch {
    wixFragment = "";
  }
  assert.equal(wixFragment, "", `unexpected Manufacturer override: ${wixFragment}`);
});

test("honestly unsigned full release verifies checksums and inventory instead of Authenticode", async () => {
  const release = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  assert.doesNotMatch(release, /signtool/);
  assert.doesNotMatch(release, /Get-AuthenticodeSignature/);
  assert.doesNotMatch(release, /TimeStamperCertificate/);
  assert.doesNotMatch(release, /passed the Authenticode verification gate/);
  assert.match(release, /unsigned/);
  assert.match(release, /SmartScreen/);
  assert.match(release, /verify_candidate_inventory/);
  assert.match(release, /sha256sum -c/);
  // The ship tip stays visible as Latest, so the publish path must not mark
  // the release as a GitHub Pre-release. Seen live: v0.4.1 shipped with
  // prerelease:true and releases/latest still pointed at v0.4.0.
  assert.match(release, /prerelease:\s*false/);
  assert.doesNotMatch(release, /\(unsigned prerelease\)/);
});

test("honestly unsigned full release reads back Latest with the unsigned asset set", async () => {
  const release = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  assert.match(release, /prerelease:\s*false/);
  assert.match(release, /\.isPrerelease/);
  assert.match(release, /honestly unsigned/);
  assert.match(release, /\(unsigned\)/);
});

test("correct the three L2 claims in the new 0.4.1 changelog section", async () => {
  const changelog = await readFile(join(process.cwd(), "CHANGELOG.md"), "utf8");
  assert.match(changelog, /The three 0\.4\.0 rows marked `Supported` had L2 direct-runtime evidence only/);
  assert.match(changelog, /Those rows did not establish Localmotive product support/);
  assert.match(changelog, /See `release-evidence\/0\.4\.1\/v0\.4\.0-corrective-note\.md` for the proposed public correction/);
});

test("the 0.4.0 corrective note stays review-gated, not silently published", async () => {
  const note = await readFile(join(process.cwd(), "release-evidence", "0.4.1", "v0.4.0-corrective-note.md"), "utf8");
  assert.match(note, /This draft does not modify the published release/);
  assert.match(note, /Review this correction before editing the public 0\.4\.0 release/);
});

test("small icon layers remain readable at native resolution", async () => {
  const { readFile: readBinary } = await import("node:fs/promises");
  const ico = await readBinary(join(process.cwd(), "src-tauri", "icons", "icon.ico"));
  const sizes = await inspectIcoSizes(ico);
  assert.ok(sizes.includes(16), "the 16px layer is missing");
  assert.ok(sizes.includes(32), "the 32px layer is missing");
  // PNG-compressed ICO layers decode to full RGBA pixels: a 16px layer must
  // decode to 16*16*4 bytes, otherwise the small layer is a stub.
  const { inflateSync } = await import("node:zlib");
  const count = ico.readUInt16LE(4);
  for (let index = 0; index < count; index += 1) {
    const offset = 6 + (index * 16);
    const width = ico[offset] || 256;
    if (width > 32) continue;
    const bytes = ico.readUInt32LE(offset + 8);
    const dataOffset = ico.readUInt32LE(offset + 12);
    const chunk = ico.subarray(dataOffset, dataOffset + bytes);
    assert.equal(chunk[0], 0x89, `the ${width}px layer is not a PNG layer`);
    assert.equal(chunk[1], 0x50, `the ${width}px layer is not a PNG layer`);
    const ihdrLength = chunk.readUInt32BE(8);
    const ihdrType = chunk.subarray(12, 16).toString("ascii");
    assert.equal(ihdrType, "IHDR");
    const pngWidth = chunk.readUInt32BE(16);
    const pngHeight = chunk.readUInt32BE(20);
    assert.equal(pngWidth, width, `the ${width}px layer has wrong PNG width`);
    assert.equal(pngHeight, width, `the ${width}px layer has wrong PNG height`);
    assert.ok(ihdrLength >= 13, `the ${width}px layer has a truncated IHDR`);
    void inflateSync;
  }
});

test("the packaged NSIS installer and uninstaller use the LM icon", async () => {
  const config = JSON.parse(await readFile(join(process.cwd(), "src-tauri", "tauri.conf.json"), "utf8"));
  assert.equal(config.bundle?.windows?.nsis?.installerIcon, "icons/icon.ico");
  assert.equal(config.bundle?.windows?.nsis?.uninstallerIcon, "icons/icon.ico");
});

test("frontend renders backend catalog facts without owning support classification", async () => {
  const app = await frontendSources();
  const model = await readFile(join(process.cwd(), "src", "model.ts"), "utf8");
  assert.doesNotMatch(app, /supportStatusForOption/);
  assert.doesNotMatch(model, /supportStatusForOption/);
  assert.doesNotMatch(model, /runtimeRevision:\s*"b10816"/);
  assert.match(app, /runtimeCatalogState\.catalog\.tag/);
});

test("hardware panel does not present heuristic hardware advice as the catalog recommendation", async () => {
  const app = await frontendSources();
  assert.doesNotMatch(app, /hardware\?\.recommendation/);
  assert.match(app, /runtimeCatalog\?\.recommendationReason/);
});

test("runtime setup ignores stale responses and owns a separate loading state", async () => {
  const app = await frontendSources();
  assert.match(app, /\[runtimeCatalogLoading, setRuntimeCatalogLoading\]/);
  assert.match(app, /async function loadRuntimeSetup\(\)[\s\S]*?const sequence = \+\+runtimeCatalogSeq\.current/);
  assert.match(app, /if \(!keepLatestRequest\(sequence, runtimeCatalogSeq\.current\)\) return;/);
  assert.match(app, /disabled=\{(?:props\.)?runtimeCatalogLoading\}/);
});

test("runtime setup invalidates pending responses during unmount", async () => {
  const app = await frontendSources();
  assert.match(
    app,
    /loadRuntimeSetup\(\);[\s\S]*?return \(\) => \{\s*runtimeCatalogSeq\.current \+= 1;/,
  );
});

test("runtime install cancellation is available before progress and ignores stale events", async () => {
  const app = await frontendSources();
  assert.match(app, /const installingRef = useRef\(""\)/);
  assert.match(app, /event\.payload\.installKey === installingRef\.current/);
  assert.match(app, /\{(?:props\.)?installing && \(/);
  assert.doesNotMatch(app, /\{installing && runtimeInstallProgress && \(/);
});

test("managed health progress is correlated to the active install key", async () => {
  const app = await frontendSources();
  const model = await readFile(join(process.cwd(), "src", "model.ts"), "utf8");
  const backend = await readFile(join(process.cwd(), "src-tauri", "src", "runtime_service.rs"), "utf8");
  assert.match(model, /export type HealthModelProgress = \{\s*installKey: string;/);
  assert.match(app, /event\.payload\.installKey === healthRunningRef\.current/);
  assert.match(backend, /HealthModelProgress \{\s*install_key:/);
});

test("runtime and health cancellation expose a pending UI state", async () => {
  const app = await frontendSources();
  assert.match(app, /\[runtimeInstallCancelling, setRuntimeInstallCancelling\]/);
  assert.match(app, /\[healthCancelling, setHealthCancelling\]/);
  assert.match(app, /runtimeInstallCancelling \? "Stopping…"/);
  assert.match(app, /healthCancelling \? "Stopping…"/);
});

test("adapter catalog IPC preserves structured retry metadata", async () => {
  const backend = await readFile(join(process.cwd(), "src-tauri", "src", "runtime_service.rs"), "utf8");
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

test("packaged verifier contains no private React-state injection (QD-03)", async () => {
  const source = await readFile(join(process.cwd(), "scripts", "verify_041.mjs"), "utf8");
  // The fiber walk and dispatch extraction are gone: presentation states
  // live in the jsdom component tests (src/App.catalog.test.tsx), and the
  // verifier speaks only through the real IPC boundary and the DOM.
  assert.doesNotMatch(source, /__reactFiber\$/);
  assert.doesNotMatch(source, /memoizedState/);
  assert.doesNotMatch(source, /queue\.dispatch/);
  assert.doesNotMatch(source, /__LM_VERIFY_SET_CATALOG/);
  assert.doesNotMatch(source, /__LM_VERIFY_ORIGINAL_INVOKE/);
  assert.doesNotMatch(source, /Page\.addScriptToEvaluateOnNewDocument/);
  // Genuine replacements stay: the real catalog fetch check and the
  // relational card-invariant check.
  assert.match(source, /ipc\.runtime-catalog-fetch/);
  assert.match(source, /ui\.runtime-cards/);
});

test("packaged cancellation check observes progress, records completion or a bounded diagnostic (QD-03)", async () => {
  const source = await readFile(join(process.cwd(), "scripts", "verify_041.mjs"), "utf8");
  assert.match(source, /health-model-progress/);
  assert.match(source, /plugin:event\|listen/);
  const cancellation = source.split("health.cancellation")[1].split("health.restart")[0];
  assert.doesNotMatch(cancellation, /setTimeout\(resolvePromise, 250\)/);
  // The semantic trigger waits for an observed progress phase; fast progress
  // records completion instead of failing; a bounded wait is a diagnostic
  // failure (audit QD-03 V3), and one shared classifier owns the acceptance
  // rules for every observed outcome.
  assert.match(cancellation, /phaseSeen\.then/);
  assert.match(cancellation, /"completed-before-cancel"/);
  assert.match(cancellation, /"bounded-timeout"/);
  assert.match(cancellation, /classifyHealthCancellation\(outcome\)/);
  const classifier = await readFile(
    join(process.cwd(), "scripts", "lib", "health_cancel.mjs"),
    "utf8",
  );
  assert.match(classifier, /completed-before-cancel/);
  assert.match(classifier, /Bounded diagnostic/);
  // The fast/slow fixture legs are wired into the test suite.
  const pkg = JSON.parse(await readFile(join(process.cwd(), "package.json"), "utf8"));
  assert.ok(
    pkg.scripts.test.includes("scripts/tests/health_cancel.test.mjs"),
    "npm test must run the health-cancel fixture legs",
  );
  // The blocked-backend card check binds to the live upstream release; when
  // every required job is green it must report the skip rather than fail on a
  // healthy upstream (the card rendering stays unit-covered).
  assert.match(source, /no blocked backend in the live upstream release/);
});

test("release package job runs the catalog/SQLite packaged matrix (GH-05)", async () => {
  const release = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  const pkg = release.split("\n  package:")[1].split("\n  clean-account-lifecycle:")[0];
  for (const needle of [
    "verify_060_catalog.mjs init",
    "verify_060_catalog.mjs first-fill",
    "verify_060_catalog.mjs prep",
    "verify_060_catalog.mjs restart",
    "verify_060_catalog.mjs merge",
    "LOCALMOTIVE_CATALOG_URL",
    "LOCALMOTIVE_CATALOG_PUBKEY",
    // Tauri known-folder cache paths ignore a redirected LOCALAPPDATA: the
    // verifier must name the catalog cache root inside the isolated profile.
    "LOCALMOTIVE_CATALOG_ROOT",
    "packaged-verification-catalog-$env:VERSION.json",
  ]) {
    assert.ok(pkg.includes(needle), `package job is missing ${needle}`);
  }
  // Two launches share the isolated profile so the restart phase runs on the
  // same application data the first-fill phase wrote.
  assert.equal((pkg.match(/Start-Candidate \$cdpPort/g) ?? []).length, 2);
});

test("rich catalog facets keep the backend camelCase contract (GH-05)", async () => {
  const app = await frontendSources();
  const model = await readFile(join(process.cwd(), "src", "model.ts"), "utf8");
  const catalog = await readFile(join(process.cwd(), "src-tauri", "src", "catalog.rs"), "utf8");
  // The Rust struct serializes with rename_all = "camelCase"; a snake_case
  // read left the pipeline state undefined and crashed the catalog render.
  assert.match(catalog, /#\[serde\(rename_all = "camelCase"\)\]\npub struct CatalogFacets/);
  assert.match(app, /rich\.pipelineTags/);
  assert.doesNotMatch(app, /rich\.pipeline_tags/);
  assert.match(model, /pipelineTags: string\[\];/);
  const component = await readFile(join(process.cwd(), "src", "App.catalog.test.tsx"), "utf8");
  assert.match(component, /pipelineTags: \["text-generation"\]/);
});

test("verifier-only catalog overrides stay gated to the isolated verifier profile (GH-05)", async () => {
  const catalog = await readFile(join(process.cwd(), "src-tauri", "src", "catalog.rs"), "utf8");
  assert.match(catalog, /LOCALMOTIVE_VERIFY_ISOLATED_ROOT/);
  assert.match(catalog, /starts_with\("http:\/\/127\.0\.0\.1:"\)/);
  assert.match(catalog, /fn parse_verify_source/);
  const verify = await readFile(join(process.cwd(), "scripts", "verify_060_catalog.mjs"), "utf8");
  assert.match(verify, /LOCALMOTIVE_VERIFY_ISOLATED_ROOT/);
  assert.match(verify, /ed25519|generateKeyPairSync\("ed25519"\)/);
  assert.match(verify, /corrupted-signature/);
  assert.match(verify, /catalog-mirror\.sqlite/);
});

test("component test environment owns the catalog presentation scenarios (QD-02, QD-03)", async () => {
  const source = await readFile(join(process.cwd(), "src", "App.catalog.test.tsx"), "utf8");
  assert.match(source, /@vitest-environment jsdom/);
  assert.match(source, /vi\.mock\("@tauri-apps\/api\/core"/);
  assert.match(source, /No curated model matches these filters/);
  assert.match(source, /COOLDOWN/);
  assert.match(source, /runtime-catalog-message/);
  assert.match(source, /rate-limited|retry after 60 seconds/);
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

test("hardware qualify workflow pins every remote action and owns the host proof", async () => {
  const workflow = await readFile(join(process.cwd(), ".github", "workflows", "hardware-qualify.yml"), "utf8");
  assert.doesNotMatch(workflow, /uses:\s*actions\/checkout@v/);
  assert.doesNotMatch(workflow, /uses:\s*actions\/upload-artifact@v/);
  assert.match(workflow, /actions\/checkout@fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09/);
  assert.match(workflow, /actions\/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a/);
  // The clean-account lifecycle moved to release.yml (audit GH-03); this
  // workflow owns the host attestation job only.
  assert.doesNotMatch(workflow, /clean-account-lifecycle/);
  const release = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  assert.match(release, /clean-account-lifecycle:\n    needs: \[resolve, package\]/);
  const generator = await readFile(
    join(process.cwd(), "scripts", "qualification", "build_host_attestation.mjs"),
    "utf8",
  );
  assert.match(generator, /supportClaimPolicy/);
});

test("hardware qualify workflow cannot claim L4 support from host match alone", async () => {
  const workflow = await readFile(join(process.cwd(), ".github", "workflows", "hardware-qualify.yml"), "utf8");
  const generator = await readFile(
    join(process.cwd(), "scripts", "qualification", "build_host_attestation.mjs"),
    "utf8",
  );
  // Statuses are DERIVED from detected hardware (audit GH-10): no static
  // HOST_MATCH rows exist in the workflow, and the generator's policy string
  // keeps host presence separate from packaged L4 qualification.
  assert.doesNotMatch(workflow, /status = "HOST_MATCH"/);
  assert.match(workflow, /build_host_attestation\.mjs/);
  assert.match(generator, /schema: "localmotive\.attestation\.v0"/);
  assert.match(generator, /supportClaimPolicy:/);
  assert.doesNotMatch(generator, /L4_PASS"/);
  assert.match(generator, /Packaged L4 checks still required\./);
});

test("workflow gate policy covers the hardware qualify night path", async () => {
  const policy = JSON.parse(await readFile(join(process.cwd(), ".github", "workflow-gates.json"), "utf8"));
  assert.ok(policy.workflows["hardware-qualify.yml"]);
  // The lifecycle gate moved to release.yml with the job (audit GH-03).
  assert.deepEqual(Object.keys(policy.workflows["hardware-qualify.yml"].gates).sort(), ["hardware-qualify"]);
  assert.deepEqual(policy.workflows["hardware-qualify.yml"].packageJobs, {});
  assert.ok(policy.workflows["release.yml"].gates["clean-account-lifecycle"]);
  assert.equal(
    policy.workflows["release.yml"].requiredCommands["clean-account-lifecycle"].length,
    1,
  );
});

test("release ship gates default to the self-hosted runner, never windows-latest", async () => {
  const release = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  assert.doesNotMatch(release, /runs-on:\s*windows-latest/);
  for (const job of ["quality", "package", "publish"]) {
    const pattern = new RegExp(`^  ${job}:[\\s\\S]*?runs-on:\\s*(.+)$`, "m");
    const found = release.match(pattern);
    assert.ok(found, `${job} runs-on is missing`);
    assert.match(found[1], /self-hosted/);
    assert.match(found[1], /localmotive-hw/);
  }
});

test("release checkout pins line endings so the packaged clean-source probe is honest", async () => {
  const release = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  assert.doesNotMatch(release, /git-config:/);
  assert.match(release, /git config --global core\.autocrlf false/);
});

test("packaged clean-source probe ignores the verifier-owned artifacts directory", async () => {
  const source = await readFile(join(process.cwd(), "scripts", "verify_041.mjs"), "utf8");
  assert.match(source, /artifacts/);
});

test("packaged rejection detail preserves the backend error message", async () => {
  const source = await readFile(join(process.cwd(), "scripts", "verify_041.mjs"), "utf8");
  assert.match(source, /errorText/);
  assert.match(source, /hardware snapshot/);
  // CDP returnByValue stringifies thrown objects: the verifier must
  // serialize the raw IPC error inside the page, or kind is lost.
  assert.match(source, /JSON\.stringify\(error\)/);
  assert.match(source, /invalid_response/);
});

test("release verify step waits for the candidate WebView before driving checks", async () => {
  const release = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  const at = release.indexOf("Verify the packaged executable");
  assert.ok(at >= 0, "verify step is missing");
  const block = release.slice(at, at + 6000);
  assert.match(block, /function Start-Candidate/);
  assert.match(block, /Start-Process \$portable/);
  // A fixed sleep races WebView startup: the launch helper must poll the
  // CDP endpoint until the page appears instead of assuming readiness, and
  // every verifier run must go through the helper (two launches for the
  // catalog restart matrix).
  assert.match(block, /json\/list/);
  assert.equal((block.match(/Start-Candidate \$cdpPort/g) ?? []).length, 2);
});

test("release workflow serializes runs so two packages never share one runner", async () => {
  const release = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  // Tag-push and publish-dispatch each ran package on the same single
  // runner; the second CDP session attached to the first WebView.
  assert.match(release, /concurrency:\s*\n\s*group:\s*localmotive-release/);
  assert.match(release, /cancel-in-progress:\s*true/);
});

test("release verify step isolates the candidate behind a per-run CDP port with tree cleanup", async () => {
  const release = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  const at = release.indexOf("Verify the packaged executable");
  assert.ok(at >= 0, "verify step is missing");
  const block = release.slice(at, at + 6000);
  // Fixed port 10041 plus parent-only Stop-Process leaves an orphan
  // WebView2 holding CDP; the next run attaches to the stale page.
  // RED: the step hard-codes one port with no pre/post cleanup.
  assert.match(block, /GITHUB_RUN_ID/);
  assert.match(block, /taskkill \/F \/T/);
  assert.match(block, /webSocketDebuggerUrl/);
});

test("release publish verification avoids hosted-only shell dependencies", async () => {
  const release = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  // Git-bash on the self-hosted runner has no jq: parse the inventory
  // with node so the publish gate cannot fail on missing tooling.
  assert.doesNotMatch(release, /jq -r/);
});

test("tag-push publish runs the same gates as the dispatch path", async () => {
  const release = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  // Preferred ship path: pushing the version tag publishes after package and
  // the lifecycle verdict, so no second parallel package run ever shares the
  // runner CDP. Audit S-26: the guard binds the RESOLVED tag - the original
  // `== 'v0.5.0'` literal silently skipped publication for every later
  // version; the dispatch republish path stays behind its explicit input.
  assert.match(release, /needs\.quality\.outputs\.tag == github\.ref_name/);
  assert.match(release, /startsWith\(github\.ref, 'refs\/tags\/v'\)/);
  assert.match(release, /github\.event_name == 'workflow_dispatch' && inputs\.publish/);
});

test("branding history set covers the archived docs layout", async () => {
  const source = await readFile(join(process.cwd(), "scripts", "verify_branding.mjs"), "utf8");
  // Docs cleanup moves design/product/branding/llama-server notes under
  // docs/ and completed TODOs under docs/history/. The branding gate
  // must keep covering those paths after the move.
  // RED: HISTORICAL_FILES pins root paths that will no longer exist.
  assert.match(source, /docs\/history\/TODO-0\.4\.1\.md/);
});

test("local catalog SQLite mirror stores verified models with migrations and controlled recovery", async () => {
  const mirror = await readFile(join(process.cwd(), "src-tauri", "src", "catalog_db.rs"), "utf8");
  assert.match(mirror, /catalog_db_path/);
  assert.match(mirror, /CATALOG_DB_SCHEMA_VERSION/);
  // Audit DC-03: recovery quarantines the previous file and rebuilds from
  // verified bytes; the old silent rebuild symbol is gone for good.
  assert.match(mirror, /recover_catalog_db_from_verified/);
  assert.doesNotMatch(mirror, /rebuild_catalog_db_from_verified/);
  assert.match(mirror, /quarantine/);
  assert.match(mirror, /read_catalog_db_models/);
  assert.match(mirror, /mirror_verified_catalog/);
  assert.match(mirror, /migrate_catalog_db/);
});

test("catalog browse uses one merged collection and never the snapshot alone", async () => {
  const app = await frontendSources();
  // Audit DC-04: rows and facets are filtered from the merged local
  // collection, so user rows survive filtering and facets stay truthful.
  assert.match(app, /models: catalogAllRows/);
  assert.doesNotMatch(app, /models: catalogSnapshot\.catalog\.models/);
});

test("override saves reject mixed-origin collisions inside an immediate transaction", async () => {
  const mirror = await readFile(join(process.cwd(), "src-tauri", "src", "catalog_db.rs"), "utf8");
  // Audit DC-05/DC-06: ownership is reserved with actionable rejections, and
  // replacement runs in an IMMEDIATE transaction so reads inside it cannot
  // race a concurrent writer.
  assert.match(mirror, /belongs to a curated catalog entry and cannot be replaced/);
  assert.match(mirror, /belongs to the curated catalog and cannot be reused/);
  assert.match(mirror, /already belongs to the local override/);
  assert.match(mirror, /transaction_with_behavior\(rusqlite::TransactionBehavior::Immediate\)/);
});

test("FE-01 selection and runtime commit as coordinated transitions", async () => {
  const app = await frontendSources();
  // No silent fallback to another model, rescan preserves the committed
  // selection (or loads the replacement), failures clear together, and one
  // committed-runtime transition keeps profile.runtime in sync only after a
  // successful inspection (audit FE-01).
  assert.doesNotMatch(app, /\?\? models\[0\]/);
  assert.match(app, /selectionAfterRescan\(result, selectedId\)/);
  assert.match(app, /clearCommittedSelection\(\)/);
  assert.match(
    app,
    /function commitRuntime\(path: string, caps: RuntimeCapabilities, identity: RuntimeIdentity\)/,
  );
  assert.doesNotMatch(app, /if \(profile\) setProfile\(\{ \.\.\.profile, runtime: path \}\);/);
  assert.match(app, /commitRuntime\(path, caps, identity\);/);
});

test("FE-02 tuning adoption binds to the originating run", async () => {
  const app = await frontendSources();
  // The dispatch record captures provider/advisor/context; the report keeps
  // its origin; adoption writes the origin's profile key and only updates
  // the editable profile when the origin is the selected model (audit
  // FE-02).
  assert.match(app, /const run = \{\n      modelId: selected\.id,/);
  assert.match(app, /provider: run\.provider,/);
  assert.match(app, /setTuneReportOrigin\(run\);/);
  const adopt = app
    .split("function adoptTunedProfile()")[1]
    .split("function loadProfile(")[0];
  // Adoption persists through the guarded writer and reports a storage
  // failure without relabelling the adoption (audit FE-02 + FE-09 I3).
  assert.match(adopt, /persistRecord\(`profile:\$\{origin\.modelId\}`, JSON\.stringify\(adopted\)\)/);
  assert.match(adopt, /persistenceFailureNote\("The tuned profile"\)/);
  assert.match(adopt, /if \(selectedId === origin\.modelId\)/);
  assert.doesNotMatch(adopt, /selected\.id/);
});

test("FE-07 cancellation state is separate from the run lifecycle", async () => {
  const panel = await readFile(join(process.cwd(), "src", "V03EvidencePanel.tsx"), "utf8");
  // Cancellation progress is its own state and never routed through the
  // generic busy wrapper; the run keeps ownership until it settles
  // (audit FE-07).
  assert.match(panel, /const \[cancelPending, setCancelPending\] = useState\(false\)/);
  assert.doesNotMatch(panel, /runAction\("cancel"/);
  // Cancellation crosses the evidence adapter (S-27.I2); the adapter
  // forwards to the cancel_benchmark command it replaces here.
  assert.match(panel, /await adapter\.cancelBenchmark\(\)/);
  const adapterSource = await readFile(
    join(process.cwd(), "src", "evidence-adapter.ts"),
    "utf8",
  );
  assert.match(adapterSource, /invoke\("cancel_benchmark"\)/);
  assert.match(panel, /busy !== "benchmark" \|\| cancelPending/);
});

test("GH-01 pull requests run only on the isolated hosted runner", async () => {
  const ci = await readFile(join(process.cwd(), ".github", "workflows", "ci.yml"), "utf8");
  // A moveable PR trigger exists so required checks can execute per PR.
  assert.match(ci, /pull_request:\n    branches: \[main\]/);
  const pr = ci.split("  pr-check:")[1];
  assert.match(pr, /if: github\.event_name == 'pull_request'/);
  assert.match(pr, /runs-on: windows-latest/);
  assert.doesNotMatch(pr, /self-hosted/);
  assert.doesNotMatch(pr, /secrets\./);
  // Trusted self-hosted jobs are unreachable from a pull request.
  const check = ci.split("\n  check:")[1].split("\n  rust-audit:")[0];
  const audit = ci.split("\n  rust-audit:")[1].split("\n  package-smoke:")[0];
  const smoke = ci.split("\n  package-smoke:")[1].split("\n  pr-check:")[0];
  for (const block of [check, audit, smoke]) {
    assert.match(block, /if: github\.event_name == 'push'/);
    assert.match(block, /self-hosted/);
  }
});

test("GH-03 the lifecycle consumes candidates instead of waiting for publication", async () => {
  const release = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  const hardware = await readFile(join(process.cwd(), ".github", "workflows", "hardware-qualify.yml"), "utf8");
  const lifecycle = release.split("clean-account-lifecycle:")[1].split("\n  publish:")[0];
  assert.match(lifecycle, /needs: \[resolve, package\]/);
  assert.match(lifecycle, /-CandidateDir "artifacts"/);
  assert.doesNotMatch(lifecycle, /ReleaseWaitMinutes/);
  const publishHead = release.split("\n  publish:")[1].split("steps:")[0];
  // Fail-closed supersession of the original "must not gate on the
  // interactive Sandbox feature" note: absent or failed lifecycle evidence
  // blocks publication (G-06.I1, G-09.V1 negative control). The lifecycle
  // itself still consumes CANDIDATE bytes rather than waiting for published
  // assets, which is this test's subject.
  assert.match(publishHead, /clean-account-lifecycle/, "publication waits for the lifecycle verdict (fail-closed)");
  assert.doesNotMatch(hardware, /clean-account-lifecycle/, "the lifecycle job belongs to release.yml");
});

test("GH-04 installer verdicts fail on leftovers and verify installed versions", async () => {
  const sandbox = await readFile(join(process.cwd(), "scripts", "sandbox", "run-lifecycle-in-sandbox.ps1"), "utf8");
  assert.doesNotMatch(sandbox, /WARNING: exe still present after MSI uninstall/);
  assert.match(sandbox, /Fail "Localmotive\.exe still present after MSI uninstall/);
  assert.match(sandbox, /Test-MsiProductInstalled/);
  assert.match(sandbox, /Assert-AppVersion \$exe \$meta\.version "MSI fresh install"/);
  assert.match(sandbox, /Assert-AppVersion \$exe \$meta\.version "Post-update install"/);
  const release = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  assert.match(release, /UPGRADE_BASELINE: v0\.4\.0/);
  assert.doesNotMatch(release, /-PreviousTag "v0\.4\.0"/, "the baseline flows through the explicit matrix variable");
});

test("GH-06 lifecycle evidence survives every terminal outcome", async () => {
  const host = await readFile(join(process.cwd(), "scripts", "sandbox", "host-run-lifecycle.ps1"), "utf8");
  assert.match(host, /function Write-FailureEvidence/);
  assert.match(host, /Write-FailureEvidence "TIMEOUT"/);
  assert.match(host, /Write-FailureEvidence "FAIL" \$_\.Exception\.Message/);
  // The FAIL branch must retain the sandbox's exact result JSON enriched
  // with host-side identity, not a bare copy: a failed run has to be
  // attributable (its source revision and candidate digests), not merely
  // retrievable. The plain copy was replaced after the wrong-candidate
  // control showed the retained FAIL document carried no host identity.
  assert.match(host, /\$failDoc \| Add-Member -NotePropertyName sourceRevision/);
  assert.match(host, /\$failDoc \| Add-Member -NotePropertyName candidateDigests/);
  // GH-06.V2: every terminal-outcome witness leg is driven through the
  // default-off fault simulations and asserted by the witness runner:
  // timeout, malformed result, early installer failure, and cancellation
  // (a killed run leaves no PASS artifact).
  for (const mode of ["timeout", "malformed-result", "missing-assets", "stall"]) {
    assert.ok(host.includes(`"${mode}"`), `host harness is missing the ${mode} fault mode`);
  }
  const witness = await readFile(
    join(process.cwd(), "scripts", "sandbox", "test-fault-evidence.ps1"),
    "utf8",
  );
  assert.match(witness, /Assert-Witness "witness-missing-assets" "FAIL" "resolve-installers"/);
  assert.match(witness, /Assert-Witness "witness-timeout" "TIMEOUT" "sandbox-timeout"/);
  assert.match(witness, /Assert-Witness "witness-malformed-result" "FAIL" "sandbox-run"/);
  assert.match(witness, /a killed run must not leave PASS evidence/);
  const release = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  const upload = release.split("Upload lifecycle evidence")[1] ?? "";
  assert.match(upload, /if: always\(\)/);
  assert.match(upload, /if-no-files-found: error/);
});

test("GH-10 host attestation statuses derive from detected hardware", async () => {
  const { deriveHostRows } = await import("../qualification/build_host_attestation.mjs");
  const match = deriveHostRows({ cpu: "AMD Ryzen 9 9950X3D 16-Core Processor", gpu: "NVIDIA GeForce RTX 5090" });
  assert.equal(match.verdict, "MATCH");
  assert.ok(match.rows.every((row) => row.status === "HOST_MATCH"));
  assert.equal(match.rows[0].observation, "AMD Ryzen 9 9950X3D 16-Core Processor");
  const cpuMismatch = deriveHostRows({ cpu: "Intel Core i9-13900K", gpu: "NVIDIA GeForce RTX 5090" });
  assert.equal(cpuMismatch.verdict, "NO_MATCH");
  assert.equal(cpuMismatch.rows[0].status, "HOST_NO_MATCH");
  const gpuMismatch = deriveHostRows({ cpu: "AMD Ryzen 9 9950X3D", gpu: "NVIDIA GeForce RTX 4080" });
  assert.equal(gpuMismatch.verdict, "NO_MATCH");
  assert.equal(gpuMismatch.rows[1].status, "HOST_NO_MATCH");
  const unknown = deriveHostRows({ cpu: "AMD Ryzen 9 9950X3D", gpu: "" });
  assert.equal(unknown.verdict, "UNKNOWN");
  assert.equal(unknown.rows[1].status, "HOST_UNKNOWN");
  const hardware = await readFile(join(process.cwd(), ".github", "workflows", "hardware-qualify.yml"), "utf8");
  assert.doesNotMatch(hardware, /status = "HOST_MATCH"/, "no static match rows may remain");
  assert.match(hardware, /build_host_attestation\.mjs/);
});

test("GH-02 release jobs share one resolved immutable revision", async () => {
  const release = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  // One resolver pins the tag to a full SHA once; every later job checks
  // out exactly that revision (audit GH-02 I1).
  assert.match(release, /\n  resolve:\n/);
  const pinned = release.match(/ref: \$\{\{ needs\.resolve\.outputs\.sha \}\}/gu) ?? [];
  assert.ok(pinned.length >= 4, `expected at least four pinned checkouts, saw ${pinned.length}`);
  assert.equal(
    (release.match(/github\.event\.inputs\.tag \|\| github\.ref \}\}/gu) ?? []).length,
    1,
    "only the resolver may check out the requested tag ref",
  );
  // Publication verifies the producer inventory instead of rewriting it.
  const publish = release.split("\n  publish:")[1];
  assert.match(publish, /verify_candidate_inventory\.mjs --verify/);
  assert.doesNotMatch(publish, /verify_candidate_inventory\.mjs artifacts "\$VERSION"/);
  assert.match(publish, /LOCALMOTIVE_SOURCE_REVISION="\$RESOLVED_SHA"/);
});

test("FE-04 previews compose provisionally and launch trust stays authoritative", async () => {
  const lib = await readFile(join(process.cwd(), "src-tauri", "src", "lib.rs"), "utf8");
  const app = await frontendSources();
  // The preview command is cheap composition only: no probing or trust work
  // on the profile-edit path (audit FE-04).
  const preview = lib.split("fn preview_command(")[1].split("#[tauri::command]")[0];
  // The preview names its shell and offers a lossless argv form (S-14) while
  // staying cheap composition only.
  assert.match(preview, /escaped_command_with_args/);
  assert.match(preview, /CommandShell::PowerShell/);
  assert.match(preview, /argv_json_with_args/);
  assert.doesNotMatch(preview, /prepare_launch/);
  // Validation and launch keep the authoritative checks.
  const validate = lib.split("fn validate_launch_profile(")[1].split("#[tauri::command]")[0];
  assert.match(validate, /prepare_launch/);
  // The server lifecycle commands moved to server_service.rs (S-27 slice 3a).
  const serverSource = await readFile(
    join(process.cwd(), "src-tauri", "src", "server_service.rs"),
    "utf8",
  );
  const start = serverSource
    .split("async fn start_server(")[1]
    .split("fn start_server_worker(")[0];
  assert.match(start, /start_server_worker/);
  const worker = serverSource
    .split("fn start_server_worker(")[1]
    .split("#[tauri::command]")[0];
  assert.match(worker, /spawn_server/);
  // The UI describes the provisional state honestly.
  assert.match(app, /Provisional command/);
  assert.match(app, /capability filtering, artifact checks, and managed-runtime trust are enforced/);
});

test("FE-16 status polling is single-flight with sequence guards", async () => {
  const app = await frontendSources();
  // Slow native polls cannot pile up, and a poll that started before a
  // start/stop transition can never overwrite the newer snapshot; the
  // running strategy renders from that snapshot, not the editable draft
  // (audit FE-16).
  assert.match(app, /if \(inFlight\) return; \/\/ single-flight/);
  assert.match(app, /requested !== statusPollSeq\.current/);
  assert.match(app, /const requested = \+\+statusPollSeq\.current;/);
  const bumps = (app.match(/statusPollSeq\.current \+= 1;/g) ?? []).length;
  assert.ok(bumps >= 2, `expected start and stop bumps, saw ${bumps}`);
  assert.match(app, /(?:props\.)?status\.running \? (?:props\.)?status\.specType : (?:props\.)?profile\?\.specType/);
  // FE-16.V3: the log well retains the last bounded output after a stop or an
  // unexpected exit, and no refactor placeholder may leak into user copy.
  const dashboard = await readFile(join(process.cwd(), "src", "screens", "DashboardScreen.tsx"), "utf8");
  assert.match(dashboard, /className="log-retained"/);
  assert.match(dashboard, /The last bounded output remains visible until the next start\./);
  assert.match(dashboard, /Start this profile to stream llama-server output here\./);
  assert.doesNotMatch(dashboard, /Start this props\./);
});

test("the cancellable local client never re-issues a slow-but-healthy response (MT-06)", async () => {
  const source = await readFile(join(process.cwd(), "src-tauri", "src", "local_client.rs"), "utf8");
  // Regression pinned after the 0.6.0 re-bind probes: the cancellable path
  // once bounded every attempt to CANCEL_ATTEMPT_SLICE and re-issued it,
  // which livelocked the v2 benchmark on the managed runtime (~560 ms per
  // completion). The request must run on a worker with the full deadline
  // while the caller observes the cancel flag in slices.
  assert.doesNotMatch(source, /remaining\.min\(CANCEL_ATTEMPT_SLICE\)/);
  assert.doesNotMatch(source, /is_timeout\(\) && cancelled\.is_some\(\)/);
  assert.match(source, /recv_timeout\(CANCEL_ATTEMPT_SLICE\)/);
  assert.match(source, /a_cancellable_response_that_outlives_the_cancel_slice_still_completes/);
  assert.match(source, /fn run_local_request\(/);
});

test("FE-16 and FE-05.V3 packaged walks drive the real routes", async () => {
  const fe16 = await readFile(join(process.cwd(), "scripts", "g05_fe16.mjs"), "utf8");
  assert.match(fe16, /fe16\.v1\.v2-survives-overlap/);
  assert.match(fe16, /fe16\.v1\.legacy-guarded-during-v2/);
  assert.match(fe16, /fe16\.v3\.running-identity-unchanged-by-edit/);
  assert.match(fe16, /fe16\.v3\.final-log-visible/);
  const fe05 = await readFile(join(process.cwd(), "scripts", "g05_fe05v3.mjs"), "utf8");
  assert.match(fe05, /Add anchor/);
  assert.match(fe05, /Replay manifest/);
  assert.match(fe05, /load_calibration_records/);
  assert.match(fe05, /fe05v3\.records-distinct-provenance/);
  assert.match(fe05, /fe05v3\.records-distinct-paths/);
});

test("FE-05 evidence history and active runs survive navigation", async () => {
  const app = await frontendSources();
  const panel = await readFile(join(process.cwd(), "src", "V03EvidencePanel.tsx"), "utf8");
  // The evidence panel is always mounted (hidden by style), never gated on
  // the benchmark view, so navigation cannot erase its state (audit FE-05).
  assert.doesNotMatch(app, /view === "benchmark" && \(/);
  assert.match(
    app,
    /style=\{view === "benchmark" \? undefined : \{ display: "none" \}\}/,
  );
  // The always-mounted section renders BenchmarkScreen, which owns the
  // panel; the mount is what guarantees survival, not the literal call site.
  assert.match(app, /<BenchmarkScreen/);
  assert.match(app, /onRunStateChange=\{(?:props\.)?setEvidenceRun\}/);
  // The active run's status and cancel handle are available app-wide.
  assert.match(app, /evidenceRun\.cancel && \(/);
  // Completed runs are retained across model/profile changes.
  assert.doesNotMatch(panel, /setHistory\(\[\]\)/);
  // Export approval resets when the reviewed payload changes.
  assert.match(panel, /\[benchmark\?\.manifestPath, quality\?\.observedAtMs\]/);
});

test("FE-03 stale responses are guarded before they can commit", async () => {
  const app = await frontendSources();
  // Audit FE-03: every deferred commit checks its request sequence and the
  // current resource identity, provider switches clear the previous
  // provider's presentation first, GGUF metadata clears with its selection,
  // and the port suggestion passes through the identity-checked helper.
  assert.match(app, /responseIsCurrent\(/);
  assert.match(app, /applySuggestedPort\(/);
  assert.match(app, /profileIdentity\(/);
  assert.match(app, /cloudSeq\.current \+= 1;\n    setCredential\(null\);/);
  assert.match(
    app,
    /async function loadGguf\(model: LogicalModel \| undefined\) \{\n    const sequence = \+\+ggufSeq\.current;\n    if \(!model\) \{/,
  );
  assert.match(app, /const sequence = \+\+previewSeq\.current;/);
});

test("user catalog overrides stay local, marked, and outside network verification", async () => {
  const mirror = await readFile(join(process.cwd(), "src-tauri", "src", "catalog_db.rs"), "utf8");
  const lib = await readFile(join(process.cwd(), "src-tauri", "src", "lib.rs"), "utf8");
  const app = await frontendSources();
  assert.match(mirror, /user_sourced/);
  assert.match(mirror, /validate_user_override/);
  assert.match(lib, /save_user_catalog_override/);
  assert.match(lib, /remove_user_catalog_override/);
  assert.match(lib, /catalog_local_models/);
  assert.match(app, /USER ADDED/);
});

test("IPC-02 production CSP and opener scope are narrow and audited", async () => {
  const configDir = join(process.cwd(), "src-tauri");
  const config = JSON.parse(await readFile(join(configDir, "tauri.conf.json"), "utf8"));
  const security = config.app?.security ?? {};
  assert.ok(
    typeof security.csp === "string" && security.csp.length > 0,
    "a production CSP must be configured",
  );
  assert.ok(!security.csp.includes("unsafe-eval"), "no unsafe-eval workaround in production");
  assert.match(security.csp, /script-src 'self'/);
  assert.match(security.csp, /connect-src [^;]*ipc:/);
  assert.match(security.csp, /connect-src [^;]*http:\/\/ipc\.localhost/);
  assert.ok(
    typeof security.devCsp === "string" && security.devCsp.includes("localhost:1420"),
    "the development-only exception must be explicit and separate",
  );
  assert.ok(!security.devCsp.includes("unsafe-eval"), "no unsafe-eval in the dev policy either");

  const capabilities = JSON.parse(
    await readFile(join(configDir, "capabilities", "default.json"), "utf8"),
  );
  assert.ok(
    !capabilities.permissions.includes("opener:default"),
    "the unscoped opener default permission set must not be granted",
  );
  const opener = capabilities.permissions.find(
    (permission) =>
      typeof permission === "object" && permission.identifier === "opener:allow-open-url",
  );
  assert.ok(opener, "opener must use the scoped allow-open-url permission");
  assert.ok(Array.isArray(opener.allow) && opener.allow.length > 0);
  for (const entry of opener.allow) {
    assert.ok(typeof entry.url === "string" && entry.url.length > 0, "every scope entry names a URL");
  }
  const patterns = opener.allow.map((entry) => entry.url);
  assert.ok(!patterns.includes("https://*") && !patterns.includes("*"));
  assert.ok(patterns.some((pattern) => pattern.startsWith("http://127.0.0.1/")));

  // Positive control: production sources contain no HTML/eval sinks.
  const sources = await readdir(join(process.cwd(), "src"));
  const sink = /(dangerouslySetInnerHTML|new Function\s*\(|document\.write|(^|[^a-zA-Z_.])eval\s*\()/;
  for (const name of sources) {
    if (!/\.(ts|tsx)$/.test(name)) continue;
    if (/\.test\.(ts|tsx)$/.test(name)) continue;
    const content = await readFile(join(process.cwd(), "src", name), "utf8");
    assert.ok(!sink.test(content), `unexpected injection sink in src/${name}`);
  }
});

test("FE-12 path-bar inputs keep a visible keyboard-focus ring", async () => {
  const css = await readFile(join(process.cwd(), "src", "App.css"), "utf8");
  assert.match(
    css,
    /\.path-bar input:focus-visible\s*\{[^}]*outline:/s,
    "the path-bar input outline reset must be paired with a focus-visible ring",
  );
  assert.ok(
    /\.path-bar input\s*\{[^}]*outline:\s*0/s.test(css.split(".path-bar input:focus-visible")[0]),
    "sanity: the outline reset still exists before the focus rule",
  );
});

test("FE-14 small-text colors keep at least 4.5:1 on their panels", async () => {
  const css = await readFile(join(process.cwd(), "src", "App.css"), "utf8");
  for (const old of ["#6f7974", "#77817d", "#7f8885", "#737b78"]) {
    assert.ok(!css.includes(old), `${old} was below 4.5:1 and must not return`);
  }
  const linear = (channel) => {
    const value = channel / 255;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  };
  const luminance = (hex) => {
    const value = hex.replace("#", "");
    const [r, g, b] = [0, 2, 4].map((offset) => Number.parseInt(value.slice(offset, offset + 2), 16));
    return 0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b);
  };
  const ratio = (fg, bg) => {
    const [a, b] = [luminance(fg), luminance(bg)].sort((x, y) => y - x);
    return (a + 0.05) / (b + 0.05);
  };
  const muted = "#9ca3a0";
  assert.match(css, /\.hardware-source \{[^}]*#9ca3a0/s, "the hardware-source line uses the muted token");
  for (const panel of ["#202622", "#222627", "#111514"]) {
    assert.ok(ratio(muted, panel) >= 4.5, `${muted} on ${panel} is ${ratio(muted, panel).toFixed(3)}:1`);
  }
});

test("DC-09 quant labels come from the file's own token and stay canonical", async () => {
  const { quantFromFilename, isCanonicalQuant } = await import("../lib/quant_label.mjs");
  const cases = [
    ["Model-IQ2_S-MTP.gguf", "IQ2_S"],
    ["Model-Q4_K_M-imatrix.gguf", "Q4_K_M"],
    ["Model-Q5_K_M-0731.gguf", "Q5_K_M"],
    ["Model-Q4_K_M-it.gguf", "Q4_K_M"],
    ["Model-q8_0.gguf", "Q8_0"],
    ["Model.q8_0.gguf", "Q8_0"],
    ["Model-UD-Q4_K_XL.gguf", "Q4_K_XL"],
    ["base-Q4_K_M-draft-Q8_0.gguf", "Q4_K_M"],
    ["gemma-4-E4B_q4_0-it.gguf", "Q4_0"],
    ["Date-Only-2025-0731.gguf", "UNKNOWN"],
    ["Qwen2.5-Coder-7B-Instruct.gguf", "UNKNOWN"],
  ];
  for (const [name, want] of cases) {
    assert.equal(quantFromFilename(name), want, name);
  }
  // The checked-in catalog keeps the labels its detached signature covers;
  // corrected labels arrive with the next signed publication, which rebuilds
  // through the fixed builder. The gate therefore pins the extractor and the
  // builder, not the signed data file.
  assert.ok(isCanonicalQuant("Q4_K_M") && !isCanonicalQuant("imatrix"));

  // The builder must use the same extractor; the suffix regex is gone.
  const builder = await readFile(join(process.cwd(), "scripts", "build_catalog.mjs"), "utf8");
  assert.ok(
    builder.includes("quantFromFilename(name)"),
    "the builder must label files through the shared extractor",
  );
  assert.ok(
    !builder.includes("match(/-([A-Za-z0-9_]+)\\.gguf$/i)"),
    "the suffix regex must not return",
  );
});

test("DC-10 the builder pins immutable revisions and signed freshness fields", async () => {
  const builder = await readFile(join(process.cwd(), "scripts", "build_catalog.mjs"), "utf8");
  assert.match(builder, /meta\.sha/, "the builder reads the repository commit");
  assert.match(builder, /revision: commitSha/, "files carry the immutable revision");
  assert.match(builder, /\.\.\.\(commitSha \? \{ revision: commitSha \} : \{\}\)/, "the revision is omitted rather than empty when the API reports none");
  assert.match(builder, /sequence: nowMs/, "the catalog carries a signed sequence");
  assert.match(
    builder,
    /expires: Math\.floor\(nowMs \/ 1000\) \+ 14 \* 24 \* 60 \* 60/,
    "the catalog carries a signed expiry deadline",
  );
});

test("QD-01 the catalog cutoff follows the build clock", async () => {
  const { catalogCutoff } = await import("../lib/catalog_window.mjs");
  // The audit's reproduction: identical inputs a year apart must not share
  // a threshold; the old literal kept June 12, 2026 for every build.
  const first = catalogCutoff(new Date("2026-09-10T12:00:00Z"), 90);
  const later = catalogCutoff(new Date("2027-09-10T12:00:00Z"), 90);
  assert.notEqual(first.toISOString(), later.toISOString());
  assert.equal(first.toISOString().slice(0, 10), "2026-06-12");
  assert.equal(later.toISOString().slice(0, 10), "2027-06-12");
  // The threshold is UTC midnight of (now - cutoffDays).
  assert.equal(catalogCutoff(new Date("2026-09-10T23:59:59Z"), 30).toISOString(), "2026-08-11T00:00:00.000Z");

  const builder = await readFile(join(process.cwd(), "scripts", "build_catalog.mjs"), "utf8");
  assert.ok(
    builder.includes("catalogCutoff(new Date(), CUTOFF_DAYS)"),
    "the builder must derive the cutoff from the build clock",
  );
  assert.ok(
    !builder.includes('new Date("2026-09-10T00:00:00Z")'),
    "the frozen literal cutoff must not return",
  );
});

test("QD-04 active docs agree with the shipped unsigned policy and current platform", async () => {
  const runtimeManager = await readFile(join(process.cwd(), "docs", "RUNTIME_MANAGER.md"), "utf8");
  assert.ok(
    runtimeManager.includes("ship **unsigned with an explicit"),
    "the runtime manager notes the deferred-signing release policy",
  );
  const product = await readFile(join(process.cwd(), "docs", "PRODUCT.md"), "utf8");
  assert.ok(
    !/^web$/m.test(product.split("## Stack")[0]),
    "the product schema no longer claims the web platform",
  );
  assert.ok(product.includes("SQLite-backed local catalog mirror"), "the SQLite mirror is described");
  const qualification = await readFile(join(process.cwd(), "docs", "qualification-tests.md"), "utf8");
  assert.ok(qualification.includes("Frozen snapshot"), "the qualification snapshot is labelled");
  assert.ok(!qualification.includes("verify_versions.mjs 0.4.1"), "the stale command is corrected");
});

test("QD-05 the toolchain minimum is declared", async () => {
  const packageJson = JSON.parse(await readFile(join(process.cwd(), "package.json"), "utf8"));
  assert.equal(packageJson.engines?.node, "^20.19.0 || >=22.12.0");
  assert.match(packageJson.packageManager ?? "", /^npm@/);
  const toolchain = await readFile(join(process.cwd(), "rust-toolchain.toml"), "utf8");
  assert.match(toolchain, /channel = "\d+\.\d+\.\d+"/, "the toolchain is pinned");
  const cargo = await readFile(join(process.cwd(), "src-tauri", "Cargo.toml"), "utf8");
  assert.match(cargo, /rust-version = "\d+\.\d+"/, "rust-version mirrors the pin");
  const readme = await readFile(join(process.cwd(), "README.md"), "utf8");
  assert.ok(readme.includes("20.19"), "the README states the real Node minimum");
});

test("QD-06 vendored upstream docs carry provenance and links resolve elsewhere", async () => {
  const vendored = await readFile(join(process.cwd(), "docs", "LLAMA-SERVER-README.md"), "utf8");
  assert.ok(vendored.includes("Provenance"), "the vendored README identifies its source");
  assert.ok(vendored.includes("ggml-org/llama.cpp"), "the upstream repository is named");
  const optionMap = await readFile(join(process.cwd(), "docs", "OPTION_MAP.md"), "utf8");
  assert.ok(optionMap.includes("--help` is authoritative"), "OPTION_MAP records its source policy");
});

test("QD-06 local doc links resolve (the vendored upstream README is excluded)", async () => {
  // The vendored docs/LLAMA-SERVER-README.md intentionally keeps upstream
  // relative targets (see its provenance banner); every other active doc
  // must resolve its relative links inside this repository.
  const files = ["README.md", "docs/OPTION_MAP.md", "docs/PRODUCT.md", "docs/RUNTIME_MANAGER.md"];
  const missing = [];
  for (const file of files) {
    const content = await readFile(join(process.cwd(), file), "utf8");
    for (const match of content.matchAll(/\]\((?!https?:|#|mailto:)([^)]+)\)/g)) {
      const link = match[1].split("#")[0].trim();
      if (!link) continue;
      const target = resolve(dirname(join(process.cwd(), file)), link);
      try {
        await stat(target);
      } catch {
        missing.push(`${file} -> ${link}`);
      }
    }
  }
  assert.deepEqual(missing, [], `unresolved relative links: ${missing.join(", ")}`);
});

test("GH-07 the evidence matrix exists and the README reads it", async () => {
  const matrix = await readFile(join(process.cwd(), "docs", "EVIDENCE-MATRIX.md"), "utf8");
  for (const cell of ["CPU packaged lifecycle", "Accelerator (CUDA) packaged", "Clean-account Sandbox"]) {
    assert.ok(matrix.includes(cell), `the matrix must define ${cell}`);
  }
  assert.ok(matrix.includes("assets"), "evidence assets must be named");
  const readme = await readFile(join(process.cwd(), "README.md"), "utf8");
  assert.ok(readme.includes("docs/EVIDENCE-MATRIX.md"), "the README points at the matrix");
  // The README must state what 0.6.0 actually established and explicitly
  // refuse to generalize it; the accelerator caveat wording moved when the
  // support matrix was introduced, so accept either explicit phrasing.
  assert.ok(
    /not\s+covered by that lifecycle evidence unless/.test(readme) ||
      /Do not generalize/.test(readme),
    "accelerator coverage is not inferred",
  );
  assert.ok(
    !readme.includes("during 01:00-06:00 Asia/Dubai."),
    "the stale night-window-only claim must not return",
  );
  const workflow = await readFile(
    join(process.cwd(), ".github", "workflows", "hardware-qualify.yml"),
    "utf8",
  );
  assert.ok(
    !workflow.includes("Runner online window: **01:00-06:00"),
    "the workflow summary must not claim a night-only window",
  );
});

test("GH-08 supply-chain posture and SBOM step are documented", async () => {
  const supply = await readFile(join(process.cwd(), "docs", "SUPPLY-CHAIN.md"), "utf8");
  assert.ok(supply.includes("not cryptographic"), "attestation limits are stated");
  assert.ok(supply.includes("does not authenticate application installers"), "catalog scope is stated");
  assert.ok(supply.includes("Unsigned installers"), "the unsigned posture is stated");
  const release = await readFile(
    join(process.cwd(), ".github", "workflows", "release.yml"),
    "utf8",
  );
  assert.ok(release.includes("npm sbom --sbom-format cyclonedx"), "the SBOM step exists");
  assert.match(release, /name: sbom-\$\{\{ needs\.quality\.outputs\.version \}\}/, "the SBOM artifact is versioned");
});

test("GH-09 governance files exist and dependency updates are configured", async () => {
  const security = await readFile(join(process.cwd(), "SECURITY.md"), "utf8");
  assert.ok(security.includes("private vulnerability reporting"), "a reporting path is documented");
  assert.ok(security.includes("no bug bounty"), "expectations are stated");
  const contributing = await readFile(join(process.cwd(), "CONTRIBUTING.md"), "utf8");
  assert.ok(contributing.includes("cargo clippy --all-targets -- -D warnings"), "the check suite is listed");
  assert.ok(contributing.includes("Maintenance and recovery"), "backup/recovery guidance exists");
  await stat(join(process.cwd(), ".github", "ISSUE_TEMPLATE", "bug_report.yml"));
  await stat(join(process.cwd(), ".github", "ISSUE_TEMPLATE", "feature_request.yml"));
  const dependabot = await readFile(join(process.cwd(), ".github", "dependabot.yml"), "utf8");
  for (const ecosystem of ["npm", "cargo", "github-actions"]) {
    assert.ok(dependabot.includes(`package-ecosystem: ${ecosystem}`), `dependabot covers ${ecosystem}`);
  }
});

test("S-17 imported evidence cannot leak into ranking or calibration writers", async () => {
  const { execSync } = await import("node:child_process");
  const offenders = execSync(
    "git grep -l ExternalEvidenceBundle -- src-tauri/src src scripts",
    { cwd: process.cwd(), encoding: "utf8" },
  )
    .trim()
    .split("\n")
    .filter(Boolean)
    .filter((file) => !file.endsWith("calibration.rs") && !file.endsWith("lib.rs") && !file.endsWith("model.ts") && !file.endsWith("V03EvidencePanel.tsx") && !file.endsWith("evidence-adapter.ts") && !file.endsWith("release-gates.test.mjs"));
  assert.deepEqual(
    offenders,
    [],
    `external evidence types may only live in calibration.rs, lib.rs, model.ts, V03EvidencePanel.tsx and the evidence-adapter acquisition seam; found: ${offenders.join(", ")}`,
  );
  // The ranking and measurement writers never mention it.
  for (const module of ["src-tauri/src/recommend.rs", "src-tauri/src/measurement.rs", "src-tauri/src/sharing.rs"]) {
    const source = await readFile(join(process.cwd(), module), "utf8");
    assert.doesNotMatch(source, /ExternalEvidence/, `${module} must not consume imported evidence`);
  }
});

test("adapter invokes wrap Rust commands whose only parameter is `request`", async () => {
  // Regression: `preflight_model(request)` was called with the request fields
  // passed flat, so the packaged panel answered "command preflight_model
  // missing required key `request`" and the v2 evidence flow silently produced
  // no result. Any adapter pass-through (`invoke("x", args)`) to a Rust command
  // whose only non-AppHandle parameter is `request` must wrap it.
  const adapter = await readFile(join(process.cwd(), "src", "evidence-adapter.ts"), "utf8");
  const rustSources = [];
  for (const module of ["lib.rs", "measurement_service.rs", "runtime_service.rs", "server_service.rs", "tune_service.rs"]) {
    try {
      rustSources.push(await readFile(join(process.cwd(), "src-tauri", "src", module), "utf8"));
    } catch {
      // module not present in this layout
    }
  }
  const entries = [...adapter.matchAll(/invoke\("([a-z0-9_]+)",\s*args\)/g)].map((match) => match[1]);
  const offenders = [];
  for (const name of entries) {
    for (const source of rustSources) {
      const signature = source.match(new RegExp(`fn ${name}\\(([^)]*)`));
      if (!signature) continue;
      const params = signature[1]
        .split(",")
        .map((part) => part.trim().split(":")[0].trim())
        .filter((part) => part && part !== "app");
      if (params.length === 1 && params[0] === "request") offenders.push(name);
      break;
    }
  }
  assert.deepEqual(
    offenders,
    [],
    `commands whose only parameter is \`request\` must be invoked as { request: args }: ${offenders.join(", ")}`,
  );
});

test("S-26: no workflow carries a literal release version and gates stay fail-fast", async () => {
  const ci = await readFile(join(process.cwd(), ".github", "workflows", "ci.yml"), "utf8");
  const release = await readFile(
    join(process.cwd(), ".github", "workflows", "release.yml"),
    "utf8",
  );
  // No workflow may pin a literal version number: the non-release jobs use
  // the no-argument manifest-consistency mode, and the release job binds the
  // RESOLVED tag version.
  for (const [name, body] of [["ci.yml", ci], ["release.yml", release]]) {
    assert.doesNotMatch(
      body,
      /verify_versions\.mjs\s+["']?\d+\.\d+\.\d+/,
      `${name} must not pass a literal version to verify_versions.mjs`,
    );
    assert.doesNotMatch(
      body,
      /only v\d/,
      `${name} must not pin a single tag literal`,
    );
  }
  assert.match(
    ci,
    /node scripts\/verify_versions\.mjs\n/,
    "ci.yml uses the no-argument manifest-consistency mode",
  );
  assert.match(
    release,
    /node scripts\/verify_versions\.mjs \$\{\{ needs\.resolve\.outputs\.version \}\}/,
    "release.yml binds the resolved tag version to the manifests",
  );
  // The tag shape is validated by pattern, and the resolved revision must
  // identify as the same version before anything is published.
  assert.match(release, /case "\$raw" in\n\s*v\[0-9\]\*\.\[0-9\]\*\.\[0-9\]\*\)/, "release.yml validates the tag pattern");
  assert.match(
    release,
    /node scripts\/verify_versions\.mjs "\$\{raw#v\}" \\/,
    "the resolve job refuses source whose manifests identify as another version",
  );
  // The fail-fast pairing is documented in every job that repeats the gates.
  const failFast = (ci.match(/fail-fast; repeated by npm run check/g) ?? []).length;
  assert.equal(failFast, 2, "both ci.yml jobs document the fail-fast pairing");
  assert.match(
    release,
    /fail-fast; repeated by npm run check/,
    "the release quality job documents the fail-fast pairing",
  );
  // Audit S-26 I2 replaced the resolve-job literal; the publish ship guard
  // must bind the RESOLVED tag the same way - a `== 'v0.5.0'`-style equality
  // silently skips publication for every later version.
  assert.doesNotMatch(
    release,
    /'v\d+\.\d+\.\d+'|"v\d+\.\d+\.\d+"/,
    "release.yml must not compare against a literal version",
  );
});

test("publish promotes the verified bytes only after every material gate (G-06/G-09)", async () => {
  const release = await readFile(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
  const publish = release.split("\n  publish:")[1];
  assert.ok(publish, "release.yml has a publish job");
  // Publication promotes the artifact the package job verified - it never
  // rebuilds and never rewrites the checksum inventory.
  assert.match(publish, /Retrieve verified release candidates/, "publish retrieves the verified candidate artifact");
  assert.match(
    publish,
    /localmotive-\$\{\{ needs\.quality\.outputs\.version \}\}-verified/,
    "publish consumes the -verified artifact uploaded by the package job",
  );
  assert.match(publish, /sha256sum -c/, "publish verifies the producer checksums");
  assert.doesNotMatch(publish, /tauri build|sha256sum Localmotive/, "publish must not rebuild or rewrite the inventory");
  // Absent or failed lifecycle evidence blocks publication (G-06.I1); a job
  // that only waits for `package` would ship without the sandbox verdict.
  assert.match(
    release,
    /needs: \[rust-audit, quality, package, clean-account-lifecycle\]/,
    "publish depends on the clean-account lifecycle gate",
  );
  assert.match(
    publish,
    /needs\.quality\.outputs\.tag == github\.ref_name/,
    "the ship guard binds the resolved tag, not a literal version",
  );
  assert.ok(
    !/needs\.quality\.outputs\.tag == '/.test(publish),
    "the ship guard must not compare the resolved tag against a literal",
  );
  // The workflow-gates policy must bind publish to the lifecycle gate too,
  // so the checker fails any regression that drops the dependency.
  const policy = JSON.parse(await readFile(join(process.cwd(), ".github", "workflow-gates.json"), "utf8"));
  const required = policy.workflows["release.yml"].publicationJobs.publish;
  assert.ok(
    required.includes("clean-account-lifecycle"),
    "workflow-gates policy binds publish to clean-account-lifecycle",
  );
  const result = validateWorkflowGates(await loadWorkflows(process.cwd()), policy);
  assert.equal(result.ok, true, result.failures.join("\n"));
});

test("DC-04.V2 the download seam is loopback-only and verifier-gated", async () => {
  const source = await readFile(join(process.cwd(), "src-tauri", "src", "download.rs"), "utf8");
  // The packaged verifier points catalog downloads at a local fixture to run
  // the correct/incorrect-checksum legs against controlled bytes. The seam
  // must stay gated on the verifier profile and restricted to plain-HTTP
  // loopback so an authenticated download can never be redirected remotely.
  assert.match(source, /LOCALMOTIVE_VERIFY_ISOLATED_ROOT/);
  assert.match(source, /LOCALMOTIVE_HF_BASE/);
  assert.match(source, /http:\/\/127\.0\.0\.1:/);
  assert.match(source, /http:\/\/localhost:/);
  assert.match(source, /resolve_url_honors_the_loopback_fixture_inside_the_verifier_profile/);
});

test("S-26: the hardware probe runs only in its explicit qualification job", async () => {
  const hw = await readFile(
    join(process.cwd(), ".github", "workflows", "hardware-qualify.yml"),
    "utf8",
  );
  assert.match(
    hw,
    /inputs\.runtime_install_key != ''/,
    "the probe job is gated on an explicit runtime install key",
  );
  assert.match(
    hw,
    /qualify_managed_runtime_on_current_host/,
    "the probe job runs the ignored hardware probe explicitly",
  );
  assert.match(
    hw,
    /--ignored --nocapture/,
    "the probe is invoked with --ignored from the qualification job only",
  );
  // The diagnostic probe in core.rs carries no acceptance assertions and says so.
  const core = await readFile(join(process.cwd(), "src-tauri", "src", "core.rs"), "utf8");
  assert.match(core, /DIAGNOSTIC, not an acceptance test/, "the local-tree probe is labelled as a diagnostic");
  // No CI or release job silently runs the ignored probes.
  for (const file of ["ci.yml", "release.yml"]) {
    const body = await readFile(join(process.cwd(), ".github", "workflows", file), "utf8");
    assert.doesNotMatch(body, /--ignored/, `${file} must not run ignored probes`);
  }
});
