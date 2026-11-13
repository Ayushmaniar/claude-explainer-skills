//! The drawing kit: an Excalidraw-style whiteboard on a near-black canvas.
//!
//! Everything is drawn "by hand" and *appears when the narrator says it*:
//! - strokes (lines, boxes, circles, arrows, ticks, crosses, braces) draw on from start to end;
//! - handwriting is revealed by a left-to-right wipe at a constant ~750 px/s;
//! - paper figures are shown as white cards, annotated with highlighter, circles and spotlights;
//! - a camera can push in / pan over the scene.
//!
//! Conventions used by every helper:
//! - `t` is scene time in seconds (`frame.seconds()` of the scene), `t0` the moment the element
//!   starts appearing. Before `t0` a helper returns `Svgr::empty()`; afterwards it stays on screen
//!   (scenes are cleared by the hard cut to the next scene). Use `until(t, t1, ..)` to remove early.
//! - Times come from the narration: `c.w(beat, "word")` is when that word starts (see `cues.rs`).
//! - Coordinates are 1920x1080 canvas pixels. Keep the bottom-right corner (x > 1690, y > 900)
//!   free: the host sprite lives there. Keep the top-left chip area (x < 700, y < 150) free.
use fframes::Svgr;

use crate::figs::FIGS;
use crate::metrics::EXCALI_ADV;
pub use crate::objs::*;

// ================================================================ palette
// One hue = one meaning, for the whole video. Fix each video's meanings in STORYBOARD.md and give them
// names in objs.rs (e.g. `pub const STATE_C: &str = TEAL;`). The defaults below are the usual ones.
pub const BG: &str = "#030303";
pub const WHITE: &str = "#f2f2ef"; // neutral objects and labels
pub const GREY: &str = "#9a9a9a"; // secondary labels, axes, provenance captions
pub const DIM: &str = "#555555"; // inactive / old things
pub const YELLOW: &str = "#fcdb43"; // emphasis, numbers, titles, chips, caveat boxes
pub const CYAN: &str = "#59e8f9"; // a recurring concept (context, data, inputs ...)
pub const PURPLE: &str = "#a08cff"; // a second recurring concept
pub const TEAL: &str = "#1cd2ac"; // a third recurring concept (tools, actions ...)
pub const BLUE: &str = "#5499fa"; // the main system / the model
pub const NAVY: &str = "#072a54"; // fill of model boxes
pub const PINK: &str = "#ec70a8"; // the task / the user / questions
pub const MINT: &str = "#7eeeb7"; // good, success, tick
pub const RED: &str = "#f06070"; // bad, failure, cross, crash
pub const ORANGE: &str = "#fe8733"; // cost, warning, overflow

// Inks for marks drawn ON a white paper card (the canvas colours are too light there).
// Text on a card should use `note(..)`, which puts the handwriting on a dark backdrop.
pub const P_RED: &str = "#e0283c";
pub const P_BLUE: &str = "#1f5fd6";
pub const P_GREEN: &str = "#13965a";
pub const P_ORANGE: &str = "#f06a10";
pub const P_PURPLE: &str = "#6d3fe0";
pub const P_INK: &str = "#1a1a1a";

pub const HAND: &str = "Excalifont";
pub const MONO: &str = "JetBrains Mono";
pub const SANS: &str = "Inter";

/// Typical lead times: start drawing a label this much before its word is spoken.
pub const LEAD: f32 = 0.12;
/// Section titles are written a bit earlier, so they are readable as the word is said.
pub const LEAD_TITLE: f32 = 0.35;

// ================================================================ math

pub fn clamp01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}
pub fn lerp(a: f32, b: f32, p: f32) -> f32 {
    a + (b - a) * p
}
pub fn ease_out(x: f32) -> f32 {
    1.0 - (1.0 - clamp01(x)).powi(3)
}
pub fn ease_in(x: f32) -> f32 {
    clamp01(x).powi(3)
}
pub fn ease_in_out(x: f32) -> f32 {
    let x = clamp01(x);
    if x < 0.5 { 4.0 * x * x * x } else { 1.0 - (-2.0 * x + 2.0).powi(3) / 2.0 }
}
/// Eased (cubic out) 0 -> 1 between `t0` and `t0 + dur`.
pub fn prog(t: f32, t0: f32, dur: f32) -> f32 {
    ease_out((t - t0) / dur.max(1e-4))
}
/// Linear 0 -> 1 between `t0` and `t0 + dur`.
pub fn lin(t: f32, t0: f32, dur: f32) -> f32 {
    clamp01((t - t0) / dur.max(1e-4))
}

/// Deterministic pseudo-random number in [0, 1) from a seed.
pub fn rnd(seed: u32) -> f32 {
    let mut x = seed.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x = x.wrapping_mul(0x297A_2D39);
    x ^= x >> 15;
    (x >> 8) as f32 / (1u32 << 24) as f32
}

/// Seed from a few numbers (positions), so the same shape always wobbles the same way.
pub fn seed_of(v: &[f32]) -> u32 {
    let mut h: u32 = 2166136261;
    for f in v {
        h ^= (f * 8.0) as i32 as u32;
        h = h.wrapping_mul(16777619);
    }
    h
}

/// Smooth 1D noise in about [-1, 1].
fn noise(seed: u32, x: f32) -> f32 {
    let a = rnd(seed) * 6.283;
    let b = rnd(seed ^ 0xABCD) * 6.283;
    0.6 * (x * 1.7 + a).sin() + 0.4 * (x * 3.1 + b).sin()
}

/// Mixes a `#rrggbb` colour with the background: `a` = 0 is the background, 1 the colour.
pub fn tint(hex: &str, a: f32) -> String {
    let p = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0) as f32;
    let (r, g, b) = (p(1), p(3), p(5));
    let m = |c: f32| (3.0 + (c - 3.0) * a).round().clamp(0.0, 255.0) as u8;
    format!("#{:02x}{:02x}{:02x}", m(r), m(g), m(b))
}

// ================================================================ hand-drawn strokes

/// A hand-drawn polyline with known length, so it can be drawn on progressively.
#[derive(Clone, Debug)]
pub struct Stroke {
    pub pts: Vec<(f32, f32)>,
    cum: Vec<f32>,
}

impl Stroke {
    pub fn from_pts(pts: Vec<(f32, f32)>) -> Self {
        let mut cum = Vec::with_capacity(pts.len());
        let mut acc = 0.0;
        for i in 0..pts.len() {
            if i > 0 {
                let (dx, dy) = (pts[i].0 - pts[i - 1].0, pts[i].1 - pts[i - 1].1);
                acc += (dx * dx + dy * dy).sqrt();
            }
            cum.push(acc);
        }
        Self { pts, cum }
    }

    pub fn len(&self) -> f32 {
        self.cum.last().copied().unwrap_or(0.0)
    }

    /// Point at fraction `p` of the length, and the direction (radians) of travel there.
    pub fn at(&self, p: f32) -> (f32, f32, f32) {
        let n = self.pts.len();
        if n == 0 {
            return (0.0, 0.0, 0.0);
        }
        if n == 1 {
            return (self.pts[0].0, self.pts[0].1, 0.0);
        }
        let target = clamp01(p) * self.len();
        let mut i = 1;
        while i < n - 1 && self.cum[i] < target {
            i += 1;
        }
        let seg = (self.cum[i] - self.cum[i - 1]).max(1e-4);
        let u = clamp01((target - self.cum[i - 1]) / seg);
        let (a, b) = (self.pts[i - 1], self.pts[i]);
        // Direction from a point ~14 px back, for a stable arrowhead.
        let back = (target - 14.0).max(0.0);
        let mut j = 1;
        while j < n - 1 && self.cum[j] < back {
            j += 1;
        }
        let ub = clamp01((back - self.cum[j - 1]) / (self.cum[j] - self.cum[j - 1]).max(1e-4));
        let pb = (lerp(self.pts[j - 1].0, self.pts[j].0, ub), lerp(self.pts[j - 1].1, self.pts[j].1, ub));
        let (x, y) = (lerp(a.0, b.0, u), lerp(a.1, b.1, u));
        let ang = if (x - pb.0).abs() + (y - pb.1).abs() > 0.5 { (y - pb.1).atan2(x - pb.0) } else { (b.1 - a.1).atan2(b.0 - a.0) };
        (x, y, ang)
    }

