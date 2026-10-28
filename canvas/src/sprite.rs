use std::ops::{Bound, RangeBounds};
use std::time::Duration;
use thiserror::Error;

use crate::cel::Cel;
use crate::color::{Color, Index};
use crate::layer::{Frame, Layer, LayerError};

#[derive(Error, Debug)]
pub enum SpriteError {
    #[error("Can't add more than 256 colors to a single palette!")]
    PaletteOverflow,

    #[error("Invalid frame index: {0}")]
    InvalidFrameIndex(usize),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid color index: {0}")]
    InvalidColorIndex(u8),

    #[error("Can't remove the last frame of the Sprite")]
    LastFrame,
}

/// An animation made of `Frame`s, with `Layer`s shared across all of them.
///
/// `palette[0]` is always transparent. `current_frame` and `active_layer` are indexes
/// into `frames` and `layers`, and decide where `set_pixel` draws.
#[derive(Debug)]
pub struct Sprite {
    width: u32,
    height: u32,
    palette: Vec<Color>,
    frames: Vec<Frame>,
    layers: Vec<Layer>,
    current_frame: usize,
    active_layer: usize,
}

/// Which view of a `Layer` the display functions print.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// What the `Layer` contributes to the canvas: every position shows its palette index,
    /// and anything without a drawn pixel shows as `0`.
    Canvas,
    /// What is actually stored: `.` is a transparent pixel inside the `Cel`, and a blank
    /// is anything not stored (no `Cel`, or outside its rectangle).
    Cel,
}

/// Resolves a range against the canvas width into inclusive ends.
/// Returns `None` if the range is empty or entirely outside the canvas.
fn resolve_range(range: impl RangeBounds<u32>, width: u32) -> Option<(u32, u32)> {
    let from = match range.start_bound() {
        Bound::Included(&start) => start,
        Bound::Excluded(&start) => start.checked_add(1)?,
        Bound::Unbounded => 0,
    };
    let to = match range.end_bound() {
        Bound::Included(&end) => end,
        Bound::Excluded(&end) => end.checked_sub(1)?,
        Bound::Unbounded => width.checked_sub(1)?,
    };
    let to = to.min(width.checked_sub(1)?);
    (from <= to).then_some((from, to))
}

impl Sprite {
    /// Creates a new `Sprite`
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            palette: vec![Color::TRANSPARENT],
            active_layer: 0,
            frames: vec![Frame::default()],
            current_frame: 0,
            layers: vec![Layer::default()],
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the `usize` index of the selected `Layer`
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if `name` does not match a `Layer.name` in `self.layers`
    fn get_layer_index(&self, name: &str) -> Result<usize, LayerError> {
        self.layers
            .iter()
            .position(|layer| layer.get_name() == name)
            .ok_or_else(|| LayerError::NotFound(name.to_string()))
    }

    /// Returns the `Index` of an existing palette entry.
    ///
    /// # Errors
    ///
    /// Returns `SpriteError::InvalidColorIndex` if the palette has no such entry.
    pub fn palette_index(&self, value: u8) -> Result<Index, SpriteError> {
        if usize::from(value) < self.palette.len() {
            Ok(Index(value))
        } else {
            Err(SpriteError::InvalidColorIndex(value))
        }
    }

    /// Returns a reference to a `Layer` of the `Sprite`.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if no `Layer` has that name.
    pub fn get_layer(&self, name: &str) -> Result<&Layer, LayerError> {
        Ok(&self.layers[self.get_layer_index(name)?])
    }

    /// Returns a reference to the `Sprite.active_layer` of the `Sprite`.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if no `Layer` is found.
    pub fn get_active_layer(&self) -> Result<&Layer, LayerError> {
        Ok(&self.layers[self.active_layer])
    }

    /// Returns an iterator over the `Layer`s of the `Sprite`
    pub fn layers(&self) -> impl Iterator<Item = &Layer> {
        self.layers.iter()
    }

    /// Returns the `Cel` of a `Layer` in the current frame, or `None` if nothing is drawn there.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if no `Layer` has that name.
    pub fn get_cel(&self, name: &str) -> Result<Option<&Cel>, LayerError> {
        let index = self.get_layer_index(name)?;
        Ok(self.layers[index].cels[self.current_frame].as_ref())
    }

