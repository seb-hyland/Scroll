use crate::types::statics::FILE_DATA;
use dioxus::prelude::*;

#[derive(Clone, Debug)]
pub struct FileGenerator {
    pub filename: String,
    pub metadata: Vec<String>,
    pub state: CreatorState,
    pub editing: bool,
}

impl FileGenerator {
    pub fn new() -> Self {
        FileGenerator {
            filename: String::new(),
            metadata: vec![String::new(); FILE_DATA.read().attributes.len()],
            state: CreatorState::Ok,
            editing: false,
            
        }
    }

    pub fn set_fields(&mut self, filename: String, metadata: Vec<String>, editing: bool) {
        self.filename = filename;
        self.metadata = metadata;
        self.state = CreatorState::Ok;
        self.editing = editing;
    }

    pub fn refresh(&mut self) {
        *self = Self::new();
    }
}



#[derive(Clone, Debug)]
pub enum CreatorState {
    Ok,
    Err { error: Vec<String> },
}


impl CreatorState {
    pub fn file_error(&mut self) {
        *self = CreatorState::Err { error: vec!["File Name".to_string()] };
    }

    pub fn component_error(&mut self, title: &str) {
        match self {
            CreatorState::Ok => {
                *self = CreatorState::Err { error: vec![title.to_string()] };
            },
            CreatorState::Err { ref mut error } => {
                error.push(title.to_string());
            }
        }
    }
}
