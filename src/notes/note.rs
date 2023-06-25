use std::fmt::Display;

use super::{long::LongNote, short::ShortNote};
use anyhow::Result;
use colored::{ColoredString, Colorize};
use inquire::Select;
use ptree::TreeItem;
use serde;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum Note {
    Short(ShortNote),
    Long(LongNote),
}

impl TreeItem for Note {
    type Child = Self;
    fn write_self<W: std::io::Write>(
        &self,
        f: &mut W,
        style: &ptree::Style,
    ) -> std::io::Result<()> {
        match *self {
            Self::Long(ref note) => note.write_self(f, style),
            Self::Short(ref note) => note.write_self(f, style),
        }
    }

    fn children(&self) -> std::borrow::Cow<[Self::Child]> {
        match *self {
            Self::Long(ref note) => note.children(),
            Self::Short(ref note) => note.children(),
        }
    }
}

impl Note {
    pub fn new() -> Result<Note> {
        let types: Vec<NoteType> = vec![NoteType::Short, NoteType::Long];
        let note_type: NoteType = Select::new("Choose note type:", types).prompt()?;
        match note_type {
            NoteType::Short => Ok(Note::Short(ShortNote::new()?)),
            NoteType::Long => Ok(Note::Long(LongNote::new()?)),
        }
    }

    pub fn render(&self) -> String {
        match *self {
            Note::Short(ref note) => note.render(),
            Note::Long(ref note) => note.render(),
        }
    }

    pub fn update(&mut self) -> Result<()> {
        match *self {
            Note::Short(ref mut note) => note.update(),
            Note::Long(ref mut note) => note.update(),
        }
    }
}

pub enum NoteType {
    Short,
    Long,
}

impl Display for NoteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            NoteType::Short => write!(f, "Shorthand note"),
            NoteType::Long => write!(f, "Detailed note"),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum NoteState {
    Pending,
    Started,
    Finished,
    Deprioritised,
}

impl NoteState {
    pub fn render(&self) -> ColoredString {
        match *self {
            NoteState::Pending => "Pending".yellow(),
            NoteState::Started => "Started".blue(),
            NoteState::Finished => "Finished".green(),
            NoteState::Deprioritised => "Deprioritised".white(),
        }
    }
}

impl Display for NoteState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render())
    }
}
