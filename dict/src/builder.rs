use itertools::Itertools;

use crate::{
    char_category_def::CharCategoryDef, connection::ConnectionTable, dict, index, morph::Morphs,
    morph_feature,
};

use self::{config::Config, record::parse_csv};

pub mod char_def;
pub mod config;
pub mod matrix_def;
pub mod record;
pub mod unk;

pub fn build(config: &Config) -> dict::Dict {
    let csv_files = config
        .root_path
        .read_dir()
        .expect("Failed to read dir")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "csv"))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    let sorted_records = csv_files
        .into_iter()
        .flat_map(|csv| parse_csv(&csv, config.encoding).expect("Failed to parse csv"))
        .sorted()
        .collect::<Vec<_>>();

    let mut morphs = Morphs::new();
    let mut sorted_keywords = vec![];
    let mut morph_feature_table_builder = morph_feature::MorphFeatureTableBuilder::default();
    for record in &sorted_records {
        if record.cost > std::i16::MAX as i64 {
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

    let matrix =
        matrix_def::parse_matrix_def(&config.root_path.join(config.matrix_def_file_name))
            .expect("Failed to parse matrix.def");
    let connection_table = ConnectionTable::from(matrix);

    // index
    let index = index::IndexTable::build(&sorted_keywords).expect("Failed to build index");

    // char.def
    let char_class_def = char_def::parse_char_def(
        &config.root_path.join(config.char_def_file_name),
        config.encoding,
    )
    .expect("Failed to parse char.def");
    let char_category_def = CharCategoryDef::new(char_class_def);

    // unk.def
    let unk_def = parse_csv(
        &config.root_path.join(config.unk_def_file_name),
        config.encoding,
    )
    .expect("Failed to parse unk.def");
    dbg!(&unk_def);

    dict::Dict::new(
        morphs,
        morph_feature_table,
        connection_table,
        index,
        char_category_def,
    )
}
