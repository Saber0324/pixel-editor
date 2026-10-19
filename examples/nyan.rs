use std::path::Path;

use pixel_editor::importer::from_gif;
use pixel_editor::renderer::play_animation_terminal;

fn main() {
    let sprite = from_gif(Path::new("assets/nyan.gif")).expect("Gif should load properly");
    play_animation_terminal(&sprite, None).expect("Should play the animation"); // None = loop forever
}
