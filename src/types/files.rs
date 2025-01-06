#![allow(non_snake_case)]
use crate::{
    prelude::*,
    tools::compare,
};
use std::{
    fs::{read_to_string, read_dir},
};



#[derive(Clone, Debug)]
pub struct FileData {
    pub current_path: PathBuf,
    path_contents: Vec<PathBuf>,
    pub directories: Vec<PathBuf>,
    pub metadata: MetadataVec,
    pub attributes: AttributeVec,
    pub breadcrumbs: BreadcrumbVec,
    pub ordering: Order,
}


#[derive(Clone, Debug)]
pub struct Order {
    direction: SortDirection,
    id: usize,
}


#[derive(Clone, Debug)]
enum SortDirection {
    Increasing,
    Decreasing,
}



impl FileData {
    pub fn new() -> Self {
        let mut files = Self {
            current_path: DOC_DIR.read().unwrap().clone(),
            attributes: Vec::new(),
            path_contents: Vec::new(),
            directories: Vec::new(),
            metadata: Vec::new(),
            breadcrumbs: Vec::new(),
            ordering: Order {direction: SortDirection::Increasing, id: 0},
        };
        files.refresh();
        files
    }


    pub fn refresh(&mut self) -> Result<(), String> {
        let current_dir = &self.current_path;
        let dir_entries = read_dir(current_dir).map_err(|e| e.to_string())?;
        self.clear();

        self.path_contents.extend(
            dir_entries
                .filter_map(|entry| entry.ok()
                .map(|e| e.path())));
        self.directories = self.get_directories();
        self.breadcrumbs = self.get_breadcrumbs()?;
        self.attributes = self.get_attributes()?;
        self.metadata = self.get_metadata()?;
        match self.ordering.direction {
            SortDirection::Increasing => {
                self.metadata.sort_by(|a, b| compare::increasing(a, b, self.ordering.id));
            }
            SortDirection::Decreasing => {
                self.metadata.sort_by(|a, b| compare::decreasing(a, b, self.ordering.id));
            }
        };
        Ok(())
    }


    fn clear(&mut self) {
        self.path_contents.clear();
    }


    fn get_directories(&self) -> Vec<PathBuf> {
        let directories: Vec<PathBuf> = self.path_contents.iter()
            .filter(|p| p.is_dir())
            .cloned()
            .collect();
        directories
    }


    fn get_breadcrumbs(&self) -> Result<BreadcrumbVec, String> {
        let mut base_path = DOC_DIR.read().map_err(|e| e.to_string())?.clone();
        base_path.pop();
        let relative_path = self.current_path.strip_prefix(&base_path).map_err(|e| e.to_string())?;

        let mut accumulator = base_path;
        let breadcrumbs: BreadcrumbVec = relative_path
            .components()
            .map(|component| {
                accumulator.push(component);
                (accumulator.clone(), component.as_os_str().to_string_lossy().into_owned())
            })
            .collect();
        Ok(breadcrumbs)
    }


    fn get_attributes(&self) -> Result<AttributeVec, String> {
        let attribute_file = self.current_path.join(".attributes.scroll");
        let data = match read_to_string(&attribute_file) {
            Ok(v) => v,
            Err(_) => return Ok(Vec::new()),
        };

        let err_stub = format!("Attribute parsing failed. File: \"{}\".\n|ADDITIONAL INFO| ", attribute_file.display());
        let err_report = |database, line_number| format!(
            "{err_stub}\nLine: {line_number}.\nInternal: {database}.");

        let trimmed_data: Vec<(&str, &str)> = scroll_processor::parse_pairs(&data)
            .map_err(|e| err_report("".to_string(), e))?;

        // Parse the components of each line
        trimmed_data.iter().enumerate()
            .map(|(i, (title, raw_type))| {
                let line_num = i + 1;
                let attr_type: InputField = scroll_processor::parse_attribute(raw_type)
                    .map_err(|e| err_report(e, line_num))?;
                Ok((title.to_string(), attr_type))
            })
            .collect()
    }


    fn get_metadata(&self) -> Result<MetadataVec, String> {
        if self.attributes.is_empty() {
            return Ok(Vec::new());
        }

        let db_path = self.current_path.join(".database.json");
        let objects = json_processor::get_json_hashmap(&db_path).map_err(|e| e.to_string())?;

        let result = objects.par_iter()
            .map(|(_, map)| {
                let mut struct_metadata: Vec<String> = Vec::new();
                struct_metadata.push(map
                    .get("__ID")
                    .cloned()
                    .ok_or("ID field not found in JSON".to_string())?);
                
                for (attribute, _) in self.attributes.iter() {
                    struct_metadata.push(map
                        .get(attribute)
                        .cloned()
                        .ok_or(format!("{attribute} field not found in JSON"))?);
                }
                Ok(struct_metadata)
            })
            .collect::<Result<MetadataVec, String>>();
        result
    }


    pub fn goto(&mut self, path: &PathBuf) {
        assert!(path.is_dir(), "Attempted navigation to a non-directory file");
        self.current_path = path.clone();
        self.refresh();
    }
}