    /// SVG path data for the first `p` (0..1) of the stroke.
    pub fn d(&self, p: f32) -> String {
        let p = clamp01(p);
        let target = p * self.len();
        let mut s = String::with_capacity(self.pts.len() * 14);
        for (i, &(x, y)) in self.pts.iter().enumerate() {
            if i > 0 && self.cum[i] > target {
                let seg = (self.cum[i] - self.cum[i - 1]).max(1e-4);
                let u = (target - self.cum[i - 1]) / seg;
                let (px, py) = self.pts[i - 1];
                s.push_str(&format!(" L{:.1} {:.1}", lerp(px, x, u), lerp(py, y, u)));
                break;
            }
            s.push_str(&format!("{}{:.1} {:.1}", if i == 0 { "M" } else { " L" }, x, y));
        }
        s
    }

    /// A slightly bowed, wobbly straight line.
    pub fn line(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let seed = seed_of(&[x1, y1, x2, y2]);
        let (dx, dy) = (x2 - x1, y2 - y1);
        let len = (dx * dx + dy * dy).sqrt().max(0.01);
        let (ux, uy) = (dx / len, dy / len);
        let (nx, ny) = (-uy, ux);
        let bow = (rnd(seed) - 0.5) * 2.0 * (len * 0.012).min(4.0);
        let n = ((len / 9.0) as usize).clamp(2, 160);
        let over = 1.5;
        let pts = (0..=n)
            .map(|k| {
                let u = k as f32 / n as f32;
                let along = -over + (len + 2.0 * over) * u;
                let off = bow * (std::f32::consts::PI * u).sin() + 0.7 * noise(seed, along / 90.0);
                (x1 + ux * along + nx * off, y1 + uy * along + ny * off)
            })
            .collect();
        Self::from_pts(pts)
    }

    /// Quadratic curve from (x1,y1) to (x2,y2) bent through control point (cx,cy).
    pub fn curve(x1: f32, y1: f32, cx: f32, cy: f32, x2: f32, y2: f32) -> Self {
        let seed = seed_of(&[x1, y1, cx, cy, x2, y2]);
        let approx = ((x2 - x1).hypot(y2 - y1) + (cx - x1).hypot(cy - y1) + (x2 - cx).hypot(y2 - cy)) / 2.0;
        let n = ((approx / 9.0) as usize).clamp(4, 200);
        let pts = (0..=n)
            .map(|k| {
                let u = k as f32 / n as f32;
                let a = (1.0 - u) * (1.0 - u);
                let b = 2.0 * (1.0 - u) * u;
                let c = u * u;
                let w = 0.6 * noise(seed, u * approx / 90.0);
                (a * x1 + b * cx + c * x2 + w, a * y1 + b * cy + c * y2 - w)
            })
            .collect();
        Self::from_pts(pts)
    }

    /// Straight segments through `points` (for ticks, zigzags, icons), lightly wobbled.
    pub fn poly(points: &[(f32, f32)]) -> Self {
        let mut pts = Vec::new();
        for w in points.windows(2) {
            let l = Self::line(w[0].0, w[0].1, w[1].0, w[1].1);
            let skip = if pts.is_empty() { 0 } else { 1 };
            pts.extend(l.pts.into_iter().skip(skip));
        }
        if pts.is_empty() {
            pts = points.to_vec();
        }
        Self::from_pts(pts)
    }

    /// A rounded rectangle drawn clockwise from the top-left, with a small overlap at the end.
    pub fn rect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Self {
        let seed = seed_of(&[x, y, w, h]);
        let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
        let j = |k: u32| (rnd(seed ^ k) - 0.5) * 3.0;
        let mut pts: Vec<(f32, f32)> = Vec::new();
        let edge = |ax: f32, ay: f32, bx: f32, by: f32, k: u32, pts: &mut Vec<(f32, f32)>| {
            let len = (bx - ax).hypot(by - ay).max(0.01);
            let (nx, ny) = (-(by - ay) / len, (bx - ax) / len);
            let bow = (rnd(seed ^ (k * 31)) - 0.5) * (len * 0.012).min(4.0);
            let n = ((len / 10.0) as usize).clamp(2, 200);
            for q in 0..=n {
                let u = q as f32 / n as f32;
                let off = bow * (std::f32::consts::PI * u).sin() + 0.6 * noise(seed ^ k, u * len / 90.0);
                pts.push((lerp(ax, bx, u) + nx * off, lerp(ay, by, u) + ny * off));
            }
        };
        let corner = |cx: f32, cy: f32, a0: f32, pts: &mut Vec<(f32, f32)>| {
            for q in 1..6 {
                let a = a0 + q as f32 / 6.0 * std::f32::consts::FRAC_PI_2;
                pts.push((cx + r * a.cos(), cy + r * a.sin()));
            }
        };
        use std::f32::consts::PI;
        edge(x + r + j(1), y + j(2), x + w - r, y, 1, &mut pts);
        corner(x + w - r, y + r, -PI / 2.0, &mut pts);
        edge(x + w, y + r, x + w, y + h - r, 2, &mut pts);
        corner(x + w - r, y + h - r, 0.0, &mut pts);
        edge(x + w - r, y + h, x + r, y + h, 3, &mut pts);
        corner(x + r, y + h - r, PI / 2.0, &mut pts);
        edge(x, y + h - r, x, y + r, 4, &mut pts);
        corner(x + r, y + r, PI, &mut pts);
        // Pen overlaps the start a little, slightly off the line.
        let over = (w * 0.08).clamp(6.0, 24.0);
        pts.push((x + r + over, y + 1.6));
        Self::from_pts(pts)
    }

    /// A hand-drawn ellipse: starts top-left, goes round ~1.08 times, slightly spiralling.
    pub fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32) -> Self {
        let seed = seed_of(&[cx, cy, rx, ry]);
        let a0 = -2.3 + (rnd(seed) - 0.5) * 0.4;
        let turn = 1.08 * std::f32::consts::TAU;
        let n = 72;
        let phi = rnd(seed ^ 7) * 6.283;
        let tilt = (rnd(seed ^ 9) - 0.5) * 0.08;
        let pts = (0..=n)
            .map(|k| {
                let u = k as f32 / n as f32;
                let a = a0 + turn * u;
                let f = 1.0 + 0.025 * (2.0 * a + phi).sin() + 0.06 * (u - 0.5);
                let (ex, ey) = (rx * f * a.cos(), ry * f * a.sin());
                (cx + ex * tilt.cos() - ey * tilt.sin(), cy + ex * tilt.sin() + ey * tilt.cos())
            })
            .collect();
        Self::from_pts(pts)
    }
}

/// Draws the first `p` of a stroke.
pub fn ink<'a>(s: &Stroke, p: f32, color: &str, width: f32) -> Svgr<'a> {
    if p <= 0.0 {
        return Svgr::empty();
    }
    let d = s.d(p);
    let color = color.to_string();
    fframes::svgr!(<path d={d} fill="none" stroke={color} stroke-width={width} stroke-linecap="round" stroke-linejoin="round" />)
}

/// Draw-on duration for a stroke of length `len` (px): fast for short marks, max ~0.6 s.
pub fn draw_dur(len: f32) -> f32 {
    (len / 1400.0 + 0.12).clamp(0.15, 0.6)
}

// ================================================================ animated marks

/// A straight hand-drawn line that draws on from (x1,y1) at `t0`.
pub fn line<'a>(t: f32, t0: f32, x1: f32, y1: f32, x2: f32, y2: f32, color: &str, w: f32) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let s = Stroke::line(x1, y1, x2, y2);
    ink(&s, prog(t, t0, draw_dur(s.len())), color, w)
}

/// Underline sweep (0.27 s), 4 px, round caps. `y` is where the line goes (~12 px under the baseline).
pub fn underline<'a>(t: f32, t0: f32, x: f32, y: f32, w: f32, color: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let s = Stroke::curve(x, y + 1.0, x + w * 0.5, y + 4.0, x + w, y - 2.0);
    ink(&s, prog(t, t0, 0.27), color, 4.0)
}

