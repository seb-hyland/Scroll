#![allow(non_snake_case)]
use crate::Route;
use crate::FILE_DATA;
use crate::files::DOC_DIR;
use dioxus::prelude::*;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub fn Home() -> Element {
    let nav = navigator();
    let base_dir = DOC_DIR.clone();
    let wetlab_path = base_dir.join("wet-lab").to_string_lossy().into_owned();
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
        h1 { "Welcome to Scroll" }
        h2 { "The UBC iGEM 2025 Documentation Manager" }
        button { onclick: move |_| {
            FILE_DATA.write().set_path(base_dir.clone().join("wet-lab"));
            nav.push(Route::Viewer {});
        }, "Wet lab" }
    }
}
