use std::ops::Index;

use crate::{dict::DictReadWrite, trie::da::KeywordID};

/// Morph represents part of speeches and an occurrence cost.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Morph {
    pub left_id: i16,
    pub right_id: i16,
    pub cost: i16,
}

impl Morph {
    pub fn new(left_id: i16, right_id: i16, cost: i16) -> Self {
        Morph {
            left_id,
            right_id,
            cost,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Morphs(Vec<Morph>);

impl Default for Morphs {
    fn default() -> Self {
        Self::new()
    }
}

impl Morphs {
    pub fn new() -> Self {
        Morphs(Vec::new())
    }

    pub fn push(&mut self, left_id: i16, right_id: i16, cost: i16) {
        self.0.push(Morph {
            left_id,
            right_id,
            cost,
        });
    }
}

impl Index<KeywordID> for Morphs {
    type Output = Morph;

    fn index(&self, index: KeywordID) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl From<Vec<Morph>> for Morphs {
    fn from(v: Vec<Morph>) -> Self {
        Morphs(v)
    }
}

impl DictReadWrite for Morphs {
    fn write_dict<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let l = self.0.len() as i64;
        w.write_all(&l.to_le_bytes())?;
        for m in &self.0 {
            w.write_all(&m.left_id.to_le_bytes())?;
            w.write_all(&m.right_id.to_le_bytes())?;
            w.write_all(&m.cost.to_le_bytes())?;
        }
        Ok(())
    }
    fn from_dict<R: std::io::Read>(r: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut l = [0; 8];
        r.read_exact(&mut l)?;
        let l = i64::from_le_bytes(l);
        let mut m = Vec::with_capacity(l as usize);
        for _ in 0..l {
            let mut buf = [0; 6];
            r.read_exact(&mut buf)?;
            let left_id = i16::from_le_bytes([buf[0], buf[1]]);
            let right_id = i16::from_le_bytes([buf[2], buf[3]]);
            let cost = i16::from_le_bytes([buf[4], buf[5]]);
            m.push(Morph {
                left_id,
                right_id,
                cost,
            });
        }
        Ok(Morphs(m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_to() {
        let morphs = Morphs(vec![
            Morph {
                left_id: 1,
                right_id: 1,
                cost: 1,
            },
            Morph {
                left_id: 2,
                right_id: 2,
                cost: 2,
            },
            Morph {
                left_id: 3,
                right_id: 3,
                cost: 3,
            },
        ]);

        let mut buf = Vec::new();
        morphs.write_dict(&mut buf).expect("Failed to write morphs");
    }

    #[test]
    fn test_read_from() {
        let morphs = Morphs(vec![
            Morph {
                left_id: 1,
                right_id: 1,
                cost: 1,
            },
            Morph {
                left_id: 2,
                right_id: 2,
                cost: 2,
            },
            Morph {
                left_id: 3,
                right_id: 3,
                cost: 3,
            },
        ]);

        let mut buf = Vec::new();
        morphs.write_dict(&mut buf).expect("Failed to write morphs");

        let mut buf = std::io::Cursor::new(buf);
        let result = Morphs::from_dict(&mut buf).expect("Failed to read morphs");
        assert_eq!(morphs, result);
    }
}
