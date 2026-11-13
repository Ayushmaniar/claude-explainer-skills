# Third-party material

What this template ships, what it only downloads or links to, and the licence of each.
Checked against the upstream projects in November 2026.

## Shipped in this folder

### Fonts (`media/`)

| File | Font | Upstream | Licence |
|---|---|---|---|
| `Excalifont-Regular.ttf` | Excalifont 1.000, (c) 2024 Excalidraw; designers Your Own Font Foundry (Virgil) and Ján Filípek / DizajnDesign | github.com/excalidraw/excalidraw, `packages/excalidraw/fonts/Excalifont` | SIL Open Font License 1.1 |
| `Inter-Regular.ttf`, `Inter-Medium.ttf`, `Inter-SemiBold.ttf`, `Inter-Bold.ttf` | Inter 4.001, (c) 2016 The Inter Project Authors | github.com/rsms/inter | SIL Open Font License 1.1 |
| `JetBrainsMono-Regular.ttf`, `JetBrainsMono-Bold.ttf` | JetBrains Mono 2.304, (c) 2020 The JetBrains Mono Project Authors | github.com/JetBrains/JetBrainsMono | SIL Open Font License 1.1 |

The full licence text for each font is in `media/licenses/` (a subfolder because fframes' `include_media_dir!`
compiles every file directly in `media/` into the binary and rejects non-media files there).

Notes:
- The OFL allows the fonts to be bundled and redistributed with software, on condition that the copyright
  notice and the licence go with them, and that the fonts are not sold on their own. Videos rendered with the
  fonts are not affected by the licence.
- Inter and JetBrains Mono carry the OFL notice in their own metadata (name IDs 13/14). The Excalifont TTF here
  has only a copyright line in its metadata. Its licence comes from upstream: the font metadata in the Excalidraw
  repository states SIL OFL 1.1, and that text is copied into `media/licenses/OFL-Excalifont.txt`.
- "Inter" is a trademark of rsms, "JetBrains Mono" of JetBrains s.r.o., and "Excalifont" of Excalidraw. The fonts
  are shipped unmodified and under their own names.

### Demo figure (`figs/guesses.png`)

Drawn for this template by `tools/make_demo_fig.py` (Pillow, using the Inter and JetBrains Mono fonts above).
The numbers are computed (floor(log2 n) + 1). It carries the same licence as the rest of the template.

## Not shipped: downloaded or installed by you

### Host sprites (`sprites/`, via `tools/get_pet.py`)

The host is a Codex pet from github.com/gennadi-kuzmin/awesome-codex-pets. The repository's `LICENSE` file is
MIT (c) 2026 Gennadii Kuzmin, but its README states the licence as "Code and original documentation: MIT", and
asks that pet artwork be original or used only with clear rights. It does not say the artwork itself is MIT.
Because of that, the template ships no pet artwork. `tools/get_pet.py` downloads a pet into your project
(`assets/<pet-id>/`, with a `SOURCE.md` note) and slices it. Check the rights of the pet you pick before you
publish a video with it. Without sprites, the video renders without a host.

### fframes (Rust crate, fetched by Cargo)

fframes (crates.io `fframes` 1.3.0 and its sub-crates `fframes-media`, `fframes_three_d`,
`fframes_skia_renderer`, `fframes-media-dir-macro`) is published under the MIT License, (c) 2025-2026 Dmitriy
Kovalenko (Cargo.toml `license = "MIT"`, and `LICENSE.txt` at github.com/dmtrKovalenko/fframes). Homepage:
fframes.studio. Its other dependencies come from crates.io under their own licences (see `Cargo.lock`).

### FFmpeg (installed by you)

fframes links FFmpeg's shared libraries at build time and at run time. The template doesn't include FFmpeg.
FFmpeg is LGPL 2.1+ (or GPL 2+ for builds with GPL parts). The default encoder in `src/main.rs` is `libx264`,
which is GPL, so it needs a GPL build of FFmpeg. Rendered videos are your own output, and the FFmpeg licence
doesn't cover them.

### Voice and speech recognition (installed by you)

- The voice comes from the `local-tts` skill's `say.py`, which runs a Qwen3-TTS model locally. Its model
  weights and engine are installed by that skill under their own licences.
- `faster-whisper` (word timings, voice audit) downloads Whisper models from Hugging Face on first use, under
  their own licences.
- Python packages used by the scripts: numpy, soundfile, librosa, faster-whisper, Pillow and fontTools
  (optional, only for `tools/font_metrics.py`).
