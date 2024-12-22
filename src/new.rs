use crate::files::DOC_DIR;
use crate::Route;
use crate::FILE_DATA;
use crate::files::InputField;
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

/// A struct that holds data while generating a new file
/// # Fields
/// - `filename`: the name of the file to create
/// - `name_error`: whether `filename` is valid
/// - `generator`: the vector of attributes to be associated with the file
/// - `ready`: associates each attribute in `generator` with its readiness
/// - `ok`: whether all attributes are ready; i.e., all required attributes are filled in
#[derive(Clone, Copy)]
struct FileGenerator {
    filename: Signal<String>,
    metadata: Signal<Vec<String>>,
    state: Signal<CreatorState>,
}

#[derive(Clone)]
enum CreatorState {
    Ok,
    Err { file_error: String, form_error: Vec<String> },
}

impl CreatorState {
    pub fn set_file_error(&mut self, err: &str) {
        match self {
            CreatorState::Ok => {
                *self = CreatorState::Err { file_error: err.to_string(), form_error: Vec::new() };
            },
            CreatorState::Err { file_error, .. } => { *file_error = err.to_string(); },
        }
    }
    pub fn append_form_error(&mut self, name: &str) {
        match self {
            CreatorState::Ok => {
                *self = CreatorState::Err { file_error: String::new(), form_error: vec![name.to_string()] };
            },
            CreatorState::Err { form_error, .. } => { form_error.push(name.to_string()); },
        }
    }
}


fn Form() -> Element {
    match &FILE_DATA.read().attributes {
        Ok(v) => {
            rsx! {
                for id in 0..v.len() {
                    ElementCreator { id }
                }
            }
        },
        Err(e) => {
            rsx! {}
        }
    }
}

#[component]
fn ElementCreator(id: usize) -> Element {
    let attribute_ref = match FILE_DATA.read().attributes.clone() {
        Ok(list) => list.get(id).cloned(),
        Err(_) => return rsx! {},
    };
    let (mut title, elem) = match attribute_ref {
        Some(v) => (v.0.clone(), v.1.clone()),
        None => return rsx! {},
    };
    if elem.is_req() {
        title.push('*');
    }
    rsx! {
        div {
            h1 { "{ title }" }
            ElementInput { id }
        }
    }
}

#[component]
fn ElementInput(id: usize) -> Element {
    let attribute_ref = match FILE_DATA.read().attributes.clone() {
        Ok(list) => list.get(id).cloned(),
        Err(_) => return rsx! {},
    };
    let element = match attribute_ref {
        Some(v) => v.1.clone(),
        None => { return rsx! {} },
    };
    if element.is_req() {
        check_req(id);
    }
    let binding = move |value| {
        if let Some(reference) = use_context::<FileGenerator>().metadata.write().get_mut(id) {
            *reference = value;
        }
    };
    let metadata = use_context::<FileGenerator>().metadata;
    let display = metadata
        .read()
        .get(id)
        .map(|s| s.clone())
        .unwrap_or_else(String::new);
    match element {
        InputField::String { req, .. } => {
            rsx! {
                textarea {
                    rows: "4",
                    cols: "50",
                    value: "{ display }",
                    oninput: move |event| { binding(event.value()); } }
            }
        },
        InputField::Date { req, .. } => {
            rsx! {
                input {
                    type: "date",
                    value: "{ display }",
                    oninput: move |event| { binding(event.value()); } }
                button { onclick: move |event| { binding(String::new()); }, "Clear" },
            }
        },
        InputField::One { options, req, .. } => {
            rsx! {
                select {
                    oninput: move |event| { binding(event.value()); },
                    option { value: "", ""}
                    for option in options.iter() {
                        option { value: "{ option }", "{ option }" }
                    },
                }
            }
        },
        InputField::Multi { options, req, ..} => {
            println!("Multifield options: {:?}", options);
            rsx! {
                select {
                    oninput: move |event| {
                        let addition = event.value();
                        let mut selections: Vec<&str> = display.split(", ").collect();
                        if !selections.contains(&addition.as_str()) {
                            selections.push(&addition);
                            if selections.get(0) == Some(&"") {
                                selections.remove(0);
                            }
                            binding(selections.join(", "));
                        };
                    },
                    option { disabled: true, selected: true, "Add..." }
                    for option in options.iter() {
                        option { value: "{ option }", "{ option }" }
                    },
                }
                button { onclick: move |event| { binding(String::new()); }, "Clear" }
                p { "Selected: { display }" }
        
            }
        }
    }
}


