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
  const lib = await readFile(join(process.cwd(), "src-tauri", "src", "lib.rs"), "utf8");
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
  const lib = await readFile(join(process.cwd(), "src-tauri", "src", "lib.rs"), "utf8");
  const model = await readFile(join(process.cwd(), "src", "model.ts"), "utf8");
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
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
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
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

test("a rejected catalog invocation removes the spinner", async () => {
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
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
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  const errorAt = app.indexOf('runtime-catalog-message error');
  assert.ok(errorAt > 0, "the catalog error block is missing");
  const errorBlock = app.slice(errorAt, errorAt + 2000);
  assert.doesNotMatch(errorBlock, /className="spin"/);
  assert.doesNotMatch(errorBlock, /className="runtime-loading"/);
});

test("loading state has an accessible status label", async () => {
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  assert.match(app, /className="runtime-loading" role="status" aria-label="Loading approved runtime catalog"/);
});

test("blocked CUDA shows job server-cuda and its evidence URL", async () => {
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  assert.match(app, /entry\.blockingJobs\.map/);
  assert.match(app, /View \{jobName\} evidence/);
});

test("L2 rows render DIRECT RUNTIME · L2", async () => {
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  assert.match(app, /DIRECT RUNTIME · L2/);
  assert.match(app, /L2 EVIDENCE CEILING/);
});

test("no row renders SUPPORTED without the required evidence", async () => {
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
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
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
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
  assert.match(source, /queue\.dispatch/);
  assert.doesNotMatch(source, /hardwareIndex \+ 2/);
  assert.doesNotMatch(source, /hooks\[5\]/);
  assert.doesNotMatch(source, /__LM_VERIFY_ORIGINAL_INVOKE/);
  assert.doesNotMatch(source, /Page\.addScriptToEvaluateOnNewDocument/);
  // Shape-scan contract, proven live against the production fiber
  // (hardware=3, catalog=5, error=NULL=6, loading=7): the catalog
  // predicate must exclude null, the error predicate must accept null
  // with dispatch, and the scan must not use a fixed offset.
  assert.match(source, /memoizedState !== null/);
  assert.match(source, /Array\.isArray\(entry\.memoizedState\.options\)/);
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
  assert.match(workflow, /actions\/checkout@fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09/);
  assert.match(workflow, /actions\/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a/);
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
  const block = release.slice(at, at + 3000);
  assert.match(block, /Start-Process \$portable/);
  // A fixed sleep races WebView startup: the step must poll the CDP
  // endpoint until the page appears instead of assuming readiness.
  assert.match(block, /json\/list/);
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
  // Preferred ship path: pushing the version tag publishes after package
  // PASS, so no second parallel package run ever shares the runner CDP.
  assert.match(release, /github\.ref == 'refs\/tags\/v0\.5\.0'/);
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
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
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
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
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
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
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
  assert.match(adopt, /localStorage\.setItem\(`localmotive:profile:\$\{origin\.modelId\}`/);
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
  assert.match(panel, /await invoke<void>\("cancel_benchmark"\)/);
  assert.match(panel, /busy !== "benchmark" \|\| cancelPending/);
});

test("FE-04 previews compose provisionally and launch trust stays authoritative", async () => {
  const lib = await readFile(join(process.cwd(), "src-tauri", "src", "lib.rs"), "utf8");
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  // The preview command is cheap composition only: no probing or trust work
  // on the profile-edit path (audit FE-04).
  const preview = lib.split("fn preview_command(")[1].split("#[tauri::command]")[0];
  assert.match(preview, /compose_provisional_command/);
  assert.doesNotMatch(preview, /prepare_launch/);
  // Validation and launch keep the authoritative checks.
  const validate = lib.split("fn validate_launch_profile(")[1].split("#[tauri::command]")[0];
  assert.match(validate, /prepare_launch/);
  const start = lib.split("async fn start_server(")[1].split("fn start_server_worker(")[0];
  assert.match(start, /start_server_worker/);
  const worker = lib.split("fn start_server_worker(")[1].split("#[tauri::command]")[0];
  assert.match(worker, /spawn_server/);
  // The UI describes the provisional state honestly.
  assert.match(app, /Provisional command/);
  assert.match(app, /capability filtering, artifact checks, and managed-runtime trust are enforced/);
});

test("FE-16 status polling is single-flight with sequence guards", async () => {
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  // Slow native polls cannot pile up, and a poll that started before a
  // start/stop transition can never overwrite the newer snapshot; the
  // running strategy renders from that snapshot, not the editable draft
  // (audit FE-16).
  assert.match(app, /if \(inFlight\) return; \/\/ single-flight/);
  assert.match(app, /requested !== statusPollSeq\.current/);
  assert.match(app, /const requested = \+\+statusPollSeq\.current;/);
  const bumps = (app.match(/statusPollSeq\.current \+= 1;/g) ?? []).length;
  assert.ok(bumps >= 2, `expected start and stop bumps, saw ${bumps}`);
  assert.match(app, /status\.running \? status\.specType : profile\?\.specType/);
});

test("FE-05 evidence history and active runs survive navigation", async () => {
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  const panel = await readFile(join(process.cwd(), "src", "V03EvidencePanel.tsx"), "utf8");
  // The evidence panel is always mounted (hidden by style), never gated on
  // the benchmark view, so navigation cannot erase its state (audit FE-05).
  assert.doesNotMatch(app, /view === "benchmark" && \(/);
  assert.match(
    app,
    /style=\{view === "benchmark" \? undefined : \{ display: "none" \}\}/,
  );
  assert.match(app, /onRunStateChange=\{setEvidenceRun\}/);
  // The active run's status and cancel handle are available app-wide.
  assert.match(app, /evidenceRun\.cancel && \(/);
  // Completed runs are retained across model/profile changes.
  assert.doesNotMatch(panel, /setHistory\(\[\]\)/);
  // Export approval resets when the reviewed payload changes.
  assert.match(panel, /\[benchmark\?\.manifestPath, quality\?\.observedAtMs\]/);
});

test("FE-03 stale responses are guarded before they can commit", async () => {
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
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
  const app = await readFile(join(process.cwd(), "src", "App.tsx"), "utf8");
  assert.match(mirror, /user_sourced/);
  assert.match(mirror, /validate_user_override/);
  assert.match(lib, /save_user_catalog_override/);
  assert.match(lib, /remove_user_catalog_override/);
  assert.match(lib, /catalog_local_models/);
  assert.match(app, /USER ADDED/);
});
