use eyre::{Report, Result};
use rayon::prelude::*;
use serde_json::{Value, Map};
use std::{
    collections::HashMap,
    fs::read_to_string,
    path::PathBuf,
};



pub fn get_json_hashmap(path: &PathBuf) -> Result<HashMap<String, HashMap<String, String>>> {
        let db_path = path.join(".database.json");

        if !db_path.exists() {
            return Ok(HashMap::new());
        }

        let data = read_to_string(&db_path)?;
        let parsed_binding: Value = serde_json::from_str(&data)?;
        match parsed_binding {
            Value::Array(file_structs) => {
                let result = file_structs
                    .par_iter()
                    .map(|file_struct| {
                        match file_struct {
                            // For each file object...
                            Value::Object(map) => {
                                // "Correct" the map by removing quotation marks and fixing type
                                let corrected_map = map.iter()
                                    .map(|(k, v)| {
                                        let k_trimmed = k.trim_matches('"');
                                        let v_string = v.to_string();
                                        let v_trimmed = v_string.trim_matches('"');
                                        (k_trimmed.to_string(), v_trimmed.to_string())
                                    })
                                    .collect::<HashMap<String, String>>();
                                let id = corrected_map
                                    .get("__ID")
                                    .cloned()
                                    .unwrap_or("".to_string());
                                Ok((id, corrected_map))
                            },
                            _ => Err(Report::msg("JSON parsing failed"))
                        }
                    })
                    .collect::<Result<HashMap<String, HashMap<String, String>>>>()?;
                Ok(result)
            },
            _ => Err(Report::msg("JSON parsing failed"))
        }
}


pub fn hashmap_to_vec(json: &HashMap<String, HashMap<String, String>>) -> Vec<Vec<(String, String)>> {
    json 
        .into_iter()
        .map(|(_, M)| {
            M.into_iter()
                .map(|(K, V)| (K.clone(), V.clone()))
                .collect()
        })
        .collect()
}


pub fn vec_to_json(vector: &Vec<Vec<(String, String)>>) -> Vec<Value> {
    vector.into_iter().map(|vec| {
        let mut map = Map::new();
        for (K, V) in vec {
            map.insert(K.clone(), Value::String(V.clone()));
        }
        Value::Object(map)
    }).collect()
}


pub fn update_json_hashmap(map: &mut HashMap<String, HashMap<String, String>>, name: &str, contents: Vec<(String, String)>) {
    let content_map: HashMap<String, String> = contents.into_iter().collect();
    map.insert(name.to_string(), content_map);
}


pub fn delete_from_hashmap(map: &mut HashMap<String, HashMap<String, String>>, name: &str) {
    map.remove(name);
}


pub fn sort_json(vec: &mut Vec<Value>) {
    let id = "__ID";
    vec.sort_by(|a, b| {
        assert!(a.get(id).is_some() && b.get(id).is_some(), "Metadata is missing __ID tags");
        a[id].to_string().cmp(&b[id].to_string())
    });
}
