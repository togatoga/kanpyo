use clap::{Parser, ValueEnum};
use kanpyo_dict::builder::{self, config::Config};

use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
enum Encoding {
    /// EUC-JP
    EucJp,
    /// UTF-8
    Utf8,
}

#[derive(Parser)]
#[command(name = "IPAdic builder", about = "Builds an ipa.dict", version = "0.1", long_about=None)]
struct IPADictBuilderCommand {
    /// Path of input dict, e.g. mecab-ipadic-2.7.0-20070801
    #[arg(short, long, default_value = "resource/mecab-ipadic-2.7.0-20070801")]
    dict: PathBuf,
    /// Path of output dict, e.g. ipa.dict    
    #[arg(short, long, default_value = "ipa.dict")]
    out: PathBuf,
    // Encoding of input dict
    #[arg(short, long, default_value = "euc-jp")]
    encoding: Encoding,
}

impl IPADictBuilderCommand {
    fn run(&self) {
        let encoding = match self.encoding {
            Encoding::EucJp => encoding_rs::EUC_JP,
            Encoding::Utf8 => encoding_rs::UTF_8,
        };
        let config = Config::new(&self.dict, encoding);
        let dict = builder::build(&config);

        let mut output = std::fs::File::create(&self.out).expect("failed to create file");
        dict.build(&mut output).expect("failed to build dict");
    }
}

fn main() {
    IPADictBuilderCommand::parse().run();
}
