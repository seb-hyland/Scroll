#![allow(non_snake_case)]
use dioxus::prelude::*;
use eyre::{Report, Result};
use homedir::my_home;
use rusqlite::{params, Connection};
use serde_json::Value;
use std::{
    error::Error,
    fs,
    path::PathBuf,
    sync::LazyLock,
};
use nom::{
    bytes::complete::{tag, take_until},
    error::ErrorKind,
    sequence::delimited};
use crate::Route;



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
    pub metadata: Vec<Vec<String>>,
    pub attributes: Result<Vec<(String, InputField)>, String>,
    pub breadcrumbs: Vec<(PathBuf, String)>,
    pub selected_file: PathBuf,
}



#[derive(Clone)]
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
            metadata: Vec::new(),
            breadcrumbs: Vec::new(),
        };
        files.refresh();
        files
    }

    
    pub fn refresh(&mut self) -> Result<()> {
        let entries = std::fs::read_dir(&self.current_path)?;
        self.clear();
        for entry in entries {
            if let Ok(entry) = entry {
                self.path_contents.push(entry.path());
            }
        }
        self.directories = self.get_directories();
        self.attributes = self.get_attributes();
        self.metadata = self.get_metadata();
        self.breadcrumbs = match self.get_breadcrumbs() {
            Ok(crumbs) => crumbs,
            Err(e) => vec![],
        };
        Ok(())
    }

    
    fn clear(&mut self) {
	self.path_contents.clear();
    }

    
    fn get_directories(&self) -> Vec<PathBuf> {
	let mut directories: Vec<PathBuf> = vec![];
	for entry in self.path_contents.iter().enumerate() {
	    if entry.1.is_dir() {
		directories.push(entry.1.clone());
	    }
	}
	directories
    }

    
    fn get_metadata(&self) -> Vec<Vec<String>> {
	let mut metadata: Vec<Vec<String>> = vec![];
        let db_path = PathBuf::from(&self.current_path).join("database.db");
        let connection: Option<Connection> = match Connection::open(&db_path) {
            Ok(conn) => Some(conn),
            Err(_) => None,
        };
	for entry in self.path_contents.iter().enumerate() {
	    let path = entry.1.clone();
	    if path.extension().map_or(false, |ext| ext == "md") {
                if let Some(file_name) = path.file_stem().unwrap_or_default().to_str() {
                    if let Some(conn) = &connection {
                        let mut attributes = vec![file_name.to_string()];
                        let query = "SELECT * FROM FileAttributes WHERE filename = ?";
                        let mut stmt = match conn.prepare(query) {
                            Ok(stmt) => stmt,
                            Err(err) => {
                                eprintln!("Failed to prepare query: {:?}", err);
                                continue;
                            }
                        };
                        let column_count = stmt.column_count();
                        let mut rows = match stmt.query(params![file_name]) {
                            Ok(rows) => rows,
                            Err(err) => {
                                eprintln!("Failed to get rows: {:?}", err);
                                continue;
                            }
                        };
                        if let Some(row) = rows.next().unwrap_or(None) {
                            for i in 1..column_count {
                                let value: Option<String> = row.get(i).unwrap_or(Some(String::new()));
                                attributes.push(value.unwrap_or(String::new()));
                            }
                        }
                        metadata.push(attributes);
                    } else {
                        metadata.push(vec![file_name.to_string()]);
                    }
                }
	    }
        }
	metadata
    }

    
    pub fn get_attributes(&self) -> Result<Vec<(String, InputField)>, String> {
        let mut results = Vec::new();
        let file_path = self.current_path.join("attributes.scroll");
        let stub = String::from(format!("Unable to parse attributes file. Please inform a lead immediately with the following debug information: |Attribute file| {}. |Additional info|", file_path.display()));
        let data = match fs::read_to_string(file_path) {
            Ok(v) => v,
            Err(_) => return Ok(Vec::new()),
        };
        for (i, line) in data.lines().enumerate() {
            let line_number = i + 1;
            let error_report = |database| String::from(format!(
                "{stub} Parsing error. Line: {line_number}. Internal error: {database}."));
            if line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split(":").collect();
            if parts.len() != 2 {
                return Err(error_report(String::new()));
            }
            let attr_title = parts.get(0)
                .ok_or_else(|| error_report(String::new()))?
                .trim().to_string();
            let raw_type = parts.get(1)
                .ok_or_else(|| error_report(String::new()))?
                .trim().to_string();
            let attr_type: InputField = Self::parse_attributes(raw_type.as_str())
                .map_err(|e| error_report(e))?;
            results.push((attr_title, attr_type));
        }
	return Ok(results);
    }

    fn parse_attributes(mut s: &str) -> Result<InputField, String> {
        let asterisk = s.starts_with("*");
        if asterisk {
            s = &s[1..s.len()];    
        }
        println!("String to parse: {:?}", s);
        let basic_parse = |keyword: &str| -> bool {
            tag::<_, _, (_, ErrorKind)>(keyword)(s).is_ok()
        };
        let advanced_parse = |stub: &str| -> Option<String> {
            println!("Passed stub: {:?}", stub);
            let search_string = format!("{stub}("); 
            let result = delimited(
                tag::<_,_,(_, ErrorKind)>(search_string.as_str()),
                take_until(")"),
                tag(")")
            )(s);
            println!("Result: {:?}", result);
            match result {
                Ok((_, parsed_content)) => {
                    Some(parsed_content.to_string())
                }
                Err(_) => None,
            }
        };
        if basic_parse("String") {
            return Ok(InputField::String { req: asterisk });
        }
        if basic_parse("Date") {
            return Ok(InputField::Date { req: asterisk });
        }
        if let Some(capture) = advanced_parse("One") {
            println!{"Received parse: {:?}", capture};
            let option_list = Self::parse_list(&capture)
                .map_err(|_| String::from(format!("|Malformed database: {capture}.|")))?;
            return Ok(InputField::One { req: asterisk, options: option_list });
        }
        if let Some(capture) = advanced_parse("Multi") {
            println!{"Received parse: {:?}", capture};
            let option_list = Self::parse_list(&capture)
                .map_err(|_| String::from(format!("|Malformed database: {capture}.|")))?;
            return Ok(InputField::Multi { req: asterisk, options: option_list });
        }
        return Err(String::new());
    }


    /// Parses a list of attribute types into a Rust vector
    ///
    /// # Props
    /// - `list_id`: The name of the list file in `DOC_DIR/sys`
    ///
    /// # Returns
    /// - `Ok` if the file is successfully parsed
    /// - `Err(e)` if the file cannot be found or JSON parsing fails
    ///     - `e` is of type [`eyre::Report`]
    fn parse_list(list_id: &str) -> Result<Vec<String>> {
        let file_path: PathBuf = DOC_DIR
            .clone()
            .join("sys")
            .join(list_id)
            .with_extension("scroll");
        let mut file_contents = fs::read_to_string(&file_path)?;
        let mut list: Vec<String> = Vec::new();
        for line in file_contents.lines() {
            list.push(String::from(line));
        }
        println!("For {}, contents: {:?}", &file_path.display(), list);
        Ok(list)
    }


    fn get_breadcrumbs(&self) -> Result<Vec<(PathBuf, String)>> {
	let mut accumulated_path = DOC_DIR.clone();
        accumulated_path.pop();
	let relative_path = self.current_path.strip_prefix(&accumulated_path)?;
	let breadcrumbs: Vec<(PathBuf, String)> = relative_path
	    .components()
	    .map(|component| {
		accumulated_path.push(component);
		(accumulated_path.clone(), component.as_os_str().to_string_lossy().into_owned())
	    })
	    .collect();
	Ok(breadcrumbs)
    }

    
    pub fn goto(&mut self, path: PathBuf) {
        if path.is_dir() {
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

    
    pub fn set_path(&mut self, path: PathBuf) {
        self.current_path = path;
        self.refresh();
    }

    
    pub fn select_file(&mut self, path: PathBuf) {
        self.selected_file = path;
    }
}
