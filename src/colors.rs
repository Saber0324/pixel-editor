#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    alpha: u8,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
    pub fn change_alpha(&mut self, alpha: u8) {
        self.alpha = alpha;
    }
    pub fn transparent() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

pub struct Sprite {
    width: u16,
    height: u16,
    palette: Vec<Color>,
    pixels: Vec<u8>,
}

impl Sprite {
    pub fn new(width: u16, height: u16) -> Self {
        let total_pixels = width as usize * height as usize;
        Self {
            width,
            height,
            palette: vec![Color::transparent()],
            pixels: vec![0; total_pixels],
        }
    }
    fn get_index(&self, x: u16, y: u16) -> usize {
        y as usize * self.width as usize + x as usize
        // basically: (y * width) + x
    }
    pub fn set_pixel(&mut self, x: u16, y: u16, palette_index: u8) {
        if x < self.width && y < self.height {
            let pixel_index = self.get_index(x, y);
            self.pixels[pixel_index] = palette_index;
        }
    }
    pub fn add_color(&mut self, color: Color) -> Result<u8, &'static str> {
        let index = u8::try_from(self.palette.len()).map_err(|_| "Palette is full")?;
        self.palette.push(color);
        Ok(index)
    }

    pub fn edit_palette_index(&mut self, index: usize, color: Color) {
        self.palette[index] = color;
    }
    pub fn display_grid(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel_index = self.get_index(x, y);
                print!("{} ", self.pixels[pixel_index]);
            }
            println!();
        }
    }
}
