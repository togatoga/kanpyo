use std::{fs, path::Path};

use anyhow::bail;
use encoding_rs::Encoding;

/// UnkDefRecord represents a record in the unk.def file.
#[derive(Debug, PartialEq, Clone, Eq, PartialOrd, Ord)]
pub struct UnkDefRecord {
    pub category: String,
    pub left_id: usize,
    pub right_id: usize,
    pub cost: i64,
    pub features: Vec<String>,
}

pub fn parse_unk_def(
    path: &Path,
    encoding: &'static Encoding,
) -> anyhow::Result<Vec<UnkDefRecord>> {
    let byte = fs::read(path)?;
    let (utf8, _, had_errors) = encoding.decode(&byte);
    if had_errors {
        bail!("Failed to decode");
    }
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(utf8.as_bytes());
    parse(&mut reader)
}

fn parse(reader: &mut csv::Reader<&[u8]>) -> anyhow::Result<Vec<UnkDefRecord>> {
    let mut records = Vec::new();
    for result in reader.records() {
        let record = result?;
        records.push(UnkDefRecord {
            category: record.get(0).unwrap().to_string(),
            left_id: record.get(1).unwrap().parse()?,
            right_id: record.get(2).unwrap().parse()?,
            cost: record.get(3).unwrap().parse()?,
            features: record.iter().skip(4).map(|s| s.to_string()).collect(),
        });
    }
    Ok(records)
}
