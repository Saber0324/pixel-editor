use pixel_editor::colors::{Color, Sprite};

fn main() {
    let blue: Color = Color::new(0, 0, 255, 255);
    let red: Color = Color::new(255, 0, 0, 255);

    let mut sprite: Sprite = Sprite::new(5, 5);
    let blue: u8 = sprite.add_color(blue).unwrap();
    let red: u8 = sprite.add_color(red).unwrap();

    for y in 0..5 {
        for x in (0..5).step_by(2) {
            sprite.set_pixel(x, y, red);
            if x + 1 < 5 {
                sprite.set_pixel(x + 1, y, blue);
            }
        }
    }

    sprite.display_grid();
}