/// Element for naming the new file
///
/// # Elements
/// - `h1`: "File name:" header
/// - `input`: Field for file name
/// - `p`: Warning if file name is invalid
fn FileNamer() -> Element {
    let mut file_name = use_context::<FileGenerator>().filename;
    let mut warning = use_context::<FileGenerator>().state;
    let message: &str = "Your file name cannot have whitespace or non-alphanumeric characters (excluding . _ and -).";
    let file_path: PathBuf = {
        let stub = FILE_DATA.read().current_path.clone().join(file_name());
        if file_name().is_empty() {
            stub
        } else {
            stub.with_extension("md")
        }
    };
    if !is_valid_name(file_name()) {
        warning.write().set_file_error(message);
    }
    rsx! {
        h1 { "File name: " }
        input { value: "{ file_name() }",
            oninput: move |event| {
                let mut trimmed_name = event.value();
                if trimmed_name.ends_with(".md") {
                    trimmed_name = String::from(&trimmed_name[0..&trimmed_name.len() - 4]);
                }
                file_name.set(trimmed_name);
            } }
        if !file_name().is_empty() {
            p { "The following file will be created: { file_path.display() }" }
        }
        if file_name().is_empty() { p { " { String::from(message) }" } }
    }
}

/// Checks if the file name is valid
///
/// # Props
/// - `name` The file name to check
///
/// # Returns
/// - `true` if the file name is valid
/// - `false` otherwise
fn is_valid_name(name: String) -> bool {
    if name
        .chars()
        .any(|c| c.is_whitespace() || !c.is_alphanumeric() && c != '.' && c != '_' && c != '-')
    { false }
    else { true }
}

/// Writes the file attributes to a database and creates the file
///
/// # Returns
/// - `Ok` if the file was successfully created
/// - `Err(e)` if either the database write or file creation failed
///     - `e` is of type [`eyre::Report`]
///
/// Gets file attributes from [`FileGenerator`]
fn laid_in_state() -> Result<()> {
    let context = use_context::<FileGenerator>();
    let current_path = FILE_DATA.read().current_path.clone();
    let mut file_name: String = (context.filename)();
    let file_path = current_path.clone().join(&file_name).with_extension("md");
    let db_path = current_path.clone().join("database.db");
    let connection = Connection::open(&db_path)?;
    File::create(file_path)?;

    let mut attributes = context.metadata.read().clone();
    attributes.insert(0, file_name);
    let placeholders = vec!["?"; attributes.len()].join(", ");
    let query = format!("INSERT INTO FileAttributes VALUES ({})", placeholders);
    connection.execute(&query, params_from_iter(attributes))?;
    Ok(())
}


/// Updates [`FileGenerator`]`.ready` based on whether the current element
/// meets the following requirements:
/// - If it is a required attribute, its value is a non-empty string
///
/// # Props
/// - `id`: The ID of the attribute to check
/// - `req`: Whether the attribute is required
///
/// # Returns
/// - `Ok` if the attribute is successfully checked
/// - `Err(e)` if the id couldn't be found
///     - `e` is of type [`eyre::Report`]
fn check_req(id: usize) -> Result<()> {
    let value = use_context::<FileGenerator>().metadata;
    let mut id_state = use_context::<FileGenerator>().state;
    let binding = FILE_DATA.read().attributes.clone().map_err(|_| Report::msg(""))?;
    let (title, _) = binding.get(id).ok_or_else(|| Report::msg(""))?;
    if value.get(id).map(|v| v.is_empty()).unwrap_or(false) {
        id_state.write().append_form_error(title);
    }
    Ok(())
}

