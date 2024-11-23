use dioxus::prelude::*;
use std::path::PathBuf;
use tokio::process::Command;
use crate::files::FileData;

async fn marktext(filename: String) {
    Command::new("/apps/marktext")
        .arg(filename)
        .output()
        .await
        .expect("Failed to start marktext");
}

#[component]
pub fn Viewer(init: String) -> Element {
    println!("Current path: {:?}", init);
    let mut files = use_signal(FileData::new);
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
            for dir_path in directories.clone().into_iter() {
                button {
                    onclick: move |_| files.write().goto(dir_path.clone()),
                    if let Some(dir_name) = dir_path.file_name().expect("File name cannot be unwrapped").to_str() {
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
