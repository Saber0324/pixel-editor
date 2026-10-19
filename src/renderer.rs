use crate::sprite::{Color, Sprite, SpriteError};

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
