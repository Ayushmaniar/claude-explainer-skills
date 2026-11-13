//! The host: a Codex pet sprite (192x208 cells, from github.com/gennadi-kuzmin/awesome-codex-pets) that
//! `tools/get_pet.py <pet-id>` cuts into `sprites/pet_<row>_<col>.png` with a light sticker outline. The
//! template ships no pet: with an empty `sprites/` folder no host is drawn and everything else renders.
//! (Each row of a Codex pet sheet is one animation; `Anim` lists them in order.)
//!
//! Most of the time the host sits small in the bottom-right corner and bobs along with the voice
//! (`envelope.rs`). In the cold open, the quizzes and the ending it stands bigger in the frame. Scenes
//! that draw the host themselves (e.g. walking along a roadmap with `sprite(..)`) return `None` from
//! `spot`, so the corner host doesn't double.
use fframes::Svgr;

use crate::cues::SCENES;
use crate::envelope::ENVELOPE;
use crate::kit::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Anim {
    Idle,
    RunRight,
    RunLeft,
    Wave,
    Jump,
    Failed,
    Waiting,
    Run,
    Review,
}

impl Anim {
    fn row(self) -> usize {
        self as usize
    }
    fn frames(self) -> usize {
        match self {
            Anim::Idle | Anim::Waiting | Anim::Run | Anim::Review => 6,
            Anim::RunRight | Anim::RunLeft | Anim::Failed => 8,
            Anim::Wave => 4,
            Anim::Jump => 5,
        }
    }
    fn fps(self) -> f32 {
        match self {
            Anim::Idle | Anim::Waiting | Anim::Review => 6.0,
            Anim::RunRight | Anim::RunLeft | Anim::Run => 12.0,
            Anim::Wave => 7.0,
            Anim::Jump => 10.0,
            Anim::Failed => 6.0,
        }
    }
    /// Plays once and holds the last frame (instead of looping).
    fn once(self) -> bool {
        matches!(self, Anim::Failed)
    }
}

const ALL: [Anim; 9] = [
    Anim::Idle,
    Anim::RunRight,
    Anim::RunLeft,
    Anim::Wave,
    Anim::Jump,
    Anim::Failed,
    Anim::Waiting,
    Anim::Run,
    Anim::Review,
];

/// File names (without .png) of every host frame.
pub fn sprite_names() -> Vec<String> {
    ALL.iter().flat_map(|a| (0..a.frames()).map(move |c| format!("pet_{}_{}", a.row(), c))).collect()
}

/// Voice loudness 0..1 at video time `g`.
pub fn voice(g: f32) -> f32 {
    let i = (g * 30.0).round().max(0.0) as usize;
    let get = |j: usize| ENVELOPE.get(j).copied().unwrap_or(0) as f32 / 255.0;
    get(i) * 0.6 + get(i.saturating_sub(1)) * 0.4
}

/// The host with its feet at (x, y), playing `anim` for `age` seconds. At scale 1 it is 208 px
/// tall. `talk` (0..1) squashes it a little, so it seems to speak.
pub fn sprite<'a>(x: f32, y: f32, scale: f32, anim: Anim, age: f32, talk: f32) -> Svgr<'a> {
    let n = anim.frames();
    let k = (age.max(0.0) * anim.fps()) as usize;
    let col = if anim.once() { k.min(n - 1) } else { k % n };
    let Some(img) = crate::image(&format!("pet_{}_{}", anim.row(), col)) else { return Svgr::empty() };
    let (w, h) = (192.0 * scale, 208.0 * scale);
    let sy = 1.0 + 0.045 * talk;
    let sx = 1.0 - 0.02 * talk;
    fframes::svgr!(<g>
        <ellipse cx={x} cy={y + 2.0} rx={52.0 * scale} ry={7.0 * scale} fill="#000000" opacity="0.5" />
        <g transform={format!("translate({x:.2} {y:.2}) scale({sx:.4} {sy:.4})")}>
            <image href={img} x={-w / 2.0} y={-h * 0.96} width={w} height={h} />
        </g>
    </g>)
}

/// Where the host is and what it does in a scene. `t` is scene time.
struct Spot {
    x: f32,
    y: f32,
    scale: f32,
    anim: Anim,
    since: f32,
}

const HOME: (f32, f32, f32) = (1838.0, 1068.0, 0.52);

fn spot(name: &str, t: f32, c: &crate::cues::SceneCues) -> Option<Spot> {
    let at = |x: f32, y: f32, scale: f32, anim: Anim, since: f32| Some(Spot { x, y, scale, anim, since });
    match name {
        // A scene that draws the host itself returns None here, e.g.:
        // "Roadmap" => None,
        "Hook" => {
            // Runs in from the right, then stands and listens; jumps when the method is named.
            let arrive = 1.6;
            if t < arrive {
                let p = ease_out(t / arrive);
                return at(lerp(2100.0, 1700.0, p), 1010.0, 1.0, Anim::RunLeft, 0.0);
            }
            let named = c.w(4, "binary");
            if t >= named - 0.2 && t < named + 0.5 {
                return at(1700.0, 1010.0, 1.0, Anim::Jump, named - 0.2);
            }
            at(1700.0, 1010.0, 1.0, Anim::Idle, arrive)
        }
        n if n.starts_with("Quiz") => {
            // The answer starts on the beat after the pause (the empty beat).
            let pause = c.words.iter().position(|w| w.is_empty()).unwrap_or(6);
            let reveal = c.b(pause + 1);
            if t >= reveal && t < reveal + 0.5 {
                return at(1700.0, 1010.0, 0.95, Anim::Jump, reveal);
            }
            if t >= c.b(pause) && t < reveal {
                return at(1700.0, 1010.0, 0.95, Anim::Waiting, c.b(pause));
            }
            at(1700.0, 1010.0, 0.95, Anim::Idle, 0.0)
        }
        "Outro" => {
            let wave = c.w(1, "thanks");
            if t >= wave - 0.2 {
                return at(1690.0, 1010.0, 1.0, Anim::Wave, wave - 0.2);
            }
            at(1690.0, 1010.0, 1.0, Anim::Idle, 0.0)
        }
        _ => at(HOME.0, HOME.1, HOME.2, Anim::Idle, 0.0),
    }
}

/// The host at video time `g`, drawn over everything (nothing if `sprites/` is empty).
pub fn host<'a>(g: f32) -> Svgr<'a> {
    let Some(c) = SCENES.iter().rev().find(|s| s.start <= g + 1e-4) else { return Svgr::empty() };
    let t = g - c.start;
    let Some(s) = spot(c.name, t, c) else { return Svgr::empty() };
    // Talking bob only while standing (not while running/jumping).
    let talk = if matches!(s.anim, Anim::Idle | Anim::Review | Anim::Waiting) { voice(g) } else { 0.0 };
    sprite(s.x, s.y, s.scale, s.anim, t - s.since, talk)
}
