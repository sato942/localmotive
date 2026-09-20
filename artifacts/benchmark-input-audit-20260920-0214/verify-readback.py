"""Read current evidence and verify the scoped v2 editor correction."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import subprocess
import zipfile

root = Path.cwd()
out = root / "artifacts/benchmark-input-audit-20260920-0214"
baseline = json.loads((out / "baseline-sha256.json").read_text())
source = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
assert source == baseline["source"]
scoped = ["src/V03EvidencePanel.tsx", "src/V03EvidencePanel.cancel.test.tsx", "docs/DESIGN.md", "CHANGELOG.md", "TODO-0.6.md", "agents_feedback.md"]
def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()
protected = {p: h for p, h in baseline["sha256"].items() if p not in scoped}
assert all((root / p).is_file() and digest(root / p) == h for p, h in protected.items())
assert digest(root / "AGENTS.md") == "6a0c8eea9a3cf9ed680ea6ccb7814a59ed8d79365dd8f1a2229a1bf6dee27c4e"
tune = json.loads((root / "artifacts/tune-input-audit-20260920-0141/readback.json").read_text())
tune_sources = {p: h for p, h in tune["changedFileSha256"].items() if p.startswith("src/")}
assert len(tune_sources) == 6 and all(digest(root / p) == h for p, h in tune_sources.items())
native = json.loads((out / "native-ui.json").read_text())
assert native["source"] == source and native["status"] == native["cleanup"] == "PASS"
assert len(native["checks"]) == 21 and all(c["status"] == "PASS" for c in native["checks"])
for field, value in [("promptTokens", "1"), ("generationTokens", "257"), ("warmups", "0"), ("trials", "5")]:
    evidence = next(c["evidence"] for c in native["checks"] if c["id"] == "benchmark-v2-input-" + field)
    assert evidence["blank"]["value"] == "" and evidence["blank"]["valid"] is False
    assert "workload." + field in evidence["blank"]["error"]
    assert evidence["rejected"]["valid"] is False
    assert "workload." + field in evidence["rejected"]["error"]
    assert evidence["recovery"] == {"value": value, "valid": True, "error": None}
artifacts = json.loads((out / "built-artifacts.json").read_text(encoding="utf-8-sig"))
assert len(artifacts) == 3 and artifacts[0]["sha256"] == native["sha256"]
for artifact in artifacts:
    assert digest(artifact["path"]) == artifact["sha256"]
    assert Path(artifact["path"]).stat().st_size == artifact["bytes"]
    assert artifact["version"] == "0.6.0" and artifact["signature"] == "NotSigned"
frontend = (out / "frontend.log").read_text(encoding="utf-8")
assert "pass 274" in frontend and "fail 0" in frontend and re.search(r"Tests\s+249 passed", frontend)
assert "A11Y_PASS" in (out / "a11y.log").read_text(encoding="utf-8")
assert "not configured to support act" not in (out / "domains-green.log").read_text(encoding="utf-8")
design = json.loads((out / "design-lint.log").read_text(encoding="utf-8"))
assert design["summary"]["errors"] == design["summary"]["warnings"] == 0
subprocess.run(["git", "diff", "--check"], check=True)
with zipfile.ZipFile(out / "baseline.zip") as archive:
    line_delta = len((root / scoped[0]).read_text().splitlines()) - len(archive.read(scoped[0]).decode().splitlines())
assert line_delta == -5
report = {
    "scope": "V2 benchmark numeric editing only; not release qualification",
    "verificationStatus": "PARTIALLY VERIFIED",
    "recordedAt": datetime.datetime.now().astimezone().isoformat(),
    "source": source, "sourceDirty": True,
    "changedFileSha256": {p: digest(root / p) for p in scoped},
    "sourceDiffSha256": digest(out / "source.diff"),
    "sourceReview": "Parent diff inspection and deterministic regression checks; no new independent verdict",
    "checks": {
        "frontend": {"status": "PASS", "command": "npm run check", "scriptTests": 274, "vitestTests": 249},
        "native": {"status": "PASS", "checks": len(native["checks"]), "cleanup": "PASS", "scope": "V2 editing and errors with idle server; request behavior covered by real-panel tests with IPC mocked"},
        "packagedAccessibility": "PASS", "designmd": "PASS", "artifactReadback": "PASS",
        "protectedFiles": {"status": "PASS", "count": len(protected)},
        "agentsMdUnchanged": "PASS", "tuneSourcesUnchanged": {"status": "PASS", "count": len(tune_sources)},
        "productionLineDelta": line_delta,
        "standaloneDesignDetector": "UNKNOWN: no repository-supplied command",
        "installerLifecycle": "UNKNOWN: not run for this candidate",
        "releaseQualification": "UNKNOWN: not run for this candidate",
    },
    "outstandingDelegations": ["deleg_f6382b96"],
    "artifacts": artifacts, "publicationAttempted": False,
}
(out / "readback.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({"protectedFiles": len(protected), "protectedFilesUnchanged": "PASS", "nativeChecks": len(native["checks"]), "artifactHashes": "PASS", "tuneSourcesUnchanged": "PASS", "outstandingDelegations": report["outstandingDelegations"]}))