/// Open "V" arrowhead at (x, y) pointing along `ang`.
pub fn arrowhead<'a>(x: f32, y: f32, ang: f32, size: f32, color: &str, w: f32) -> Svgr<'a> {
    let (a1, a2) = (ang + 2.55, ang - 2.55);
    let d = format!(
        "M{:.1} {:.1} L{:.1} {:.1} L{:.1} {:.1}",
        x + size * a1.cos(),
        y + size * a1.sin(),
        x,
        y,
        x + size * a2.cos(),
        y + size * a2.sin()
    );
    let color = color.to_string();
    fframes::svgr!(<path d={d} fill="none" stroke={color} stroke-width={w} stroke-linecap="round" stroke-linejoin="round" />)
}

/// Draws a stroke and gives it an arrowhead once the shaft is nearly complete.
pub fn arrow_stroke<'a>(t: f32, t0: f32, s: &Stroke, color: &str, w: f32) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let p = prog(t, t0, draw_dur(s.len()));
    let shaft = ink(s, p, color, w);
    if p < 0.8 {
        return shaft;
    }
    let (x, y, ang) = s.at(1.0);
    let k = clamp01((p - 0.8) / 0.2);
    let head = arrowhead(x, y, ang, 9.0 + 9.0 * k, color, w);
    fframes::svgr!(<g>{shaft}{head}</g>)
}

/// Straight arrow from (x1,y1) to (x2,y2).
pub fn arrow<'a>(t: f32, t0: f32, x1: f32, y1: f32, x2: f32, y2: f32, color: &str) -> Svgr<'a> {
    arrow_stroke(t, t0, &Stroke::line(x1, y1, x2, y2), color, 3.0)
}

/// Curved arrow bending through (cx, cy).
pub fn arrow_curve<'a>(t: f32, t0: f32, x1: f32, y1: f32, cx: f32, cy: f32, x2: f32, y2: f32, color: &str) -> Svgr<'a> {
    arrow_stroke(t, t0, &Stroke::curve(x1, y1, cx, cy, x2, y2), color, 3.0)
}

/// Hand-drawn box: the outline draws on (~0.25-0.45 s), a dark tint of the colour fills in.
/// `fill` = false leaves it empty (outline only).
pub fn boxed<'a>(t: f32, t0: f32, x: f32, y: f32, w: f32, h: f32, color: &str, fill: bool) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let s = Stroke::rect(x, y, w, h, 14.0);
    let p = prog(t, t0, (s.len() / 3000.0 + 0.2).min(0.45));
    let outline = ink(&s, p, color, 3.0);
    if !fill {
        return outline;
    }
    let f = tint(color, 0.13);
    let fo = prog(t, t0 + 0.1, 0.3);
    fframes::svgr!(<g>
        <rect x={x} y={y} width={w.max(1.0)} height={h.max(1.0)} rx="14" fill={f} opacity={fo} />
        {outline}
    </g>)
}

/// Like `boxed` but with an explicit fill colour (e.g. NAVY for the model).
pub fn boxed_fill<'a>(t: f32, t0: f32, x: f32, y: f32, w: f32, h: f32, color: &str, fill: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let s = Stroke::rect(x, y, w, h, 14.0);
    let p = prog(t, t0, (s.len() / 3000.0 + 0.2).min(0.45));
    let fo = prog(t, t0 + 0.1, 0.3);
    let outline = ink(&s, p, color, 3.0);
    let fill = fill.to_string();
    fframes::svgr!(<g>
        <rect x={x} y={y} width={w.max(1.0)} height={h.max(1.0)} rx="14" fill={fill} opacity={fo} />
        {outline}
    </g>)
}

/// Dashed outline box (placeholders, "optional", "removed"), fades in over 0.25 s.
pub fn dashed_box<'a>(t: f32, t0: f32, x: f32, y: f32, w: f32, h: f32, color: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let o = prog(t, t0, 0.25);
    let color = color.to_string();
    fframes::svgr!(<rect x={x} y={y} width={w.max(1.0)} height={h.max(1.0)} rx="14" fill="none" stroke={color} stroke-width="3" stroke-dasharray="14 10" opacity={o} />)
}

/// Hand-drawn ellipse/circle around something (0.45 s).
pub fn circle<'a>(t: f32, t0: f32, cx: f32, cy: f32, rx: f32, ry: f32, color: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let s = Stroke::ellipse(cx, cy, rx, ry);
    ink(&s, prog(t, t0, 0.45), color, 3.5)
}

/// Green tick, drawn in 0.2 s. `size` is the tick's width.
pub fn tick<'a>(t: f32, t0: f32, cx: f32, cy: f32, size: f32, color: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let k = size / 2.0;
    let s = Stroke::poly(&[(cx - k, cy), (cx - k * 0.3, cy + k * 0.7), (cx + k, cy - k * 0.8)]);
    ink(&s, prog(t, t0, 0.2), color, (size * 0.11).max(3.0))
}

/// Cross (two strokes of 0.15 s, the second 0.1 s later).
pub fn cross<'a>(t: f32, t0: f32, cx: f32, cy: f32, size: f32, color: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let k = size / 2.0;
    let a = Stroke::line(cx - k, cy - k, cx + k, cy + k);
    let b = Stroke::line(cx + k, cy - k, cx - k, cy + k);
    let w = (size * 0.09).max(3.5);
    fframes::svgr!(<g>{ink(&a, prog(t, t0, 0.15), color, w)}{ink(&b, prog(t, t0 + 0.1, 0.15), color, w)}</g>)
}

/// Strike-through line across text from x1 to x2 at height y (0.25 s).
pub fn strike<'a>(t: f32, t0: f32, x1: f32, x2: f32, y: f32, color: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let s = Stroke::line(x1 - 6.0, y + 3.0, x2 + 6.0, y - 3.0);
    ink(&s, prog(t, t0, 0.25), color, 4.0)
}

/// Curly brace from x1 to x2 at height y, tip pointing down (`down`) or up (0.4 s).
pub fn brace<'a>(t: f32, t0: f32, x1: f32, x2: f32, y: f32, down: bool, color: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let k = if down { 1.0 } else { -1.0 };
    let m = (x1 + x2) / 2.0;
    let h = 14.0 * k;
    let pts = [
        (x1, y - h),
        (x1 + 6.0, y - h * 0.15),
        (x1 + 22.0, y),
        (m - 22.0, y),
        (m - 6.0, y + h * 0.3),
        (m, y + h),
    ];
    let pts2 = [
        (x2, y - h),
        (x2 - 6.0, y - h * 0.15),
        (x2 - 22.0, y),
        (m + 22.0, y),
        (m + 6.0, y + h * 0.3),
        (m, y + h),
    ];
    let p = prog(t, t0, 0.4);
    fframes::svgr!(<g>{ink(&Stroke::poly(&pts), p, color, 3.0)}{ink(&Stroke::poly(&pts2), p, color, 3.0)}</g>)
}

/// Highlighter band behind something (on paper cards): grows left to right in 0.25 s.
/// Yellow at ~40 % reads like a real marker on the white paper.
pub fn highlight<'a>(t: f32, t0: f32, x: f32, y: f32, w: f32, h: f32, color: &str, alpha: f32) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let ww = (w * prog(t, t0, 0.25)).max(0.5);
    let color = color.to_string();
    fframes::svgr!(<rect x={x} y={y} width={ww} height={h.max(0.5)} rx="4" fill={color} opacity={alpha} />)
}

/// Dims everything except the rectangle (x,y,w,h) inside the area (ax,ay,aw,ah), for 0.35 s in.
/// Use it to say "look only here" on a figure. Ends at `t1` (fades out 0.3 s).
pub fn spotlight<'a>(t: f32, t0: f32, t1: f32, area: (f32, f32, f32, f32), hole: (f32, f32, f32, f32), dim: f32) -> Svgr<'a> {
    if t < t0 || t > t1 + 0.3 {
        return Svgr::empty();
    }
    let o = prog(t, t0, 0.35).min(1.0 - ease_in_out((t - t1) / 0.3)) * dim;
    let (ax, ay, aw, ah) = area;
    let (x, y, w, h) = hole;
    let d = format!(
        "M{ax} {ay} h{aw} v{ah} h{} Z M{x} {y} v{h} h{w} v{} Z",
        -aw, -h
    );
    fframes::svgr!(<path d={d} fill="#000000" fill-rule="evenodd" opacity={o} />)
}

// ================================================================ text

