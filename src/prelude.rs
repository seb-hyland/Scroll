pub use dioxus::prelude::*;
pub use eyre::{Result, Report};
pub use std::{
    collections::HashMap,
    sync::LazyLock,
    path::PathBuf,
};
pub use rayon::prelude::*;

pub use crate::{
    Route,
    tools::{
        serde::*,
        scroll_processor,
        json_processor,
    },
    types::{
        aliases::*,
        input::*,
        statics::*,
        generator::FileGenerator,
        files::FileData,
    },
};
