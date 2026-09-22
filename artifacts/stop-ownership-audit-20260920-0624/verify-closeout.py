"""Verify the paused working-tree candidate without running new application work."""
from datetime import datetime
import hashlib
import json
from pathlib import Path
import subprocess

out = Path(__file__).resolve().parent
repo = out.parents[1]
legacy = repo / "artifacts/legacy-ownership-audit-20260920-0435"


def load(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


baseline = load(out / "baseline-sha256.json")
protected = {name: value for name, value in baseline["sha256"].items() if name not in baseline["scope"]}
changed = [name for name, value in protected.items() if digest(repo / name) != value]
assert not changed, changed
sources = {**load(legacy / "reviewed-sha256.json"), **load(out / "reviewed-sha256.json")}
for name, value in sources.items():
    assert digest(repo / name) == value, name
assert digest(repo / "AGENTS.md") == "6a0c8eea9a3cf9ed680ea6ccb7814a59ed8d79365dd8f1a2229a1bf6dee27c4e"
revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip()
branch = subprocess.check_output(["git", "branch", "--show-current"], cwd=repo, text=True).strip()
assert revision == baseline["source"]
subprocess.run(["git", "diff", "--check"], cwd=repo, check=True)

artifacts = load(out / "built-artifacts.json")
assert len(artifacts) == 3
for item in artifacts:
    path = Path(item["path"])
    assert path.stat().st_size == item["bytes"] > 0
    assert digest(path) == item["sha256"]
    assert item["version"] == "0.6.0" and item["signature"] == "NotSigned"
native = load(out / "native-ui.json")
assert native["status"] == native["cleanup"] == "PASS"
assert len(native["checks"]) == 26 and all(check["status"] == "PASS" for check in native["checks"])
assert native["sha256"] == artifacts[0]["sha256"]
assert any(check["id"] == "idle-stop-command-remains-repeatable" for check in native["checks"])
assert "619 passed; 0 failed; 7 ignored" in (out / "rust-full.log").read_text()
assert "9 passed; 0 failed; 0 ignored" in (out / "release-tests-project-cwd.log").read_text()
assert "Finished `dev` profile" in (out / "clippy.log").read_text()
assert (out / "fmt.log").read_text() == ""
frontend = (out / "frontend.log").read_text(encoding="utf-8")
assert "pass 274" in frontend and "278 passed" in frontend
assert "A11Y_PASS" in (out / "a11y.log").read_text()
soak = load(out / "stop-soak.json")
assert len(soak) == 8
assert all(row["exit"] == 0 and "9 passed; 0 failed; 0 ignored" in row["summary"] for row in soak)
for name, text in {
    "queue-red.log": "queued Stop bypassed Benchmark ownership",
    "replacement-red.log": "queued Stop terminated the replacement process",
    "epoch-mutant.log": "assertion failed: reserve_stop(&state, request).is_err()",
    "lease-mutant.log": "Stop released its reservation before the server slot",
    "signal-mutant.log": "assertion failed: !cancel.load(Ordering::Relaxed)",
}.items():
    assert text in (out / name).read_text(), name
cleanup = load(out / "close-cleanup.json")
assert cleanup["status"] == "PASS"
assert all(not cleanup[key] for key in ("appProcesses", "testProcesses", "debuggerListeners", "standInCandidates"))
legacy_review = load(legacy / "review.json")
assert legacy_review["passed"] is False and len(legacy_review["logic_errors"]) == 3
stop_review = load(out / "review.json")
assert stop_review["passed"] is True
assert not stop_review["security_concerns"] and not stop_review["logic_errors"]
report = {
    "checkedAt": datetime.now().astimezone().isoformat(),
    "workState": "PAUSED_BY_USER",
    "status": "PARTIALLY VERIFIED",
    "branch": branch,
    "revision": revision,
    "sourceAndArtifactReadback": "PASS",
    "protectedFiles": len(protected),
    "reviewedSource": sources,
    "artifacts": artifacts,
    "rustTests": 619,
    "ignoredRustEntries": 7,
    "scriptTests": 274,
    "frontendTests": 278,
    "releaseStopTests": 9,
    "stopSoakRepetitions": len(soak),
    "nativeChecks": len(native["checks"]),
    "cleanup": cleanup,
    "legacySourceReview": {
        "delegationId": "deleg_16c0e72a",
        "status": "FAIL",
        "reportedLogicFindings": legacy_review["logic_errors"],
        "runtimeReproductions": "UNKNOWN; deferred while paused",
        "implementationChangedAfterReview": False,
    },
    "stopSourceReview": {
        "delegationId": "deleg_060e7c42",
        "status": "PASS",
        "implementationChangedAfterReview": False,
        "followups": [
            "Optional intervening-operation wraparound regression",
            "Starting-slot cleanup after post-spawn construction error still needs reproduction; ContainedProcess Drop already attempts termination and reap",
        ],
    },
    "pendingReviews": [],
    "outstandingDelegations": [],
    "reviewMessagesReconciled": True,
    "limitations": [
        "Legacy source review failed; cancellation finalization, construction and aborted-future gaps require correction",
        "Working tree remains intentionally dirty and uncommitted",
        "No installer lifecycle or clean-checkout release qualification",
        "No real-model performance or packaged active-model cancellation claim",
        "Post-spawn startup client-construction cleanup still needs reproduction",
        "No signing, tag change, push or publication",
    ],
}
(out / "readback.json").write_bytes((json.dumps(report, indent=2) + "\n").encode())
print(json.dumps({key: report[key] for key in ("workState", "status", "sourceAndArtifactReadback", "protectedFiles", "rustTests", "nativeChecks")}))
