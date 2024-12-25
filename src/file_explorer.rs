use dioxus::prelude::*;
use tokio::process::Command;
use crate::FILE_DATA;
use crate::Route;



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
			"{name}"
                    }
                    if i < FILE_DATA.read().breadcrumbs.len() - 1 {
			span { " > " }
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
    let component = match attributes {
        Ok(v) => {
            if !v.is_empty() {
                rsx! {
		    Link { to: Route::Creator {}, "+   New" }
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
	    table {
                thead {
		    tr {
			th { "" }
			th { "" }
                        for attribute_name in attributes.iter() {
			    th {
                                "{attribute_name.0}"
			    }
                        }
		    }
                }
                tbody {
		    for (i, data) in metadata.into_iter().enumerate() {
                        tr {
			    td {
                                button {
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
				    "{ data.get(0).unwrap() }"
                                }
			    }
                            td {
                                button {
                                    onclick: move |_| {
                                        let nav = navigator();
                                        let name = FILE_DATA.write().get_item_name(i);
                                        nav.push(Route::Editor { name: name.clone() });
                                        println!("Pushed route: {:?}", name);
                                    },
                                    "⚙"
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




#[component]
pub fn Viewer() -> Element {
    rsx! {
	document::Link { rel: "stylesheet", href: asset!("/assets/main.css") },
	div {
            Breadcrumbs {}
	    br {}
            Directories {}
	    br {}
            FileTable {}
        }
    }
}
