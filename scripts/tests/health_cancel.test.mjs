// Fixture legs for the health-cancellation contract (audit QD-03 V3): fast
// and slow progress shapes are driven through the same classifier the
// packaged verifier applies, including the refused and bounded-timeout
// diagnostics.
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  classifyHealthCancellation,
  PHASE_OBSERVATION_BOUND_SECONDS,
} from "../lib/health_cancel.mjs";

function stage(name, status, failureReason = null) {
  return { stage: name, status, failureReason };
}

function passingStages() {
  return [
    stage("runtime-identity", "PASS"),
    stage("bundle-readiness", "PASS"),
    stage("model-availability", "PASS"),
    stage("launch-readiness", "PASS"),
    stage("supervised-completion", "PASS", null),
    stage("throughput-observation", "PASS"),
    stage("cleanup", "PASS"),
  ];
}

function cancelledStages() {
  return [
    stage("runtime-identity", "PASS"),
    stage("bundle-readiness", "PASS"),
    stage("model-availability", "PASS"),
    stage("launch-readiness", "CANCELLED", "cancelled"),
    stage("supervised-completion", "SKIPPED", "cancelled"),
    stage("throughput-observation", "SKIPPED", "cancelled"),
    stage("cleanup", "PASS"),
  ];
}

test("slow fixture progress: an accepted cancel records the cancelled contract", () => {
  const record = classifyHealthCancellation({
    trigger: "cancelled-after-phase",
    accepted: true,
    firstPhase: "downloading",
    health: { passed: false, stages: cancelledStages() },
  });
  assert.equal(record.outcome, "cancelled-after-phase");
  assert.deepEqual(record.cancelled_stages, [
    "launch-readiness",
    "supervised-completion",
    "throughput-observation",
  ]);
  assert.equal(record.cancelled_after_phase, "downloading");
});

test("slow fixture progress: a cancelled run without attribution fails", () => {
  assert.throws(
    () =>
      classifyHealthCancellation({
        trigger: "cancelled-after-phase",
        accepted: true,
        health: { passed: false, stages: passingStages().map((entry) => ({ ...entry, failureReason: null })) },
      }),
    /did not attribute cancellation/,
  );
  assert.throws(
    () =>
      classifyHealthCancellation({
        trigger: "cancelled-after-phase",
        accepted: false,
        health: { passed: false, stages: cancelledStages() },
      }),
    /did not accept health cancellation/,
  );
  assert.throws(
    () =>
      classifyHealthCancellation({
        trigger: "cancelled-after-phase",
        accepted: true,
        health: { passed: true, stages: cancelledStages() },
      }),
    /reported success/,
  );
});

test("fast fixture progress: completion before the cancel is recorded, not failed", () => {
  const record = classifyHealthCancellation({
    trigger: "completed-before-cancel",
    health: { passed: true, stages: passingStages() },
  });
  assert.equal(record.outcome, "completed-before-cancel");
  assert.equal(record.passed, true);
  assert.equal(record.stage_count, 7);
});

test("fast fixture progress: a completed run without the full PASS contract fails", () => {
  const withFailure = passingStages().map((entry, index) =>
    index === 4 ? stage("supervised-completion", "FAIL") : entry,
  );
  assert.throws(
    () => classifyHealthCancellation({ trigger: "completed-before-cancel", health: { passed: false, stages: withFailure } }),
    /passing contract/,
  );
  assert.throws(
    () => classifyHealthCancellation({ trigger: "completed-before-cancel", health: { passed: true, stages: withFailure } }),
    /all seven stages PASS/,
  );
  assert.throws(
    () => classifyHealthCancellation({ trigger: "completed-before-cancel", health: { passed: true, stages: passingStages().slice(0, 6) } }),
    /seven-stage contract/,
  );
});

test("a refused cancellation that did not pass surfaces its reason", () => {
  assert.throws(
    () =>
      classifyHealthCancellation({
        trigger: "cancel-refused",
        error: "no health run is active",
        health: { passed: false, stages: cancelledStages() },
      }),
    /Cancellation was refused.*no health run is active/,
  );
});

test("a bounded wait with no observable phase is a bounded diagnostic failure", () => {
  assert.throws(
    () => classifyHealthCancellation({ trigger: "bounded-timeout" }),
    new RegExp(`no health progress phase within ${PHASE_OBSERVATION_BOUND_SECONDS} s`),
  );
  assert.throws(
    () => classifyHealthCancellation({ trigger: "bounded-timeout", bound_seconds: 30 }),
    /within 30 s/,
  );
  assert.throws(() => classifyHealthCancellation({ trigger: "surprising" }), /Unknown cancellation outcome/);
  assert.throws(() => classifyHealthCancellation(undefined), /Unknown cancellation outcome/);
});
