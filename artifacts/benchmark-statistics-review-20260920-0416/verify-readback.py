"""Verify review follow-up without treating test-only edits as release qualification."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import subprocess
import zipfile

root = Path.cwd()
out = root / "artifacts/benchmark-statistics-review-20260920-0416"
build = root / "artifacts/benchmark-statistics-audit-20260920-0349"
legacy = root / "artifacts/legacy-benchmark-input-audit-20260920-0310"
def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()
baseline = json.loads((out / "baseline-sha256.json").read_text())
source = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
assert source == baseline["source"]
protected = {p: h for p, h in baseline["sha256"].items() if p not in baseline["scope"]}
assert all((root / p).is_file() and digest(root / p) == h for p, h in protected.items())
assert digest(root / "AGENTS.md") == "6a0c8eea9a3cf9ed680ea6ccb7814a59ed8d79365dd8f1a2229a1bf6dee27c4e"
reviewed = json.loads((build / "reviewed-sha256.json").read_text())
marker = "#[cfg(test)]\nmod tests {"
production = {}
with zipfile.ZipFile(out / "baseline.zip") as archive:
    for name, expected in reviewed.items():
        original = archive.read(name)
        assert hashlib.sha256(original).hexdigest() == expected
        old = original.decode().replace("\r\n", "\n")
        current = (root / name).read_text()
        assert old.count(marker) == current.count(marker) == 1
        assert old.split(marker)[0] == current.split(marker)[0]
        production[name] = hashlib.sha256(current.split(marker)[0].encode()).hexdigest()
for path in [out / "pre-mutation-sha256.json", legacy / "reviewed-sha256.json"]:
    assert all(digest(root / p) == h for p, h in json.loads(path.read_text()).items())
reviews = {}
for batch, folder in [("deleg_5887e5e5", legacy), ("deleg_91977f18", build)]:
    review = json.loads((folder / "review.json").read_text())
    assert review["passed"] is True and review["security_concerns"] == review["logic_errors"] == []
    reviews[batch] = {"status": "PASS", "verdictSha256": digest(folder / "review.json"), "scope": "Original candidate source review; executable checks recorded separately"}
assert "607 passed; 0 failed; 6 ignored" in (out / "rust-final.log").read_text()
release = re.findall(r"test result: ok. (\d+) passed; 0 failed", (out / "release-tests.log").read_text())
assert release == ["5", "4", "3", "1"]
for name, failure in [("near-equal-mutant.log", "assertion failed: actual > 0.0"), ("median-mutant.log", "left: inf")]:
    log = (out / name).read_text()
    assert "test result: FAILED" in log and failure in log
artifacts = json.loads((build / "built-artifacts.json").read_text(encoding="utf-8-sig"))
assert len(artifacts) == 3
for artifact in artifacts:
    assert digest(artifact["path"]) == artifact["sha256"] and Path(artifact["path"]).stat().st_size == artifact["bytes"]
native = json.loads((build / "native-ui.json").read_text())
assert native["source"] == source and native["status"] == native["cleanup"] == "PASS"
assert len(native["checks"]) == 23 and all(c["status"] == "PASS" for c in native["checks"])
assert native["sha256"] == artifacts[0]["sha256"]
frontend = (build / "frontend.log").read_text(encoding="utf-8")
assert "pass 274" in frontend and "fail 0" in frontend and re.search(r"Tests\s+276 passed", frontend)
assert "A11Y_PASS" in (build / "a11y.log").read_text(encoding="utf-8")
subprocess.run(["git", "diff", "--check"], check=True)
record = {
    "scope": "Statistics review reconciliation and test-only follow-up; not release qualification",
    "recordedAt": datetime.datetime.now().astimezone().isoformat(), "source": source, "sourceDirty": True,
    "currentFileSha256": {p: digest(root / p) for p in baseline["scope"]},
    "reviewedProductionSha256": production, "testOnlyDiffSha256": digest(out / "test-only.diff"),
    "checks": {"sourceReviews": reviews, "reviewedProductionUnchanged": "PASS", "mutationsDetectedAndRestored": "PASS", "rust": {"status": "PASS", "passed": 607, "failed": 0, "ignored": 6}, "releaseModeTests": {"status": "PASS", "passed": sum(map(int, release))}, "protectedFiles": {"status": "PASS", "count": len(protected)}, "agentsMdUnchanged": "PASS", "retainedFrontendEvidence": {"status": "PASS", "scriptTests": 274, "vitestTests": 276}, "retainedNativeEvidence": {"status": "PASS", "checks": 23}, "artifactBytesUnchanged": "PASS", "installerLifecycle": "UNKNOWN: no new candidate lifecycle run", "releaseQualification": "UNKNOWN: no clean-checkout qualification", "legacyOperationOwnership": "UNKNOWN: source finding only"},
    "artifacts": artifacts, "outstandingDelegations": [], "outstandingBoundedProcesses": [],
    "verificationStatus": "PARTIALLY VERIFIED", "publicationAttempted": False,
}
(out / "readback.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"protectedFiles": len(protected), "productionAndArtifactsUnchanged": "PASS", "rustTests": 607, "releaseTests": sum(map(int, release)), "outstandingDelegations": []}))
