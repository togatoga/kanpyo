use std::{
    fs::{self},
    io::{BufRead, BufReader},
    path::Path,
};

use encoding_rs::Encoding;
use regex::Regex;

use crate::error::{KanpyoError, Result};

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct CharClassDef {
    pub char_class: Vec<String>,
    pub char_category: Vec<u8>,
    pub invoke_map: Vec<bool>,
    pub group_map: Vec<bool>,
}

pub fn parse_char_def(path: &Path, encoding: &'static Encoding) -> Result<CharClassDef> {
    let byte = fs::read(path)?;
    let (utf8, _, had_errors) = encoding.decode(&byte);
    if had_errors {
        return Err(KanpyoError::EncodingError);
    }

    let reader = BufReader::new(utf8.as_bytes());
    parse(reader)
}

fn parse(reader: BufReader<&[u8]>) -> Result<CharClassDef> {
    let mut char_class = Vec::new();
    let mut char_category = vec![0; 1 << 16];
    let mut invoke_map = Vec::new();
    let mut group_map = Vec::new();
    let mut cc2id: std::collections::HashMap<String, u8> = std::collections::HashMap::new();

    // e.g. C 1 1 1
    let re_char_class =
        Regex::new(r"^(\w+)\s+(\d+)\s+(\d+)\s+(\d+)").expect("Failed to compile regex");
    // e.g. 0x0000 0x007F 1 1  # C
    let re_char_category = Regex::new(r"^(0x[0-9A-F]+)(?:\s+([^#\s]+))(?:\s+([^#\s]+))?")
        .expect("Failed to compile regex");
    // e.g. 0x0000..0x007F 1 1  # C
    let re_char_category_range =
        Regex::new(r"^(0x[0-9A-F]+)..(0x[0-9A-F]+)(?:\s+([^#\s]+))(?:\s+([^#\s]+))?")
            .expect("Failed to compile regex");

    for line in reader
        .lines()
        .map(|line| line.expect("Failed to read line").trim().to_string())
        .filter(|line| !line.starts_with('#') && !line.is_empty())
    {
        if let Some(matches) = re_char_class.captures(&line) {
            let cc = matches
                .get(1)
                .expect("Failed to get character category")
                .as_str();
            let invoke = matches.get(2).expect("Failed to get invoke").as_str();
            let group = matches.get(3).expect("Failed to get group").as_str();
            invoke_map.push(invoke == "1");
            group_map.push(group == "1");
            cc2id.insert(cc.to_string(), char_class.len() as u8);
            char_class.push(cc.to_string());
        } else if let Some(matches) = re_char_category.captures(&line) {
            let ch = u32::from_str_radix(matches[1].trim_start_matches("0x"), 16)?;
            let cc = matches
                .get(2)
                .expect("Failed to get character category")
                .as_str();
            char_category[ch as usize] = cc2id[cc];
        } else if let Some(matches) = re_char_category_range.captures(&line) {
            // let start = u32::from_str_radix(matches[1].trim_start_matches("0x"), 16)?;
            // let end = u32::from_str_radix(matches[2].trim_start_matches("0x"), 16)?;
            let start = matches.get(1).expect("Failed to get start").as_str();
            let end = matches.get(2).expect("Failed to get end").as_str();
            let cc = matches
                .get(3)
                .expect("Failed to get character category")
                .as_str();
            let start = u32::from_str_radix(start.trim_start_matches("0x"), 16)?;
            let end = u32::from_str_radix(end.trim_start_matches("0x"), 16)?;
            for x in start..=end {
                char_category[x as usize] = cc2id[cc];
            }
        } else {
            return Err(KanpyoError::InvalidFormat(format!(
                "Invalid char.def format: {}",
                line
            )));
        }
    }
    Ok(CharClassDef {
        char_class,
        char_category,
        invoke_map,
        group_map,
    })
}
