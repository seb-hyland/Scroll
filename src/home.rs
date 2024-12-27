#![allow(non_snake_case)]
use crate::Route;
use crate::FILE_DATA;
use crate::files::DOC_DIR;
use dioxus::prelude::*;

pub fn Home() -> Element {
    let nav = navigator();
    let base = &DOC_DIR;
    rsx! {
        div {
            class: "homepage",
            img { src: asset!("/assets/icon.svg"), height: "220px", width: "220px" }
            h1 { "Welcome to Scroll" }
            div {
                class: "navigation-buttons",
                button { onclick: move |_| {
                    FILE_DATA.write().goto(base.join("wet-lab"));
                    nav.push(Route::Viewer {});
                }, "Wet Lab" }

                button { onclick: move |_| {
                    FILE_DATA.write().goto(base.join("dry-lab"));
                    nav.push(Route::Viewer {});
                }, "Dry Lab" }
                
            }
        }
    }
}
