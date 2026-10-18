use std::time::Duration;
use thiserror::Error;

/// Represents a color with red, green, blue and alpha values.
#[derive(Debug, Clone)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Color {
    /// Creates a new color with the provided RGBA values.
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Changes the alpha value.
    pub fn change_alpha(&mut self, alpha: u8) {
        self.alpha = alpha;
    }

    /// Generates a transparent `Color`
    fn transparent() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

/// Index into a `Sprite`'s palette. `Index(0)` is always the transparent entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index(u8);

impl Index {
    /// Returns the internal `u8` value.
    pub fn value(&self) -> u8 {
        self.0
    }
}

/// The pixels of one `Layer` in one `Frame`, stored as a sub-rectangle of the canvas.
///
/// `x` and `y` are the canvas position of the rectangle's top-left corner, and `pixels`
/// holds `width * height` entries in row-major order, in local coordinates.
/// Anything on the canvas outside the rectangle counts as transparent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cel {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    pixels: Vec<Index>,
}

impl Cel {
    /// Creates a 1x1 `Cel` holding a single pixel at the given canvas position.
    fn single(col: u32, row: u32, index: Index) -> Self {
        Self {
            x: col,
            y: row,
            width: 1,
            height: 1,
            pixels: vec![index],
        }
    }

    /// Enlarges the cel's rectangle so it contains the given canvas position.
    /// New area is filled with transparent pixels.
    fn grow_to_include(&mut self, col: u32, row: u32) {
        let new_x = self.x.min(col);
        let new_y = self.y.min(row);
        let new_right = (self.x + self.width).max(col + 1);
        let new_bottom = (self.y + self.height).max(row + 1);
        let new_width = new_right - new_x;
        let new_height = new_bottom - new_y;

        if (new_x, new_y, new_width, new_height) == (self.x, self.y, self.width, self.height) {
            return; // already inside
        }

        let mut new_pixels = vec![Index(0); new_width as usize * new_height as usize];
        let old_width = self.width as usize;
        let x_offset = (self.x - new_x) as usize;
        let y_offset = (self.y - new_y) as usize;

        for ly in 0..self.height as usize {
            let src = ly * old_width;
            let dst = (y_offset + ly) * new_width as usize + x_offset;
            new_pixels[dst..dst + old_width].copy_from_slice(&self.pixels[src..src + old_width]);
        }

        self.x = new_x;
        self.y = new_y;
        self.width = new_width;
        self.height = new_height;
        self.pixels = new_pixels;
    }

    /// Returns whether a canvas position is inside the cel's rectangle.
    fn contains(&self, col: u32, row: u32) -> bool {
        col >= self.x && row >= self.y && col - self.x < self.width && row - self.y < self.height
    }

    /// Returns the `Index` at a canvas position. Outside the cel is transparent.
    fn get(&self, col: u32, row: u32) -> Index {
        if col < self.x || row < self.y {
            return Index(0);
        }
        let (lx, ly) = (col - self.x, row - self.y);
        if lx >= self.width || ly >= self.height {
            return Index(0);
        }
        self.pixels[ly as usize * self.width as usize + lx as usize]
    }

    /// Sets the `Index` at a canvas position, growing the cel if needed.
    fn set(&mut self, col: u32, row: u32, index: Index) {
        self.grow_to_include(col, row);
        let (lx, ly) = (col - self.x, row - self.y);
        self.pixels[ly as usize * self.width as usize + lx as usize] = index;
    }
}

#[derive(Error, Debug)]
pub enum LayerError {
    #[error("Layer {0} not found in the Sprite")]
    NotFound(String),

    #[error("A Layer with this name already exists")]
    NameCollision,
}

/// A named layer, shared by every `Frame` of a `Sprite`.
///
/// `cels[f]` is this layer's `Cel` in frame `f`, or `None` if nothing is drawn there.
/// Its length always equals the number of frames in the `Sprite`.
#[derive(Debug, Clone)]
pub struct Layer {
    name: String,
    is_visible: bool,
    cels: Vec<Option<Cel>>,
}

