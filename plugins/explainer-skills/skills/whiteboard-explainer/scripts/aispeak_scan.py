"""Flag AI-speak in a narration script (see references/natural-narration.md).

Usage:  python -I aispeak_scan.py path/to/narration/script.py   (a SCENES list of (name, [beats]))
        python -I aispeak_scan.py notes.txt                      (any text file: one line per sentence group)
Prints each finding as [Scene.beat] pattern: line, then a summary with counts per pattern. Findings are
candidates, not verdicts: a contrast that corrects a real belief, a quiz trap or a real list of three stays.
"""
import importlib.util
import pathlib
import re
import sys

WPM = 150  # narration speed, for the questions-per-minute check

STRONG = {
    "staged run-up": r"\bhere'?s (the|a|an|his|her|their|what|why|how)\b|\bthe (key|big|real) (picture|question|idea|point|thing) is\b"
                     r"|^(now,? )?the big question\b|\bthe crux\b|\bbecause the details matter\b|\blet'?s (dive|explore|break)\b",
    "not X but Y": r"\bnot just\b|\bnot only\b|\bisn'?t [^.?!]{1,40}\. (it|the [a-z]+) (is|'s)\b|\bdoesn'?t just\b"
                   r"|\b(was|is) [^.?!]{1,30}, not (the|a|an) [a-z]+\.?$",
    "quotable closer": r"\b(that'?s|this is) (the honest|science|what [a-z]+ (really|actually)|the real)\b|\bturns a [a-z]+ into an?\b"
                       r"|\bnot a law of\b|\bthe rest is history\b|\band that'?s (fine|okay|ok)\.?$",
    "staged candor": r"\b(honestly|frankly|let'?s be (clear|honest)|to be clear|make no mistake)\b",
}
WEAK = {
    "self-answered question": r"\?\s+\S",
    "inflated stakes": r"\b(best [a-z]+ in all of|strongest [a-z]+ there is|scary|astronomical\w*|game.?changer|revolution\w*"
                       r"|groundbreaking|mind.?blowing|incredibl\w+|unprecedented)\b",
    "meta-narration": r"\bas we'?ll see\b|\bthis (whole )?video is about\b|\bkeep this [a-z]+ in mind\b|\blet'?s look at\b",
    "adverb crutch": r"\b(actually|really|simply|truly|basically|exactly|literally|genuinely)\b",
    "AI vocabulary": r"\b(delve|pivotal|crucial|testament|tapestry|underscore[sd]?|showcas\w+|vibrant|robust|seamless\w*|leverag\w+"
                     r"|realm|landscape|intricate|meticulous\w*|additionally|moreover|furthermore|notably)\b",
    "em dash": r"—|–| -- ",
}


def load(path):
    p = pathlib.Path(path)
    if p.suffix == ".py":
        spec = importlib.util.spec_from_file_location("script", p)
        m = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(m)
        return [(n, i, b) for n, beats in m.SCENES for i, b in enumerate(beats) if isinstance(b, str) and not b.startswith("__")]
    return [("text", i, ln.strip()) for i, ln in enumerate(p.read_text(encoding="utf-8").splitlines()) if ln.strip()]


def main():
    lines = load(sys.argv[1])
    words = sum(len(b.split()) for _, _, b in lines)
    counts = {}
    for name, i, b in lines:
        for group, pats in (("STRONG", STRONG), ("weak", WEAK)):
            for k, pat in pats.items():
                if re.search(pat, b, re.I):
                    counts[k] = counts.get(k, 0) + 1
                    if group == "STRONG" or k in ("inflated stakes", "AI vocabulary", "em dash"):
                        print(f"[{name}.{i}] {k}: {ascii(b[:160])}")
    starts = {}
    for _, _, b in lines:
        w = re.match(r"[A-Za-z']+", b)
        if w:
            starts[w.group(0)] = starts.get(w.group(0), 0) + 1
    minutes = words / WPM
    print(f"\n{len(lines)} lines, {words} words, ~{minutes:.0f} min at {WPM} wpm")
    for k in list(STRONG) + list(WEAK):
        print(f"  {k:24s} {counts.get(k, 0)}")
    q = counts.get("self-answered question", 0)
    print(f"  self-answered questions per 3 min: {3 * q / max(minutes, 1):.1f}  (aim <= 1)")
    top = sorted(starts.items(), key=lambda kv: -kv[1])[:6]
    print("  most common first words:", ", ".join(f"{w} {n}" for w, n in top), " (vary them)")


if __name__ == "__main__":
    main()
