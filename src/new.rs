use crate::files::DOC_DIR;
use crate::Route;
use crate::FILE_DATA;
use dioxus::prelude::*;
use eyre::{Report, Result};
use serde_json::Value;
use rusqlite::{params_from_iter, Connection};
use std::{
    error,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};


#[derive(Clone, Copy)]
struct AttributeCreator {
    generator: Signal<Vec<String>>,
    ready: Signal<Vec<bool>>,
    filename: Signal<String>,
    name_error: Signal<Option<String>>,
    ok: Signal<bool>,
}

fn FileNamer() -> Element {
    let mut file_name = use_context::<AttributeCreator>().filename;
    let mut warning = use_context::<AttributeCreator>().name_error;
    let message: &str = "Your file name cannot have whitespace or non-alphanumeric characters (excluding . _ and -).";
    let file_path: PathBuf = {
        let stub = FILE_DATA.read().current_path.clone().join(file_name());
        if file_name().is_empty() {
            stub
        } else {
            stub.with_extension("md")
        }
    };
    warning.set({
        if is_valid_name(file_name()) { None }
        else { Some(message.to_owned()) }
    });
    rsx! {
        h1 { "File name: " }
        input { value: "{ file_name() }",
            oninput: move |event| {
                let mut filename = event.value();
                if filename.ends_with(".md") {
                    filename = String::from(&filename[0..&filename.len() - 4]);
                }
                file_name.set(filename);
            } }
        if !file_name().is_empty() {
            p { "The following file will be created: { file_path.display() }" }
        }
        if warning().is_some() { p { " { warning().unwrap_or(String::new()) }" } }
    }
}

fn is_valid_name(name: String) -> bool {
    if name
        .chars()
        .any(|c| c.is_whitespace() || !c.is_alphanumeric() && c != '.' && c != '_' && c != '-')
    { false }
    else { true }
}

fn laid_in_state() -> Result<()> {
    let context = use_context::<AttributeCreator>();
    let current_path = FILE_DATA.read().current_path.clone();
    let mut file_name: String = (context.filename)();
    let file_path = current_path.clone().join(&file_name);
    let db_path = current_path.clone().join("database.db");
    let connection = Connection::open(&db_path)?;
    File::create(file_path)?;

    let mut attributes = (context.generator)().clone();
    attributes.insert(0, file_name);
    let placeholders = vec!["?"; attributes.len()].join(", ");
    let query = format!("INSERT INTO FileAttributes VALUES ({})", placeholders);
    connection.execute(&query, params_from_iter(attributes))?;
    Ok(())
}

fn parse_json(list_id: &str) -> Result<Vec<String>> {
    let file_path: PathBuf = DOC_DIR
        .clone()
        .join("sys")
        .join(list_id)
        .with_extension("json");
    println!("{}", file_path.display());
    let mut file = File::open(file_path)?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    let list: Vec<String> = serde_json::from_str(&file_contents)?;
    Ok(list)
}

fn check_req(id: usize, req: bool) -> Result<()> {
    let value = use_context::<AttributeCreator>().generator;
    let error = |id| Report::msg(format!("No value found for ID: {}", id));
    if req {
        if value.get(id).map(|v| v.is_empty()).unwrap_or(false) {
            *use_context::<AttributeCreator>().ready.get_mut(id).ok_or_else(|| error(id))? = false;
        } else {
            *use_context::<AttributeCreator>().ready.get_mut(id).ok_or_else(|| error(id))? = true;
        }
    } else {
        *use_context::<AttributeCreator>().ready.get_mut(id).ok_or_else(|| error(id))? = true;
    }
    Ok(())
}


#[component]
fn AttributeParser(id: usize, title: String, attr_type: String) -> Element {
    let mut required = false;
    if attr_type.starts_with("\"*") {
        required = true;
        attr_type = String::from("\"") + &String::from(&attr_type[2..attr_type.len()]);
    }
    let mut type_slice = attr_type.as_str();
    if type_slice.starts_with(r#""Multi("#) && type_slice.ends_with(r#")""#) {
        let content = &type_slice[7..type_slice.len() - 2];
        let options: Vec<String> = parse_json(content).unwrap_or(Vec::new());
        return rsx! { InputMulti { id: id, title: title, options: options, req: required } };
    }
    if type_slice.starts_with(r#""One("#) && type_slice.ends_with(r#")""#) {
        let content = &type_slice[5..type_slice.len() - 2];
        let options: Vec<String> = parse_json(content).unwrap_or(Vec::new());
        return rsx! { InputOne { id: id, title: title, options: options, req: required } };
    }
    let output = match type_slice {
        r#""String""# => rsx! { InputText { id: id, title: title, req: required } },
        r#""Date""# => rsx! { InputDate { id: id, title: title, req: required } },
        _ => rsx! {},
    };
    output
}