/// Width in px of `s` in Excalifont at `size` (from the font's advance widths).
pub fn text_w(s: &str, size: f32) -> f32 {
    segs(s).iter().map(|g| seg_w(g, size)).sum()
}

/// Width of monospace text.
pub fn mono_w(s: &str, size: f32) -> f32 {
    s.chars().count() as f32 * size * 0.6
}

#[derive(Debug)]
enum Seg {
    Text(String),
    Glyph(char),
}

/// Excalifont has no arrows, ticks or ⋆ (as in h⋆): those characters are drawn as strokes.
fn is_drawn(c: char) -> bool {
    matches!(c, '→' | '←' | '↑' | '↓' | '✓' | '✗' | '≈' | '⋆')
}

fn segs(s: &str) -> Vec<Seg> {
    let mut v = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        if is_drawn(c) {
            if !cur.is_empty() {
                v.push(Seg::Text(std::mem::take(&mut cur)));
            }
            v.push(Seg::Glyph(c));
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        v.push(Seg::Text(cur));
    }
    v
}

fn char_w(c: char) -> f32 {
    let i = c as u32;
    if (32..=126).contains(&i) {
        return EXCALI_ADV[(i - 32) as usize];
    }
    match c {
        '·' => 0.2,
        '×' => 0.55,
        '…' => 0.71,
        '—' => 0.94,
        '–' => 0.7,
        '’' => 0.3,
        _ => 0.6,
    }
}

fn seg_w(g: &Seg, size: f32) -> f32 {
    match g {
        Seg::Text(s) => s.chars().map(char_w).sum::<f32>() * size,
        Seg::Glyph('↑') | Seg::Glyph('↓') => 0.6 * size,
        Seg::Glyph('⋆') => 0.42 * size,
        Seg::Glyph(_) => 0.95 * size,
    }
}

fn glyph<'a>(c: char, x: f32, base: f32, size: f32, color: &str) -> Svgr<'a> {
    let w = (size * 0.075).max(2.5);
    let mid = base - size * 0.3;
    let s = size;
    match c {
        '→' => {
            let st = Stroke::line(x + 0.1 * s, mid, x + 0.82 * s, mid);
            fframes::svgr!(<g>{ink(&st, 1.0, color, w)}{arrowhead(x + 0.82 * s, mid, 0.0, 0.22 * s, color, w)}</g>)
        }
        '←' => {
            let st = Stroke::line(x + 0.85 * s, mid, x + 0.13 * s, mid);
            fframes::svgr!(<g>{ink(&st, 1.0, color, w)}{arrowhead(x + 0.13 * s, mid, std::f32::consts::PI, 0.22 * s, color, w)}</g>)
        }
        '↑' => {
            let st = Stroke::line(x + 0.3 * s, base + 0.02 * s, x + 0.3 * s, base - 0.66 * s);
            fframes::svgr!(<g>{ink(&st, 1.0, color, w)}{arrowhead(x + 0.3 * s, base - 0.66 * s, -std::f32::consts::FRAC_PI_2, 0.2 * s, color, w)}</g>)
        }
        '↓' => {
            let st = Stroke::line(x + 0.3 * s, base - 0.68 * s, x + 0.3 * s, base);
            fframes::svgr!(<g>{ink(&st, 1.0, color, w)}{arrowhead(x + 0.3 * s, base, std::f32::consts::FRAC_PI_2, 0.2 * s, color, w)}</g>)
        }
        '✓' => {
            let st = Stroke::poly(&[(x + 0.12 * s, mid), (x + 0.35 * s, base - 0.05 * s), (x + 0.82 * s, base - 0.62 * s)]);
            ink(&st, 1.0, color, w * 1.2)
        }
        '✗' => {
            let a = Stroke::line(x + 0.15 * s, base - 0.6 * s, x + 0.75 * s, base);
            let b = Stroke::line(x + 0.75 * s, base - 0.6 * s, x + 0.15 * s, base);
            fframes::svgr!(<g>{ink(&a, 1.0, color, w * 1.2)}{ink(&b, 1.0, color, w * 1.2)}</g>)
        }
        '⋆' => {
            // A small filled five-point star, superscript (h⋆).
            let (cx, cy, r) = (x + 0.2 * s, base - 0.62 * s, 0.17 * s);
            let mut d = String::new();
            for i in 0..10 {
                let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
                let rr = if i % 2 == 0 { r } else { r * 0.45 };
                d.push_str(&format!("{}{:.1} {:.1}", if i == 0 { "M" } else { " L" }, cx + rr * a.cos(), cy + rr * a.sin()));
            }
            d.push_str(" Z");
            let color = color.to_string();
            fframes::svgr!(<path d={d} fill={color.clone()} stroke={color} stroke-width={w * 0.5} stroke-linejoin="round" />)
        }
        _ => {
            // ≈: two wavy lines.
            let a = Stroke::curve(x + 0.12 * s, mid - 0.08 * s, x + 0.47 * s, mid - 0.28 * s, x + 0.82 * s, mid - 0.08 * s);
            let b = Stroke::curve(x + 0.12 * s, mid + 0.12 * s, x + 0.47 * s, mid - 0.08 * s, x + 0.82 * s, mid + 0.12 * s);
            fframes::svgr!(<g>{ink(&a, 1.0, color, w)}{ink(&b, 1.0, color, w)}</g>)
        }
    }
}

/// Text alignment.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Al {
    Left,
    Mid,
    Right,
}

/// Static handwriting, `x`,`y` = anchor and baseline. Handles → ← ↑ ↓ ✓ ✗ ≈ as drawn glyphs.
pub fn txt<'a>(x: f32, y: f32, size: f32, color: &str, al: Al, s: &str) -> Svgr<'a> {
    let total = text_w(s, size);
    let mut cx = match al {
        Al::Left => x,
        Al::Mid => x - total / 2.0,
        Al::Right => x - total,
    };
    let mut v: Vec<Svgr<'a>> = Vec::new();
    for g in segs(s) {
        let w = seg_w(&g, size);
        match g {
            Seg::Text(ref s) => {
                // usvg trims leading spaces, so move the text right by their width instead.
                let lead = s.len() - s.trim_start().len();
                let tx = cx + lead as f32 * char_w(' ') * size;
                let s = s.trim().to_string();
                let color = color.to_string();
                if !s.is_empty() {
                    v.push(fframes::svgr!(<text x={tx} y={y} font-family={HAND} font-size={size} font-weight="400" fill={color}>{s}</text>));
                }
            }
            Seg::Glyph(c) => v.push(glyph(c, cx, y, size, color)),
        }
        cx += w;
    }
    v.into()
}

/// Duration of a handwriting wipe for text of width `w` px.
pub fn write_dur(w: f32) -> f32 {
    (w / 750.0).clamp(0.22, 1.1)
}

/// Clips `child` to x in [x0, x1] (whole height of the canvas).
fn clip_x<'a>(x0: f32, x1: f32, id: String, child: Svgr<'a>) -> Svgr<'a> {
    let url = format!("url(#{id})");
    fframes::svgr!(<g>
        <defs><clipPath id={id}><rect x={x0} y="-3000" width={(x1 - x0).max(0.01)} height="9000" /></clipPath></defs>
        <g clip-path={url}>{child}</g>
    </g>)
}

/// Reveals `child` (laid out between x0 and x0+w) left to right with a soft 22 px edge.
pub fn wipe<'a>(p: f32, x0: f32, w: f32, key: u32, child: impl Fn() -> Svgr<'a>) -> Svgr<'a> {
    if p <= 0.0 {
        return Svgr::empty();
    }
    if p >= 1.0 {
        return child();
    }
    let edge = x0 - 4.0 + (w + 30.0) * p;
    let soft = 22.0;
    let a = clip_x(x0 - 400.0, edge - soft, format!("wa{key}"), child());
    let b = clip_x(edge - soft, edge, format!("wb{key}"), child());
    fframes::svgr!(<g>{a}<g opacity="0.45">{b}</g></g>)
}

/// Handwriting that writes itself left to right from `t0` (constant speed, ~750 px/s).
/// Returns empty before `t0`. `s` may contain → ✓ etc.
pub fn hand<'a>(t: f32, t0: f32, x: f32, y: f32, size: f32, color: &str, al: Al, s: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let w = text_w(s, size);
    let p = lin(t, t0, write_dur(w));
    let x0 = match al {
        Al::Left => x,
        Al::Mid => x - w / 2.0,
        Al::Right => x - w,
    };
    let key = seed_of(&[x, y, size, s.len() as f32, t0]);
    wipe(p, x0, w, key, || txt(x, y, size, color, al, s))
}

