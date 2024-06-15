use serde::{Deserialize, Serialize};

use crate::dict::DictReadWrite;

// MorphFeatureTable represents a table for managing part of speeches.
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MorphFeatureTable {
    pub morph_features: Vec<MorphFeatureIDs>,
    pub name_list: Vec<String>,
}

// MorphFeatureID represents a ID of part of speech.
pub type MorphFeatureID = u32;

// MorphFeature represents a vector of part of speech.
pub type MorphFeatureIDs = Vec<MorphFeatureID>;

const MAX_FEATURE_ID: MorphFeatureID = MorphFeatureID::MAX;

impl DictReadWrite for MorphFeatureTable {
    fn write_dict<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        match bincode::serialize(self) {
            Ok(enc) => w.write_all(&enc),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }
    fn from_dict<R: std::io::Read>(r: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = Vec::new();
        r.read_to_end(&mut buf)?;
        match bincode::deserialize(&buf) {
            Ok(ret) => Ok(ret),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }
}
#[derive(Debug, Default)]
pub struct MorphFeatureTableBuilder(
    std::collections::HashMap<String, MorphFeatureID>,
    Vec<MorphFeatureIDs>,
);

impl MorphFeatureTableBuilder {
    // adds part of speech item to the MorphFeature control table and returns it's id.
    pub fn push(&mut self, pos: &[&str]) {
        let mut ret = vec![];
        for name in pos {
            let id = self.insert(name);
            ret.push(id);
        }
        self.1.push(ret);
    }

    fn insert(&mut self, pos: &str) -> MorphFeatureID {
        if let Some(&id) = self.0.get(pos) {
            id
        } else {
            if self.0.len() > MAX_FEATURE_ID as usize {
                panic!(
                    "new MorphFeatureID overflowed {} {} > {}",
                    pos,
                    self.0.len(),
                    MAX_FEATURE_ID
                );
            }
            let id = self.0.len() as MorphFeatureID + 1;
            self.0.insert(pos.to_string(), id);
            id
        }
    }

    // List returns a list whose index is MorphFeature ID and value is its name.
    fn list(&self) -> Vec<String> {
        let mut ret = vec![String::new(); self.0.len() + 1];
        for (k, &v) in &self.0 {
            ret[v as usize].clone_from(k);
        }
        ret
    }

    // build returns a MorphFeatureTable from MorphFeature control table.
    pub fn build(self) -> MorphFeatureTable {
        let morph_features = self.1.clone();
        let name_list = self.list();
        MorphFeatureTable {
            morph_features,
            name_list,
        }
    }
}

impl From<Vec<Vec<&str>>> for MorphFeatureTableBuilder {
    fn from(v: Vec<Vec<&str>>) -> Self {
        let mut builder = MorphFeatureTableBuilder::default();
        for pos in v {
            builder.push(&pos);
        }
        builder
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_push() {
        let data = [
            (
                vec!["動詞", "自立", "*", "*", "五段・マ行", "基本形"],
                vec![1, 2, 3, 3, 4, 5],
            ),
            (
                vec!["動詞", "接尾", "*", "*", "五段・サ行", "未然形"],
                vec![1, 6, 3, 3, 7, 8],
            ),
            (vec!["一般", "*", "*", "*", "*"], vec![9, 3, 3, 3, 3]),
            (
                vec!["動詞", "自立", "*", "*", "五段・マ行", "未然形"],
                vec![1, 2, 3, 3, 4, 8],
            ),
        ];
        let mut builder = MorphFeatureTableBuilder::default();
        for (input, _) in data.iter() {
            builder.push(input);
        }
        let table = builder.build();
        for (i, (_, want)) in data.iter().enumerate() {
            assert_eq!(table.morph_features[i], *want, "{}", i);
        }
    }

    #[test]
    fn test_list() {
        let data = vec![
            vec!["動詞", "接尾", "*", "*"],
            vec!["動詞", "接尾", "*", "*", "五段・サ行,未然形"],
            vec!["自立", "*", "*", "五段・マ行,基本形"],
            vec!["動詞", "自立", "*", "*", "五段・マ行,未然形"],
        ];

        let table = MorphFeatureTableBuilder::from(data.clone()).build();
        for (i, want) in data.iter().enumerate() {
            table.morph_features[i]
                .iter()
                .zip(want.iter())
                .for_each(|(id, name)| assert_eq!(table.name_list[*id as usize], *name));
        }
    }

    #[test]
    fn test_read_and_write() {
        let data = vec![
            vec!["動詞", "接尾", "*", "*"],
            vec!["動詞", "接尾", "*", "*", "五段・サ行,未然形"],
            vec!["自立", "*", "*", "五段・マ行,基本形"],
            vec!["動詞", "自立", "*", "*", "五段・マ行,未然形"],
        ];
        let org = MorphFeatureTableBuilder::from(data.clone()).build();
        let mut buf = Vec::new();
        org.write_dict(&mut buf)
            .expect("Failed to write MorphFeatureTable");

        let cpy = MorphFeatureTable::from_dict(&mut buf.as_slice())
            .expect("Failed to read MorphFeatureTable");
        assert_eq!(org, cpy);
    }
}
