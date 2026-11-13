---
name: whiteboard-explainer
description: Make a long-form (5-20 min) narrated explainer video of a paper, article or concept in a hand-drawn whiteboard style — every label, underline, circle and figure is drawn exactly on the word that names it, built up slide by slide, with the paper's own figures annotated on screen, chapter chips, hard multiple-choice quizzes and a small animated host. Use whenever the user asks for an explainer video, a video about a paper/arXiv link/blog post/concept, or "a video like the last one". Built with fframes (Rust; 1.3+ adds three.js-style 3D shots) plus the local-tts skill for the voice.
---

# Whiteboard explainer videos

The core of the style: **the voice and the picture move together.** Everything is drawn, written,
underlined and circled as it is said, and every slide is built up progressively. Read
`references/style-guide.md` before writing the storyboard; it holds the colours, the rules for
words on screen, progressive disclosure, the sync timing table and the whole-video structure.

**Start every video from `template/`** (copy it, don't start from scratch): copy this skill's
`template/` to `<project>/video/`, then `source env.sh && cargo build --release`. The binary is
`target/release/explainer` (`explainer.exe` on Windows; `R ...` in env.sh runs it). The template is a
~95 s demo ("binary search in about a minute") with four scenes to copy from; `template/README.md` has
every command. Run the Python scripts from the project folder as `python -I narration/<x>.py` with
numpy, soundfile, librosa, faster-whisper and pillow installed. What is in it:
- `src/kit.rs`: the drawing kit (hand-drawn strokes, `hand`/`note` text, `underline`, `circle`,
  `arrow`, `stamp`, `counter`, `Card` for paper figures with `mark`/`ring`/`spot`/`zoom`, `camera`,
  `shrink_aside`, icons). Use it whole.
- `src/lib.rs`: scene dispatch, chapter chips with the 0.1 s dip, image table, `SceneCues`
  (`c.w(beat, "word")` = when a word is spoken). Demo scenes: `s_intro.rs` (cold open, title, outro
  recap), `s_idea.rs` (a slide built on spoken words), `s_figure.rs` (figure card with
  ring/mark/note/spot and a zoom), `s_quiz.rs` (the `mcq` quiz with a trap stamp).
- `src/objs.rs`: reusable objects (`mcq`, `grid`, `token`, `vecbar`, `hbar`, `code_panel`,
  `chat_card`, `quote_card`, `axes`/`trace`, `vec_arrow`, `photo`, ...) plus a "this video" section:
  put each video's recurring objects there so every scene draws them identically.
- `src/duck.rs`: the host sprite (feet position, talking bob from the voice envelope, per-scene spots:
  big in the cold open / quizzes / outro, small bottom-right otherwise). No sprites → no host.
- `src/t3d.rs` (optional): immediate-mode cubes and `View3` for tensor/array 3D insets.
- `narration/`: `script.py` (scenes → beats, `PAUSE` for quizzes), `tts.py` (voice settings; finds the
  local-tts `say.py`), `build_vo.py` (voice + per-line cache + pause shaping + tempo + faster-whisper
  word alignment → `audio/narration.wav`, `src/cues.rs`, `src/envelope.rs`), `words.py` (checks every
  `c.w(...)` key, missing dispatch arms and leftover placeholders; writes WORDS.md), `est_cues.py`
  (provisional cues and a silent track before the voice exists), `audit_vo.py` / `tts_try.py`
  (pronunciation audit), `make_srt.py` (subtitles with a DISPLAY map, OVERRIDE and `--review`),
  `envelope.py`, `glitch.py` (final-file check).
- `tools/`: `prep_figs.py` (writes `src/figs.rs` from `figs/*.png`), `get_pet.py` (downloads a host
  sprite), `font_metrics.py`, `make_demo_fig.py`, `lowprio.py` (run a job at idle priority).
- `media/`: the fonts. Only media files may sit at its top level (`include_media_dir!` panics on
  others), so the licence texts live in `media/licenses/`.
- `HELPERS.md`: the brief to give every scene helper (rules, file map, build-and-check loop).
- `STORYBOARD.md`: the demo's storyboard, showing the level of detail helpers need.
- `THIRD_PARTY.md`: font, sprite, fframes and FFmpeg licences.

## Requirements

- **Rust** (stable) and **fframes 1.3.0** from crates.io (pinned in `template/Cargo.toml`; check its
  licence terms for your use).
- **FFmpeg shared libraries** (FFmpeg 7+ "shared" build) and **LLVM/libclang** for the fframes build.
  `template/env.sh` reads `FFMPEG_DIR` and `LIBCLANG_PATH`; on Windows also put FFmpeg's `bin` on PATH,
  for both cargo and the built binary (without the DLLs it exits 127 silently when piped).
- **Python 3.10+** with `faster-whisper` (word alignment, audit) and `numpy`.
- The **local-tts** skill (same plugin) for the voice. A GPU helps a lot; everything also runs on a CPU.
- A **host sprite** (optional): a Codex pet from github.com/gennadi-kuzmin/awesome-codex-pets
  (192x208 cells). `python -I tools/get_pet.py --list`, then `python -I tools/get_pet.py <pet-id>`
  downloads and slices one into `sprites/`. That repo's MIT licence covers its code; check each pet's
  artwork rights before publishing a video. Pick a bright one (dark pets vanish on the black board).
  With no sprite the video renders without a host.

## fframes version and the official skill

New videos use **fframes 1.3.0**. The creator's official skill, `fframes-video`, covers the general API
and changes with every release, so read it for anything this skill doesn't cover: its `SKILL.md`, then
`references/api.md`, `audio.md`, `design.md` and `3d.md`. Install it with
`npx skills add https://fframes.studio -g -a claude-code -y` (ask the user to run it if your
permissions block installs); the same files are in github.com/dmtrKovalenko/fframes under
`skills/fframes-video/`. This skill keeps only what is specific to this style: the look, the pipeline
and the lessons below. When the two disagree on rendering, the gotchas below win.

New in 1.3 and worth using:
- **3D shots** (`fframes::three_d`): see the next section.
- **Sharper checks**: `frame 3s --crop x,y,w,h` (a zoomed crop in video pixels, for touching strokes
  and small labels), `frame --ids` (outlines and names every element with an `id`), `strip 2s,5s,9s`
  (chosen times side by side) and `inspect --pops` (finds things that pop in or out in one frame;
  groups the host's blinks). `--pops` checks the frames, not the encoded mp4, so still run
  `glitch.py` on the final file.
- **Lottie**: feature `lottie`, `frame.play_lottie(ctx, "x.json", LottieOptions { .. })` for a
  ready-made animated icon. Hand-drawn kit strokes stay the default; use Lottie only when it matches
  the whiteboard look.
- `include_media_dir!(pub struct M, "media", stereo)` keeps stereo files stereo.

## 3D shots in a whiteboard video

Use 3D only when depth or a moving camera *is* the explanation: a stack of layers that one lifts out
of, tensor shapes, a memory hierarchy, a physical device, a loss surface, a paper figure card that
tilts in. Text, labels, charts and arrows stay hand-drawn SVG. Keep it to one or two 3D moments per
chapter: the video is still a whiteboard. `references/three_d_example.rs` is a 4 s test (a turning
layer stack with a lifting layer, a hand label pinned to it, a paper card drawn in svgr on a 3D plane).

- `fframes = { version = "=1.3.0", features = ["compile-time-svgtree", "cli", "3d", "wgpu"] }`.
  **The 3D works with the CPU frame renderer**: with `wgpu` the 3D renders on the GPU and is read
  back (120 frames at 1080p: 1.4 s on an RTX 4070 Laptop, 4.2 s with `Renderer::cpu()`). When the GPU is
  busy, switch the renderers to `.cpu()` (the example reads `FF_3D=cpu`) and nothing else changes.
  The wgpu types may need `#![recursion_limit = "256"]` in main.rs.
- Board look: `Background::Transparent` over the kit's `BG` rect, `Material::shadow_catcher` floor,
  `ToneMapping::Neutral` so the kit palette stays true, and
  `Material::standard(c).roughness(0.45).clearcoat(0.4)` for chunky objects that read at strip size.
- **Hand labels on 3D objects**: `camera.project(scene.local_to_world(id, local), w, h)` gives the
  pixel inside the inset; add the inset's `x`/`y`. Draw the label with the kit, on the trigger word.
- **Paper figures in 3D**: `ctx.render_svgr(svgr!(..), w, h)` → `Texture::from_image(&img)` on a
  `Geometry::plane` with `Material::unlit(0xffffff)`. The same path puts a figure PNG on a card.
- An object that moves must stay inside its inset renderer's box, or it is cut off at the edge.
- Build the scene once in `new()`, clone and pose it per frame (see the official `3d.md`).
- **Tensor / array visuals**: immediate mode works well. The scene fn builds a `Vec<Cube>` from `t`
  each frame (positions are plain arithmetic, so blocks collapse, split, unroll, ghost-fill), renders
  an inset with one shared `rounded_box` geometry, and projects points for hand-drawn arrows. Write
  cube numbers on the front layer only and skip labels of faded cubes (alpha < 0.6) or they show
  through. A full `inspect` with 3D is slow (~10 min for 5,400 frames): run it in the background.
- Keep a pinned label clear of every object across the whole camera move (check a still at the
  shot's start and end), and start a 3D shot after a `shrink_aside` has finished, not during it.

## Workflow

1. **Research** the source into `research/paper_notes.md`: every number with its table/page, plus
   wording hazards (percent vs points etc.). Extract the paper's figures as images (`figs/`, white
   cards in the video). Show the paper's own diagrams and figures on screen.
2. **Script** (`narration/script.py`): ~150 wpm, 1 sentence per beat, spelled for the voice ("Swee
   Bench", "T zero", numbers in words). Cold open with the most surprising number → title + paper
   card → roadmap → chapters → 3-4 quizzes (question, options A-D, `PAUSE`, answer) → verdict poster
   reusing the same objects drawn earlier → fine print → outro. Check the length after the voice
   build; trim lines or raise `TEMPO` (1.06 works) rather than rushing.
   - **Quizzes are hard, accurate multiple choice** (style guide section 6): reasoning questions, four
     options whose wrong ones are real misconceptions, exactly one defensible answer. Write
     `research/quiz_keys.md` (every option justified with a page/table ref, derived numbers computed
     in Python) and check it before building the voice.
   - **Teach, don't summarise.** Build each idea from zero with a worked example in real numbers.
     Define every term in the line that introduces it and shrink jargon to an everyday thing ("a node
     is just one job"). Start most beats with a connecting word ("So,", "Now,", "But") so the turns are
     audible. Stay near 150 wpm: the drawing needs time.
   - **Introduce every symbol properly before it is used**; repeat a short reminder in the line that
     uses it.
   - **Make it sound like a person, not AI.** Read `references/natural-narration.md` before writing,
     then run `python -I scripts/aispeak_scan.py <project>/narration/script.py`. Fix staged run-ups
     ("Here's the crux"), not-X-but-Y reframes, quotable closers and self-answered questions before
     building the voice.
   - Avoid colon-fragment lines ("First: ...", "Three benchmarks."): the voice pauses there and the
     next line sounds abrupt. A scene's first word alone ("Probe.") may be swallowed: write "Next, probe.".
     Give "Thanks for watching!" its own beat.
3. **Storyboard** (`STORYBOARD.md`): per scene, which element appears on which trigger word, its rough
   position (x, y), size and colour, which figure/crop, and the recurring objects + colour meanings.
   Concrete enough to implement, not design. Reserve the chip area (top-left) and the host area
   (bottom-right).
4. **Voice**: `narration/build_vo.py` voices through the local-tts skill's `say.py`, found via
   `LOCAL_TTS_SAY`, then `~/.claude/skills/local-tts/`, then `~/.claude/plugins/**`. GPU busy?
   `GGML_BACKEND=CPU` and `WHISPER_DEVICE=cpu`. Cached per line; rewording a line
   forces a new take. `shape()` caps each silence inside a clip by the punctuation before it
   (0.14 s none, 0.24 `,`, 0.3 `:;`, 0.42 `.?!`), with 0.5 s between lines (0.7 after a question).
   **Audit the voice before drawing scenes**: `audit_vo.py` transcribes every clip *without* the
   script as a prompt (build_vo's prompted Whisper hides mispronunciations) and diffs it against the
   script. Ignore US spellings and homophones; fix real misreads, testing fixes with `tts_try.py`
   (inside the whole sentence: a name can be fine alone and garbled mid-line). Known misreads are in
   the local-tts skill's gotchas. Re-run the audit after every re-voice (Qwen occasionally doubles a
   word, "the the"). Voice not ready yet? `est_cues.py` writes provisional cues and a silent track, and
   scenes re-time themselves when build_vo.py overwrites both.
5. **Kit + 1-2 exemplar scenes** drawn by you and checked at full size, so helpers have a quality bar.
6. **Fan out scene groups to parallel subagents** (small, fast models with high reasoning effort do
   this well when the brief is tight; a stronger model is only needed for the exemplars). Put the
   recurring objects in `objs.rs` and fill in `HELPERS.md` first, with **absolute** paths to
   `research/`. Copy the project into `work/<group>/` per agent (skip renders and the narration cache)
   with the target dir hardlinked (`cp -al target work/g/target`, or a per-helper `CARGO_TARGET_DIR`; then delete that copy's own `explainer` crate
   artifacts) to save disk and build time. Each helper edits only its `src/s_<group>.rs`, builds, runs
   `words.py` and `inspect`, checks strips and full frames. Merge by copying their `s_*.rs` back; diff
   other files first.
   - **Keep helpers fast.** Builds are the bottleneck: keep `[profile.release] incremental = true` +
     `codegen-units = 256` (rebuild ~50 s → ~25 s) and run at most ~4 helpers at once; give later groups
     to whoever finishes first. One group = 2 scenes of similar weight; split a quiz or a dense scene
     off. Budget: at most 3 build-and-look rounds per scene, a 12-frame strip plus `@end`, then list
     leftover doubts instead of polishing. One targeted fix round per helper is cheap.
   - **Verify their facts**: check every on-screen number against paper_notes and compute every worked
     example (helpers have invented numbers, carries going the wrong way, and a demo that gave away a
     later quiz's answer). Check that photos match the narration.
   - **Review** each helper's strips and frames yourself (their own check images can be stale), then
     send concrete fixes. Their common fault is bunching each phase into the top half in small type:
     give y-ranges and px sizes.
7. **Check** every scene: `inspect`, `strip -n 12-16`, full frames at `@end`. Look for overlaps, tiny
   text, empty stretches > 6-8 s, labels late vs the word. Zoom into doubtful spots with
   `frame <t> --crop x,y,w,h` rather than judging a scaled-down frame, and run `inspect --pops` once
   per scene group.
8. **Prove every scene is drawn** before the final render: no scene still calls `crate::placeholder`
   (`grep -n "crate::placeholder" src/s_*.rs` prints nothing) and every scene in script.py has a
   dispatch arm in lib.rs. Keep the group list in HELPERS.md and tick each group off as it merges.
9. **Subtitles** (`python -I narration/make_srt.py out/<Video Name>.srt --review`, named like the
   mp4), then **render** with
   fframes' CPU renderer (see gotchas), wrapped in `timeout 1200`. An 18-minute video renders in
   ~3 min on a recent 8-core laptop. Run `python -I narration/glitch.py out.mp4` before handing it over.

## Gotchas

- **Render the final video with fframes' built-in CPU frame renderer** (the template's default). The
  Skia/Vulkan video path was unreliable over long renders: `MaxPerformance` pasted bands of other
  frames into ~3% of frames and once deadlocked after the last frame; even `OnePipeline` pasted 1-px
  streaks into ~200 frames. Short ranges and single previews were clean, so it only shows on a full run.
- **glitch.py** flags pixels that change and snap back within 1-4 frames at 480x270. Host blinks and
  counters give ~20-30 harmless events; look at the flagged frames. Playing video clips trip it too.
- **When the GPU is busy** (a game, another render), everything still runs on the CPU:
  `GGML_BACKEND=CPU` for the TTS (~4x slower), Whisper on the CPU, the CPU frame renderer. On Windows
  run each job at IDLE priority (`SetPriorityClass(..., 0x40)`; children inherit it) with fewer
  threads (`WHISPER_THREADS`, `RAYON_NUM_THREADS`) so the other app stays smooth.
- **Fonts.** Excalifont lacks → ← ↑ ↓ ✓ ✗ ≈ ⋆ (the kit draws them as strokes) and ² ³ √ µ ± Σ θ ≠:
  the whole label then falls back to bold Inter and clashes. Write "n×n", "grad^2", "sqrt(L)",
  "theta", "sum", "A and B differ" instead; `inspect` flags the fallback. "÷" is fine. JetBrains Mono
  lacks the return sign: show newlines as a pilcrow. Copy `media/` (fonts) into every new project or
  every label falls back.
- usvg trims leading spaces in text: offset x by their width (the kit's `txt` does this).
- Coloured handwriting on white paper cards has poor contrast: use paper inks (`P_RED`, `P_BLUE`…) or
  `note()` (text on a dark backdrop).
- Camera zooms on figures: fill ~0.8 of the frame, not more; keep annotation text 15-20 px in figure
  space so it isn't huge after the zoom. Clamp camera keys so the view never leaves the figure.
- **Covers must be fully opaque and full width.** A backdrop that hides an earlier stage of a slide at
  opacity 0.92-0.99 leaves ghost text on a black board, and a narrow one leaks dimmed strokes at the
  edges. Start a backdrop on its first new element's word, not the beat start, or the board sits empty.
- A scene that draws the host itself (e.g. walking the roadmap) needs `None` in duck.rs `spot`, or it
  doubles.
- Windows console is cp1252: print `ascii()` of strings that may hold unicode, or
  `sys.stdout.reconfigure(encoding="utf-8", errors="replace")` (`python -I` ignores PYTHONIOENCODING).
- Regexes with `\b` written through a bash heredoc or `python -c` can become backspace characters:
  write such files with a file-writing tool, or check for `chr(8)`.
- **Rewording a finished video**: keep each scene's beat count and every trigger word's occurrence;
  `words.py` must report 0 bad cues. Re-voice only changed lines, re-audit them, look at a strip of
  every changed scene.
- **Subtitles without a display map**: take each line's text from the unprompted audit transcript
  (which already writes "95%", "RTX 3090"), fix names with regexes, and read every line once: it can
  also catch a word missing from the script.
- **Blog posts with videos**: download the post's figures and clips into `figs/` and `clips/`; draw a
  playing clip in a frame (fit, play from a trigger time, caption), and loop short clips by restarting
  them. A 40-minute render with clips takes ~30 min on a CPU.
- **Sources that need a login** (e.g. X threads) don't load for a plain fetch: find the linked posts by
  search, or read them through a browser tool. Interactive examples in a blog often live in its JS
  bundles: parse them as data in Python rather than running downloaded JS.
