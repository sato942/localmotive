from pathlib import Path
import datetime
import hashlib
import json
import subprocess
import zipfile

root = Path.cwd()
out = root / "artifacts/benchmark-statistics-audit-20260920-0349"
scoped = ["src-tauri/src/core.rs", "src-tauri/src/measurement.rs", "src-tauri/src/tune.rs", "agents_feedback.md", "CHANGELOG.md", "TODO-0.6.md"]
paths = subprocess.check_output(["git", "ls-files", "-z"]).decode().split("\0")
hashes = {p: hashlib.sha256((root / p).read_bytes()).hexdigest() for p in paths if p and (root / p).is_file() and not Path(p).name.startswith(".env")}
record = {"source": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(), "recordedAt": datetime.datetime.now().astimezone().isoformat(), "sha256": hashes, "scope": scoped}
assert hashes["AGENTS.md"] == "6a0c8eea9a3cf9ed680ea6ccb7814a59ed8d79365dd8f1a2229a1bf6dee27c4e"
assert not (out / "baseline.zip").exists()
with zipfile.ZipFile(out / "baseline.zip", "w", zipfile.ZIP_DEFLATED) as archive:
    for name in scoped:
        archive.write(root / name, name)
(out / "baseline-sha256.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"source": record["source"], "filesHashed": len(hashes), "filesArchived": len(scoped)}))
