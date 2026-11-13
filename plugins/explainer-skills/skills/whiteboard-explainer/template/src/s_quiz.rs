//! Scene: Quiz1. Hard multiple choice on the shared `mcq` board (objs.rs). Keys and a justification for
//! every option belong in research/quiz_keys.md; for this demo:
//!   worst-case guesses for n entries = floor(log2 n) + 1: 1,000,000 -> 20, 2,000,000 -> 21 (2^21 = 2,097,152).
//!   A (40) is linear growth, the trap; B (20) fails since 2^20 = 1,048,576 < 2,000,000; D (23) is the step
//!   for a list ten times longer, not twice as long.
use fframes::Svgr;

use crate::cues::SceneCues;
#[allow(unused_imports)]
use crate::kit::*;
#[allow(unused_imports)]
use crate::objs::*;

/// Doubling the list adds one halving: 21 guesses. Answer C.
pub fn quiz1<'a>(t: f32, c: &SceneCues) -> Svgr<'a> {
    mcq(
        t,
        c,
        &["the sorted list doubles: 1M → 2M names.", "at most, how many guesses now?"],
        ["40: twice as many", "still 20: nothing changes", "21: one more", "23: three more"],
        2,
        0,
        "that's one by one",
        &[
            (c.w(8, "doubling") - LEAD, "2M = 2 × 1M", WHITE),
            (c.w(8, "halving") - LEAD, "one more halving", YELLOW),
            (c.w(8, "enough") - LEAD, "2^21 ≈ 2.1M > 2M", YELLOW),
            (c.w(9, "tempting") - LEAD, "40 = checking one by one", GREY),
        ],
    )
}
