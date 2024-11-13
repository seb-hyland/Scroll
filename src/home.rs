#![allow(non_snake_case)]
use crate::Route;
use dioxus::prelude::*;

pub fn Home() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
        h1 { "UBC iGEM 2025" }
        h2 { "Documentation Manager" }
        Link { to: Route::FileExplorer{}, "Go to files" }
    }
}
