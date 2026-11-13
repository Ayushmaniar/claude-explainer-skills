//! Optional 3D blocks (fframes 1.3 `three_d`): things drawn as chunky cubes (a tensor, one cube per number;
//! a stack of layers; a memory hierarchy). Not used by the demo scenes; delete this module (and the `3d`/`wgpu`
//! features in Cargo.toml) if a video has no 3D shots. See the skill's references/three_d_example.rs too.
//!
//! Immediate mode: a scene function builds a list of `Cube`s for the current frame (positions are
//! plain arithmetic of `t`, so blocks can collapse, stretch, split or unroll), and `View3::render`
//! draws them into an inset of the frame. Labels, arrows and numbers stay hand-drawn SVG on top:
//! `View3::px` projects a 3D point to frame pixels.
//!
//! Axes convention for every block in the video (right-handed, Y up, camera looks at the front):
//! - x (to the right)  = the LAST axis (features, columns, k / j in matmul)
//! - y (downwards: row r sits at y = -r) = the second-to-last axis (rows, sequence)
//! - z (into the screen: item b sits at z = -b) = a leading axis (batch, groups)
//! `grid_pos(b, r, c)` gives the cube centre for index (b, r, c) with unit spacing.
//!
//! The 3D renders on the GPU through wgpu and is read back into the CPU frame renderer. With
//! `FF_3D=cpu` (e.g. if another app is using the GPU) it renders on the CPU instead.
use std::sync::{Arc, OnceLock};

use fframes::Svgr;
use fframes::three_d::*;

pub use fframes::three_d::{Vec3, vec3};

/// Colours of the 3D cubes as hex numbers, matching kit.rs (`TEAL` = 0x1cd2ac ...).
pub const C_CYAN: u32 = 0x59e8f9;
pub const C_PURPLE: u32 = 0xa08cff;
pub const C_TEAL: u32 = 0x1cd2ac;
pub const C_BLUE: u32 = 0x5499fa;
pub const C_YELLOW: u32 = 0xfcdb43;
pub const C_ORANGE: u32 = 0xfe8733;
pub const C_PINK: u32 = 0xec70a8;
pub const C_MINT: u32 = 0x7eeeb7;
pub const C_RED: u32 = 0xf06070;
pub const C_GREY: u32 = 0x9a9a9a;
pub const C_WHITE: u32 = 0xe8e8e4;

/// One number of a tensor, drawn as a rounded cube.
#[derive(Clone, Debug)]
pub struct Cube {
    /// Centre, in world units (cube spacing 1).
    pub p: Vec3,
    /// Edge length (0.86 leaves a gap between neighbours; scale it for pop-in / squash).
    pub s: Vec3,
    pub color: u32,
    /// 0..1. Below 1 the cube is drawn transparent (ghost copies, faded context).
    pub alpha: f32,
    /// Text written on the cube's front face (a number), drawn in SVG by `View3::render`.
    pub label: Option<String>,
}

impl Cube {
    pub fn new(p: Vec3, color: u32) -> Self {
        Self { p, s: Vec3::splat(0.86), color, alpha: 1.0, label: None }
    }
    pub fn label(mut self, s: impl Into<String>) -> Self {
        self.label = Some(s.into());
        self
    }
    pub fn alpha(mut self, a: f32) -> Self {
        self.alpha = a;
        self
    }
    /// Uniform scale of the cube (1 = normal size 0.86).
    pub fn scale(mut self, k: f32) -> Self {
        self.s *= k;
        self
    }
    /// Separate scale per axis (squash a cube flat along one axis).
    pub fn scale3(mut self, kx: f32, ky: f32, kz: f32) -> Self {
        self.s = vec3(self.s.x * kx, self.s.y * ky, self.s.z * kz);
        self
    }
}

/// Centre of index (b, r, c) in a block: c to the right, r downwards, b into the screen.
pub fn grid_pos(b: f32, r: f32, c: f32) -> Vec3 {
    vec3(c, -r, -b)
}

/// Centre of a block of shape (nb, nr, nc) built with `grid_pos` (to aim the camera at).
pub fn grid_center(nb: f32, nr: f32, nc: f32) -> Vec3 {
    vec3((nc - 1.0) / 2.0, -(nr - 1.0) / 2.0, -(nb - 1.0) / 2.0)
}

fn unit_box() -> Arc<Geometry> {
    static G: OnceLock<Arc<Geometry>> = OnceLock::new();
    G.get_or_init(|| Arc::new(Geometry::rounded_box(1.0, 1.0, 1.0, 0.09, 3))).clone()
}

fn cpu() -> bool {
    static CPU: OnceLock<bool> = OnceLock::new();
    *CPU.get_or_init(|| std::env::var("FF_3D").is_ok_and(|v| v == "cpu"))
}

/// A 3D inset: where it sits in the frame (x, y, w, h in frame pixels) and the camera.
#[derive(Clone, Debug)]
pub struct View3 {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub cam: Camera,
}

