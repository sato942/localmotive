// Rolling catalog recency window (audit QD-01).
//
// The builder previously derived its cutoff from a literal date, so every
// future build kept the same stale threshold while the emitted document
// claimed a rolling `cutoffDays` policy. The window is now a pure function
// of the build time.

/// The inclusive recency threshold: `now` minus `cutoffDays` days, at UTC
/// midnight. `now` defaults to the real clock at the call site.
export function catalogCutoff(now, cutoffDays) {
  const cutoff = new Date(now.getTime());
  cutoff.setUTCHours(0, 0, 0, 0);
  cutoff.setUTCDate(cutoff.getUTCDate() - cutoffDays);
  return cutoff;
}
