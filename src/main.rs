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

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/files")]
    Viewer { init: String },
    #[route("/new")]
    Creator { from: PathBuf, attributes: Vec<(String, String)> }
}


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
