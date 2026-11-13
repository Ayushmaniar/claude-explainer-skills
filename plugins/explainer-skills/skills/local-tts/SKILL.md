---
name: local-tts
description: Generate human-sounding speech locally on your own GPU (or CPU) with no API cost — voiceovers, narration for explainer videos, reading text aloud, audio clips from text — plus matching .srt subtitles from the known line timings. Uses Qwen3-TTS 1.7B (8-bit GGUF) through qwentts.cpp. Use whenever a task needs text-to-speech (TTS), a voice, or a narration track, including inside HyperFrames, fframes, Remotion or other video projects, before reaching for a paid or cloud TTS API or a lower-quality built-in voice. Also use when the user asks to find, compare or try a newer/better local TTS model.
---

# Local TTS

The default voice: **Qwen3-TTS 1.7B CustomVoice, speaker `aiden`, 8-bit GGUF (Q8_0), run by
[qwentts.cpp](https://github.com/ServeurpersoCom/qwentts.cpp)** (C++/ggml, MIT) with this style
instruction:

```
Warm, clear and curious, like a friendly tech explainer video narrator. Relaxed, even pace.
```

It won a blind listening test against Kokoro, Chatterbox, PyTorch Qwen3-TTS, VoxCPM2 and Phonon
(see `references/benchmarks.md`). On an 8 GB RTX 4070 Laptop GPU it runs at ~4x realtime including
model loading: an 18-minute narration takes ~4.5 minutes.

## Setup (once per machine)

Run `python -I <skill>/scripts/say.py --check` first. If it prints `OK`, skip this section.
Otherwise set the engine up, or ask the user to. It takes 10-20 minutes and ~2.5 GB of disk.

1. **Build qwentts.cpp.** No prebuilt Windows binary exists, so build it from source:
   ```bash
   git clone --recurse-submodules https://github.com/ServeurpersoCom/qwentts.cpp ~/tools/qwentts.cpp
   cd ~/tools/qwentts.cpp
   ./buildcuda.sh       # NVIDIA (Linux); ./buildvulkan.sh for AMD/Intel; ./buildcpu.sh for CPU only (e.g. macOS)
   ```
   On Windows, install VS 2022 (or newer) Build Tools with the C++ workload, CMake, Ninja and the CUDA
   Toolkit, then run `buildcuda.cmd` from the repo. If `vcvars64.bat` lives elsewhere, edit the
   path at the top of that script. To target one GPU only, add `-DCMAKE_CUDA_ARCHITECTURES=89`
   (RTX 40xx; 86 = RTX 30xx, 120 = RTX 50xx) to the cmake line.
2. **Download the weights** (~2 GB) from Hugging Face
   [`Serveurperso/Qwen3-TTS-GGUF`](https://huggingface.co/Serveurperso/Qwen3-TTS-GGUF):
   ```bash
   pip install -U huggingface_hub
   hf download Serveurperso/Qwen3-TTS-GGUF qwen-talker-1.7b-customvoice-Q8_0.gguf qwen-tokenizer-12hz-Q8_0.gguf --local-dir ~/tools/qwen-gguf
   ```
3. **Point say.py at them** if you used other folders: set `QWEN_TTS_HOME` (the checkout),
   `QWEN_TTS_EXE` (the binary) or `QWEN_TTS_MODELS` (the .gguf folder). On Windows `say.py` adds
   `%CUDA_PATH%\bin\x64` to PATH for you (override with `CUDA_BIN`).
4. `say.py --check` again, then voice one test line.

Hardware: 8 GB VRAM is plenty for Q8_0 (~3 GB in use). With no GPU, a CPU build works at roughly
a quarter of the speed. `GGML_BACKEND=CPU` forces a GPU build onto the CPU.

## Before generating

1. **Is something else using the GPU?** A game or another GPU-heavy app makes generation ~4x slower
   and makes that app stutter. Ask before starting a long job. If the user names such apps, set
   `TTS_BUSY_APPS=app1,app2` and `say.py` warns when one is running.
2. **Is a newer model worth checking?** See "Keep looking for better models" below. If the last check
   is over ~6 weeks old and the task is a real narration (not a one-line test), do the quick scan first.
3. **Does the text sound human?** A voice reads AI-speak with the same lift every time, so the pattern
   stands out more than on a page. For any narration longer than a few lines, read
   `natural-narration.md` from the `whiteboard-explainer` skill (same plugin, `references/`) and run
   its `scripts/aispeak_scan.py`, which takes a script.py or a plain text file.

## Generate

The script uses the standard library only, so any Python 3.10+ works. `<skill>` is this skill's folder.

```bash
python -I <skill>/scripts/say.py "First line." "Second line." -o out.wav       # one WAV, 0.35 s gaps
python -I <skill>/scripts/say.py --file lines.txt -o clips/                    # 000.wav, 001.wav, ...
```

Options:

| Flag | What it does |
|---|---|
| `--speaker` | `aiden` (default); `ryan` is the other native English voice |
| `--instruct` | Style text, e.g. "Excited, fast-paced" or "Calm and serious" |
| `--seed N` | Gives a repeatable take |
| `--gap S` | Seconds of silence between lines in a joined file |
| `--srt F` | Also write subtitles timed to the joined WAV (see Subtitles) |
| `--quant` | `Q8_0` (default, best) or `Q4_K_M` (smaller, a bit flatter; download it first) |
| `--check` | Report the engine, models and CUDA folders found |

How it works:
- Every line goes through one process, so the model loads once (3–8 s). Pass lines in bulk, not one
  call per line.
- Output is 24 kHz mono 16-bit WAV.

Takes are sampled, so each run differs:
- Keep clips you like. Cache per line, keyed by text plus settings.
- Retake a line if its length is far off ~0.07 s per character. That means words were dropped or invented.

For a video narration, also:
- Trim each clip's silence to ~50 ms padding, then place the lines on the timeline with your own gaps.
- Loudness-normalize the final track: `ffmpeg -af loudnorm=I=-16:TP=-1.5:LRA=11`.

## Subtitles (.srt): offer them with every narrated video

You already know when each line starts and ends, so there's nothing to transcribe. Build the
subtitles from those timings.

- **With `say.py`:** add `--srt narration.srt` (needs a `.wav` `-o`). The subtitles match the joined
  track exactly.
- **In your own pipeline:** call `scripts/srt.py`, `to_srt([(start_s, end_s, text), ...], display=[...])`,
  with video-time start/end per spoken line. Or use the CLI: `python srt.py timings.tsv -o video.srt`.
- **Fix voice spellings.** Narration text is often written for the voice: "D I L", names respelled
  phonetically, "runner dot HTML", numbers in words. Map them back with `display` /
  `--display display.tsv` (spoken<TAB>written). Take the written forms from what's shown on screen,
  and use figures for numbers ("$232", "83%").
- **Check the result.** Grep it for leftover spaced capitals (`[A-Z] [A-Z]`) and " dot ".
- **Rules the module applies:**
  - at most 2 lines of 42 characters per subtitle
  - breaks at sentence, then clause, then word
  - short sentences merged so nothing flashes by
  - time shared in proportion to length
- **Name it like the video** (`Video Name.srt` next to `Video Name.mp4`) so players load it
  automatically. In VLC, `--sub-file=` forces a file.

The `whiteboard-explainer` skill's `template/narration/build_vo.py` is a full worked pipeline:
per-line cache, retakes, silence trimming, pause shaping, word alignment, a cue table and a
lip-sync envelope.

## Gotchas

- **The CUDA DLLs must be on PATH (Windows).** CUDA 13 keeps them in `...\CUDA\v13.x\bin\x64`. Without
  that folder on PATH, `qwen-tts.exe` exits silently with no output. `say.py` adds it for you.
- **`--stream-by-line -o -` writes streaming WAV headers** (sizes 0x7fffffff), one per line. Split at
  `RIFF....WAVEfmt ` and drop each 44-byte header. Don't trust the size fields.
- **Aiden misreads single letters and odd tokens.** Found by an unprompted Whisper audit:
  - "h star" came out as "a hap" or "our star", "y t" as "VT", "y" as "we", and "arXiv" as "ArcSireFolk".
  - These work: capital "H star" / "under H", "why tee" / "why tilde", "archive", and "SMILES check".
  - "aitch" is erratic. A name can be fine alone and garbled mid-sentence: test it inside the whole line.
  - A sentence-initial K can drop ("Kwenputer" → "When Pooter"): start such lines with "And" or "So".
  - Audit by transcribing each clip *without* the script as an initial prompt, then diffing against
    the script. A prompted Whisper "hears" the intended words and hides the errors. Tools:
    `whiteboard-explainer/template/narration/audit_vo.py` and `tts_try.py`.
- **Pauses at colons.** Qwen pauses 0.6-0.8 s at a colon and sometimes mid-phrase. In a narration
  cap each silence by the punctuation before it (`shape()` in the template's build_vo.py) and avoid
  colon-fragment lines ("First: ...").
- **CPU fallback**: Kokoro 82M (`kokoro-onnx`) is flat but fast on a CPU, fine for drafts while the GPU
  is busy.

## Keep looking for better models

Local TTS is improving fast, and the current choice will be beaten. Treat it as the current champion,
not the final answer. Look for newer and better models, especially before a big narration job, or
when the last check below is more than ~6 weeks old.

1. **Scan.** Search the web for open-weight TTS released or updated since the last check. Look at:
   - TTS Arena / Artificial Analysis speech leaderboards
   - Hugging Face trending text-to-speech models
   - r/LocalLLaMA
   - GitHub releases of the engines below
2. **Filter.** Keep a candidate only if it:
   - can be used locally for free (open weights, permissive or personal-use licence)
   - runs on the user's GPU memory and OS
   - speaks the language needed
   - beats the champion on quality, or matches it while being much faster
3. **Trial.** Generate `eval/lines.txt` with the candidate. Score it against the champion with
   `scripts/voice_metrics.py` (`pip install numpy librosa`; the champion and baseline recordings are
   in `eval/`):
   - Pitch sd ≥ ~5 semitones and range ≥ ~17 semitones are needed to sound natural.
   - Measure speed as seconds of audio per second of generation.
4. **Listen.** Ask the user to choose by ear between the candidate and `eval/champion_*.wav`
   (`ffplay -nodisp -autoexit file.wav`). The metrics only shortlist candidates; the ear decides.
5. **Record.** Update the "Last checked" line below and `references/benchmarks.md`, whether or not
   anything changed. If the champion changes, replace `eval/champion_*.wav` and update `say.py`, and
   ask before deleting the old model.

**Last checked: 2026-11-02** (web survey plus local VoxCPM2 Q8 and Phonon 31m/Miles trials). Aiden was
preferred by ear over both. Not yet tried locally: Phonon 40m, OmniVoice, Breeze TTS 2, Drama 3,
Orpheus, Dia, Sesame CSM, Spark-TTS, IndexTTS-2, VibeVoice, Higgs Audio, Kyutai TTS and
Fish/OpenAudio. Details are in `references/benchmarks.md`.
