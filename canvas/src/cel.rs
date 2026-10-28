use crate::color::Index;

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
    pub(crate) fn single(col: u32, row: u32, index: Index) -> Self {
        Self {
            x: col,
            y: row,
            width: 1,
            height: 1,
            pixels: vec![index],
        }
    }

    // Cel
    /// Enlarges the rectangle so it covers the given canvas rectangle (inclusive corners).
    fn grow_to_cover(&mut self, left: u32, top: u32, right: u32, bottom: u32) {
        let new_x = self.x.min(left);
        let new_y = self.y.min(top);
        let new_right = (self.x + self.width).max(right + 1);
        let new_bottom = (self.y + self.height).max(bottom + 1);
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

    /// Enlarges the cel's rectangle so it contains the given canvas position.
    /// New area is filled with transparent pixels.
    pub(crate) fn grow_to_include(&mut self, col: u32, row: u32) {
        self.grow_to_cover(col, row, col, row);
    }

    /// Fills the inclusive canvas rectangle, growing the cel at most once.
    /// Writing transparent pixels never grows it, so only the part inside the rectangle is cleared.
    pub(crate) fn set_rect(&mut self, left: u32, top: u32, right: u32, bottom: u32, index: Index) {
        let (mut left, mut top, mut right, mut bottom) = (left, top, right, bottom);
        if index == Index(0) {
            left = left.max(self.x);
            top = top.max(self.y);
            right = right.min(self.x + self.width - 1);
            bottom = bottom.min(self.y + self.height - 1);
            if left > right || top > bottom {
                return;
            }
        } else {
            self.grow_to_cover(left, top, right, bottom);
        }
        let width = self.width as usize;
        let len = (right - left + 1) as usize;
        for row in top..=bottom {
            let start = (row - self.y) as usize * width + (left - self.x) as usize;
            self.pixels[start..start + len].fill(index);
        }
    }

    /// Returns whether a canvas position is inside the cel's rectangle.
    pub(crate) fn contains(&self, col: u32, row: u32) -> bool {
        col >= self.x && row >= self.y && col - self.x < self.width && row - self.y < self.height
    }

    /// Returns the `Index` at a canvas position. Outside the cel is transparent.
    pub(crate) fn get(&self, col: u32, row: u32) -> Index {
        if !self.contains(col, row) {
            return Index(0);
        }
        let (lx, ly) = (col - self.x, row - self.y);
        self.pixels[ly as usize * self.width as usize + lx as usize]
    }

    /// Sets the `Index` at a canvas position, growing the cel if needed.
    /// Writing a transparent pixel outside the rectangle does nothing.
    pub(crate) fn set(&mut self, col: u32, row: u32, index: Index) {
        if index == Index(0) && !self.contains(col, row) {
            return;
        }
        self.grow_to_include(col, row);
        let (lx, ly) = (col - self.x, row - self.y);
        self.pixels[ly as usize * self.width as usize + lx as usize] = index;
    }

    /// Shrinks the rectangle to the bounding box of the non-transparent pixels.
    /// Returns `false` if every pixel is transparent, leaving the cel unchanged.
    pub(crate) fn trim(&mut self) -> bool {
        let mut bounds: Option<(u32, u32, u32, u32)> = None; // min_x, min_y, max_x, max_y
        for ly in 0..self.height {
            for lx in 0..self.width {
                if self.pixels[ly as usize * self.width as usize + lx as usize] == Index(0) {
                    continue;
                }
                bounds = Some(match bounds {
                    None => (lx, ly, lx, ly),
                    Some((x0, y0, x1, y1)) => (x0.min(lx), y0.min(ly), x1.max(lx), y1.max(ly)),
                });
            }
        }
        let Some((x0, y0, x1, y1)) = bounds else {
            return false;
        };

        let new_width = x1 - x0 + 1;
        let new_height = y1 - y0 + 1;
        let mut new_pixels = Vec::with_capacity(new_width as usize * new_height as usize);
        for ly in y0..=y1 {
            let start = ly as usize * self.width as usize + x0 as usize;
            new_pixels.extend_from_slice(&self.pixels[start..start + new_width as usize]);
        }

        self.x += x0;
        self.y += y0;
        self.width = new_width;
        self.height = new_height;
        self.pixels = new_pixels;
        true
    }
}

// Getters
impl Cel {
    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn iter_pixels(&self) -> impl Iterator<Item = (u32, u32, Index)> + '_ {
        self.pixels.iter().enumerate().map(move |(i, &index)| {
            let i = u32::try_from(i).expect("Should not overflow");
            (self.x + i % self.width, self.y + i / self.width, index)
        })
    }
}
