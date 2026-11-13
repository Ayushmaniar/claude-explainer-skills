"""Try spellings: voices each line of a text file and prints what Whisper hears (unprompted).

    python -I narration/tts_try.py variants.txt

Test a tricky name inside the whole sentence it will appear in: a word that is fine alone can come out wrong
mid-sentence. The takes are not cached (the TTS is not deterministic: try each variant two or three times).
"""
import pathlib
import sys

import numpy as np

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
ROOT = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "narration"))
import tts  # noqa: E402

texts = [ln.strip() for ln in open(sys.argv[1], encoding="utf-8") if ln.strip()]
clips = tts.synth(texts)
m = tts.whisper("mobiuslabsgmbh/faster-whisper-large-v3-turbo")
for t, a in zip(texts, clips):
    a16 = np.interp(np.arange(0, len(a), tts.SR / 16000), np.arange(len(a)), a).astype(np.float32)
    heard = " ".join(s.text for s in m.transcribe(a16, language="en", beam_size=5, condition_on_previous_text=False)[0]).strip()
    print(f"SAID : {t}\nHEARD: {heard}\n")
