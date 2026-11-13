"""Finds frames whose pixels change and snap back within 1-4 frames (render glitches: pasted bands, streaks).

    python -I narration/glitch.py out.mp4      (needs ffmpeg on PATH, or FFMPEG / FFMPEG_DIR as in tts.py)

Checks the encoded file at 480x270. The host's blinks and rolling counters give some harmless events; look at the
flagged frames (`<binary> frame <seconds>s`) before deciding. Prints (frame, frames until it snaps back, changed
pixels, y range) for each event.
"""
import os
import pathlib
import shutil
import subprocess
import sys
from collections import deque

import numpy as np


def ffmpeg():
    if os.environ.get("FFMPEG"):
        return os.environ["FFMPEG"]
    if os.environ.get("FFMPEG_DIR"):
        for name in ("ffmpeg.exe", "ffmpeg"):
            p = pathlib.Path(os.environ["FFMPEG_DIR"]) / "bin" / name
            if p.is_file():
                return str(p)
    return shutil.which("ffmpeg") or "ffmpeg"


src = sys.argv[1]
W, H = 480, 270
N = W * H
p = subprocess.Popen([ffmpeg(), "-v", "error", "-i", src, "-vf", f"scale={W}:{H}", "-f", "rawvideo", "-pix_fmt", "gray", "-"],
                     stdout=subprocess.PIPE)
win = deque(maxlen=6)
i = -1
hits = []
while True:
    b = p.stdout.read(N)
    if len(b) < N:
        break
    i += 1
    win.append(np.frombuffer(b, np.uint8).astype(np.int16))
    if len(win) < 6:
        continue
    # candidate frame c = i-4, previous = i-5: do the pixels that changed at c revert within 1..4 frames?
    prev, cur = win[0], win[1]
    ch = np.abs(cur - prev) > 40
    nch = ch.sum()
    if nch < 150:
        continue
    for k in range(2, 6):
        rev = ch & (np.abs(win[k] - prev) < 15)
        if rev.sum() > 0.7 * nch:
            rows = np.where(ch.reshape(H, W).any(1))[0]
            hits.append((i - 4, k - 1, int(nch), int(rows.min()) * 4, int(rows.max()) * 4))
            break
print("frames", i + 1, "transient events", len(hits))
for h in hits[:80]:
    print(h, f"{h[0] / 30:.2f}s")