/// An Element that parses an attribute type and calls the appropriate input field Element
///
/// | Attribute type | Function call |
/// | --- | --- |
/// | String | [`InputText`] |
/// | Date | [`InputDate`] |
/// | One(...) | [`InputOne`] |
/// | Multi(...) | [`InputMulti`] |
///
/// # Arguments
/// - `id`: The attribute ID
/// - `title`: The attribute title
/// - `attr_type`: The attribute type
///
/// # Passes
/// - `id, title` to all child components
/// - `req: bool` whether the field is required or not
/// - `options: Vec<String>` the options to select from
///     - Only for [`InputOne`] and [`InputMulti`]
//#[component]
//fn AttributeParser(id: usize, title: String, attr_type: String) -> Element {
//    let mut required = false;
//    if attr_type.starts_with("\"*") {
//        required = true;
//        attr_type = String::from("\"") + &String::from(&attr_type[2..attr_type.len()]);
//    }
//    let mut type_slice = attr_type.as_str();
//    if type_slice.starts_with(r#""Multi("#) && type_slice.ends_with(r#")""#) {
//        let content = &type_slice[7..type_slice.len() - 2];
//        let options: Vec<String> = parse_json(content).unwrap_or(Vec::new());
//        return rsx! { InputMulti { id: id, title: title, options: options, req: required } };
//    }
//    if type_slice.starts_with(r#""One("#) && type_slice.ends_with(r#")""#) {
//        let content = &type_slice[5..type_slice.len() - 2];
//        let options: Vec<String> = parse_json(content).unwrap_or(Vec::new());
//        return rsx! { InputOne { id: id, title: title, options: options, req: required } };
//    }
//    let output = match type_slice {
//        r#""String""# => rsx! { InputText { id: id, title: title, req: required } },
//        r#""Date""# => rsx! { InputDate { id: id, title: title, req: required } },
//        _ => rsx! {},
//    };
//    output
//}

/// Input field Element for text
///
/// # Elements
/// - `h1`: Displays the title of the attribute
/// - `textarea`: Input area for text
///
/// # Arguments
/// - See "Passes" in [`AttributeParser`]
//#[component]
//fn InputText(id: usize, title: String, req: bool) -> Element {
//    let mut value = use_context::<FileGenerator>().generator;
//    check_req(id, req);
//    if req {
//        title.push('*');
//    }
//    let current_value = value
//        .get(id)
//        .map(|v| v.to_string())
//        .unwrap_or_else(|| String::new());
//    rsx! {
//        div {
//            h1 { "{ title }" }
//            textarea {
//                rows: "4",
//                cols: "50",
//                value: "{ current_value.clone() }",
//                oninput: move |event| { if let Some(mut v) = value.get_mut(id) { *v = event.value(); } }
//            }
//        }
//    }
//}

/// Input field Element for dates
///
/// # Elements
/// - `h1`: Displays the title of the attribute
/// - `input`: Input area for date
///
/// # Arguments
/// - See "Passes" in [`AttributeParser`]
//#[component]
//fn InputDate(id: usize, title: String, req: bool) -> Element {
//    let mut value = use_context::<FileGenerator>().generator;
//    check_req(id, req);
//    if req {
//        title.push('*');
//    }
//    let current_value = value.get(id).map(|v| v.to_string()).unwrap_or_else(|| String::new());
//    rsx! {
//    div {
//            h1 { "{ title }" }
//            input {
//                type: "date",
//                value: "{ current_value.clone() }",
//                oninput: move |event| { if let Some(mut v) = value.get_mut(id) { *v = event.value(); } }
//            }
//        button { onclick: move |event| { if let Some(mut v) = value.get_mut(id) { *v = String::new(); } }, "Clear"}
//        }
//    }
//}

