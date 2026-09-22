"""Read-only source/artifact checks; write audit records, not release evidence."""
import difflib
import hashlib
import json
import re
import subprocess
import zipfile
from datetime import datetime, timezone
from pathlib import Path

root = Path.cwd()
out = root / "artifacts/ideas-audit-20260919-2202"
baseline = json.loads((out / "baseline-sha256.json").read_text(encoding="utf-8"))
sha256 = lambda data: hashlib.sha256(data).hexdigest()
with zipfile.ZipFile(out / "baseline.zip") as archive:
    assert set(archive.namelist()) == set(baseline), "Incomplete baseline archive"
    assert all(sha256(archive.read(name)) == digest for name, digest in baseline.items())
    changed = {name: sha256((root / name).read_bytes())
               for name, digest in baseline.items()
               if (root / name).exists() and sha256((root / name).read_bytes()) != digest}
    missing = [name for name in baseline if not (root / name).exists()]
    assert not missing, f"Baseline files missing: {missing}"
    protected = [name for name in baseline if name == "AGENTS.md"
                 or name.startswith((".github/", "scripts/", "docs/history/", "src-tauri/capabilities/"))
                 or name in {"src-tauri/tauri.conf.json", "src-tauri/approved_runtimes.json",
                             "package.json", "package-lock.json", "src-tauri/Cargo.toml", "src-tauri/Cargo.lock"}]
    assert "AGENTS.md" in protected
    assert not set(protected).intersection(changed), "Protected input changed"
    extra = ["agents_feedback.md", "src/screens/AboutScreen.test.tsx", "src/screens/ProfileScreen.test.tsx"]
    names = sorted(set(changed).union(name for name in extra if (root / name).exists() and name not in baseline))
    parts = []
    for name in names:
        before = archive.read(name).decode("utf-8").replace("\r\n", "\n").splitlines(keepends=True) if name in baseline else []
        after = (root / name).read_text(encoding="utf-8").splitlines(keepends=True)
        parts.extend(difflib.unified_diff(before, after, fromfile=f"baseline/{name}", tofile=f"working/{name}"))
    delta = "".join(parts)
    (out / "audit-change.diff").write_text(delta, encoding="utf-8")
    sources = {name: sha256((root / name).read_bytes()) for name in names}
    (out / "changed-source-sha256.json").write_text(json.dumps(sources, indent=2) + "\n", encoding="utf-8")

artifacts = json.loads((out / "built-artifacts.json").read_text(encoding="utf-8-sig"))
assert len(artifacts) == 3 and all(isinstance(item, dict) for item in artifacts)
for item in artifacts:
    file = Path(item["path"])
    assert file.stat().st_size == item["bytes"]
    assert sha256(file.read_bytes()) == item["sha256"]
    assert item["version"] == "0.6.0"

native = json.loads((out / "native-ui.json").read_text(encoding="utf-8"))
assert native["status"] == native["cleanup"] == "PASS"
assert len(native["checks"]) == 13 and all(item["status"] == "PASS" for item in native["checks"])
assert native["sha256"] == artifacts[0]["sha256"]
assert "A11Y_PASS" in (out / "a11y.log").read_text(encoding="utf-8")
assert "585 passed; 0 failed; 6 ignored" in (out / "rust-final.log").read_text(encoding="utf-8")
frontend = (out / "frontend-final-checked.log").read_text(encoding="utf-8")
assert "198 passed" in frontend and "pass 274" in frontend

patterns = {
    "literal-secret": r"(?i)(api_key|secret|password|token|passwd)\s*=\s*['\"][^'\"]{6,}['\"]",
    "shell-injection": r"os\.system\(|shell\s*=\s*True",
    "dynamic-eval": r"\beval\(|\bexec\(",
    "unsafe-deserialization": r"pickle\.loads?\(",
}
scan = []
current = ""
for number, line in enumerate(delta.splitlines(), 1):
    if line.startswith("+++ working/"):
        current = line.removeprefix("+++ working/")
    if current.startswith(("src/", "src-tauri/src/")) and line.startswith("+") and not line.startswith("+++"):
        for kind, pattern in patterns.items():
            if re.search(pattern, line):
                scan.append({"kind": kind, "path": current, "diffLine": number})

report = {
    "scope": "Working-tree audit; not release qualification",
    "recordedAt": datetime.now(timezone.utc).isoformat(),
    "status": "PARTIALLY VERIFIED",
    "source": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
    "baselineArchive": "PASS",
    "protectedFiles": {"status": "PASS", "count": len(protected), "paths": protected},
    "agentsSha256": sha256((root / "AGENTS.md").read_bytes()),
    "changedFiles": names,
    "frontend": {"status": "PASS", "scriptTests": 274, "vitestTests": 198},
    "rust": {"status": "PASS", "passed": 585, "failed": 0, "ignored": 6},
    "packagedUi": {"status": "PASS", "checks": len(native["checks"]), "sha256": native["sha256"]},
    "accessibility": "PASS",
    "artifacts": artifacts,
    "staticScan": {"status": "PASS" if not scan else "REVIEW", "candidates": scan,
                   "coverage": "Heuristic scan of added source lines only; not a full security audit"},
    "openFindings": "research/ideas-audit-20260919/review-findings.md",
    "notExecuted": ["Clean-checkout release qualification", "Installer upgrade/preservation campaign",
                    "New inference-method or quality benchmarks", "Signing or publication"],
}
(out / "final-readback.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps({key: value for key, value in report.items() if key not in ("protectedFiles", "artifacts")}, indent=2))
print(f"Protected source files unchanged: {len(protected)}")
assert not scan, "Added-source security scan requires review"
