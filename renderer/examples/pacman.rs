use std::time::Duration;

use canvas::{Color, Index, Sprite};
use renderer::play_animation_terminal;

const WIDTH: u32 = 33;
const HEIGHT: u32 = 13;

/// Pac-Man is 13x13, at the left edge.
const PAC_SIZE: i32 = 13;

/// `#` is a pixel, `.` is empty. Frame order: closed, half, wide, half.
const PAC_FRAMES: [[&str; 13]; 4] = [
    // 0: mouth closed
    [
        "....#####....",
        "..#########..",
        ".###########.",
        ".###########.",
        "#############",
        "#############",
        "#############",
        "#############",
        "#############",
        ".###########.",
        ".###########.",
        "..#########..",
        "....#####....",
    ],
    // 1: mouth half open
    [
        "....#####....",
        "..#########..",
        ".###########.",
        ".###########.",
        "##########...",
        "########.....",
        "#######......",
        "########.....",
        "##########...",
        ".###########.",
        ".###########.",
        "..#########..",
        "....#####....",
    ],
    // 2: mouth wide open
    [
        "....#####....",
        "..#########..",
        ".#########...",
        ".########....",
        "########.....",
        "#######......",
        "######.......",
        "#######......",
        "########.....",
        ".########....",
        ".#########...",
        "..#########..",
        "....#####....",
    ],
    // 3: mouth half open again
    [
        "....#####....",
        "..#########..",
        ".###########.",
        ".###########.",
        "##########...",
        "########.....",
        "#######......",
        "########.....",
        "##########...",
        ".###########.",
        ".###########.",
        "..#########..",
        "....#####....",
    ],
];

/// Small highlight on the upper left, drawn above the body.
const SHINE: [&str; 4] = [
    ".............",
    ".............",
    "...##........",
    "..#..........",
];

/// One eye, 1 pixel wide and 2 tall.
const EYE: [&str; 4] = [
    ".............",
    ".............",
    ".......#.....",
    ".......#.....",
];

/// A round-ish pellet (plus shape), 3x3.
const PELLET: [&str; 3] = [".#.", "###", ".#."];

const FRAME_MS: u64 = 120;

/// Pellets are 8 pixels apart and move 2 pixels per frame,
/// so after 4 frames the pattern repeats exactly.
const PELLET_SPACING: i32 = 8;
const PELLET_SPEED: i32 = 2;

/// Paints every `#` of the art on the given layer, with its top-left at (x0, y0).
fn draw_art(sprite: &mut Sprite, layer: &str, art: &[&str], color: Index, x0: i32, y0: i32) {
    sprite.set_active_layer(layer).expect("Layer should exist");

    for (row, line) in (0i32..).zip(art) {
        for (col, ch) in (0i32..).zip(line.chars()) {
            if ch == '#' {
                let (x, y) = (x0 + col, y0 + row);
                if x >= 0 && y >= 0 && (x.cast_unsigned()) < WIDTH && (y.cast_unsigned()) < HEIGHT {
                    sprite.set_pixel(x.cast_unsigned(), y.cast_unsigned(), color);
                }
            }
        }
    }
}

/// Draws the scrolling pellets for the given frame number.
fn draw_pellets(sprite: &mut Sprite, frame: i32, color: Index) {
    let mut x = PAC_SIZE + 1 - PELLET_SPEED * frame;
    // Move the first pellet to the first slot that is clear of Pac-Man.
    while x < PAC_SIZE + 1 {
        x += PELLET_SPACING;
    }
    while x + 2 < WIDTH.cast_signed() {
        draw_art(
            sprite,
            "Background",
            &PELLET,
            color,
            x,
            (HEIGHT / 2).cast_signed() - 1,
        );
        x += PELLET_SPACING;
    }
}

fn main() {
    let mut sprite = Sprite::new(WIDTH, HEIGHT);
    let yellow = sprite
        .add_color(Color::new(255, 221, 0, 255))
        .expect("Should have less than 256 colors");
    let light = sprite
        .add_color(Color::new(255, 245, 150, 255))
        .expect("Should have less than 256 colors");
    let black = sprite
        .add_color(Color::new(20, 20, 20, 255))
        .expect("Should have less than 256 colors");
    let cream = sprite
        .add_color(Color::new(255, 224, 189, 255))
        .expect("Should have less than 256 colors");

    sprite
        .add_layer("Pacman")
        .expect("Name should be available");
    sprite.add_layer("Shine").expect("Name should be available");
    sprite.add_layer("Eye").expect("Name should be available");

    // Create all frames first, then draw, so nothing leaks between frames
    // whether or not `add_frame` copies the previous one.
    for _ in 1..PAC_FRAMES.len() {
        sprite.add_frame();
    }

    for (i, art) in PAC_FRAMES.iter().enumerate() {
        sprite.set_current_frame(i).expect("Frame should exist");

        draw_pellets(
            &mut sprite,
            i32::try_from(i).expect("Should not wrap"),
            cream,
        );
        draw_art(&mut sprite, "Pacman", art, yellow, 0, 0);
        draw_art(&mut sprite, "Shine", &SHINE, light, 0, 0);
        draw_art(&mut sprite, "Eye", &EYE, black, 0, 0);

        sprite
            .set_frame_duration(i, Duration::from_millis(FRAME_MS))
            .expect("Frame should exist");
    }

    play_animation_terminal(&sprite, None).expect("Should play the animation"); // None = loop forever
}
