//! A hand-drawn whiteboard explainer: everything is drawn, written, circled and underlined as the narrator
//! says it. The demo in this template explains binary search in about ninety seconds.
//!
//! Timing comes from the narration: `narration/build_vo.py` writes `src/cues.rs` with every scene's start
//! and duration, the start/end of each narration line ("beat"), and the start/end of every spoken word.
//! Scenes ask for `c.w(beat, "word")` to start a drawing on that word.
//!
//! Layout of the source:
//! - `kit.rs`     drawing kit: strokes, handwriting, camera, paper cards, widgets, icons
//! - `objs.rs`    recurring objects (the quiz board, cells, code/chat/quote cards, plots) + this video's own
//! - `t3d.rs`     optional 3D blocks (fframes 1.3 three_d): cubes, camera insets, projected labels
//! - `duck.rs`    the host (a Codex pet sprite, if `sprites/` has one) and its schedule
//! - `s_*.rs`     the scenes, one function per scene: `fn(t, c) -> Svgr`
//! - generated:   `cues.rs` (build_vo.py / est_cues.py), `envelope.rs` (envelope.py), `figs.rs`
//!                (tools/prep_figs.py), `metrics.rs` (tools/font_metrics.py)
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use fframes::{
    AudioMap, AudioTimestamp::*, AudioTrack, Color, Duration, FFramesContext, Frame, Scene, Scenes, Svgr, Video,
    include_media_dir,
};

pub mod cues;
pub mod duck;
pub mod envelope;
pub mod figs;
pub mod kit;
pub mod metrics;
pub mod objs;
#[allow(dead_code)]
pub mod t3d;
mod s_figure;
mod s_idea;
mod s_intro;
mod s_quiz;

use cues::{SCENES, SceneCues, W};
use kit::*;

include_media_dir!(pub struct ExplainerMedia, "media");

pub const WIDTH: usize = 1920;
pub const HEIGHT: usize = 1080;

// ================================================================ cue lookups

/// Lowercase letters and digits only, the same key `build_vo.py` writes for each word.
fn key(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_alphanumeric()).flat_map(|c| c.to_lowercase()).collect()
}

impl SceneCues {
    /// Start of beat `i` (scene time). Past the end: the scene's duration.
    pub fn b(&self, i: usize) -> f32 {
        self.beats.get(i).copied().unwrap_or(self.dur)
    }
    /// End of beat `i` (scene time): when its last word ends.
    pub fn e(&self, i: usize) -> f32 {
        self.ends.get(i).copied().unwrap_or(self.dur)
    }
    fn find(&self, i: usize, word: &str, nth: usize) -> Option<&W> {
        let k = key(word);
        self.words.get(i)?.iter().filter(|w| w.w == k).nth(nth)
    }
    /// When `word` (first occurrence) starts in beat `i`. Falls back to the beat's start.
    /// Words are matched on lowercase letters+digits: "T zero," -> "t", "zero"; "fifty-two" ->
    /// "fiftytwo". See narration/WORDS.md (written by narration/words.py).
    pub fn w(&self, i: usize, word: &str) -> f32 {
        self.find(i, word, 0).map(|w| w.t).unwrap_or_else(|| self.b(i))
    }
    /// When the `nth` (0-based) occurrence of `word` starts in beat `i`.
    pub fn wn(&self, i: usize, word: &str, nth: usize) -> f32 {
        self.find(i, word, nth).map(|w| w.t).unwrap_or_else(|| self.b(i))
    }
    /// When `word` (first occurrence) ends in beat `i`.
    pub fn we(&self, i: usize, word: &str) -> f32 {
        self.find(i, word, 0).map(|w| w.e).unwrap_or_else(|| self.e(i))
    }
}

/// The cues of a scene by name (for scenes that need another scene's timing).
pub fn scene(name: &str) -> Option<&'static SceneCues> {
    SCENES.iter().find(|s| s.name == name)
}

// ================================================================ images

/// Every image (figures and sprites), loaded once from the media provider.
static IMAGES: OnceLock<HashMap<String, Arc<fframes::usvgr::PreloadedImageData>>> = OnceLock::new();

fn init_images(ctx: &FFramesContext<'_, '_>) {
    IMAGES.get_or_init(|| {
        let mut m = HashMap::new();
        // Only ask for files that exist, so an empty `sprites/` (no host) or a missing figure is not an error.
        let figs = figs::FIGS.iter().map(|f| ("figs", f.0.to_string()));
        let sprites = duck::sprite_names().into_iter().map(|n| ("sprites", n));
        for (dir, n) in figs.chain(sprites) {
            let file = format!("{n}.png");
            if !std::path::Path::new(dir).join(&file).is_file() {
                continue;
            }
            if let Some(img) = ctx.get_image(file) {
                m.insert(n, img.href());
            }
        }
        m
    });
}

/// An image by name (without ".png"): a figure from `figs/` or a sprite from `sprites/`. None if missing.
pub fn image(name: &str) -> Option<Arc<fframes::usvgr::PreloadedImageData>> {
    IMAGES.get()?.get(name).cloned()
}

// ================================================================ chapters

