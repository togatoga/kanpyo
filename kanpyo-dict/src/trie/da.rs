use std::collections::BTreeMap;

use crate::dict::DictReadWrite;

const INIT_BUFFER_SIZE: usize = 50 * 1024;
const EXPAND_RATIO: usize = 2;
const TERMINATOR: u8 = 0;
const ROOT_ID: usize = 1;

pub type KeywordID = isize;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node {
    base: i32,
    check: i32,
}

#[derive(Debug, PartialEq, Clone, Eq, PartialOrd, Ord)]
pub struct DoubleArray(Vec<Node>);

impl DoubleArray {
    pub fn new(len: usize) -> Self {
        let mut nodes = vec![Node::default(); len];
        nodes[0].base = ROOT_ID as i32 + 1;
        Self(nodes)
    }

    fn truncate(&mut self) {
        let mut len = self.0.len();
        while len > 1 && self.0[len - 1].check == 0 {
            len -= 1;
        }
        self.0.truncate(len);
    }

    fn expand(&mut self) {
        let old_len = self.0.len();
        let new_len = old_len * EXPAND_RATIO;
        self.0.resize(new_len, Node::default());
    }

    fn seek(&mut self, byte_chars: &[u8]) -> usize {
        let left = self.0[0].base as usize;
        for i in left.. {
            let mut found = true;
            while i >= self.0.len() {
                self.expand();
            }
            for &ch in byte_chars {
                let q = i as i32 + ch as i32;
                while q >= self.0.len() as i32 {
                    self.expand();
                }
                if self.0[q as usize].check != 0 {
                    found = false;
                    break;
                }
            }
            if found {
                let mut used = 0;

                for x in left..=i {
                    if self.0[x].check != 0 {
                        used += 1;
                    }
                }
                // 95% or more
                let occupancy = used as f64 / (i - left + 1) as f64;
                if occupancy >= 0.95 {
                    self.0[0].base = i as i32 + 1;
                }
                return i;
            }
        }
        unreachable!();
    }

    pub fn add(
        &mut self,
        p: usize,
        i: usize,
        branches: &[KeywordID],
        sorted_keywords: &[String],
        ids: &[KeywordID],
    ) {
        while p >= self.0.len() {
            self.expand();
        }
        let mut char_bytes = vec![];
        let mut byte_to_child_branches = BTreeMap::default();
        for &key_id in branches {
            let str = sorted_keywords[key_id as usize].as_bytes();
            let ch = *str.get(i).unwrap_or(&TERMINATOR);
            if char_bytes.last() != Some(&ch) {
                char_bytes.push(ch);
            }

            if ch != TERMINATOR {
                byte_to_child_branches
                    .entry(ch)
                    .or_insert(vec![])
                    .push(key_id);
            }
        }
        let left = self.seek(&char_bytes);
        self.0[p].base = left as i32;

        for &ch in char_bytes.iter() {
            let q = left as i32 + ch as i32;
            assert!(
                self.0[q as usize].check == 0,
                "q: {}, check: {}",
                q,
                self.0[q as usize].check
            );
            self.0[q as usize].check = p as i32;
            if ch == TERMINATOR {
                // if the node is a leaf node
                let idx = -(ids[branches[0] as usize] as i32);
                assert!(idx < 0, "idx: {}", idx);
                self.0[q as usize].base = idx;
            }
        }
        for ch in char_bytes {
            if let Some(child_branchs) = byte_to_child_branches.get(&ch) {
                let q = self.0[p].base + ch as i32;
                self.add(q as usize, i + 1, child_branchs, sorted_keywords, ids);
            }
        }
    }

    pub fn search(&self, keyword: &str) -> Option<KeywordID> {
        let mut p = ROOT_ID as i32;
        for ch in keyword.bytes() {
            let shift = self.0.get(p as usize)?.base; // check if p is out of bounds
            let q = shift + ch as i32;
            let check = self.0.get(q as usize)?.check;
            if check != p {
                return None;
            }
            p = q;
        }

        let shift = self.0.get(p as usize)?.base + TERMINATOR as i32;
        let q = shift + TERMINATOR as i32;
        let check = self.0.get(q as usize)?.check;
        if check == p {
            Some(-self.0[q as usize].base as KeywordID)
        } else {
            None
        }
    }
    // searches finds keywords sharing common prefix in a keyword and returns the ids and it's lenghts if found.
    pub fn search_common_prefix_of(&self, keyword: &str) -> Option<Vec<(KeywordID, usize)>> {
        let mut p = ROOT_ID as i32;
        let mut id_and_byte_lengths = vec![];

        for (i, ch) in keyword.bytes().enumerate() {
            let prev = p;
            p = self.0[prev as usize].base + ch as i32;
            let check_value = self.0.get(p as usize).map(|node| node.check);
            if check_value != Some(prev) {
                break;
            }
            let ahead = self.0[p as usize].base + TERMINATOR as i32;
            let node = self.0.get(ahead as usize);
            if let Some(node) = node {
                if node.check == p && node.base < 0 {
                    // found
                    let id = -self.0[ahead as usize].base as KeywordID;
                    id_and_byte_lengths.push((id, i + 1));
                }
            }
        }
        if id_and_byte_lengths.is_empty() {
            None
        } else {
            Some(id_and_byte_lengths)
        }
    }
}

