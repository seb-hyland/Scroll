use dioxus::prelude::*;
use tokio::process::Command;
use crate::{
    FILE_DATA,
    files::InputField,
    new::{Creator, POPUP_GENERATOR, name_deser},
    tools::scroll_processor,
};
use rayon::prelude::*;



async fn marktext(filename: String) {
    Command::new("/apps/marktext")
        .arg(filename)
        .output()
        .await
        .unwrap();
}


fn Breadcrumbs() -> Element {
    let breadcrumbs = FILE_DATA.read().breadcrumbs.clone();
    rsx! {
	div {
            class: "breadcrumbs-container",
            div {
		class: "breadcrumbs",
		for (i, (path, name)) in breadcrumbs.into_iter().enumerate() {
                    button {
			onclick: move |_| FILE_DATA.write().goto(path.clone()),   
			"{ name }"
                    }
                    if i < FILE_DATA.read().breadcrumbs.len() - 1 {
			span { "/" }
                    }
		}
            }
            span {
                class: "new-button",
                NewButton {}
            }
        }
    }
}


fn NewButton() -> Element {
    let attributes = &FILE_DATA.read().attributes;
    let metadata = &FILE_DATA.read().metadata;
    let component = match attributes {
        Ok(v) => {
            if !v.is_empty() && metadata.is_ok() {
                rsx! {
		    button { onclick: move |_| {
                        POPUP_GENERATOR.write().refresh();
                        document::eval(r#"
const dialog = document.getElementById("file-creator");
dialog.showModal();"#);
                    }, "New" }
                }
            }
            else { rsx! {} }
        },
        Err(_) => rsx! {}
    };
    component
}


fn Directories() -> Element {
    let directories = FILE_DATA.read().directories.clone();
    rsx! {
        div {
            class: "directories-div",
	    for dir_path in directories.into_iter() {
                button {
                    class: "directory-button",
		    onclick: move |_| {
                        FILE_DATA.write().goto(dir_path.clone());
                        POPUP_GENERATOR.write().refresh();
                    },
		    if let Some(dir_name) = dir_path.file_name().unwrap_or_default().to_str() {
                        "{dir_name}"
		    }
		    span {
                        "> "
		    }
                }
	    }
        }
    }
}


fn FileTable() -> Element {
    let attribute_binding = &FILE_DATA.read().attributes;
    let metadata_binding = &FILE_DATA.read().metadata;
    let attributes = match attribute_binding {
        Err(e) => { return rsx! { "An error occured:\n{e}" }; },
        Ok(v) => v.clone(),
    };
    let metadata = match metadata_binding {
        Err(e) => { return rsx! { "An error occured:\n{e}" }; },
        Ok(v) => v.clone(),
    };
    if attributes.is_empty() {
        return rsx! {};
    } else {
        rsx! {
            div {
                class: "table-div",
	        table {
                    thead {
		        tr {
			    th { "" }
			    th { "" }
                            for (attribute_name, attribute_type) in attributes.iter() {
			        th {
                                    "{attribute_name}"
                                    match attribute_type {
                                        InputField::String { .. } => rsx! {},
                                        InputField::Date { .. }=> rsx! {},
                                        InputField::One { id, .. } => rsx! {
                                            PopupOpener { id: id }
                                        },
                                        InputField::Multi { id, .. } => rsx! {
                                            PopupOpener { id: id }
                                        }
                                    }
			        }
                            }
		        }
                    }
                    tbody {
		        for (i, data) in metadata.into_iter().enumerate() {
                            tr {
			        td {
                                    class: "marktext-cell",
                                    button {
                                        class: "marktext-button",
                                        title: "Open file \"{ data.get(0).unwrap() }.md\" in editor",
				        onclick: move |_| {
                                            let filepath = {
                                                assert!(data.get(0).is_some(), "File name non existent in metadata");
					        let mut path = FILE_DATA.read().current_path.clone();
					        path.push(data
					            .get(0)
					            .unwrap()
					            .clone());
					        path.set_extension("md");
					        path
                                            };
				            tokio::spawn(async move {
					        marktext(filepath.to_string_lossy().into_owned()).await;  
                                            });
				        },
				        "{ name_deser(&data.get(0).unwrap()) }"
                                    }
                                }
                                td {
                                    class: "action-cell",
                                    button {
                                        class: "action-button",
                                        onclick: move |_| {
                                            let nav = navigator();
                                            let name = FILE_DATA.write().get_item_name(i);
                                            set_editor_environment(name);
                                            document::eval(r#"
const dialog = document.getElementById("file-creator");
dialog.showModal();"#);
                                        },
                                        "⚙"
                                    }
                                }
			        for data_out in data.iter().skip(1) {
                                    if data_out.len() >= 30 {
                                        td {
                                            class: "table-content",
                                            title: "{data_out.clone()}",
				            "{data_out}"
                                        }
			            } else {
                                        td {
                                            class: "table-content",
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
    }
}

fn set_editor_environment(name: String) {
    assert!(FILE_DATA.read().metadata.is_ok(), "Invalid metadata yet file creator called.");

    let metadata_binding = &FILE_DATA.read().metadata;
    let metadata_vec = metadata_binding.as_ref().unwrap().par_iter()
        .find_any(|inner_vec| inner_vec.get(0).map(|v| *v == name).unwrap_or(false));

    assert!(metadata_vec.is_some(), "{}", format!("No metadata found for {name}"));
    let mut metadata = metadata_vec.unwrap().into_iter()
        .skip(1)
        .cloned()
        .collect();

    POPUP_GENERATOR.write().set_fields(name, metadata, true);
}

#[derive(Clone, Debug)]
struct CurrentDB(Signal<String>);

#[component]
fn DBPopup() -> Element {
    let db_binding = use_context::<CurrentDB>().0;
    let db_name = db_binding.read().clone();
    println!{"Name: {:?}", db_name};
    let db_content = scroll_processor::collect_table(&db_name);
    println!{"Name: {:?}", db_content};
    assert!(db_content.is_ok(), "DB Malformed!");
    let content = db_content.unwrap();
    let first = content.get(0).unwrap();
    rsx! {
        dialog {
            id: "db-popup",
            class: "creator-popup",
            div {
                class: "metadata-div",
                h1 { "Database: " u { "{db_name}" } }
                div {
                    class: "table-div",
                    table {
                        thead {
                            tr {
                                for title in first.iter() {
                                    th {
                                        "{title}"
                                    }
                                }
                            }
                        }
                        tbody {
                            for row in content.iter().skip(1) {
                                tr {
                                    for cell in row.iter() {
                                        td {
                                            "{cell}"
                                        }
                                    } 
                                }
                            }
                        }
                    }
                }
                button {
                    onclick: move |_| {
                        document::eval(r#"
const dialog = document.getElementById("db-popup");
dialog.close();"#);
                    },
                    "Close"
                }
            }
        }
    }
}



#[component]
pub fn PopupOpener(id: String) -> Element {
    rsx! {
        button {
            onclick: move |_| {
                *use_context::<CurrentDB>().0.write() = id.clone();
                document::eval(r#"
const dialog = document.getElementById("db-popup");
dialog.showModal();"#);
            },
            "?"
        }
    }
}



#[component]
pub fn Viewer() -> Element {
    use_context_provider(|| CurrentDB(Signal::new("Members".to_string())));
    rsx! {
	div {
            Breadcrumbs {}
	    br {}
            Directories {}
	    br {}
            FileTable {}
            br {}
            Creator {}
            DBPopup {}
        }
    }
}
