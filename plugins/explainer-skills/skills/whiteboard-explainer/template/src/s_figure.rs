//! Scene: Figure (chapter 2). A "paper figure" on a white card (figs/guesses.png, drawn by
//! tools/make_demo_fig.py), annotated in its own pixel coordinates: a ring and a highlighter band on the
//! axes, rings on the bars that are named, small notes on the card, a camera push-in on the plot, a
//! spotlight on the last bar, and the camera back out for the summary in the right-hand column.
use fframes::Svgr;

use crate::cues::SceneCues;
#[allow(unused_imports)]
use crate::kit::*;
#[allow(unused_imports)]
use crate::objs::*;

/// Bars of figs/guesses.png in figure pixels (x0, y0, x1, y1), as printed by tools/make_demo_fig.py.
const BARS: [(f32, f32, f32, f32); 6] = [
    (247.0, 724.0, 377.0, 840.0),   // 10 entries: 4 guesses
    (470.0, 636.0, 600.0, 840.0),   // 100: 7
    (693.0, 549.0, 823.0, 840.0),   // 1k: 10
    (917.0, 433.0, 1047.0, 840.0),  // 10k: 14
    (1140.0, 345.0, 1270.0, 840.0), // 100k: 17
    (1363.0, 258.0, 1493.0, 840.0), // 1M: 20
];
/// Guesses added from each bar to the next (computed: 4 7 10 14 17 20).
const STEPS: [&str; 5] = ["+3", "+3", "+4", "+3", "+3"];

pub fn figure<'a>(t: f32, c: &SceneCues) -> Svgr<'a> {
    let card = Card::fit("guesses", 140.0, 175.0, 1200.0, 730.0);
    let mut v: Vec<Svgr<'a>> = Vec::new();

    // The card is on the board from the scene's start (a set-up diagram comes in before the sentence that
    // uses it), with its provenance caption.
    v.push(card.show(t, 0.0));
    v.push(card.caption(t, 0.5, "demo figure · drawn for this template"));

    // b0: what the axes mean
    v.push(card.ring(t, c.w(0, "guesses") - 0.05, 40.0, 300.0, 95.0, 740.0, P_RED));
    v.push(stamp(t, c.w(0, "most") - 0.05, 1560.0, 270.0, 40.0, ORANGE, -2.0, "worst case"));
    v.push(card.mark(t, c.w(0, "size") - 0.1, 230.0, 850.0, 1510.0, 895.0));

    // b1: two bars, named by size
    let ring_bar = |t0: f32, i: usize, col: &'static str| {
        let (x0, y0, x1, y1) = BARS[i];
        card.ring(t, t0, x0, y0 - 60.0, x1, y1, col)
    };
    v.push(ring_bar(c.w(1, "four") - 0.1, 0, P_BLUE));
    v.push(hand(t, c.w(1, "four") - LEAD, 1400.0, 440.0, 60.0, YELLOW, Al::Left, "10 → 4"));
    let t_k = c.wn(1, "ten", 1) - 0.1; // "Ten names ... take ten": the second "ten"
    v.push(ring_bar(t_k, 2, P_BLUE));
    v.push(hand(t, t_k, 1400.0, 540.0, 60.0, YELLOW, Al::Left, "1k → 10"));

    // b2: each step up adds three or four: notes between the bars (they ride along with the zoom)
    let t_add = c.w(2, "add") - 0.1;
    for i in 0..5 {
        let mx = (BARS[i].0 + BARS[i].2) / 2.0 + 0.5 * (BARS[i + 1].0 - BARS[i].0);
        let (nx, ny) = card.at(mx, BARS[i + 1].1 - 24.0);
        let col = if STEPS[i] == "+4" { ORANGE } else { MINT };
        v.push(note(t, t_add + 0.15 * i as f32, nx, ny, 30.0, col, Al::Mid, STEPS[i]));
    }

    // b3: back out; spotlight on the million bar, its label highlighted, the summary on the right
    let t_mil = c.w(3, "million") - 0.1;
    let (x0, y0, x1, y1) = BARS[5];
    v.push(card.spot(t, t_mil, c.dur, x0 - 14.0, y0 - 70.0, x1 + 14.0, y1));
    v.push(ring_bar(t_mil + 0.2, 5, P_GREEN));
    let t20 = c.w(3, "twenty") - 0.1;
    v.push(card.mark(t, t20, x0 + 20.0, y0 - 52.0, x1 - 20.0, y0 - 6.0));
    v.push(hand(t, t20, 1400.0, 660.0, 72.0, MINT, Al::Left, "1M → 20"));
    v.push(underline(t, t20 + 0.6, 1400.0, 682.0, text_w("1M → 20", 72.0), MINT));

    // Camera: push in on the plot while the growth is described (the right-hand column leaves the view),
    // back out as the last line starts.
    let keys = [
        Key(0.0, 1.0, 960.0, 540.0),
        card.zoom(c.w(2, "ten") - 0.3, 130.0, 230.0, 1540.0, 900.0, 0.9),
        Key(c.b(3) - 0.4, 1.0, 960.0, 540.0),
    ];
    camera(t, &keys, v.into())
}
