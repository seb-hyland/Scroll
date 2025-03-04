//! Scroll is a documentation management and electronic lab notebook (ELN) tool, which relies on local Markdown editing and Git version control rather than online real-time collaboration.
//! _It is currently in beta development._
//!
//! View the Git repo [here](https://github.com/seb-hyland/Scroll).



#![allow(non_snake_case)]
#![feature(panic_payload_as_str)]
use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};

mod prelude;
mod load;
mod abyss;
mod home;
mod file_explorer;
mod metadata_popup;
mod db_popup;
mod tools;
mod types;

use crate::{
    load::Loader,
    abyss::Purgatory,
    file_explorer::Viewer,
    tools::custom_panic,
};



#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/")]
    Loader {},
    #[route("/purgatory/:error")]
    Purgatory { error: String },
    #[route("/files")]
    Viewer {},
}



static CSS: Asset = asset!("/assets/main.css");
static DARK_CSS: Asset = asset!("/assets/dark.css");
static INTER_API: &str = "fonts.googleapis.com/css2?family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap";
    


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

