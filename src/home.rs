#![allow(non_snake_case)]
use crate::prelude::*;


pub fn Home() -> Element {
    let base = DOC_DIR.read().unwrap().clone();
    let wetlab_path = base.join("wet-lab");
    let drylab_path = base.join("dry-lab");
    rsx! {
        div {
            class: "homepage",
            img { src: asset!("/assets/icon.svg"), height: "220px", width: "220px" }
            h1 { "Welcome to Scroll" }
            div {
                class: "navigation-buttons",
                button { onclick: move |_| {
                    FILE_DATA.write().goto(&wetlab_path);
                }, "Wet Lab" }

                button { onclick: move |_| {
                    FILE_DATA.write().goto(&drylab_path);
                }, "Dry Lab" }
                
            }
        }
    }
}
