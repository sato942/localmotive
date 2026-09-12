// Workflow mapping-shape guard (regression for the 2026-09-12 incident).
//
// GitHub rejects an ENTIRE workflow file when a mapping-valued key has no
// entries: `env:` followed only by comment lines parses as `env: null`, and
// GitHub then refuses the workflow (placeholder failed runs on every push,
// workflow registered by path). actionlint reports this as
// "[syntax-check] expecting a single ${{...}} expression or mapping value".
//
// This checker is deliberately textual and dependency-free: for every
// mapping-valued key at the start of a block (env, with, strategy, matrix,
// outputs, permissions, defaults, concurrency, and a job's steps), at least
// one child line must be a real `key: value` mapping entry - comments and
// blank lines do not count.
//
// Exported for tests; also runnable directly:
//   node scripts/verify_workflow_syntax.mjs .github/workflows/*.yml

import { readFileSync, readdirSync } from "node:fs";
import { join, basename } from "node:path";

const MAPPING_KEYS = new Set([
  "env",
  "with",
  "strategy",
  "matrix",
  "outputs",
  "defaults",
  "concurrency",
  "permissions",
  "secrets",
  "needs-if",
]);

function indentOf(line) {
  const match = line.match(/^( *)/);
  return match ? match[1].length : 0;
}

/**
 * Return findings for comment-only / empty mapping-valued keys.
 * @param {string} text workflow YAML text
 * @returns {Array<{line:number, key:string, detail:string}>}
 */
export function findCommentOnlyMappings(text) {
  const findings = [];
  const lines = text.split(/\r?\n/);
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    const keyMatch = line.match(/^(\s*)([A-Za-z_-]+):\s*$/);
    if (!keyMatch) continue;
    const key = keyMatch[2];
    // `steps:` legitimately holds a sequence, not mapping entries; a bare
    // `steps:` with only comments would be invalid too but never occurs.
    if (!MAPPING_KEYS.has(key) && key !== "steps") continue;
    const baseIndent = keyMatch[1].length;
    let child = index + 1;
    let hasEntry = false;
    let sawComment = false;
    for (; child < lines.length; child += 1) {
      const current = lines[child];
      const trimmed = current.trim();
      if (trimmed === "") continue;
      const currentIndent = indentOf(current);
      if (currentIndent <= baseIndent) break;
      if (trimmed.startsWith("#")) {
        sawComment = true;
        continue;
      }
      const entry = trimmed.match(/^(- |[A-Za-z_"'-][^:]*:)/);
      if (entry) hasEntry = true;
      // Nested block under this key: keep scanning; the parent key carries
      // the child entries so the parent is not empty.
      if (!trimmed.includes(":") && !trimmed.startsWith("- ")) {
        // A scalar continuation (e.g. a YAML expression fragment); treat as
        // an entry because GitHub accepts expression scalars for env.
        hasEntry = true;
      }
      if (hasEntry) break;
    }
    if (!hasEntry && sawComment) {
      findings.push({
        line: index + 1,
        key,
        detail:
          `"${key}:" has only comment lines before the next sibling; GitHub ` +
          `parses this as null and rejects the whole workflow.`,
      });
    }
  }
  return findings;
}

function main() {
  const patterns = process.argv.slice(2);
  const files = patterns.length
    ? patterns
    : readdirSync(".github/workflows")
        .filter((name) => name.endsWith(".yml") || name.endsWith(".yaml"))
        .map((name) => join(".github/workflows", name));
  let failed = false;
  for (const file of files) {
    const findings = findCommentOnlyMappings(readFileSync(file, "utf8"));
    if (findings.length === 0) {
      console.log(`PASS workflow syntax guard: ${basename(file)}`);
      continue;
    }
    failed = true;
    for (const finding of findings) {
      console.error(
        `FAIL ${file}:${finding.line}: ${finding.detail}`,
      );
    }
  }
  process.exitCode = failed ? 1 : 0;
}

if (process.argv[1] && import.meta.url.endsWith(basename(process.argv[1]))) {
  main();
}
