use crate::prelude::*;
use tokio::process::Command;
use crate::{
    metadata_popup::Creator,
    db_popup::*,
    home::Home,
};



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
            onclick: move |_| FILE_DATA.write().goto(&path),   
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
    let component = if !attributes.is_empty() {
        rsx! {
            button { onclick: move |_| {
                POPUP_GENERATOR.write().refresh();
                document::eval(r#"
const dialog = document.getElementById("file-creator");
dialog.showModal();"#);
            }, "New" }
        }
    } else { rsx! {} };
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
                        FILE_DATA.write().goto(&dir_path);
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
    let attributes = &FILE_DATA.read().attributes;
    let metadata = FILE_DATA.read().metadata.clone();
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
        "{ deserialize(&data.get(0).unwrap()) }"
    }
    }
        td {
            class: "action-cell",
            button {
            class: "action-button",
            onclick: move |_| {
                let name = {
                    let metadata_ref = FILE_DATA.read().metadata.clone();
                    let item = metadata_ref.get(i);
                    assert!(item.is_some(), "{}",
                        "ERR[1|1]: Invalid metadata item {i} queried");
                    let title_binding = item.unwrap().get(0);

                    assert!(title_binding.is_some(), "{}",
                        "ERR[1|1]: Metadata item {i} is empty.");
                    title_binding.unwrap().clone()
                };
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
    let metadata = &FILE_DATA.read().metadata;
    let metadata_vec = metadata.par_iter()
        .find_any(|inner_vec| 
            inner_vec
                .get(0)
                .map(|v| *v == name)
                .unwrap_or(false)
        );

    assert!(metadata_vec.is_some(), "{}", format!("ERR[0|0]: No metadata found for {name}"));
    let env_metadata = metadata_vec.unwrap().into_iter()
        .skip(1)
        .cloned()
        .collect();

    POPUP_GENERATOR.write().set_fields(name, env_metadata, true);
}


#[component]
pub fn Viewer() -> Element {
    use_context_provider(|| CurrentDB(Signal::new("Members".to_string())));

    if FILE_DATA.read().current_path == *DOC_DIR.read().unwrap() {
        rsx! { Home {} }
    } else {
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
}
