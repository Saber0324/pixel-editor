use canvas::{Color, Sprite, SpriteError};
use std::fmt::Write as _;
use std::io::{self, Write};
use std::thread;
use std::time::Instant;
use thiserror::Error;

pub mod importer;
pub use importer::from_gif;

#[derive(Error, Debug)]
pub enum RendererError {
    #[error(transparent)]
    Sprite(#[from] SpriteError),

    #[error(transparent)]
    Io(#[from] io::Error),
}

pub fn render_frame(sprite: &Sprite, frame: usize) -> Result<Vec<Color>, RendererError> {
    if frame >= sprite.get_frame_count() {
        Err(SpriteError::InvalidFrameIndex(frame))?;
    }

    let width = sprite.width() as usize;
    let height = sprite.height() as usize;
    let mut canvas = vec![Color::TRANSPARENT; width * height];

    for layer in sprite.layers().filter(|layer| layer.is_visible()) {
        let Some(cel) = layer.get_cel(frame) else {
            continue;
        };
        for (col, row, index) in cel.iter_pixels() {
            if index.value() == 0 {
                continue;
            }
            let src = sprite.get_color(index);
            let px = &mut canvas[row as usize * width + col as usize];
            *px = src.blend(*px);
        }
    }
    Ok(canvas)
}

pub fn render_frame_to_terminal(sprite: &Sprite, frame: usize) -> Result<(), RendererError> {
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
                .expect("Should be able to write to the buffer");
            }
        }
        buf.push_str("\x1b[0m\n"); // resets ansi formatting.
    }

    let mut out = io::stdout().lock();
    out.write_all(buf.as_bytes())?;
    out.flush()?;
    Ok(())
}

pub fn play_animation_terminal(sprite: &Sprite, loops: Option<usize>) -> Result<(), RendererError> {
    let mut out = std::io::stdout();
    write!(out, "\x1b[2J\x1b[?25l")?;
    let mut played = 0;
    while loops.is_none_or(|max| played < max) {
        for frame in 0..sprite.get_frame_count() {
            let start = Instant::now();
            render_frame_to_terminal(sprite, frame)?;
            let duration = sprite.get_frame_duration(frame)?;
            if let Some(rest) = duration.checked_sub(start.elapsed()) {
                thread::sleep(rest);
            }
        }
        played += 1;
    }
    write!(out, "\x1b[0m\x1b[?25h")?;
    out.flush()?;

    Ok(())
}
