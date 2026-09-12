// Bounded HTTP retry policy tests (audit S-10.V1): numeric and date
// Retry-After values, malformed and extreme delays, budget exhaustion and a
// stalled fetch aborted by the request deadline.
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  fetchWithRetry,
  retryAfterMs,
  retryDelayMs,
  RETRY_AFTER_MAX_MS,
  RETRY_AFTER_MIN_MS,
  TOTAL_WAIT_BUDGET_MS,
} from "../lib/http_retry.mjs";

test("retry-after: numeric seconds parse and clamp", () => {
  assert.equal(retryAfterMs("5"), 5000);
  assert.equal(retryDelayMs({ retryAfterHeader: "0", attempt: 0 }), RETRY_AFTER_MIN_MS);
  assert.equal(retryDelayMs({ retryAfterHeader: "600", attempt: 0 }), RETRY_AFTER_MAX_MS);
});

test("retry-after: a sane HTTP date is honored, stale or extreme dates are not", () => {
  const now = Date.UTC(2026, 8, 11, 12, 0, 0);
  const soon = new Date(now + 4000).toUTCString();
  assert.equal(retryAfterMs(soon, now), 4000);
  assert.equal(retryAfterMs(new Date(now - 5000).toUTCString(), now), null);
  assert.equal(retryAfterMs(new Date(now + 3 * 86_400_000).toUTCString(), now), null);
});

test("retry-after: malformed and hostile values fall back to bounded backoff", () => {
  for (const value of ["", "soon", "1e99", "-5", "999999999", null, undefined]) {
    assert.equal(retryAfterMs(value), null, JSON.stringify(value));
  }
  assert.equal(retryDelayMs({ retryAfterHeader: "junk", attempt: 0 }), 2000);
  assert.equal(retryDelayMs({ retryAfterHeader: "junk", attempt: 1 }), 4000);
  assert.equal(retryDelayMs({ retryAfterHeader: "junk", attempt: 9 }), RETRY_AFTER_MAX_MS);
});

test("fetch retries a 429 with the header delay and stops at success", async () => {
  const waits = [];
  const responses = [
    { status: 429, headers: { get: () => "2" } },
    { status: 200, headers: { get: () => null } },
  ];
  let calls = 0;
  const fetchImpl = async () => responses[calls++];
  const result = await fetchWithRetry("https://example.test/x", {
    fetchImpl,
    sleep: async (ms) => waits.push(ms),
  });
  assert.equal(result.status, 200);
  assert.deepEqual(waits, [2000]);
  assert.equal(calls, 2);
});

test("fetch gives up at the attempts limit and the total budget is bounded", async () => {
  let calls = 0;
  const fetchImpl = async () => {
    calls += 1;
    return { status: 429, headers: { get: () => "600" } };
  };
  const waits = [];
  const result = await fetchWithRetry("https://example.test/x", {
    fetchImpl,
    sleep: async (ms) => waits.push(ms),
  });
  assert.equal(result.status, 429, "the last 429 is returned to the caller");
  assert.ok(calls <= 6, `attempts bounded, saw ${calls}`);
  const total = waits.reduce((sum, value) => sum + value, 0);
  assert.ok(total <= TOTAL_WAIT_BUDGET_MS, `total wait bounded: ${total}`);
});

test("a stalled fetch is aborted by the request deadline", async () => {
  // A fetch implementation that never settles on its own but rejects when
  // the signal aborts: the deadline must fire, not hang the builder. The
  // deadline is injected so the test finishes in bounded time with the real
  // AbortSignal.timeout path; a stalled promise left pending on the
  // unref'd default timer was cancelled by the CI runner ("event loop has
  // already resolved") on 2026-09-12, so the default value is never the
  // test's only clock.
  const fetchImpl = (url, options) =>
    new Promise((_, reject) => {
      options.signal.addEventListener("abort", () =>
        reject(new Error("The operation was aborted")),
      );
    });
  const keeper = setTimeout(() => {}, 1000);
  // AbortSignal.timeout's timer is unref'd: pending on it alone, the runner
  // can cancel the promise when the event loop drains ("cancelledByParent"
  // in CI). A referenced keeper timer holds the loop past the deadline, so
  // the abort is guaranteed to fire before the loop is allowed to resolve.
  try {
    await assert.rejects(
      fetchWithRetry("https://example.test/stall", { fetchImpl, deadlineMs: 200 }),
      /abort/i,
    );
  } finally {
    clearTimeout(keeper);
  }
});
