// Shared bounded HTTP retry policy for the catalog builder (audit S-10).
//
// Rules:
// - every request carries an explicit deadline (AbortSignal.timeout);
// - a 429 honors Retry-After when it is a small, sane value (numeric seconds
//   or an HTTP date), clamped into [RETRY_AFTER_MIN_MS, RETRY_AFTER_MAX_MS];
// - a malformed or extreme Retry-After falls back to bounded exponential
//   backoff;
// - the total wait for one logical request is bounded, so a hostile or
//   confused server cannot stall the builder indefinitely.

export const REQUEST_TIMEOUT_MS = 20_000;
export const RETRY_ATTEMPTS = 5;
export const RETRY_AFTER_MIN_MS = 1_000;
export const RETRY_AFTER_MAX_MS = 30_000;
export const TOTAL_WAIT_BUDGET_MS = 60_000;

/** Parse a Retry-After header into milliseconds, or null when unusable. */
export function retryAfterMs(header, nowMs = Date.now()) {
  if (typeof header !== "string") return null;
  const value = header.trim();
  if (/^\d+$/.test(value)) {
    const seconds = Number(value);
    if (!Number.isFinite(seconds) || seconds > 86_400) return null;
    return seconds * 1000;
  }
  // HTTP date form: only a sane future date is honored.
  const when = Date.parse(value);
  if (Number.isNaN(when)) return null;
  const delta = when - nowMs;
  if (delta <= 0 || delta > 86_400_000) return null;
  return delta;
}

/** The wait before the next attempt: Retry-After when usable, else backoff. */
export function retryDelayMs({ retryAfterHeader, attempt, nowMs = Date.now() }) {
  const fromHeader = retryAfterMs(retryAfterHeader, nowMs);
  if (fromHeader !== null) {
    return Math.min(
      RETRY_AFTER_MAX_MS,
      Math.max(RETRY_AFTER_MIN_MS, fromHeader),
    );
  }
  return Math.min(RETRY_AFTER_MAX_MS, 2000 * 2 ** attempt);
}

/**
 * Fetch with the bounded policy. `fetchImpl`, `sleep` and `now` are injectable
 * so tests can drive malformed headers, stalls and the budget without a
 * network. Returns the Response for the caller to validate.
 */
export async function fetchWithRetry(
  url,
  { headers = {}, fetchImpl = fetch, sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms)), now = Date.now, onRetry = null, deadlineMs = REQUEST_TIMEOUT_MS } = {},
) {
  let waited = 0;
  for (let attempt = 0; ; attempt += 1) {
    const response = await fetchImpl(url, {
      headers,
      // The deadline is injectable so tests can exercise the real abort
      // path in bounded time: AbortSignal.timeout unrefs its timer, and a
      // stalled-fetch test left pending on that timer alone can be cancelled
      // by the runner as 'event loop already resolved' (a real CI failure on
      // 2026-09-12). Production keeps the full 20 s deadline.
      signal: AbortSignal.timeout(deadlineMs),
    });
    if (response.status !== 429 || attempt >= RETRY_ATTEMPTS) {
      return response;
    }
    const delay = retryDelayMs({
      retryAfterHeader: response.headers?.get?.("retry-after") ?? null,
      attempt,
      nowMs: now(),
    });
    if (waited + delay > TOTAL_WAIT_BUDGET_MS) {
      return response;
    }
    waited += delay;
    onRetry?.(attempt + 1, delay);
    await sleep(delay);
  }
}