impl View3 {
    /// An inset with a perspective camera orbiting `target` at `dist`, `azimuth` and `elevation`
    /// in degrees (azimuth 0 = straight at the front face; try 25-35 and 18-25 for a 3/4 view).
    pub fn orbit(x: f32, y: f32, w: f32, h: f32, target: Vec3, dist: f32, azimuth: f32, elevation: f32) -> Self {
        let cam = Camera::perspective(30.0).orbit(target, dist, azimuth, elevation);
        Self { x, y, w, h, cam }
    }

    /// Frame pixel of a world point (None when behind the camera).
    pub fn px(&self, p: Vec3) -> Option<(f32, f32)> {
        self.cam.project(p, self.w, self.h).map(|q| (self.x + q.x, self.y + q.y))
    }

    /// Frame pixel of a world point, (−1000, −1000) when behind the camera.
    pub fn pt(&self, p: Vec3) -> (f32, f32) {
        self.px(p).unwrap_or((-1000.0, -1000.0))
    }

    /// Screen size in pixels of `meters` at world point `p` (to size labels on cubes).
    pub fn pix(&self, p: Vec3, meters: f32) -> f32 {
        let a = self.pt(p);
        let b = self.pt(p + self.cam_right() * meters);
        ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt()
    }

    fn cam_right(&self) -> Vec3 {
        let f = (self.cam.target - self.cam.position).normalize();
        f.cross(Vec3::Y).normalize()
    }

    /// Render the cubes (with soft studio light and a shadow on an invisible floor at `floor_y`,
    /// or no floor with `None`), then write each cube's label on its camera-facing front face.
    /// Labels of cubes hidden behind others are the caller's business: only label the front layer.
    pub fn render<'a>(&self, cubes: &[Cube], floor_y: Option<f32>) -> Svgr<'a> {
        if cubes.is_empty() {
            return Svgr::empty();
        }
        let mut scene = Scene::new().environment(Environment::studio()).background(Background::Transparent);
        let geo = unit_box();
        for c in cubes {
            if c.alpha <= 0.01 || c.s.min_element() <= 0.001 {
                continue;
            }
            let mut m = Material::standard(c.color).roughness(0.45).clearcoat(0.4);
            if c.alpha < 0.999 {
                m = m.opacity(c.alpha).transparent();
            }
            let mut node = Node::mesh(geo.clone(), m).at(c.p.x, c.p.y, c.p.z);
            node.scale = c.s;
            if c.alpha < 0.999 {
                node = node.shadows(false, false);
            }
            scene.add(node);
        }
        if let Some(fy) = floor_y {
            scene.add(Node::mesh(Geometry::floor(60.0, 60.0), Material::shadow_catcher(0.45)).at(0.0, fy, 0.0).shadows(false, true));
        }
        scene.add(Node::light(Light::sun(Rgb::WHITE, 2.4, vec3(3.0, 7.0, 6.0))));
        let mut r = Renderer::new(self.w as u32, self.h as u32).tone_mapping(ToneMapping::Neutral);
        if cpu() {
            r = r.cpu();
        }
        let img = r.render(&scene, &self.cam);
        let (x, y, w, h) = (self.x, self.y, self.w, self.h);
        let mut v: Vec<Svgr<'a>> = vec![fframes::svgr!(<image href={img.href()} x={x} y={y} width={w} height={h} />)];
        // numbers on the front faces
        let toward = (self.cam.position - self.cam.target).normalize();
        for c in cubes {
            let Some(label) = &c.label else { continue };
            if c.alpha < 0.6 {
                continue;
            }
            let face = c.p + toward * (c.s.z * 0.5 + 0.02);
            let Some((px, py)) = self.px(face) else { continue };
            let size = (self.pix(face, c.s.x) * 0.5).clamp(10.0, 46.0);
            let o = c.alpha.min(1.0);
            let s = label.clone();
            v.push(fframes::svgr!(
                <text x={px} y={py + size * 0.35} font-family="JetBrains Mono" font-weight="700" font-size={size} fill="#0b0b0b" text-anchor="middle" opacity={o}>{s}</text>
            ));
        }
        v.into()
    }
}

/// Every cube of a block of shape (nb, nr, nc) in colour `color`, labelled with its row-major index
/// when `numbers` is true (front layer b = 0 only, so labels never show through).
pub fn block(nb: usize, nr: usize, nc: usize, color: u32, numbers: bool) -> Vec<Cube> {
    let mut v = Vec::new();
    for b in 0..nb {
        for r in 0..nr {
            for c in 0..nc {
                let mut cube = Cube::new(grid_pos(b as f32, r as f32, c as f32), color);
                if numbers && b == 0 {
                    cube = cube.label(format!("{}", (b * nr + r) * nc + c));
                }
                v.push(cube);
            }
        }
    }
    v
}
