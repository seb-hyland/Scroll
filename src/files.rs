#![allow(non_snake_case)]
use crate::Route;
use crate::tools::{JSONProcessor, ScrollProcessor};
use dioxus::prelude::*;
use eyre::{Report, Result};
use homedir::my_home;
use std::{
    fs::{read_to_string, read_dir},
    path::PathBuf,
    sync::LazyLock,
};
use rayon::prelude::*;



pub static DOC_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    assert!(my_home().is_ok() && my_home().unwrap().is_some(), "Home directory could not be found");
    my_home()
        .unwrap()
        .unwrap()
        .join("Documents/iGEM-2025")
});



#[derive(Clone)]
pub struct FileData {
    pub current_path: PathBuf,
    path_contents: Vec<PathBuf>,
    pub directories: Vec<PathBuf>,
    pub metadata: Result<Vec<Vec<String>>, String>,
    pub attributes: Result<Vec<(String, InputField)>, String>,
    pub breadcrumbs: Vec<(PathBuf, String)>,
    pub selected_file: PathBuf,
}



#[derive(Clone, Debug)]
pub enum InputField {
    String { req: bool },
    Date { req: bool },
    One { options: Vec<String>, req: bool },
    Multi { options: Vec<String>, req: bool },
}



impl InputField {
    pub fn is_req(&self) -> bool {
        match self {
            InputField::String { req, .. } => *req,
            InputField::Date { req, .. } => *req,
            InputField::One { req, .. } => *req,
            InputField::Multi { req, .. } => *req,
        }
    }
}



impl FileData {
    pub fn new() -> Self {
        let mut files = Self {
            current_path: DOC_DIR.clone(),
            selected_file: PathBuf::new(),
            attributes: Ok(Vec::new()),
            path_contents: Vec::new(),
            directories: Vec::new(),
            metadata: Ok(Vec::new()),
            breadcrumbs: Vec::new(),
        };
        files.refresh();
        files
    }

    
    pub fn refresh(&mut self) {
        let current_dir = &self.current_path;
        let dir_read = read_dir(current_dir);
        assert!(dir_read.is_ok(), "{}", format!("The directory {} could not be read", current_dir.display()));
        let entries = dir_read.unwrap();
        self.clear();
        for entry in entries {
            if let Ok(entry) = entry {
                self.path_contents.push(entry.path());
            }
        }
        self.directories = self.get_directories();
        self.attributes = self.get_attributes();
        self.metadata = if self.attributes.is_ok() {
            self.get_metadata().map_err(|e| e.to_string())
        }
        else {
            Ok(Vec::new())
        };
        self.breadcrumbs = self.get_breadcrumbs();
    }

    
    fn clear(&mut self) {
	self.path_contents.clear();
    }

    
    fn get_directories(&self) -> Vec<PathBuf> {
	let directories: Vec<PathBuf> = self.path_contents.iter()
            .filter(|p| p.is_dir())
            .cloned()
            .collect();
        directories
    }

    
    pub fn get_attributes(&self) -> Result<Vec<(String, InputField)>, String> {
        let attribute_file = self.current_path.join(".attributes.scroll");
        let data = match read_to_string(&attribute_file) {
            Ok(v) => v,
            Err(_) => return Ok(Vec::new()),
        };

        let err_stub = format!("Unable to parse attributes file. Please inform a lead immediately with the following debug information: |Attribute file| {}. |Additional info|", attribute_file.display());
        let err_report = |database, line_number| format!(
            "{err_stub} Parsing error. Line: {line_number}. Internal error: {database}.");

        let trimmed_data: Vec<(&str, &str)> = ScrollProcessor::parse_pairs(&data)
            .map_err(|e| err_report(String::new(), e))?;

        // Parse the components of each line
        trimmed_data.iter().enumerate()
            .map(|(i, (title, raw_type))| {
                let line_num = i + 1;
                let attr_type: InputField = ScrollProcessor::parse_attribute(raw_type)
                    .map_err(|e| err_report(e, line_num))?;
                Ok((title.to_string(), attr_type))
            })
            .collect()
    }

    
    fn get_metadata(&self) -> Result<Vec<Vec<String>>> {
        let objects = JSONProcessor::get_json_hashmap(&self.current_path);
        let result = objects?.par_iter()
            .map(|(_, map)| {
                let mut struct_metadata: Vec<String> = Vec::new();
                struct_metadata.push(map
                    .get("__ID")
                    .cloned()
                    .unwrap_or("".to_string()));
                
                assert!(self.attributes.is_ok(), "Metadata computed with invalid attributes");
                for (attribute, _) in self.attributes.as_ref().unwrap().iter() {
                    struct_metadata.push(map
                        .get(attribute)
                        .cloned()
                        .unwrap_or("".to_string()));
                }
                struct_metadata
            })
            .collect::<Vec<Vec<String>>>();
        Ok(result)
    }

    
    fn get_breadcrumbs(&self) -> Vec<(PathBuf, String)> {
	let mut base_path = DOC_DIR.clone();
        base_path.pop();

        let relative_path = self.current_path.strip_prefix(&base_path);
        assert!(relative_path.is_ok(), "Current directory is not a valid documentation directory.");

        let mut accumulator = base_path;
	let breadcrumbs: Vec<(PathBuf, String)> = relative_path.unwrap()
	    .components()
	    .map(|component| {
		accumulator.push(component);
		(accumulator.clone(), component.as_os_str().to_string_lossy().into_owned())
	    })
	    .collect();
        breadcrumbs
    }

    
    pub fn goto(&mut self, path: PathBuf) {
        assert!(path.is_dir(), "Attempted navigation to a non-directory file");
        if path == DOC_DIR.clone() {
            let nav = navigator();
            nav.push(Route::Home {});
        }
        else {
	    self.current_path = path;
	    self.refresh();
        }
    }
}

