use clap::{Parser, Subcommand, ValueEnum};
use kanpyo::{lattice::node::BOS_EOS_ID, tokenizer::Tokenizer};
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
        /// Dictionary
        #[arg(short, long, value_enum, default_value = "ipa")]
        dict: Dict,
        /// Custom dictionary
        #[arg(short, long)]
        custom_dict: Option<PathBuf>,
    },
    /// Output lattice in Graphviz format
    Graphviz {
        /// Input text to analyze
        #[arg(index = 1)]
        input: Option<String>,
        /// Dictionary
        #[arg(short, long, value_enum, default_value = "ipa")]
        dict: Dict,
        /// Custom dictionary
        #[arg(short, long)]
        custom_dict: Option<PathBuf>,
        /// Output full state of lattice
        #[arg(short, long, default_value = "false")]
        full_state: bool,
        /// DPI of output image
        #[arg(long, default_value = "48")]
        dpi: usize,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum Dict {
    Ipa,
    // Unidic,
}

fn get_dict_path(dict: Dict) -> PathBuf {
    let mut path = dirs::config_dir()
        .expect("failed to get config dir")
        .join("kanpyo");
    match dict {
        Dict::Ipa => {
            path.push("ipa.dict");
        } // Dict::Unidic => {
          //     path.push("unidic.dict");
          // }
    }
    path
}

impl KanpyoCommand {
    fn tokenizer(dict: Dict, custom_dict: Option<PathBuf>) -> Tokenizer {
        let dict_file = custom_dict.unwrap_or_else(|| get_dict_path(dict));
        let mut reader = std::io::BufReader::new(
            std::fs::File::open(dict_file).expect("failed to open custom dict"),
        );
        Tokenizer::new(dict::Dict::load(&mut reader).expect("failed to load dict"))
    }

    fn tokenize(input: Option<String>, dict: Dict, custom_dict: Option<PathBuf>) {
        let tokenizer = KanpyoCommand::tokenizer(dict, custom_dict);
        loop {
            match &input {
                Some(text) => {
                    print_tokens(tokenizer.tokenize(text), &tokenizer.dict);
                    break;
                }
                None => {
                    let mut buf = String::new();
                    std::io::stdin()
                        .read_line(&mut buf)
                        .expect("failed to read from stdin");
                    if buf.is_empty() {
                        break;
                    }
                    print_tokens(tokenizer.tokenize(buf.trim_end()), &tokenizer.dict);
                }
            };
        }
    }
    fn graphviz(
        input: Option<String>,
        dict: Dict,
        custom_dict: Option<PathBuf>,
        dpi: usize,
        full_state: bool,
    ) {
        let input = match input {
            Some(text) => text,
            None => {
                let mut buf = String::new();
                std::io::stdin()
                    .read_line(&mut buf)
                    .expect("failed to read from stdin");
                buf.trim_end().to_string()
            }
        };

        let tokenizer = KanpyoCommand::tokenizer(dict, custom_dict);
        let lattice = kanpyo::lattice::Lattice::build(&tokenizer.dict, &input);
        kanpyo::graphviz::Graphviz { lattice }.graphviz(dpi, full_state);
    }
    fn run(self) {
        match self.subcommand {
            Some(SubCommand::Tokenize {
                input,
                dict,
                custom_dict,
            }) => {
                KanpyoCommand::tokenize(input, dict, custom_dict);
            }
            Some(SubCommand::Graphviz {
                input,
                dict,
                custom_dict,
                dpi,
                full_state,
            }) => {
                KanpyoCommand::graphviz(input, dict, custom_dict, dpi, full_state);
            }
            None => {
                KanpyoCommand::tokenize(None, Dict::Ipa, None);
            }
        }
    }
}

fn print_tokens(tokens: Vec<kanpyo::token::Token>, dict: &dict::Dict) {
    for token in tokens {
        let morph_features = if token.id != BOS_EOS_ID {
            match token.class {
                kanpyo::token::TokenClass::Known => dict.morph_feature_table.morph_features
                    [token.id as usize - 1]
                    .iter()
                    .map(|&idx| dict.morph_feature_table.name_list[idx as usize].clone())
                    .collect::<Vec<_>>(),
                kanpyo::token::TokenClass::Unknown => dict
                    .unk_dict
                    .morph_feature_table
                    .morph_features[token.id as usize - 1]
                    .iter()
                    .map(|&idx| dict.unk_dict.morph_feature_table.name_list[idx as usize].clone())
                    .collect::<Vec<_>>(),
                kanpyo::token::TokenClass::Dummy => Vec::new(),
            }
        } else {
            Vec::new()
        };
        println!("{}\t{}", token.surface, morph_features.join(","))
    }
}

fn main() {
    KanpyoCommand::parse().run();
}
