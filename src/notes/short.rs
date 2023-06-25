use super::note::{Note, NoteState};
use anyhow::Result;
use colored::{ColoredString, Colorize};
use inquire::{Confirm, DateSelect, Select, Text};
use ptree::TreeItem;
use serde;
use std::{borrow::Cow, fmt::Display};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct ShortNote {
    pub title: String,
    pub created_at: chrono::NaiveDate,
    pub due_at: Option<chrono::NaiveDate>,
    pub state: NoteState,
}

enum UpdateChoice {
    Title,
    Due,
    State,
}

impl Display for UpdateChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            UpdateChoice::Title => write!(f, "Change Title"),
            UpdateChoice::Due => write!(f, "Update or Set Due"),
            UpdateChoice::State => write!(f, "Update State"),
        }
    }
}

impl TreeItem for ShortNote {
    type Child = Note;
    fn write_self<W: std::io::Write>(
        &self,
        f: &mut W,
        style: &ptree::Style,
    ) -> std::io::Result<()> {
        write!(f, "{}", style.paint(self.render()))
    }

    fn children(&self) -> std::borrow::Cow<[Note]> {
        Cow::from(vec![])
    }
}

impl ShortNote {
    pub fn new() -> Result<ShortNote> {
        let title = Text::new("Enter note text:").prompt()?;
        let with_due_at = Confirm::new("With due date?").prompt()?;

        if with_due_at {
            let due_at = DateSelect::new("Choose due date:").prompt()?;
            Ok(ShortNote::new_with_deadline(title, due_at))
        } else {
            Ok(ShortNote::new_no_deadline(title))
        }
    }

    fn new_no_deadline(title: String) -> ShortNote {
        let now = chrono::Utc::now().naive_local().date();
        ShortNote {
            title: title,
            created_at: now,
            due_at: None,
            state: NoteState::Pending,
        }
    }

    fn new_with_deadline(title: String, due_at: chrono::NaiveDate) -> ShortNote {
        let now = chrono::Utc::now().naive_local().date();
        ShortNote {
            title: title,
            created_at: now,
            due_at: Some(due_at),
            state: NoteState::Pending,
        }
    }

    pub fn render(&self) -> String {
        let state_string = self.state.render();
        let base_string = format!("{}: {}: {}", state_string, self.title, self.created_at);

        if let Some(due_at) = self.due_at {
            let due_at_string = format_due_at(&due_at);
            format!("{} due: {}", base_string, due_at_string)
        } else {
            base_string
        }
    }

    pub fn update(&mut self) -> Result<()> {
        let choice_options = vec![UpdateChoice::Title, UpdateChoice::Due, UpdateChoice::State];
        let choice = Select::new("Choose how to update", choice_options).prompt()?;

        match choice {
            UpdateChoice::Title => self.update_title(),
            UpdateChoice::Due => self.update_due(),
            UpdateChoice::State => self.update_state(),
        }
    }

    fn update_state(&mut self) -> Result<()> {
        let state_choices = vec![
            NoteState::Pending,
            NoteState::Started,
            NoteState::Finished,
            NoteState::Deprioritised,
        ];
        let choice = Select::new("Choose new state", state_choices).prompt()?;
        self.state = choice;
        Ok(())
    }

    fn update_due(&mut self) -> Result<()> {
        let with_due = Confirm::new("Have due date? (y/n)").prompt()?;
        if with_due {
            let due_at = DateSelect::new("Choose due date:").prompt()?;
            self.due_at = Some(due_at);
        } else {
            self.due_at = None;
        };

        Ok(())
    }

    fn update_title(&mut self) -> Result<()> {
        let new_title = Text::new("Enter new title:").prompt()?;
        self.title = new_title;
        Ok(())
    }
}

fn format_due_at(due_at: &chrono::NaiveDate) -> ColoredString {
    let current_time = chrono::Utc::now().naive_local().date();

    if current_time > *due_at {
        due_at.to_string().red()
    } else {
        due_at.to_string().normal()
    }
}
