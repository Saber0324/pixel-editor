use pixel_editor::sprite::Color;
use pixel_editor::sprite::Sprite;

fn main() {
    let blue = Color::new(0, 0, 255, 255);
    let red = Color::new(255, 0, 0, 255);

    let mut sprite = Sprite::new(16, 16);

    let blue = sprite
        .add_color(blue)
        .expect("Should have less than 256 colors");
    let red = sprite
        .add_color(red)
        .expect("Should have less than 256 colors");

    sprite.set_pixel(0, 2, red);
    sprite.set_pixel(1, 2, blue);
    sprite.set_pixel(2, 2, blue);
    sprite.set_pixel(3, 2, blue);
    sprite.set_pixel(4, 2, blue);
    sprite.set_pixel(4, 3, blue);
    sprite.set_pixel(4, 1, blue);

    sprite
        .add_layer("Foreground")
        .expect("Name should be available");

    sprite
        .set_active_layer("Foreground")
        .expect("Foreground Layer should exist");

    sprite.set_pixel(0, 0, blue);
    sprite.set_pixel(0, 1, blue);
    sprite.set_pixel(0, 2, blue);
    sprite.set_pixel(0, 3, blue);
    sprite.set_pixel(0, 4, blue);

    sprite.set_pixel(1, 0, blue);
    sprite.set_pixel(1, 4, blue);
    sprite.set_pixel(2, 0, blue);
    sprite.set_pixel(2, 4, blue);
    sprite.set_pixel(3, 0, blue);
    sprite.set_pixel(3, 4, blue);

    sprite.set_pixel(4, 0, blue);
    sprite.set_pixel(4, 1, blue);
    sprite.set_pixel(4, 2, blue);
    sprite.set_pixel(4, 3, blue);
    sprite.set_pixel(4, 4, blue);

    sprite.display_all_layers();
}
