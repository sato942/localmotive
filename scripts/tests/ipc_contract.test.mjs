// Shared IPC contract test (audit S-18.V1): the SAME fixture the Rust test
// round-trips is validated against the TypeScript consumer expectations, so
// the two sides cannot silently disagree on a tested wire shape.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const fixture = JSON.parse(
  await readFile(new URL("./fixtures/ipc-contract.json", import.meta.url), "utf8"),
);

// key -> expected typeof per model.ts type declaration. `null` means the
// value must be either null or a string (optional/nullable fields).
const expectations = {
  tokenStatus: { configured: "boolean", masked: "string", cleanupNotice: "string" },
  commandPreview: {
    powerShell: "string",
    argv: "string",
    cmd: "nullable-string",
    cmdNotice: "nullable-string",
  },
  catalogDrop: { id: "string", repo: "string", reason: "string" },
};

test("IPC contract: fixture entries match the TypeScript wire expectations", () => {
  for (const [name, fields] of Object.entries(expectations)) {
    const entry = fixture.types[name];
    assert.ok(entry && typeof entry === "object", `${name} fixture entry must exist`);
    assert.deepEqual(
      Object.keys(entry).sort(),
      Object.keys(fields).sort(),
      `${name} must carry exactly the declared keys`,
    );
    for (const [key, kind] of Object.entries(fields)) {
      const value = entry[key];
      if (kind === "nullable-string") {
        assert.ok(
          value === null || typeof value === "string",
          `${name}.${key} must be a string or null, got ${typeof value}`,
        );
      } else {
        assert.equal(typeof value, kind, `${name}.${key} must be ${kind}`);
      }
      // camelCase only: a snake_case leak would mean the Rust struct is
      // missing its rename_all attribute.
      assert.doesNotMatch(key, /_/, `${name}.${key} must be camelCase`);
    }
  }
});

test("IPC contract: enums and record versions use their documented spellings", () => {
  assert.equal(fixture.types.externalEvidenceState, "pending");
  assert.equal(fixture.types.externalProvenance, "importedExternal");
  assert.equal(fixture.types.calibrationRecordVersion, 1);
});