    /// Shrinks the active `Layer`'s cel in the current frame to the pixels actually drawn.
    /// A cel with nothing left becomes `None`.
    pub fn trim_active_cel(&mut self) {
        let slot = &mut self.layers[self.active_layer].cels[self.current_frame];
        let is_empty = slot.as_mut().is_some_and(|cel| !cel.trim());
        if is_empty {
            *slot = None;
        }
    }

    /// Adds a `Layer` on top of the stack, with an empty slot for every existing `Frame`.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NameCollision` if a `Layer` with that name already exists.
    pub fn add_layer(&mut self, name: &str) -> Result<(), LayerError> {
        if self.get_layer_index(name).is_ok() {
            return Err(LayerError::NameCollision);
        }
        self.layers.push(Layer::new(name, self.frames.len()));
        Ok(())
    }

    /// Changes the `Layer` that `set_pixel` draws on.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if no `Layer` has that name.
    pub fn set_active_layer(&mut self, name: &str) -> Result<(), LayerError> {
        self.active_layer = self.get_layer_index(name)?;
        Ok(())
    }

    /// Sets the pixel at the given row and column on the active `Layer` in the current frame.
    ///
    /// Creates the `Cel` on the first write and grows it when drawing outside its rectangle.
    /// Positions outside the canvas are ignored.
    pub fn set_pixel(&mut self, col: u32, row: u32, palette_index: Index) {
        if row >= self.height || col >= self.width {
            return;
        }
        let slot = &mut self.layers[self.active_layer].cels[self.current_frame];
        match slot {
            Some(cel) => cel.set(col, row, palette_index),
            None if palette_index == Index(0) => {} // erasing an empty slot: nothing to do
            None => *slot = Some(Cel::single(col, row, palette_index)),
        }
    }

    /// Shows or hides a `Layer`.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if the `Layer` is not found in the `Sprite`.
    pub fn set_layer_visible(&mut self, name: &str, visible: bool) -> Result<(), LayerError> {
        let index = self.get_layer_index(name)?;
        self.layers[index].set_visible(visible);
        Ok(())
    }

    /// Removes a `Layer` and all of its `Cel`s.
    ///
    /// If the active `Layer` is removed, the `Layer` above it becomes active
    /// (or the one below, if it was the top one).
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if no `Layer` has that name, and
    /// `LayerError::LastLayer` if it is the only one left.
    pub fn remove_layer(&mut self, name: &str) -> Result<(), LayerError> {
        let index = self.get_layer_index(name)?;
        if self.layers.len() == 1 {
            return Err(LayerError::LastLayer);
        }
        self.layers.remove(index);

        if self.active_layer > index {
            self.active_layer -= 1;
        }
        self.active_layer = self.active_layer.min(self.layers.len() - 1);
        Ok(())
    }

    /// Resets a pixel to `Index(0)`, making it transparent.
    /// Erasing where no `Cel` exists does nothing.
    pub fn erase_pixel(&mut self, col: u32, row: u32) {
        self.set_pixel(col, row, Index(0));
    }

    /// Adds a color to the palette and returns its `Index`.
    ///
    /// # Errors
    ///
    /// Returns `SpriteError::PaletteOverflow` if the palette already holds 256 entries.
    pub fn add_color(&mut self, color: Color) -> Result<Index, SpriteError> {
        if self.palette.len() >= 256 {
            Err(SpriteError::PaletteOverflow)
        } else {
            let idx = u8::try_from(self.palette.len()).expect("Should be always a valid u8");
            self.palette.push(color);
            Ok(Index(idx))
        }
    }

    /// Returns the `Color` in `Sprite.palette` from the provided `Index`
    pub fn get_color(&self, index: Index) -> Color {
        self.palette[index.value() as usize]
    }

    /// Prints a `Layer`'s grid for the current frame, one palette index per position.
    /// See `DisplayMode` for what each view shows.
    fn display_layer_data(&self, layer: &Layer, mode: DisplayMode) {
        let cel = layer.cels[self.current_frame].as_ref();
        println!(
            "Layer: '{}' (Visible: {})",
            layer.get_name(),
            layer.is_visible()
        );
        for row in 0..self.height {
            for col in 0..self.width {
                let cell = match (mode, cel) {
                    (DisplayMode::Canvas, _) => {
                        let value = cel.map_or(0, |cel| cel.get(col, row).value());
                        value.to_string()
                    }
                    (DisplayMode::Cel, Some(cel)) if cel.contains(col, row) => {
                        match cel.get(col, row).value() {
                            0 => ".".to_string(),
                            value => value.to_string(),
                        }
                    }
                    (DisplayMode::Cel, _) => " ".to_string(),
                };
                print!("{cell} ");
            }
            println!();
        }
        println!();
    }

