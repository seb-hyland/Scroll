//! Scroll is a documentation management and electronic lab notebook (ELN) tool, which relies on local Markdown editing and Git version control rather than online real-time collaboration.
//! _It is currently in beta development._
//!
//! View the Git repo [here](https://github.com/seb-hyland/Scroll).



#![allow(non_snake_case)]
#![feature(panic_payload_as_str)]
use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};
use std::{
    collections::HashMap,
    sync::LazyLock,
};
use eyre::Result;

mod file_explorer;
mod home;
mod files;
mod new;
mod tools;
use crate::file_explorer::Viewer;
use crate::home::Home;
use crate::files::FileData;
use crate::tools::custom_panic;
use crate::tools::scroll_processor;



#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/files")]
    Viewer {},
}


pub static FILE_DATA: GlobalSignal<FileData> = Global::new(|| FileData::new());


static CSS: Asset = asset!("/assets/main.css");
static DARK_CSS: Asset = asset!("/assets/dark.css");
static INTER_API: &str = "fonts.googleapis.com/css2?family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap";


pub static DATABASE_HOLD: LazyLock<HashMap<String, Result<(Vec<Vec<String>>, Vec<String>)>>> = LazyLock::new(|| {
    let result = scroll_processor::parse_all_databases();
    assert!(result.is_ok(), "Database directory could not be accessed. Please check it exists and Scroll has the correct permissions.");
    result.unwrap()
});
    


fn main() {
    let cfg = Config::new()
        .with_window(WindowBuilder::new()
            .with_resizable(true)
            .with_title("Scroll"))
        .with_menu(None);

    custom_panic::set_custom_panic();

    dioxus::LaunchBuilder::desktop().with_cfg(cfg).launch(App)
}


#[component]
fn App() -> Element {
    rsx! {
	document::Stylesheet { href: CSS },
	document::Stylesheet { href: DARK_CSS },
        document::Style { href: INTER_API },
        Router::<Route> {}
    }
}

