//! Recurring objects, re-exported by `kit` (so `use crate::kit::*` brings them in).
//!
//! These are the reusable ones: the quiz board (`mcq`), cells and values, code / chat / quote cards,
//! simple plots, vectors, a picture stand-in. Put each new video's own recurring objects (the
//! things that come back scene after scene, drawn the same way every time) at the end of this file
//! under a new section, together with its colour meanings, e.g.
//!
//! ```ignore
//! /// The thing being searched for (pink everywhere).
//! pub const TARGET_C: &str = PINK;
//! ```
//!
//! and list them in HELPERS.md so every scene helper draws them identically.
use fframes::Svgr;

use crate::cues::SceneCues;
use crate::kit::*;

// ================================================================ the quiz

/// The multiple-choice quiz layout shared by every quiz. Beat layout (narration/script.py):
/// 0 "Quiz N." | 1 question | 2-5 options A-D | 6 PAUSE | 7 "The answer is X." | 8 why right |
/// 9 why the trap is wrong.
///
/// - `question`: 1-2 lines (max ~60 chars each at 46 px), written from beat 1.
/// - `opts`: four short on-screen labels (max ~36 chars), each written on its own beat.
/// - `correct`, `trap`: indices 0-3. On the reveal (beat 7) the correct box turns mint with a
///   tick, the others dim; on beat 9 the trap gets an orange stamp `trap_label`.
/// - `notes`: explanation labels for the right-hand column (x 1400-1870, y 400-760), as
///   (start time, text, colour); each on its own line, 32 px, max ~26 characters.
/// - `trap_label`: max ~22 characters (the stamp sits on the right end of the trap option).
pub fn mcq<'a>(t: f32, c: &SceneCues, question: &[&str], opts: [&str; 4], correct: usize, trap: usize, trap_label: &str, notes: &[(f32, &str, &str)]) -> Svgr<'a> {
    let mut v: Vec<Svgr<'a>> = Vec::new();
    // question
    let mut qt = c.b(1) - 0.1;
    for (i, l) in question.iter().enumerate() {
        v.push(hand(t, qt, 150.0, 250.0 + i as f32 * 62.0, 46.0, WHITE, Al::Left, l));
        qt = hand_end(qt, l, 46.0);
    }
    // options
    let reveal = c.b(7) - 0.05;
    let (ox, ow, oh) = (150.0, 1100.0, 86.0);
    let oy0 = 250.0 + question.len() as f32 * 62.0 + 40.0;
    let letters = ["A", "B", "C", "D"];
    for i in 0..4 {
        let y = oy0 + i as f32 * (oh + 20.0);
        let t0 = c.b(2 + i) - 0.15;
        let is_right = i == correct;
        let col = if t >= reveal && is_right { MINT } else { WHITE };
        let body = fframes::svgr!(<g>
            {boxed(t, t0, ox, y, ow, oh, if is_right && t >= reveal { MINT } else { GREY }, true)}
            {num(t, t0 + 0.05, ox + 46.0, y + oh / 2.0, 24.0, if is_right && t >= reveal { MINT } else { YELLOW }, letters[i])}
            {hand(t, t0 + 0.2, ox + 92.0, y + oh / 2.0 + 14.0, 40.0, col, Al::Left, opts[i])}
        </g>);
        let body = if is_right { body } else { dim_after(t, reveal, 0.38, body) };
        v.push(body);
        if is_right {
            v.push(tick(t, reveal + 0.1, ox + ow - 50.0, y + oh / 2.0, 56.0, MINT));
        } else if i == trap {
            v.push(stamp(t, c.b(9) - 0.1, ox + ow - text_w(trap_label, 32.0) / 2.0 + 60.0, y + oh / 2.0, 32.0, ORANGE, -2.0, trap_label));
        } else {
            v.push(cross(t, reveal + 0.25 + 0.1 * i as f32, ox + ow - 50.0, y + oh / 2.0, 34.0, RED));
        }
    }
    // pause: countdown on the right
    v.push(countdown(t, c.b(6), c.e(6) - c.b(6), 1560.0, 560.0, 90.0));
    // answer
    let ans = format!("answer: {}", letters[correct]);
    v.push(hand(t, reveal, 1400.0, 330.0, 64.0, MINT, Al::Left, &ans));
    v.push(underline(t, reveal + 0.4, 1400.0, 348.0, text_w(&ans, 64.0), MINT));
    for (i, (t0, s, col)) in notes.iter().enumerate() {
        v.push(hand(t, *t0, 1400.0, 440.0 + i as f32 * 62.0, 32.0, col, Al::Left, s));
    }
    v.into()
}

