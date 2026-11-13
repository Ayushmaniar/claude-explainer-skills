"""Runs a command at the lowest CPU priority (its child processes inherit it), so another app keeps the machine
responsive while a voice build or a render runs.

    python -I tools/lowprio.py cargo run --release -- render -o out/video.mp4
"""
import os
import subprocess
import sys

if os.name == "nt":
    import ctypes

    IDLE_PRIORITY_CLASS = 0x40
    ctypes.windll.kernel32.SetPriorityClass(ctypes.windll.kernel32.GetCurrentProcess(), IDLE_PRIORITY_CLASS)
else:
    os.nice(19)
sys.exit(subprocess.run(sys.argv[1:]).returncode)
