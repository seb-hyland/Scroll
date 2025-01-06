use crate::types::input::InputField;
use std::path::PathBuf;

pub type MetadataVec = Vec<Vec<String>>;
pub type AttributeVec = Vec<(String, InputField)>;
pub type BreadcrumbVec = Vec<(PathBuf, String)>;
