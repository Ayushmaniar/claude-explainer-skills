//! Scenes: Hook (cold open + title), Outro.
use fframes::Svgr;

use crate::cues::SceneCues;
#[allow(unused_imports)]
use crate::kit::*;
#[allow(unused_imports)]
use crate::objs::*;

/// Cold open: a sorted list of a million names, the yes/no question you may ask, "how many questions?",
/// the surprising number (20). Then everything shrinks aside and the title is written.
/// The host stands big bottom-right in this scene (x > 1600, y > 800): keep that corner clear.
pub fn hook<'a>(t: f32, c: &SceneCues) -> Svgr<'a> {
    let mut a: Vec<Svgr<'a>> = Vec::new();
    // b0: the list, drawn as a long sheet of names, A at the top and Z at the bottom
    let (lx, ly, lw, lh) = (170.0, 200.0, 380.0, 700.0);
    let t_list = c.w(0, "list") - 0.15;
    a.push(boxed(t, t_list, lx, ly, lw, lh, WHITE, true));
    let names = ["Aaron", "Abby", "Adam", "...", "Maya", "...", "Yara", "Zane", "Zoe"];
    for (i, n) in names.iter().enumerate() {
        let col = if n.starts_with('.') { GREY } else { WHITE };
        a.push(hand(t, t_list + 0.3 + 0.1 * i as f32, lx + 40.0, ly + 70.0 + i as f32 * 72.0, 36.0, col, Al::Left, n));
    }
    let t_mil = c.w(0, "million") - LEAD;
    a.push(hand(t, t_mil, 720.0, 330.0, 96.0, YELLOW, Al::Left, "1,000,000 names"));
    // "sorted from A to Z": an arrow down the side of the list. "a list", "a million", "A": the third "a".
    let t_sorted = c.w(0, "sorted") - 0.1;
    a.push(arrow(t, t_sorted, lx + lw + 40.0, ly + 40.0, lx + lw + 40.0, ly + lh - 40.0, GREY));
    a.push(hand(t, c.wn(0, "a", 2) - LEAD, lx + lw + 64.0, ly + 70.0, 40.0, GREY, Al::Left, "A"));
    a.push(hand(t, c.w(0, "z") - LEAD, lx + lw + 64.0, ly + lh - 30.0, 40.0, GREY, Al::Left, "Z"));

    // b1: you look for one name; you may point at an entry and ask "before or after?"
    let t_look = c.w(1, "find") - 0.15;
    a.push(icon_search(t, t_look, 760.0, 455.0, 70.0, PINK));
    a.push(hand(t, t_look + 0.1, 820.0, 478.0, 52.0, PINK, Al::Left, "find one name"));
    let row_y = ly + 70.0 + 4.0 * 72.0; // the "Maya" row
    let t_point = c.w(1, "point") - 0.1;
    a.push(arrow(t, t_point, 700.0, row_y - 12.0, lx + lw - 20.0, row_y - 12.0, YELLOW));
    a.push(boxed(t, c.w(1, "entry") - 0.1, lx + 22.0, row_y - 46.0, lw - 140.0, 52.0, YELLOW, false));
    let before = "before ↑";
    a.push(hand(t, c.w(1, "before") - LEAD, 730.0, row_y, 48.0, CYAN, Al::Left, before));
    a.push(hand(t, c.w(1, "after") - LEAD, 760.0 + text_w(before, 48.0), row_y, 48.0, CYAN, Al::Left, "or after ↓ ?"));

    // b2: the question
    a.push(hand(t, c.w(2, "questions") - LEAD_TITLE, 720.0, 740.0, 56.0, PINK, Al::Left, "how many questions?"));

    // b3: the surprising number, circled and ticked
    let t20 = c.w(3, "twenty") - 0.08;
    a.push(pop(t, t20, 1530.0, 740.0, txt(1530.0, 795.0, 150.0, YELLOW, Al::Mid, "20")));
    a.push(circle(t, c.w(3, "always") - 0.1, 1530.0, 745.0, 112.0, 90.0, YELLOW));
    a.push(tick(t, c.w(3, "enough") - 0.05, 1530.0, 880.0, 54.0, MINT));

    // b4: everything shrinks into the top-right corner and the title is written big
    let t_aside = c.b(4) - 0.1;
    let board = shrink_aside(t, t_aside, (140.0, 180.0, 1640.0, 760.0), (1300.0, 190.0, 460.0), 0.35, a.into());
    let title = "binary search";
    let t_title = c.w(4, "binary") - LEAD_TITLE;
    let size = 140.0;
    let tw = text_w(title, size);
    fframes::svgr!(<g>
        {board}
        {hand(t, t_title, 900.0, 590.0, size, YELLOW, Al::Mid, title)}
        {underline(t, c.w(4, "search") + 0.3, 900.0 - tw / 2.0, 622.0, tw, YELLOW)}
    </g>)
}

/// Outro: the method in three moves, on the same sorted row as in Idea, then "thanks for watching".
pub fn outro<'a>(t: f32, c: &SceneCues) -> Svgr<'a> {
    let mut v: Vec<Svgr<'a>> = Vec::new();
    let title = "binary search";
    let t_title = c.b(0) - 0.2;
    v.push(hand(t, t_title, 960.0, 260.0, 90.0, YELLOW, Al::Mid, title));
    // the same row of sorted cells
    let values = [3, 12, 21, 31, 44, 58, 70];
    let (x, y, cell) = (435.0, 380.0, 150.0);
    let t_half = c.w(0, "half") - 0.05;
    let out = [t_half, t_half, t_half, f32::INFINITY, f32::INFINITY, f32::INFINITY, f32::INFINITY];
    v.push(sorted_row(t, c.b(0) + 0.3, 0.6, x, y, cell, &values, &out));
    // look in the middle
    let t_mid = c.w(0, "middle") - 0.1;
    v.push(circle(t, t_mid, cell_cx(x, cell, 3), y + 67.0, 82.0, 80.0, GUESS_C));
    v.push(hand(t, t_mid + 0.2, cell_cx(x, cell, 3), y - 30.0, 40.0, GUESS_C, Al::Mid, "look"));
    // keep the half that can still hold it
    v.push(brace(t, t_half, x + 8.0, x + 3.0 * cell - 8.0, y + 170.0, true, RED));
    v.push(hand(t, t_half + 0.3, x + 1.5 * cell, y + 235.0, 40.0, RED, Al::Mid, "drop half"));
    // repeat
    let t_rep = c.w(0, "repeat") - 0.15;
    v.push(arrow_curve(t, t_rep, cell_cx(x, cell, 6), y + 150.0, cell_cx(x, cell, 5), y + 250.0, cell_cx(x, cell, 4) + 10.0, y + 152.0, MINT));
    v.push(hand(t, t_rep + 0.3, cell_cx(x, cell, 5) + 40.0, y + 300.0, 44.0, MINT, Al::Mid, "repeat"));
    // thanks
    v.push(hand(t, c.w(1, "thanks") - LEAD, 960.0, 820.0, 64.0, WHITE, Al::Mid, "thanks for watching!"));
    v.into()
}
