#![allow(non_snake_case)]
use crate::Route;
use crate::files::DOC_DIR;
use dioxus::prelude::*;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub fn Home() -> Element {
    let base_dir = DOC_DIR.clone();
    let wetlab_path = base_dir.join("wet-lab").clone();
    println!("Path to wet-lab: {:?}", wetlab_path);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
        h1 { "UBC iGEM 2025" }
        h2 { "Documentation Manager" }
        Link { to: Route::FileExplorer { init: wetlab_path }, "Wet lab" }
    }
}
