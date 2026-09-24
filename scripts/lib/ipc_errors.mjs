// Shared IPC error shape (P0-3, LAB-01): normalize a Tauri command
// rejection into `{ kind, message, retryAfterSeconds }` without losing the
// structured kind. The packaged page ships the raw rejection across CDP as
// JSON text (CDP stringifies thrown objects into "[object Object]"); the
// harness parses that text and normalizes it here, so both `invoke` and
// `rejectedInvoke` share one serializer and one LCD format.

/**
 * Normalize a parsed IPC rejection into a structured error record.
 * Rust `RuntimeCatalogError` arrives as `{ kind, message,
 * retryAfterSeconds }`; plain-string rejections and missing values degrade
 * to `unknown` kind instead of throwing.
 */
export function serializeIpcError(value) {
  const source =
    value && typeof value === "object" && !Array.isArray(value) ? value : null;
  if (!source) {
    return {
      kind: "unknown",
      message: typeof value === "string" ? value : String(value),
      retryAfterSeconds: null,
    };
  }
  const kind = typeof source.kind === "string" ? source.kind : "unknown";
  const message =
    typeof source.message === "string"
      ? source.message
      : source.message == null
        ? "(no detail)"
        : String(source.message);
  const retryAfterSeconds =
    typeof source.retryAfterSeconds === "number"
      ? source.retryAfterSeconds
      : null;
  return { kind, message, retryAfterSeconds };
}

/**
 * Render the LCD text for a failed IPC call: the kind and the detail.
 */
export function formatIpcError(structured) {
  const record = serializeIpcError(structured);
  return `[${record.kind}] ${record.message}`;
}