/// When a `hand(..)` started at `t0` finishes writing.
pub fn hand_end(t0: f32, s: &str, size: f32) -> f32 {
    t0 + write_dur(text_w(s, size))
}

/// Handwriting on a small dark rounded backdrop: readable anywhere, also on top of a white
/// paper card. The backdrop pops in, then the text writes. Use for labels on figures.
pub fn note<'a>(t: f32, t0: f32, x: f32, y: f32, size: f32, color: &str, al: Al, s: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let w = text_w(s, size);
    let x0 = match al {
        Al::Left => x,
        Al::Mid => x - w / 2.0,
        Al::Right => x - w,
    };
    let (px, py) = (size * 0.35, size * 0.22);
    let (bx, by, bw, bh) = (x0 - px, y - size * 0.82 - py, w + 2.0 * px, size * 1.12 + 2.0 * py);
    let o = prog(t, t0, 0.12);
    fframes::svgr!(<g>
        <rect x={bx} y={by} width={bw} height={bh} rx={size * 0.3} fill="#0b0b0b" opacity={0.93 * o} />
        {hand(t, t0 + 0.05, x, y, size, color, al, s)}
    </g>)
}

/// Several lines written one after another (each starts when the previous ends).
pub fn hand_lines<'a>(t: f32, t0: f32, x: f32, y: f32, size: f32, lh: f32, color: &str, al: Al, lines: &[&str]) -> Svgr<'a> {
    let mut start = t0;
    let mut v = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        v.push(hand(t, start, x, y + i as f32 * lh, size, color, al, l));
        start = hand_end(start, l, size);
    }
    v.into()
}

/// Static monospace text.
pub fn mono<'a>(x: f32, y: f32, size: f32, color: &str, s: &str) -> Svgr<'a> {
    let lead = s.len() - s.trim_start().len();
    let x = x + lead as f32 * size * 0.6;
    let s = s.trim().to_string();
    if s.is_empty() {
        return Svgr::empty();
    }
    let color = color.to_string();
    fframes::svgr!(<text x={x} y={y} font-family={MONO} font-size={size} font-weight="400" fill={color}>{s}</text>)
}

/// Monospace text typed character by character at `cps` characters per second.
pub fn mono_type<'a>(t: f32, t0: f32, x: f32, y: f32, size: f32, color: &str, s: &str, cps: f32) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let n = (((t - t0) * cps) as usize).min(s.chars().count());
    let shown: String = s.chars().take(n).collect();
    mono(x, y, size, color, &shown)
}

/// Number that counts up from `from` to `to` with ease-out over `dur` seconds (stepped every
/// 0.1 s like the reference), formatted with `dec` decimals between `pre` and `suf`.
pub fn count_str(t: f32, t0: f32, dur: f32, from: f32, to: f32, dec: usize, pre: &str, suf: &str) -> String {
    let tt = if t >= t0 + dur { t } else { t0 + ((t - t0) / 0.1).floor() * 0.1 };
    let v = lerp(from, to, prog(tt, t0, dur));
    format!("{pre}{v:.dec$}{suf}")
}

/// A counting handwritten number (shown from `t0`).
pub fn counter<'a>(t: f32, t0: f32, dur: f32, x: f32, y: f32, size: f32, color: &str, al: Al, from: f32, to: f32, dec: usize, pre: &str, suf: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let s = count_str(t, t0, dur, from, to, dec, pre, suf);
    let o = prog(t, t0, 0.12);
    fframes::svgr!(<g opacity={o}>{txt(x, y, size, color, al, &s)}</g>)
}

// ================================================================ entrances and exits

pub fn opacity<'a>(o: f32, child: Svgr<'a>) -> Svgr<'a> {
    if o <= 0.001 {
        return Svgr::empty();
    }
    if o >= 0.999 {
        return child;
    }
    fframes::svgr!(<g opacity={o}>{child}</g>)
}

pub fn translate<'a>(x: f32, y: f32, child: Svgr<'a>) -> Svgr<'a> {
    fframes::svgr!(<g transform={format!("translate({x:.2} {y:.2})")}>{child}</g>)
}

/// Scales `child` by `s` around (cx, cy).
pub fn scale_at<'a>(cx: f32, cy: f32, s: f32, child: Svgr<'a>) -> Svgr<'a> {
    let (tx, ty) = (cx * (1.0 - s), cy * (1.0 - s));
    fframes::svgr!(<g transform={format!("translate({tx:.2} {ty:.2}) scale({s:.4})")}>{child}</g>)
}

pub fn rotate_at<'a>(cx: f32, cy: f32, deg: f32, child: Svgr<'a>) -> Svgr<'a> {
    fframes::svgr!(<g transform={format!("rotate({deg:.3} {cx:.1} {cy:.1})")}>{child}</g>)
}

/// Fades in over `dur` from `t0`.
pub fn fade<'a>(t: f32, t0: f32, dur: f32, child: Svgr<'a>) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    opacity(prog(t, t0, dur), child)
}

/// Icon/photo entrance: fade + 10 px rise in 0.25 s.
pub fn rise<'a>(t: f32, t0: f32, child: Svgr<'a>) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let p = prog(t, t0, 0.25);
    opacity(p, translate(0.0, (1.0 - p) * 10.0, child))
}

/// Pop: appears in one frame at scale 1.06 and settles to 1 in 0.12 s (numbers, marks, stamps).
pub fn pop<'a>(t: f32, t0: f32, cx: f32, cy: f32, child: Svgr<'a>) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let p = prog(t, t0, 0.14);
    scale_at(cx, cy, 1.06 - 0.06 * p, opacity(prog(t, t0, 0.05), child))
}

/// Removes `child` at `t1` with a quick 0.25 s fade (most things just stay until the cut).
pub fn until<'a>(t: f32, t1: f32, child: Svgr<'a>) -> Svgr<'a> {
    if t >= t1 + 0.25 {
        return Svgr::empty();
    }
    if t <= t1 {
        return child;
    }
    opacity(1.0 - ease_in((t - t1) / 0.25), child)
}

/// Dims `child` to `to` opacity from `t0` over 0.4 s (old content that stays as context).
pub fn dim_after<'a>(t: f32, t0: f32, to: f32, child: Svgr<'a>) -> Svgr<'a> {
    opacity(1.0 - (1.0 - to) * prog(t, t0, 0.4), child)
}

/// A placement: content translated by (x, y), scaled by `s` around the origin, with opacity `o`.
#[derive(Clone, Copy, Debug)]
pub struct Place {
    pub x: f32,
    pub y: f32,
    pub s: f32,
    pub o: f32,
}

pub const HOME: Place = Place { x: 0.0, y: 0.0, s: 1.0, o: 1.0 };

/// Moves `child` from place `a` to place `b` between `t0` and `t0+dur` (ease-in-out).
/// The classic "shrink-aside": `move_to(t, t0, 0.8, HOME, Place { x: 1300.0, y: 120.0, s: 0.3, o: 0.35 }, ..)`.
pub fn move_to<'a>(t: f32, t0: f32, dur: f32, a: Place, b: Place, child: Svgr<'a>) -> Svgr<'a> {
    let p = ease_in_out((t - t0) / dur);
    let (x, y, s, o) = (lerp(a.x, b.x, p), lerp(a.y, b.y, p), lerp(a.s, b.s, p), lerp(a.o, b.o, p));
    opacity(o, fframes::svgr!(<g transform={format!("translate({x:.2} {y:.2}) scale({s:.4})")}>{child}</g>))
}

/// Shrinks the box (x,y,w,h) of the canvas into the target box (tx,ty,tw) (aspect kept) and dims it.
pub fn shrink_aside<'a>(t: f32, t0: f32, from: (f32, f32, f32, f32), to: (f32, f32, f32), dim: f32, child: Svgr<'a>) -> Svgr<'a> {
    let (x, y, w, _h) = from;
    let (tx, ty, tw) = to;
    let s = tw / w;
    let b = Place { x: tx - x * s, y: ty - y * s, s, o: dim };
    move_to(t, t0, 0.8, HOME, b, child)
}

