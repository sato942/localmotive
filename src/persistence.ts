/// Browser-storage persistence helpers (audit FE-09), extracted from `App.tsx`
/// (audit FE-04): one coherent piece — record reads with upgrade fallback,
/// quarantine of unparseable records, failure-proof writes, and the
/// persistence-failure sentence. No imports; no component state.

export const readRecord = (key: string): string | null => {
  // Upgrades read records saved under the previous product prefix once.
  // New writes use the current prefix; the old value stays for downgrade.
  try {
    const current = localStorage.getItem(`localmotive:${key}`);
    if (current !== null) return current;
    return localStorage.getItem(`gguf-pilot:${key}`);
  } catch {
    return null;
  }
};

/// Move a record that cannot be parsed or validated out of the way
/// (audit FE-09): the raw text is kept under a quarantine key, the live
/// key is cleared, and the caller shows a notice instead of crashing.
export function quarantineRecord(key: string, raw: string) {
  try {
    localStorage.setItem(`localmotive:quarantine:${key}:${Date.now()}`, raw);
    localStorage.removeItem(`localmotive:${key}`);
    localStorage.removeItem(`gguf-pilot:${key}`);
  } catch {
    // Storage may be unavailable; the caller still falls back safely.
  }
}

/// Persist a record without letting a storage failure masquerade as an
/// operation failure (audit FE-09 I3): the operation's own result stays
/// visible and the caller reports the persistence problem separately.
export function persistRecord(key: string, value: string): boolean {
  try {
    localStorage.setItem(`localmotive:${key}`, value);
    return true;
  } catch {
    return false;
  }
}

/// The persistence-failure sentence a caller appends to its own notice so a
/// completed operation is never relabelled as failed (audit FE-09 I3).
export function persistenceFailureNote(thing: string): string {
  return `${thing} could not be saved to browser storage (unavailable or full) and will not survive a restart.`;
}

export const readSetting = (key: string): string => readRecord(key) ?? "";
