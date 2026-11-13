"""Text to speech with the local Qwen3-TTS GGUF engine (qwentts.cpp). Standard library only.

    python say.py "First line." "Second line." -o out_dir/        # one WAV per line
    python say.py --file lines.txt -o narration.wav --gap 0.35     # all lines joined into one WAV
    echo "Hello." | python say.py - -o hello.wav
    python say.py --file lines.txt -o narration.wav --srt narration.srt [--display display.tsv]
    python say.py --check                                          # show which engine/models/CUDA it found

One process generates every line, so the model loads once (3-8 s). Output is 24 kHz mono 16-bit.

Where it looks (override with environment variables):
    QWEN_TTS_EXE     path to qwen-tts(.exe)       default: built inside QWEN_TTS_HOME
    QWEN_TTS_HOME    the qwentts.cpp checkout     default: ~/tools/qwentts.cpp
    QWEN_TTS_MODELS  folder with the .gguf files  default: ~/tools/qwen-gguf, then QWEN_TTS_HOME/models
    CUDA_BIN         CUDA runtime DLL folder      default: %CUDA_PATH%/bin (Windows only)
    TTS_BUSY_APPS    comma-separated process names that share the GPU (e.g. a game); say.py warns
                     when one is running, because generation slows ~4x and the app stutters
"""

import argparse
import array
import os
import pathlib
import re
import subprocess
import sys
import time
import wave

HOME = pathlib.Path(os.environ.get("QWEN_TTS_HOME", pathlib.Path.home() / "tools" / "qwentts.cpp")).expanduser()
SR = 24000
STYLE = "Warm, clear and curious, like a friendly tech explainer video narrator. Relaxed, even pace."
WIN = os.name == "nt"


def find_exe() -> pathlib.Path | None:
    if os.environ.get("QWEN_TTS_EXE"):
        return pathlib.Path(os.environ["QWEN_TTS_EXE"]).expanduser()
    name = "qwen-tts.exe" if WIN else "qwen-tts"
    for sub in ("build/Release", "build", "build/bin", "build/bin/Release"):
        p = HOME / sub / name
        if p.is_file():
            return p
    return None


def find_models() -> pathlib.Path | None:
    if os.environ.get("QWEN_TTS_MODELS"):
        return pathlib.Path(os.environ["QWEN_TTS_MODELS"]).expanduser()
    for p in (pathlib.Path.home() / "tools" / "qwen-gguf", HOME / "models"):
        if any(p.glob("qwen-talker-*.gguf")):
            return p
    return None


def cuda_dirs() -> list[str]:
    """CUDA 13 on Windows keeps its runtime DLLs in bin/x64; without it on PATH the exe exits silently."""
    if not WIN:
        return []
    base = os.environ.get("CUDA_BIN") or (os.environ.get("CUDA_PATH") and os.path.join(os.environ["CUDA_PATH"], "bin"))
    return [d for d in (base and os.path.join(base, "x64"), base) if d and os.path.isdir(d)]


HEADER = re.compile(rb"RIFF....WAVEfmt ", re.S)


def split_pcm(blob: bytes) -> list[bytes]:
    """--stream-by-line writes one 44-byte WAV header + 16-bit PCM per input line, back to back.
    The headers are streaming placeholders (sizes 0x7fffffff), so cut at each header instead."""
    starts = [m.start() for m in HEADER.finditer(blob)]
    return [blob[a + 44:b] for a, b in zip(starts, starts[1:] + [len(blob)])]


