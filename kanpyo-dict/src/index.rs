use std::collections::BTreeMap;

use crate::{
    dict::DictReadWrite,
    error::Result,
    trie::{self, da::KeywordID},
};

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct IndexTable {
    da: trie::da::DoubleArray,
    dup: BTreeMap<trie::da::KeywordID, usize>,
}

impl IndexTable {
    pub fn build(sorted_keywords: &[String]) -> Result<Self> {
        // check sorted
        let mut keys = vec![];
        let mut ids = vec![];
        let mut prev = None;
        let mut dup = BTreeMap::default();
        for (i, key) in sorted_keywords.iter().enumerate() {
            if let Some((prev_key, prev_no)) = prev
                && prev_key == key
            {
                *dup.entry(prev_no as KeywordID).or_insert(0) += 1;
                continue;
            }
            prev = Some((key, i + 1));
            keys.push(key.clone());
            ids.push((i + 1) as KeywordID);
        }

        Ok(Self {
            da: trie::da::build_with_ids(&keys, &ids)?,
            dup,
        })
    }

    pub fn search_common_prefix_of(
        &self,
        input: &str,
    ) -> Option<Vec<(trie::da::KeywordID, usize)>> {
        let ids_and_lengths = self.da.search_common_prefix_of(input)?;
        let mut results = vec![];
        for (id, len) in ids_and_lengths.iter() {
            let dup = *self.dup.get(id).unwrap_or(&0);
            for i in 0..=dup {
                results.push((*id + i as isize, *len));
            }
        }
        Some(results)
    }
}

impl DictReadWrite for IndexTable {
    fn from_dict<R: std::io::Read>(r: &mut R) -> std::io::Result<Self> {
        let da = trie::da::DoubleArray::from_dict(r)?;

        let mut buf = [0u8; 8];
        r.read_exact(&mut buf)?;
        let len = u64::from_le_bytes(buf);
        let mut dup = BTreeMap::default();
        for _ in 0..len {
            r.read_exact(&mut buf)?;
            let k = KeywordID::from_le_bytes(buf);
            r.read_exact(&mut buf)?;
            let v: usize = usize::from_le_bytes(buf);
            dup.insert(k, v);
        }

        Ok(Self { da, dup })
    }

    fn write_dict<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        self.da.write_dict(w)?;
        let len = self.dup.len();
        w.write_all(&(len as u64).to_le_bytes())?;
        for (&k, &v) in &self.dup {
            w.write_all(&k.to_le_bytes())?;
            w.write_all(&v.to_le_bytes())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_table_build_empty() {
        let result = IndexTable::build(&[]);
        assert!(result.is_ok(), "Should handle empty keyword list");
    }

    #[test]
    fn test_index_table_build_with_duplicates() {
        let keywords = vec![
            "apple".to_string(),
            "apple".to_string(),
            "banana".to_string(),
            "banana".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ];

        let index = IndexTable::build(&keywords).expect("Failed to build index");

        // Search for duplicates
        let results = index.search_common_prefix_of("apple");
        assert!(results.is_some(), "Should find 'apple'");
        let results = results.unwrap();
        assert_eq!(results.len(), 2, "Should find 2 occurrences of 'apple'");

        let results = index.search_common_prefix_of("banana");
        assert!(results.is_some(), "Should find 'banana'");
        let results = results.unwrap();
        assert_eq!(results.len(), 3, "Should find 3 occurrences of 'banana'");
    }

    #[test]
    fn test_index_table_search_not_found() {
        let keywords = vec!["apple".to_string(), "banana".to_string()];
        let index = IndexTable::build(&keywords).expect("Failed to build index");

        let results = index.search_common_prefix_of("cherry");
        assert!(results.is_none(), "Should not find 'cherry'");
    }

    #[test]
    fn test_index_table_search_common_prefix() {
        let keywords = vec![
            "東京".to_string(),
            "東京大学".to_string(),
            "東京大学大学院".to_string(),
        ];

        let index = IndexTable::build(&keywords).expect("Failed to build index");

        let results = index.search_common_prefix_of("東京大学大学院情報学");
        assert!(results.is_some(), "Should find common prefixes");

        let results = results.unwrap();
        assert_eq!(
            results.len(),
            3,
            "Should find all 3 prefixes (東京, 東京大学, 東京大学大学院)"
        );
    }

    #[test]
    fn test_index_table_write_read() {
        let keywords = vec![
            "test1".to_string(),
            "test2".to_string(),
            "test2".to_string(),
            "test3".to_string(),
        ];

        let original = IndexTable::build(&keywords).expect("Failed to build index");

        let mut buffer = Vec::new();
        original
            .write_dict(&mut buffer)
            .expect("Failed to write index");

        let mut cursor = std::io::Cursor::new(buffer);
        let restored = IndexTable::from_dict(&mut cursor).expect("Failed to read index");

        // Verify that the restored index works the same
        let results1 = original.search_common_prefix_of("test2");
        let results2 = restored.search_common_prefix_of("test2");

        assert_eq!(results1, results2, "Restored index should work the same");
    }
}
