use crate::files::{DOC_DIR, InputField};
use crate::DATABASE_HOLD;
use dioxus::prelude::*;
use std::{collections::HashMap, path::PathBuf, fs::{read_to_string, read_dir}};
use eyre::{Result, Report};
use nom::{
    bytes::complete::{tag, take_until},
    error::ErrorKind,
    sequence::delimited};
use rayon::prelude::*;



pub fn parse_pairs(pairs: &str) -> Result<Vec<(&str, &str)>, usize> {
    pairs.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(i, line)| {
            let line_num = i + 1;
            let parts: Vec<&str> = line.split(":").collect();
            if parts.len() == 2 {
                assert!(parts.get(0).is_some() && parts.get(1).is_some(),
                    "Attribute key-value pair improperly validated.");
                Ok((parts.get(0).unwrap().trim(), parts.get(1).unwrap().trim()))
            } else {
                Err(line_num)
            }
        })
        .collect()
}


pub fn parse_attribute(mut s: &str) -> Result<InputField, String> {
    let asterisk = s.starts_with("*");
    if asterisk {
        s = &s[1..s.len()];    
    }
    let basic_parse = |keyword: &str| -> bool {
        tag::<_, _, (_, ErrorKind)>(keyword)(s).is_ok()
    };
    let advanced_parse = |stub: &str| -> Option<String> {
        let search_string = format!("{stub}("); 
        let result = delimited(
            tag::<_,_,(_, ErrorKind)>(search_string.as_str()),
            take_until(")"),
            tag(")")
        )(s);
        match result {
            Ok((_, parsed_content)) => {
                Some(parsed_content.to_string())
            }
            Err(_) => None,
        }
    };
        
    if basic_parse("String") {
        return Ok(InputField::String { req: asterisk });
    }
    if basic_parse("Date") {
        return Ok(InputField::Date { req: asterisk });
    }
    if let Some(capture) = advanced_parse("One") {
        return Ok(InputField::One { req: asterisk, id: capture });
    }
    if let Some(capture) = advanced_parse("Multi") {
        return Ok(InputField::Multi { req: asterisk, id: capture });
    }
    return Err("Malformed attribute syntax.".to_string());
}


/// Parses a list of attribute types into a Rust vector
///
/// # Props
/// - `list_id`: The name of the list file in `DOC_DIR/sys`
///
/// # Returns
/// - `Ok` if the file is successfully parsed
/// - `Err(e)` if the file cannot be found or JSON parsing fails
///     - `e` is of type [`eyre::Report`]
pub fn collect_options(list_id: &str) -> Result<Vec<String>> {
    let search_result = DATABASE_HOLD
        .get(list_id)
        .ok_or_else(|| Report::msg("Key not found in the database"))?;
    let result = search_result.as_ref()
        .map_err(|_| Report::msg("File not properly read"))?
        .1
        .clone();
    Ok(result)
}


pub fn collect_table(list_id: &str) -> Result<Vec<Vec<String>>> {
    let search_result = DATABASE_HOLD
        .get(list_id)
        .ok_or_else(|| Report::msg("Key not found in the database"))?;
    let result = search_result.as_ref()
        .map_err(|_| Report::msg("File not properly read"))?
        .0
        .clone();
    Ok(result)
}


pub fn parse_all_databases() -> Result<HashMap<String, Result<(Vec<Vec<String>>, Vec<String>)>>> {
    let base_path = DOC_DIR.join("sys");
    let databases: Vec<PathBuf> = read_dir(base_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "scroll"))
        .map(|entry| entry.path())
        .collect();

    let results = databases.par_iter()
        .map(|path| {
            let mut name = path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let data_result = parse_db(path);
            (name, data_result)
        })
        .collect();

    Ok(results)
}


fn parse_db(path: &PathBuf) -> Result<(Vec<Vec<String>>, Vec<String>)> {
    let content = read_to_string(path)?;

    let mut data_tuple = content.lines()
        .map(|line| {
            let split = line.split(", ");
            let first = split.clone().next().unwrap_or_default().to_string();
            let all = split.map(String::from).collect::<Vec<String>>();
            (all, first)
        })
        .collect::<(Vec<Vec<String>>, Vec<String>)>();

    if !data_tuple.1.is_empty() {
        data_tuple.1.remove(0);
    }

    Ok(data_tuple)
}