def write_wav(path: pathlib.Path, pcm: bytes) -> None:
    with wave.open(str(path), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes(pcm)


def trim(pcm: bytes, pad: float = 0.05) -> bytes:
    """Cut leading/trailing near-silence (the model pads clips unevenly), keeping `pad` seconds."""
    a = array.array("h", pcm)
    loud = [i for i in range(0, len(a), 120) if abs(a[i]) > 300]
    if not loud:
        return pcm
    lo, hi = max(0, loud[0] - int(pad * SR)), min(len(a), loud[-1] + int(pad * SR))
    return a[lo:hi].tobytes()


def busy_app() -> str | None:
    """The first TTS_BUSY_APPS process that is running, if any."""
    names = [n.strip().lower() for n in os.environ.get("TTS_BUSY_APPS", "").split(",") if n.strip()]
    if not names:
        return None
    try:
        cmd = ["tasklist"] if WIN else ["ps", "-A", "-o", "comm="]
        tasks = subprocess.run(cmd, capture_output=True, text=True).stdout.lower()
    except OSError:
        return None
    return next((n for n in names if n in tasks), None)


def check(exe, models) -> int:
    ok = True
    print(f"engine : {exe or 'NOT FOUND'}" + ("" if exe and exe.is_file() else "   <- build qwentts.cpp or set QWEN_TTS_EXE"))
    ok &= bool(exe and exe.is_file())
    talkers = sorted(p.name for p in models.glob("qwen-talker-*.gguf")) if models else []
    codecs = sorted(p.name for p in models.glob("qwen-tokenizer-*.gguf")) if models else []
    print(f"models : {models or 'NOT FOUND'}   talkers={talkers or 'none'} tokenizers={codecs or 'none'}")
    ok &= bool(talkers and codecs)
    if WIN:
        print(f"cuda   : {cuda_dirs() or 'not found (fine for a CPU or Vulkan build)'}")
    print("OK" if ok else "setup incomplete: see the Setup section of SKILL.md")
    return 0 if ok else 1


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("text", nargs="*", help="lines to speak; '-' reads lines from stdin")
    ap.add_argument("--file", help="text file, one line per clip (blank lines skipped)")
    ap.add_argument("-o", "--out", help="a .wav file (lines joined) or a folder (one WAV per line)")
    ap.add_argument("--speaker", default="aiden", help="aiden (default), ryan, dylan, eric, serena, vivian, ...")
    ap.add_argument("--instruct", default=STYLE, help="style instruction")
    ap.add_argument("--quant", default="Q8_0", choices=["Q8_0", "Q4_K_M"], help="Q8_0 sounds best; Q4_K_M is smaller and faster")
    ap.add_argument("--gap", type=float, default=0.35, help="seconds of silence between lines in a joined WAV")
    ap.add_argument("--seed", type=int, default=-1, help="fix for repeatable takes")
    ap.add_argument("--srt", help="also write subtitles timed to the joined WAV (needs a .wav --out)")
    ap.add_argument("--display", help="TSV of spoken<TAB>written spellings for the subtitles, e.g. 'D I L<TAB>DIL'")
    ap.add_argument("--check", action="store_true", help="report the engine, models and CUDA folders found, then exit")
    args = ap.parse_args()

    exe, models = find_exe(), find_models()
    if args.check:
        raise SystemExit(check(exe, models))
    if not args.out:
        ap.error("-o/--out is required")
    if not (exe and exe.is_file() and models):
        check(exe, models)
        raise SystemExit(1)

    lines = []
    if args.file:
        lines += pathlib.Path(args.file).read_text(encoding="utf-8").splitlines()
    for t in args.text:
        lines += sys.stdin.read().splitlines() if t == "-" else t.splitlines()
    lines = [ln.strip() for ln in lines if ln.strip()]
    if not lines:
        raise SystemExit("nothing to say")
    if app := busy_app():
        print(f"warning: {app} is running; generation will be ~4x slower and it will stutter", file=sys.stderr)

    talker = models / f"qwen-talker-1.7b-customvoice-{args.quant}.gguf"
    codec = next(iter(sorted(models.glob("qwen-tokenizer-12hz-Q8_0.gguf")) or sorted(models.glob("qwen-tokenizer-*.gguf"))), None)
    if not talker.is_file() or not codec:
        raise SystemExit(f"missing {talker.name} or a qwen-tokenizer-*.gguf in {models}")
    cmd = [
        str(exe), "--model", str(talker), "--codec", str(codec),
        "--speaker", args.speaker, "--lang", "English", "--instruct", args.instruct,
        "--seed", str(args.seed), "--stream-by-line", "-o", "-",
    ]
    env = dict(os.environ, PATH=os.pathsep.join([*cuda_dirs(), os.environ.get("PATH", "")]))
    t0 = time.time()
    res = subprocess.run(cmd, input="".join(ln + "\n" for ln in lines).encode(), capture_output=True, env=env)
    if res.returncode != 0 or not res.stdout:
        raise SystemExit(f"{exe.name} failed (exit {res.returncode}):\n{res.stderr.decode(errors='replace')[-2000:]}")
    clips = [trim(c) for c in split_pcm(res.stdout)]
    if len(clips) != len(lines):
        raise SystemExit(f"got {len(clips)} clips for {len(lines)} lines")

    out = pathlib.Path(args.out)
    if out.suffix.lower() == ".wav":
        out.parent.mkdir(parents=True, exist_ok=True)
        write_wav(out, (b"\0\0" * int(args.gap * SR)).join(clips))
        if args.srt:
            sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
            from srt import read_display, to_srt

            timed, t = [], 0.0
            for line, pcm in zip(lines, clips):
                dur = len(pcm) / 2 / SR
                timed.append((t, t + dur, line))
                t += dur + int(args.gap * SR) / SR
            pathlib.Path(args.srt).write_text(to_srt(timed, read_display(args.display)), encoding="utf-8")
    elif args.srt:
        raise SystemExit("--srt needs a .wav --out (subtitles are timed to the joined track)")
    else:
        out.mkdir(parents=True, exist_ok=True)
        for n, pcm in enumerate(clips):
            write_wav(out / f"{n:03d}.wav", pcm)
    total = sum(len(c) for c in clips) / 2 / SR
    took = time.time() - t0
    print(f"{len(clips)} clips, {total:.1f}s of speech in {took:.1f}s ({total / took:.1f}x realtime) -> {out}")


if __name__ == "__main__":
    main()
