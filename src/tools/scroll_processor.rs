use crate::prelude::*;
use std::fs::{read_to_string, read_dir};
use nom::{
    bytes::complete::{tag, take_until},
    error::ErrorKind,
    sequence::delimited};



pub fn parse_pairs(pairs: &str) -> Result<Vec<(&str, &str)>, usize> {
    pairs.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(i, line)| {
            let line_num = i + 1;
            let parts: Vec<&str> = line.split(":").collect();
            if parts.len() == 2 {
                assert!(parts.get(0).is_some() && parts.get(1).is_some(),
                    "ERR[0|0]: Attribute key-value pair improperly validated.");
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
pub fn db_query(list_id: &str) -> Result<(Vec<Vec<String>>, Vec<String>), String> {
    let guard = DATABASE_HOLD.read().unwrap();
    let search_result = guard
        .get(list_id)
        .ok_or("ERR(0|1): Key not found in the database".to_string())?;
    if search_result.0.is_empty() || search_result.1.is_empty() {
        return Err(format!("ERR(0|1): Database {list_id} is empty!"));
    }
    Ok(search_result.clone())
}


pub fn parse_all_databases() -> Result<HashMap<String, (Vec<Vec<String>>, Vec<String>)>> {
    let base_path = DOC_DIR.read().unwrap().join("sys");
    let databases: Vec<PathBuf> = read_dir(base_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "scroll"))
        .map(|entry| entry.path())
        .collect();

    let results: Result<HashMap<String, (Vec<Vec<String>>, Vec<String>)>> = databases.par_iter()
        .map(|path| {
            let  name = path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let data_result = parse_db(path)?;
            Ok((name, data_result))
        })
        .collect();

    results
}


fn parse_db(path: &PathBuf) -> Result<(Vec<Vec<String>>, Vec<String>)> {
    let content = read_to_string(path)?;
    let first_line: Vec<&str> = content.lines()
        .next().ok_or(Report::msg(format!("Database at {} is empty", path.display())))?
        .split(", ").collect();
    let width = first_line.len();

    let result = content.lines()
        .map(|line| {
            let split = line.split(", ");
            let first = split.clone().next().unwrap_or_default().to_string();
            let all = split.map(String::from).collect::<Vec<String>>();
            if all.len() == width {
                Ok((all, first))
            } else {
                Err(Report::msg(format!("Database at {} has uneven widths", path.display())))
            }
        })
        .collect::<Result<(Vec<Vec<String>>, Vec<String>)>>();

    let mut data_tuple: (Vec<Vec<String>>, Vec<String>) = result?;

    if !data_tuple.1.is_empty() {
        data_tuple.1.remove(0);
    }

    Ok(data_tuple)
}
