"""Build the voice-over, per-word timings and the cue table.

    python -I narration/build_vo.py          (from the project folder; needs numpy soundfile librosa faster-whisper)

Voice: the local-tts skill's say.py (Qwen3-TTS, run locally; see narration/tts.py for how it is found and for the
voice settings). Each narration line ("beat") is generated once and cached in narration/cache by a hash of the voice
settings + text; then faster-whisper finds when each word is spoken inside the clip and the words are aligned back
to the script text, so scenes can start a drawing on the exact word that names it. Over-long pauses inside a line
are shortened (`shape`), so the voice never stops mid-thought for as long as the gap between lines.

Writes audio/narration.wav, src/cues.rs (scene starts, durations, beat starts/ends and every word's time),
src/envelope.rs (voice loudness per frame, for the host's talking bob) and narration/SCRIPT.md.
Then: narration/words.py (checks every c.w(..) key), and later narration/make_srt.py for subtitles.

If another app is using the GPU: GGML_BACKEND=CPU (TTS on the CPU, ~4x slower) and WHISPER_DEVICE=cpu.
"""

import difflib
import hashlib
import importlib.util
import json
import math
import pathlib
import re
import runpy
import subprocess
import sys

import numpy as np
import soundfile as sf
import asyncio  # noqa: F401  imported up front: on Windows a first import after the TTS process ran failed with WinError 10106
import librosa.effects  # noqa: F401  (lazy-loaded otherwise, which imports asyncio late)

ROOT = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "narration"))  # `python -I` doesn't put the script's folder on sys.path
import tts  # noqa: E402

FFMPEG = tts.ffmpeg_exe()
SR = tts.SR
FPS = 30
VOICE = tts.VOICE

LEAD = 0.35      # silence before a scene's first beat
GAP = 0.5        # silence between beats (each clip also keeps 50 ms of its own silence at both ends)
GAP_Q = 0.7      # after a question: let it land
PAUSE_LEN = 8.0  # quiz thinking time (MCQ: read four options)
TAIL = 0.5       # silence after a scene's last beat
BATCH = 32       # lines per TTS process (the model loads once per process)
TEMPO = 1.06     # speed-up of every clip (pitch kept, ffmpeg atempo): ~155 wpm

spec = importlib.util.spec_from_file_location("script", ROOT / "narration" / "script.py")
script = importlib.util.module_from_spec(spec)
spec.loader.exec_module(script)


def cache_path(text: str, ext: str = ".wav") -> pathlib.Path:
    key = hashlib.sha1(f"{json.dumps(VOICE, sort_keys=True)}|{text}".encode()).hexdigest()[:16]
    return ROOT / "narration" / "cache" / f"{key}{ext}"


def synth(texts: list[str]) -> list[np.ndarray]:
    return tts.synth(texts, VOICE)


def finish(samples: np.ndarray, text: str) -> tuple[np.ndarray, bool]:
    """Trim to the speech plus 50 ms; say whether the length fits the text (~0.07 s per char)."""
    import librosa
    _, (a, b) = librosa.effects.trim(samples, top_db=40)
    pad = int(0.05 * SR)
    samples = samples[max(0, a - pad): b + pad]
    per_char = len(samples) / SR / len(text)
    return samples, 0.045 <= per_char <= 0.11


def fill_cache(texts: list[str]) -> None:
    todo = sorted({t for t in texts if not cache_path(t).exists()}, key=len)
    for attempt in range(3):
        retry = []
        for i in range(0, len(todo), BATCH):
            chunk = todo[i:i + BATCH]
            for text, samples in zip(chunk, synth(chunk)):
                samples, ok = finish(samples, text)
                if ok or attempt == 2:
                    cache_path(text).parent.mkdir(exist_ok=True)
                    sf.write(cache_path(text), samples, SR)
                else:
                    print(f"  retake {attempt + 1}: {len(samples) / SR / len(text):.3f} s/char: {text[:60]}")
                    retry.append(text)
            print(f"  {min(i + BATCH, len(todo))}/{len(todo)} lines", flush=True)
        todo = retry
        if not todo:
            break


def key(word: str) -> str:
    return re.sub(r"[^a-z0-9]", "", word.lower())


_whisper = None


