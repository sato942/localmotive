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
    classifyCoverage({ processingAtCancel: "1", processingBeforeInvocation: "1", processingAfterInvocation: "1" }).coverage,
    "active",
  );
  // The reviewed driver accepted this shape as proof: active at cancellation,
  // already ended at the invocation. It must be classified, not credited.
  const ended = classifyCoverage({ processingAtCancel: "1", processingBeforeInvocation: "0", processingAfterInvocation: "0" });
  assert.equal(ended.coverage, "not-exercised");
  assert.match(ended.detail, /active at cancellation=true/);
  assert.equal(
    classifyCoverage({ processingAtCancel: "0", processingBeforeInvocation: "0" }).coverage,
    "not-exercised",
  );
});

test("the immediate after-sample reaches the verdict: overlap at the invocation fails", () => {
  // Owner replay: the recorded runs carry processingAfterInvocation "1", and
  // changing only that sample to "2" must fail both scenarios through the
  // driver's actual data-to-verdict wiring. The wiring under test is
  // classifyCoverage (with the after-sample) followed by evaluateScenario.
  const wire = (scenario, after) => {
    const attempt = {
      scenario,
      processingAtCancelRequest: "1",
      processingBeforeInvocation: "1",
      processingAfterInvocation: after,
      buttonStateBefore: "enabled",
      uiAccepted: scenario === "ui" ? false : null,
      apiAttempt: scenario === "api" ? { status: "refused" } : null,
    };
    const coverage = classifyCoverage({
      processingAtCancel: attempt.processingAtCancelRequest,
      processingBeforeInvocation: attempt.processingBeforeInvocation,
      processingAfterInvocation: attempt.processingAfterInvocation,
    });
    const verdict = evaluateScenario({
      scenario,
      coverage: coverage.coverage,
      coverageDetail: coverage.detail,
      uiStateBefore: attempt.buttonStateBefore,
      uiAccepted: attempt.uiAccepted,
      apiOutcome: attempt.apiAttempt?.status ?? null,
      overlapObserved: false,
      serializationProved: false,
    });
    return { coverage, verdict };
  };
  for (const scenario of ["ui", "api"]) {
    const { coverage, verdict } = wire(scenario, "2");
    assert.equal(coverage.coverage, "overlap");
    assert.equal(verdict.status, "FAIL", `${scenario} must fail on immediate overlap`);
  }
  // A count above "1" is overlap; anything else bracketed is not.
  for (const scenario of ["ui", "api"]) {
    assert.equal(wire(scenario, "3").verdict.status, "FAIL");
  }
  // The recorded shape (after-sample "1") keeps passing through the same wiring.
  const recorded = classifyCoverage({
    processingAtCancel: "1",
    processingBeforeInvocation: "1",
    processingAfterInvocation: "1",
  });
  assert.equal(recorded.coverage, "active");
  assert.equal(
    evaluateScenario({
      scenario: "api",
      coverage: recorded.coverage,
      coverageDetail: recorded.detail,
      uiStateBefore: "enabled",
      uiAccepted: null,
      apiOutcome: "refused",
      overlapObserved: false,
      serializationProved: false,
    }).status,
    "PASS",
  );
});

test("a drained or unavailable after-sample is not exercised, never overlap and never active", () => {
  // Correction pass: "0" means the request drained within the observation
  // interval; null/missing/error means the required observation is
  // unavailable. Both are NOT-EXERCISED through the same classify-then-
  // evaluate wiring, for both scenarios. Missing evidence must not earn
  // active coverage credit, and zero must not be diagnosed as overlap.
  const wire = (scenario, after) => {
    const coverage = classifyCoverage({
      processingAtCancel: "1",
      processingBeforeInvocation: "1",
      ...(after === undefined ? {} : { processingAfterInvocation: after }),
    });
    const verdict = evaluateScenario({
      scenario,
      coverage: coverage.coverage,
      coverageDetail: coverage.detail,
      uiStateBefore: "enabled",
      uiAccepted: scenario === "ui" ? false : null,
      apiOutcome: scenario === "api" ? "refused" : null,
      overlapObserved: false,
      serializationProved: false,
    });
    return { coverage, verdict };
  };
  for (const scenario of ["ui", "api"]) {
    const drained = wire(scenario, "0");
    assert.equal(drained.coverage.coverage, "not-exercised");
    assert.match(drained.coverage.detail, /drained within the observation interval/);
    assert.equal(drained.verdict.status, "NOT-EXERCISED");
    assert.match(drained.verdict.detail, /drained within the observation interval/);
    for (const missing of [null, undefined, "ERR:timeout"]) {
      const result = wire(scenario, missing);
      assert.equal(result.coverage.coverage, "not-exercised", `${scenario} after=${String(missing)}`);
      assert.match(result.coverage.detail, /unavailable/);
      assert.equal(result.verdict.status, "NOT-EXERCISED");
      assert.match(result.verdict.detail, /unavailable/);
    }
  }
});

test("genuine drain-loop overlap still fails with an incomplete immediate sample", () => {
  // An incomplete sample must not override genuine overlap evidence observed
  // independently by the drain loop.
  for (const scenario of ["ui", "api"]) {
    const coverage = classifyCoverage({
      processingAtCancel: "1",
      processingBeforeInvocation: "1",
      processingAfterInvocation: "0",
    });
    assert.equal(coverage.coverage, "not-exercised");
    const verdict = evaluateScenario({
      scenario,
      coverage: coverage.coverage,
      coverageDetail: coverage.detail,
      uiStateBefore: "enabled",
      uiAccepted: true,
      apiOutcome: "accepted",
      overlapObserved: true,
      serializationProved: false,
    });
    assert.equal(verdict.status, "FAIL");
    assert.match(verdict.detail, /concurrently/);
  }
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
