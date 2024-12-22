#![allow(non_snake_case)]
use crate::Route;
use crate::FILE_DATA;
use crate::files::DOC_DIR;
use dioxus::prelude::*;

pub fn Home() -> Element {
    let nav = navigator();
    let base = &DOC_DIR;
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
        h1 { "Welcome to Scroll" }
        h2 { "The UBC iGEM 2025 Documentation Manager" }

        button { onclick: move |_| {
            FILE_DATA.write().goto(base.join("wet-lab"));
            nav.push(Route::Viewer {});
        }, "Wet lab" }

        button { onclick: move |_| {
            FILE_DATA.write().goto(base.join("dry-lab"));
            nav.push(Route::Viewer {});
        }, "Dry lab" }
    }
}
