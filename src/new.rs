use crate::Route;
use crate::FILE_DATA;
use crate::files::InputField;
use crate::tools::json_processor;
use dioxus::prelude::*;
use eyre::Result;
use std::{
    fs::{File, write},
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
struct FileGenerator {
    filename: Signal<String>,
    metadata: Signal<Vec<String>>,
    state: Signal<CreatorState>,
}


#[derive(Clone, Debug)]
enum CreatorState {
    Ok,
    Err { error: Vec<String> },
}


#[derive(Clone, Debug, PartialEq, Props)]
struct SubmissionType {
    new: Option<()>,
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


fn Form(props: SubmissionType) -> Element {
    let attribute_binding = &FILE_DATA.read().attributes;
    assert!(&attribute_binding.is_ok(), "Invalid attributes, yet file creator called.");
    let attributes_length = attribute_binding.as_ref().unwrap().len();

    error_checker();

    rsx! {
        if props.new.is_some() {
            FileNamer {}
            br {}
        }
        for id in 0..attributes_length {
            ElementCreator { id }
        }
    }
}


fn error_checker() {
    let context = use_context::<FileGenerator>();
    let name_binding = context.filename;
    let metadata_binding = context.metadata;
    let mut state_binding = context.state;

    let attribute_binding = &FILE_DATA.read().attributes;
    assert!(&attribute_binding.is_ok(), "Invalid attributes, yet file creator called.");
    let attributes = attribute_binding.as_ref().unwrap();
    let required: Vec<(bool, String)> = attributes.iter()
        .map(|(title, element)| {
            (element.is_req(), title.clone())
        })
        .collect();

    // Start by clearing previous errors
    state_binding.set(CreatorState::Ok);

    // Check for filename error
    if !is_valid_name(&name_binding.read()) {
        state_binding.write().file_error();
    }

    // Check for required elements that are empty
    assert!(required.len() == metadata_binding.read().len(), "Metadata struct size does not match attribute struct");
    for id in 0..required.len() {
        assert!(required.get(id).is_some() && metadata_binding.get(id).is_some(), "Metadata struct size does not match attribute struct");
        let (is_required, title) = required.get(id).unwrap();
        let metadata = metadata_binding.get(id).unwrap();
        if *is_required && metadata.is_empty() {
            state_binding.write().component_error(title);
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

    let metadata = use_context::<FileGenerator>().metadata.clone();
    assert!(&metadata.get(id).is_some(), "{}", format!("Invalid metadata vector. ID: {id}"));
    let display = metadata.read().get(id).unwrap().clone();

    // Closure to grab writable reference to metadata item
    let binding = move |value| {
        let mut binding = use_context::<FileGenerator>();
        let mut write_binding = binding.metadata.get_mut(id).unwrap();
        *write_binding = value;
    };

    let (mut title, elem) = attr_ref.unwrap().clone();

    if elem.is_req() {
        title.push('*');
    }


    rsx! {
        div {
            h1 { "{ title }" }
            match element_type {
                InputField::String { .. } => {
                    rsx! {
                        textarea {
                            rows: "4",
                            cols: "50",
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
                        p { "Selected: { display }" }
                    }
                }
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
    let message: &str = "Your file name cannot have whitespace or non-alphanumeric characters (excluding . _ and -).";

    let file_path: PathBuf = {
        let stub = FILE_DATA.read().current_path.clone().join(file_name());
        if file_name().is_empty() {
            stub
        } else {
            stub.with_extension("md")
        }
    };

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
        if is_valid_name(&file_name.read()) {
            p { "The following file will be created: { file_path.display() }" }
        } else {
            p { " { message.to_string() }" }
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
        .any(|c| c.is_whitespace() || !c.is_alphanumeric() && c != '.' && c != '_' && c != '-') ||
        name.is_empty() {
            false
        } else {
            true
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
    let context = use_context::<FileGenerator>();
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

    let new_filename = context.filename;
    let new_metadata = context.metadata;

    let mut new_vector = vec![("__ID".to_string(), new_filename.read().clone())];
    assert!(attributes.len() == new_metadata.read().len(), "Attribute and metadata vectors do not match");
    for ((title, _), metadata) in attributes.iter().zip(new_metadata.read().iter()) {
        new_vector.push((title.clone(), metadata.clone()));
    }
    metadata.push(new_vector);

    let json_array = json_processor::vec_to_json(&metadata);
    let json_string = serde_json::to_string_pretty(&json_array)?;

    let file_path = current_path.clone().join(&*new_filename.read()).with_extension("md");
    File::create(file_path)?;
    write(db_path, json_string)?;

    Ok(())
}


fn Submission(props: SubmissionType) -> Element {
    let binding = use_context::<FileGenerator>().state;
    let state = binding.read().clone();
    match state {
        CreatorState::Ok => {
            match props.new {
                Some(_) => rsx! { Eternity {} },
                None => rsx! { Rebase {} },
            }
        },
        CreatorState::Err { error } => {
            let output_string = error.join(", ");
            rsx! {
                "WARNING: The following fields are empty or invalid: { output_string }"
            }
        },
    }
}


fn Eternity() -> Element {
    let nav = navigator();
    let mut message = use_signal(|| String::new());
    rsx! {
        button { onclick: move |_| {
            match laid_in_state() {
		Ok(()) => {
		    FILE_DATA.write().refresh();
		    nav.push(Route::Viewer {});
		}
		Err(e) => {
		    message.set(e.to_string());
		}
	    }},
		 "Create" }
	" { message() } "
    } 
}


fn Rebase() -> Element {
    let nav = navigator();
    let mut message = use_signal(|| String::new());
    rsx! {
        button { onclick: move |_| {
            match refreeze() {
		Ok(()) => {
		    FILE_DATA.write().refresh();
		    nav.push(Route::Viewer {});
		}
		Err(e) => {
		    message.set(e.to_string());
		}
	    }},
		 "Update" }
	" { message() } "
    } 
}

fn refreeze() -> Result<()> {
    let context = use_context::<FileGenerator>();
    let current_path = &FILE_DATA.read().current_path;
    let db_path = current_path.join(".database.json");
    assert!(db_path.exists(), "Database does not exist yet file creator called");
    
    let attributes_binding = FILE_DATA.read().attributes.clone();
    assert!(attributes_binding.is_ok(), "Invalid attributes, yet file creator called");
    let attributes = attributes_binding.as_ref().unwrap();

    let metadata_json_binding = json_processor::get_json_hashmap(&FILE_DATA.read().current_path);
    assert!(metadata_json_binding.is_ok(), "Invalid metadata, yet file creator called");
    let mut metadata_json = metadata_json_binding.unwrap();

    let new_filename = context.filename;
    let new_metadata = context.metadata;

    let mut new_vector = vec![("__ID".to_string(), new_filename.read().clone())];
    assert!(attributes.len() == new_metadata.read().len(), "Attribute and metadata vectors do not match");
    for ((title, _), metadata) in attributes.iter().zip(new_metadata.read().iter()) {
        new_vector.push((title.clone(), metadata.clone()));
    }

    json_processor::update_json_hashmap(&mut metadata_json, &new_filename.read(), new_vector);
    let mut metadata: Vec<Vec<(String, String)>> = json_processor::hashmap_to_vec(&metadata_json);

    let json_array = json_processor::vec_to_json(&metadata);
    let json_string = serde_json::to_string_pretty(&json_array)?;

    let file_path = current_path.clone().join(&*new_filename.read()).with_extension("md");
    File::create(file_path)?;
    write(db_path, json_string)?;

    Ok(())
}
/// Major component for new file creation UI
#[component]
pub fn Creator() -> Element {
    assert!(FILE_DATA.read().attributes.is_ok(), "Invalid attributes yet file creator called.");

    let _attr_creator = use_context_provider(|| FileGenerator {
        filename: Signal::new(String::new()),
        metadata: Signal::new(vec![String::new(); FILE_DATA.read().attributes.as_ref().unwrap().len()]),
        state: Signal::new(CreatorState::Ok),
    });

    rsx! {
        div {
            Form { new: Some(()) }
            br {}
            Submission { new: Some(()) }
            Link { to: Route::Viewer {}, "Cancel" }
        }
    }
}



#[component]
pub fn Editor(name: String) -> Element {
    assert!(FILE_DATA.read().metadata.is_ok(), "Invalid metadata yet file creator called.");
    let metadata_binding = &FILE_DATA.read().metadata;
    let metadata_vec = metadata_binding.as_ref().unwrap().par_iter()
        .find_any(|inner_vec| inner_vec.get(0).map(|v| *v == name).unwrap_or(false));
    assert!(metadata_vec.is_some(), "{}", format!("No metadata found for {name}"));
    let mut metadata = metadata_vec.unwrap().into_iter()
        .skip(1)
        .cloned()
        .collect();

    let _attr_creator = use_context_provider(|| FileGenerator {
        filename: Signal::new(name.clone()),
        metadata: Signal::new(metadata),
        state: Signal::new(CreatorState::Ok),
    });

    println!("{:?}", use_context::<FileGenerator>());
    
    rsx! {
        div {
            h1 { "Updating metadata for {name} "}
            br {}
            Form {}
            br {}
            Submission {}
            Link { to: Route::Viewer {}, "Cancel" }
        }
    }
}
