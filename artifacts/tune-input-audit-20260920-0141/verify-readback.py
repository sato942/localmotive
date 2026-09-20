"""Read-only working-tree and artifact checks for the Tune input correction."""
from pathlib import Path
import datetime
import difflib
import hashlib
import json
import re
import subprocess
import zipfile

root = Path.cwd()
out = root / "artifacts/tune-input-audit-20260920-0141"
baseline = json.loads((out / "baseline-sha256.json").read_text())
source = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
assert source == baseline["source"]
scoped = [
    "src/model.ts", "src/model.test.ts", "src/App.tsx",
    "src/App.staleResponses.test.tsx", "src/screens/TuneScreen.tsx",
    "src/screens/TuneScreen.test.tsx", "docs/DESIGN.md", "TODO-0.6.md",
    "CHANGELOG.md", "agents_feedback.md",
]

def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

protected = {p: h for p, h in baseline["sha256"].items() if p not in scoped}
changed = [p for p, h in protected.items() if not (root / p).is_file() or digest(root / p) != h]
assert not changed, changed
assert digest(root / "AGENTS.md") == "6a0c8eea9a3cf9ed680ea6ccb7814a59ed8d79365dd8f1a2229a1bf6dee27c4e"
with zipfile.ZipFile(out / "baseline.zip") as archive:
    # The independent review receives this exact source diff, not the whole dirty tree.
    current_diff = "".join("".join(difflib.unified_diff(
        archive.read(p).decode().splitlines(True), (root / p).read_text().splitlines(True),
        fromfile="before/" + p, tofile="after/" + p,
    )) for p in scoped[:6])
assert current_diff == (out / "source.diff").read_text(), "Reviewed source diff changed"
native = json.loads((out / "native-ui.json").read_text())
assert native["source"] == source
assert native["status"] == native["cleanup"] == "PASS"
assert len(native["checks"]) == 17
assert all(c["status"] == "PASS" for c in native["checks"])
for field, corrected in [("maxTrials", "12"), ("tokens", "257"), ("repeats", "5")]:
    evidence = next(c["evidence"] for c in native["checks"] if c["id"] == "tune-input-" + field)
    assert evidence["blank"]["value"] == ""
    assert evidence["blank"]["valid"] is False and evidence["blank"]["disabled"] is True
    assert evidence["rejected"]["disabled"] is True
    assert evidence["recovery"] == {"value": corrected, "valid": True, "enabled": True, "error": None}
artifacts = json.loads((out / "built-artifacts.json").read_text(encoding="utf-8-sig"))
assert len(artifacts) == 3
assert artifacts[0]["sha256"] == native["sha256"]
for artifact in artifacts:
    assert digest(artifact["path"]) == artifact["sha256"]
    assert Path(artifact["path"]).stat().st_size == artifact["bytes"]
    assert artifact["version"] == "0.6.0" and artifact["signature"] == "NotSigned"
frontend = (out / "frontend.log").read_text(encoding="utf-8")
assert "pass 274" in frontend and "fail 0" in frontend
assert re.search(r"Tests\s+224 passed", frontend)
assert "A11Y_PASS" in (out / "a11y.log").read_text(encoding="utf-8")
subprocess.run(["git", "diff", "--check"], check=True)
review_path = out / "review.json"
review = json.loads(review_path.read_text()) if review_path.exists() else None
report = {
    "scope": "Working-tree Tune input correction; not release qualification",
    "recordedAt": datetime.datetime.now().astimezone().isoformat(),
    "source": source,
    "sourceDirty": True,
    "sourceDiffSha256": digest(out / "source.diff"),
    "changedFileSha256": {p: digest(root / p) for p in scoped},
    "protectedFiles": len(protected),
    "protectedFilesUnchanged": "PASS",
    "agentsMdUnchanged": "PASS",
    "frontend": {"command": "npm run check", "status": "PASS", "scriptTests": 274, "vitestTests": 224},
    "native": {"checks": len(native["checks"]), "status": "PASS", "cleanup": "PASS", "scope": "Metadata-only readiness fixture; no valid Tune dispatch"},
    "packagedAccessibility": "PASS",
    "artifacts": artifacts,
    "independentReview": review if review is not None else "UNKNOWN: result not delivered",
    "standaloneDesignDetector": "UNKNOWN: no repository-supplied detector command",
    "installerLifecycle": "UNKNOWN: not run for this candidate",
    "releaseQualification": "UNKNOWN: not run for this candidate",
    "publicationAttempted": False,
}
(out / "readback.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({"protectedFiles": len(protected), "protectedFilesUnchanged": "PASS", "nativeChecks": len(native["checks"]), "artifactHashes": "PASS", "sourceBinding": "PASS", "independentReview": "pending" if review is None else review["passed"]}))
