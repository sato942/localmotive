// Shared contract for the packaged health-cancellation check (audit QD-03
// V3 / I4).
//
// The packaged check observes a real managed health run through the console
// and reports one of a small set of terminal outcomes. This module owns the
// acceptance rules for those outcomes so fixture tests can drive fast and
// slow progress shapes through the exact same decisions the packaged
// verifier applies:
//
// - slow progress: an active operation was observable, the backend accepted
//   the cancellation, and the run returned the bounded cancelled contract;
// - fast progress: the run completed before the cancel could land; completion
//   is a valid observed outcome and must carry the full passing contract;
// - refused cancellation or a bounded wait with no observable phase: a
//   bounded diagnostic failure, never a silent pass.

export const PHASE_OBSERVATION_BOUND_SECONDS = 120;

/** @returns the normalized evidence record for `classifyHealthCancellation`. */
export function classifyHealthCancellation(outcome) {
  const health = outcome?.health ?? null;
  const stages = Array.isArray(health?.stages) ? health.stages : null;
  switch (outcome?.trigger) {
    case "completed-before-cancel":
      require(
        health?.passed === true,
        "A run that completed before cancellation must report its full passing contract",
      );
      require(
        stages !== null && stages.length === 7,
        "The completed run must keep the seven-stage contract",
      );
      require(
        stages.every((stage) => stage.status === "PASS"),
        "The completed run must show all seven stages PASS",
      );
      return { outcome: "completed-before-cancel", passed: true, stage_count: stages.length };
    case "cancelled-after-phase": {
      require(
        outcome.accepted === true,
        "The backend did not accept health cancellation",
      );
      require(health?.passed === false, "The cancelled health run reported success");
      require(
        stages !== null && stages.length === 7,
        "The cancelled run must keep the seven-stage contract",
      );
      const cancelled = stages
        .filter((stage) => stage.failureReason === "cancelled")
        .map((stage) => stage.stage);
      require(cancelled.length >= 1, "The cancelled run did not attribute cancellation to a stage");
      return {
        outcome: "cancelled-after-phase",
        cancelled_stages: cancelled,
        cancelled_after_phase: outcome.firstPhase ?? null,
      };
    }
    case "cancel-refused":
      require(
        health?.passed !== true,
        "A refused cancellation with a reportedly passing run must be re-checked",
      );
      throw new Error(
        `Cancellation was refused and the run did not report success: ${outcome.error ?? "no detail"}`,
      );
    case "bounded-timeout":
      throw new Error(
        `Bounded diagnostic: no health progress phase within ${outcome.bound_seconds ?? PHASE_OBSERVATION_BOUND_SECONDS} s`,
      );
    default:
      throw new Error(
        `Unknown cancellation outcome: ${JSON.stringify(outcome?.trigger ?? null)}`,
      );
  }
}

function require(condition, message) {
  if (!condition) throw new Error(message);
}
