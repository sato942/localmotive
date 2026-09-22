"""Read back this working-tree correction; do not qualify a release."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import subprocess

root = Path.cwd()
out = root / "artifacts/benchmark-statistics-audit-20260920-0349"
def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()
baseline = json.loads((out / "baseline-sha256.json").read_text())
source = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
assert source == baseline["source"]
protected = {p: h for p, h in baseline["sha256"].items() if p not in baseline["scope"]}
assert all((root / p).is_file() and digest(root / p) == h for p, h in protected.items())
assert digest(root / "AGENTS.md") == "6a0c8eea9a3cf9ed680ea6ccb7814a59ed8d79365dd8f1a2229a1bf6dee27c4e"
for record in [out / "pre-mutation-sha256.json", out / "reviewed-sha256.json", root / "artifacts/legacy-benchmark-input-audit-20260920-0310/reviewed-sha256.json"]:
    assert all(digest(root / p) == h for p, h in json.loads(record.read_text()).items())
legacy_review_path = root / "artifacts/legacy-benchmark-input-audit-20260920-0310/review.json"
legacy_review = json.loads(legacy_review_path.read_text())
assert legacy_review["passed"] is True and legacy_review["security_concerns"] == legacy_review["logic_errors"] == []
rust = (out / "rust-final.log").read_text()
assert "603 passed; 0 failed; 6 ignored" in rust and "Doc-tests" in rust
release = re.findall(r"test result: ok. (\d+) passed; 0 failed", (out / "release-tests.log").read_text())
assert release == ["4", "3", "2"]
frontend = (out / "frontend.log").read_text(encoding="utf-8")
assert "pass 274" in frontend and "fail 0" in frontend and re.search(r"Tests\s+276 passed", frontend)
for name, failure in [("summary-red.log", "left: None"), ("median-red.log", "right: 5e-324"), ("deviation-red.log", "left: inf"), ("mean-mutant.log", "left: None"), ("denominator-mutant.log", "normalized sample deviation=1")]:
    text = (out / name).read_text()
    assert "test result: FAILED" in text and failure in text
native = json.loads((out / "native-ui.json").read_text())
assert native["source"] == source and native["status"] == native["cleanup"] == "PASS"
assert len(native["checks"]) == 23 and all(check["status"] == "PASS" for check in native["checks"])
artifacts = json.loads((out / "built-artifacts.json").read_text(encoding="utf-8-sig"))
assert len(artifacts) == 3 and artifacts[0]["sha256"] == native["sha256"]
for artifact in artifacts:
    assert digest(artifact["path"]) == artifact["sha256"]
    assert Path(artifact["path"]).stat().st_size == artifact["bytes"]
    assert artifact["version"] == "0.6.0" and artifact["signature"] == "NotSigned"
assert "A11Y_PASS" in (out / "a11y.log").read_text(encoding="utf-8")
cleanup = json.loads((out / "process-cleanup.json").read_text(encoding="utf-8-sig"))
assert cleanup == {"applicationProcesses": 0, "diagnosticPortListeners": 0, "ownedWebViewProcesses": 0}
subprocess.run(["git", "diff", "--check"], check=True)
record = {
    "scope": "Benchmark statistics correction; not release qualification",
    "recordedAt": datetime.datetime.now().astimezone().isoformat(),
    "source": source, "sourceDirty": True,
    "changedFileSha256": {p: digest(root / p) for p in baseline["scope"]},
    "sourceDiffSha256": digest(out / "source.diff"),
    "checks": {
        "rust": {"status": "PASS", "passed": 603, "failed": 0, "ignored": 6},
        "releaseMode": {"status": "PASS", "passed": sum(map(int, release))},
        "frontend": {"status": "PASS", "scriptTests": 274, "vitestTests": 276},
        "mutationDetectionAndRestoration": "PASS", "nativeSmoke": {"status": "PASS", "checks": 23},
        "artifactReadback": "PASS", "packagedAccessibility": "PASS", "cleanup": cleanup,
        "protectedFiles": {"status": "PASS", "count": len(protected)},
        "agentsMdUnchanged": "PASS", "bothReviewScopesUnchanged": "PASS",
        "legacySourceReview": {"status": "PASS", "sha256": digest(legacy_review_path), "testsExecutedByReviewer": False},
        "statisticsSourceReview": "UNKNOWN: deleg_91977f18 completion not yet received",
        "installerLifecycle": "UNKNOWN: not run for this candidate",
        "releaseQualification": "UNKNOWN: not run for this candidate",
        "legacyOperationOwnership": "UNKNOWN: source finding only; no overlap reproduction or fix",
    },
    "artifacts": artifacts, "outstandingDelegations": ["deleg_91977f18"],
    "verificationStatus": "PARTIALLY VERIFIED", "publicationAttempted": False,
}
(out / "readback.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"protectedFiles": len(protected), "sourceAndArtifactReadback": "PASS", "nativeChecks": 23, "outstandingDelegations": record["outstandingDelegations"]}))
