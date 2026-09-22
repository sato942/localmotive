"""Verify the delivered review, restored production source and current evidence."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import subprocess

root = Path.cwd()
out = root / "artifacts/tune-review-20260920-0252"
review_dir = root / "artifacts/tune-input-audit-20260920-0141"
packaged_dir = root / "artifacts/benchmark-input-audit-20260920-0214"
def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()
baseline = json.loads((out / "baseline-sha256.json").read_text())
source = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
assert source == baseline["source"]
changed = {"src/screens/TuneScreen.test.tsx", "src/App.staleResponses.test.tsx", "TODO-0.6.md", "agents_feedback.md"}
protected = {p: h for p, h in baseline["sha256"].items() if p not in changed}
assert all((root / p).is_file() and digest(root / p) == h for p, h in protected.items())
assert digest(root / "AGENTS.md") == "6a0c8eea9a3cf9ed680ea6ccb7814a59ed8d79365dd8f1a2229a1bf6dee27c4e"
prior_review = json.loads((review_dir / "readback.json").read_text())
reviewed_production = {p: h for p, h in prior_review["changedFileSha256"].items() if p.startswith("src/") and ".test." not in p}
assert len(reviewed_production) == 3
assert all(digest(root / p) == h for p, h in reviewed_production.items())
prior_v2 = json.loads((packaged_dir / "readback.json").read_text())
assert digest(root / "src/V03EvidencePanel.tsx") == prior_v2["changedFileSha256"]["src/V03EvidencePanel.tsx"]
review = json.loads((review_dir / "review.json").read_text())
assert review["passed"] is True and review["security_concerns"] == review["logic_errors"] == []
assert len(review["suggestions"]) == 3
frontend = (out / "frontend.log").read_text(encoding="utf-8")
assert "pass 274" in frontend and "fail 0" in frontend and re.search(r"Tests\s+253 passed", frontend)
for filename, expected in [("hidden-error-mutant.log", "AssertionError: expected undefined"), ("nonfinite-guard-mutant.log", "to have a length of +0 but got 1")]:
    assert expected in (out / filename).read_text(encoding="utf-8")
artifacts = json.loads((packaged_dir / "built-artifacts.json").read_text(encoding="utf-8-sig"))
native = json.loads((packaged_dir / "native-ui.json").read_text())
assert native["source"] == source and native["status"] == native["cleanup"] == "PASS"
assert len(native["checks"]) == 21 and all(c["status"] == "PASS" for c in native["checks"])
assert len(artifacts) == 3 and artifacts[0]["sha256"] == native["sha256"]
for artifact in artifacts:
    assert digest(artifact["path"]) == artifact["sha256"]
    assert Path(artifact["path"]).stat().st_size == artifact["bytes"]
assert "A11Y_PASS" in (packaged_dir / "a11y.log").read_text(encoding="utf-8")
subprocess.run(["git", "diff", "--check"], check=True)
record = {
    "scope": "Tune review reconciliation and test-only follow-up",
    "recordedAt": datetime.datetime.now().astimezone().isoformat(),
    "source": source, "sourceDirty": True,
    "review": {"status": "PASS", "path": str(review_dir / "review.json"), "sha256": digest(review_dir / "review.json"), "scope": "Tune source delta only; reviewer did not run tests"},
    "suggestionDispositions": [
        "Added prior-report error regression; hidden-error mutation failed",
        "Added three App overflow cases; non-finite guard mutation failed",
        "Retained current labels, native validity and status; optional per-control ARIA association not added",
    ],
    "checks": {
        "frontend": {"status": "PASS", "command": "npm run check", "scriptTests": 274, "vitestTests": 253},
        "productionRestoredAndUnchanged": "PASS", "agentsMdUnchanged": "PASS",
        "protectedFiles": {"status": "PASS", "count": len(protected)},
        "existingPackagedEvidence": {"status": "PASS", "nativeChecks": 21, "accessibility": "PASS", "artifactHashesMatch": True, "rebuiltDuringReviewFollowup": False},
        "standaloneDesignDetector": "UNKNOWN: no repository-supplied command",
        "manualScreenReader": "UNKNOWN: not run",
        "installerLifecycle": "UNKNOWN: not run for this candidate",
        "releaseQualification": "UNKNOWN: not run for this candidate",
    },
    "changedFileSha256": {p: digest(root / p) for p in sorted(changed)},
    "artifacts": artifacts,
    "deliveredDelegations": ["deleg_f6382b96"], "outstandingDelegations": [],
    "verificationStatus": "PARTIALLY VERIFIED", "publicationAttempted": False,
}
(out / "readback.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"review": "PASS", "productionUnchanged": "PASS", "protectedFiles": len(protected), "frontendTests": 253, "artifactHashes": "PASS", "outstandingDelegations": []}))
