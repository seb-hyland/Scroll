use crate::prelude::*;

#[component]
pub fn Purgatory(error: String) -> Element {
    rsx! {
        "{error}"
    }
}

