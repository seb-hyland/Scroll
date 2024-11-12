#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};

mod files;
mod home;

use crate::files::FileExplorer;
use crate::home::Home;

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/files")]
    FileExplorer {},
}


fn main() {
    let cfg = Config::new()
        .with_window(WindowBuilder::new().with_resizable(true))
        .with_custom_head(r#"<link rel="stylesheet" href="./assets/main.css"> <style> @import url('https://fonts.googleapis.com/css2?family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap'); </style>"#.to_string());

    dioxus::LaunchBuilder::desktop().with_cfg(cfg).launch(App)
}


#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
