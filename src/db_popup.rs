use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct CurrentDB(pub Signal<String>);

#[component]
pub fn DBPopup() -> Element {
    let db_binding = use_context::<CurrentDB>().0;
    let db_name = db_binding.read().clone();
    let db_content = scroll_processor::db_query(&db_name);
    assert!(db_content.is_ok(), "{}", "ERR[0|1]: Database {db_name} is malformed.");
    let content = db_content.unwrap().0;
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