def align(text: str) -> list[tuple[str, float, float]]:
    """(script word, start, end) for every whitespace word of `text`, in clip seconds."""
    path = cache_path(text, ".words.json")
    if path.exists():
        return [tuple(x) for x in json.loads(path.read_text(encoding="utf-8"))]
    global _whisper
    if _whisper is None:
        _whisper = tts.whisper("small")
    audio, _ = sf.read(cache_path(text), dtype="float32")
    import librosa
    audio16 = librosa.resample(audio, orig_sr=SR, target_sr=16000)
    segs, _ = _whisper.transcribe(audio16, language="en", word_timestamps=True, initial_prompt=text)
    heard = [(key(w.word), w.start, w.end) for s in segs for w in s.words if key(w.word)]
    words = text.split()
    keys = [key(w) for w in words]
    times: list[tuple[float, float] | None] = [None] * len(words)
    sm = difflib.SequenceMatcher(a=keys, b=[h[0] for h in heard], autojunk=False)
    for blk in sm.get_matching_blocks():
        for j in range(blk.size):
            times[blk.a + j] = heard[blk.b + j][1:]
    # Unmatched words (numbers said as words, misheard names): spread them by length between neighbours.
    dur = len(audio) / SR
    i = 0
    while i < len(words):
        if times[i] is not None:
            i += 1
            continue
        j = i
        while j < len(words) and times[j] is None:
            j += 1
        lo = times[i - 1][1] if i > 0 else 0.05
        hi = times[j][0] if j < len(words) else dur - 0.05
        hi = max(hi, lo + 0.05)
        lens = [len(w) + 1 for w in words[i:j]]
        acc = lo
        for k, n in zip(range(i, j), lens):
            span = (hi - lo) * n / sum(lens)
            times[k] = (acc, acc + span)
            acc += span
        i = j
    out = [(w, round(a, 3), round(b, 3)) for w, (a, b) in zip(words, times)]
    path.write_text(json.dumps(out), encoding="utf-8", newline="\n")
    return out


def clip(text: str) -> np.ndarray:
    """The cached clip of a line, sped up by TEMPO (also cached)."""
    src = cache_path(text)
    if TEMPO == 1.0:
        return sf.read(src, dtype="float32")[0]
    fast = cache_path(text, f".x{TEMPO:.2f}.wav")
    if not fast.exists():
        subprocess.run([FFMPEG, "-hide_banner", "-loglevel", "error", "-y", "-i", str(src),
                        "-af", f"atempo={TEMPO}", "-ar", str(SR), str(fast)], check=True)
    return sf.read(fast, dtype="float32")[0]


# Longest silence kept inside a line, by the punctuation before it. The TTS sometimes pauses 0.6-0.8 s at a colon or
# mid-phrase, as long as the gap between lines, which makes the voice stop mid-thought and then rush on.
HOLD = {".": 0.42, "?": 0.42, "!": 0.42, ",": 0.24, ";": 0.3, ":": 0.3}
HOLD_NONE = 0.14