// ================================================================ camera

/// A camera key: from time `t`, move (1.0 s, ease-in-out) so that the world point (fx, fy) is
/// at the centre of the screen at zoom `s`. The first key is usually `Key(0.0, 1.0, 960.0, 540.0)`.
#[derive(Clone, Copy, Debug)]
pub struct Key(pub f32, pub f32, pub f32, pub f32);

/// Duration of a camera move.
pub const CAM_MOVE: f32 = 1.0;

/// Wraps the scene in the camera. Adds a slow push-in (+0.08 %/s, max +3 %) while holding.
pub fn camera<'a>(t: f32, keys: &[Key], child: Svgr<'a>) -> Svgr<'a> {
    let drift = |since: f32| 1.0 + (since.max(0.0) * 0.0008).min(0.03);
    let mut cur = (1.0f32, 960.0f32, 540.0f32);
    let mut cur_t = 0.0f32;
    let mut moving: Option<((f32, f32, f32), (f32, f32, f32), f32)> = None;
    for k in keys {
        if k.0 > t {
            break;
        }
        // State at the start of this key (where the previous one left off, incl. drift).
        let base = match moving {
            Some((a, b, t0)) => {
                let p = ease_in_out((k.0 - t0) / CAM_MOVE);
                let arrived = (lerp(a.0, b.0, p), lerp(a.1, b.1, p), lerp(a.2, b.2, p));
                (arrived.0 * if p >= 1.0 { drift(k.0 - t0 - CAM_MOVE) } else { 1.0 }, arrived.1, arrived.2)
            }
            None => (cur.0 * drift(k.0 - cur_t), cur.1, cur.2),
        };
        moving = Some((base, (k.1, k.2, k.3), k.0));
        cur = (k.1, k.2, k.3);
        cur_t = k.0;
    }
    let (s, fx, fy) = match moving {
        Some((a, b, t0)) => {
            let p = ease_in_out((t - t0) / CAM_MOVE);
            let v = (lerp(a.0, b.0, p), lerp(a.1, b.1, p), lerp(a.2, b.2, p));
            let d = if t > t0 + CAM_MOVE { drift(t - t0 - CAM_MOVE) } else { 1.0 };
            (v.0 * d, v.1, v.2)
        }
        None => (cur.0 * drift(t), cur.1, cur.2),
    };
    let (tx, ty) = (960.0 - s * fx, 540.0 - s * fy);
    fframes::svgr!(<g transform={format!("translate({tx:.2} {ty:.2}) scale({s:.4})")}>{child}</g>)
}

// ================================================================ paper figures

/// Pixel size of a figure in `figs/` (0,0 if unknown).
pub fn fig_size(name: &str) -> (f32, f32) {
    FIGS.iter().find(|f| f.0 == name).map(|f| (f.1, f.2)).unwrap_or((0.0, 0.0))
}

/// A figure from the paper, shown as a white card. Coordinates of things *inside* the figure
/// are given in the figure's own pixels (as in `figs/<name>.png`)
/// and converted with `at` / `r`. A card can show only a crop of the figure.
#[derive(Clone, Copy, Debug)]
pub struct Card {
    pub name: &'static str,
    /// Top-left of the shown image area on the canvas.
    pub x: f32,
    pub y: f32,
    /// Size of the shown image area on the canvas.
    pub w: f32,
    pub h: f32,
    /// Canvas px per figure px.
    pub k: f32,
    /// Shown crop of the figure, in figure px (x0, y0, x1, y1).
    pub crop: (f32, f32, f32, f32),
}

impl Card {
    /// Whole figure, `w` px wide, top-left at (x, y).
    pub fn new(name: &'static str, x: f32, y: f32, w: f32) -> Self {
        let (fw, fh) = fig_size(name);
        Self::crop(name, (0.0, 0.0, fw, fh), x, y, w)
    }

    /// Whole figure, as large as fits in the box (bx, by, bw, bh), centred in it.
    pub fn fit(name: &'static str, bx: f32, by: f32, bw: f32, bh: f32) -> Self {
        let (fw, fh) = fig_size(name);
        Self::fit_crop(name, (0.0, 0.0, fw, fh), bx, by, bw, bh)
    }

    /// Part (x0,y0,x1,y1 in figure px) of a figure, `w` px wide, top-left at (x, y).
    pub fn crop(name: &'static str, crop: (f32, f32, f32, f32), x: f32, y: f32, w: f32) -> Self {
        let cw = (crop.2 - crop.0).max(1.0);
        let k = w / cw;
        Self { name, x, y, w, h: (crop.3 - crop.1) * k, k, crop }
    }

    /// A crop, as large as fits in the box, centred in it.
    pub fn fit_crop(name: &'static str, crop: (f32, f32, f32, f32), bx: f32, by: f32, bw: f32, bh: f32) -> Self {
        let (cw, ch) = ((crop.2 - crop.0).max(1.0), (crop.3 - crop.1).max(1.0));
        let k = (bw / cw).min(bh / ch);
        let (w, h) = (cw * k, ch * k);
        Self { name, x: bx + (bw - w) / 2.0, y: by + (bh - h) / 2.0, w, h, k, crop }
    }

    /// Canvas point of figure pixel (px, py).
    pub fn at(&self, px: f32, py: f32) -> (f32, f32) {
        (self.x + (px - self.crop.0) * self.k, self.y + (py - self.crop.1) * self.k)
    }

    /// Canvas rectangle (x, y, w, h) of the figure-pixel rectangle (x0, y0, x1, y1).
    pub fn r(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> (f32, f32, f32, f32) {
        let (a, b) = self.at(x0, y0);
        (a, b, (x1 - x0) * self.k, (y1 - y0) * self.k)
    }

    /// The card's outer rectangle (image area plus the white margin).
    pub fn outer(&self) -> (f32, f32, f32, f32) {
        (self.x - PAD, self.y - PAD, self.w + 2.0 * PAD, self.h + 2.0 * PAD)
    }

    /// Bottom edge of the card (for a caption).
    pub fn bottom(&self) -> f32 {
        self.y + self.h + PAD
    }

    /// The card, static.
    pub fn draw<'a>(&self) -> Svgr<'a> {
        let Some(href) = crate::image(self.name) else {
            return Svgr::empty();
        };
        let (fw, fh) = fig_size(self.name);
        let (ox, oy, ow, oh) = self.outer();
        let id = format!("cc{}", seed_of(&[self.x, self.y, self.w, self.crop.0, self.crop.1]));
        let url = format!("url(#{id})");
        // The whole image is placed so that the crop lands on (x, y); a clip keeps only the crop.
        let ix = self.x - self.crop.0 * self.k;
        let iy = self.y - self.crop.1 * self.k;
        fframes::svgr!(<g>
            <rect x={ox + 5.0} y={oy + 7.0} width={ow} height={oh} rx="8" fill="#000000" opacity="0.6" />
            <rect x={ox} y={oy} width={ow} height={oh} rx="8" fill="#ffffff" />
            <defs><clipPath id={id}><rect x={self.x} y={self.y} width={self.w} height={self.h} /></clipPath></defs>
            <g clip-path={url}>
                <image href={href} x={ix} y={iy} width={fw * self.k} height={fh * self.k} preserveAspectRatio="none" />
            </g>
        </g>)
    }

    /// The card entering at `t0`: fade + 14 px rise over 0.35 s.
    pub fn show<'a>(&self, t: f32, t0: f32) -> Svgr<'a> {
        if t < t0 {
            return Svgr::empty();
        }
        let p = prog(t, t0, 0.35);
        opacity(p, translate(0.0, (1.0 - p) * 14.0, self.draw()))
    }

    /// Grey handwritten provenance caption under the card, e.g. "paper · Figure 3".
    pub fn caption<'a>(&self, t: f32, t0: f32, s: &str) -> Svgr<'a> {
        hand(t, t0, self.x - PAD, self.bottom() + 34.0, 26.0, GREY, Al::Left, s)
    }

    /// Highlighter band over figure-pixel rect (yellow 40 % by default).
    pub fn mark<'a>(&self, t: f32, t0: f32, x0: f32, y0: f32, x1: f32, y1: f32) -> Svgr<'a> {
        let (x, y, w, h) = self.r(x0, y0, x1, y1);
        highlight(t, t0, x, y, w, h, YELLOW, 0.42)
    }

    /// Hand-drawn ellipse around figure-pixel rect, with a little margin.
    pub fn ring<'a>(&self, t: f32, t0: f32, x0: f32, y0: f32, x1: f32, y1: f32, color: &str) -> Svgr<'a> {
        let (x, y, w, h) = self.r(x0, y0, x1, y1);
        circle(t, t0, x + w / 2.0, y + h / 2.0, w / 2.0 + 14.0, h / 2.0 + 12.0, color)
    }

    /// Hand-drawn box around figure-pixel rect.
    pub fn frame<'a>(&self, t: f32, t0: f32, x0: f32, y0: f32, x1: f32, y1: f32, color: &str) -> Svgr<'a> {
        let (x, y, w, h) = self.r(x0, y0, x1, y1);
        boxed(t, t0, x - 8.0, y - 8.0, w + 16.0, h + 16.0, color, false)
    }

    /// Dims the card except figure-pixel rect between t0 and t1.
    pub fn spot<'a>(&self, t: f32, t0: f32, t1: f32, x0: f32, y0: f32, x1: f32, y1: f32) -> Svgr<'a> {
        let (x, y, w, h) = self.r(x0, y0, x1, y1);
        spotlight(t, t0, t1, self.outer(), (x - 6.0, y - 6.0, w + 12.0, h + 12.0), 0.62)
    }

    /// Camera key that frames the figure-pixel rect (x0,y0,x1,y1) to fill ~`fill` of the screen.
    pub fn zoom(&self, t: f32, x0: f32, y0: f32, x1: f32, y1: f32, fill: f32) -> Key {
        let (x, y, w, h) = self.r(x0, y0, x1, y1);
        let s = (1920.0 * fill / w).min(1080.0 * fill / h).max(1.0);
        Key(t, s, x + w / 2.0, y + h / 2.0)
    }
}

/// White margin around a card's image.
pub const PAD: f32 = 12.0;

// ================================================================ widgets

/// Outlined pill with handwritten label; outline draws on, text writes after 0.1 s.
/// Returns the pill; its width is `pill_w(label, size)`.
pub fn pill<'a>(t: f32, t0: f32, x: f32, y: f32, size: f32, color: &str, label: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let w = pill_w(label, size);
    let h = size * 1.7;
    let s = Stroke::rect(x, y, w, h, h / 2.0);
    let fill = tint(color, 0.1);
    fframes::svgr!(<g>
        <rect x={x} y={y} width={w} height={h} rx={h / 2.0} fill={fill} opacity={prog(t, t0, 0.25)} />
        {ink(&s, prog(t, t0, 0.3), color, 2.5)}
        {hand(t, t0 + 0.1, x + size * 0.75, y + h * 0.68, size, color, Al::Left, label)}
    </g>)
}

