// G-10.V1 traceability and link validation for the v0.6 record.
//
// Checks (all mechanical, no judgement):
//   1. Every in-document `](#anchor)` link in TODO-0.6.md resolves to a
//      heading in the same file (GitHub slug rules, duplicate suffixes).
//   2. Every cross-file link into the comprehensive audit resolves to a
//      heading of that document.
//   3. Every open (`- [ ]`) task id in the tracker appears in
//      docs/RELEASE-REVIEW-0.6.md (each open item keeps a disposition row).
//   4. Every `V06-` id cited in the review file exists in the tracker
//      (no phantom register rows).
//   5. No task id appears both checked and unchecked (lost/duplicated work).
//
// Usage: node scripts/verify_tracker_links.mjs
import { readFileSync } from "node:fs";
import { join } from "node:path";

const root = process.cwd();
const trackerPath = join(root, "docs", "history", "TODO-0.6.md");
const reviewPath = join(root, "docs", "RELEASE-REVIEW-0.6.md");
const auditPath = join(root, "docs", "history", "localmotive-comprehensive-audit.md");

const slugify = (heading) =>
  heading
    .trim()
    .toLowerCase()
    .replace(/[`*_~[\]]/g, "")
    .replace(/[^\p{L}\p{N}\s-]/gu, "")
    .trim()
    .replace(/\s+/g, "-");

const anchorsOf = (text) => {
  const seen = new Map();
  const anchors = new Set();
  for (const line of text.split(/\r?\n/)) {
    const match = /^(#{1,6})\s+(.*?)\s*$/.exec(line);
    if (!match) continue;
    const base = slugify(match[2]);
    const count = seen.get(base) ?? 0;
    seen.set(base, count + 1);
    anchors.add(count === 0 ? base : `${base}-${count}`);
  }
  return anchors;
};

const failures = [];
const note = (message) => failures.push(message);

const tracker = readFileSync(trackerPath, "utf8");
const review = readFileSync(reviewPath, "utf8");
const audit = readFileSync(auditPath, "utf8");

const trackerAnchors = anchorsOf(tracker);
const auditAnchors = anchorsOf(audit);

// 1 + 2: link resolution.
const linkPattern = /\]\(([^)\s]+)\)/g;
let linkCount = 0;
for (const [docName, text] of [
  ["TODO-0.6.md", tracker],
  ["RELEASE-REVIEW-0.6.md", review],
]) {
  for (const match of text.matchAll(linkPattern)) {
    const target = match[1];
    if (target.startsWith("http") || target.startsWith("mailto:")) continue;
    linkCount += 1;
    const [filePart, anchor] = target.split("#");
    if (filePart === "" || filePart === undefined) {
      if (anchor && !trackerAnchors.has(anchor)) {
        note(`${docName}: dead in-document anchor #${anchor}`);
      }
      continue;
    }
    const resolved = join(docName === "TODO-0.6.md" ? join(root, "docs", "history") : join(root, "docs"), filePart);
    let targetText;
    try {
      targetText = readFileSync(resolved, "utf8");
    } catch {
      note(`${docName}: link target missing: ${target}`);
      continue;
    }
    if (anchor) {
      const targetAnchors = filePart.endsWith("localmotive-comprehensive-audit.md")
        ? auditAnchors
        : anchorsOf(targetText);
      if (!targetAnchors.has(anchor)) {
        note(`${docName}: dead anchor ${target}`);
      }
    }
  }
}

// 3: every open box keeps a disposition row in the review.
const taskIds = (text, state) =>
  [...text.matchAll(new RegExp(`^- \\[${state}\\] \\*\\*(V06-[A-Za-z0-9.-]+)\\*\\*`, "gm"))].map(
    (match) => match[1],
  );
const checked = taskIds(tracker, "x");
const open = taskIds(tracker, " ");
for (const id of open) {
  if (!review.includes(id)) {
    note(`open task ${id} has no row in RELEASE-REVIEW-0.6.md`);
  }
}

// 4: review register ids exist in the tracker.
const trackerIds = new Set([...checked, ...open]);
const reviewIds = new Set(
  [...review.matchAll(/\bV06-[A-Za-z0-9]+(?:-[A-Za-z0-9]+)*(?:\.[A-Za-z0-9]+)*/g)].map((m) => m[0]),
);
for (const id of reviewIds) {
  if (!trackerIds.has(id)) {
    note(`review cites ${id} which does not exist as a tracker task line`);
  }
}

// 5: no id is both checked and open.
for (const id of checked) {
  if (open.includes(id)) {
    note(`task ${id} appears both checked and unchecked`);
  }
}

console.log(`links checked: ${linkCount}`);
console.log(`checked tasks: ${checked.length}; open tasks: ${open.length}`);
console.log(`review register ids: ${reviewIds.size}`);
if (failures.length === 0) {
  console.log("TRACEABILITY OK");
  process.exit(0);
}
for (const failure of failures.slice(0, 40)) console.log(`FAIL ${failure}`);
console.log(`TRACEABILITY FAIL (${failures.length} findings)`);
process.exit(1);
