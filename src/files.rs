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
use tokio::process::Command;
use indexmap::IndexMap;
use crate::Route;


pub static DOC_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    my_home()
        .expect("Failed to get documentation directory")
        .unwrap()
        .join("Documents/iGEM-2025")
});

#[derive(Clone)]
struct Files {
    current_path: PathBuf,
    path_contents: Vec<PathBuf>,
    directories: Vec<PathBuf>,
    metadata: Vec<Vec<String>>,
    attributes: Vec<(String, String)>,
    breadcrumbs: Vec<(PathBuf, String)>,
    err: Option<String>,
}

impl Files {
    fn new() -> Self {
        let mut files = Self {
            current_path: DOC_DIR.clone(),
            path_contents: vec![],
            directories: vec![],
            metadata: vec![],
            attributes: vec![],
            breadcrumbs: vec![],
            err: None,
        };
        files.refresh();
        files
    }

    fn refresh(&mut self) {
        match std::fs::read_dir(&self.current_path) {
            Ok(entries) => {
                self.clear();
                for entry in entries {
                    if let Ok(entry) = entry {
                        self.path_contents.push(entry.path());
                    }
                }
                self.directories = self.get_directories();
                self.metadata = self.get_metadata();
                self.attributes = self.get_attributes();
                self.breadcrumbs = self.get_breadcrumbs();
            }
            Err(err) => {
                self.err = Some(format!("An error occurred: {:?}", err));
            }
        }
    }

    fn clear(&mut self) {
	self.path_contents.clear();
	self.err = None;
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
	    if path.extension().map(|ext| ext == "md").unwrap_or(false) {
		if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                    if let Some(conn) = &connection {
			let mut attributes = vec![file_name.to_string()];
			let query = "SELECT attribute_value FROM FileAttributes WHERE file_name = ?";
			let mut stmt = conn.prepare(query).expect("Failed to prepare query");
			let mut rows = stmt.query(params![file_name]).expect("Failed to execute query");
			while let Some(row) = rows.next().expect("Failed to fetch row") {
                            let attribute_value: String =
				row.get(0).expect("Failed to get attribute value");
                            attributes.push(attribute_value);
			}
			metadata.push(attributes);
                    } else { metadata.push(vec![file_name.to_string()]); }
		} 
	    }
	}
	metadata
    }

    fn get_attributes(&self) -> Vec<(String, String)> {
        if let Ok(data) = fs::read_to_string(self.current_path.join("attributes.json")) {
            let parsed: IndexMap<String, Value> = serde_json::from_str(&data).expect("Failed to parse JSON");
            let attributes: Vec<(String, String)> = parsed
                .into_iter()
                .map(|(key, value)| (key, value.to_string()))
                .collect();
	    return attributes;
	}
	return vec![];
    }

    fn get_breadcrumbs(&self) -> Vec<(PathBuf, String)> {
	let mut accumulated_path = DOC_DIR.clone();
        accumulated_path.pop();
	let relative_path = self.current_path.strip_prefix(&accumulated_path).unwrap();
	let breadcrumbs: Vec<(PathBuf, String)> = relative_path
	    .components()
	    .map(|component| {
		accumulated_path.push(component);
		(accumulated_path.clone(), component.as_os_str().to_string_lossy().into_owned())
	    })
	    .collect();
	breadcrumbs
    }

    fn back_dir(&mut self) {
        if let Some(parent) = self.current_path.parent() {
	    self.current_path = parent.to_path_buf();
	    self.refresh();
        } else {
	    self.err = Some("Cannot go up from the current directory".to_string());
        }
    }

    fn enter_dir(&mut self, dir_id: usize) {
        if let Some(path) = self.get_directories().get(dir_id) {
	    if path.is_dir() {
                self.current_path = path.to_path_buf();
                self.refresh();
	    }
        }
    }

    fn goto(&mut self, path: PathBuf) {
        if path.is_dir() {
	    self.current_path = path;
	    self.refresh();
        }
    }

    fn current(&self) -> String {
        self.current_path.display().to_string()
    }

    fn set_path(&mut self, path: PathBuf) {
        self.current_path = path;
        self.refresh();
    }
}

async fn marktext(filename: String) {
    Command::new("/apps/marktext")
        .arg(filename)
        .output()
        .await
        .expect("Failed to start marktext");
}

#[component]
pub fn FileExplorer(init: PathBuf) -> Element {
    println!("Current path: {:?}", init);
    let mut files = use_signal(Files::new);
    let breadcrumbs = files.read().breadcrumbs.clone();
    let directories = files.read().directories.clone();
    let attributes = files.read().attributes.clone();
    let metadata = files.read().metadata.clone();
    rsx! {
	document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
	div {
	    for (i, (path, name)) in breadcrumbs.clone().into_iter().enumerate() {
		button {
		    onclick: move |_| files.write().goto(path.clone()),   
		    "{name}"
		}
		if i < breadcrumbs.len() - 1 {
		    span { " > " }
		}
	    }
            br {}
            for (index, file_path) in directories.iter().enumerate() {
                button {
                    onclick: move |_| files.write().enter_dir(index),
                    if let Some(dir_name) = file_path.file_name().expect("File name cannot be unwrapped").to_str() {
                        "{dir_name}"
                    }
                    span {
                        "> "
                    }
                }
            }
            br {}
            table {
                thead {
                    tr {
                        if !attributes.is_empty() {
                            th { "" }
                        }
                        for attribute_name in attributes.iter() {
                            th {
                                "{attribute_name.0}"
                            }
                        }
                    }
                }
                tbody {
                    for data in metadata.into_iter() {
                        tr {
                            td {
                                button {
                                    onclick: move |_| {
                                        let filepath = {
                                            let mut path = files.read().current_path.clone();
                                            path.push(data.get(0).unwrap().clone());
                                            let path_string = path.to_string_lossy().to_string().clone();
                                            path_string
                                        };
                                        let _ = tokio::spawn(async move {
                                            marktext(filepath).await;
                                        });
                                    },
                                    "{data.get(0).unwrap()}"
                                }
                            }
                            for data_out in data.iter().skip(1) {
                                td {
                                    "{data_out}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