    /// Prints one `Layer`'s grid for the current frame in the given `DisplayMode`.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if no `Layer` has that name.
    pub fn display_layer(&self, name: &str, mode: DisplayMode) -> Result<(), LayerError> {
        let layer = self.get_layer(name)?;
        self.display_layer_data(layer, mode);
        Ok(())
    }

    /// Prints every `Layer`'s grid for the current frame in the given `DisplayMode`,
    /// bottom layer first.
    pub fn display_all_layers(&self, mode: DisplayMode) {
        for layer in &self.layers {
            self.display_layer_data(layer, mode);
        }
    }

    /// Prints the active `Layer`'s grid for the current frame in the given `DisplayMode`.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if no `Layer` has that name.
    pub fn display_active_layer(&self, mode: DisplayMode) -> Result<(), LayerError> {
        let layer = self.get_active_layer()?;
        self.display_layer_data(layer, mode);
        Ok(())
    }

    /// Adds a `Frame` with the default duration, and an empty slot in every `Layer`.
    pub fn add_frame(&mut self) {
        self.frames.push(Frame::default());
        for layer in &mut self.layers {
            layer.cels.push(None);
        }
    }

    /// Switches the current `Frame`. The active `Layer` stays the same.
    ///
    /// # Errors
    ///
    /// Returns `SpriteError::InvalidFrameIndex` if the `Frame` doesn't exist.
    pub fn set_current_frame(&mut self, index: usize) -> Result<(), SpriteError> {
        if index < self.frames.len() {
            self.current_frame = index;
            Ok(())
        } else {
            Err(SpriteError::InvalidFrameIndex(index))
        }
    }

    /// Returns the total number of frames in the sprite animation.
    pub fn get_frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Returns the index of the currently active frame.
    pub fn get_current_frame_index(&self) -> usize {
        self.current_frame
    }

    /// Removes a `Frame` and the `Cel` of every `Layer` in it.
    ///
    /// The current frame keeps pointing at the same `Frame` when possible, and is
    /// clamped to the last one otherwise.
    ///
    /// # Errors
    ///
    /// Returns `SpriteError::InvalidFrameIndex` if the `Frame` doesn't exist, and
    /// `SpriteError::LastFrame` if it is the only one left.
    pub fn remove_frame(&mut self, index: usize) -> Result<(), SpriteError> {
        if index >= self.frames.len() {
            return Err(SpriteError::InvalidFrameIndex(index));
        }
        if self.frames.len() == 1 {
            return Err(SpriteError::LastFrame);
        }
        self.frames.remove(index);
        for layer in &mut self.layers {
            layer.cels.remove(index);
        }

        if self.current_frame > index {
            self.current_frame -= 1;
        }
        self.current_frame = self.current_frame.min(self.frames.len() - 1);
        Ok(())
    }

    /// Sets how long a `Frame` stays on screen.
    ///
    /// # Errors
    ///
    /// Returns `SpriteError::InvalidFrameIndex` if the `Frame` doesn't exist.
    pub fn set_frame_duration(
        &mut self,
        index: usize,
        duration: Duration,
    ) -> Result<(), SpriteError> {
        self.frames
            .get_mut(index)
            .ok_or(SpriteError::InvalidFrameIndex(index))?
            .set_duration(duration);
        Ok(())
    }

    pub fn get_frame_duration(&self, index: usize) -> Result<Duration, SpriteError> {
        self.frames
            .get(index)
            .map(Frame::get_duration)
            .ok_or(SpriteError::InvalidFrameIndex(index))
    }

