use canvas::{Color, Index, Sprite, SpriteError};
use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, RgbaImage};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error(transparent)]
    Image(#[from] image::ImageError),
    #[error(transparent)]
    Sprite(#[from] SpriteError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("The file has no frames")]
    NoFrames,
    #[error("A {frame_w}x{frame_h} frame doesn't fit in this sheet")]
    BadGrid { frame_w: u32, frame_h: u32 },
}

/// Maps RGBA values to palette indexes, adding colors as they appear.
#[derive(Default)]
pub struct PaletteMap {
    known: HashMap<[u8; 4], Index>,
}

impl PaletteMap {
    /// Returns the palette index for a pixel, or `None` if it's fully transparent.
    /// Once the palette is full, new colors snap to the closest existing one.
    fn index_for(
        &mut self,
        sprite: &mut Sprite,
        px: [u8; 4],
    ) -> Result<Option<Index>, SpriteError> {
        if px[3] == 0 {
            return Ok(None);
        }
        if let Some(&idx) = self.known.get(&px) {
            return Ok(Some(idx));
        }
        match sprite.add_color(Color::new(px[0], px[1], px[2], px[3])) {
            Ok(idx) => {
                self.known.insert(px, idx);
                Ok(Some(idx))
            }
            Err(SpriteError::PaletteOverflow) => {
                let distance = |c: &[u8; 4]| -> i32 {
                    (0..4)
                        .map(|i| (i32::from(c[i]) - i32::from(px[i])).pow(2))
                        .sum()
                };
                let nearest = self
                    .known
                    .iter()
                    .min_by_key(|(c, _)| distance(c))
                    .map(|(_, &idx)| idx);
                Ok(nearest)
            }
            Err(e) => Err(e),
        }
    }
}

/// Paints an image onto the sprite's current frame and active layer.
fn paint_frame(
    sprite: &mut Sprite,
    img: &RgbaImage,
    palette: &mut PaletteMap,
) -> Result<(), SpriteError> {
    for (x, y, px) in img.enumerate_pixels() {
        if let Some(idx) = palette.index_for(sprite, px.0)? {
            sprite.set_pixel(x, y, idx);
        }
    }
    Ok(())
}

pub fn from_gif(path: &Path) -> Result<Sprite, ImportError> {
    let decoder = GifDecoder::new(BufReader::new(File::open(path)?))?;
    let frames = decoder.into_frames().collect_frames()?;
    let first = frames.first().ok_or(ImportError::NoFrames)?;
    let (w, h) = first.buffer().dimensions();

    let mut sprite = Sprite::new(w, h);
    let mut palette = PaletteMap::default();

    for (i, frame) in frames.iter().enumerate() {
        if i > 0 {
            sprite.add_frame();
        }
        sprite.set_current_frame(i)?;
        paint_frame(&mut sprite, frame.buffer(), &mut palette)?;

        let (num, denom) = frame.delay().numer_denom_ms();
        let mut ms = u64::from(num) / u64::from(denom.max(1));
        if ms < 20 {
            ms = 100; // browsers treat 0-10 ms GIF delays as 100 ms
        }
        sprite.set_frame_duration(i, Duration::from_millis(ms))?;
    }
    Ok(sprite)
}
