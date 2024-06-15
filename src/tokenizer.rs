use dict::dict::Dict;

use crate::{
    lattice::{self, node::NodeClass},
    token::{Token, TokenClass},
};

pub struct Tokenzier {
    pub dict: Dict,
}

impl Tokenzier {
    pub fn new(dict: Dict) -> Self {
        Tokenzier { dict }
    }
}

impl Tokenzier {
    pub fn tokenize(&self, input: &str) -> Vec<Token> {
        let la = lattice::Lattice::build(&self.dict, input);
        let nodes = la.viterbi();
        let mut tokens = vec![];
        for node in nodes.into_iter() {
            let token_class = match node.class {
                NodeClass::Dummy => TokenClass::Dummy,
                NodeClass::Known => TokenClass::Known,
                NodeClass::Unknown => TokenClass::Unknown,
            };
            let surface = node.surface.unwrap_or("EOS".to_string());
            let token = Token {
                id: node.id,
                class: token_class,
                position: node.byte_pos,
                start: node.char_pos,
                end: node.char_pos + surface.chars().count(),
                surface,
            };
            tokens.push(token);
        }
        tokens
    }
}
