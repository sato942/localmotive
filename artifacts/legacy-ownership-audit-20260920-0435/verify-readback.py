"""Read back the local ownership candidate. This is not a release qualifier."""
import hashlib
import json
from pathlib import Path
import subprocess
from datetime import datetime

out = Path(__file__).resolve().parent
repo = out.parents[1]


def load(name):
    return json.loads((out / name).read_text(encoding="utf-8-sig"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


baseline = load("baseline-sha256.json")
source = load("reviewed-sha256.json")
allowed = set(source) | {
    "CHANGELOG.md", "TODO-0.6.md", "docs/DESIGN.md", "agents_feedback.md",
}
protected = {name: value for name, value in baseline["sha256"].items() if name not in allowed}
changed = [name for name, value in protected.items() if digest(repo / name) != value]
assert not changed, changed
assert digest(repo / "AGENTS.md") == "6a0c8eea9a3cf9ed680ea6ccb7814a59ed8d79365dd8f1a2229a1bf6dee27c4e"
for name, value in source.items():
    assert digest(repo / name) == value, name
for name in ("src-tauri/Cargo.toml", "src-tauri/Cargo.lock"):
    assert digest(repo / name) == baseline["sha256"][name], name
revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip()
assert revision == baseline["source"]
subprocess.run(["git", "diff", "--check"], cwd=repo, check=True)

native = load("native-ui.json")
assert native["status"] == native["cleanup"] == "PASS"
assert len(native["checks"]) == 25
assert all(check["status"] == "PASS" for check in native["checks"])
checks = {check["id"]: check for check in native["checks"]}
assert "legacy-cancel-handle-survives-navigation-and-panel-publish" in checks
assert "idle-cancel-command-refuses-without-work" in checks
artifacts = load("built-artifacts.json")
assert len(artifacts) == 3 and all(isinstance(item, dict) for item in artifacts)
for item in artifacts:
    path = Path(item["path"])
    assert path.is_file() and path.stat().st_size == item["bytes"] > 0
    assert digest(path) == item["sha256"]
    assert item["version"] == "0.6.0" and item["signature"] == "NotSigned"
assert artifacts[0]["sha256"] == native["sha256"]

rust = (out / "rust-full.log").read_text()
assert "610 passed; 0 failed; 7 ignored" in rust
assert "Finished `dev` profile" in (out / "clippy.log").read_text()
assert (out / "fmt.log").read_text() == ""
frontend = (out / "frontend.log").read_text(encoding="utf-8")
assert "pass 274" in frontend and "278 passed" in frontend
assert "A11Y_PASS" in (out / "a11y.log").read_text()
assert "3 passed; 0 failed; 1 ignored" in (out / "release-tests.log").read_text()
assert load("design-lint.log")["summary"]["errors"] == 0
for name, failure in {
    "overlap-red.log": "a second legacy command reached HTTP while the first response was held",
    "cancel-red.log": "No benchmark is running",
    "cancel-ui-red.log": "the legacy run must retain a visible cancel owner",
    "reentrant-ui-red.log": "to have a length of 1 but got 2",
    "drain-mutant.log": "cancellation released a request before drain",
    "handle-mutant.log": "the panel must not clear the legacy run's handle",
}.items():
    assert failure in (out / name).read_text(encoding="utf-8"), name
soak = load("ownership-soak.json")
assert len(soak) == 8 and all(item["exit"] == 0 for item in soak)
assert all("3 passed; 0 failed; 1 ignored" in item["summary"] for item in soak)
assert "TaskDialogIndirect" in (out / "test-imports.log").read_text()
assert "TaskDialogIndirect" not in (out / "restored-test-imports.log").read_text()
cleanup = load("cleanup.json")
assert cleanup["AppProcesses"] == cleanup["FixtureProcesses"] == cleanup["CdpListeners"] == 0
report = {
    "checkedAt": datetime.now().astimezone().isoformat(),
    "revision": revision,
    "status": "PARTIALLY VERIFIED",
    "sourceAndArtifactReadback": "PASS",
    "protectedFiles": len(protected),
    "agentsSha256": digest(repo / "AGENTS.md"),
    "reviewedSource": source,
    "artifacts": artifacts,
    "rustTests": 610,
    "ignoredRustEntries": 7,
    "scriptTests": 274,
    "frontendTests": 278,
    "ownershipSoakRepetitions": len(soak),
    "releaseOwnershipTests": 3,
    "nativeChecks": len(native["checks"]),
    "pendingReview": "Legacy ownership source review; full completion message not yet reconciled",
    "unknown": [
        "Independent source-review result",
        "Separate Stop check-before-worker interleavings",
        "Packaged real-model cancellation",
        "Installer lifecycle and clean-checkout release qualification",
    ],
}
(out / "readback.json").write_bytes((json.dumps(report, indent=2) + "\n").encode())
print(json.dumps({key: report[key] for key in (
    "status", "sourceAndArtifactReadback", "protectedFiles", "rustTests",
    "scriptTests", "frontendTests", "nativeChecks", "pendingReview",
)}))
