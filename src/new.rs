use crate::Route;
use crate::FILE_DATA;
use crate::files::InputField;
use crate::tools::json_processor;
use dioxus::prelude::*;
use eyre::Result;
use std::{
    fs::{File, write, remove_file},
    path::PathBuf,
};
use rayon::prelude::*;


/// A struct that holds data while generating a new file
/// # Fields
/// - `filename`: the name of the file to create
/// - `name_error`: whether `filename` is valid
/// - `generator`: the vector of attributes to be associated with the file
/// - `ready`: associates each attribute in `generator` with its readiness
/// - `ok`: whether all attributes are ready; i.e., all required attributes are filled in
#[derive(Clone, Debug)]
pub struct FileGenerator {
    filename: String,
    metadata: Vec<String>,
    state: CreatorState,
    editing: bool,
}

pub static POPUP_GENERATOR: GlobalSignal<FileGenerator> = Global::new(|| FileGenerator::new());

impl FileGenerator {
    pub fn new() -> Self {
        FileGenerator {
            filename: String::new(),
            metadata: vec![String::new(); FILE_DATA.read().attributes.as_ref().unwrap().len()],
            state: CreatorState::Ok,
            editing: false,
            
        }
    }

    pub fn set_fields(&mut self, filename: String, metadata: Vec<String>, editing: bool) {
        self.filename = filename;
        self.metadata = metadata;
        self.state = CreatorState::Ok;
        self.editing = editing;
    }

    pub fn refresh(&mut self) {
        *self = Self::new();
    }
}



#[derive(Clone, Debug)]
enum CreatorState {
    Ok,
    Err { error: Vec<String> },
}


impl CreatorState {
    pub fn file_error(&mut self) {
        *self = CreatorState::Err { error: vec!["File Name".to_string()] };
    }

    pub fn component_error(&mut self, title: &str) {
        match self {
            CreatorState::Ok => {
                *self = CreatorState::Err { error: vec![title.to_string()] };
            },
            CreatorState::Err { ref mut error } => {
                error.push(title.to_string());
            }
        }
    }
}



