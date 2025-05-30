use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use itertools::Itertools;
use unk::parse_unk_def;

use crate::{
    char_category_def::CharCategoryDef, connection::ConnectionTable, dict, index, morph::Morphs,
    morph_feature, unk_dict,
};

use self::{config::Config, record::parse_csv};

pub mod char_def;
pub mod config;
pub mod matrix_def;
pub mod record;
pub mod unk;

pub struct DictionaryBuilder {}

impl DictionaryBuilder {
    fn collect_csv_files(config: &Config) -> Result<Vec<PathBuf>> {
        let csv_files = fs::read_dir(config.root_path)
            .with_context(|| format!("Failed to read directory: {:?}", config.root_path))?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                path.extension()
                    .is_some_and(|ext| ext == "csv")
                    .then_some(path)
            })
            .collect::<Vec<_>>();

        Ok(csv_files)
    }

    pub fn from_config(config: &Config) -> Result<dict::Dict, anyhow::Error> {
        let csv_files = Self::collect_csv_files(config)?;

        let sorted_records = csv_files
            .into_iter()
            .flat_map(|csv| parse_csv(&csv, config.encoding).expect("Failed to parse csv"))
            .sorted()
            .collect::<Vec<_>>();

        let mut morphs = Morphs::new();
        let mut sorted_keywords = vec![];
        let mut morph_feature_table_builder = morph_feature::MorphFeatureTableBuilder::default();
        for record in &sorted_records {
            if record.cost > i16::MAX as i64 {
                panic!("Cost is too large: {}", record.cost);
            }
            sorted_keywords.push(record.surface.clone());
            morphs.push(
                record.left_id as i16,
                record.right_id as i16,
                record.cost as i16,
            );
            // 品詞
            // 品詞細分類1
            // 品詞細分類2
            // 品詞細分類3
            let morph_features = &record
                .user_data
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>();
            morph_feature_table_builder.push(morph_features);
        }
        let morph_feature_table = morph_feature_table_builder.build();
        let connection_table = ConnectionTable::from(
            matrix_def::parse_matrix_def(&config.root_path.join(config.matrix_def_file_name))
                .expect("Failed to parse matrix.def"),
        );

        // index
        let index = index::IndexTable::build(&sorted_keywords).expect("Failed to build index");

        // char.def
        let char_category_def = CharCategoryDef::new(
            char_def::parse_char_def(
                &config.root_path.join(config.char_def_file_name),
                config.encoding,
            )
            .expect("Failed to parse char.def"),
        );

        // unk.def
        let unk_dict = unk_dict::UnkDict::build(
            parse_unk_def(
                &config.root_path.join(config.unk_def_file_name),
                config.encoding,
            )
            .expect("Failed to parse unk.def"),
            &char_category_def.char_class,
        )
        .expect("Failed to build unk dict");

        Ok(dict::Dict::new(
            morphs,
            morph_feature_table,
            connection_table,
            index,
            char_category_def,
            unk_dict,
        ))
    }
}
