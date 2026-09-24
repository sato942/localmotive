// Shared IPC error shape test (P0-3, LAB-01): a failed Tauri command must
// record its real structured kind instead of "[object Object]". The page
// ships the raw rejection across CDP as JSON text; `serializeIpcError`
// normalizes the parsed value harness-side, and `formatIpcError` renders
// the LCD text with the kind and the detail.
import { test } from "node:test";
import assert from "node:assert/strict";
import { serializeIpcError, formatIpcError } from "../lib/ipc_errors.mjs";

// A Rust `RuntimeCatalogError` after a CDP JSON round-trip.
const RUST_SHAPE = {
  kind: "invalidResponse",
  message: "Selected adapter is not in the current hardware snapshot",
  retryAfterSeconds: null,
};

test("IPC errors: a Rust-shaped rejection keeps its kind, detail, and retry data", () => {
  const structured = serializeIpcError(JSON.parse(JSON.stringify(RUST_SHAPE)));
  assert.equal(structured.kind, "invalidResponse");
  assert.equal(
    structured.message,
    "Selected adapter is not in the current hardware snapshot",
  );
  assert.equal(structured.retryAfterSeconds, null);
});

test("IPC errors: the LCD text carries the kind and the detail", () => {
  const text = formatIpcError(serializeIpcError(RUST_SHAPE));
  assert.ok(!text.includes("[object Object]"), `kind lost: ${text}`);
  assert.ok(text.includes("invalidResponse"), `kind missing: ${text}`);
  assert.ok(
    text.includes("hardware snapshot"),
    `detail missing: ${text}`,
  );
});

test("IPC errors: a busy rejection keeps the busy kind", () => {
  const structured = serializeIpcError({
    kind: "busy",
    message: "A runtime catalog request is already active",
    retryAfterSeconds: null,
  });
  assert.equal(structured.kind, "busy");
  assert.ok(formatIpcError(structured).startsWith("[busy]"));
});

test("IPC errors: a plain-string rejection degrades to unknown kind", () => {
  const structured = serializeIpcError(JSON.parse(JSON.stringify("boom")));
  assert.equal(structured.kind, "unknown");
  assert.equal(structured.message, "boom");
  assert.equal(formatIpcError(structured), "[unknown] boom");
});

test("IPC errors: a missing rejection degrades without throwing", () => {
  for (const value of [null, undefined]) {
    const structured = serializeIpcError(value);
    assert.equal(structured.kind, "unknown");
    assert.equal(typeof structured.message, "string");
  }
});

test("IPC errors: a message-only object keeps its message with unknown kind", () => {
  const structured = serializeIpcError({ message: "plain failure" });
  assert.equal(structured.kind, "unknown");
  assert.equal(structured.message, "plain failure");
});
