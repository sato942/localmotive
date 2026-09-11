/// The active evidence run's status and cancel handle, published by the
/// evidence panel and consumed by the control screen (audit FE-05).
export type EvidenceRun = {
  kind: "benchmark" | "quality";
  cancel: (() => void) | null;
};
