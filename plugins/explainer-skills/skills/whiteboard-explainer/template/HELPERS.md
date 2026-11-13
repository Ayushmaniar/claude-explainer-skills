# Brief for scene helpers (read all of it before writing code)

> Template: the lead fills in the parts in `<angle brackets>` for each video, and gives every helper this file
> plus its scene group (e.g. "group g3: scenes Ladder and Counter, file src/s_ladder.rs").

You are drawing scenes for a ~`<N>`-minute hand-drawn whiteboard **explainer video, "`<title>`"**. It explains
`<one or two sentences: the source (paper / post / concept) and what the video covers>`. It is a Rust program using
fframes 1.3: every scene is a pure function `fn(t, c) -> Svgr` returning the SVG of the frame at scene time `t`.
Elements appear **exactly when the narrator says the word that names them**, are hand-drawn (strokes draw on,
handwriting writes itself), and each scene is built up progressively on an empty black board.

## Where things are
- Your work copy: `<absolute path>/work/<group>/` (a full copy of the video project). **Edit only
  `src/s_<group>.rs` in your work copy**, and in it only the functions you were given. Never edit the main
  project, nor kit.rs / objs.rs / lib.rs / duck.rs / cues.rs / figs.rs. If you need a helper, write a private
  `fn` inside your own file.
- `STORYBOARD.md`: the plan for every scene. Read the top part (colours, recurring objects, figures) and your
  scenes' sections. It is a plan, not a contract: improve layouts if needed, but keep the meanings, the colours
  and the trigger words. Its `b3:"word"` notation means `c.w(3, "word")`. If a storyboard key isn't in WORDS.md,
  pick the nearest word that is.
- The skill's `references/style-guide.md`: the visual style (sections 2-4). `<absolute path to it>`
- `narration/script.py`: the narration; `narration/WORDS.md`: every scene's beats with each word's key and start
  time (scene time). Use `c.w(beat, "key")` for when a word starts, `c.wn(beat, "key", n)` for the n-th
  occurrence (0-based), `c.we(beat, "key")` for its end, `c.b(i)` / `c.e(i)` for beat start/end. Keys are
  lowercase letters+digits only: "fifty-two" -> "fiftytwo", "T zero," -> "t", "zero". **Always copy keys from
  WORDS.md.** The voice spells some words: `<list: "Kwen" = Qwen, "C P U" = CPU, ...>`. **On screen always write
  the real spelling.**
- `src/kit.rs` (drawing kit; read its doc comments) and `src/objs.rs`: the reusable objects (`mcq`, `grid`,
  `vals`, `hbar`, `score_bar`, `code_panel` / `code_h` / `code_hl`, `chat_card`, `quote_card`, `axes` / `fpts` /
  `trace` / `dot`, `vec_arrow`, `photo`, `person`) and, in its last section, **this video's recurring objects and
  colour consts**: `<list them>`.
  Kit widgets: `hand`, `note`, `hand_lines`, `mono` (no t: gate it yourself, e.g. `fade(t, t0, 0.25, mono(..))`),
  `mono_type`, `counter`, `underline`, `circle`, `arrow`, `arrow_curve`, `line`, `boxed`, `boxed_fill`,
  `dashed_box`, `label_box`, `model_box`, `pill`, `stamp`, `num`, `tick`, `cross`,
  `strike(t, t0, x1, x2, y, color)`, `brace`, `highlight`, `pop`, `rise`, `fade`, `until`, `dim_after`,
  `move_to`, `shrink_aside`, `camera` + `Key`, `countdown`, `bar`; icons `icon_doc`, `icon_term`, `icon_tools`,
  `icon_search`, `icon_plan`, `icon_coin`, `icon_drawer`, `icon_db`, `icon_bot`, `icon_clock`.
