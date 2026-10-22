use std::{ops::RangeInclusive, str::FromStr};

use pixel_editor::sprite::{Color, DisplayMode};
use thiserror::Error;

#[derive(Debug)]
pub enum Item {
    Pixel { col: u32, row: u32, index: u8 },
    Layer(String),
    Frame(usize),
    Color(Color),
}

#[derive(Debug)]
pub enum Command {
    Set(Item),
    Add(Item),
    AddFrame,
    DeletePixel {
        col: u32,
        row: u32,
    },
    DeleteLayer(String),
    DeleteFrame(usize),
    Display(DisplayMode),
    DisplayAll(DisplayMode),
    RenderFrame,
    RenderAnimation,
    Clear,
    SetRow {
        row: u32,
        cols: RangeInclusive<u32>,
        index: u8,
    },
    SetCol {
        col: u32,
        rows: RangeInclusive<u32>,
        index: u8,
    },
    SetRect {
        cols: RangeInclusive<u32>,
        rows: RangeInclusive<u32>,
        index: u8,
    },
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Empty statement, expected option")]
    Empty,

    #[error("Unknown token: {0}")]
    Unknown(String),

    #[error("{0}")]
    Usage(&'static str),

    #[error("Unexpected {0}, expected: {1}")]
    Unexpected(String, String),
}

const SET_USAGE: &str = "\
usage:
  set pixel|px <col> <row> <index>
  set row      <row> <from> <to> <index>
  set col      <col> <from> <to> <index>
  set rect     <left> <top> <right> <bottom> <index>
  set layer    <name>
  set frame    <n>
(ranges include both ends)";

const ADD_USAGE: &str = "\
usage:
  add layer <name>
  add frame
  add color <r> <g> <b> <a>";

const DELETE_USAGE: &str = "\
usage:
  delete pixel|px <col> <row>
  delete layer    <name>
  delete frame    <n>";

const DISPLAY_USAGE: &str = "\
usage:
  display canvas|cel
  display all canvas|cel";

const RENDER_USAGE: &str = "\
usage:
  render [frame]
  render animation";

const CLEAR_USAGE: &str = "usage: clear";

fn num<T: FromStr>(token: &str) -> Result<T, ParseError> {
    token
        .parse()
        .map_err(|_| ParseError::Unexpected(token.to_string(), "num".to_string()))
}

/// Parses two inclusive ends into a range, rejecting a start past the end.
fn span(from: &str, to: &str) -> Result<RangeInclusive<u32>, ParseError> {
    let (from, to): (u32, u32) = (num(from)?, num(to)?);
    if from > to {
        return Err(ParseError::Usage(
            "a range start must not be greater than its end",
        ));
    }
    Ok(from..=to)
}

/// Parsers input to `Command`
pub fn parse_command(line: &str) -> Result<Command, ParseError> {
    let tokens: Vec<&str> = line.split_whitespace().collect();

    match tokens.as_slice() {
        [] => Err(ParseError::Empty),

        // Setters
        ["set", "pixel" | "px", col, row, index] => Ok(Command::Set(Item::Pixel {
            col: num(col)?,
            row: num(row)?,
            index: num(index)?,
        })),

        ["set", "layer", name] => Ok(Command::Set(Item::Layer(name.to_string()))),

        ["set", "frame", number] => Ok(Command::Set(Item::Frame(num(number)?))),

        ["set", "row", row, from, to, index] => Ok(Command::SetRow {
            row: num(row)?,
            cols: span(from, to)?,
            index: num(index)?,
        }),
        ["set", "col", col, from, to, index] => Ok(Command::SetCol {
            col: num(col)?,
            rows: span(from, to)?,
            index: num(index)?,
        }),
        ["set", "rect", left, top, right, bottom, index] => Ok(Command::SetRect {
            cols: span(left, right)?,
            rows: span(top, bottom)?,
            index: num(index)?,
        }),
        // Adders
        ["add", "layer", name] => Ok(Command::Add(Item::Layer(name.to_string()))),

        ["add", "frame"] => Ok(Command::AddFrame),

        ["add", "color", r, g, b, a] => Ok(Command::Add(Item::Color(Color::new(
            num(r)?,
            num(g)?,
            num(b)?,
            num(a)?,
        )))),

        // Deleters
        ["delete", "pixel" | "px", col, row] => Ok(Command::DeletePixel {
            col: num(col)?,
            row: num(row)?,
        }),

        ["delete", "layer", name] => Ok(Command::DeleteLayer(name.to_string())),

        ["delete", "frame", number] => Ok(Command::DeleteFrame(num(number)?)),

        // Displaying and rendering
        ["display", "canvas"] => Ok(Command::Display(DisplayMode::Canvas)),

        ["display", "cel"] => Ok(Command::Display(DisplayMode::Cel)),

        ["display", "all", "canvas"] => Ok(Command::DisplayAll(DisplayMode::Canvas)),

        ["display", "all", "cel"] => Ok(Command::DisplayAll(DisplayMode::Cel)),

        ["render"] | ["render", "frame"] => Ok(Command::RenderFrame),

        ["render", "animation"] => Ok(Command::RenderAnimation),

        // Clearing the screen
        ["clear"] => Ok(Command::Clear),

        // Errors
        ["set", ..] => Err(ParseError::Usage(SET_USAGE)),

        ["add", ..] => Err(ParseError::Usage(ADD_USAGE)),

        ["delete", ..] => Err(ParseError::Usage(DELETE_USAGE)),

        ["display", ..] => Err(ParseError::Usage(DISPLAY_USAGE)),

        ["render", ..] => Err(ParseError::Usage(RENDER_USAGE)),

        ["clear", ..] => Err(ParseError::Usage(CLEAR_USAGE)),

        [other, ..] => Err(ParseError::Unknown((*other).to_string())),
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn set_pixel_is_col_then_row() {
        let command = parse_command("set px 3 1 2").unwrap();
        assert!(matches!(
            command,
            Command::Set(Item::Pixel {
                col: 3,
                row: 1,
                index: 2
            })
        ));
    }

    #[test]
    fn pixel_has_two_spellings() {
        assert!(parse_command("set pixel 1 2 3").is_ok());
        assert!(parse_command("delete pixel 1 2").is_ok());
    }

    #[test]
    fn add_color_is_an_add() {
        let command = parse_command("add color 255 0 0 255").unwrap();
        assert!(matches!(command, Command::Add(Item::Color(_))));
    }

    #[test]
    fn delete_variants() {
        assert!(matches!(
            parse_command("delete px 4 5").unwrap(),
            Command::DeletePixel { col: 4, row: 5 }
        ));
        assert!(matches!(
            parse_command("delete layer Shine").unwrap(),
            Command::DeleteLayer(name) if name == "Shine"
        ));
        assert!(matches!(
            parse_command("delete frame 2").unwrap(),
            Command::DeleteFrame(2)
        ));
    }

    #[test]
    fn wrong_argument_count_reports_usage() {
        assert!(matches!(
            parse_command("set px 1"),
            Err(ParseError::Usage(_))
        ));
        assert!(matches!(
            parse_command("set px 1 2 3 4"),
            Err(ParseError::Usage(_))
        ));
        assert!(matches!(
            parse_command("add frame extra"),
            Err(ParseError::Usage(_))
        ));
        assert!(matches!(
            parse_command("delete px 1"),
            Err(ParseError::Usage(_))
        ));
    }

    #[test]
    fn non_numbers_are_reported() {
        assert!(matches!(
            parse_command("set px a 1 2"),
            Err(ParseError::Unexpected(_, _))
        ));
        assert!(matches!(
            parse_command("add color 300 0 0 255"),
            Err(ParseError::Unexpected(_, _))
        ));
    }

    #[test]
    fn empty_and_unknown_input() {
        assert!(matches!(parse_command(""), Err(ParseError::Empty)));
        assert!(matches!(parse_command("   "), Err(ParseError::Empty)));
        assert!(matches!(parse_command("nope"), Err(ParseError::Unknown(_))));
    }
}
