use crate::{
    lattice::{self, node::Node},
    token::{Token, TokenClass},
};
use kanpyo_dict::dict::Dict;

pub struct Tokenizer {
    pub dict: Dict,
}

impl Tokenizer {
    pub fn new(dict: Dict) -> Self {
        Self { dict }
    }

    pub fn tokenize(&self, input: &str) -> Vec<Token> {
        let lattice = lattice::Lattice::build(&self.dict, input);

        lattice
            .viterbi()
            .into_iter()
            .map(|node| {
                let token_class = match &node {
                    Node::Dummy { .. } => TokenClass::Dummy,
                    Node::Known(_) => TokenClass::Known,
                    Node::Unknown(_) => TokenClass::Unknown,
                };
                let surface = match &node {
                    Node::Dummy { .. } => "EOS".to_string(),
                    Node::Known(n) | Node::Unknown(n) => n.surface.clone(),
                };

                let char_pos = node.char_pos();
                let end_pos = char_pos + surface.chars().count();
                Token {
                    id: node.id(),
                    class: token_class,
                    position: node.byte_pos(),
                    start: char_pos,
                    end: end_pos,
                    surface,
                }
            })
            .collect()
    }
}
