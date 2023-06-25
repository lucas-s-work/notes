use std::{
    borrow::Cow,
    fmt::{Display, Error},
    fs,
    path::Path,
};

use anyhow::{bail, Result};
use inquire::{InquireError, Select, Text};
use ptree::{print_tree, TreeItem};

use super::note::Note;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum ViewState {
    Add,
    View,
    Tree,
    Main,
    Remove,
    Exit,
    Update(usize),
}

impl Display for ViewState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ViewState::Add => write!(f, "Add note"),
            ViewState::Remove => write!(f, "Delete note"),
            ViewState::View => write!(f, "View notes"),
            ViewState::Main => write!(f, "Goto main menu"),
            ViewState::Exit => write!(f, "Exit"),
            ViewState::Update(_) => Err(Error),
            ViewState::Tree => write!(f, "View note tree"),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct View {
    name: String,
    notes: Vec<Note>,
    state: ViewState,
}

impl TreeItem for View {
    type Child = Note;

    fn write_self<W: std::io::Write>(
        &self,
        f: &mut W,
        style: &ptree::Style,
    ) -> std::io::Result<()> {
        write!(f, "{}", style.paint(&self.name))
    }

    fn children(&self) -> std::borrow::Cow<[Self::Child]> {
        Cow::from(self.notes.clone())
    }
}

const VIEW_FILE_PATH: &str = "./notes_view.json";

impl View {
    pub fn new() -> Result<View> {
        let file_path = Path::new(VIEW_FILE_PATH);
        if file_path.exists() {
            // Ensure that we always start in main view
            let mut loaded_view = View::load_from_file()?;
            loaded_view.state = ViewState::Main;
            Ok(loaded_view)
        } else {
            Ok(View {
                name: Text::new("Enter name for notes:").prompt()?,
                notes: Vec::new(),
                state: ViewState::Main,
            })
        }
    }

    pub fn new_from_vec(name: &str, notes: Vec<Note>) -> View {
        View {
            name: name.to_string(),
            notes: notes,
            state: ViewState::Main,
        }
    }

    pub fn get_notes(&self) -> Vec<Note> {
        self.notes.clone()
    }

    fn load_from_file() -> Result<View> {
        let file = fs::read(VIEW_FILE_PATH)?;
        Ok(serde_json::from_slice(&file)?)
    }

    pub fn save(&self) -> Result<()> {
        let file = serde_json::to_vec(self)?;
        fs::write(VIEW_FILE_PATH, file)?;
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        match self.state {
            ViewState::Main => self.render_main(),
            ViewState::Add => self.render_add_note(),
            ViewState::View => self.render_view_notes(),
            ViewState::Remove => self.render_remove_note(),
            ViewState::Update(index) => self.render_update_note(index),
            ViewState::Tree => self.render_tree(),
            ViewState::Exit => {
                self.save()?;
                Ok(())
            }
        }
    }

    fn render_main(&mut self) -> Result<()> {
        let mut options: Vec<ViewState> = Vec::new();

        // don't show the option to view notes if we don't have any
        if self.notes.len() > 0 {
            options.append(&mut vec![ViewState::View, ViewState::Tree]);
        };
        let mut other_options = vec![ViewState::Add, ViewState::Remove, ViewState::Exit];
        options.append(&mut other_options);

        match Select::new("Choose action", options).prompt() {
            Ok(action) => {
                self.state = action;
                self.render()
            }
            Err(e) => match e {
                InquireError::OperationCanceled | InquireError::OperationInterrupted => Ok(()),
                _ => bail!(e),
            },
        }
    }

    fn render_add_note(&mut self) -> Result<()> {
        match Note::new() {
            Ok(note) => self.notes.push(note),
            Err(e) => match e.downcast_ref() {
                Some(InquireError::OperationCanceled)
                | Some(InquireError::OperationInterrupted) => (),
                _ => bail!(e),
            },
        };

        self.to_menu()
    }

    fn render_view_notes(&mut self) -> Result<()> {
        let render_context: Vec<_> = self.notes.iter().map(|note| note.render()).collect();
        let choice = Select::new(
            "Select Note to update or press esc to return",
            render_context.clone(),
        )
        .prompt();

        match choice {
            Ok(choice_str) => {
                let index = render_context
                    .iter()
                    .position(|s| *s == choice_str)
                    .unwrap();
                self.state = ViewState::Update(index);
                self.render()
            }
            Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
                self.to_menu()
            }
            Err(e) => bail!(e),
        }
    }

    fn render_remove_note(&mut self) -> Result<()> {
        let render_context: Vec<_> = self.notes.iter().map(|note| note.render()).collect();
        let choice = Select::new(
            "Select Note to remove or press esc to return",
            render_context.clone(),
        )
        .prompt();

        match choice {
            Ok(choice_str) => {
                let index = render_context
                    .iter()
                    .position(|s| *s == choice_str)
                    .unwrap();
                self.notes.remove(index);
            }
            Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => (),
            Err(e) => bail!(e),
        };

        self.to_menu()
    }

    fn render_update_note(&mut self, index: usize) -> Result<()> {
        match self.notes.get_mut(index).unwrap().update() {
            Ok(()) => (),
            Err(e) => match e.downcast_ref() {
                Some(InquireError::OperationCanceled)
                | Some(InquireError::OperationInterrupted) => (),
                _ => bail!(e),
            },
        };

        self.to_menu()
    }

    fn render_tree(&mut self) -> Result<()> {
        print_tree(self)?;
        self.to_menu()
    }

    fn to_menu(&mut self) -> Result<()> {
        self.state = ViewState::Main;
        self.render()
    }
}
