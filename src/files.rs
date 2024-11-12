#![allow(non_snake_case)]
use dioxus::prelude::*;
use homedir::my_home;
use std::{path::PathBuf, sync::{LazyLock, Arc}};
use tokio::process::Command;

static DOC_DIR: LazyLock<PathBuf> = LazyLock::new(|| { my_home().expect("Failed to get user home directory").unwrap().join("Documents") });

struct Files {
    current_path: PathBuf,
    path_contents: Vec<PathBuf>,
    directories: Vec<PathBuf>,
    mdfiles: Vec<PathBuf>,
    attributes: Vec<Option<String>>,
    err: Option<String>,
}

impl Files {
    fn new() -> Self {
        let mut files = Self {
            current_path: DOC_DIR.clone(),
            path_contents: vec![],
	    directories: vec![],
	    mdfiles: vec![],
	    attributes: vec![],
            err: None,
        };
        files.refresh_paths();
        files
    }

    fn refresh_paths(&mut self) {
        match std::fs::read_dir(&self.current_path) {
            Ok(entries) => {
                self.path_contents.clear();
                self.clear_err();

                for entry in entries {
                    if let Ok(entry) = entry {
                        self.path_contents.push(entry.path());
                    }
                }

		self.refresh_display();
            }
            Err(err) => {
                self.err = Some(format!("An error occurred: {:?}", err));
            }
        }
    }

    fn refresh_display(&mut self) {
	self.directories.clear();
	self.mdfiles.clear();
	for entry in self.path_contents.iter().enumerate() {
	    let path = entry.1.clone();
	    if path.is_dir() {
		self.directories.push(path);
	    }
	    else if path.extension().is_some() && path.extension().unwrap() == "md" {
		self.mdfiles.push(path);
	    }
	}
    }

    fn back_dir(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.refresh_paths();
        } else {
            self.err = Some("Cannot go up from the current directory".to_string());
        }
    }

    fn enter_dir(&mut self, dir_id: usize) {
        if let Some(path) = self.directories.get(dir_id) {
            if path.is_dir() {
                self.current_path = path.to_path_buf();
                self.refresh_paths();
            }
        }
    }

    fn current(&self) -> String {
        self.current_path.display().to_string()
    }

    fn clear_err(&mut self) {
        self.err = None;
    }
}

async fn marktext(filename: Arc<String>) {
    Command::new("/apps/marktext")
        .arg(filename.as_str())
        .output()
        .await
	.expect("Failed to start marktext");
}

#[component]
pub fn FileExplorer() -> Element {
    let mut files = use_signal(Files::new);
    rsx! {
        div {
            h1 { "Current Directory: {files.write().current()}" }
            if files.read().current_path != DOC_DIR.clone() {
                button {
                    onclick: move |_| files.write().back_dir(),
                    "⬅️"
                }
                br {}
                br {}
            }
            for (index, file_path) in files.read().directories.clone().into_iter().enumerate() {
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
            br {}
            for (_index, file_path) in files.read().mdfiles.clone().into_iter().enumerate() {
                button {
                    onclick: move |_| {
                        let filepath = Arc::new(file_path.to_string_lossy().into_owned());
                        let _ = tokio::spawn(async move {
                            marktext(filepath).await;
                        });
                    },
                    if let Some(file_name) = file_path.file_name().expect("File name cannot be unwrapped").to_str() {
                        "{file_name}"
                    }
                    span {
                        "📝"
                    }
                }
            }
        }
    }
}
