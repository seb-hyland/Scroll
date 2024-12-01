use dioxus::prelude::*;
use std::{path::PathBuf, ffi::OsStr};
use tokio::process::Command;
use eyre::Result;
use crate::FILE_DATA;
use crate::Route;

async fn marktext(filename: String) -> Result<()> {
    Command::new("/apps/marktext")
        .arg(filename)
        .output()
        .await?;
    Ok(())
}

#[component]
pub fn Viewer() -> Element {
    let breadcrumbs = FILE_DATA.read().breadcrumbs.clone();
    let directories = FILE_DATA.read().directories.clone();
    let attributes = FILE_DATA.read().attributes.clone();
    let metadata = FILE_DATA.read().metadata.clone();
    rsx! {
	document::Link { rel: "stylesheet", href: asset!("/assets/main.css") },
	div {
	    div {
                class: "breadcrumbs-container",
                div {
		    class: "breadcrumbs",
		    for (i, (path, name)) in breadcrumbs.into_iter().enumerate() {
                        button {
			    onclick: move |_| FILE_DATA.write().goto(path.clone()),   
			    "{name}"
                        }
                        if i < FILE_DATA.read().breadcrumbs.len() - 1 {
			    span { " > " }
                        }
		    }
                }
                span {
		    class: "new-button",
		    Link { 
                        to: Route::Creator {},
                        "+   New"
		    }
                }
	    }
	    br {}
	    for dir_path in directories.into_iter() {
                button {
		    onclick: move |_| FILE_DATA.write().goto(dir_path.clone()),
		    if let Some(dir_name) = dir_path.file_name().unwrap_or_default().to_str() {
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
					    let mut path = FILE_DATA.read().current_path.clone();
					    path.push(data.get(0).unwrap_or(&String::new()).clone());
					    path
                                        };
					tokio::spawn(async move {
					    marktext(filepath.to_string_lossy().into_owned()).await;  
                                        });
				    },
				    "{ data.get(0).unwrap_or(&String::new()) }"
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

