#!/usr/bin/env node
import { readFile, readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { parse } from "yaml";

function commandLines(run) {
  return String(run ?? "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("#"));
}

/// Normalize a GitHub Actions `permissions` block into exact
/// `scope: value` strings so a forbidden grant cannot match by substring
/// (a `checks: write` grant must never be reported as `contents: write`).
function permissionGrants(permissions) {
  if (permissions == null) return [];
  if (typeof permissions !== "object") return [String(permissions)];
  return Object.entries(permissions).map(([scope, value]) => `${scope}: ${value}`);
}

function dependencies(jobs, jobName) {
  const seen = new Set();
  const visit = (name) => {
    const needs = jobs[name]?.needs;
    for (const dependency of typeof needs === "string" ? [needs] : (needs ?? [])) {
      if (!seen.has(dependency)) {
        seen.add(dependency);
        visit(dependency);
      }
    }
  };
  visit(jobName);
  return seen;
}

export function validateWorkflowGates(workflows, policy) {
  const failures = [];
  for (const [file, rules] of Object.entries(policy.workflows ?? {})) {
    const workflow = workflows[file];
    if (!workflow) {
      failures.push(`${file} is missing`);
      continue;
    }
    const jobs = workflow.jobs ?? {};
    for (const [jobName, commands] of Object.entries(rules.gates ?? {})) {
      const job = jobs[jobName];
      if (!job) {
        failures.push(`${file} material gate job ${jobName} is missing`);
        continue;
      }
      if (job["continue-on-error"] === true) failures.push(`${file}:${jobName} permits job failure`);
      for (const command of commands) {
        const matching = (job.steps ?? []).filter((step) => commandLines(step.run).includes(command));
        if (matching.length !== 1) {
          failures.push(`${file}:${jobName} must execute ${JSON.stringify(command)} exactly once`);
          continue;
        }
        const step = matching[0];
        if (step["continue-on-error"] === true) failures.push(`${file}:${jobName} permits ${JSON.stringify(command)} to fail`);
        if (commandLines(step.run).length !== 1) {
          failures.push(`${file}:${jobName} must isolate ${JSON.stringify(command)} in one fail-fast step`);
        }
      }
    }
    for (const [jobName, commands] of Object.entries(rules.requiredCommands ?? {})) {
      const job = jobs[jobName];
      if (!job) {
        failures.push(`${file} required-command job ${jobName} is missing`);
        continue;
      }
      for (const command of commands) {
        const matching = (job.steps ?? []).filter((step) => commandLines(step.run).includes(command));
        if (matching.length !== 1) {
          failures.push(`${file}:${jobName} must execute ${JSON.stringify(command)} exactly once`);
          continue;
        }
        if (matching[0]["continue-on-error"] === true) {
          failures.push(`${file}:${jobName} permits ${JSON.stringify(command)} to fail`);
        }
      }
    }
    const requiredGates = Object.keys(rules.gates ?? {});
    const protectedJobs = { ...(rules.packageJobs ?? {}), ...(rules.publicationJobs ?? {}) };
    for (const [jobName, required] of Object.entries(protectedJobs)) {
      if (!jobs[jobName]) {
        failures.push(`${file} protected job ${jobName} is missing`);
        continue;
      }
      const reachable = dependencies(jobs, jobName);
      for (const gate of required.length ? required : requiredGates) {
        if (!reachable.has(gate)) failures.push(`${file}:${jobName} does not depend on material gate ${gate}`);
      }
    }
    // R01 (follow-up review db548c8): the needs context exposes ONLY the jobs
    // listed directly in a job's `needs`. Transitive reachability does not
    // make an ancestor's outputs readable, so every needs.<job>.outputs /
    // needs.<job>.result reference must point at a DIRECT dependency.
    for (const [jobName, job] of Object.entries(jobs)) {
      const needs = job?.needs;
      const direct = new Set(typeof needs === "string" ? [needs] : (needs ?? []));
      for (const match of JSON.stringify(job).matchAll(/needs\.([A-Za-z0-9_-]+)\.(?:outputs|result)/gu)) {
        if (!direct.has(match[1])) {
          failures.push(
            `${file}:${jobName} reads needs.${match[1]} outputs without a direct dependency (needs: ${[...direct].join(", ") || "none"})`,
          );
        }
      }
    }
    // Forbidden commands: a verify-only workflow must not be able to publish,
    // so the policy names the publication mechanisms it must never contain
    // (R16 release behavior). Checked structurally, not against a comment: a
    // step `uses:` or any executed run line that matches fails the gate.
    for (const forbidden of rules.forbiddenCommands ?? []) {
      for (const [jobName, job] of Object.entries(jobs)) {
        for (const [index, step] of (job.steps ?? []).entries()) {
          const uses = String(step?.uses ?? "");
          if (uses.includes(forbidden)) {
            failures.push(`${file}:${jobName}:step ${index} uses forbidden ${JSON.stringify(forbidden)}`);
          }
          const runLines = commandLines(step?.run);
          if (runLines.some((line) => line.includes(forbidden))) {
            failures.push(`${file}:${jobName}:step ${index} runs forbidden ${JSON.stringify(forbidden)}`);
          }
          // A permissions block grants capability without a command line; the
          // string form catches both `contents: write` here and in the file
          // header because jobs inherit the top-level permissions.
          if (permissionGrants(job?.permissions).includes(forbidden)) {
            failures.push(`${file}:${jobName} is granted ${forbidden}`);
          }
        }
      }
      if (permissionGrants(workflow?.permissions).includes(forbidden)) {
        failures.push(`${file} is granted ${forbidden} at the workflow level`);
      }
    }
    for (const [jobName, job] of Object.entries(jobs)) {
      for (const [index, step] of (job.steps ?? []).entries()) {
        if (step?.["continue-on-error"] === true) failures.push(`${file}:${jobName}:step ${index} permits failure`);
        const run = String(step?.run ?? "");
        if (/\|\|\s*true\b|\$ErrorActionPreference\s*=\s*["']Continue["']/i.test(run)) {
          failures.push(`${file}:${jobName}:step ${index} suppresses a native failure`);
        }
      }
    }
  }
  return { ok: failures.length === 0, failures };
}

export async function loadWorkflows(root) {
  const directory = resolve(root, ".github", "workflows");
  const files = (await readdir(directory)).filter((file) => /\.ya?ml$/i.test(file)).sort();
  return Object.fromEntries(await Promise.all(files.map(async (file) => [file, parse(await readFile(resolve(directory, file), "utf8"))])));
}

async function main() {
  const root = process.cwd();
  const policy = JSON.parse(await readFile(resolve(root, ".github", "workflow-gates.json"), "utf8"));
  const result = validateWorkflowGates(await loadWorkflows(root), policy);
  console.log(JSON.stringify(result, null, 2));
  if (!result.ok) process.exitCode = 1;
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  main().catch((error) => {
    console.error(`workflow-gate: ${error.message}`);
    process.exitCode = 1;
  });
}
