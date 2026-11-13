"""Subtitles (.srt) from narration whose line timings are known. Standard library only.

As a module:
    from srt import to_srt
    text = to_srt([(start_s, end_s, "spoken line"), ...], display=[("D I L", "DIL"), ...])

As a script, from a TSV of start<TAB>end<TAB>text (seconds):
    python srt.py timings.tsv -o video.srt [--display display.tsv]

The voice script is often spelled for the voice ("D I L", respelled names, numbers in words), so
`display` maps spoken spellings back to what should be read, applied in order (longest first).
Each line is split into cues of at most two lines of 42 characters, at sentence breaks, then clause
breaks, then words. Each cue gets a share of the line's spoken span in proportion to its length.
"""

import argparse
import pathlib
import re

LINE = 42  # characters per subtitle line (the usual broadcast limit)
MAX = 80  # characters per cue; a little under 2 * LINE so a fitting line break usually exists


def pieces(text: str) -> list[str]:
    """Sentences, with long ones cut at clause breaks and then at words, each <= MAX characters."""
    out = []
    for sentence in re.split(r"(?<=[.?!])\s+", text.strip()):
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


def to_srt(lines: list[tuple[float, float, str]], display: list[tuple[str, str]] = ()) -> str:
    """lines: (start, end, text) per spoken clip, in seconds of video time."""
    entries = []
    for start, end, text in lines:
        for spoken, shown in display:
            text = text.replace(spoken, shown)
        parts = pieces(text)
        total = sum(len(p) for p in parts)
        t = start
        for p in parts:
            dt = (end - start) * len(p) / total
            entries.append((t, t + dt, wrap(p)))
            t += dt
    return "".join(f"{n}\n{stamp(a)} --> {stamp(b)}\n{text}\n\n" for n, (a, b, text) in enumerate(entries, 1))


def read_display(path: str | None) -> list[tuple[str, str]]:
    if not path:
        return []
    rows = [ln.split("\t") for ln in pathlib.Path(path).read_text(encoding="utf-8").splitlines() if "\t" in ln]
    return sorted(((a, b) for a, b in rows), key=lambda r: -len(r[0]))


if __name__ == "__main__":
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("timings", help="TSV: start<TAB>end<TAB>text, seconds")
    ap.add_argument("-o", "--out", required=True)
    ap.add_argument("--display", help="TSV: spoken spelling<TAB>display spelling")
    args = ap.parse_args()
    rows = [ln.split("\t", 2) for ln in pathlib.Path(args.timings).read_text(encoding="utf-8").splitlines() if ln.strip()]
    pathlib.Path(args.out).write_text(to_srt([(float(a), float(b), t) for a, b, t in rows], read_display(args.display)), encoding="utf-8")