// ================================================================ boxes, cells and values

/// A grid of `rows` x `cols` cells (a tensor) with top-left (x, y) and cell size `cell`. Cells pop
/// in row by row over ~0.5 s. Returns the group; its size is (cols*cell, rows*cell).
pub fn grid<'a>(t: f32, t0: f32, x: f32, y: f32, rows: usize, cols: usize, cell: f32, color: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let fill = tint(color, 0.18);
    let mut v: Vec<Svgr<'a>> = Vec::new();
    let step = 0.5 / rows.max(1) as f32;
    for r in 0..rows {
        let tr = t0 + step * r as f32;
        if t < tr {
            break;
        }
        let o = prog(t, tr, 0.15);
        for k in 0..cols {
            let (cx, cy) = (x + k as f32 * cell, y + r as f32 * cell);
            let (f, c) = (fill.clone(), color.to_string());
            v.push(opacity(o, fframes::svgr!(<rect x={cx + 1.5} y={cy + 1.5} width={cell - 3.0} height={cell - 3.0} rx="3" fill={f} stroke={c} stroke-width="1.6" />)));
        }
    }
    v.into()
}

/// A token: a small rounded square of side `s` centred at (cx, cy) in `color`,
/// popping in at t0.
pub fn token<'a>(t: f32, t0: f32, cx: f32, cy: f32, s: f32, color: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let c = color.to_string();
    let f = tint(color, 0.35);
    pop(t, t0, cx, cy, fframes::svgr!(<rect x={cx - s / 2.0} y={cy - s / 2.0} width={s} height={s} rx="4" fill={f} stroke={c} stroke-width="2" />))
}

/// A vector (an embedding): a row of `n` cells of `s` px with 4 px gaps from (x, y) in `color`, popping in left to right
/// over ~0.4 s. Width `n * (s + 4) - 4`, height `s`.
pub fn vecbar<'a>(t: f32, t0: f32, x: f32, y: f32, n: usize, s: f32, color: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let v: Vec<Svgr<'a>> = (0..n).map(|i| token(t, t0 + 0.4 * i as f32 / n.max(1) as f32, x + s / 2.0 + i as f32 * (s + 4.0), y + s / 2.0, s, color)).collect();
    v.into()
}

/// A row of tensor cells (a small tensor) with a value written in each cell, appearing left to right.
/// `col` colours each cell's outline; values are drawn in mono.
pub fn vals<'a>(t: f32, t0: f32, x: f32, y: f32, cell: f32, values: &[&str], col: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let mut v: Vec<Svgr<'a>> = Vec::new();
    for (i, s) in values.iter().enumerate() {
        let cx = x + i as f32 * cell;
        let ti = t0 + 0.06 * i as f32;
        v.push(boxed_fill(t, ti, cx + 3.0, y, cell - 6.0, cell - 6.0, col, &tint(col, 0.18)));
        if t >= ti + 0.1 {
            let size = (cell * 0.46).min(40.0);
            let tw = mono_w(s, size);
            v.push(mono(cx + cell / 2.0 - tw / 2.0, y + cell / 2.0 + size * 0.32, size, WHITE, s));
        }
    }
    v.into()
}

/// A horizontal timing bar growing from the left in ~0.6 s (filled, outlined).
pub fn hbar<'a>(t: f32, t0: f32, x: f32, y: f32, w: f32, h: f32, color: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let ww = (w * ease_in_out(prog(t, t0, 0.6))).max(1.0);
    let fill = tint(color, 0.3);
    let color = color.to_string();
    fframes::svgr!(<rect x={x} y={y} width={ww} height={h} rx="4" fill={fill} stroke={color} stroke-width="2.5" />)
}

/// A score bar: a grey 0..100 track `w` px wide at (x, y), `h` tall, filled to `score` (percent) in `color` from `t0`
/// (grows over 0.6 s), with the number written at its right end (`label`, e.g. "38%").
pub fn score_bar<'a>(t: f32, t0: f32, x: f32, y: f32, w: f32, h: f32, score: f32, color: &str, label: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let fw = (w * score / 100.0).max(3.0) * ease_out(prog(t, t0, 0.6));
    let c = color.to_string();
    fframes::svgr!(<g>
        <rect x={x} y={y} width={w} height={h} fill="none" stroke="#5a5a5a" stroke-width="2" rx="4" />
        <rect x={x} y={y} width={fw} height={h} fill={tint(&c, 0.45)} stroke={c.clone()} stroke-width="3" rx="4" />
        {hand(t, t0 + 0.5, x + w + 18.0, y + h * 0.5 + 13.0, 38.0, color, Al::Left, label)}
    </g>)
}

