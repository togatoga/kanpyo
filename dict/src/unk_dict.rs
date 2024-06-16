use std::collections::BTreeMap;

use crate::{builder::unk, dict::DictReadWrite, morph, morph_feature, trie::da::KeywordID};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnkDict {
    pub morphs: morph::Morphs,
    pub morph_feature_table: morph_feature::MorphFeatureTable,
    pub char_category_to_morph_id: BTreeMap<u8, (KeywordID, usize)>,
}

impl UnkDict {
    pub fn build(
        mut records: Vec<unk::UnkDefRecord>,
        char_class: &Vec<String>,
    ) -> Result<Self, anyhow::Error> {
        records.sort();
        let mut morphs = morph::Morphs::new();
        let mut feature_table_builder = morph_feature::MorphFeatureTableBuilder::default();
        let mut char_category_to_morph_id = BTreeMap::new();
        for (morph_id, record) in records.iter().enumerate() {
            if record.cost > std::i16::MAX as i64 {
                return Err(anyhow::anyhow!("Cost is too large: {}", record.cost));
            }
            morphs.push(
                record.left_id as i16,
                record.right_id as i16,
                record.cost as i16,
            );
            // search index record.category in char_class;
            let char_category = char_class
                .iter()
                .position(|s| s == &record.category)
                .ok_or_else(|| anyhow::anyhow!("char category not found"))?
                as u8;
            char_category_to_morph_id
                .entry(char_category)
                .or_insert(((morph_id as KeywordID) + 1, 0))
                .1 += 1;

            feature_table_builder.push(
                &record
                    .features
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>(),
            );
        }
        Ok(UnkDict {
            morphs,
            morph_feature_table: feature_table_builder.build(),
            char_category_to_morph_id,
        })
    }
}

impl DictReadWrite for UnkDict {
    fn write_dict<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let size = self.char_category_to_morph_id.len();
        w.write_all(&size.to_le_bytes())?;
        for (&char_category, &(morph_id, count)) in self.char_category_to_morph_id.iter() {
            w.write_all(&char_category.to_le_bytes())?;
            w.write_all(&morph_id.to_le_bytes())?;
            w.write_all(&count.to_le_bytes())?;
        }

        self.morphs.write_dict(w)?;
        self.morph_feature_table.write_dict(w)?;
        Ok(())
    }
    fn from_dict<R: std::io::Read>(r: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut u8_buf = [0; 1];
        let mut buf = [0; 8];
        r.read_exact(&mut buf)?;
        let size = usize::from_le_bytes(buf);
        let mut char_category_to_morph_id = BTreeMap::new();
        for _ in 0..size {
            r.read_exact(&mut u8_buf)?;
            let char_category: u8 = u8::from_le_bytes(u8_buf);
            r.read_exact(&mut buf)?;
            let morph_id = KeywordID::from_le_bytes(buf);
            r.read_exact(&mut buf)?;
            let count = usize::from_le_bytes(buf);
            char_category_to_morph_id.insert(char_category, (morph_id, count));
        }
        let morphs = morph::Morphs::from_dict(r)?;
        let morph_feature_table = morph_feature::MorphFeatureTable::from_dict(r)?;
        Ok(UnkDict {
            morphs,
            morph_feature_table,
            char_category_to_morph_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_and_write() {
        let unk_dict = UnkDict {
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
        };
        let mut buf = Vec::new();
        unk_dict
            .write_dict(&mut buf)
            .expect("Failed to write unk_dict");
        let mut buf = std::io::Cursor::new(buf);
        let read_unk_dict = UnkDict::from_dict(&mut buf).expect("Failed to read unk_dict");
        assert_eq!(unk_dict.morphs, read_unk_dict.morphs);
    }
}
