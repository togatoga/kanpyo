use encoding_rs::Encoding;
use std::{fs, path::Path};
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Record represents a record in the CSV file.
pub struct Record {
    // 表層系
    pub surface: String,
    // 左文脈ID
    pub left_id: usize,
    // 右文脈ID
    pub right_id: usize,
    // 単語コスト
    pub cost: i64,
    // 5カラム以降は素性
    // ref: https://taku910.github.io/mecab/learn.html
    pub user_data: Vec<String>,
}

pub fn parse_csv(path: &Path, encoding: &'static Encoding) -> Result<Vec<Record>, anyhow::Error> {
    let byte = fs::read(path)?;
    let (utf8, _, had_errors) = encoding.decode(&byte);
    if had_errors {
        return Err(anyhow::anyhow!("Failed to decode"));
    }
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(utf8.as_bytes());
    let mut records = Vec::new();
    for result in reader.records() {
        let record = result?;
        records.push(Record {
            surface: record.get(0).unwrap().to_string(),
            left_id: record.get(1).unwrap().parse()?,
            right_id: record.get(2).unwrap().parse()?,
            cost: record.get(3).unwrap().parse()?,
            user_data: record.iter().skip(4).map(|s| s.to_string()).collect(),
        });
    }
    Ok(records)
}
