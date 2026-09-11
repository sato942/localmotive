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

const goodFile = (index) => ({
  filename: `file-${index}.gguf`,
  quant: "Q4_K_M",
  sizeBytes: 100 + index,
  revision: "main",
  sha256: "a".repeat(64),
});
const goodRow = (id, files = [goodFile(0)], tags = []) => ({
  id,
  repo: `fixture/${id}`,
  files,
  tags,
});

test("catalog schema: too many files in one row is dropped", () => {
  const files = Array.from({ length: 257 }, (_, index) => goodFile(index));
  const outcome = validateCatalogRows([goodRow("big", files)]);
  assert.ok(
    outcome.error?.includes("too many files"),
    `expected a too-many-files error, got: ${outcome.error}`,
  );
});

test("catalog schema: too many tags or an overlong tag is dropped", () => {
  const manyTags = Array.from({ length: 129 }, (_, index) => `tag-${index}`);
  const many = validateCatalogRows([goodRow("tagged", [goodFile(0)], manyTags)]);
  assert.ok(many.error?.includes("too many tags"), many.error);
  const long = validateCatalogRows([goodRow("longtag", [goodFile(0)], ["x".repeat(257)])]);
  assert.ok(long.error?.includes("tag is too long"), long.error);
});

test("catalog schema: many bounded rows are accepted, Unicode names are legal", () => {
  const rows = Array.from({ length: 150 }, (_, index) =>
    goodRow(`row-${index}`, [goodFile(index)]),
  );
  rows[0].files = [{ ...goodFile(1), filename: "模型-Q4_K_M.gguf" }];
  const outcome = validateCatalogRows(rows);
  assert.equal(outcome.error, null);
  assert.equal(outcome.accepted.length, 150);
  assert.equal(outcome.dropped.length, 0);
});