- **Figures**: `src/figs.rs` lists every image in `figs/` with its pixel size: `<name: what it is, white or dark>`.
  Look at a figure with your image tool (`figs/<name>.png`) before placing marks on it. Draw one with
  `Card::fit(name, x, y, w, h)` then `.show(t, t0)`, `.caption(t, t0, "<source> · Figure N")`, `.mark` / `.ring`
  / `.frame` / `.spot` (coordinates in the figure's own pixels), and `.zoom` + `camera` for a camera move. On
  white cards write with paper inks (`P_RED`, `P_BLUE`, `P_GREEN`, ...) or `note()`; labels next to the card go
  on the black board.
- **Exemplars (study before writing):** `<src/s_x.rs (fn)>` and `<src/s_y.rs (fn)>`, and their checks
  `check/<x>_end.png`, `check/<x>_strip.png`: that is the quality bar (sizes, spacing, how full the canvas is).
  In the template, `src/s_idea.rs` (a building slide) and `src/s_figure.rs` (a figure card with marks and a
  camera move) play this role.
- Facts: `<absolute path>/research/paper_notes.md` (every number with its source). **Never put a number, a
  quote or a claim on screen that is not in the narration or paper_notes.md.** Check every worked example by
  computing it. Labels are 1-6 words (max ~8), lowercase, telegraphic.

## Hard rules (the lead will send it back if you break these)
- Canvas 1920x1080. Keep **x < 760, y < 150** free (chapter chip) and **x > 1690, y > 900** free (the host).
  Use the space **x 140..1780, y 180..930**: content must fill it. A frame whose lower half is empty, or whose
  drawing sits in a small cluster, is wrong.
- **Text at least 32 px** for anything the viewer must read (28 px only for tiny tags inside a diagram); scene
  titles 54-60 px at (160, 215) left-aligned; key numbers 46-96 px. No text may overlap other text, a box
  border, a figure card, or run off the canvas.
- **Big shapes:** boxes for main objects at least ~200x80; figure cards as large as the layout allows (a figure
  the viewer must read is >= 900 px wide).
- Every element starts on (or 0.1-0.35 s before) the word that names it. **Never more than ~6 s with nothing new
  appearing** (check the WORDS.md times). Aim for 8-16 elements per scene.
- **Don't wipe the board mid-scene with `until()`.** When room runs out, `dim_after` the older part (to ~0.3) or
  `shrink_aside` it into a corner, or put a backdrop rect behind the new part: a backdrop must be **opacity 1**,
  the **full canvas width**, start on the first new element's word, and be tall enough to cover every old label.
  Use `until` only for a short-lived pointer/label.
- Colours have fixed meanings (STORYBOARD.md top).
- Excalifont lacks some glyphs: the kit draws → ← ↑ ↓ ✓ ✗ ≈ ⋆ itself. Avoid other unusual unicode in
  handwriting: no Greek letters, no "²" "³" "√" "∞" "≠" "≤" "≥" "…" "µ" "±" "Σ" (write "theta", "!=", "<=",
  "...", "n^2", "sqrt(L)", "sum"). "×" and "÷" are fine. Code, tool calls, hex and formulas go in `mono` /
  `code_panel`. `inspect` flags a font fallback.

## Build and check loop (bash; Git Bash on Windows)
```bash
cd "<absolute path>/work/<group>"
source env.sh; export RAYON_NUM_THREADS=4
cargo build --release -j 4 2>&1 | grep -E "^(error|warning: unused)" -A 6 | head -80   # no errors
B=./target/release/explainer          # explainer.exe on Windows
python -I narration/words.py                            # "0 bad word cues"
$B inspect <Scene>                                      # no errors (clipped text, font fallback)
$B strip <Scene> -n 12 -o check/<scene>_strip.png       # contact sheet of the build-up
$B frame "<Scene>@end" "<Scene>@50%" -o check           # full-size frames
$B frame "<Scene>@end" --crop 900,300,700,500 -o check  # zoomed crop to check small labels
```
`<Scene>` is the scene name as in narration/script.py. Then **look at the PNGs** (the strip and the `@end` frame
at full size) and fix: overlaps, text running off the canvas or into the chip/host areas, tiny text, empty
stretches, a mostly empty canvas, things appearing late. If a frame shows the placeholder (scene name + narration
lines), your build is stale: `touch src/lib.rs` and rebuild.
**Budget: at most 3 build-and-look rounds per scene.** Then stop and list any leftover doubts in your reply; the
lead reviews every scene and sends fixes. Batch your edits before each build.

## When done
Reply with: each scene, a one-line description of what it shows, and anything you could not do or are unsure
about (e.g. a coordinate you guessed). Do not render the video.