impl Default for DoubleArray {
    fn default() -> Self {
        Self::new(INIT_BUFFER_SIZE)
    }
}

pub fn build(sorted_unique_keywords: &[String]) -> anyhow::Result<DoubleArray> {
    let mut da = DoubleArray::default();
    da.add(
        ROOT_ID,
        0,
        &(0..sorted_unique_keywords.len() as KeywordID).collect::<Vec<_>>(),
        sorted_unique_keywords,
        &(1..=sorted_unique_keywords.len())
            .map(|x| x as KeywordID)
            .collect::<Vec<_>>(),
    );
    da.truncate();
    Ok(da)
}

pub fn build_with_ids(
    sorted_unique_keywords: &[String],
    ids: &[KeywordID],
) -> anyhow::Result<DoubleArray> {
    let mut da = DoubleArray::default();
    da.add(
        ROOT_ID,
        0,
        &(0..sorted_unique_keywords.len() as KeywordID).collect::<Vec<_>>(),
        sorted_unique_keywords,
        ids,
    );
    da.truncate();
    Ok(da)
}

impl DictReadWrite for DoubleArray {
    fn from_dict<R: std::io::Read>(r: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = [0; 8];
        r.read_exact(&mut buf)?;
        let size = usize::from_le_bytes(buf);
        let mut nodes = vec![Node::default(); size];
        for node in nodes.iter_mut() {
            let mut buf = [0; 4];
            r.read_exact(&mut buf)?;
            node.base = i32::from_le_bytes(buf);
            r.read_exact(&mut buf)?;
            node.check = i32::from_le_bytes(buf);
        }
        Ok(Self(nodes))
    }
    fn write_dict<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let size = self.0.len();
        w.write_all(&size.to_le_bytes())?;
        for node in self.0.iter() {
            w.write_all(&node.base.to_le_bytes())?;
            w.write_all(&node.check.to_le_bytes())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_and_search() {
        let keywords = vec![
            "a",
            "ab",
            "abc",
            "abcd",
            "abcde",
            "abcdef",
            "abcdefg",
            "abcdefgh",
            "abcdefghi",
            "abcdefghij",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
        let da = build(&keywords).expect("failed to build");

        // found
        for (i, keyword) in keywords.iter().enumerate() {
            assert_eq!(
                da.search(keyword),
                Some((i + 1) as KeywordID),
                "keyword: {}",
                keyword
            );
        }

        // not found
        let not_found = vec!["", "b", "abcdeh", "abcdefghijj"];
        for keyword in not_found {
            assert_eq!(da.search(keyword), None, "keyword: {}", keyword);
        }
    }

    #[test]
    fn test_search_common_prefix() {
        let sorted_keywords = vec![
            "早稲田",
            "早稲田大学",
            "東京",
            "東京大学",
            "東京大学大学院",
            "東京大学大学院情報理工学研究科",
            "東京大学大学院情報理工学研究科創造情報学専攻",
            "東京工業大学",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
        let da = build(&sorted_keywords).expect("failed to build");
        let ids_and_lengths =
            da.search_common_prefix_of("東京大学大学院情報理工学研究科創造情報学専攻");
        let expecteds = vec![
            (3, 6),  // 東京
            (4, 12), // 東京大学
            (5, 21), // 東京大学大学院
            (6, 45), // 東京大学大学院情報理工学研究科
            (7, 66), // 東京大学大学院情報理工学研究科創造情報学専攻
        ];
        assert_eq!(ids_and_lengths, Some(expecteds));
        let ids_and_lengths = da.search_common_prefix_of("早稲田大学");
        let expecteds = vec![
            (1, 9),  // 早稲田
            (2, 15), // 早稲田大学
        ];
        assert_eq!(ids_and_lengths, Some(expecteds));

        let ids_and_lengths = da.search_common_prefix_of("大学");
        assert_eq!(ids_and_lengths, None);
    }

    #[test]
    fn test_build_and_search_multibyte() {
        let mut keywords = vec!["12345", "2345", "１２３", "abc", "ABCD", "あいう", "Ａ"];
        keywords.sort();

        let keywords = keywords
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let da = build(&keywords).expect("failed to build");

        // found
        for (i, keyword) in keywords.iter().enumerate() {
            assert_eq!(
                da.search(keyword),
                Some((i + 1) as KeywordID),
                "keyword: {}",
                keyword
            );
        }

        // not found
        let not_found = vec!["", "b", "ab", "abcdeh", "abcdefghijj", "あい", "あいうえお"];
        for keyword in not_found {
            assert_eq!(da.search(keyword), None, "keyword: {}", keyword);
        }
    }
}
