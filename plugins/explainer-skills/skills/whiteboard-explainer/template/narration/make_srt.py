"""Write subtitles for the rendered video from the narration script and its cue times.

    python -I narration/make_srt.py [out.srt] [--review]      (default out/video.srt; run after build_vo.py)

Put the .srt next to the mp4 with the same base name, so players pick it up automatically.
Reads narration/script.py and src/cues.rs (beat start/end times). The script is spelled for the voice (numbers in
words, spaced letters, respelled names), so the subtitles map it back:
  - DISPLAY: (spoken, shown) replacements applied in order, longer phrases first;
  - OVERRIDE: whole lines that are easier to rewrite than to patch (exact script text -> shown text).
Each beat is split into cues of at most two 42-character lines, timed in proportion to their length within the
beat's spoken span. --review prints every line DISPLAY or OVERRIDE changed, as it will be shown.

For a long video with many numbers, an alternative is to start from the voice audit's unprompted transcript
(narration/audit.json, which already writes "21" and "2.1 million") and fix names with regexes; DISPLAY is
simpler while the script is short.
"""

import importlib.util
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
args = [a for a in sys.argv[1:] if not a.startswith("--")]
OUT = pathlib.Path(args[0]) if args else ROOT / "out" / "video.srt"
LINE = 42
MAX = 80  # characters per cue; a little under 2 * LINE so a fitting line break usually exists

# Voice spelling -> display spelling, applied in order (longer phrases first).
DISPLAY = [
    ("one million", "1,000,000"), ("two million", "2,000,000"),
    ("twenty-one", "21"), ("twenty-three", "23"), ("sixty-three", "63"), ("fifty-two", "52"),
    ("twenty", "20"), ("forty", "40"),
]

OVERRIDE = {
    # "exact script line": "shown text",
}

spec = importlib.util.spec_from_file_location("script", ROOT / "narration" / "script.py")
script = importlib.util.module_from_spec(spec)
spec.loader.exec_module(script)


def floats(s: str) -> list[float]:
    return [float(x) for x in s.split(",") if x.strip()]


cues_rs = (ROOT / "src" / "cues.rs").read_text(encoding="utf-8")
timing = [
    (name, float(start), floats(beats), floats(ends))
    for name, start, beats, ends in re.findall(
        r'name: "(\w+)", start: ([\d.]+), dur: [\d.]+, beats: &\[([^\]]*)\], ends: &\[([^\]]*)\]', cues_rs
    )
]
if [t[0] for t in timing] != [name for name, _ in script.SCENES]:
    raise SystemExit("src/cues.rs doesn't match narration/script.py; run build_vo.py first")


def display(text: str) -> str:
    if text in OVERRIDE:
        return OVERRIDE[text]
    for spoken, shown in DISPLAY:
        # whole words only, and keep a capital at the start of a sentence ("Forty is" -> "40 is")
        text = re.sub(rf"(?<![\w-]){re.escape(spoken)}(?![\w-])", shown, text, flags=re.I)
    return text


def pieces(text: str) -> list[str]:
    """Sentences, with long ones cut at clause breaks and then at words, each <= 2 lines."""
    out = []
    for sentence in re.split(r"(?<=[.?!])\s+", text):
        if len(sentence) <= MAX:
            out.append(sentence)
            continue
        chunk = ""
        for part in re.split(r"(?<=[,:;])\s+", sentence):
            for word in part.split(" ") if len(part) > MAX else [part]:
                if chunk and len(chunk) + 1 + len(word) > MAX:
                    out.append(chunk)
                    chunk = word
                else:
                    chunk = f"{chunk} {word}".strip()
        out.append(chunk)
    # Join short neighbours ("A slider. A live price.") so cues don't flash by.
    merged = []
    for p in out:
        if merged and len(merged[-1]) + 1 + len(p) <= MAX and min(len(merged[-1]), len(p)) < 30:
            merged[-1] += " " + p
        else:
            merged.append(p)
    return merged


def wrap(text: str) -> str:
    if len(text) <= LINE:
        return text
    spaces = [i for i, c in enumerate(text) if c == " "]
    # Prefer a break where both lines fit, then after punctuation, then near the middle.
    cut = min(spaces, key=lambda i: (max(0, max(i, len(text) - i - 1) - LINE), text[i - 1] not in ",.:;?!", abs(i - len(text) / 2)))
    return text[:cut] + "\n" + text[cut + 1:]


def stamp(t: float) -> str:
    ms = round(t * 1000)
    return f"{ms // 3600000:02d}:{ms // 60000 % 60:02d}:{ms // 1000 % 60:02d},{ms % 1000:03d}"


entries = []
for (name, beats), (_, start, starts, ends) in zip(script.SCENES, timing):
    for beat, a, b in zip(beats, starts, ends):
        if beat == script.PAUSE:
            continue
        shown = display(beat)
        if "--review" in sys.argv and shown != beat:
            print(f"[{name}] {ascii(shown)}")
        parts = pieces(shown)
        total = sum(len(p) for p in parts)
        t = start + a
        for p in parts:
            dt = (b - a) * len(p) / total
            entries.append((t, t + dt, wrap(p)))
            t += dt

# Spellings for the voice that probably should not be shown (spaced capitals, numbers in words).
leftover = [e[2] for e in entries if re.search(
    r"\b[A-Z] [A-Z]\b|\bpercent\b|\bpoint (one|two|three|four|five|six|seven|eight|nine)\b|\b(thirty|forty|fifty|sixty|seventy|eighty|ninety)-", e[2])]
if leftover:
    print("warning: possible voice spellings left:", *[ascii(x) for x in leftover], sep="\n  ")
OUT.parent.mkdir(parents=True, exist_ok=True)
OUT.write_text(
    "".join(f"{n}\n{stamp(a)} --> {stamp(b)}\n{text}\n\n" for n, (a, b, text) in enumerate(entries, 1)),
    encoding="utf-8",
)
print(f"{len(entries)} subtitles -> {OUT}")
