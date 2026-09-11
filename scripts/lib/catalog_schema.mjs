// Shared catalog row validation (audit S-05). This module is the single
// JavaScript-side contract for catalog row acceptance; the Rust side
// (`catalog::parse_catalog`) implements the same rules and both read the
// shared fixture cases in scripts/tests/fixtures/catalog-schema-cases.json.
//
// Validators agree on: accepted rows, dropped rows with reasons, fatal
// duplicate model ids and fatal duplicate filenames. Dropped rows are never
// silent: the caller must surface the counts and reasons.

const SHA256 = /^[0-9a-fA-F]{64}$/;

// Per-row nested bounds, shared with the Rust contract (audit S-05/S-06).
export const MAX_MODEL_FILES = 64;
export const MAX_MODEL_TAGS = 128;
export const MAX_TAG_TEXT_LEN = 256;

export function isSafeFilename(name) {
  if (typeof name !== "string" || name.length === 0 || name.length > 255) {
    return false;
  }
  if (name.includes("/") || name.includes("\\") || name.includes(":")) {
    return false;
  }
  if (name !== name.trim() || name.endsWith(".") || name.endsWith(" ")) {
    return false;
  }
  const reserved = /^(con|prn|aux|nul|com[1-9]|lpt[1-9])(\.|$)/i;
  return !reserved.test(name) && !/[\u0000-\u001f]/.test(name);
}

export function isSafeRevision(value) {
  return (
    typeof value === "string" &&
    value.length > 0 &&
    value.length <= 160 &&
    !value.includes("..") &&
    /^[A-Za-z0-9._/-]+$/.test(value) &&
    value.split("/").every((segment) => segment.length > 0)
  );
}

export function isSha256(value) {
  return typeof value === "string" && SHA256.test(value);
}

export function isValidRepo(value) {
  if (typeof value !== "string" || value.length === 0 || value.length > 200) {
    return false;
  }
  const segments = value.split("/");
  return (
    segments.length === 2 &&
    segments.every(
      (segment) =>
        segment.length > 0 &&
        /^[A-Za-z0-9._-]+$/.test(segment) &&
        segment !== "." &&
        segment !== "..",
    )
  );
}

function rowProblem(row) {
  if (typeof row?.id !== "string" || row.id.length === 0) return "missing model id";
  if (!isValidRepo(row.repo)) return "invalid repository";
  if (!Array.isArray(row.files) || row.files.length === 0) return "no files";
  if (row.files.length > MAX_MODEL_FILES) {
    return `model has too many files (maximum ${MAX_MODEL_FILES})`;
  }
  if (Array.isArray(row.tags) && row.tags.length > MAX_MODEL_TAGS) {
    return `model has too many tags (maximum ${MAX_MODEL_TAGS})`;
  }
  if (Array.isArray(row.tags) && row.tags.some((tag) => typeof tag !== "string" || tag.length > MAX_TAG_TEXT_LEN)) {
    return "tag is too long";
  }
  for (const file of row.files) {
    if (!isSafeFilename(file?.filename)) return `unsafe filename: ${file?.filename}`;
    if (
      !Number.isSafeInteger(file?.sizeBytes) ||
      file.sizeBytes <= 0
    ) {
      return `invalid size for ${file.filename}`;
    }
    // Absent revision means the default branch, exactly like the Rust
    // deserializer's default_revision(); only an explicitly malformed
    // revision is a drop.
    const revision = file?.revision ?? "main";
    if (!isSafeRevision(revision)) return `unsafe revision for ${file.filename}`;
    if (!isSha256(file?.sha256)) return `invalid sha256 for ${file.filename}`;
    if (typeof file?.quant !== "string" || file.quant.length === 0) {
      return `missing quant label for ${file.filename}`;
    }
    for (const key of ["lastModified", "createdAt"]) {
      const value = file?.[key];
      if (typeof value === "string" && value.length > 0 && !/^\d{4}-\d{2}-\d{2}/.test(value)) {
        return `invalid date for ${file.filename}`;
      }
    }
  }
  return null;
}

// Drop reasons visible to the user, one per removed row (audit S-05.I2).
export function validateCatalogRows(models) {
  const accepted = [];
  const dropped = [];
  const seenIds = new Set();
  const seenRepos = new Set();
  const seenFiles = new Map();
  for (const row of Array.isArray(models) ? models : []) {
    const problem = rowProblem(row);
    if (problem) {
      dropped.push({ id: row?.id ?? "", repo: row?.repo ?? "", reason: problem });
      continue;
    }
    const id = row.id;
    if (seenIds.has(id)) {
      return { error: `duplicate model id: ${id}`, accepted: [], dropped };
    }
    seenIds.add(id);
    // One row per repository: a second row for the same repository would make
    // the files of the later row unreachable behind the first. The duplicate
    // is dropped with a visible reason instead.
    if (seenRepos.has(row.repo)) {
      dropped.push({
        id,
        repo: row.repo,
        reason: "duplicate repository row (first row stays authoritative)",
      });
      continue;
    }
    seenRepos.add(row.repo);
    for (const file of row.files) {
      const key = file.filename.toLowerCase();
      if (seenFiles.has(key)) {
        return {
          error: `model ${id} contains duplicate file: ${file.filename} (also in ${seenFiles.get(key)})`,
          accepted: [],
          dropped,
        };
      }
      seenFiles.set(key, id);
    }
    accepted.push(row);
  }
  // Mirrors the Rust validator: an input with rows that all failed validation
  // is an unusable catalog, not an empty one.
  if (accepted.length === 0 && (Array.isArray(models) ? models.length : 0) > 0) {
    const summary = dropped
      .slice(0, 3)
      .map((drop) => drop.reason)
      .join("; ");
    return {
      error: `Catalog contains no usable models. Dropped rows: ${summary}`,
      accepted,
      dropped,
    };
  }
  return { error: null, accepted, dropped };
}
