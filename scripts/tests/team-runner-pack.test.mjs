// Team-runner pack invariants (P1-12, A2). Static checks: the pack is a
// human-run provisioner, so live provisioning stays UNKNOWN until a real
// machine runs it. These tests pin the safety invariants only.
import { strict as assert } from "node:assert";
import { readFileSync, existsSync } from "node:fs";
import { test } from "node:test";

const root = "scripts/team-runner/";
const files = ["README.md", "provision-team-runner.ps1", "REGISTER.md", "start-team-runner.ps1"];

for (const file of files) {
  test(`pack ships ${file}`, () => {
    assert.ok(existsSync(root + file), `missing ${root + file}`);
  });
}

test("provisioner refuses the owner host", () => {
  const ps = readFileSync(root + "provision-team-runner.ps1", "utf8");
  assert.ok(
    ps.includes('$env:COMPUTERNAME -eq "DESKTOP-HPTF57N"'),
    "must compare the machine name against the owner host guard",
  );
});

test("provisioner takes and stores no token", () => {
  const ps = readFileSync(root + "provision-team-runner.ps1", "utf8");
  assert.equal((ps.match(/--token/g) ?? []).length, 0, "token must stay in the human config step");
  assert.ok(ps.includes("SupportsShouldProcess"), "provisioner must support -WhatIf dry-run");
});

test("registration uses exactly the release label set", () => {
  const md = readFileSync(root + "REGISTER.md", "utf8");
  const configLine = md.split("\n").find((line) => line.includes("config.cmd --unattended")) ?? "";
  assert.ok(configLine.includes("--labels localmotive-release"), "exact label set");
  assert.ok(!configLine.includes("localmotive-hw"), "team runner must not take hardware labels");
});

test("cutover removes the owner label the same day", () => {
  const md = readFileSync(root + "README.md", "utf8");
  assert.ok(md.includes("remove the owner label"), "dual-label race must be called out");
});
