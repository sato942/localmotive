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
 *
 * The boundary proof is strictly bracketed: the previous request must be
 * observed in flight immediately before the replacement invocation AND the
 * immediate observation after the invocation must still show exactly that
 * one in-flight request.
 *
 * - before "1", after "1": `active`. The boundary was exercised.
 * - before "1", after a valid request count above "1": `overlap`. The
 *   immediate sample shows overlapping requests at the invocation itself.
 * - before "1", after "0": `not-exercised`. The request drained within the
 *   observation interval, so this proof cannot tell whether the replacement
 *   was refused while the request was active. A drained request is not
 *   overlapping requests.
 * - before "1", after null, missing, or an error result: `not-exercised`.
 *   The required observation is unavailable, and missing evidence never
 *   earns active coverage credit. An unreadable sample is not overlapping
 *   requests.
 * - before anything but "1": `not-exercised`. The previous request was
 *   already inactive before the invocation.
 *
 * Neither `not-exercised` outcome is proof of a product failure nor a
 * passing test of the boundary.
 */
export function classifyCoverage({ processingAtCancel, processingBeforeInvocation, processingAfterInvocation = null }) {
  if (processingBeforeInvocation !== "1") {
    return {
      coverage: "not-exercised",
      detail: `requests_processing=${processingBeforeInvocation} before the invocation; the previous request was active at cancellation=${processingAtCancel === "1"}`,
    };
  }
  if (processingAfterInvocation === "1") {
    return {
      coverage: "active",
      detail: `requests_processing=1 immediately before the invocation (cancel sample ${processingAtCancel})`,
    };
  }
  const afterNumeric = Number(processingAfterInvocation);
  if (Number.isFinite(afterNumeric) && afterNumeric > 1) {
    return {
      coverage: "overlap",
      detail: `requests_processing=${processingAfterInvocation} immediately after the invocation while the previous request was active (cancel sample ${processingAtCancel})`,
    };
  }
  if (processingAfterInvocation === "0") {
    return {
      coverage: "not-exercised",
      detail: `requests_processing=1 before the invocation but 0 immediately after: the request drained within the observation interval, so this strictly bracketed proof was not exercised (cancel sample ${processingAtCancel})`,
    };
  }
  return {
    coverage: "not-exercised",
    detail: `the observation after the invocation is unavailable (${String(processingAfterInvocation)}), so the strictly bracketed proof was not exercised (cancel sample ${processingAtCancel})`,
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
  coverageDetail = null,
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
  if (coverage === "overlap") {
    return {
      status: "FAIL",
      detail: `${scenario}: the immediate observation after the invocation shows overlapping requests while the previous request was active`,
    };
  }
  if (coverage !== "active") {
    return {
      status: "NOT-EXERCISED",
      detail: `${scenario}: boundary proof not exercised (${coverageDetail ?? "the required bracketed observation is unavailable"})`,
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
