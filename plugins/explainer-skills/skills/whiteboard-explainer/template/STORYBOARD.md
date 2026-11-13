# Storyboard: "Binary search, in about a minute" (the template's demo)

~1.5 min, 5 scenes. Canvas 1920x1080. Content area x 140..1780, y 180..930. Keep x<760,y<150 free (chapter chip)
and x>1690,y>900 free (the host, small, bottom-right; it stands bigger at x>1600,y>800 in Hook, the quiz and
the outro). Notation `b3:"word"` = `c.w(3, "word")` (copy keys from narration/WORDS.md). On screen write the real
spellings and digits; the voice says "fifty-two", "one million".

This file is the plan helpers implement. Make it concrete enough to place, not design: per element the trigger
word, rough position (x, y), size and colour.

## Colours (fixed meanings, consts in objs.rs "this video")
- `TARGET_C` = PINK: the value being searched for, the question
- `LIVE_C` = CYAN: cells still in play (dimmed to 0.25 once thrown away)
- `GUESS_C` = YELLOW: a guess (numbered circle + ring); YELLOW also = titles, numbers, emphasis
- MINT: found / correct; RED: thrown away ("gone"); ORANGE: caveats ("worst case"), the trap
- GREY: provenance captions, secondary labels
- On the white figure card: paper inks `P_RED`, `P_BLUE`, `P_GREEN`, the yellow highlighter, `note()` labels

## Recurring objects (objs.rs)
- `sorted_row(t, t0, dur, x, y, cell, &values, &out)` + `cell_cx(x, cell, i)`: the sorted list as numbered cells;
  `out[i]` = when cell i is dimmed. Used in Idea and Outro (same look both times).
- `mcq(...)`: the quiz board.
- Figure (`src/figs.rs`): `guesses` (1600x1000, white; bars of worst-case guesses for 10..1M entries; bar
  rectangles in figure pixels in `s_figure.rs`). Caption: "demo figure · drawn for this template" in grey.

## Chapters (lib.rs `chapter`)
Hook: no chip · Idea: "1 · the idea" · Figure: "2 · bigger lists" · Quiz1: "Quiz 1 / 1" · Outro: no chip.

## Scene plan

### Hook (s_intro::hook): cold open + title
- b0:"list": a tall white box (170, 200, 380x700) with 9 handwritten names (36 px), "..." rows grey.
- b0:"million": "1,000,000 names" yellow 96 px at (720, 330).
- b0:"sorted": grey arrow down the list's right side; "A" on the third "a" (`c.wn(0, "a", 2)`), "Z" on "z".
- b1:"find": pink search icon + "find one name" 52 px at (760..1160, 478).
- b1:"point": yellow arrow into the list at the "Maya" row; b1:"entry": yellow box around that row.
- b1:"before" / "after": "before ↑" and "or after ↓ ?" cyan 48 px right of the arrow.
- b2:"questions": "how many questions?" pink 56 px at (720, 740).
- b3:"twenty": "20" yellow 150 px popping at (1530, 795); b3:"always": yellow ring round it; b3:"enough": mint tick.
- b4 start: everything shrinks into the top-right corner (shrink_aside, dim 0.35);
  b4:"binary": "binary search" yellow 140 px centred at (900, 590); b4:"search": underline.

### Idea (s_idea::idea): the method on a 15-cell row
- b0:"want": "looking for" white 56 at (160, 262); b0:"fiftytwo": "52" pink 72 after it.
- b0:"sorted": the row 3 7 12 18 21 25 31 40 44 52 58 63 70 77 85 at (160, 430), cells 106 px.
- Guesses (numbered circle above the cell + ring around it): b1:"middle" -> 40 (yellow "1"),
  b3:"middle" -> 63 (yellow "2"), b5:"middle" -> 52 (mint "3").
- b2:"left": cells 3..40 dim, red brace under them, "gone"; b4:"right": cells 63..85 the same.
- Log of guesses at (160, 750), 44 px, line height 66: b1:"forty" "guess 1:  40" + b2:"bigger" "too small → drop
  left"; b3:"sixtythree" "guess 2:  63" + b4:"big" "too big → drop right"; b5:"fiftytwo" "guess 3:  52" +
  b5:"found" "= 52 ✓ found" (mint).
- b5: second "three": yellow stamp "3 guesses" at (1360, 770).
- b6:"guess": "each guess: ÷ 2" yellow 56 at (1120, 885); b6:"half": underline.

### Figure (s_figure::figure): the "paper figure", annotated
- From the start: white card `guesses` fitted into (140, 175, 1200x730) + grey caption.
- b0:"guesses": red ring round the y-axis label; b0:"most": orange stamp "worst case" at (1560, 270);
  b0:"size": highlighter band over the x-axis labels.
- b1:"four": blue ring round the "10" bar, "10 → 4" yellow 60 at (1400, 440); b1: second "ten": ring round
  the "1k" bar, "1k → 10" at (1400, 540).
- b2:"ten" - 0.3: camera pushes in on the plot (`card.zoom`, the right column leaves the view);
  b2:"add": `note`s "+3 +3 +4 +3 +3" between the bars, 0.15 s apart (+4 orange).
- b3 start - 0.4: camera back out; b3:"million": spotlight on the 1M bar + green ring;
  b3:"twenty": highlighter on its "20" label, "1M → 20" mint 72 at (1400, 660) + underline.

### Quiz1 (s_quiz::quiz1): `mcq`
Question "the sorted list doubles: 1M → 2M names." / "at most, how many guesses now?"; options "40: twice as
many", "still 20: nothing changes", "21: one more" (correct, C), "23: three more"; trap A, stamp "that's one by
one". Notes: b8:"doubling" "2M = 2 × 1M", b8:"halving" "one more halving", b8:"enough" "2^21 ≈ 2.1M > 2M",
b9:"tempting" "40 = checking one by one".

### Outro (s_intro::outro): recap with the same objects
- b0 start: "binary search" yellow 90 centred at (960, 260); a 7-cell `sorted_row` at (435, 380), cells 150.
- b0:"middle": ring round the middle cell + "look"; b0:"half": left three cells dim, red brace, "drop half";
  b0:"repeat": mint curved arrow under the right half + "repeat".
- b1:"thanks": "thanks for watching!" white 64 centred at (960, 820); the host waves.
