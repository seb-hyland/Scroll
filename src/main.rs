#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};
use std::path::PathBuf;

mod files;
mod home;

use crate::files::FileExplorer;
use crate::home::Home;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/files")]
    FileExplorer { init: PathBuf },
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