/// A labelled link between two points: a thick line (draws on) with its label at the midpoint, above.
pub fn link<'a>(t: f32, t0: f32, x1: f32, y1: f32, x2: f32, y2: f32, color: &str, size: f32, label: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    fframes::svgr!(<g>
        {line(t, t0, x1, y1, x2, y2, color, 6.0)}
        {hand(t, t0 + 0.3, (x1 + x2) / 2.0, (y1 + y2) / 2.0 - 16.0, size, color, Al::Mid, label)}
    </g>)
}

/// A person tag: a pill with the name in the person's colour.
pub fn person<'a>(t: f32, t0: f32, x: f32, y: f32, size: f32, name: &str, color: &str) -> Svgr<'a> {
    pill(t, t0, x, y, size, color, name)
}

// ================================================================ code, chat and quotes

/// Code panels: dark backdrop.
pub const CODE_BG: &str = "#0f1418";

/// Monospace code on a dark panel, top-left at (x, y), `w` wide. Lines type in one after the other at
/// `cps` characters per second, starting at `t0`. Line height is 1.45 x `size`. Returns the panel.
/// Use `code_h` for its height and `code_line_y` for the baseline of line i (to place highlights).
pub fn code_panel<'a>(t: f32, t0: f32, x: f32, y: f32, w: f32, lines: &[&str], size: f32, cps: f32) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let h = code_h(lines.len(), size);
    let mut v: Vec<Svgr<'a>> = vec![boxed_fill(t, t0, x, y, w, h, GREY, CODE_BG)];
    let mut start = t0 + 0.25;
    for (i, l) in lines.iter().enumerate() {
        v.push(mono_type(t, start, x + 18.0, code_line_y(y, i, size), size, WHITE, l, cps));
        start += l.chars().count() as f32 / cps + 0.05;
    }
    v.into()
}

/// Height of a code panel with `n` lines.
pub fn code_h(n: usize, size: f32) -> f32 {
    n as f32 * size * 1.45 + size * 0.9
}

/// Baseline y of line `i` of a code panel at y.
pub fn code_line_y(y: f32, i: usize, size: f32) -> f32 {
    y + size * 0.45 + size * 1.1 + i as f32 * size * 1.45
}

/// Highlighter band over line `i` of a code panel at (x, y), `w` wide.
pub fn code_hl<'a>(t: f32, t0: f32, x: f32, y: f32, w: f32, i: usize, size: f32, color: &str) -> Svgr<'a> {
    let by = code_line_y(y, i, size) - size * 1.0;
    highlight(t, t0, x + 8.0, by, w - 16.0, size * 1.35, color, 0.28)
}

/// Splits `text` into lines that fit `w` px of monospace at `size`.
pub fn wrap_mono(text: &str, w: f32, size: f32) -> Vec<String> {
    let max = ((w / (size * 0.6)).floor() as usize).max(8);
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        if !cur.is_empty() && cur.chars().count() + 1 + word.chars().count() > max {
            lines.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

/// Height of a `chat_card` whose text wraps to `n` lines at `size`.
pub fn chat_card_h(n: usize, size: f32) -> f32 {
    64.0 + n as f32 * size * 1.35
}

/// Number of lines `text` wraps to in a `chat_card` of width `w` at `size`.
pub fn chat_lines(text: &str, w: f32, size: f32) -> usize {
    wrap_mono(text, w - 44.0, size).len()
}

/// A chat message card (real model output, typed in monospace): top-left (x, y), width w. A small handwritten
/// header `who` (e.g. "you", "the model") in `color`, a rounded border in `color`. `text` is
/// wrapped to the card and typed at `cps` characters per second from `t0 + 0.3`.
/// Height = chat_card_h(chat_lines(text, w, size), size).
pub fn chat_card<'a>(t: f32, t0: f32, x: f32, y: f32, w: f32, who: &str, text: &str, size: f32, color: &str, cps: f32) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let lines = wrap_mono(text, w - 44.0, size);
    let h = chat_card_h(lines.len(), size);
    let mut v: Vec<Svgr<'a>> = vec![boxed_fill(t, t0, x, y, w, h, color, "#0c0f12")];
    v.push(hand(t, t0 + 0.1, x + 22.0, y + 36.0, 28.0, color, Al::Left, who));
    let mut start = t0 + 0.3;
    for (i, l) in lines.iter().enumerate() {
        v.push(mono_type(t, start, x + 22.0, y + 52.0 + (i as f32 + 0.8) * size * 1.35, size, WHITE, l, cps));
        start += l.chars().count() as f32 / cps;
    }
    v.into()
}

/// A quoted post / essay card, hand-drawn (not a screenshot): top-left (x, y), width w. Header: `who` in
/// `color` (handwriting 36 px) and `sub` (date / handle / venue) in grey 28 px; then `lines` of handwriting
/// (34 px, white), each written after the previous. Height = quote_card_h(lines.len()).
pub fn quote_card<'a>(t: f32, t0: f32, x: f32, y: f32, w: f32, who: &str, sub: &str, lines: &[&str], color: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let h = quote_card_h(lines.len());
    let mut v: Vec<Svgr<'a>> = vec![boxed_fill(t, t0, x, y, w, h, color, "#0c0c10")];
    v.push(hand(t, t0 + 0.15, x + 30.0, y + 56.0, 36.0, color, Al::Left, who));
    v.push(hand(t, t0 + 0.3, x + 50.0 + text_w(who, 36.0), y + 56.0, 28.0, GREY, Al::Left, sub));
    v.push(hand_lines(t, t0 + 0.5, x + 30.0, y + 116.0, 34.0, 50.0, WHITE, Al::Left, lines));
    v.into()
}

