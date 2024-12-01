#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};
use std::path::PathBuf;

mod file_explorer;
mod home;
mod files;
mod new;
use crate::file_explorer::Viewer;
use crate::home::Home;
use crate::new::Creator;
use crate::files::FileData;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/files")]
    Viewer {},
    #[route("/new")]
    Creator {}
}

pub static FILE_DATA: GlobalSignal<FileData> = Global::new(|| FileData::new());


fn main() {
    let cfg = Config::new()
        .with_window(WindowBuilder::new().with_resizable(true).with_title("DocManager")).with_menu(None);

    dioxus::LaunchBuilder::desktop().with_cfg(cfg).launch(App)
}


#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