#[component]
fn InputText(id: usize, title: String, req: bool) -> Element {
    let mut value = use_context::<AttributeCreator>().generator;
    let mut req_indicator = String::new();
    check_req(id, req);
    if req {
        req_indicator.push('*');
    }
    let current_value = value
        .get(id)
        .map(|v| v.to_string())
        .unwrap_or_else(|| String::new());
    rsx! {
        div {
            h1 { "{ title }{ req_indicator }" }
            textarea {
                rows: "4",
                cols: "50",
                value: "{ current_value.clone() }",
                oninput: move |event| { if let Some(mut v) = value.get_mut(id) { *v = event.value(); } }
            }
        }
    }
}

#[component]
fn InputDate(id: usize, title: String, req: bool) -> Element {
    let mut value = use_context::<AttributeCreator>().generator;
    let mut req_indicator = String::new();
    check_req(id, req);
    if req {
        req_indicator.push('*');
    }
    let current_value = value.get(id).map(|v| v.to_string()).unwrap_or_else(|| String::new());
    rsx! {
    div {
            h1 { "{ title }{ req_indicator }" }
            input {
                type: "date",
                value: "{ current_value.clone() }",
                oninput: move |event| { if let Some(mut v) = value.get_mut(id) { *v = event.value(); } }
            }
        button { onclick: move |event| { if let Some(mut v) = value.get_mut(id) { *v = String::new(); } }, "Clear"}
        }
    }
}

#[component]
fn InputMulti(id: usize, title: String, options: Vec<String>, req: bool) -> Element {
    let mut value = use_context::<AttributeCreator>().generator;
    let mut req_indicator = String::new();
    check_req(id, req);
    if req {
        req_indicator.push('*');
    }
    let current_value = value.get(id).map(|v| v.to_string()).unwrap_or_else(|| String::new());
    rsx! {
        div {
            h1 { "{ title }{ req_indicator }" }
            select {
                oninput: move |event| {
                    let addition: String = event.value();
                    let binding: String = value.get(id).map(|v| v.to_string()).unwrap_or_else(|| String::new());
                    let mut vector: Vec<&str> = binding.split(", ").collect();
                    if !vector.contains(&addition.as_str()) {
                        vector.push(addition.as_str());
                        if vector.get(0) == Some(&"") {
                            vector.remove(0);
                        }
                        if let Some(mut v) = value.get_mut(id) { *v = vector.join(", "); };
                    }
                },
                option { disabled: true, selected: true, "Add { title }" }
                for option in options.iter() {
                    option { value: "{option}", "{option}" }
                },
            }
            button { onclick: move |event| { if let Some(mut v) = value.get_mut(id) { *v = String::new(); } }, "Clear"}
            p { "Selected: { current_value }" }
        }
    }
}

#[component]
fn InputOne(id: usize, title: String, options: Vec<String>, req: bool) -> Element {
    let mut value = use_context::<AttributeCreator>().generator;
    let mut req_indicator = String::new();
    check_req(id, req);
    if req {
        req_indicator.push('*');
    }
    let current_value = value.get(id).map(|v| v.to_string()).unwrap_or_else(|| String::new());
    rsx! {
        div {
            h1 { "{ title }{ req_indicator }" }
            select {
                oninput: move |event| { if let Some(mut v) = value.get_mut(id) { *v = event.value(); } },
                option { value: "", ""}
                for option in options.iter() {
                    option { value: "{option}", "{option}" }
                },
            }
        }
    }
}

fn ErrorAnalyzer() -> Element {
    let mut context = use_context::<AttributeCreator>();
    let mut empty_attrs: Vec<String> = Vec::new();
    context.ok.set(true);
    if (context.name_error)().is_some() || (context.filename)().is_empty() {
        empty_attrs.push(String::from("File name"));
        context.ok.set(false);
    }
    for (id, ok) in (context.ready)().iter().enumerate() {
        if !ok {
            empty_attrs.push(FILE_DATA.read().attributes.get(id).unwrap_or(&(String::new(), String::new())).0.clone());
            context.ok.set(false);
        }
    }
    let output_string = empty_attrs.join(", ");
    rsx! {
        if !output_string.is_empty() {
            "WARNING: The following fields are empty or invalid: { output_string }"
        }
        else {
            Eternity {}
        }
    }
}

fn Eternity() -> Element {
    let navigator = use_navigator();
    rsx! {
        button { onclick: move |_| {
            laid_in_state();
            FILE_DATA.write().refresh();
            navigator.push(Route::Viewer {});
        },
            "Save" }
    } 
}

#[component]
pub fn Creator() -> Element {
    let attributes = FILE_DATA.read().attributes.clone();
    let attr_creator = use_context_provider(|| AttributeCreator {
        generator: Signal::new(vec![String::new(); attributes.len()]),
        ready: Signal::new(vec![true; attributes.len()]),
        filename: Signal::new(String::new()),
        name_error: Signal::new(None),
        ok: Signal::new(false),
    });
    println!("{:?}", use_context::<AttributeCreator>().generator);
    println!("{:?}", use_context::<AttributeCreator>().ready);
    rsx! {
        div {
            FileNamer {}

            for (i, (name, attr_type)) in attributes.iter().enumerate() {
                AttributeParser { id: i, title: name, attr_type: attr_type }
            }

            ErrorAnalyzer {}

            Link { to: Route::Viewer {}, "Cancel" }
        }
    }
}
