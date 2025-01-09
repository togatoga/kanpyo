use kanpyo_dict::dict::Dict;

use crate::{
    lattice::{self, node::Node},
    token::{Token, TokenClass},
};

pub struct Tokenizer {
    pub dict: Dict,
}

impl Tokenizer {
    pub fn new(dict: Dict) -> Self {
        Tokenizer { dict }
    }
}

impl Tokenizer {
    pub fn tokenize(&self, input: &str) -> Vec<Token> {
        let la = lattice::Lattice::build(&self.dict, input);
        let nodes = la.viterbi();
        let mut tokens = vec![];
        for node in nodes.into_iter() {
            let token_class = match node {
                Node::Dummy { .. } => TokenClass::Dummy,
                Node::Known(_) => TokenClass::Known,
                Node::Unknown(_) => TokenClass::Unknown,
            };

            let surface = match &node {
                Node::Dummy { .. } => "EOS".to_string(),
                Node::Known(node) | Node::Unknown(node) => node.surface.clone(),
            };

            let token = Token {
                id: node.id(),
                class: token_class,
                position: node.byte_pos(),
                start: node.char_pos(),
                end: node.char_pos() + surface.chars().count(),
                surface,
            };
            tokens.push(token);
        }
        tokens
    }
}
