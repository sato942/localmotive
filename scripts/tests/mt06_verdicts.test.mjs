// F9-01 controls: the verdict rules must reject the configurations the reviewed
// driver silently accepted, and must classify an already-inactive attempt as
// not exercised rather than as proof of the boundary.
import assert from "node:assert/strict";
import { test } from "node:test";

import {
  classifyCoverage,
  evaluateIdentity,
  evaluateProhibited,
  evaluateScenario,
} from "../lib/mt06_verdicts.mjs";

test("coverage is active only when the request is in flight immediately before the invocation", () => {
  assert.equal(
    classifyCoverage({ processingAtCancel: "1", processingBeforeInvocation: "1" }).coverage,
    "active",
  );
  // The reviewed driver accepted this shape as proof: active at cancellation,
  // already ended at the invocation. It must be classified, not credited.
  const ended = classifyCoverage({ processingAtCancel: "1", processingBeforeInvocation: "0" });
  assert.equal(ended.coverage, "not-exercised");
  assert.match(ended.detail, /active at cancellation=true/);
  assert.equal(
    classifyCoverage({ processingAtCancel: "0", processingBeforeInvocation: "0" }).coverage,
    "not-exercised",
  );
});

test("an already-inactive attempt is not exercised, not a failure and not a pass", () => {
  const verdict = evaluateScenario({
    scenario: "api",
    coverage: "not-exercised",
    uiStateBefore: "enabled",
    uiAccepted: true,
    apiOutcome: "refused",
  });
  assert.equal(verdict.status, "NOT-EXERCISED");
});

test("the API scenario is refused only by the API outcome, never by a disabled UI", () => {
  // Negative control: the reviewed predicate credited an accepted API attempt
  // whenever the UI state happened to be disabled.
  const accepted = evaluateScenario({
    scenario: "api",
    coverage: "active",
    uiStateBefore: "disabled",
    uiAccepted: null,
    apiOutcome: "accepted",
  });
  assert.equal(accepted.status, "FAIL");
  const refused = evaluateScenario({
    scenario: "api",
    coverage: "active",
    uiStateBefore: "enabled",
    apiOutcome: "refused",
  });
  assert.equal(refused.status, "PASS");
});

test("the UI scenario credits a refusal from the click or the disabled control", () => {
  assert.equal(
    evaluateScenario({
      scenario: "ui",
      coverage: "active",
      uiStateBefore: "disabled",
      uiAccepted: false,
    }).status,
    "PASS",
  );
  // An accepted click while the previous request is active is a failure unless
  // serialization is proved.
  const accepted = evaluateScenario({
    scenario: "ui",
    coverage: "active",
    uiStateBefore: "enabled",
    uiAccepted: true,
  });
  assert.equal(accepted.status, "FAIL");
  assert.equal(
    evaluateScenario({
      scenario: "ui",
      coverage: "active",
      uiStateBefore: "enabled",
      uiAccepted: true,
      serializationProved: true,
    }).status,
    "PASS",
  );
});

test("prohibited backend overlap is rejected as a negative control", () => {
  const verdict = evaluateScenario({
    scenario: "ui",
    coverage: "active",
    uiStateBefore: "enabled",
    uiAccepted: true,
    overlapObserved: true,
  });
  assert.equal(verdict.status, "FAIL");
  assert.match(verdict.detail, /concurrently/);
  assert.equal(evaluateProhibited({ overlapObserved: true }).ok, false);
});

test("substitution of an unrelated saved record is rejected as a negative control", () => {
  const original = "benchmark-1-technical-explanation-v1-1.json";
  const retry = "benchmark-2-technical-explanation-v1-1.json";
  assert.equal(
    evaluateIdentity({
      originalRecordPath: original,
      selectedRecordPath: retry,
      knownRecordPaths: [original, retry],
    }).ok,
    false,
  );
  assert.equal(
    evaluateIdentity({
      originalRecordPath: original,
      selectedRecordPath: "benchmark-3-technical-explanation-v1-1.json",
      knownRecordPaths: [original, retry],
    }).ok,
    false,
  );
  assert.equal(
    evaluateIdentity({
      originalRecordPath: original,
      selectedRecordPath: original,
      knownRecordPaths: [original, retry],
    }).ok,
    true,
  );
  assert.equal(
    evaluateProhibited({ originalRecordPath: original, retryRecordPath: original }).ok,
    false,
  );
  assert.equal(
    evaluateProhibited({ originalRecordPath: original, retryRecordPath: retry }).ok,
    true,
  );
});
