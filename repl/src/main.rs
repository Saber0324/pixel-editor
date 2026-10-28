use rustyline::{DefaultEditor, error::ReadlineError};

use crate::parse::ParseError;
use canvas::Sprite;

mod apply;
mod parse;

/// Asks for a positive whole number. Returns `None` if the user quits with Ctrl-C or Ctrl-D.
fn ask_dimension(editor: &mut DefaultEditor, name: &str) -> rustyline::Result<Option<u32>> {
    loop {
        match editor.readline(&format!("{name}: ")) {
            Ok(line) => match line.trim().parse::<u32>() {
                Ok(value) if value > 0 => return Ok(Some(value)),
                _ => eprintln!("error: {name} must be a whole number greater than 0"),
            },
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => return Ok(None),
            Err(error) => return Err(error),
        }
    }
}

fn main() -> rustyline::Result<()> {
    let mut editor = DefaultEditor::new()?;

    let Some(width) = ask_dimension(&mut editor, "Insert *width* for your sprite")? else {
        return Ok(());
    };
    let Some(height) = ask_dimension(&mut editor, "Insert *height* for your sprite")? else {
        return Ok(());
    };

    let mut sprite = Sprite::new(width, height);
    let _ = editor.load_history(".pixel_history");

    loop {
        match editor.readline("> ") {
            Ok(line) => {
                let line = line.trim();
                if matches!(line, "quit" | "exit") {
                    break;
                }
                if line.is_empty() {
                    continue;
                }
                editor.add_history_entry(line)?;

                match parse::parse_command(line) {
                    Ok(command) => {
                        if let Err(error) = apply::apply(command, &mut sprite) {
                            eprintln!("error: {error}");
                        }
                    }
                    Err(ParseError::Usage(usage)) => eprintln!("{usage}"),
                    Err(error) => eprintln!("error: {error}"),
                }
            }
            Err(ReadlineError::Interrupted) => println!("(Ctrl-D or `quit` to exit)"),
            Err(ReadlineError::Eof) => break,
            Err(error) => return Err(error),
        }
    }

    editor.save_history(".pixel_history")?;
    Ok(())
}
