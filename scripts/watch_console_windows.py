"""Poll for console windows while GGUF Pilot runs its child-process probes.

Fails loudly if any new console window (ConsoleWindowClass / CASCADIA_HOSTING_WINDOW_CLASS)
appears while the app is spawning nvidia-smi, powershell, and llama-server --help.
"""

import ctypes
import ctypes.wintypes as wt
import json
import sys
import time

user32 = ctypes.WinDLL("user32", use_last_error=True)

EnumWindows = user32.EnumWindows
EnumWindows.argtypes = [ctypes.WINFUNCTYPE(wt.BOOL, wt.HWND, wt.LPARAM), wt.LPARAM]
GetClassNameW = user32.GetClassNameW
GetWindowTextW = user32.GetWindowTextW
IsWindowVisible = user32.IsWindowVisible

CONSOLE_CLASSES = {
    "ConsoleWindowClass",
    "CASCADIA_HOSTING_WINDOW_CLASS",
    "PseudoConsoleWindow",
}


def console_windows():
    found = {}

    @ctypes.WINFUNCTYPE(wt.BOOL, wt.HWND, wt.LPARAM)
    def callback(hwnd, _lparam):
        buf = ctypes.create_unicode_buffer(256)
        GetClassNameW(hwnd, buf, 256)
        cls = buf.value
        if cls in CONSOLE_CLASSES:
            title = ctypes.create_unicode_buffer(512)
            GetWindowTextW(hwnd, title, 512)
            found[int(hwnd)] = {
                "class": cls,
                "title": title.value,
                "visible": bool(IsWindowVisible(hwnd)),
            }
        return True

    EnumWindows(callback, 0)
    return found


def main():
    duration = float(sys.argv[1]) if len(sys.argv) > 1 else 25.0
    baseline = console_windows()
    appeared = {}
    deadline = time.time() + duration
    polls = 0
    while time.time() < deadline:
        for hwnd, info in console_windows().items():
            if hwnd not in baseline and hwnd not in appeared:
                appeared[hwnd] = info
        polls += 1
        time.sleep(0.05)
    print(
        json.dumps(
            {
                "polls": polls,
                "baseline_console_windows": len(baseline),
                "new_console_windows": appeared,
                "pass": not appeared,
            },
            indent=2,
        )
    )
    return 0 if not appeared else 1


if __name__ == "__main__":
    raise SystemExit(main())
