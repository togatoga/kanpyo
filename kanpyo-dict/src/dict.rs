use std::io::Read;
use std::io::Seek;
use std::io::Write;

use crate::char_category_def;
use crate::connection;
use crate::index;
use crate::morph;
use crate::morph_feature;
use crate::unk_dict;

pub trait DictReadWrite {
    fn from_dict<R: std::io::Read>(r: &mut R) -> std::io::Result<Self>
    where
        Self: Sized;
    fn write_dict<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()>;
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct Dict {
    pub morphs: morph::Morphs,
    pub morph_feature_table: morph_feature::MorphFeatureTable,
    // contents_meta
    // contents
    pub connection_table: connection::ConnectionTable,
    pub index_table: index::IndexTable,
    pub char_category_def: char_category_def::CharCategoryDef,
    pub unk_dict: unk_dict::UnkDict,
}

impl Dict {
    pub fn new(
        morphs: morph::Morphs,
        morph_feature_table: morph_feature::MorphFeatureTable,
        connection_table: connection::ConnectionTable,
        index: index::IndexTable,
        char_category_def: char_category_def::CharCategoryDef,
        unk_dict: unk_dict::UnkDict,
    ) -> Self {
        Dict {
            morphs,
            morph_feature_table,
            connection_table,
            index_table: index,
            char_category_def,
            unk_dict,
        }
    }

    pub fn build<W: Write + Seek>(&self, f: &mut W) -> anyhow::Result<()> {
        let mut zip = zip::ZipWriter::new(f);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);
        zip.start_file("morph.dict", options)?;
        self.morphs.write_dict(&mut zip)?;
        zip.start_file("morph_feature.dict", options)?;
        self.morph_feature_table.write_dict(&mut zip)?;
        zip.start_file("connection.dict", options)?;
        self.connection_table.write_dict(&mut zip)?;
        zip.start_file("index.dict", options)?;
        self.index_table.write_dict(&mut zip)?;
        zip.start_file("chardef.dict", options)?;
        self.char_category_def.write_dict(&mut zip)?;
        zip.start_file("unk.dict", options)?;
        self.unk_dict.write_dict(&mut zip)?;
        Ok(())
    }
    pub fn load<R: Read + Seek>(r: &mut R) -> anyhow::Result<Self> {
        let mut zip = zip::ZipArchive::new(r)?;
        let morphs = {
            let morph_dict = zip.by_name("morph.dict")?;
            let mut r = std::io::BufReader::new(morph_dict);
            morph::Morphs::from_dict(&mut r)?
        };

        let morph_feature_table = {
            let pos_dict = zip.by_name("morph_feature.dict")?;
            let mut r = std::io::BufReader::new(pos_dict);
            morph_feature::MorphFeatureTable::from_dict(&mut r)?
        };

        let connection_table = {
            let connection_dict = zip.by_name("connection.dict")?;
            let mut r = std::io::BufReader::new(connection_dict);
            connection::ConnectionTable::from_dict(&mut r)?
        };

        let index = {
            let index_dict = zip.by_name("index.dict")?;
            let mut r = std::io::BufReader::new(index_dict);
            index::IndexTable::from_dict(&mut r)?
        };

        let char_category_def = {
            let chardef_dict = zip.by_name("chardef.dict")?;
            let mut r = std::io::BufReader::new(chardef_dict);
            char_category_def::CharCategoryDef::from_dict(&mut r)?
        };

        let unk_dict = {
            let unk_dict = zip.by_name("unk.dict")?;
            let mut r = std::io::BufReader::new(unk_dict);
            unk_dict::UnkDict::from_dict(&mut r)?
        };

        Ok(Dict::new(
            morphs,
            morph_feature_table,
            connection_table,
            index,
            char_category_def,
            unk_dict,
        ))
    }
}

#[cfg(test)]
mod tests {

    use crate::builder::matrix_def;

    use super::*;

    fn new_test_dict() -> Dict {
        let index =
            index::IndexTable::build(&["key1".to_string(), "key2".to_string(), "key3".to_string()])
                .expect("Failed to build index table");

        Dict {
            morphs: morph::Morphs::from(vec![
                morph::Morph::new(111, 222, 333),
                morph::Morph::new(444, 555, 666),
            ]),
            morph_feature_table: morph_feature::MorphFeatureTableBuilder::from(vec![
                vec!["str1", "str2", "str3", "str3", "str4", "str5"],
                vec!["str1", "str2", "str3", "str6", "str7", "str8"],
            ])
            .build(),
            connection_table: connection::ConnectionTable::from(matrix_def::MatrixDef {
                row: 2,
                col: 3,
                data: vec![0, 1, 2, 3, 4, 5],
            }),
            index_table: index,
            char_category_def: char_category_def::CharCategoryDef {
                char_class: vec![
                    "class1".to_string(),
                    "class2".to_string(),
                    "class3".to_string(),
                ],
                char_category: vec![b'a', b'b', b'c'],
                invoke_list: vec![true, false, true],
                group_list: vec![false, true, false],
            },
            unk_dict: unk_dict::UnkDict {
                morphs: morph::Morphs::from(vec![
                    morph::Morph::new(1, 2, 3),
                    morph::Morph::new(11, 22, 33),
                ]),
                morph_feature_table: morph_feature::MorphFeatureTableBuilder::from(vec![
                    vec!["hello", "goodbye"],
                    vec!["こんにちは", "さようなら"],
                ])
                .build(),
                char_category_to_morph_id: vec![(1, (1, 1)), (2, (2, 2))].into_iter().collect(),
            },
        }
    }

    #[test]
    fn test_build_load() {
        let org = new_test_dict();
        let mut cursor = std::io::Cursor::new(Vec::new());
        org.build(&mut cursor).expect("Failed to build dict");
        let cpy = Dict::load(&mut cursor).expect("Failed to load dict");
        assert_eq!(org, cpy);
    }
}
