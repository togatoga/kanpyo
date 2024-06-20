use std::path::Path;

use encoding_rs::Encoding;

#[derive(Debug, PartialEq)]
pub struct Config<'a> {
    pub root_path: &'a Path,
    pub encoding: &'static Encoding,

    // マトリクス定義ファイル名
    pub matrix_def_file_name: &'a str,
    // 文字種定義ファイル名
    pub char_def_file_name: &'a str,
    // 未知語定義ファイル名
    pub unk_def_file_name: &'a str,
}

impl<'a> Config<'a> {
    pub fn new(root_path: &'a Path, encoding: &'static Encoding) -> Self {
        Config {
            root_path,
            encoding,
            matrix_def_file_name: "matrix.def",
            char_def_file_name: "char.def",
            unk_def_file_name: "unk.def",
        }
    }
}
