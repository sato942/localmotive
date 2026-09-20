from pathlib import Path
import difflib
import hashlib
import json
import re
import zipfile

root = Path.cwd()
out = root / "artifacts/benchmark-statistics-audit-20260920-0349"
paths = ["src-tauri/src/core.rs", "src-tauri/src/measurement.rs", "src-tauri/src/tune.rs"]
diffs = []
with zipfile.ZipFile(out / "baseline.zip") as archive:
    for path in paths:
        old = archive.read(path).decode().splitlines(keepends=True)
        new = (root / path).read_text().splitlines(keepends=True)
        diffs.extend(difflib.unified_diff(old, new, fromfile="before/" + path, tofile="after/" + path))
diff = "".join(diffs)
(out / "source.diff").write_text(diff)
(out / "reviewed-sha256.json").write_text(json.dumps({p: hashlib.sha256((root / p).read_bytes()).hexdigest() for p in paths}, indent=2) + "\n")
added = "\n".join(line[1:] for line in diff.splitlines() if line.startswith("+") and not line.startswith("+++"))
patterns = {"unsafe": r"\bunsafe\b", "process_execution": r"Command::|hidden_command|\.spawn\(", "new_network": r"reqwest|TcpStream|TcpListener", "credential_literals": r"(?i)(?:api_key|secret|password)\s*[:=]\s*\"[^\"]{6,}\""}
scan = {name: len(re.findall(pattern, added)) for name, pattern in patterns.items()}
(out / "static-scan.json").write_text(json.dumps(scan, indent=2) + "\n")
print(json.dumps({"diffCharacters": len(diff), "scan": scan}))
