use super::{
    note::{Note, NoteState},
    view::View,
};
use anyhow::{bail, Result};
use chrono::NaiveDate;
use colored::{ColoredString, Colorize};
use inquire::{Confirm, DateSelect, Editor, Select, Text};
use serde;
use std::fmt::Display;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct LongNote {
    pub title: String,
    pub description: Option<String>,
    pub created_at: chrono::NaiveDate,
    pub due_at: Option<chrono::NaiveDate>,
    pub sub_notes: Option<Vec<Note>>,
    pub state: NoteState,
}

enum UpdateChoice {
    Title,
    ViewDescription,
    SubNotes,
    Description,
    Due,
    State,
}

impl Display for UpdateChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            UpdateChoice::Title => write!(f, "Change Title"),
            UpdateChoice::Due => write!(f, "Update or Set Due"),
            UpdateChoice::State => write!(f, "Update State"),
            UpdateChoice::Description => write!(f, "Update or Set Description"),
            UpdateChoice::ViewDescription => write!(f, "View Description"),
            UpdateChoice::SubNotes => write!(f, "View and Update sub Notes"),
        }
    }
}

impl LongNote {
    pub fn new() -> Result<LongNote> {
        let title = Text::new("Enter note title:").prompt()?;
        let created_at = chrono::Utc::now().naive_local().date();

        Ok(LongNote {
            title: title,
            description: LongNote::maybe_add_description()?,
            created_at: created_at,
            sub_notes: None,
            due_at: LongNote::maybe_add_due_at()?,
            state: NoteState::Pending,
        })
    }

    fn maybe_add_description() -> Result<Option<String>> {
        let with_description = Confirm::new("Add description?").prompt()?;

        let description = if with_description {
            Some(Editor::new("Enter description").prompt()?)
        } else {
            None
        };

        Ok(description)
    }

    fn maybe_add_due_at() -> Result<Option<NaiveDate>> {
        let with_due_at = Confirm::new("Add due at?").prompt()?;

        let due_at = if with_due_at {
            Some(DateSelect::new("Select due at").prompt()?)
        } else {
            None
        };

        Ok(due_at)
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
        let choice_options = vec![
            UpdateChoice::Title,
            UpdateChoice::ViewDescription,
            UpdateChoice::Description,
            UpdateChoice::Due,
            UpdateChoice::State,
            UpdateChoice::SubNotes,
        ];
        let choice = Select::new("Choose option", choice_options).prompt()?;

        match choice {
            UpdateChoice::Title => self.update_title(),
            UpdateChoice::Due => self.update_due(),
            UpdateChoice::State => self.update_state(),
            UpdateChoice::Description => self.update_description(),
            UpdateChoice::ViewDescription => self.view_description(),
            UpdateChoice::SubNotes => self.update_sub_notes(),
        }
    }

    fn update_title(&mut self) -> Result<()> {
        let title = Text::new("Enter new title").prompt()?;
        self.title = title;
        Ok(())
    }

    fn update_description(&mut self) -> Result<()> {
        let predefined_text = match self.description {
            Some(ref description) => description.clone(),
            None => String::new(),
        };

        let new_description = Editor::new("Update description")
            .with_predefined_text(&predefined_text)
            .prompt()?;
        if new_description.len() == 0 {
            self.description = None;
        } else {
            self.description = Some(new_description);
        }

        Ok(())
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

    fn view_description(&self) -> Result<()> {
        if let Some(ref description) = self.description {
            println!("{}\n{}", self.render(), description);
        } else {
            println!("{}", self.render());
        };

        Ok(())
    }

    fn update_sub_notes(&mut self) -> Result<()> {
        let mut view = match self.sub_notes.clone() {
            Some(notes) => View::new_from_vec(notes),
            None => View::new_from_vec(vec![]),
        };

        println!("Viewing sub notes of: {}", self.render());
        view.render()?;
        let new_notes = view.get_notes();
        if new_notes.len() > 0 {
            self.sub_notes = Some(new_notes);
        } else {
            self.sub_notes = None;
        };
        println!("Exiting sub note view of: {}", self.render());

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
