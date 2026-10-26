/// Represents a color with red, green, blue and alpha values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Color {
    /// Creates a new color with the provided RGBA values.
    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
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

    /// A completely transparent `Color`.
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);

    /// Draws self over below while keeping in mind the alpha values.
    #[must_use]
    pub fn blend(self, below: Self) -> Self {
        let sa = u32::from(self.alpha);
        if sa == 0 {
            return below;
        }
        if sa == 255 {
            return self;
        }

        let da = u32::from(below.alpha);
        let out_a = sa + (da * (255 - sa) + 127) / 255;

        let channel = |s: u8, d: u8| -> u8 {
            let num = u32::from(s) * sa * 255 + u32::from(d) * da * (255 - sa);
            ((num + out_a * 255 / 2) / (out_a * 255)).min(255) as u8
        };

        Self::new(
            channel(self.red, below.red),
            channel(self.green, below.green),
            channel(self.blue, below.blue),
            u8::try_from(out_a).expect("Should not wrap"),
        )
    }
}

/// Index into a `Sprite`'s `Sprite.palette`. `Index(0)` is always the transparent entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index(pub(crate) u8);

impl Index {
    /// Returns the internal `u8` value.
    pub fn value(&self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Verifies that colors can be instantiated with custom RGBA values,
    /// their alpha channel can be modified, and transparent
    /// defaults are initialized with an alpha of zero.
    #[test]
    fn test_color_creation_and_alpha() {
        let mut color = Color::new(100, 150, 200, 255);
        assert_eq!(color.alpha, 255);

        color.change_alpha(128);
        assert_eq!(color.alpha, 128);

        assert_eq!(Color::TRANSPARENT.alpha, 0);
    }

    /// Verifies that the `Index` wrapper type correctly holds a `u8` value
    /// and that its `.value()` method returns the expected integer.
    #[test]
    fn test_index_value() {
        let idx = Index(5);
        assert_eq!(idx.value(), 5);
    }
}
