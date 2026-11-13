"""Shared voice settings and tools for the narration scripts (build_vo.py, tts_try.py, audit_vo.py).

The voice comes from the `local-tts` skill's say.py (a local Qwen3-TTS engine). It is found, in order:
  1. env LOCAL_TTS_SAY = path to say.py
  2. ~/.claude/skills/local-tts/scripts/say.py
  3. ~/.claude/plugins/**/local-tts/scripts/say.py (a plugin install; the newest one wins)
say.py holds the engine's paths (its own env vars), so there is one engine config for every project.

FFmpeg (tempo change, loudness): env FFMPEG (path to the ffmpeg executable), else $FFMPEG_DIR/bin/ffmpeg,
else `ffmpeg` on PATH.
Whisper (word timings, voice audit): env WHISPER_DEVICE = cuda | cpu | auto (default auto: CUDA if it
loads, else CPU), WHISPER_THREADS for the CPU.
"""

import os
import pathlib
import shutil
import subprocess
import sys
import tempfile

import numpy as np

# Every clip in narration/cache is keyed by these settings + the text: change one and every line is voiced again.
VOICE = {"speaker": "aiden", "quant": "Q8_0",
         "instruct": "Warm, clear and curious, like a friendly tech explainer video narrator. Relaxed, even pace."}
SR = 24000


def find_say() -> pathlib.Path:
    env = os.environ.get("LOCAL_TTS_SAY")
    if env:
        p = pathlib.Path(env).expanduser()
        if p.is_file():
            return p
        raise SystemExit(f"LOCAL_TTS_SAY points to {p}, which is not a file")
    claude = pathlib.Path.home() / ".claude"
    p = claude / "skills" / "local-tts" / "scripts" / "say.py"
    if p.is_file():
        return p
    found = sorted((claude / "plugins").glob("**/local-tts/scripts/say.py"), key=lambda q: q.stat().st_mtime)
    if found:
        return found[-1]
    raise SystemExit(
        "Can't find the local-tts skill's say.py. Install the local-tts skill (it sets up the local voice), or set\n"
        "LOCAL_TTS_SAY=/path/to/say.py. No voice yet? Run narration/est_cues.py for provisional timings and a\n"
        "silent track, so the scenes can be drawn now and re-time themselves once build_vo.py runs."
    )


def synth(texts: list[str], voice: dict = VOICE) -> list[np.ndarray]:
    """Voice each line with say.py (one process, so the model loads once). Returns float32 mono clips at SR."""
    import soundfile as sf

    say = find_say()
    with tempfile.TemporaryDirectory(prefix="tts_") as tmp:
        lines = pathlib.Path(tmp) / "lines.txt"
        lines.write_text("".join(t + "\n" for t in texts), encoding="utf-8")
        out = pathlib.Path(tmp) / "clips"
        cmd = [sys.executable, str(say), "--file", str(lines), "-o", str(out),
               "--speaker", voice["speaker"], "--quant", voice["quant"], "--instruct", voice["instruct"]]
        res = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")
        if res.returncode != 0:
            raise SystemExit(f"say.py failed (exit {res.returncode}):\n{res.stderr[-2000:]}")
        clips = []
        for n in range(len(texts)):
            a, sr = sf.read(out / f"{n:03d}.wav", dtype="float32")
            if a.ndim > 1:
                a = a.mean(axis=1)
            if sr != SR:
                import librosa
                a = librosa.resample(a, orig_sr=sr, target_sr=SR)
            clips.append(a.astype("float32"))
    return clips


def ffmpeg_exe() -> str:
    if os.environ.get("FFMPEG"):
        return os.environ["FFMPEG"]
    if os.environ.get("FFMPEG_DIR"):
        for name in ("ffmpeg.exe", "ffmpeg"):
            p = pathlib.Path(os.environ["FFMPEG_DIR"]) / "bin" / name
            if p.is_file():
                return str(p)
    exe = shutil.which("ffmpeg")
    if exe:
        return exe
    raise SystemExit("ffmpeg not found: put it on PATH, or set FFMPEG (the executable) or FFMPEG_DIR (see env.sh)")


def whisper(model: str, compute_cuda: str = "float16"):
    """A faster-whisper model on CUDA if it loads (WHISPER_DEVICE=auto|cuda), else on the CPU."""
    from faster_whisper import WhisperModel

    dev = os.environ.get("WHISPER_DEVICE", "auto")
    threads = int(os.environ.get("WHISPER_THREADS", "0"))  # 0 = faster-whisper's default
    if dev in ("auto", "cuda"):
        try:
            try:
                import torch  # noqa: F401  (on Windows this preloads the cuBLAS/cuDNN DLLs that ctranslate2 needs)
            except ImportError:
                pass
            m = WhisperModel(model, device="cuda", compute_type=os.environ.get("WHISPER_COMPUTE", compute_cuda))
            # The model can load while cuBLAS is missing; that only fails at the first encode. Probe it now.
            list(m.transcribe(np.zeros(16000, dtype=np.float32), language="en")[0])
            return m
        except Exception as e:  # no GPU, no CUDA libraries (e.g. cublas64_12.dll), or out of memory
            if dev == "cuda":
                raise
            print(f"  whisper: CUDA unavailable ({type(e).__name__}), using the CPU", file=sys.stderr)
    return WhisperModel(model, device="cpu", compute_type=os.environ.get("WHISPER_COMPUTE", "int8"), cpu_threads=threads)
