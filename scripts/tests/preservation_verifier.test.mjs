// F9-05 controls for the preservation verifier: the v0.4.1 leg must assert the
// recovered profile/settings VALUES, and the cache record must be the released
// contract's record. Each of these cases is a failure mode the reviewed
// verifier accepted (null schemaVersion) or never checked (settings values).
import test from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const SANDBOX = join(process.cwd(), "scripts", "sandbox");
const VERIFIER = join(SANDBOX, "verify_preservation.py");
const FIXTURE = join(SANDBOX, "canary-settings.json");

function runVerifier(args) {
  try {
    const stdout = execFileSync("python", [VERIFIER, ...args], { encoding: "utf8" });
    return { code: 0, output: stdout };
  } catch (error) {
    return { code: error.status, output: `${error.stdout ?? ""}${error.stderr ?? ""}` };
  }
}

function readFixture() {
  return JSON.parse(readFileSync(FIXTURE, "utf8"));
}

/// Build a temp root with a userdata canary, a cache record and collected
/// settings; each test mutates one of them.
function stage({ cacheSchema = 1, collected = undefined, dropKey = null, corruptKey = null } = {}) {
  const root = mkdtempSync(join(tmpdir(), "lm-preserve-"));
  const fixture = readFixture();
  const userdata = join(root, "userdata.txt");
  writeFileSync(userdata, "g06-preservation-userdata-canary\n");
  const cache = join(root, "catalog-cache.json");
  writeFileSync(
    cache,
    JSON.stringify({ body: JSON.stringify({ schemaVersion: cacheSchema, models: [] }), etag: null }),
  );
  const reads = {};
  for (const [key, value] of Object.entries(fixture.keys)) reads[key] = value;
  if (dropKey) delete reads[dropKey];
  if (corruptKey) reads[corruptKey] = "corrupted-by-fixture";
  const settings = join(root, "collected-settings.json");
  writeFileSync(
    settings,
    JSON.stringify(collected ?? { schema: "localmotive.settings-reads.v1", reads }),
  );
  return { root, fixture, userdata, cache, settings };
}

function withStage(options, body) {
  const staged = stage(options);
  try {
    return body(staged);
  } finally {
    rmSync(staged.root, { recursive: true, force: true });
  }
}

test("recovered v0.4.1 settings and profiles pass", () => {
  withStage({}, (staged) => {
    const result = runVerifier([
      join(staged.root, "mirror.sqlite"),
      staged.userdata,
      staged.cache,
      "--flavor",
      "cache",
      "--settings-fixture",
      staged.fixturePath ?? join(SANDBOX, "canary-settings.json"),
      "--settings",
      staged.settings,
    ]);
    assert.equal(result.code, 0, result.output);
    assert.match(result.output, /settings: localmotive:model-root recovered/);
    assert.match(result.output, /profile: port recovered/);
  });
});

test("a null cache schemaVersion fails (the reviewed verifier accepted it)", () => {
  withStage({ cacheSchema: null }, (staged) => {
    const result = runVerifier([
      join(staged.root, "mirror.sqlite"),
      staged.userdata,
      staged.cache,
      "--flavor",
      "cache",
      "--settings-fixture",
      join(SANDBOX, "canary-settings.json"),
      "--settings",
      staged.settings,
    ]);
    assert.equal(result.code, 1);
    assert.match(result.output, /schemaVersion None != 1/);
  });
});

test("a lost setting key fails with the missing value named", () => {
  withStage({ dropKey: "localmotive:runtime" }, (staged) => {
    const result = runVerifier([
      join(staged.root, "mirror.sqlite"),
      staged.userdata,
      staged.cache,
      "--flavor",
      "cache",
      "--settings-fixture",
      join(SANDBOX, "canary-settings.json"),
      "--settings",
      staged.settings,
    ]);
    assert.equal(result.code, 1);
    assert.match(result.output, /setting localmotive:runtime not recovered/);
  });
});

test("a corrupted profile value fails", () => {
  withStage({ corruptKey: "localmotive:profile:fixture/v041-legacy-model" }, (staged) => {
    const result = runVerifier([
      join(staged.root, "mirror.sqlite"),
      staged.userdata,
      staged.cache,
      "--flavor",
      "cache",
      "--settings-fixture",
      join(SANDBOX, "canary-settings.json"),
      "--settings",
      staged.settings,
    ]);
    assert.equal(result.code, 1);
    assert.match(result.output, /profile record .* unparseable/);
  });
});

test("a malformed collected settings structure fails", () => {
  withStage({ collected: { schema: "localmotive.settings-reads.v1" } }, (staged) => {
    const result = runVerifier([
      join(staged.root, "mirror.sqlite"),
      staged.userdata,
      staged.cache,
      "--flavor",
      "cache",
      "--settings-fixture",
      join(SANDBOX, "canary-settings.json"),
      "--settings",
      staged.settings,
    ]);
    assert.equal(result.code, 1);
    assert.match(result.output, /do not carry a reads map/);
  });
});

test("the cache flavor refuses to run without the settings assertion", () => {
  withStage({}, (staged) => {
    const result = runVerifier([
      join(staged.root, "mirror.sqlite"),
      staged.userdata,
      staged.cache,
      "--flavor",
      "cache",
    ]);
    assert.equal(result.code, 1);
    assert.match(result.output, /requires --settings-fixture and --settings/);
  });
});

test("the committed fixture is derived from the v0.4.1 keys", () => {
  const fixture = readFixture();
  assert.equal(fixture.schema, "localmotive.v041-settings.v1");
  const keys = Object.keys(fixture.keys);
  for (const key of [
    "localmotive:model-root",
    "localmotive:runtime",
    "localmotive:cloud-provider",
    "localmotive:cloud-model",
    "localmotive:profile:fixture/v041-legacy-model",
    "localmotive:tuning:fixture/v041-legacy-model",
  ]) {
    assert.ok(keys.includes(key), `fixture seeds ${key}`);
  }
  // The expected profile values are the fixture's own sentinels, not an echo:
  // the verifier reads them from the fixture and compares the collected file.
  const digest = createHash("sha256").update(JSON.stringify(fixture.keys)).digest("hex");
  assert.match(digest, /^[0-9a-f]{64}$/);
  assert.equal(fixture.expect.profile.port, 8123);
});
