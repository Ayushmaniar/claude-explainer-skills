// Uncomment if a build fails with "overflow evaluating the requirement ...: Sync" (seen once when adding 3D
// insets that hold wgpu types to a scene struct):
// #![recursion_limit = "256"]
use explainer_video::{ExplainerMedia, ExplainerVideo};
use fframes::{CombinedMediaProvider, EncoderOptions, MediaDirectory, MediaProvider, RenderOptions, StaticMediaProvider, cli};
use std::process::ExitCode;

/// Reads a media folder at runtime. `sprites/` and `figs/` may be empty or missing (no host, no figures):
/// they are created empty then, and the video renders without them.
fn folder(name: &str) -> MediaDirectory {
    if !std::path::Path::new(name).is_dir() {
        std::fs::create_dir_all(name).unwrap_or_else(|e| panic!("can't create the {name}/ folder: {e}"));
    }
    MediaDirectory::read_folder(name).unwrap_or_else(|e| panic!("can't read the {name}/ folder: {e}"))
}

fn main() -> ExitCode {
    let args = cli::parse::<cli::NoArgs>();
    // Fonts are compiled in from `media/`; the narration (`audio/`), the host's frames (`sprites/`) and the
    // figures (`figs/`) load at runtime from the project folder (run the binary from there).
    let media = ExplainerMedia::prepare().expect("media");
    if !std::path::Path::new("audio/narration.wav").is_file() {
        eprintln!("audio/narration.wav is missing: run narration/build_vo.py (or narration/est_cues.py for a silent draft)");
        return ExitCode::from(1);
    }
    let audio_dir = folder("audio");
    let audio = audio_dir.process_media_source().expect("audio media");
    let sprite_dir = folder("sprites");
    let sprites = sprite_dir.process_media_source().expect("sprite media");
    let fig_dir = folder("figs");
    let figs = fig_dir.process_media_source().expect("figure media");
    let all_media = CombinedMediaProvider::from([&media as &dyn MediaProvider, &audio, &sprites, &figs]);
    let video = ExplainerVideo::new();
    let options = RenderOptions {
        media: Some(&all_media),
        default_font: "Excalifont",
        video_encoder_options: EncoderOptions {
            preferred_encoder: Some("libx264"),
            codec_params: Some(&[("crf", "20"), ("preset", "medium"), ("tune", "animation")]),
            ..Default::default()
        },
        ..Default::default()
    };
    // fframes' built-in CPU frame renderer: the GPU (Skia/Vulkan) video path has pasted streaks and bands of
    // other frames into long renders. 3D insets (src/t3d.rs) still render on the GPU through wgpu unless FF_3D=cpu.
    cli::new(&video, options).args(args).run()
}
