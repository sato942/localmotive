#!/usr/bin/env node
// Structural validation for docs/history/TODO-0.6.md and its audit source.
//
// This checks tracker *structure* only: finding coverage, priority totals,
// unique checkbox IDs and resolvable cross-document anchors. It never proves
// that an implementation checkbox was actually verified -- that evidence lives
// in the tracker's verification ledger. Exit code 1 on any structural defect.

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const auditPath = join(root, "docs", "history", "localmotive-comprehensive-audit.md");
const trackerPath = join(root, "docs", "history", "TODO-0.6.md");

const audit = readFileSync(auditPath, "utf8");
const tracker = readFileSync(trackerPath, "utf8");

const failures = [];
const assert = (condition, message) => {
  if (!condition) failures.push(message);
};

// GitHub-style anchor for a heading: lower case, drop punctuation, spaces -> -.
const anchorOf = (heading) =>
  heading
    .toLowerCase()
    .replace(/[^\w\s-]/g, "")
    .trim()
    .replace(/\s+/g, "-");

const headings = (text) =>
  text
    .split(/\r?\n/)
    .filter((line) => /^#{1,6} /.test(line))
    .map((line) => line.replace(/^#{1,6} /, ""));

const auditAnchors = new Set(headings(audit).map(anchorOf));
const trackerAnchors = new Set(headings(tracker).map(anchorOf));

// 1. Audit finding headers: exactly 72 unique IDs, priority totals 19/43/10.
const findingPattern = /^### ([A-Z]{2,3}-\d{2})$/gm;
const auditFindings = [];
for (const match of audit.matchAll(findingPattern)) {
  auditFindings.push({ id: match[1], index: match.index });
}
const auditIds = auditFindings.map((finding) => finding.id);
assert(auditIds.length === 72, `audit finding count is ${auditIds.length}, expected 72`);
assert(new Set(auditIds).size === auditIds.length, "audit finding IDs are not unique");

const priorityCounts = { High: 0, Medium: 0, Low: 0 };
for (let i = 0; i < auditFindings.length; i += 1) {
  const start = auditFindings[i].index;
  const end = i + 1 < auditFindings.length ? auditFindings[i + 1].index : audit.length;
  const block = audit.slice(start, end);
  const priority = block.match(/\*\*Priority: (High|Medium|Low)\.\*\*/);
  if (!priority) {
    failures.push(`audit finding ${auditFindings[i].id} has no priority line`);
  } else {
    priorityCounts[priority[1]] += 1;
  }
}
assert(
  priorityCounts.High === 19 && priorityCounts.Medium === 43 && priorityCounts.Low === 10,
  `audit priority totals are ${priorityCounts.High}/${priorityCounts.Medium}/${priorityCounts.Low}, expected 19/43/10`
);

// 2. Tracker finding packages: V06-<AREA>-<NN> (NN >= 01), one per audit ID.
const packagePattern = /^### (V06-[A-Z]+-\d{2})$/gm;
const packageIds = [...tracker.matchAll(packagePattern)].map((match) => match[1]);
assert(new Set(packageIds).size === packageIds.length, "tracker package IDs are not unique");

for (const id of auditIds) {
  const expected = `V06-${id}`;
  assert(packageIds.includes(expected), `audit finding ${id} has no tracker package ${expected}`);
}

// 3. Every task checkbox carries a unique ID of the form V06-...I<n>/.V<n>.
const checkboxPattern = /- \[[ x]\] \*\*(V06-[A-Z]+-\d{2}\.(?:I|V)\d+)\*\*/g;
const checkboxIds = [...tracker.matchAll(checkboxPattern)].map((match) => match[1]);
assert(checkboxIds.length > 0, "no checkboxes found");
assert(new Set(checkboxIds).size === checkboxIds.length, "checkbox IDs are not unique");

// Every package must expose at least one implementation and one verification box.
for (const id of packageIds) {
  const hasI = checkboxIds.some((box) => box.startsWith(`${id}.I`));
  assert(hasI, `package ${id} has no implementation checkbox`);
}

// 4. Cross-document anchors: tracker links into the audit must resolve there.
const auditLinkPattern = /\]\(\.\/localmotive-comprehensive-audit\.md#([\w.-]+)\)/g;
let auditLinkCount = 0;
for (const match of tracker.matchAll(auditLinkPattern)) {
  auditLinkCount += 1;
  assert(auditAnchors.has(match[1]), `tracker links to missing audit anchor #${match[1]}`);
}
assert(auditLinkCount > 0, "tracker has no audit links");

// 5. Tracker-internal anchors must resolve inside the tracker.
const internalPattern = /\]\(#([\w.-]+)\)/g;
for (const match of tracker.matchAll(internalPattern)) {
  assert(trackerAnchors.has(match[1]), `tracker links to missing internal anchor #${match[1]}`);
}

if (failures.length > 0) {
  for (const failure of failures) console.error(`FAIL: ${failure}`);
  process.exit(1);
}

console.log(
  `tracker structure OK: ${auditIds.length} findings (${priorityCounts.High}/${priorityCounts.Medium}/${priorityCounts.Low}), ` +
    `${packageIds.length} packages, ${checkboxIds.length} checkboxes, ${auditLinkCount} audit links`
);
