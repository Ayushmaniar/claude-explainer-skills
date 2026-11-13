//! A 4 s whiteboard-style test of fframes 1.3 3D: a turning stack of layers, one lifting
//! out with a hand label pinned to it, and a paper card (an svgr drawing as a texture) that
//! tilts in. Renders on fframes' CPU frame renderer; the 3D renders on the GPU through wgpu
//! unless FF_3D=cpu.

use std::process::ExitCode;

use fframes::animation::Easing;
use fframes::three_d::*;
use fframes::{
    AudioMap, Color, Duration, FFramesContext, Frame, RenderOptions, StaticMediaProvider, Svgr,
    Video, cli, include_media_dir, svgr,
};

include_media_dir!(pub struct TestMedia, "media");

const W: u32 = 1920;
const H: u32 = 1080;
// the 3D inset on the left
const IX: f32 = 80.0;
const IY: f32 = 200.0;
const IW: u32 = 960;
const IH: u32 = 800;

struct Test {
    scene: Scene,
    layers: Vec<NodeId>,
    renderer: Renderer,
    card_scene: Scene,
    card: NodeId,
    card_renderer: Renderer,
}

impl Test {
    fn new(cpu: bool) -> Self {
        let mut scene = Scene::new()
            .environment(Environment::studio())
            .background(Background::Transparent);
        let colors = [0x5499fa, 0xa08cff, 0x1cd2ac, 0xfcdb43];
        let layers = colors
            .iter()
            .enumerate()
            .map(|(i, &c)| {
                scene.add(
                    Node::mesh(
                        Geometry::rounded_box(1.6, 0.22, 1.0, 0.05, 4),
                        Material::standard(c).roughness(0.45).clearcoat(0.4),
                    )
                    .at(0.0, 0.2 + i as f32 * 0.3, 0.0),
                )
            })
            .collect();
        scene.add(
            Node::mesh(Geometry::floor(12.0, 12.0), Material::shadow_catcher(0.55))
                .shadows(false, true),
        );
        scene.add(Node::light(Light::sun(Rgb::WHITE, 2.6, vec3(2.0, 6.0, 3.0))));
        let mut renderer = Renderer::new(IW, IH).tone_mapping(ToneMapping::Neutral);

        let mut card_scene = Scene::new().environment(Environment::studio());
        let card = card_scene.add(Node::mesh(Geometry::plane(1.6, 1.0), Material::unlit(0xffffff)));
        let mut card_renderer = Renderer::new(720, 480).tone_mapping(ToneMapping::Neutral);
        if cpu {
            renderer = renderer.cpu();
            card_renderer = card_renderer.cpu();
        }
        Self { scene, layers, renderer, card_scene, card, card_renderer }
    }
}

impl Video for Test {
    const FPS: usize = 30;
    const WIDTH: usize = W as usize;
    const HEIGHT: usize = H as usize;
    const BACKGROUND_COLOR: Color = Color::BLACK;

    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(4.0)
    }

    fn audio(&self) -> AudioMap<'_> {
        AudioMap::none()
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let t = frame.seconds();
        let lift = frame.animate(&fframes::timeline!(
            at 1.0, animate 0.0_f32 => 1.0, Easing::Spring { mass: 1.0, stiffness: 120.0, damping: 16.0 },
        ));
        let mut scene = self.scene.clone();
        let top = self.layers[3];
        scene[top].position = vec3(lift * 0.35, 1.1 + lift * 0.6, 0.0);
        let camera = Camera::perspective(32.0).orbit(vec3(0.0, 0.7, 0.0), 5.2, 30.0 + t * 10.0, 20.0);
        let stack = self.renderer.render(&scene, &camera);
        // the label sits on the lifted layer's right end, in frame pixels
        let pin = camera
            .project(scene.local_to_world(top, vec3(0.8, 0.0, 0.0)), IW as f32, IH as f32)
            .map(|p| (IX + p.x, IY + p.y))
            .unwrap_or((0.0, 0.0));
        let label_op = frame.animate(&fframes::timeline!(at 1.6 => 2.0, animate 0.0_f32 => 1.0, Easing::EaseOut));

        // a paper card drawn in svgr, as a texture on a plane that tilts in
        let Some(paper) = ctx.render_svgr(
            svgr!(
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 500" width="800" height="500">
                    <rect x="0" y="0" width="800" height="500" fill="#ffffff" />
                    <text x="60" y="120" font-family="Excalifont" font-size="64" fill="#1a1a1a">"attention"</text>
                    <ellipse cx="400" cy="300" rx="240" ry="90" fill="none" stroke="#e0283c" stroke-width="8" />
                    <text x="400" y="320" text-anchor="middle" font-family="Excalifont" font-size="48" fill="#1f5fd6">"Q K V"</text>
                </svg>
            ),
            800,
            500,
        ) else {
            return Svgr::empty();
        };
        let tilt = frame.animate(&fframes::timeline!(at 0.3 => 1.6, animate 1.0_f32 => 0.0, Easing::EaseInOut));
        let mut card_scene = self.card_scene.clone();
        card_scene[self.card].rotation = euler(deg(-8.0) * tilt, deg(70.0) * tilt + deg(-12.0), 0.0);
        if let Some(material) = card_scene[self.card].material_mut() {
            material.map = Some(Texture::from_image(&paper).into());
        }
        let card = self.card_renderer.render(&card_scene, &Camera::perspective(35.0).at(0.0, 0.0, 2.2));

        let (px, py) = pin;
        let (lx, ly) = (px + 60.0, py - 170.0);
        svgr!(
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1920 1080" width="1920" height="1080">
                <rect x="0" y="0" width="1920" height="1080" fill="#030303" />
                <text x="80" y="130" font-family="Excalifont" font-size="72" fill="#fcdb43">"3D on the whiteboard"</text>
                <image href={stack.href()} x={IX} y={IY} width={IW as f32} height={IH as f32} />
                <g opacity={label_op}>
                    <line x1={px} y1={py} x2={lx} y2={ly} stroke="#f2f2ef" stroke-width="4" stroke-linecap="round" />
                    <circle cx={px} cy={py} r="8" fill="#f2f2ef" />
                    <text x={lx + 12.0} y={ly + 10.0} font-family="Excalifont" font-size="44" fill="#f2f2ef">"one layer"</text>
                </g>
                <image href={card.href()} x="1120" y="300" width="720" height="480" />
            </svg>
        )
    }
}

fn main() -> ExitCode {
    let args = cli::parse::<cli::NoArgs>();
    let media = TestMedia::prepare().expect("media");
    let video = Test::new(std::env::var("FF_3D").is_ok_and(|v| v == "cpu"));
    let options = RenderOptions { media: Some(&media), default_font: "Excalifont", ..Default::default() };
    cli::new(&video, options).args(args).run()
}