    /// Fills a rectangle on the active `Layer` in the current frame. Each axis accepts any range
    /// form. Empty or off-canvas ranges do nothing, and ranges past the canvas edge are clipped.
    pub fn set_rect(
        &mut self,
        cols: impl RangeBounds<u32>,
        rows: impl RangeBounds<u32>,
        index: Index,
    ) {
        let (Some((left, right)), Some((top, bottom))) = (
            resolve_range(cols, self.width),
            resolve_range(rows, self.height),
        ) else {
            return;
        };
        let slot = &mut self.layers[self.active_layer].cels[self.current_frame];
        match slot {
            Some(cel) => cel.set_rect(left, top, right, bottom, index),
            None if index == Index(0) => {}
            None => {
                let mut cel = Cel::single(left, top, index);
                cel.set_rect(left, top, right, bottom, index);
                *slot = Some(cel);
            }
        }
    }

    pub fn set_row(&mut self, row: u32, cols: impl RangeBounds<u32>, index: Index) {
        self.set_rect(cols, row..=row, index);
    }

    pub fn set_col(&mut self, col: u32, rows: impl RangeBounds<u32>, index: Index) {
        self.set_rect(col..=col, rows, index);
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Returns the Index at a canvas position of a Layer in the current frame,
    /// or Index(0) if no Cel exists.
    fn pixel_at(sprite: &Sprite, layer: &str, col: u32, row: u32) -> Index {
        sprite
            .get_cel(layer)
            .unwrap()
            .map_or(Index(0), |cel| cel.get(col, row))
    }

    /// Verifies that a new sprite is correctly initialized with the specified
    /// width and height, and automatically contains a default "Background" layer.
    #[test]
    fn test_sprite_initialization() {
        let sprite = Sprite::new(10, 5);
        assert_eq!(sprite.width, 10);
        assert_eq!(sprite.height, 5);
        assert!(sprite.get_layer("Background").is_ok());
    }

    /// Verifies that colors can be successfully added to the sprite's palette,
    /// and that exceeding the 256-color limit correctly triggers a
    ///  `SpriteError::PaletteOverflow`.
    #[test]
    fn test_color_addition_and_limit() {
        let mut sprite = Sprite::new(5, 5);

        let color = Color::new(255, 0, 0, 255);
        let idx = sprite.add_color(color);
        assert!(idx.is_ok());
        assert_eq!(idx.unwrap().value(), 1);

        // Fill up to the 256 limit (index 0 is already taken, so 255 more)
        for _ in 0..255 {
            let _ = sprite.add_color(Color::new(0, 0, 0, 255));
        }

        // The next one should fail with PaletteOverflow
        let overflow_result = sprite.add_color(Color::new(255, 255, 255, 255));
        assert!(matches!(overflow_result, Err(SpriteError::PaletteOverflow)));
    }

    /// Verifies that new layers can be successfully added and retrieved,
    /// that duplicate layer names trigger a `LayerError::NameCollision`,
    /// and that looking up non-existent layers returns a `LayerError::NotFound` error.
    #[test]
    fn test_layer_management_and_errors() {
        let mut sprite = Sprite::new(4, 4);

        // Add a new unique layer successfully
        assert!(sprite.add_layer("Foreground").is_ok());
        assert!(sprite.get_layer("Foreground").is_ok());
        assert!(sprite.get_layer("Foreground").unwrap().is_visible());

        // Attempting to add a duplicate layer name should cause a NameCollision
        let duplicate_result = sprite.add_layer("Foreground");
        assert!(matches!(duplicate_result, Err(LayerError::NameCollision)));

        // Requesting a non-existent layer should return NotFound
        let missing_result = sprite.get_layer("GhostLayer");
        assert!(matches!(missing_result, Err(LayerError::NotFound(_))));

        assert!(sprite.set_layer_visible("Foreground", false).is_ok());
    }

    /// Verifies that the active layer can be successfully switched to an existing layer,
    /// and that attempting to switch to a non-existent layer returns an error.
    #[test]
    fn test_active_layer_switching() {
        let mut sprite = Sprite::new(4, 4);
        sprite.add_layer("Foreground").unwrap();

        // Switch to Foreground successfully
        assert!(sprite.set_active_layer("Foreground").is_ok());

        // Switching to a non-existent layer should fail
        assert!(sprite.set_active_layer("BadLayer").is_err());
    }

    /// Verifies that pixels can be successfully drawn on the active layer using a palette index,
    /// erased back to the default transparent index (0), and that out-of-bounds pixel coordinates
    /// are safely ignored without panicking.
    #[test]
    fn test_pixel_drawing_and_erasing() {
        let mut sprite = Sprite::new(3, 3);
        let red = sprite.add_color(Color::new(255, 0, 0, 255)).unwrap();

        // Draw a pixel
        sprite.set_pixel(1, 1, red);
        assert_eq!(pixel_at(&sprite, "Background", 1, 1), red);

        // Erase the pixel
        sprite.erase_pixel(1, 1);

        // Check it again
        assert_eq!(pixel_at(&sprite, "Background", 1, 1), Index(0));

        // Out of bounds should be silently ignored without crashing
        sprite.set_pixel(10, 10, red);
    }

    /// Verifies that `set_pixel` takes `(col, row)`, using a position where the two differ.
    #[test]
    fn test_set_pixel_uses_col_then_row() {
        let mut sprite = Sprite::new(4, 4);
        let red = sprite.add_color(Color::new(255, 0, 0, 255)).unwrap();

        sprite.set_pixel(3, 1, red); // col 3, row 1

        assert_eq!(pixel_at(&sprite, "Background", 3, 1), red);
        assert_eq!(pixel_at(&sprite, "Background", 1, 3), Index(0));
    }

    /// Verifies that `erase_pixel` takes `(col, row)`, using positions where the two differ.
    #[test]
    fn test_erase_pixel_uses_col_then_row() {
        let mut sprite = Sprite::new(4, 4);
        let red = sprite.add_color(Color::new(255, 0, 0, 255)).unwrap();

        sprite.set_pixel(3, 1, red);
        sprite.set_pixel(1, 3, red);
        sprite.erase_pixel(3, 1);

        assert_eq!(pixel_at(&sprite, "Background", 3, 1), Index(0));
        assert_eq!(pixel_at(&sprite, "Background", 1, 3), red);
    }

    /// Verifies that the display and printing helper functions execute cleanly in both
    /// `DisplayMode`s, including for a `Layer` with no `Cel` and for a `Cel` with a
    /// transparent gap inside it, and that a non-existent layer returns an error.
    #[test]
    fn test_display_functions_dont_panic() {
        let mut sprite = Sprite::new(3, 3);
        let blue = sprite.add_color(Color::new(0, 0, 255, 255)).unwrap();
        sprite.add_layer("Empty").unwrap();

        // Background gets a cel with a gap: (0, 0) and (2, 2) drawn, (1, 1) transparent.
        sprite.set_pixel(0, 0, blue);
        sprite.set_pixel(2, 2, blue);

        for mode in [DisplayMode::Canvas, DisplayMode::Cel] {
            // A layer with a cel, and a layer with no cel at all
            assert!(sprite.display_layer("Background", mode).is_ok());
            assert!(sprite.display_layer("Empty", mode).is_ok());
            assert!(sprite.display_layer("NonExistent", mode).is_err());

            sprite.display_all_layers(mode);
        }
    }

    /// Verifies that new `Frame`s can be added, `Frame` counts increment correctly,
    /// and the initial frame index is zero.
    #[test]
    fn test_frame_initialization_and_addition() {
        let mut sprite = Sprite::new(8, 8);
        assert_eq!(sprite.get_frame_count(), 1);
        assert_eq!(sprite.get_current_frame_index(), 0);

        sprite.add_frame();
        assert_eq!(sprite.get_frame_count(), 2);
    }

    /// Verifies that switching between valid `Frame`s succeeds and
    /// attempting to switch to an out-of-bounds `Frame` returns an `InvalidFrameIndex` error.
    #[test]
    fn test_frame_navigation_and_errors() {
        let mut sprite = Sprite::new(8, 8);
        sprite.add_frame(); // Now has frames 0 and 1

        // Switch to `Frame` 1 successfully
        assert!(sprite.set_current_frame(1).is_ok());
        assert_eq!(sprite.get_current_frame_index(), 1);

        // Attempting to switch to a non-existent `Frame` should fail
        let invalid_switch = sprite.set_current_frame(5);
        assert!(matches!(
            invalid_switch,
            Err(SpriteError::InvalidFrameIndex(5))
        ));
    }

    /// Verifies that pixel changes are isolated to the currently active `Frame`.
    #[test]
    fn test_frame_pixel_isolation() {
        let mut sprite = Sprite::new(4, 4);
        let red = sprite.add_color(Color::new(255, 0, 0, 255)).unwrap();

        // Draw on frame 0
        sprite.set_pixel(0, 0, red);

        // Add and switch to frame 1
        sprite.add_frame();
        sprite.set_current_frame(1).unwrap();

        // `Frame` 1's background layer should still be transparent (Index 0) at (0, 0)
        assert_eq!(pixel_at(&sprite, "Background", 0, 0), Index(0));

        // Switch back to `Frame` 0 and verify the red pixel is still there
        sprite.set_current_frame(0).unwrap();
        assert_eq!(pixel_at(&sprite, "Background", 0, 0), red);
    }

    /// Verifies that every layer has one slot per frame, and that new slots are empty.
    #[test]
    fn test_layers_shared_across_frames() {
        let mut sprite = Sprite::new(4, 4);
        sprite.add_frame();
        sprite.add_layer("Foreground").unwrap();
        sprite.add_frame();

        for layer in &sprite.layers {
            assert_eq!(layer.cels.len(), sprite.frames.len());
            assert!(layer.cels.iter().all(Option::is_none));
        }
    }

    /// Verifies that drawing creates the cel lazily and erasing an empty slot doesn't.
    #[test]
    fn test_cel_created_on_first_write() {
        let mut sprite = Sprite::new(4, 4);
        let red = sprite.add_color(Color::new(255, 0, 0, 255)).unwrap();

        sprite.erase_pixel(0, 0);
        assert!(sprite.get_cel("Background").unwrap().is_none());

        sprite.set_pixel(0, 0, red);
        assert!(sprite.get_cel("Background").unwrap().is_some());
    }

    /// Verifies that a cel grows to include pixels drawn outside its rectangle,
    /// keeping the existing pixels in place.
    #[test]
    fn test_cel_grows_to_include_new_pixels() {
        let mut sprite = Sprite::new(8, 8);
        let red = sprite.add_color(Color::new(255, 0, 0, 255)).unwrap();

        sprite.set_pixel(2, 3, red); // col 2, row 3
        sprite.set_pixel(4, 5, red); // col 4, row 5

        assert_eq!(pixel_at(&sprite, "Background", 2, 3), red);
        assert_eq!(pixel_at(&sprite, "Background", 4, 5), red);
        assert_eq!(pixel_at(&sprite, "Background", 3, 4), Index(0)); // gap inside
        assert_eq!(pixel_at(&sprite, "Background", 0, 0), Index(0)); // outside

        let cel = sprite.get_cel("Background").unwrap().unwrap();
        assert_eq!((cel.x(), cel.y(), cel.width(), cel.height()), (2, 3, 3, 3));
    }

    /// Verifies that growing up and to the left also preserves existing pixels.
    #[test]
    fn test_cel_grows_up_and_left() {
        let mut sprite = Sprite::new(8, 8);
        let red = sprite.add_color(Color::new(255, 0, 0, 255)).unwrap();

        sprite.set_pixel(5, 5, red);
        sprite.set_pixel(1, 2, red); // col 1, row 2

        assert_eq!(pixel_at(&sprite, "Background", 5, 5), red);
        assert_eq!(pixel_at(&sprite, "Background", 1, 2), red);
    }

    /// Verifies that erasing a position outside a cel doesn't grow it.
    #[test]
    fn test_erasing_outside_cel_does_not_grow_it() {
        let mut sprite = Sprite::new(8, 8);
        let red = sprite.add_color(Color::new(255, 0, 0, 255)).unwrap();

        sprite.set_pixel(2, 3, red); // col 2, row 3
        sprite.erase_pixel(0, 0);

        let cel = sprite.get_cel("Background").unwrap().unwrap();
        assert_eq!((cel.x(), cel.y(), cel.width(), cel.height()), (2, 3, 1, 1));
    }

    /// Verifies that trimming shrinks the rectangle after an erase and keeps the remaining pixels.
    #[test]
    fn test_trim_shrinks_to_drawn_pixels() {
        let mut sprite = Sprite::new(8, 8);
        let red = sprite.add_color(Color::new(255, 0, 0, 255)).unwrap();

        sprite.set_pixel(2, 3, red); // col 2, row 3
        sprite.set_pixel(4, 5, red); // col 4, row 5
        sprite.erase_pixel(4, 5); // inside the rectangle, so it stays 3x3
        sprite.trim_active_cel();

        let cel = sprite.get_cel("Background").unwrap().unwrap();
        assert_eq!((cel.x(), cel.y(), cel.width(), cel.height()), (2, 3, 1, 1));
        assert_eq!(pixel_at(&sprite, "Background", 2, 3), red);
    }

    /// Verifies that a cel with nothing left drawn becomes `None` after trimming.
    #[test]
    fn test_trim_empty_cel_becomes_none() {
        let mut sprite = Sprite::new(4, 4);
        let red = sprite.add_color(Color::new(255, 0, 0, 255)).unwrap();

        sprite.set_pixel(0, 0, red);
        sprite.erase_pixel(0, 0);
        sprite.trim_active_cel();

        assert!(sprite.get_cel("Background").unwrap().is_none());
    }

    /// Verifies that removing a layer drops it and its cels, and that the last layer is protected.
    #[test]
    fn test_remove_layer() {
        let mut sprite = Sprite::new(4, 4);
        sprite.add_layer("Foreground").unwrap();

        assert!(sprite.remove_layer("Foreground").is_ok());
        assert!(sprite.get_layer("Foreground").is_err());
        assert_eq!(sprite.layers.len(), 1);

        assert!(matches!(
            sprite.remove_layer("Ghost"),
            Err(LayerError::NotFound(_))
        ));
        assert!(matches!(
            sprite.remove_layer("Background"),
            Err(LayerError::LastLayer)
        ));
    }

    /// Verifies that the active layer keeps pointing at the same layer when one below it is removed,
    /// and falls back to a valid layer when the active one is removed.
    #[test]
    fn test_remove_layer_keeps_active_layer_valid() {
        let mut sprite = Sprite::new(4, 4);
        let red = sprite.add_color(Color::new(255, 0, 0, 255)).unwrap();
        sprite.add_layer("A").unwrap();
        sprite.add_layer("B").unwrap();

        // Active "B" (index 2), remove "A" (index 1): "B" moves down and stays active.
        sprite.set_active_layer("B").unwrap();
        sprite.remove_layer("A").unwrap();
        assert_eq!(sprite.active_layer, 1);
        sprite.set_pixel(0, 0, red);
        assert_eq!(pixel_at(&sprite, "B", 0, 0), red);

        // Remove the active top layer: the one below becomes active.
        sprite.remove_layer("B").unwrap();
        assert_eq!(sprite.active_layer, 0);
    }

    /// Verifies that removing a frame removes the right cels and durations, so the later
    /// frames shift down, and that the last frame is protected.
    #[test]
    fn test_remove_frame() {
        let mut sprite = Sprite::new(4, 4);
        let red = sprite.add_color(Color::new(255, 0, 0, 255)).unwrap();
        let blue = sprite.add_color(Color::new(0, 0, 255, 255)).unwrap();

        sprite.set_pixel(0, 0, red); // frame 0
        sprite.add_frame();
        sprite.set_current_frame(1).unwrap();
        sprite.set_pixel(0, 0, blue); // frame 1
        sprite
            .set_frame_duration(1, Duration::from_millis(250))
            .unwrap();

        sprite.remove_frame(0).unwrap();

        assert_eq!(sprite.get_frame_count(), 1);
        assert_eq!(sprite.layers[0].cels.len(), 1);
        assert_eq!(sprite.frames[0].get_duration(), Duration::from_millis(250));
        assert_eq!(pixel_at(&sprite, "Background", 0, 0), blue);

        assert!(matches!(
            sprite.remove_frame(0),
            Err(SpriteError::LastFrame)
        ));
        assert!(matches!(
            sprite.remove_frame(5),
            Err(SpriteError::InvalidFrameIndex(5))
        ));
    }

    /// Verifies that the current frame stays valid after removing frames before it or at the end.
    #[test]
    fn test_remove_frame_keeps_current_frame_valid() {
        let mut sprite = Sprite::new(4, 4);
        sprite.add_frame();
        sprite.add_frame();

        // On frame 2, remove frame 0: the same frame is now at index 1.
        sprite.set_current_frame(2).unwrap();
        sprite.remove_frame(0).unwrap();
        assert_eq!(sprite.get_current_frame_index(), 1);

        // On the last frame, remove it: clamp to the new last one.
        sprite.remove_frame(1).unwrap();
        assert_eq!(sprite.get_current_frame_index(), 0);
    }
}
