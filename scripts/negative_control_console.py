"""Negative control for watch_console_windows.py.

Spawns a windowed-app-style child WITHOUT CREATE_NO_WINDOW while the watcher runs,
to prove the watcher actually detects console windows (so a 'pass' result is meaningful).
"""

import json
import subprocess
import sys
import time

sys.path.insert(0, __file__.rsplit("\\", 1)[0].rsplit("/", 1)[0])

from watch_console_windows import console_windows  # noqa: E402

DETACHED_PROCESS = 0x00000008
CREATE_NEW_CONSOLE = 0x00000010


def main():
    exe = sys.argv[1]
    baseline = console_windows()
    proc = subprocess.Popen(
        [exe, "--version"],
        creationflags=CREATE_NEW_CONSOLE,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    appeared = {}
    deadline = time.time() + 6
    while time.time() < deadline:
        for hwnd, info in console_windows().items():
            if hwnd not in baseline:
                appeared[hwnd] = info
        if appeared:
            break
        time.sleep(0.02)
    try:
        proc.wait(timeout=10)
    except subprocess.TimeoutExpired:
        proc.kill()
    print(
        json.dumps(
            {
                "detector_saw_console_window": bool(appeared),
                "windows": appeared,
            },
            indent=2,
        )
    )
    return 0 if appeared else 1


if __name__ == "__main__":
    raise SystemExit(main())
