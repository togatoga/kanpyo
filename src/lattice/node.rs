use kanpyo_dict::{morph::Morph, trie::da::KeywordID};

pub const BOS_EOS_ID: KeywordID = 0;

// Node is a lattice node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Word {
    pub id: KeywordID,
    pub byte_pos: usize,
    pub char_pos: usize,
    pub morph: Morph,
    pub surface: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Node {
    Dummy {
        byte_pos: usize,
        char_pos: usize,
        morph: Morph,
    },
    Known(Word),
    Unknown(Word),
}

impl Node {
    pub fn id(&self) -> KeywordID {
        match self {
            Node::Dummy { .. } => BOS_EOS_ID,
            Node::Known(word) | Node::Unknown(word) => word.id,
        }
    }
    pub fn byte_pos(&self) -> usize {
        match self {
            Node::Dummy { byte_pos, .. } => *byte_pos,
            Node::Known(word) | Node::Unknown(word) => word.byte_pos,
        }
    }

    pub fn char_pos(&self) -> usize {
        match self {
            Node::Dummy { char_pos, .. } => *char_pos,
            Node::Known(word) | Node::Unknown(word) => word.char_pos,
        }
    }

    pub fn morph(&self) -> &Morph {
        match self {
            Node::Dummy { morph, .. } => morph,
            Node::Known(word) | Node::Unknown(word) => &word.morph,
        }
    }
}
