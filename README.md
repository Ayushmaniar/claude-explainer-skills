# Explainer skills for Claude Code

Two [Claude Code](https://claude.com/claude-code) skills for turning a paper, article or concept into a
long, narrated, hand-drawn explainer video, with a free voice that runs on your own machine.

| Skill | What it does |
|---|---|
| **whiteboard-explainer** | 5-20 minute explainer videos in a hand-drawn whiteboard style. Every label, underline, circle and figure is drawn on the word that names it, slides build up as the narrator speaks, the paper's own figures are annotated on screen, and there are chapter chips, hard multiple-choice quizzes and a small animated host. Built with [fframes](https://github.com/dmtrKovalenko/fframes) (Rust). Ships a starter project in `template/`. |
| **local-tts** | Human-sounding text-to-speech on your own GPU (or CPU) with no API cost: Qwen3-TTS 1.7B CustomVoice (voice "aiden", 8-bit GGUF) run by [qwentts.cpp](https://github.com/ServeurpersoCom/qwentts.cpp). Also writes `.srt` subtitles from the known line timings and has a routine for testing newer TTS models. |

## Install

In Claude Code:

```
/plugin marketplace add Ayushmaniar/claude-explainer-skills
/plugin install explainer-skills@explainer-skills
```

Or copy the two folders under `plugins/explainer-skills/skills/` into `~/.claude/skills/`.

Then ask Claude for something like *"make a whiteboard explainer video of arxiv.org/abs/…"* or
*"read this paragraph aloud with the local voice"*.

## Requirements

**local-tts** needs the qwentts.cpp engine and ~2 GB of model weights. The skill's `SKILL.md` has the
setup steps, and Claude can walk you through them. Run `python scripts/say.py --check` to see what it
found. A GPU with ~4 GB free VRAM is ideal (it was built on an 8 GB RTX 4070 Laptop at ~4x realtime);
a CPU build works, about 4x slower.

**whiteboard-explainer** needs Rust, the FFmpeg shared libraries, LLVM/libclang, Python 3.10+ with
`faster-whisper`, and the local-tts skill. See its `SKILL.md` and `template/README.md`. It was
developed on Windows 11; the commands are written for Git Bash and should work on Linux/macOS with
the paths changed.

Recommended companion: the official `fframes-video` skill
(`npx skills add https://fframes.studio -g -a claude-code -y`).

## Layout

```
.claude-plugin/marketplace.json
plugins/explainer-skills/
  .claude-plugin/plugin.json
  skills/
    whiteboard-explainer/   SKILL.md, references/, scripts/, template/
    local-tts/              SKILL.md, scripts/, references/, eval/
```

## Licence

The skills' own text and code: MIT (see `LICENSE`). Third-party pieces (fonts, the fframes crate, the
TTS engine and model weights) keep their own licences; see
`plugins/explainer-skills/skills/whiteboard-explainer/template/THIRD_PARTY.md` and the links above.
The Qwen3-TTS weights are Apache-2.0.
