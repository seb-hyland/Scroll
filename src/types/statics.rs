use crate::prelude::*;
use std::sync::RwLock;


pub static DOC_DIR: LazyLock<RwLock<PathBuf>> = LazyLock::new(|| RwLock::new(PathBuf::new()));


pub static DATABASE_HOLD: LazyLock<RwLock<HashMap<String, (Vec<Vec<String>>, Vec<String>)>>> = LazyLock::new(
    || RwLock::new(HashMap::new())
);


pub static FILE_DATA: GlobalSignal<FileData> = Global::new(|| FileData::new());

pub static POPUP_GENERATOR: GlobalSignal<FileGenerator> = Global::new(|| FileGenerator::new());