pub fn quote_card_h(n: usize) -> f32 {
    96.0 + n as f32 * 50.0
}

// ================================================================ plots

/// Axes in the box (x, y, w, h): origin at the bottom-left, x label under the right end of the x axis,
/// y label above the y axis. Pass "log D" etc. for log-log plots.
pub fn axes<'a>(t: f32, t0: f32, x: f32, y: f32, w: f32, h: f32, xlabel: &str, ylabel: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let (ox, oy) = (x, y + h);
    fframes::svgr!(<g>
        {arrow(t, t0, ox, oy, ox + w + 20.0, oy, GREY)}
        {arrow(t, t0, ox, oy, ox, y - 20.0, GREY)}
        {hand(t, t0 + 0.2, ox + w, oy + 44.0, 30.0, GREY, Al::Right, xlabel)}
        {hand(t, t0 + 0.2, ox - 14.0, y - 36.0, 30.0, GREY, Al::Left, ylabel)}
    </g>)
}

/// A point of the axes box (x, y, w, h) from fractions: fx 0 = left edge, fy 0 = bottom edge.
pub fn ax(x: f32, y: f32, w: f32, h: f32, fx: f32, fy: f32) -> (f32, f32) {
    (x + fx * w, y + h - fy * h)
}

/// Samples `f` (fx in 0..1 -> fy) into `n` points of the axes box, for `trace`.
pub fn fpts(x: f32, y: f32, w: f32, h: f32, fx0: f32, fx1: f32, n: usize, f: impl Fn(f32) -> f32) -> Vec<(f32, f32)> {
    (0..=n).map(|i| {
        let fx = fx0 + (fx1 - fx0) * i as f32 / n as f32;
        ax(x, y, w, h, fx, f(fx))
    }).collect()
}

/// A smooth curve through `pts` (no wobble: it's a plotted function), drawn on over `dur` seconds from `t0`.
/// Make `dur` match the words that describe it.
pub fn trace<'a>(t: f32, t0: f32, dur: f32, pts: &[(f32, f32)], color: &str, width: f32) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let s = Stroke::from_pts(pts.to_vec());
    ink(&s, ease_in_out(prog(t, t0, dur)), color, width)
}

/// A data point that pops in at `t0`.
pub fn dot<'a>(t: f32, t0: f32, cx: f32, cy: f32, r: f32, color: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let rr = r * ease_out(prog(t, t0, 0.25));
    let c = color.to_string();
    fframes::svgr!(<circle cx={cx} cy={cy} r={rr} fill={tint(&c, 0.6)} stroke={c.clone()} stroke-width="2.5" />)
}

// ================================================================ vectors and rotations

