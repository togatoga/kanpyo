use clap::{Parser, Subcommand};
use kanpyo::{lattice::node::BOS_EOS_ID, tokenizer::Tokenzier};
use kanpyo_dict::dict;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "kanpyo", about = "Japanese Morphological Analyzer", version = "0.1", long_about=None)]
struct KanpyoCommand {
    /// Subcommand
    #[command(subcommand)]
    subcommand: Option<SubCommand>,
}

#[derive(Debug, Subcommand)]
enum SubCommand {
    /// Tokenize input text
    Tokenize {
        /// Input text to analyze [default: stdin]
        #[arg(index = 1)]
        input: Option<String>,
        /// Dictionary file
        #[arg(short, long, default_value = "kanpyo-dict/ipa.dict")]
        dict: PathBuf,
    },
    /// Output lattice in Graphviz format
    Graphviz {
        /// Input text to analyze
        #[arg(index = 1)]
        input: String,
        /// Dictionary file
        #[arg(short, long, default_value = "kanpyo-dict/ipa.dict")]
        dict: PathBuf,
        #[arg(short, long, default_value = "48")]
        dpi: usize,
        /// Output unknown morphemes
        #[arg(short, long)]
        unk: bool,
    },
}

impl KanpyoCommand {
    fn tokenize(input: Option<String>, dict: PathBuf) {
        let mut reader =
            std::io::BufReader::new(std::fs::File::open(dict).expect("failed to open dict"));
        let tokenzier = Tokenzier::new(dict::Dict::load(&mut reader).expect("failed to load dict"));
        loop {
            match &input {
                Some(text) => {
                    print_tokens(tokenzier.tokenize(text), &tokenzier.dict);
                    break;
                }
                None => {
                    let mut buf = String::new();
                    std::io::stdin()
                        .read_line(&mut buf)
                        .expect("failed to read from stdin");
                    print_tokens(tokenzier.tokenize(buf.trim_end()), &tokenzier.dict);
                }
            };
        }
    }
    fn graphviz(input: String, dict: PathBuf, dpi: usize, unk: bool) {
        let mut reader =
            std::io::BufReader::new(std::fs::File::open(dict).expect("failed to open dict"));
        let tokenzier = Tokenzier::new(dict::Dict::load(&mut reader).expect("failed to load dict"));
        let lattice = kanpyo::lattice::Lattice::build(&tokenzier.dict, &input);
        kanpyo::graphviz::Graphviz { lattice }.graphviz(dpi, unk);
    }
    fn run(self) {
        match self.subcommand {
            Some(SubCommand::Tokenize { input, dict }) => {
                KanpyoCommand::tokenize(input, dict);
            }
            Some(SubCommand::Graphviz {
                input,
                dict,
                dpi,
                unk,
            }) => {
                KanpyoCommand::graphviz(input, dict, dpi, unk);
            }
            None => {
                KanpyoCommand::tokenize(None, PathBuf::from("dict/ipa.dict"));
            }
        }
    }
}

fn print_tokens(tokens: Vec<kanpyo::token::Token>, dict: &dict::Dict) {
    for token in tokens {
        let morph_features = if token.id != BOS_EOS_ID {
            let mut features = vec![];
            match token.class {
                kanpyo::token::TokenClass::Known => {
                    for idx in dict.morph_feature_table.morph_features[token.id as usize - 1].iter()
                    {
                        features.push(dict.morph_feature_table.name_list[*idx as usize].clone());
                    }
                }
                kanpyo::token::TokenClass::Unknown => {
                    for idx in dict.unk_dict.morph_feature_table.morph_features
                        [token.id as usize - 1]
                        .iter()
                    {
                        features.push(
                            dict.unk_dict.morph_feature_table.name_list[*idx as usize].clone(),
                        );
                    }
                }
                kanpyo::token::TokenClass::Dummy => {}
            }
            features
        } else {
            vec![]
        };
        println!("{}\t{}", token.surface, morph_features.join(","))
    }
}

fn main() {
    KanpyoCommand::parse().run();
}