/// The chip shown top-left for each scene. Empty = no chip. A change of chip = new chapter,
/// which starts with a 0.1 s cut to black and the new chip fading in.
pub fn chapter(name: &str) -> &'static str {
    match name {
        "Hook" | "Outro" => "",
        "Idea" => "1 · the idea",
        "Figure" => "2 · bigger lists",
        "Quiz1" => "Quiz 1 / 1",
        _ => "",
    }
}

/// True if scene `idx` starts a new chapter (its chip differs from the previous scene's).
fn starts_chapter(idx: usize) -> bool {
    idx > 0 && chapter(SCENES[idx].name) != chapter(SCENES[idx - 1].name)
}

/// Length of the black dip at a chapter start.
pub const DIP: f32 = 0.1;

fn chip<'a>(g: f32) -> Svgr<'a> {
    let Some(idx) = SCENES.iter().rposition(|s| s.start <= g + 1e-4) else { return Svgr::empty() };
    let label = chapter(SCENES[idx].name);
    if label.is_empty() {
        return Svgr::empty();
    }
    // First scene of this chapter.
    let mut first = idx;
    while first > 0 && chapter(SCENES[first - 1].name) == label {
        first -= 1;
    }
    let t0 = SCENES[first].start + DIP;
    if g < t0 {
        return Svgr::empty();
    }
    let o = 1.0 - (1.0 - clamp01((g - t0) / 0.17)).powi(2);
    let size = 38.0;
    let (x, y, h) = (100.0, 52.0, 74.0);
    let w = text_w(label, size) + 52.0;
    opacity(o, fframes::svgr!(<g>
        <rect x={x} y={y} width={w} height={h} rx={h / 2.0} fill="#141205" />
        {ink(&Stroke::rect(x, y, w, h, h / 2.0), 1.0, YELLOW, 2.5)}
        {txt(x + 26.0, y + 49.0, size, YELLOW, Al::Left, label)}
    </g>))
}

// ================================================================ scenes

#[derive(Debug)]
pub struct Shot {
    idx: usize,
}

impl Scene for Shot {
    fn duration(&self) -> Duration<'_> {
        Duration::Frames((SCENES[self.idx].dur * 30.0).round() as usize)
    }

    fn name(&self) -> &'static str {
        SCENES[self.idx].name
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        init_images(ctx);
        let c = &SCENES[self.idx];
        let t = frame.seconds();
        if starts_chapter(self.idx) && t < DIP {
            return Svgr::empty();
        }
        // One arm per scene of narration/script.py (narration/words.py checks that none is missing).
        match c.name {
            "Hook" => s_intro::hook(t, c),
            "Idea" => s_idea::idea(t, c),
            "Figure" => s_figure::figure(t, c),
            "Quiz1" => s_quiz::quiz1(t, c),
            "Outro" => s_intro::outro(t, c),
            _ => placeholder(t, c),
        }
    }
}

/// Stand-in for scenes that are not drawn yet: the scene name and its narration, line by line.
pub fn placeholder<'a>(t: f32, c: &SceneCues) -> Svgr<'a> {
    let mut v = vec![hand(t, 0.3, 160.0, 260.0, 64.0, YELLOW, Al::Left, &format!("[{}]", c.name))];
    for i in 0..c.beats.len() {
        let words: Vec<&str> = c.words.get(i).map(|ws| ws.iter().map(|w| w.w).collect()).unwrap_or_default();
        let line: String = words.join(" ").chars().take(70).collect();
        v.push(hand(t, c.b(i), 160.0, 360.0 + i as f32 * 64.0, 34.0, WHITE, Al::Left, &line));
    }
    v.into()
}

// ================================================================ the video

pub struct ExplainerVideo {
    shots: Vec<Shot>,
}

impl ExplainerVideo {
    pub fn new() -> Self {
        Self { shots: (0..SCENES.len()).map(|idx| Shot { idx }).collect() }
    }
}

impl Default for ExplainerVideo {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ExplainerVideo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExplainerVideo").finish()
    }
}

impl Video for ExplainerVideo {
    const FPS: usize = 30;
    const WIDTH: usize = WIDTH;
    const HEIGHT: usize = HEIGHT;
    const BACKGROUND_COLOR: Color = Color::BLACK;

    fn duration(&self) -> Duration<'_> {
        Duration::Auto
    }

    fn audio(&self) -> AudioMap<'_> {
        AudioMap::from([AudioTrack::new("narration.wav", Second(0.)..Eof).voice()])
    }

    fn define_scenes(&self) -> Scenes<'_> {
        Scenes::from(self.shots.iter().map(|s| s as &dyn Scene).collect::<Vec<_>>())
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        init_images(ctx);
        let g = frame.seconds();
        fframes::svgr!(
            <svg xmlns="http://www.w3.org/2000/svg" width={WIDTH} height={HEIGHT} viewBox="0 0 1920 1080">
                <defs>
                    <pattern id="dots" x="0" y="0" width="36" height="36" patternUnits="userSpaceOnUse">
                        <rect x="16" y="16" width="4" height="4" fill="#0e0e0e" />
                    </pattern>
                </defs>
                <rect width="1920" height="1080" fill={BG} />
                <rect width="1920" height="1080" fill="url(#dots)" />
                {ctx.render_scenes(&frame)}
                {chip(g)}
                {duck::host(g)}
            </svg>
        )
    }
}