/// Input field Element for multiple selection
///
/// # Elements
/// - `h1`: Displays the title of the attribute
/// - `select`: Input area for selection
/// - The current selections
/// - `button`: A button to clear current selections
///
/// # Arguments
/// - See "Passes" in [`AttributeParser`]
//#[component]
//fn InputMulti(id: usize, title: String, options: Vec<String>, req: bool) -> Element {
//    let mut value = use_context::<FileGenerator>().generator;
//    check_req(id, req);
//    if req {
//        title.push('*');
//    }
//    let current_value = value.get(id).map(|v| v.to_string()).unwrap_or_else(|| String::new());
//    rsx! {
//        div {
//            h1 { "{ title }" }
//            select {
//                oninput: move |event| {
//                    let addition: String = event.value();
//                    let binding: String = value.get(id).map(|v| v.to_string()).unwrap_or_else(|| String::new());
//                    let mut vector: Vec<&str> = binding.split(", ").collect();
//                    if !vector.contains(&addition.as_str()) {
//                        vector.push(addition.as_str());
//                        if vector.get(0) == Some(&"") {
//                            vector.remove(0);
//                        }
//                        if let Some(mut v) = value.get_mut(id) { *v = vector.join(", "); };
//                    }
//                },
//                option { disabled: true, selected: true, "Add { title }" }
//                for option in options.iter() {
//                    option { value: "{option}", "{option}" }
//                },
//            }
//            button { onclick: move |event| { if let Some(mut v) = value.get_mut(id) { *v = String::new(); } }, "Clear"}
//            p { "Selected: { current_value }" }
//        }
//    }
//}

/// Input field Element to select one element from a list
///
/// # Elements
/// - `h1`: Displays the title of the attribute
/// - `select`: Input area for selection
/// - `button`: A button to clear current selections
///
/// # Arguments
/// - See "Passes" in [`AttributeParser`]
//#[component]
//fn InputOne(id: usize, title: String, options: Vec<String>, req: bool) -> Element {
//    let mut value = use_context::<FileGenerator>().generator;
//    check_req(id, req);
//    if req {
//        title.push('*');
//    }
//    let current_value = value.get(id).map(|v| v.to_string()).unwrap_or_else(|| String::new());
//    rsx! {
//        div {
//            h1 { "{ title }" }
//            select {
//                oninput: move |event| { if let Some(mut v) = value.get_mut(id) { *v = event.value(); } },
//                option { value: "", ""}
//                for option in options.iter() {
//                    option { value: "{option}", "{option}" }
//                },
//            }
//        }
//    }
//}

fn ErrorAnalyzer() -> Element {
    let mut binding = use_context::<FileGenerator>().state;
    let mut state = binding.read().clone();
    let mut error: Vec<String> = Vec::new();
    match state {
        CreatorState::Ok => rsx! { Eternity {} },
        CreatorState::Err { file_error, form_error } => {
            error.push(file_error);
            error.extend(form_error);
            let output_string = error.join(", ");
            rsx! {
                "WARNING: The following fields are empty or invalid: { output_string }"
            }
        }
    }
}

fn Eternity() -> Element {
    let navigator = use_navigator();
    let mut message = use_signal(|| String::new());
    rsx! {
        button { onclick: move |_| {
            match laid_in_state() {
		Ok(()) => {
		    FILE_DATA.write().refresh();
		    navigator.push(Route::Viewer {});
		}
		Err(e) => {
		    message.set(e.to_string());
		}
	    }},
		 "Save" }
	" { message() } "
    } 
}

/// Major component for new file creation UI
#[component]
pub fn Creator() -> Element {
    let attr_creator = use_context_provider(|| FileGenerator {
        filename: Signal::new(String::new()),
        metadata: Signal::new(vec![String::new(); FILE_DATA.read().attributes.clone().unwrap_or(Vec::new()).len()]),
        state: Signal::new(CreatorState::Ok),

    });
    use_context::<FileGenerator>().state.set(CreatorState::Ok);
    println!("{:?}", use_context::<FileGenerator>().filename);
    println!("{:?}", use_context::<FileGenerator>().metadata);
    rsx! {
        div {
            FileNamer {}
            br {}
            Form {}
            br {}
            ErrorAnalyzer {}
            Link { to: Route::Viewer {}, "Cancel" }
        }
    }
}
