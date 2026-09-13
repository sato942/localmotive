// F9-01: verdict rules for the packaged cancellation/restart proof.
//
// The reviewed driver could not establish what it claimed: it clicked the UI
// Run button before issuing its direct API attempt (so a refusal of the second
// call could be caused by the replacement the first click had just started),
// it accepted activity observed at cancellation or helper entry as proof that
// the previous request was active at the invocation, and its "newest record"
// selection could substitute a retry's record for the original one.
//
// These rules are pure so they can be tested against synthetic scenarios,
// including the two required negative controls: prohibited backend overlap and
// substitution of an unrelated saved record.

/**
 * Classify whether the active-request boundary was actually exercised.
 * Coverage is `active` only when the previous request was observed in flight
 * immediately before the replacement invocation. An attempt made after the
 * old request already ended is `not-exercised`: neither proof of a product
 * failure nor a passing test of the boundary.
 */
export function classifyCoverage({ processingAtCancel, processingBeforeInvocation }) {
  if (processingBeforeInvocation === "1") {
    return {
      coverage: "active",
      detail: `requests_processing=1 immediately before the invocation (cancel sample ${processingAtCancel})`,
    };
  }
  return {
    coverage: "not-exercised",
    detail: `requests_processing=${processingBeforeInvocation} before the invocation; the previous request was active at cancellation=${processingAtCancel === "1"}`,
  };
}

/**
 * Evaluate one replacement scenario. `scenario` is `ui` (a click on the Run
 * control) or `api` (a direct command invocation with no preceding UI attempt).
 * A refusal is only credited from the surface the scenario actually exercises:
 * for `api` the UI state is context, never the refusal.
 */
export function evaluateScenario({
  scenario,
  coverage,
  uiStateBefore = null,
  uiAccepted = null,
  apiOutcome = null,
  overlapObserved = false,
  serializationProved = false,
}) {
  if (overlapObserved) {
    return {
      status: "FAIL",
      detail: `${scenario}: replacement work ran concurrently with the previous request`,
    };
  }
  if (coverage !== "active") {
    return {
      status: "NOT-EXERCISED",
      detail: `${scenario}: the previous request had already ended before the invocation`,
    };
  }
  const refused =
    scenario === "api"
      ? apiOutcome === "refused"
      : uiAccepted === false || uiStateBefore === "disabled";
  if (refused) {
    return {
      status: "PASS",
      detail:
        scenario === "api"
          ? `api=${apiOutcome} while the previous request was active`
          : `ui=${uiAccepted === false ? "refused" : uiStateBefore} while the previous request was active`,
    };
  }
  if (serializationProved) {
    return {
      status: "PASS",
      detail: `${scenario}: replacement was safely serialized until the previous request terminated`,
    };
  }
  return {
    status: "FAIL",
    detail:
      scenario === "api"
        ? `api=${apiOutcome} accepted replacement work while the previous request was active`
        : `ui=${uiAccepted === true ? "accepted" : uiStateBefore} accepted replacement work while the previous request was active`,
  };
}

/**
 * Reject a saved record that is not the run this scenario observed. The
 * driver must never report a retry's record as the original run's outcome, and
 * an unrelated record must never satisfy the scenario.
 */
export function evaluateIdentity({ originalRecordPath, selectedRecordPath, knownRecordPaths = [] }) {
  if (!selectedRecordPath) {
    return { ok: false, detail: "no saved record was selected for this scenario" };
  }
  if (!originalRecordPath) {
    return { ok: false, detail: "the original run's record identity was never captured" };
  }
  if (selectedRecordPath !== originalRecordPath) {
    return {
      ok: false,
      detail: `the reported record ${selectedRecordPath} is not the original run ${originalRecordPath}`,
    };
  }
  if (!knownRecordPaths.includes(selectedRecordPath)) {
    return {
      ok: false,
      detail: `${selectedRecordPath} is not one of the records this scenario produced`,
    };
  }
  return { ok: true, detail: `record identity preserved: ${selectedRecordPath}` };
}

/**
 * Configurations that appear in a passing run but are prohibited: a retry
 * replacing the original assertion, or two accepted runs overlapping.
 */
export function evaluateProhibited({ originalRecordPath, retryRecordPath, overlapObserved }) {
  if (overlapObserved) {
    return { ok: false, detail: "backend overlap was observed" };
  }
  if (retryRecordPath && retryRecordPath === originalRecordPath) {
    return {
      ok: false,
      detail: "the retry reused the original run's record identity",
    };
  }
  return { ok: true, detail: "no prohibited configuration observed" };
}
