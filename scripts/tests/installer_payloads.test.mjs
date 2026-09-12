// Installer payload identity tests: the bundler marker investigation is a
// measured claim, so its comparison logic is pinned here. The main campaign
// runs the real extraction against the staged installers and records the
// evidence document; these tests prove the comparison itself can fail.
import { test } from "node:test";
import assert from "node:assert/strict";
import { compareBundlePayloads } from "../verify_installer_payloads.mjs";

const BASE = Buffer.from(`prefix: TYPE_VAR_UNK suffix payload ${"x".repeat(64)}`, "latin1");
const markerIndex = BASE.indexOf("UNK");

function withMarker(label) {
  const copy = Buffer.from(BASE);
  copy.write(label, markerIndex, "latin1");
  return copy;
}

test("a payload that differs only by the bundle-type marker passes", () => {
  const nsis = compareBundlePayloads(BASE, withMarker("NSS"), "nsis");
  assert.equal(nsis.ok, true, nsis.reason);
  assert.equal(nsis.portableMarker, "UNK");
  assert.equal(nsis.payloadMarker, "NSS");
  const msi = compareBundlePayloads(BASE, withMarker("MSI"), "msi");
  assert.equal(msi.ok, true, msi.reason);
});

test("a marker that does not belong to the bundle type fails", () => {
  const swapped = compareBundlePayloads(BASE, withMarker("NSS"), "msi");
  assert.equal(swapped.ok, false);
  assert.match(swapped.reason, /unexpected marker bytes/u);
  const foreign = compareBundlePayloads(BASE, withMarker("XYZ"), "nsis");
  assert.equal(foreign.ok, false);
});

test("a payload that differs anywhere else fails instead of being excused", () => {
  const doctored = withMarker("NSS");
  doctored[doctored.length - 1] = 0x21;
  const result = compareBundlePayloads(BASE, doctored, "nsis");
  assert.equal(result.ok, false);
  assert.match(result.reason, /expected exactly the 3-byte bundle-type marker/u);
  assert.equal(result.diffPositions.length, 4);
});

test("a size difference fails before any byte walk", () => {
  const result = compareBundlePayloads(BASE, BASE.subarray(0, 10), "nsis");
  assert.equal(result.ok, false);
  assert.match(result.reason, /differs from the portable size/u);
});

test("a portable executable that is not the unbundled build fails", () => {
  const result = compareBundlePayloads(withMarker("NSS"), withMarker("NSS"), "nsis");
  assert.equal(result.ok, false);
  assert.match(result.reason, /expected exactly the 3-byte bundle-type marker/u);
});
