# Style guide: the hand-drawn, narration-synced explainer

Distilled from a frame-by-frame study of a hand-drawn paper explainer on YouTube. The core:
**the voice and the picture move together**, everything is **drawn, written, underlined and
circled** as it is said, and every slide is **built up progressively**. The examples below come
from a video about agent harnesses (knobs = context, planning, tools); swap in your own topic's
meanings but keep one hue per meaning.

## 1. The look
- Near-black canvas `#030303`, a barely visible 36 px dot grid. No gradients or glow. White paper
  figures are the only light areas.
- Everything looks hand-drawn: wobbly 3 px strokes with round caps, rounded boxes (r ≈ 14), fills
  are very dark tints of the stroke colour, so strokes pop.
- One handwriting font (Excalifont) for nearly everything. Monospace only for code, tool names,
  file names and stubs.
- **One hue = one meaning, for the whole video.** Here:
  - yellow = emphasis, numbers, titles, caveats
  - cyan = context, history, tokens (knob 1)
  - purple = planning (knob 2)
  - teal = tools and actions (knob 3)
  - blue (navy fill) = the model
  - mint = success
  - red = failure, crash
  - orange = cost, warning
  - pink = the task or a question
  - grey = provenance and secondary text
- Paper figures are white cards, never full-bleed, with a grey handwritten caption
  ("paper · Figure 3").
  - Marks drawn **on** paper use the darker paper inks (`P_RED`, `P_BLUE`, `P_ORANGE`, `P_GREEN`)
    or the yellow highlighter.
  - Text on paper goes on a dark backdrop (`note`).

## 2. Words on screen
- On-screen text is **labels, never sentences**: 1-4 words, max ~7. The narrator carries the
  sentence. Put the **noun or number being introduced** on screen.
- Telegraphic, lowercase: `elision`, `T0 = no management`, `59 → 46`, `6.4% → 58.4%`, `the catch`.
  Use arrows, `=`, `≈`, `×`, `>`, `<`.
- Numbers from the paper are **re-written big** (yellow by default, mint for good, red for bad),
  next to the figure they come from. Counters roll up with an ease-out and land on the spoken number.
- Caveats, limitations and opinions are **boxed stamps** of 1-3 words: `the catch`, `not
  significant`, `analysis`, `our guess`. Use yellow or orange, tilted 1-2°.

## 3. Progressive disclosure
- One idea per slide (= scene, 20-60 s). It starts empty after a hard cut and is **built in
  narration order**, left to right and top to bottom, one element per spoken noun. Aim for 6-14
  elements per scene.
- Rhythm: a new mark every ~1.5-2.5 s (28-40 events/min). Max ~3 new elements per second. Never
  more than ~6-8 s with nothing new: during long sentences, add the supporting marks (underline,
  circle, tick).
- Old content, three options:
  1. stays (the slide keeps growing);
  2. **shrinks aside** to a corner and dims (`shrink_aside`, 0.8 s) when the next idea needs room
     but the old one is context;
  3. is removed (`until`), rarely.

  Hard cuts happen only between scenes.
- Pointing attention, all hand-drawn:
  - underline sweep (0.27 s)
  - circle/ellipse (0.45 s)
  - highlighter band on paper
  - spotlight (dim the rest of a figure)
  - arrow
  - brace
  - tick / cross
  - strike-through
  - camera push-in (zoom ~1.6-2.5 on a figure region)
- **Same layout, new data**: when the narration compares models or cases, keep one layout and
  swap or add the data (the viewer learns the layout once).
- **Concrete example first, then the abstraction** (a running example: the date-parser GitHub issue).

## 4. Timing rules (sync)
Visual onset relative to the start of its spoken word:

