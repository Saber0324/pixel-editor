use crate::sprite::{Color, Sprite, SpriteError};
use std::fmt::Write as _;
use std::io::{self, Write};

pub fn render_frame(sprite: &Sprite, frame: usize) -> Result<Vec<Color>, SpriteError> {
    if frame >= sprite.get_frame_count() {
        return Err(SpriteError::InvalidFrameIndex(frame));
    }

    let width = sprite.width() as usize;
    let height = sprite.height() as usize;
    let mut canvas = vec![Color::transparent(); width * height];

    for layer in sprite.layers().iter().filter(|layer| layer.is_visible()) {
        let Some(cel) = layer.cel(frame) else {
            continue;
        };
        for (col, row, index) in cel.iter_pixels() {
            if index.value() == 0 {
                continue;
            }
            let Some(&src) = sprite.palette_colors(index) else {
                continue;
            };
            let px = &mut canvas[row as usize * width + col as usize];
            *px = src.blend(*px);
        }
    }
    Ok(canvas)
}

pub fn render_frame_to_terminal(sprite: &Sprite, frame: usize) -> Result<(), SpriteError> {
    let canvas = render_frame(sprite, frame)?;
    let width = sprite.width() as usize;

    let mut buf = String::new();
    buf.push_str("\x1b[H"); // cursor to top-left so frames overwrite each other

    for row in canvas.chunks(width) {
        for color in row {
            if color.alpha == 0 {
                buf.push_str("\x1b[0m  "); // resets ansi formatting
            } else {
                write!(
                    buf,
                    "\x1b[48;2;{};{};{}m  ",
                    color.red, color.green, color.blue
                )
                .unwrap();
            }
        }
        buf.push_str("\x1b[0m\n"); // resets ansi formatting.
    }

    let mut out = io::stdout().lock();
    out.write_all(buf.as_bytes())?;
    out.flush()?;
    Ok(())
}