pub fn pill_w(label: &str, size: f32) -> f32 {
    text_w(label, size) + size * 1.5
}

/// A caveat / verdict stamp: thick box rotated by `deg` with 1-3 words, e.g. "the catch", "our guess".
pub fn stamp<'a>(t: f32, t0: f32, cx: f32, cy: f32, size: f32, color: &str, deg: f32, label: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let w = text_w(label, size) + size * 1.2;
    let h = size * 1.75;
    let (x, y) = (cx - w / 2.0, cy - h / 2.0);
    let s = Stroke::rect(x, y, w, h, 10.0);
    let s2 = Stroke::rect(x + 5.0, y + 5.0, w - 10.0, h - 10.0, 7.0);
    let fill = tint(color, 0.12);
    let body = fframes::svgr!(<g>
        <rect x={x} y={y} width={w} height={h} rx="10" fill={fill} opacity={prog(t, t0, 0.2)} />
        {ink(&s, prog(t, t0, 0.3), color, 3.5)}
        {ink(&s2, prog(t, t0 + 0.12, 0.3), color, 1.8)}
        {hand(t, t0 + 0.12, cx, y + h * 0.69, size, color, Al::Mid, label)}
    </g>);
    rotate_at(cx, cy, deg, body)
}

/// Numbered circle (roadmap stops, steps, quiz letters).
pub fn num<'a>(t: f32, t0: f32, cx: f32, cy: f32, r: f32, color: &str, n: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let fill = tint(color, 0.15);
    fframes::svgr!(<g>
        <circle cx={cx} cy={cy} r={r} fill={fill} opacity={prog(t, t0, 0.2)} />
        {circle(t, t0, cx, cy, r, r, color)}
        {pop(t, t0 + 0.15, cx, cy, txt(cx, cy + r * 0.38, r * 1.1, color, Al::Mid, n))}
    </g>)
}

/// A box with a centred handwritten label (the label writes as the outline finishes).
pub fn label_box<'a>(t: f32, t0: f32, x: f32, y: f32, w: f32, h: f32, color: &str, size: f32, label: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    fframes::svgr!(<g>
        {boxed(t, t0, x, y, w, h, color, true)}
        {hand(t, t0 + 0.15, x + w / 2.0, y + h / 2.0 + size * 0.32, size, WHITE, Al::Mid, label)}
    </g>)
}

/// The model: navy box with a blue outline and a white label.
pub fn model_box<'a>(t: f32, t0: f32, x: f32, y: f32, w: f32, h: f32, label: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let size = (h * 0.32).min(46.0);
    fframes::svgr!(<g>
        {boxed_fill(t, t0, x, y, w, h, BLUE, NAVY)}
        {hand(t, t0 + 0.15, x + w / 2.0, y + h / 2.0 + size * 0.32, size, WHITE, Al::Mid, label)}
    </g>)
}

/// Bar that grows up from `base` (y of the axis) to height `h` over 0.6 s; hatched fill.
pub fn bar<'a>(t: f32, t0: f32, x: f32, base: f32, w: f32, h: f32, color: &str) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let hh = (h * prog(t, t0, 0.6)).max(0.5);
    let y = base - hh;
    let fill = tint(color, 0.25);
    let color = color.to_string();
    let n = ((w + hh) / 16.0) as usize + 1;
    let hatch: Vec<Svgr<'a>> = (0..n)
        .map(|i| {
            let o = i as f32 * 16.0;
            fframes::svgr!(<line x1={x + o - hh} y1={base} x2={x + o} y2={y} stroke={color.clone()} stroke-width="2" opacity="0.45" />)
        })
        .collect();
    let id = format!("hb{}", seed_of(&[x, base, w]));
    let url = format!("url(#{id})");
    fframes::svgr!(<g>
        <rect x={x} y={y} width={w} height={hh} fill={fill} />
        <defs><clipPath id={id}><rect x={x} y={y} width={w} height={hh} /></clipPath></defs>
        <g clip-path={url}>{hatch}</g>
        <rect x={x} y={y} width={w} height={hh} fill="none" stroke={color} stroke-width="3" />
    </g>)
}

/// Quiz countdown: a ring that empties over `dur` seconds from `t0`, with seconds left inside.
pub fn countdown<'a>(t: f32, t0: f32, dur: f32, cx: f32, cy: f32, r: f32) -> Svgr<'a> {
    if t < t0 || t > t0 + dur + 0.3 {
        return Svgr::empty();
    }
    let p = lin(t, t0, dur);
    let ring = Stroke::ellipse(cx, cy, r, r);
    let left = (dur - (t - t0)).ceil().max(0.0) as i32;
    let o = 1.0 - ease_in((t - t0 - dur) / 0.3);
    opacity(o, fframes::svgr!(<g>
        {ink(&ring, 1.0, &tint(YELLOW, 0.25), 6.0)}
        {ink(&ring, 1.0 - p, YELLOW, 6.0)}
        {txt(cx, cy + r * 0.32, r * 0.95, YELLOW, Al::Mid, &left.to_string())}
    </g>))
}

// ================================================================ icons
// Small outline icons drawn on in ~0.3 s. (cx, cy) is the centre, `s` the size (px).

pub fn icon_draw<'a>(t: f32, t0: f32, strokes: Vec<Stroke>, color: &str, w: f32) -> Svgr<'a> {
    if t < t0 {
        return Svgr::empty();
    }
    let n = strokes.len().max(1) as f32;
    let v: Vec<Svgr<'a>> = strokes
        .iter()
        .enumerate()
        .map(|(i, s)| ink(s, prog(t, t0 + 0.3 * i as f32 / n, 0.3 / n + 0.12), color, w))
        .collect();
    v.into()
}