| visual | onset |
|---|---|
| icon, tick, cross, number pop, ring | -0.10 .. +0.05 s (on the word) |
| handwritten label | -0.12 s (`LEAD`), so it finishes as the word ends |
| section title / question at the top of a slide | -0.35 s (`LEAD_TITLE`) or at the beat start |
| box with a label | box -0.15 s, label +0.15 s after the box starts |
| arrow from A to B | after A exists, on the verb ("goes into", "calls", "comes back") |
| counter | starts on the number word, lands ~0.9 s later |
| set-up diagram (a figure card, an empty chart) | 0.3-1.5 s before the sentence that uses it |
| camera move | starts ~0.2-0.3 s before the word that names the region; 1.0 s ease-in-out |

- Never later than +0.3 s after the word.
- Lists: each item on its own word.
- Draw durations come from the kit:
  - handwriting ~750 px/s;
  - stroke 0.15-0.6 s by length;
  - box 0.25-0.45 s;
  - underline 0.27 s.

## 5. Structure of the whole video
- **Cold open** (no chip) with the most surprising number, then the title and paper card.
- **Roadmap**: a winding path with numbered stops, each drawn as it is named.
- **Chapters**: a yellow chip top-left (`N · title`) that changes after a 0.1 s black dip. The
  chips are the progress indicator.
- **Quizzes**: hard multiple-choice questions, see section 6.
- **Verdict**: a recap poster that brings back the *same objects* drawn earlier (model box +
  harness wrapper, the desk, the drawer, the to-do list, the toolbox vs terminal) with
  ✓ / ✗ / ★ / "depends" marks, in the order spoken. Then the fine print as stamps, and a
  handwritten "thanks for watching".
- The host (a Codex-pet sprite) sits in the bottom-right corner, bobbing with the voice; bigger in the cold
  open, the quizzes and the outro.

## 6. Quizzes: hard, accurate multiple choice
Quizzes must be **difficult but strictly accurate MCQs**. A wrong answer key
teaches the wrong thing, which is worse than no quiz. A quiz you can pass by remembering the
last sentence is a failed quiz.

**What to ask** (3-4 per video, each one on a different chapter):
- Reasoning, not recall: *why* a result happened, *what would change* if a setting changed,
  *which conclusion the table supports*, or applying the idea to a new case. Recalling a number
  that was just said is too easy.
- The answer must follow from what the video showed (plus the paper). Don't ask about something
  the video never explained.
- Hard means the viewer has to think, not that the wording is tricky. No trick negations ("which
  is NOT…") unless the "not" is drawn big and underlined.

**The options** (4 options, A-D, exactly one correct):
- Every distractor is a **real misconception** that a smart viewer would find tempting: the
  intuitive-but-wrong reading, percent vs points, the wrong row or column of the table, mixing
  up cause and correlation, a claim the paper explicitly rejects, a true statement that doesn't
  answer the question.
- Options are parallel: similar length, same grammar, same level of detail. The correct one must
  not stand out by being longest, most hedged or most specific. Spread the correct letter across
  the quizzes (not always B or C).
- No "all of the above", "none of the above" or joke options.
- Each option is short enough to write on the board: max ~8 words (labels, not sentences, as in
  section 2). The narrator reads all four.

**Accuracy check** (before the voice build, written into `research/quiz_keys.md`):
- For each quiz: the question, the four options, the correct letter, and for **every** option one
  line on why it is right or wrong, with the page/table/figure in the paper that proves it.
- Every number in a question or option is checked against `paper_notes.md` and any derived
  number (difference, ratio, relative gain) is **computed in Python**, never by mental arithmetic.
- Re-read each distractor and ask: could it also be defended from the paper? If yes, rewrite it
  until exactly one option is correct.

**On screen and in the script:**
- Script beats: "Quiz N." → the question → "A: …" "B: …" "C: …" "D: …" (one beat each, so each
  option is written on its own word) → `PAUSE` → "The answer is C." → why C is right → why the
  most tempting wrong option (name it) is wrong.
- Thinking time: set `PAUSE_LEN` to ~8 s in `build_vo.py` (the old open-ended quizzes used 4.5 s,
  too short for reading four options). The countdown runs during the pause.
- Reveal: circle or tick the correct option in mint, cross out or dim the others, and stamp the
  main trap with a short orange label (e.g. `points, not %`). Then build the explanation as usual.
