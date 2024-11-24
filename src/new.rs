use dioxus::prelude::*;
use std::path::PathBuf;
use crate::Route;


#[component]
pub fn Creator(from: PathBuf, attributes: Vec<(String, String)>) -> Element {
    rsx! {
	div {
            Link { to: Route::Viewer { init: String::new() }, "Cancel" }
        }
    }
}
