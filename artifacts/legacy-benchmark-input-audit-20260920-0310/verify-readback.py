"""Verify current legacy-editor source and retained execution evidence."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import subprocess

root = Path.cwd()
out = root / "artifacts/legacy-benchmark-input-audit-20260920-0310"
def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()
baseline = json.loads((out / "baseline-sha256.json").read_text())
source = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
assert source == baseline["source"]
scoped = ["src/model.ts", "src/model.test.ts", "src/App.tsx", "src/App.staleResponses.test.tsx", "src/screens/BenchmarkScreen.tsx", "docs/DESIGN.md", "CHANGELOG.md", "TODO-0.6.md", "agents_feedback.md"]
protected = {p: h for p, h in baseline["sha256"].items() if p not in scoped}
assert all((root / p).is_file() and digest(root / p) == h for p, h in protected.items())
assert digest(root / "AGENTS.md") == "6a0c8eea9a3cf9ed680ea6ccb7814a59ed8d79365dd8f1a2229a1bf6dee27c4e"
reviewed = json.loads((out / "reviewed-sha256.json").read_text())
assert all(digest(root / p) == h for p, h in reviewed.items())
frontend = (out / "frontend.log").read_text(encoding="utf-8")
assert "pass 274" in frontend and "fail 0" in frontend and re.search(r"Tests\s+276 passed", frontend)
mutant = (out / "token-bound-mutant.log").read_text(encoding="utf-8")
assert "tokens=4097" in mutant and "to have a length of +0 but got 1" in mutant
native = json.loads((out / "native-ui.json").read_text())
assert native["source"] == source and native["status"] == native["cleanup"] == "PASS"
assert len(native["checks"]) == 23 and all(c["status"] == "PASS" for c in native["checks"])
for field, label, invalid, corrected in [("tokens", "Forced output tokens", "4097", "65"), ("repeats", "Measured repeats", "11", "10")]:
    evidence = next(c["evidence"] for c in native["checks"] if c["id"] == "benchmark-legacy-input-" + field)
    assert evidence["blank"]["value"] == "" and evidence["blank"]["valid"] is False
    assert label in evidence["blank"]["error"]
    assert evidence["rejected"]["value"] == invalid and evidence["rejected"]["valid"] is False
    assert label in evidence["rejected"]["error"]
    assert evidence["recovery"] == {"value": corrected, "valid": True, "error": None}
artifacts = json.loads((out / "built-artifacts.json").read_text(encoding="utf-8-sig"))
assert len(artifacts) == 3 and artifacts[0]["sha256"] == native["sha256"]
for artifact in artifacts:
    assert digest(artifact["path"]) == artifact["sha256"]
    assert Path(artifact["path"]).stat().st_size == artifact["bytes"]
    assert artifact["version"] == "0.6.0" and artifact["signature"] == "NotSigned"
assert "A11Y_PASS" in (out / "a11y.log").read_text(encoding="utf-8")
design = json.loads((out / "design-lint.log").read_text(encoding="utf-8"))
assert design["summary"]["errors"] == design["summary"]["warnings"] == 0
subprocess.run(["git", "diff", "--check"], check=True)
record = {
    "scope": "Legacy benchmark editor correction; not release qualification",
    "recordedAt": datetime.datetime.now().astimezone().isoformat(),
    "source": source, "sourceDirty": True,
    "changedFileSha256": {p: digest(root / p) for p in scoped},
    "sourceDiffSha256": digest(out / "source.diff"),
    "checks": {
        "frontend": {"status": "PASS", "command": "npm run check", "scriptTests": 274, "vitestTests": 276},
        "fixedBoundMutation": "PASS: 4097-token dispatch detected",
        "native": {"status": "PASS", "checks": 23, "cleanup": "PASS", "scope": "Legacy editing and errors with idle server; running-server request behavior covered by real-App IPC fixtures"},
        "packagedAccessibility": "PASS", "designmd": "PASS", "artifactReadback": "PASS",
        "protectedFiles": {"status": "PASS", "count": len(protected)},
        "agentsMdUnchanged": "PASS", "reviewedSourceUnchanged": "PASS",
        "independentReview": "UNKNOWN: deleg_5887e5e5 completion not yet received",
        "standaloneDesignDetector": "UNKNOWN: no repository-supplied command",
        "installerLifecycle": "UNKNOWN: not run for this candidate",
        "releaseQualification": "UNKNOWN: not run for this candidate",
    },
    "artifacts": artifacts, "outstandingDelegations": ["deleg_5887e5e5"],
    "verificationStatus": "PARTIALLY VERIFIED", "publicationAttempted": False,
}
(out / "readback.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"protectedFiles": len(protected), "protectedFilesUnchanged": "PASS", "reviewedSourceUnchanged": "PASS", "nativeChecks": 23, "artifactHashes": "PASS", "outstandingDelegations": record["outstandingDelegations"]}))
