#!/usr/bin/env node
import { readFile, readdir } from "node:fs/promises";
import { basename, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { parse } from "yaml";

const FULL_SHA = /^[0-9a-f]{40}$/;

export function validatePinnedUses(usesEntries, records) {
  const failures = [];
  const usedRecords = new Set();
  for (const entry of usesEntries) {
    if (entry.uses.startsWith("./")) continue;
    const separator = entry.uses.lastIndexOf("@");
    const action = separator < 0 ? entry.uses : entry.uses.slice(0, separator);
    const sha = separator < 0 ? "" : entry.uses.slice(separator + 1);
    if (!FULL_SHA.test(sha)) {
      failures.push(`${entry.file}:${entry.job}: step ${entry.step} must pin ${entry.uses} to a full 40-character commit SHA`);
      continue;
    }
    const recordIndex = records.findIndex((record) => record.action === action && record.sha === sha);
    if (recordIndex < 0) {
      failures.push(`${entry.file}:${entry.job}: ${action}@${sha} has no reviewed action record`);
      continue;
    }
    const record = records[recordIndex];
    usedRecords.add(recordIndex);
    if (typeof record.reviewedRef !== "string" || !record.reviewedRef.trim()) {
      failures.push(`action record ${action}@${sha} has no reviewedRef`);
    }
    if (record.sourceUrl !== `https://github.com/${action}/commit/${sha}`) {
      failures.push(`action record ${action}@${sha} has an unexpected sourceUrl`);
    }
    if (record.tagObjectSha !== undefined) {
      if (!FULL_SHA.test(record.tagObjectSha)) {
        failures.push(`action record ${action}@${sha} has a malformed tagObjectSha`);
      } else if (record.tagObjectSha === sha) {
        failures.push(`action record ${action}@${sha} uses an annotated tag object instead of its peeled commit`);
      }
    }
  }
  records.forEach((record, index) => {
    if (!usedRecords.has(index)) failures.push(`action record ${record.action}@${record.sha} is not used by a workflow`);
  });
  return { ok: failures.length === 0, entries: usesEntries, failures };
}

export async function loadWorkflowUses(root) {
  const directory = resolve(root, ".github", "workflows");
  const files = (await readdir(directory)).filter((file) => /\.ya?ml$/i.test(file)).sort();
  const entries = [];
  for (const file of files) {
    const workflow = parse(await readFile(resolve(directory, file), "utf8"));
    for (const [job, definition] of Object.entries(workflow?.jobs ?? {})) {
      for (const [step, value] of (definition?.steps ?? []).entries()) {
        if (typeof value?.uses === "string") entries.push({ file, job, step, uses: value.uses });
      }
    }
  }
  return entries;
}

async function main() {
  const root = process.cwd();
  const records = JSON.parse(await readFile(resolve(root, ".github", "workflow-actions.json"), "utf8"));
  if (!Array.isArray(records)) throw new Error(".github/workflow-actions.json must contain an array");
  const result = validatePinnedUses(await loadWorkflowUses(root), records);
  console.log(JSON.stringify(result, null, 2));
  if (!result.ok) process.exitCode = 1;
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  main().catch((error) => {
    console.error(`workflow-pin-gate: ${error.message}`);
    process.exitCode = 1;
  });
}