/// Major component for new file creation UI
#[component]
pub fn Creator() -> Element {
    assert!(FILE_DATA.read().attributes.is_ok(), "Invalid attributes yet file creator called.");
    let name = &POPUP_GENERATOR.read().filename.clone();
    let editing = POPUP_GENERATOR.read().editing;

    rsx! {
        div {
            class: "creator-popup",
            dialog {
                id: "file-creator",
                if editing {
                    div {
                        class: "metadata-div",
                        h1 { "Updating: " u { "{ name_deser(name) }" } }
                        Deleter {}
                        br {}
                    }
                }
                Form {}
                br {}
                button {
                    onclick: move |_| {
                        document::eval(r#"
const dialog = document.getElementById("file-creator");
dialog.close();"#);
                    },
                    class: "close-button",
                    "Cancel" }
                Submission {}
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
    let file_name = POPUP_GENERATOR.read().filename.clone();
    let message: &str = "Your file name cannot have non-alphanumeric characters (excluding '-') or be empty.";

    let file_path: PathBuf = {
        let stub = FILE_DATA.read().current_path.clone().join(&name_ser(&file_name));
        if file_name.is_empty() {
            stub
        } else {
            stub.with_extension("md")
        }
    };

    rsx! {
        div {
            class: "metadata-div",
            h1 { "Creating new file..." }
            h2 { "File name" }
            input { value: "{ file_name.clone() }",
                oninput: move |event| {
                    let mut trimmed_name = event.value();
                    if trimmed_name.ends_with(".md") {
                        trimmed_name = String::from(&trimmed_name[0..&trimmed_name.len() - 4]);
                    }
                    POPUP_GENERATOR.write().filename = trimmed_name;
                } }
            if is_valid_name(&file_name) {
                p { "The following file will be created: { file_path.display() }" }
            } else {
                p { " { message.to_string() }" }
            }
        }
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
fn is_valid_name(name: &str) -> bool {
    if name.chars()
        .any(|c| !c.is_alphanumeric() && c != '-' && c != ' ') ||
        name.is_empty() {
            false
        } else {
            true
        }
}

pub fn name_ser(name: &str) -> String {
    name.replace(" ", "_")
}

pub fn name_deser(name: &str) -> String {
    name.replace("_", " ")
}


fn Form() -> Element {
    let attribute_binding = &FILE_DATA.read().attributes;
    assert!(&attribute_binding.is_ok(), "Invalid attributes, yet file creator called.");
    let attributes_length = attribute_binding.as_ref().unwrap().len();

    error_checker();

    rsx! {
        if !POPUP_GENERATOR.read().editing {
            FileNamer {}
        }
        for id in 0..attributes_length {
            ElementCreator { id }
        }
    }
}


fn error_checker() {
    let name_binding = POPUP_GENERATOR.read().filename.clone();
    let metadata_binding = POPUP_GENERATOR.read().metadata.clone();
    let mut state_binding = &mut POPUP_GENERATOR.write().state;

    let attribute_binding = &FILE_DATA.read().attributes;
    assert!(&attribute_binding.is_ok(), "Invalid attributes, yet file creator called.");
    let attributes = attribute_binding.as_ref().unwrap();
    let required: Vec<(bool, String)> = attributes.iter()
        .map(|(title, element)| {
            (element.is_req(), title.clone())
        })
        .collect();

    // Start by clearing previous errors
    *state_binding = CreatorState::Ok;

    // Check for filename error
    if !is_valid_name(&name_binding) {
        state_binding.file_error();
    }

    // Check for required elements that are empty
    for id in 0..required.len() {
        assert!(required.get(id).is_some() && metadata_binding.get(id).is_some(), "Metadata struct size does not match attribute struct");
        let (is_required, title) = required.get(id).unwrap();
        let metadata = metadata_binding.get(id).unwrap();
        if *is_required && metadata.is_empty() {
            state_binding.component_error(title);
        }
    }
}


#[component]
fn ElementCreator(id: usize) -> Element {
    let attributes = &FILE_DATA.read().attributes;
    assert!(&attributes.is_ok(), "Invalid attributes, yet file creator called.");

    let attr_ref = attributes.as_ref().unwrap().get(id);
    assert!(&attr_ref.is_some(), "{}", format!("Invalid attribute ID: {id}"));
    let element_type = attr_ref.unwrap().1.clone();

    let metadata = POPUP_GENERATOR.read().metadata.clone();
    assert!(&metadata.get(id).is_some(), "{}", format!("Invalid metadata vector. ID: {id}"));
    let display = metadata.get(id).unwrap().clone();

    // Closure to grab writable reference to metadata item
    let binding = move |value| {
        let binding = &mut POPUP_GENERATOR.write();
        let mut write_binding = binding.metadata.get_mut(id).unwrap();
        *write_binding = value;
    };

    let (mut title, elem) = attr_ref.unwrap().clone();

    if elem.is_req() {
        title.push('*');
    }


    rsx! {
        div {
            class: "metadata-div",
            h2 { "{ title }" }
            match element_type {
                InputField::String { .. } => {
                    rsx! {
                        textarea {
                            rows: "4",
                            value: "{ display }",
                            oninput: move |event| { binding(event.value()); } }
                    }
                },
                InputField::Date { .. } => {
                    rsx! {
                        input {
                            type: "date",
                            value: "{ display }",
                            oninput: move |event| { binding(event.value()); } }
                        button { onclick: move |_| { binding(String::new()); }, "Clear" },
                    }
                },
                InputField::One { options, .. } => {
                    rsx! {
                        select {
                            oninput: move |event| { binding(event.value()); },
                            option { value: "", "" }
                            for option in options.iter() {
                                option { value: "{ option }", selected: *option == display, "{ option }" }
                            }
                        }
                    }
                },
                InputField::Multi { options, .. } => {
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
                        button { onclick: move |_| { binding(String::new()); }, "Clear" }
                        p { b {"Selected: "} " { display }" }
                    }
                }
            }
        }
    }
}



fn Submission() -> Element {
    let state = POPUP_GENERATOR.read().state.clone();
    match state {
        CreatorState::Ok => {
            match POPUP_GENERATOR.read().editing {
                false => rsx! { Eternity {} },
                true => rsx! { Rebase {} },
            }
        },
        CreatorState::Err { error } => {
            let output_string = error.join(", ");
            rsx! {
                h3 { "⚠️ WARNING!" }
                p { "The following required fields are empty or invalid: { output_string }" }
            }
        },
    }
}


fn Eternity() -> Element {
    let nav = navigator();
    let mut message = use_signal(|| String::new());
    rsx! {
        button {
            onclick: move |_| {
                match laid_in_state() {
		    Ok(()) => {
		        FILE_DATA.write().refresh();
                        document::eval(r#"
const dialog = document.getElementById("file-creator");
dialog.close();"#);
		    }
		    Err(e) => {
		        message.set(e.to_string());
		    }
	        }},
            class: "creation-button",
		 "Create" }
	p { " { message() } " }
    } 
}


fn Rebase() -> Element {
    let nav = navigator();
    let mut message = use_signal(|| String::new());
    rsx! {
        button {
            class: "creation-button",
            onclick: move |_| {
            match refreeze() {
		Ok(()) => {
		    FILE_DATA.write().refresh();
                        document::eval(r#"
const dialog = document.getElementById("file-creator");
dialog.close();"#);
		}
		Err(e) => {
		    message.set(e.to_string());
		}
	    }},
		 "Update" }
	" { message() } "
    } 
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
    let context = &POPUP_GENERATOR;
    let current_path = &FILE_DATA.read().current_path;
    let db_path = current_path.join(".database.json");
    assert!(db_path.exists(), "Database does not exist yet file creator called");
    
    let attributes_binding = FILE_DATA.read().attributes.clone();
    assert!(attributes_binding.is_ok(), "Invalid attributes, yet file creator called");
    let attributes = attributes_binding.as_ref().unwrap();

    let metadata_json_binding = json_processor::get_json_hashmap(&FILE_DATA.read().current_path);
    assert!(metadata_json_binding.is_ok(), "Invalid metadata, yet file creator called");
    let metadata_json = metadata_json_binding.as_ref().unwrap();
    let mut metadata: Vec<Vec<(String, String)>> = json_processor::hashmap_to_vec(metadata_json);

    let mut new_filename = context.read().filename.clone();
    new_filename = name_ser(&new_filename);
    let new_metadata = &context.read().metadata;

    let mut new_vector = vec![("__ID".to_string(), new_filename.clone())];
    assert!(attributes.len() == new_metadata.len(), "Attribute and metadata vectors do not match");
    for ((title, _), metadata) in attributes.iter().zip(new_metadata.iter()) {
        new_vector.push((title.clone(), metadata.clone()));
    }
    metadata.push(new_vector);

    let mut json_array = json_processor::vec_to_json(&metadata);
    let json_string = serde_json::to_string_pretty(&json_array)?;

    let file_path = current_path.clone().join(&new_filename).with_extension("md");
    File::create(file_path)?;
    write(db_path, json_string)?;

    Ok(())
}


fn refreeze() -> Result<()> {
    let context = POPUP_GENERATOR.read();
    let current_path = &FILE_DATA.read().current_path;
    let db_path = current_path.join(".database.json");
    assert!(db_path.exists(), "Database does not exist yet file creator called");
    
    let attributes_binding = FILE_DATA.read().attributes.clone();
    assert!(attributes_binding.is_ok(), "Invalid attributes, yet file creator called");
    let attributes = attributes_binding.as_ref().unwrap();

    let metadata_json_binding = json_processor::get_json_hashmap(&FILE_DATA.read().current_path);
    assert!(metadata_json_binding.is_ok(), "Invalid metadata, yet file creator called");
    let mut metadata_json = metadata_json_binding.unwrap();

    let new_filename = &context.filename;
    let new_metadata = &context.metadata;

    let mut new_vector = vec![("__ID".to_string(), new_filename.clone())];
    assert!(attributes.len() == new_metadata.len(), "Attribute and metadata vectors do not match");
    for ((title, _), metadata) in attributes.iter().zip(new_metadata.iter()) {
        new_vector.push((title.clone(), metadata.clone()));
    }

    json_processor::update_json_hashmap(&mut metadata_json, &new_filename, new_vector);
    let mut metadata: Vec<Vec<(String, String)>> = json_processor::hashmap_to_vec(&metadata_json);

    let mut json_array = json_processor::vec_to_json(&metadata);
    let json_string = serde_json::to_string_pretty(&json_array)?;

    let file_path = current_path.clone().join(&new_filename).with_extension("md");
    File::create(file_path)?;
    write(db_path, json_string)?;

    Ok(())
}



#[component]
fn Deleter() -> Element {
    let nav = navigator();
    let mut message = use_signal(|| String::new());
    rsx! {
        button {
            class: "close-button",
            onclick: move |_| {
            match fall_out_of_window() {
		Ok(()) => {
		    FILE_DATA.write().refresh();
                        document::eval(r#"
const dialog = document.getElementById("file-creator");
dialog.close();"#);
		}
		Err(e) => {
		    message.set(e.to_string());
		}
	    }},
            "🗑️ Delete this file and its associated data"
        }
        p { "{ message }"}
    }
}


fn fall_out_of_window() -> Result<()> {
    let context = POPUP_GENERATOR.read();
    let current_path = &FILE_DATA.read().current_path;
    let db_path = current_path.join(".database.json");
    assert!(db_path.exists(), "Database does not exist yet file creator called");
    
    let filename = &context.filename;

    let metadata_json_binding = json_processor::get_json_hashmap(&FILE_DATA.read().current_path);
    assert!(metadata_json_binding.is_ok(), "Invalid metadata, yet file creator called");
    let mut metadata_json = metadata_json_binding.unwrap();
    json_processor::delete_from_hashmap(&mut metadata_json, &filename);

    let mut metadata: Vec<Vec<(String, String)>> = json_processor::hashmap_to_vec(&metadata_json);
    let mut json_array = json_processor::vec_to_json(&metadata);
    let json_string = serde_json::to_string_pretty(&json_array)?;
    write(db_path, json_string)?;

    let file_path = current_path.clone().join(filename).with_extension("md");
    remove_file(file_path)?;
    Ok(()) 
}
