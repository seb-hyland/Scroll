#![allow(non_snake_case)]
use dioxus::prelude::*;
use crate::Route;

pub fn Home() -> Element {
    rsx! {
        Link { to: Route::FileExplorer{}, "Go to files" }
    }
}
