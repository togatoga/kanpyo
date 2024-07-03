use std::collections::BTreeMap;

use crate::{
    dict::DictReadWrite,
    trie::{self, da::KeywordID},
};

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct IndexTable {
    da: trie::da::DoubleArray,
    dup: BTreeMap<trie::da::KeywordID, usize>,
}

impl IndexTable {
    pub fn build(sorted_keywords: &[String]) -> anyhow::Result<Self> {
        // check sorted
        let mut keys = vec![];
        let mut ids = vec![];
        let mut prev = None;
        let mut dup = BTreeMap::default();
        for (i, key) in sorted_keywords.iter().enumerate() {
            if let Some((prev_key, prev_no)) = prev {
                if prev_key == key {
                    *dup.entry(prev_no as KeywordID).or_insert(0) += 1;
                    continue;
                }
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
