use crate::files::{DOC_DIR, InputField};
use std::{path::PathBuf, fs::read_to_string};
use eyre::Result;
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
        let option_list = parse_list(&capture)
            .map_err(|_| format!("|Malformed database: {capture}.|"))?;
        return Ok(InputField::One { req: asterisk, options: option_list });
    }
    if let Some(capture) = advanced_parse("Multi") {
        let option_list = parse_list(&capture)
            .map_err(|_| format!("|Malformed database: {capture}.|"))?;
        return Ok(InputField::Multi { req: asterisk, options: option_list });
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
fn parse_list(list_id: &str) -> Result<Vec<String>> {
    let file_path: PathBuf = DOC_DIR
        .join("sys")
        .join(list_id)
        .with_extension("scroll");
    let file_contents = read_to_string(&file_path)?;
    Ok(file_contents.lines()
        .map(String::from)
        .collect())
}
