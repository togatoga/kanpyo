use clap::Parser;
use dict::dict;
use kanpyo::{lattice::node::BOS_EOS_ID, tokenizer::Tokenzier};
use std::{io::Read, path::PathBuf};

#[derive(Parser)]
#[command(name = "kanpyo", about = "Japanese Morphological Analyzer", version = "0.1", long_about=None)]
struct KanpyoCommand {
    /// Input text to analyze [default: stdin]
    #[arg(index = 1)]
    input: Option<String>,
    /// Dictionary file
    #[arg(short, long, default_value = "ipa.dict")]
    dict: PathBuf,
}

impl KanpyoCommand {
    fn run(&self) {
        let mut reader =
            std::io::BufReader::new(std::fs::File::open(&self.dict).expect("failed to open dict"));
        let tokenzier = Tokenzier::new(dict::Dict::load(&mut reader).expect("failed to load dict"));
        let tokens = match &self.input {
            Some(text) => tokenzier.tokenize(text),
            None => {
                let mut buf = String::new();
                std::io::stdin()
                    .read_to_string(&mut buf)
                    .expect("failed to read from stdin");
                // trim last newline
                tokenzier.tokenize(buf.trim_end())
            }
        };
        print_tokens(tokens, &tokenzier.dict);
    }
}

fn print_tokens(tokens: Vec<kanpyo::token::Token>, dict: &dict::Dict) {
    for token in tokens {
        let mut morph_features = vec![];
        if token.id != BOS_EOS_ID {
            for idx in dict.pos_table.morph_features[token.id as usize - 1].iter() {
                morph_features.push(dict.pos_table.name_list[*idx as usize].clone());
            }
        }
        println!("{}\t{}", token.surface, morph_features.join(","))
    }
}

fn main() {
    KanpyoCommand::parse().run();
}