/// Document: page with a folded corner and text lines.
pub fn icon_doc<'a>(t: f32, t0: f32, cx: f32, cy: f32, s: f32, color: &str) -> Svgr<'a> {
    let (w, h) = (s * 0.72, s);
    let (x, y) = (cx - w / 2.0, cy - h / 2.0);
    let f = s * 0.22;
    let mut v = vec![Stroke::poly(&[(x + w - f, y), (x, y), (x, y + h), (x + w, y + h), (x + w, y + f), (x + w - f, y), (x + w - f, y + f), (x + w, y + f)])];
    for i in 0..4 {
        let ly = y + h * (0.38 + 0.15 * i as f32);
        let lw = if i == 3 { w * 0.4 } else { w * 0.62 };
        v.push(Stroke::line(x + w * 0.18, ly, x + w * 0.18 + lw, ly));
    }
    icon_draw(t, t0, v, color, (s * 0.04).max(2.5))
}

/// Terminal window with a prompt `>_`.
pub fn icon_term<'a>(t: f32, t0: f32, cx: f32, cy: f32, s: f32, color: &str) -> Svgr<'a> {
    let (w, h) = (s * 1.2, s * 0.9);
    let (x, y) = (cx - w / 2.0, cy - h / 2.0);
    let v = vec![
        Stroke::rect(x, y, w, h, s * 0.08),
        Stroke::line(x, y + h * 0.2, x + w, y + h * 0.2),
        Stroke::poly(&[(x + w * 0.16, y + h * 0.42), (x + w * 0.32, y + h * 0.56), (x + w * 0.16, y + h * 0.7)]),
        Stroke::line(x + w * 0.4, y + h * 0.72, x + w * 0.62, y + h * 0.72),
    ];
    icon_draw(t, t0, v, color, (s * 0.045).max(2.5))
}

/// Wrench and screwdriver crossed (the toolbox).
pub fn icon_tools<'a>(t: f32, t0: f32, cx: f32, cy: f32, s: f32, color: &str) -> Svgr<'a> {
    let k = s / 2.0;
    let v = vec![
        Stroke::line(cx - k * 0.75, cy + k * 0.75, cx + k * 0.45, cy - k * 0.45),
        Stroke::ellipse(cx + k * 0.6, cy - k * 0.6, k * 0.28, k * 0.28),
        Stroke::line(cx + k * 0.75, cy + k * 0.75, cx - k * 0.35, cy - k * 0.35),
        Stroke::poly(&[(cx - k * 0.35, cy - k * 0.35), (cx - k * 0.8, cy - k * 0.6), (cx - k * 0.6, cy - k * 0.8), (cx - k * 0.35, cy - k * 0.35)]),
    ];
    icon_draw(t, t0, v, color, (s * 0.06).max(3.0))
}

/// Magnifying glass.
pub fn icon_search<'a>(t: f32, t0: f32, cx: f32, cy: f32, s: f32, color: &str) -> Svgr<'a> {
    let r = s * 0.3;
    let v = vec![
        Stroke::ellipse(cx - s * 0.1, cy - s * 0.1, r, r),
        Stroke::line(cx + s * 0.12, cy + s * 0.12, cx + s * 0.42, cy + s * 0.42),
    ];
    icon_draw(t, t0, v, color, (s * 0.06).max(3.0))
}

/// Checklist (clipboard with three ticked rows) = a plan.
pub fn icon_plan<'a>(t: f32, t0: f32, cx: f32, cy: f32, s: f32, color: &str) -> Svgr<'a> {
    let (w, h) = (s * 0.8, s);
    let (x, y) = (cx - w / 2.0, cy - h / 2.0);
    let mut v = vec![Stroke::rect(x, y, w, h, s * 0.07), Stroke::rect(x + w * 0.3, y - s * 0.06, w * 0.4, s * 0.12, 3.0)];
    for i in 0..3 {
        let ly = y + h * (0.32 + 0.22 * i as f32);
        v.push(Stroke::rect(x + w * 0.14, ly - s * 0.06, s * 0.12, s * 0.12, 2.0));
        v.push(Stroke::line(x + w * 0.42, ly, x + w * 0.84, ly));
    }
    icon_draw(t, t0, v, color, (s * 0.035).max(2.5))
}

/// Coin with a dollar sign (cost).
pub fn icon_coin<'a>(t: f32, t0: f32, cx: f32, cy: f32, s: f32, color: &str) -> Svgr<'a> {
    let r = s * 0.45;
    let mut v = vec![Stroke::ellipse(cx, cy, r, r)];
    v.push(Stroke::poly(&[
        (cx + r * 0.32, cy - r * 0.38),
        (cx - r * 0.1, cy - r * 0.46),
        (cx - r * 0.34, cy - r * 0.26),
        (cx - r * 0.2, cy - r * 0.04),
        (cx + r * 0.22, cy + r * 0.06),
        (cx + r * 0.34, cy + r * 0.28),
        (cx + r * 0.1, cy + r * 0.46),
        (cx - r * 0.34, cy + r * 0.38),
    ]));
    v.push(Stroke::line(cx, cy - r * 0.66, cx, cy + r * 0.66));
    icon_draw(t, t0, v, color, (s * 0.05).max(2.5))
}

/// A drawer / cabinet (storage, an archive).
pub fn icon_drawer<'a>(t: f32, t0: f32, cx: f32, cy: f32, s: f32, color: &str) -> Svgr<'a> {
    let (w, h) = (s * 1.1, s * 0.9);
    let (x, y) = (cx - w / 2.0, cy - h / 2.0);
    let v = vec![
        Stroke::rect(x, y, w, h, 6.0),
        Stroke::line(x, y + h / 2.0, x + w, y + h / 2.0),
        Stroke::line(cx - w * 0.14, y + h * 0.25, cx + w * 0.14, y + h * 0.25),
        Stroke::line(cx - w * 0.14, y + h * 0.75, cx + w * 0.14, y + h * 0.75),
    ];
    icon_draw(t, t0, v, color, (s * 0.04).max(2.5))
}

/// Database cylinder.
pub fn icon_db<'a>(t: f32, t0: f32, cx: f32, cy: f32, s: f32, color: &str) -> Svgr<'a> {
    let (rx, ry, h) = (s * 0.42, s * 0.12, s * 0.75);
    let top = cy - h / 2.0;
    let v = vec![
        Stroke::ellipse(cx, top, rx, ry),
        Stroke::line(cx - rx, top, cx - rx, top + h),
        Stroke::curve(cx - rx, top + h, cx, top + h + ry * 2.0, cx + rx, top + h),
        Stroke::line(cx + rx, top + h, cx + rx, top),
    ];
    icon_draw(t, t0, v, color, (s * 0.04).max(2.5))
}

/// Gear-free "brain-ish" model icon: a rounded box with two eyes (a model, an agent).
pub fn icon_bot<'a>(t: f32, t0: f32, cx: f32, cy: f32, s: f32, color: &str) -> Svgr<'a> {
    let (w, h) = (s, s * 0.75);
    let (x, y) = (cx - w / 2.0, cy - h / 2.0 + s * 0.08);
    let v = vec![
        Stroke::rect(x, y, w, h, s * 0.18),
        Stroke::line(cx, y, cx, y - s * 0.16),
        Stroke::ellipse(cx, y - s * 0.21, s * 0.06, s * 0.06),
        Stroke::ellipse(cx - w * 0.2, cy + s * 0.05, s * 0.07, s * 0.08),
        Stroke::ellipse(cx + w * 0.2, cy + s * 0.05, s * 0.07, s * 0.08),
    ];
    icon_draw(t, t0, v, color, (s * 0.045).max(2.5))
}

/// Hourglass/clock face (turns, time).
pub fn icon_clock<'a>(t: f32, t0: f32, cx: f32, cy: f32, s: f32, color: &str) -> Svgr<'a> {
    let r = s * 0.45;
    let v = vec![
        Stroke::ellipse(cx, cy, r, r),
        Stroke::poly(&[(cx, cy - r * 0.6), (cx, cy), (cx + r * 0.45, cy + r * 0.25)]),
    ];
    icon_draw(t, t0, v, color, (s * 0.05).max(2.5))
}
