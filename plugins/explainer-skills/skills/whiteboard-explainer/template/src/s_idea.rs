//! Scene: Idea (chapter 1). A slide that builds up on the spoken words: the sorted row, three guesses
//! (numbered and circled on "middle"), each thrown-away half dimmed with a brace, a log of the guesses,
//! a stamp with the count and the rule underlined at the end.
use fframes::Svgr;

use crate::cues::SceneCues;
#[allow(unused_imports)]
use crate::kit::*;
#[allow(unused_imports)]
use crate::objs::*;

const VALUES: [u32; 15] = [3, 7, 12, 18, 21, 25, 31, 40, 44, 52, 58, 63, 70, 77, 85];

pub fn idea<'a>(t: f32, c: &SceneCues) -> Svgr<'a> {
    let mut v: Vec<Svgr<'a>> = Vec::new();
    let (x, y, cell) = (160.0, 430.0, 106.0);
    let cy = y + cell * 0.45; // vertical centre of the cells
    let cx = |i: usize| cell_cx(x, cell, i);

    // b0: what we look for, then the sorted row
    let want = "looking for ";
    v.push(hand(t, c.w(0, "want") - LEAD_TITLE, 160.0, 262.0, 56.0, WHITE, Al::Left, want));
    v.push(hand(t, c.w(0, "fiftytwo") - LEAD, 160.0 + text_w(want, 56.0), 262.0, 72.0, TARGET_C, Al::Left, "52"));
    let t_left = c.w(2, "left") - 0.05; // left half (cells 0-7) goes
    let t_right = c.w(4, "right") - 0.05; // right side (cells 11-14) goes
    let out: Vec<f32> = (0..VALUES.len())
        .map(|i| if i <= 7 { t_left } else if i >= 11 { t_right } else { f32::INFINITY })
        .collect();
    v.push(sorted_row(t, c.w(0, "sorted") - 0.15, 0.9, x, y, cell, &VALUES, &out));

    // the three guesses: number above the cell, ring around it
    let guesses = [(7usize, c.w(1, "middle") - 0.1, "1"), (11, c.w(3, "middle") - 0.1, "2"), (9, c.w(5, "middle") - 0.1, "3")];
    for (k, (i, t0, n)) in guesses.iter().enumerate() {
        let col = if k == 2 { MINT } else { GUESS_C };
        v.push(num(t, *t0, cx(*i), y - 52.0, 26.0, col, n));
        v.push(circle(t, *t0 + 0.15, cx(*i), cy, 60.0, 62.0, col));
    }

    // the thrown-away halves: a brace under them and "gone"
    let bottom = y + cell * 0.9 + 34.0;
    v.push(brace(t, t_left, x + 8.0, x + 8.0 * cell - 8.0, bottom, true, RED));
    v.push(hand(t, t_left + 0.3, x + 4.0 * cell, bottom + 62.0, 40.0, RED, Al::Mid, "gone"));
    v.push(brace(t, t_right, x + 11.0 * cell + 8.0, x + 15.0 * cell - 8.0, bottom, true, RED));
    v.push(hand(t, t_right + 0.3, x + 13.0 * cell, bottom + 62.0, 40.0, RED, Al::Mid, "gone"));

    // the log of guesses, one line per guess, written as it is said
    let (lx, ly, lh, size) = (160.0, 750.0, 66.0, 44.0);
    let rows = [
        (c.w(1, "forty") - LEAD, "guess 1:  40", c.w(2, "bigger") - LEAD, "too small → drop left", YELLOW),
        (c.w(3, "sixtythree") - LEAD, "guess 2:  63", c.w(4, "big") - LEAD, "too big → drop right", YELLOW),
        (c.w(5, "fiftytwo") - LEAD, "guess 3:  52", c.w(5, "found") - LEAD, "= 52 ✓ found", MINT),
    ];
    let bx = lx + text_w("guess 1:  40", size) + 34.0;
    for (i, (ta, a, tb, b, col)) in rows.iter().enumerate() {
        let yy = ly + i as f32 * lh;
        v.push(hand(t, *ta, lx, yy, size, WHITE, Al::Left, a));
        v.push(hand(t, *tb, bx, yy, size, col, Al::Left, b));
    }

    // b5: the count; b6: the rule
    v.push(stamp(t, c.wn(5, "three", 1) - 0.05, 1360.0, 770.0, 48.0, YELLOW, -2.0, "3 guesses"));
    let rule = "each guess: ÷ 2";
    v.push(hand(t, c.w(6, "guess") - LEAD, 1120.0, 885.0, 56.0, YELLOW, Al::Left, rule));
    v.push(underline(t, c.w(6, "half") - 0.05, 1120.0, 905.0, text_w(rule, 56.0), YELLOW));
    v.into()
}
