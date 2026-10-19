use pixel_editor::renderer::render_frame_to_terminal;
use pixel_editor::sprite::{Color, Index, Sprite};
/// # is a pixel, . is empty.
const HEART: [&str; 8] = [
    ".##...##.",
    "####.####",
    "#########",
    "#########",
    ".#######.",
    "..#####..",
    "...###...",
    "....#....",
];

const SHINE: [&str; 3] = [".........", ".##......", ".#......."];

/// Paints every # of the art on the given layer of the current frame.
fn draw_art(sprite: &mut Sprite, layer: &str, art: &[&str], color: Index) {
    sprite.set_active_layer(layer).expect("Layer should exist");

    for (row, line) in (0u32..).zip(art) {
        for (col, ch) in (0u32..).zip(line.chars()) {
            if ch == '#' {
                sprite.set_pixel(col, row, color);
            }
        }
    }
}

fn main() {
    let mut sprite = Sprite::new(9, 8);
    let red = sprite
        .add_color(Color::new(220, 20, 60, 255))
        .expect("Should have less than 256 colors");
    let white = sprite
        .add_color(Color::new(255, 255, 255, 255))
        .expect("Should have less than 256 colors");

    sprite.add_layer("Shine").expect("Name should be available");

    draw_art(&mut sprite, "Background", &HEART, red);
    draw_art(&mut sprite, "Shine", &SHINE, white);

    print!("\x1b[2J"); // clear the screen once
    render_frame_to_terminal(&sprite, 0).expect("Frame should be rendered");
    println!();
}
