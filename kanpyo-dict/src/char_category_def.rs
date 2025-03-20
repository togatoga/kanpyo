use crate::{builder::char_def::CharClassDef, dict::DictReadWrite};
use bincode::{Decode, Encode};

// CharClass represents  a character class.
type CharClass = Vec<String>;
// CharCategory represents categories for characters.
type CharCategory = Vec<u8>;
// InvokeList represents whether to invoke unknown word processing.
type InvokeList = Vec<bool>;
// GroupList represents whether to group unknown word processing.
type GroupList = Vec<bool>;

// CharTable represents character category table.
#[derive(Debug, PartialEq, Clone, Eq, Decode, Encode)]
pub struct CharCategoryDef {
    pub char_class: CharClass,
    pub char_category: CharCategory,
    pub invoke_list: InvokeList,
    pub group_list: GroupList,
}

impl CharCategoryDef {
    pub fn new(char_def: CharClassDef) -> Self {
        CharCategoryDef {
            char_class: char_def.char_class.to_vec(),
            char_category: char_def.char_category.to_vec(),
            invoke_list: char_def.invoke_map.to_vec(),
            group_list: char_def.group_map.to_vec(),
        }
    }

    // char_category returns a category of a char.
    pub fn char_category(&self, ch: char) -> u8 {
        *self
            .char_category
            .get(ch as usize)
            .unwrap_or(&self.char_category[0])
    }
}

impl DictReadWrite for CharCategoryDef {
    fn write_dict<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let mut buf = Vec::new();
        match bincode::encode_into_std_write(self, &mut buf, bincode::config::standard()) {
            Ok(_) => w.write_all(&buf),
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }

    fn from_dict<R: std::io::Read>(r: &mut R) -> std::io::Result<Self> {
        let mut buf = Vec::new();
        r.read_to_end(&mut buf)?;
        match bincode::decode_from_slice(&buf, bincode::config::standard()) {
            Ok(v) => Ok(v.0),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_read_dict() {
        let def = CharCategoryDef {
            char_class: vec![
                "class1".to_string(),
                "class2".to_string(),
                "class3".to_string(),
            ],
            char_category: vec![b'a', b'b', b'c'],
            invoke_list: vec![true, false, true],
            group_list: vec![false, true, false],
        };

        let mut buf = Vec::new();
        def.write_dict(&mut buf).unwrap();
        let def2 = CharCategoryDef::from_dict(&mut buf.as_slice()).unwrap();
        assert_eq!(def, def2);
    }
}
