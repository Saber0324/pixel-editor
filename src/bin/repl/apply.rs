use crate::parse::{Command, Item};
use pixel_editor::{
    renderer::{self, RendererError},
    sprite::{LayerError, Sprite, SpriteError},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplyError {
    #[error("Unsupported command")]
    Unsupported,

    #[error(transparent)]
    Layer(#[from] LayerError),

    #[error(transparent)]
    Sprite(#[from] SpriteError),

    #[error("Rendering error: {0}")]
    Render(#[from] RendererError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn apply(cmd: Command, sprite: &mut Sprite) -> Result<(), ApplyError> {
    match cmd {
        // Setters
        Command::Set(Item::Pixel { col, row, index }) => {
            let index = sprite.palette_index(index)?;
            sprite.set_pixel(col, row, index);
        }
        Command::Set(Item::Layer(name)) => sprite.set_active_layer(&name)?,
        Command::Set(Item::Frame(number)) => sprite.set_current_frame(number)?,

        Command::SetRow { row, cols, index } => {
            let index = sprite.palette_index(index)?;
            sprite.set_row(row, cols, index);
        }
        Command::SetCol { col, rows, index } => {
            let index = sprite.palette_index(index)?;
            sprite.set_col(col, rows, index);
        }
        Command::SetRect { cols, rows, index } => {
            let index = sprite.palette_index(index)?;
            sprite.set_rect(cols, rows, index);
        }

        // Adders
        Command::Add(Item::Layer(name)) => sprite.add_layer(&name)?,
        Command::AddFrame => sprite.add_frame(),
        Command::Add(Item::Color(color)) => {
            let index = sprite.add_color(color)?;
            println!("Color added at index {}", index.value());
        }

        // Valid parses that mean nothing, listed so a new `Item` is a compile error here
        Command::Add(Item::Pixel { .. } | Item::Frame(_)) | Command::Set(Item::Color(_)) => {
            return Err(ApplyError::Unsupported);
        }

        // Deleters
        Command::DeletePixel { col, row } => sprite.erase_pixel(col, row),
        Command::DeleteLayer(name) => sprite.remove_layer(&name)?,
        Command::DeleteFrame(number) => sprite.remove_frame(number)?,

        // Displaying and rendering
        Command::Display(mode) => sprite.display_active_layer(mode)?,
        Command::DisplayAll(mode) => sprite.display_all_layers(mode),
        Command::RenderFrame => {
            renderer::render_frame_to_terminal(sprite, sprite.get_current_frame_index())?;
        }
        Command::RenderAnimation => renderer::play_animation_terminal(sprite, None)?,

        Command::Clear => {
            print!("\x1B[2J\x1B[H");
            std::io::Write::flush(&mut std::io::stdout())?;
        }
    }
    Ok(())
}
