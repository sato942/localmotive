// Shared catalog schema fixtures for both validators (audit S-05.V1).
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { validateCatalogRows } from "../lib/catalog_schema.mjs";

const fixtures = JSON.parse(
  await readFile(new URL("./fixtures/catalog-schema-cases.json", import.meta.url), "utf8"),
);

for (const testCase of fixtures.cases) {
  test(`catalog schema: ${testCase.name}`, () => {
    const outcome = validateCatalogRows(testCase.models);
    if (testCase.expect.result === "error") {
      assert.ok(outcome.error, `expected an error for ${testCase.name}`);
      assert.ok(
        outcome.error.includes(testCase.expect.errorContains),
        `error must contain ${JSON.stringify(testCase.expect.errorContains)}, got: ${outcome.error}`,
      );
      return;
    }
    assert.equal(outcome.error, null, `${testCase.name}: ${outcome.error}`);
    assert.deepEqual(
      outcome.accepted.map((row) => row.id),
      testCase.expect.acceptedIds,
      testCase.name,
    );
    assert.equal(outcome.dropped.length, testCase.expect.dropped, testCase.name);
    for (const reason of testCase.expect.reasonsContain ?? []) {
      assert.ok(
        outcome.dropped.some((drop) => drop.reason.includes(reason)),
        `${testCase.name}: dropped reasons must include ${JSON.stringify(reason)}: ${JSON.stringify(outcome.dropped)}`,
      );
    }
  });
}
