"""Objective prosody checks for TTS takes of eval/lines.txt (needs numpy + librosa).

    python -I voice_metrics.py a.wav b.wav ...      (pip install numpy librosa)
    (no arguments: scores the reference recordings in ../eval)

Flat pitch is what sounds robotic, so pitch movement (semitone sd and 90% range) is the main
signal; pace and pauses catch rushed delivery; a duration far off the others means dropped or
invented words. Numbers only shortlist candidates: decide by ear.
"""

import pathlib
import sys

import librosa
import numpy as np

EVAL = pathlib.Path(__file__).resolve().parent.parent / "eval"
WORDS = len((EVAL / "lines.txt").read_text(encoding="utf-8").split())
GAPS = 2 * 0.35  # say.py's default gap between the three lines

paths = [pathlib.Path(p) for p in sys.argv[1:]] or sorted(EVAL.glob("*.wav"))
for path in paths:
    y, sr = librosa.load(path, sr=16000)
    dur = len(y) / sr
    f0, voiced, _ = librosa.pyin(y, fmin=60, fmax=400, sr=sr, frame_length=1024, hop_length=160)
    f0 = f0[voiced & ~np.isnan(f0)]
    semis = 12 * np.log2(f0 / np.median(f0))
    spread = float(np.percentile(semis, 95) - np.percentile(semis, 5))
    rms = librosa.feature.rms(y=y, frame_length=400, hop_length=160)[0]
    runs, n = [], 0
    for q in rms < 0.02 * rms.max():
        if q:
            n += 1
        elif n:
            runs.append(n * 0.01)
            n = 0
    pauses = [r for r in runs if r >= 0.15]
    print(
        f"{path.stem:30s} {dur:5.1f}s  {WORDS / (dur - GAPS) * 60:4.0f} wpm  pitch sd {np.std(semis):4.2f} st, "
        f"90% range {spread:4.1f} st  pauses {len(pauses):2d}  long>1.2s: {sum(r > 1.2 for r in pauses)}"
    )
