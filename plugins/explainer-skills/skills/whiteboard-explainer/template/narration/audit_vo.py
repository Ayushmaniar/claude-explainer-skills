"""Voice audit: transcribe every cached clip WITHOUT the script as a hint and diff it against the script.

    python -I narration/audit_vo.py [out.json]       (default narration/audit.json; run after build_vo.py)

build_vo.py aligns words with the script as Whisper's prompt, which hides mispronunciations; this unprompted pass
(large-v3-turbo) shows what a listener actually hears. Ignore US/UK spellings and homophones; fix real misreads by
respelling the line in script.py (try variants with tts_try.py first), then build the voice again.
AUDIT_SCENES=Hook,Idea limits it to some scenes. Needs faster-whisper; WHISPER_DEVICE=cpu|cuda|auto as in tts.py.
"""
import difflib
import hashlib
import importlib.util
import json
import os
import pathlib
import re
import sys

import numpy as np
import soundfile as sf

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
ROOT = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "narration"))
import tts  # noqa: E402

OUT = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "narration" / "audit.json"
spec = importlib.util.spec_from_file_location("script", ROOT / "narration" / "script.py")
script = importlib.util.module_from_spec(spec)
spec.loader.exec_module(script)


def cpath(text):
    key = hashlib.sha1(f"{json.dumps(tts.VOICE, sort_keys=True)}|{text}".encode()).hexdigest()[:16]
    return ROOT / "narration" / "cache" / f"{key}.wav"


ONES = "zero one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen".split()
TENS = "_ _ twenty thirty forty fifty sixty seventy eighty ninety".split()


def n2w(n):
    n = int(n)
    if n < 20:
        return ONES[n]
    if n < 100:
        return TENS[n // 10] + ("" if n % 10 == 0 else " " + ONES[n % 10])
    if n < 1000:
        return ONES[n // 100] + " hundred" + ("" if n % 100 == 0 else " " + n2w(n % 100))
    if n < 1000000:
        return n2w(n // 1000) + " thousand" + ("" if n % 1000 == 0 else " " + n2w(n % 1000))
    if n < 1000000000:
        return n2w(n // 1000000) + " million" + ("" if n % 1000000 == 0 else " " + n2w(n % 1000000))
    return str(n)


def norm(s):
    """Lowercase words with numbers spelled out, so "21" and "twenty-one" compare equal."""
    s = s.lower().replace("%", " percent").replace("&", " and ").replace("×", " times")
    s = re.sub(r"(\d),(\d{3})", r"\1\2", s)
    s = re.sub(r"(\d+)\.(\d+)", lambda m: n2w(m.group(1)) + " point " + " ".join(ONES[int(c)] for c in m.group(2)), s)
    s = re.sub(r"\d+", lambda m: n2w(m.group(0)), s)
    s = s.replace("-", " ")
    return [w for w in (re.sub(r"[^a-z0-9']", "", x).replace("'", "") for x in s.split()) if w]


m = tts.whisper("mobiuslabsgmbh/faster-whisper-large-v3-turbo")
out = []
ONLY = [x for x in os.environ.get("AUDIT_SCENES", "").split(",") if x]
for scene, beats in script.SCENES:
    if ONLY and scene not in ONLY:
        continue
    for b in beats:
        if b == script.PAUSE:
            continue
        if not cpath(b).exists():
            print(f"[{scene}] not voiced yet (run build_vo.py): {b[:60]}")
            continue
        a, sr = sf.read(cpath(b), dtype="float32")
        if a.ndim > 1:
            a = a.mean(1)
        a16 = np.interp(np.arange(0, len(a), sr / 16000), np.arange(len(a)), a).astype(np.float32)
        segs, _ = m.transcribe(a16, language="en", beam_size=5, condition_on_previous_text=False)
        heard = " ".join(s.text for s in segs).strip()
        A, B = norm(b), norm(heard)
        sm = difflib.SequenceMatcher(a=A, b=B, autojunk=False)
        diffs = [(" ".join(A[i1:i2]), " ".join(B[j1:j2])) for op, i1, i2, j1, j2 in sm.get_opcodes() if op != "equal"]
        out.append({"scene": scene, "text": b, "heard": heard, "diffs": diffs, "dur": len(a) / sr})
        if diffs:
            print(f"[{scene}] {diffs}  || {heard}", flush=True)
OUT.write_text(json.dumps(out, indent=1, ensure_ascii=False), encoding="utf-8")
print("lines", len(out), "with diffs", sum(1 for o in out if o["diffs"]), "->", OUT)