def shape(samples: np.ndarray, words: list[tuple[str, float, float]]) -> tuple[np.ndarray, list[tuple[str, float, float]]]:
    """Shorten over-long silences between words (clip seconds in, clip seconds out)."""
    hop, win = int(0.005 * SR), int(0.02 * SR)
    pad = np.pad(samples, (win // 2, win // 2))
    rms = np.sqrt(np.convolve(pad ** 2, np.ones(win) / win, "valid")[::hop])
    quiet = rms < np.percentile(rms, 95) * 10 ** (-40 / 20)
    cuts = []  # (start sample, samples removed)
    for (w, _, e), (_, a, _) in zip(words, words[1:]):
        hold = HOLD.get(w[-1:], HOLD_NONE)
        lo, hi = int(max(0.0, e - 0.08) * SR / hop), int((a + 0.08) * SR / hop)
        best, run, best_end = 0, 0, 0
        for i in range(lo, min(hi, len(quiet))):
            run = run + 1 if quiet[i] else 0
            if run > best:
                best, best_end = run, i + 1
        length = best * hop / SR
        if length > hold + 0.04:
            drop = int((length - hold) * SR)
            mid = (best_end - best) * hop + (best * hop - drop) // 2
            cuts.append((mid, drop))
    if not cuts:
        return samples, words
    fade = int(0.01 * SR)
    out, pos = [], 0
    for start, drop in cuts:
        out.append(samples[pos:start])
        pos = start + drop
        if out[-1].size > fade and pos + fade < len(samples):  # short crossfade over the seam
            ramp = np.linspace(0, 1, fade, dtype="float32")
            out[-1] = out[-1].copy()
            out[-1][-fade:] = out[-1][-fade:] * (1 - ramp) + samples[pos:pos + fade] * ramp
            pos += fade

    def remap(t: float) -> float:
        s, shift = t * SR, 0
        for start, drop in cuts:
            if s >= start + drop:
                shift += drop + fade
            elif s > start:
                shift += s - start
        return (s - shift) / SR

    out.append(samples[pos:])
    return np.concatenate(out), [(w, round(remap(a), 3), round(remap(b), 3)) for w, a, b in words]


def frames(seconds: float) -> float:
    return math.ceil(seconds * FPS - 1e-6) / FPS


beats_all = [b for _, beats in script.SCENES for b in beats if b != script.PAUSE]
fill_cache(beats_all)
for n, b in enumerate(beats_all):
    align(b)
    if n % 20 == 19:
        print(f"  aligned {n + 1}/{len(beats_all)}", flush=True)

scene_rows, track, cursor, shaped_s = [], [], 0.0, 0.0
md = ["# Narration script\n", f"Voice: local-tts `{VOICE['speaker']}` ({VOICE['quant']}), tempo {TEMPO}. Timestamps are video time.\n"]
for name, beats in script.SCENES:
    t, cues, ends, words = LEAD, [], [], []
    md.append(f"\n## {name}  ({int(cursor // 60)}:{cursor % 60:04.1f})\n")
    for beat in beats:
        cues.append(t)
        if beat == script.PAUSE:
            md.append(f"- *[{PAUSE_LEN:.0f} s pause to think]*\n")
            t += PAUSE_LEN
            ends.append(t)
            words.append([])
            continue
        samples, ws = shape(clip(beat), [(w, a / TEMPO, b / TEMPO) for w, a, b in align(beat)])
        shaped_s += (len(clip(beat)) - len(samples)) / SR
        track.append((cursor + t, samples))
        words.append([(w, t + a, t + b) for w, a, b in ws])
        md.append(f"- `{cursor + t:6.1f}s` {beat}\n")
        t += len(samples) / SR
        ends.append(t)
        t += GAP_Q if beat.endswith("?") else GAP
    dur = frames(t - GAP + TAIL)
    scene_rows.append((name, cursor, dur, cues, ends, words))
    cursor += dur

total = cursor
out = np.zeros(int(math.ceil(total * SR)) + SR, dtype="float32")
for start, samples in track:
    i = int(round(start * SR))
    out[i:i + len(samples)] += samples
out *= 0.89 / (float(np.max(np.abs(out))) or 1.0)
(ROOT / "audio").mkdir(exist_ok=True)
raw = ROOT / "narration" / "narration_raw.wav"
sf.write(raw, out[: int(total * SR)], SR, subtype="PCM_16")
subprocess.run([FFMPEG, "-hide_banner", "-loglevel", "error", "-y", "-i", str(raw),
                "-af", "loudnorm=I=-16:TP=-1.5:LRA=11", "-ar", str(SR), "-c:a", "pcm_s16le",
                str(ROOT / "audio" / "narration.wav")], check=True)


def arr(values):
    return "&[" + ", ".join(f"{v:.3f}" for v in values) + "]"


def warr(ws):
    return "&[" + ", ".join(f"W {{ w: {json.dumps(key(w) or w.lower())}, t: {a:.3f}, e: {b:.3f} }}" for w, a, b in ws) + "]"


rs = [
    "// Generated by narration/build_vo.py. Do not edit by hand.\n",
    "/// One spoken word: lowercase letters and digits only (\"T4,\" -> \"t4\"), start and end in scene time.",
    "#[derive(Debug)]",
    "pub struct W {",
    "    pub w: &'static str,",
    "    pub t: f32,",
    "    pub e: f32,",
    "}\n",
    "#[derive(Debug)]",
    "pub struct SceneCues {",
    "    pub name: &'static str,",
    "    /// Video time where the scene starts.",
    "    pub start: f32,",
    "    pub dur: f32,",
    "    /// Scene-relative start of each beat (narration line or quiz pause).",
    "    pub beats: &'static [f32],",
    "    /// Scene-relative end of each beat.",
    "    pub ends: &'static [f32],",
    "    /// Every word of each beat with its scene-relative timing.",
    "    pub words: &'static [&'static [W]],",
    "}\n",
    f"pub const TOTAL: f32 = {total:.3f};\n",
    "pub const SCENES: &[SceneCues] = &[",
]
for name, start, dur, cues, ends, words in scene_rows:
    rs.append(
        f'    SceneCues {{ name: "{name}", start: {start:.3f}, dur: {dur:.3f}, beats: {arr(cues)}, ends: {arr(ends)}, '
        f'words: &[{", ".join(warr(w) for w in words)}] }},'
    )
rs.append("];\n")
(ROOT / "src" / "cues.rs").write_text("\n".join(rs), encoding="utf-8", newline="\n")
(ROOT / "narration" / "SCRIPT.md").write_text("".join(md), encoding="utf-8", newline="\n")
for name, start, dur, cues, *_ in scene_rows:
    print(f"{name:16s} {start:7.1f}s  +{dur:5.1f}s  beats={len(cues)}")
print(f"TOTAL {total:.1f}s = {int(total // 60)}m{total % 60:04.1f}s; {shaped_s:.1f}s of over-long pauses removed")

# The host's talking bob follows the new track.
runpy.run_path(str(ROOT / "narration" / "envelope.py"), run_name="__main__")
