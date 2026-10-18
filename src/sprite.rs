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

/// Safe wrapper around u8 to represent a pixel's `Color`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index(u8);

impl Index {
    /// Returns the internal `u8` value.
    pub fn value(&self) -> u8 {
        self.0
    }
}

#[derive(Debug)]
pub struct Cel {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    pixels: Vec<Index>,
}

#[derive(Error, Debug)]
pub enum LayerError {
    #[error("Layer {0} not found in the Sprite")]
    NotFound(String),

    #[error("A Layer with this name already exists")]
    NameCollision,
}

#[derive(Debug, Clone)]
pub struct Layer {
    name: String,
    is_visible: bool,
    pixels: Vec<Index>,
}

impl Layer {
    /// Creates a new `Layer`.
    pub fn new(name: &str, width: u32, height: u32) -> Self {
        let total_pixels = width as usize * height as usize;
        let pixels = vec![Index(0); total_pixels];
        Self {
            name: name.to_string(),
            is_visible: true,
            pixels,
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

#[derive(Debug)]
pub struct Frame {
    pub layers: Vec<Layer>,
}

impl Frame {
    /// Creates a new `Frame`.
    fn new(width: u32, height: u32) -> Self {
        Self {
            layers: vec![Layer::new("Background", width, height)],
        }
    }
}

#[derive(Error, Debug)]
pub enum SpriteError {
    #[error("Can't add more than 255 colors to a single palette!")]
    PaletteOverflow,

    #[error("Couldn't find the active layer")]
    MissingActiveLayer,

    #[error("Invalid column position: {0}")]
    InvalidColumn(u32),

    #[error("Invalid row position: {0}")]
    InvalidRow(u32),

    #[error("Invalid frame index: {0}")]
    InvalidFrameIndex(usize),
}

#[derive(Debug)]
pub struct Sprite {
    width: u32,
    height: u32,
    palette: Vec<Color>,
    frames: Vec<Frame>,
    current_frame: usize,
    active_layer: String,
}

impl Sprite {
    /// Creates a new `Sprite`
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            palette: vec![Color::transparent()],
            active_layer: "Background".to_string(),
            frames: vec![Frame::new(width, height)],
            current_frame: 0,
        }
    }

    /// Returns a reference to a `Layer` within the current frame.
    ///
    /// # errors
    ///
    /// Returns `LayerError::NotFound` if the `Layer` is not found in the current `Frame`.
    pub fn get_layer(&self, name: &str) -> Result<&Layer, LayerError> {
        let current_frame = &self.frames[self.current_frame];
        if let Some(layer) = current_frame.layers.iter().find(|layer| layer.name == name) {
            Ok(layer)
        } else {
            Err(LayerError::NotFound(name.to_string()))
        }
    }

    /// Returns a mutable reference to a `Layer` within the current frame.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if the `Layer` is not found in the current `Frame`.
    pub fn get_mut_layer(&mut self, name: &str) -> Result<&mut Layer, LayerError> {
        let current_frame = &mut self.frames[self.current_frame];
        if let Some(layer) = current_frame
            .layers
            .iter_mut()
            .find(|layer| layer.name == name)
        {
            Ok(layer)
        } else {
            Err(LayerError::NotFound(name.to_string()))
        }
    }

    /// Adds a `Layer` to the current frame.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NameCollision` if there's already a `Layer`
    /// with the same name.
    pub fn add_layer(&mut self, name: &str) -> Result<(), LayerError> {
        if self.get_layer(name).is_ok() {
            Err(LayerError::NameCollision)
        } else {
            let layer = Layer::new(name, self.width, self.height);
            let current_frame = &mut self.frames[self.current_frame];
            current_frame.layers.push(layer);
            Ok(())
        }
    }

    /// Returns the index of the provided row and column.
    fn get_index(&self, row: u32, col: u32) -> usize {
        let width = self.width as usize;
        let row = row as usize;
        let col = col as usize;

        (row * width) + col
    }

    /// Changes the active `Layer` to another one.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if the `Layer` is not found in the current `Frame`.
    pub fn set_active_layer(&mut self, name: &str) -> Result<(), LayerError> {
        if self.active_layer == name {
            Ok(())
        } else if self.get_layer(name).is_ok() {
            self.active_layer = name.to_string();
            Ok(())
        } else {
            Err(LayerError::NotFound(name.to_string()))
        }
    }

    /// Sets the pixel at the provided row and column to the `Index` of the provided color.
    pub fn set_pixel(&mut self, row: u32, col: u32, palette_index: Index) {
        if row < self.height && col < self.width {
            let idx = self.get_index(row, col);
            let active_name = self.active_layer.clone();
            if let Ok(active_layer) = self.get_mut_layer(&active_name) {
                active_layer.pixels[idx] = palette_index;
            }
        }
    }

    /// Resets a pixel to `Index` of 0, making it transparent.
    pub fn erase_pixel(&mut self, row: u32, col: u32) {
        self.set_pixel(row, col, Index(0));
    }

    /// Adds a color to the palette.
    ///
    /// # Errors
    ///
    /// Returns `SpriteError::PaletteOverflow` if the palette has more than 256 colors.
    // This migh have to be changed later.
    pub fn add_color(&mut self, color: Color) -> Result<Index, SpriteError> {
        if self.palette.len() >= 256 {
            Err(SpriteError::PaletteOverflow)
        } else {
            let idx = u8::try_from(self.palette.len()).expect("Should be always a valid u8");
            self.palette.push(color);
            Ok(Index(idx))
        }
    }

    /// Displays all the pixels of the selected `Layer` in a grid.
    fn display_layer_data(&self, layer: &Layer) {
        println!("Layer: '{}' (Visible: {})", layer.name, layer.is_visible);
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let palette_index = layer.pixels[idx].value();
                print!("{palette_index} ");
            }
            println!();
        }
        println!();
    }

    /// Displays all the pixels of the selected `Layer` in a grid.
    ///
    /// # Errors
    ///
    /// Returns `LayerError::NotFound` if the `Layer` is not found in the current `Frame`.
    pub fn display_layer(&self, name: &str) -> Result<(), LayerError> {
        let layer = self.get_layer(name)?;
        self.display_layer_data(layer);
        Ok(())
    }

    /// Displays all the pixels of all the `Layer`s in the current `Frame` as a sequence of grids.
    pub fn display_all_layers(&self) {
        let current_frame = &self.frames[self.current_frame];
        for layer in &current_frame.layers {
            self.display_layer_data(layer);
        }
    }

    /// Adds a `Frame` to the `Sprite`.
    pub fn add_frame(&mut self) {
        let new_frame = Frame::new(self.width, self.height);
        self.frames.push(new_frame);
    }

    /// Switches the active `Frame` index, updating the active `Layer` fallback if necessary.
    pub fn set_current_frame(&mut self, index: usize) -> Result<(), SpriteError> {
        if index < self.frames.len() {
            self.current_frame = index;
            let target_frame = &self.frames[index];
            if !target_frame
                .layers
                .iter()
                .any(|l| l.name == self.active_layer)
                && let Some(first_layer) = target_frame.layers.first()
            {
                self.active_layer = first_layer.name.clone();
            }
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

        let transparent_color = Color::transparent();
        assert_eq!(transparent_color.alpha, 0);
    }

    /// Verifies that the `Index` wrapper type correctly holds a `u8` value
    /// and that its `.value()` method returns the expected integer.
    #[test]
    fn test_index_value() {
        // yes I did this just for coverage
        let idx = Index(5);
        assert_eq!(idx.value(), 5);
    }

    /// Verifies that layer names can be correctly retrieved and updated,
    /// and that visibility toggles properly modify the layer's state.
    #[test]
    fn test_layer_getters_and_setters() {
        let mut layer = Layer::new("TestLayer", 4, 4);
        assert_eq!(layer.get_name(), "TestLayer");

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

        {
            let layer = sprite.get_layer("Background").unwrap();
            let idx = sprite.get_index(1, 1);
            assert_eq!(layer.pixels[idx], red);
        }

        // Erase the pixel
        sprite.erase_pixel(1, 1);

        // Check it again
        let layer = sprite.get_layer("Background").unwrap();
        let idx = sprite.get_index(1, 1);
        assert_eq!(layer.pixels[idx], Index(0));

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
        {
            let layer = sprite.get_layer("Background").unwrap();
            let idx = sprite.get_index(0, 0);
            assert_eq!(layer.pixels[idx], Index(0));
        }

        // Switch back to `Frame` 0 and verify the red pixel is still there
        sprite.set_current_frame(0).unwrap();
        {
            let layer = sprite.get_layer("Background").unwrap();
            let idx = sprite.get_index(0, 0);
            assert_eq!(layer.pixels[idx], red);
        }
    }
}
