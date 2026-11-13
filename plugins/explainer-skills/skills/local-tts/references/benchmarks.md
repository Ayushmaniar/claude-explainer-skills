# Local TTS benchmarks and history

All numbers are from one RTX 4070 Laptop GPU (8 GB, ~60 W cap) on Windows 11 with nothing else on the
GPU. "4x realtime" means 10 s of audio takes 2.5 s to generate. Your numbers will differ.

## 2026-10-10: first survey

### Voices compared

The same three lines (`eval/lines.txt`) were rendered with each voice and scored with `voice_metrics.py`.

| Voice | Pitch sd | 90% pitch range | Pace | Pauses | Verdict |
|---|---|---|---|---|---|
| Kokoro 82M `af_heart` 1.08x (CPU, kokoro-onnx) | 2.94 st | 9.4 st | 170 wpm | 15 | Clear but flat and robotic |
| Chatterbox (MIT), default voice | – | – | fast | few | Lively but rushed |
| Qwen3-TTS 1.7B CustomVoice `Ryan` (PyTorch) | – | – | – | – | Good; lost to Aiden on the metrics |
| Qwen3-TTS 1.7B CustomVoice `Aiden` (PyTorch, bf16) | 5.80 st | 19.7 st | 161 wpm | 18 | Most natural on paper |
| Qwen3-TTS 0.6B CustomVoice `Aiden` | 5.73 st | 19.3 st | 146 wpm | 19 | About as natural |
| **GGUF Q8_0 Aiden (qwentts.cpp)** | 5.62 st | 17.9 st | 152 wpm | 18 | **Picked by ear: champion** |
| GGUF Q4_K_M Aiden | 5.31 st | 18.7 st | 137 wpm | 18 | A bit flatter and slower |

The pace numbers for the rows other than Kokoro and Q8_0 were measured with an undercounted word total
and rescaled by 90/61.

### Speed: PyTorch Qwen3-TTS

Measured on 8 real script lines.

| Model (bf16, sdpa) | 1 line per call | 4 lines per call | 8 lines per call |
|---|---|---|---|
| 1.7B CustomVoice | 0.49x, 4.4 GB | 1.62x, 5.8 GB | 1.82x, 7.4 GB |
| 0.6B CustomVoice | 0.48x, 2.5 GB | 1.64x, 3.8 GB | 2.18x, 5.7 GB |

PyTorch Qwen is bound by per-step overhead (Python and kernel launches), not by GPU math. That is why the
0.6B model is no faster than the 1.7B one unbatched, and why batching gave 3.3x. Quantizing inside
PyTorch would not help.

### Speed: GGUF via qwentts.cpp

Same 8 lines, whole process including model loading.

| Weights | Size | Overall | Excluding load | Load |
|---|---|---|---|---|
| Q8_0 | 2.0 GB | 4.1x | 6.3x | ~8 s cold |
| Q4_K_M | 1.2 GB | 5.4x | 6.6x | ~3 s |

That is 8–13x faster than PyTorch one line at a time. An 18.5-minute narration built in 4.5 minutes,
versus ~37 minutes with PyTorch.

### Sources

- Engine: github.com/ServeurpersoCom/qwentts.cpp (MIT).
- Weights: Hugging Face `Serveurperso/Qwen3-TTS-GGUF`.
- Base models: `Qwen/Qwen3-TTS-12Hz-{1.7B,0.6B}-CustomVoice` (Apache-2.0).
- Speakers: aiden, dylan, eric, ono_anna, ryan, serena, sohee, uncle_fu, vivian. Aiden and Ryan are the
  native English ones.

### Setup notes, in case a PyTorch engine is needed

- **PyTorch:** install it from the CUDA index (`--index-url https://download.pytorch.org/whl/cu124`).
  Otherwise pip on Windows installs the CPU-only build.
- **Qwen:** use `attn_implementation="sdpa"` (flash-attn does not build on Windows).
  `generate_custom_voice` accepts lists for batching. It prints a harmless SoX warning.
- **Chatterbox:** needs `setuptools<81` because its watermarker imports `pkg_resources`.

## 2026-11-02: web survey

- **VoxCPM2**: 2B, Apache-2.0, 30 languages, 48 kHz, voice design and cloning. The official bf16 estimate
  (~8 GB) is tight on an 8 GB GPU; a GGUF build through audio.cpp (CUDA, Windows) fits.
  https://huggingface.co/openbmb/VoxCPM2 · https://github.com/0xShug0/audio.cpp
- **Phonon** (Gradium, Oct 2026): 31m English (~64 MB q8, Apache-2.0) and 40m (~75 MB, CC BY-NC, gated).
  Windows x64 CLI. Strong CPU fallback candidate. https://gradium.ai/blog/release-phonon-open-weight
- **OmniVoice**: multilingual, cloning/design; GGUF available, but the GGUF card lists CC-BY-NC-4.0.
  https://github.com/k2-fsa/OmniVoice
- **Breeze TTS 2**: expressive voice direction; officially 12 GB minimum, Linux; research/noncommercial
  weights and outputs. Not a drop-in for 8 GB Windows.
- **Fish Audio Drama 3**: 35B-A3B, needs ≥48 GB FP8 NVIDIA on Linux/WSL; restrictive licence.
- **Higgs TTS 3**: ~4B, research/noncommercial licence.
- Artificial Analysis' arena ranks Drama 3 and Breeze TTS 2 highly, but hosted multi-voice results are
  not a controlled comparison with a local Aiden Q8_0 setup.

## 2026-11-02: local VoxCPM2 / Phonon 31m trials

Four identical English passages (everyday narration, dialogue, technical explanation, raw numbers),
seed 42, no retakes, each turned into a side-by-side comparison video.

| Setup | Speed (observed wall time) | Notes |
|---|---|---|
| Aiden, qwentts.cpp Q8_0 via say.py | ~2.1x | champion |
| VoxCPM2 Q8_0, audio.cpp v0.9.3 CUDA, steps 10, guidance 2, mem_saver | ~2.4x (excl. 5.6 s voice setup) | peak ~7.0 GB VRAM; synthetic reference voice |
| Phonon 31m-en Q8, ptts v0.6.0, Miles preset, CPU 4 threads | ~5.1x | runs on the CPU |

Short practical observations, not a controlled benchmark. An unprompted Whisper large-v3-turbo pass
got the first three passages right for every setup; each setup slipped on one number in the last
passage. Windowed-autocorrelation pitch stats (not comparable to the pYIN table above): Aiden sd 4.99 /
range 16.3 st, VoxCPM2 3.37 / 11.0 st, Phonon 4.01 / 12.2 st.

**Listening decision:** Aiden preferred over both. Champion and settings unchanged.
