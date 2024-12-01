#![allow(non_snake_case)]
use dioxus::prelude::*;
use homedir::my_home;
use rusqlite::{params, Connection};
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    sync::LazyLock,
};
use indexmap::IndexMap;
use eyre::Result;
use crate::Route;


pub static DOC_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    my_home()
        .expect("Failed to get home directory")
        .expect("Path to home directory is empty")
        .join("Documents/iGEM-2025")
});

#[derive(Clone)]
pub struct FileData {
    pub current_path: PathBuf,
    path_contents: Vec<PathBuf>,
    pub directories: Vec<PathBuf>,
    pub metadata: Vec<Vec<String>>,
    pub attributes: Vec<(String, String)>,
    pub breadcrumbs: Vec<(PathBuf, String)>,
    pub selected_file: PathBuf,
}

impl FileData {
    pub fn new() -> Self {
        let mut files = Self {
            current_path: DOC_DIR.clone(),
            path_contents: vec![],
            directories: vec![],
            metadata: vec![],
            attributes: vec![],
            breadcrumbs: vec![],
            selected_file: PathBuf::new()
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
        self.metadata = self.get_metadata();
        self.attributes = match self.get_attributes() {
            Ok(attr) => attr,
            Err(e) => vec![],
        };
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
                if let Some(file_name) = path.file_name().unwrap_or_default().to_str() {
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

    fn get_attributes(&self) -> Result<Vec<(String, String)>> {
        let data = fs::read_to_string(self.current_path.join("attributes.json"))?;
        let parsed: IndexMap<String, Value> = serde_json::from_str(&data)?;
        let attributes: Vec<(String, String)> = parsed
            .into_iter()
            .map(|(key, value)| (key, value.to_string()))
            .collect();
	return Ok(attributes);
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