/// An arrow (a vector) from the origin (ox, oy) of length `len` at `deg` degrees (0 = right,
/// counter-clockwise positive, as on paper), with `label` written beyond its tip at `size` px.
pub fn vec_arrow<'a>(t: f32, t0: f32, ox: f32, oy: f32, len: f32, deg: f32, color: &str, size: f32, label: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let a = deg.to_radians();
    let (ex, ey) = (ox + len * a.cos(), oy - len * a.sin());
    let shaft = arrow_stroke(t, t0, &Stroke::line(ox, oy, ex, ey), color, 4.0);
    let (lx, ly) = (ox + (len + 36.0) * a.cos(), oy - (len + 36.0) * a.sin());
    let lab = hand(t, t0 + 0.2, lx, ly + size * 0.35, size, color, Al::Mid, label);
    fframes::svgr!(<g>{shaft}{lab}</g>)
}

/// A curved arrow along a circle of radius `r` around (ox, oy), from `deg0` to `deg1` degrees (same
/// convention as `vec_arrow`): "rotated by this much".
pub fn rot_arc<'a>(t: f32, t0: f32, ox: f32, oy: f32, r: f32, deg0: f32, deg1: f32, color: &str) -> Svgr<'a> {
    let n = 24;
    let pts = (0..=n)
        .map(|k| {
            let a = (deg0 + (deg1 - deg0) * k as f32 / n as f32).to_radians();
            (ox + r * a.cos(), oy - r * a.sin())
        })
        .collect();
    arrow_stroke(t, t0, &Stroke::from_pts(pts), color, 3.0)
}

// ================================================================ pictures

/// A picture (a stand-in for any image): a frame (x, y, w, h) in `color` with a yellow sun and a hill line inside.
/// Draws on in ~0.65 s from t0.
pub fn photo<'a>(t: f32, t0: f32, x: f32, y: f32, w: f32, h: f32, color: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let m = w.min(h);
    let sw = (m * 0.025).max(2.5);
    let fill = tint(color, 0.10);
    let o = prog(t, t0, 0.2);
    let v: Vec<Svgr<'a>> = vec![
        opacity(o, fframes::svgr!(<rect x={x} y={y} width={w} height={h} rx="6" fill={fill} />)),
        ink(&Stroke::rect(x, y, w, h, 6.0), prog(t, t0, 0.35), color, sw * 1.2),
        ink(&Stroke::ellipse(x + w * 0.72, y + h * 0.3, m * 0.1, m * 0.1), prog(t, t0 + 0.25, 0.25), YELLOW, sw),
        ink(
            &Stroke::poly(&[(x + w * 0.06, y + h * 0.88), (x + w * 0.32, y + h * 0.45), (x + w * 0.5, y + h * 0.7), (x + w * 0.64, y + h * 0.55), (x + w * 0.94, y + h * 0.88)]),
            prog(t, t0 + 0.35, 0.3),
            color,
            sw,
        ),
    ];
    v.into()
}

// ================================================================ this video
// The demo video's own objects and colour meanings (replace them for a new video).

/// The value being searched for (pink: the task / the question).
pub const TARGET_C: &str = PINK;
/// The part of the list that is still in play.
pub const LIVE_C: &str = CYAN;
/// A guess (the middle element that gets compared).
pub const GUESS_C: &str = YELLOW;

/// Centre x of cell `i` of a `sorted_row` that starts at x with cells `cell` px wide.
pub fn cell_cx(x: f32, cell: f32, i: usize) -> f32 {
    x + cell * (i as f32 + 0.5)
}

/// A sorted list as a row of numbered cells from (x, y), each `cell` px wide (and 0.9 * cell tall), in
/// `LIVE_C`. Cells pop in left to right over `dur` seconds from `t0`. Cell `i` is dimmed to 0.25 from
/// `out[i]` (pass `f32::INFINITY` to keep it), for the halves that are thrown away.
pub fn sorted_row<'a>(t: f32, t0: f32, dur: f32, x: f32, y: f32, cell: f32, values: &[u32], out: &[f32]) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let n = values.len().max(1);
    let h = cell * 0.9;
    let size = (cell * 0.42).min(44.0);
    let mut v: Vec<Svgr<'a>> = Vec::new();
    for (i, val) in values.iter().enumerate() {
        let ti = t0 + dur * i as f32 / n as f32;
        let cx = cell_cx(x, cell, i);
        let body = fframes::svgr!(<g>
            {boxed(t, ti, cx - cell / 2.0 + 4.0, y, cell - 8.0, h, LIVE_C, true)}
            {hand(t, ti + 0.1, cx, y + h / 2.0 + size * 0.34, size, WHITE, Al::Mid, &val.to_string())}
        </g>);
        v.push(dim_after(t, out.get(i).copied().unwrap_or(f32::INFINITY), 0.25, body));
    }
    v.into()
}
