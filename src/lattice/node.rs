use dict::{morph::Morph, trie::da::KeywordID};

pub const BOS_EOS_ID: KeywordID = 0;

// NodeClass codes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeClass {
    Dummy,
    Known,
    Unknown,
}

// Node is a lattice node.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: KeywordID,
    pub byte_pos: usize,
    pub char_pos: usize,
    pub class: NodeClass,
    pub morph: Option<Morph>,
    pub surface: Option<String>,
}