impl Layer {
    /// Creates a new `Layer`.
    pub fn new(name: &str, frame_count: usize) -> Self {
        Self {
            name: name.to_string(),
            is_visible: true,
            cels: vec![None; frame_count],
        }
    }

    /// Returns the name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Sets the name.
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    /// Sets the visibility.
    pub fn visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }
}

/// Creates the default "Background" layer with a single empty slot.
impl Default for Layer {
    fn default() -> Self {
        Self {
            name: "Background".to_string(),
            is_visible: true,
            cels: vec![None; 1],
        }
    }
}

/// A point in the animation timeline. It holds timing only; the pixels live in the `Cel`s.
#[derive(Debug)]
pub struct Frame {
    duration: Duration,
}

impl Frame {
    /// Creates a `Frame` that stays on screen for the given `Duration`.
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }

    /// Returns how long the Frame stays on screen.
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Sets how long the Frame stays on screen.
    pub fn set_duration(&mut self, duration: std::time::Duration) {
        self.duration = duration;
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new(Duration::from_millis(100))
    }
}

#[derive(Error, Debug)]
pub enum SpriteError {
    #[error("Can't add more than 256 colors to a single palette!")]
    PaletteOverflow,

    #[error("Invalid frame index: {0}")]
    InvalidFrameIndex(usize),
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

impl Sprite {
    /// Creates a new `Sprite`
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            palette: vec![Color::transparent()],
            active_layer: 0,
            frames: vec![Frame::default()],
            current_frame: 0,
            layers: vec![Layer::default()],
        }
    }

    /// Returns the `usize` index of the selected `Layer`
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if `name` does not match a `Layer.name` in `self.layers`
    fn get_layer_index(&self, name: &str) -> Result<usize, LayerError> {
        self.layers
            .iter()
            .position(|layer| layer.name == name)
            .ok_or_else(|| LayerError::NotFound(name.to_string()))
    }

    /// Returns a reference to a `Layer` of the `Sprite`.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if no `Layer` has that name.
    pub fn get_layer(&self, name: &str) -> Result<&Layer, LayerError> {
        Ok(&self.layers[self.get_layer_index(name)?])
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
    pub fn set_pixel(&mut self, row: u32, col: u32, palette_index: Index) {
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
        self.layers[index].visible(visible);
        Ok(())
    }

    /// Resets a pixel to `Index(0)`, making it transparent.
    /// Erasing where no `Cel` exists does nothing.
    pub fn erase_pixel(&mut self, row: u32, col: u32) {
        self.set_pixel(row, col, Index(0));
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

    /// Prints a `Layer`'s palette indexes for the current frame as a grid.
    /// An empty slot prints as zeros.
    fn display_layer_data(&self, layer: &Layer) {
        let cel = layer.cels[self.current_frame].as_ref();
        println!("Layer: '{}' (Visible: {})", layer.name, layer.is_visible);
        for row in 0..self.height {
            for col in 0..self.width {
                let value = cel.map_or(0, |cel| cel.get(col, row).value());
                print!("{value} ");
            }
            println!();
        }
        println!();
    }

    /// Prints one `Layer`'s grid for the current frame.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if no `Layer` has that name.
    pub fn display_layer(&self, name: &str) -> Result<(), LayerError> {
        let layer = self.get_layer(name)?;
        self.display_layer_data(layer);
        Ok(())
    }

    /// Prints every `Layer`'s grid for the current frame, bottom layer first.
    pub fn display_all_layers(&self) {
        for layer in &self.layers {
            self.display_layer_data(layer);
        }
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

    /// Verifies that colors can be instantiated with custom RGBA values,
    /// their alpha channel can be modified, and transparent
    /// defaults are initialized with an alpha of zero.
    #[test]
    fn test_color_creation_and_alpha() {
        let mut color = Color::new(100, 150, 200, 255);
        assert_eq!(color.alpha, 255);

        color.change_alpha(128);
        assert_eq!(color.alpha, 128);

        assert_eq!(Color::transparent().alpha, 0);
    }

    /// Verifies that the `Index` wrapper type correctly holds a `u8` value
    /// and that its `.value()` method returns the expected integer.
    #[test]
    fn test_index_value() {
        let idx = Index(5);
        assert_eq!(idx.value(), 5);
    }

    /// Verifies that layer names can be correctly retrieved and updated,
    /// and that visibility toggles properly modify the layer's state.
    #[test]
    fn test_layer_getters_and_setters() {
        let mut layer = Layer::new("TestLayer", 1);
        assert_eq!(layer.get_name(), "TestLayer");

        assert!(layer.cels == vec![None]);

        layer.set_name("RenamedLayer");
        assert_eq!(layer.get_name(), "RenamedLayer");

        layer.visible(false);
        assert!(!layer.is_visible);
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

        // Attempting to add a duplicate layer name should cause a NameCollision
        let duplicate_result = sprite.add_layer("Foreground");
        assert!(matches!(duplicate_result, Err(LayerError::NameCollision)));

        // Requesting a non-existent layer should return NotFound
        let missing_result = sprite.get_layer("GhostLayer");
        assert!(matches!(missing_result, Err(LayerError::NotFound(_))));
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

    /// Verifies that the display and printing helper functions execute cleanly
    /// without panicking, correctly succeeding for existing layers and returning
    /// an error for non-existent layers.
    #[test]
    fn test_display_functions_dont_panic() {
        let mut sprite = Sprite::new(2, 2);
        let blue = sprite.add_color(Color::new(0, 0, 255, 255)).unwrap();
        sprite.set_pixel(0, 0, blue);

        // Ensure display methods run cleanly without throwing errors
        assert!(sprite.display_layer("Background").is_ok());
        assert!(sprite.display_layer("NonExistent").is_err());

        sprite.display_all_layers();
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

    /// Verifies that new frames start with the default duration and that it can be changed.
    #[test]
    fn test_frame_duration() {
        let mut frame = Frame::default();
        assert_eq!(frame.duration(), Duration::from_millis(100));

        frame.set_duration(Duration::from_millis(250));
        assert_eq!(frame.duration(), Duration::from_millis(250));
    }

    /// Verifies that a cel grows to include pixels drawn outside its rectangle,
    /// keeping the existing pixels in place.
    #[test]
    fn test_cel_grows_to_include_new_pixels() {
        let mut sprite = Sprite::new(8, 8);
        let red = sprite.add_color(Color::new(255, 0, 0, 255)).unwrap();

        sprite.set_pixel(3, 2, red); // row 3, col 2
        sprite.set_pixel(5, 4, red); // row 5, col 4

        assert_eq!(pixel_at(&sprite, "Background", 2, 3), red);
        assert_eq!(pixel_at(&sprite, "Background", 4, 5), red);
        assert_eq!(pixel_at(&sprite, "Background", 3, 4), Index(0)); // gap inside
        assert_eq!(pixel_at(&sprite, "Background", 0, 0), Index(0)); // outside

        let cel = sprite.get_cel("Background").unwrap().unwrap();
        assert_eq!((cel.x, cel.y, cel.width, cel.height), (2, 3, 3, 3));
    }

    /// Verifies that growing up and to the left also preserves existing pixels.
    #[test]
    fn test_cel_grows_up_and_left() {
        let mut sprite = Sprite::new(8, 8);
        let red = sprite.add_color(Color::new(255, 0, 0, 255)).unwrap();

        sprite.set_pixel(5, 5, red);
        sprite.set_pixel(2, 1, red);

        assert_eq!(pixel_at(&sprite, "Background", 5, 5), red);
        assert_eq!(pixel_at(&sprite, "Background", 1, 2), red);
    }
}
