//! Scroll is a documentation management and electronic lab notebook (ELN) tool, which relies on local Markdown editing and Git version control rather than online real-time collaboration.
//! _It is currently in beta development._
//!
//! View the Git repo [here](https://github.com/seb-hyland/Scroll).



#![allow(non_snake_case)]
#![feature(panic_payload_as_str)]
use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};
use std::{panic, path::PathBuf};
use native_dialog::{MessageDialog};

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

    panic::set_hook(Box::new(|panic_info| {
        let message: &str = match panic_info.payload_as_str() {
            Some(s) => s,
            None => "Unknown error occured"
        };

        MessageDialog::new()
            .set_title("Scroll")
            .set_text(&format!(
                "An error has occured. Please contact a lead with the following debug trace:\n\n{message}"))
            .show_alert()
            .unwrap();

        std::process::exit(1);
    }));

    dioxus::LaunchBuilder::desktop().with_cfg(cfg).launch(App)
}


#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
