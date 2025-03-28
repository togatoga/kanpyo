use clap::{Parser, ValueEnum};
use kanpyo_dict::builder::{config::Config, DictionaryBuilder};

use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
enum Encoding {
    /// EUC-JP
    EucJp,
    /// UTF-8
    Utf8,
}

fn get_default_output_path() -> String {
    let mut path = dirs::config_dir()
        .expect("failed to get config dir")
        .join("kanpyo");
    path.push("ipa.dict");
    path.into_os_string()
        .into_string()
        .expect("failed to convert path to string")
}

#[derive(Parser)]
#[command(name = "IPAdic builder", about = "Builds an ipa.dict", version = "0.1", long_about=None)]
struct IPADictBuilderCommand {
    /// Path of input dict, e.g. mecab-ipadic-2.7.0-20070801
    #[arg(short, long)]
    dict: PathBuf,
    /// Path of output dict, e.g. ipa.dict    
    #[arg(short, long, default_value_t = get_default_output_path())]
    out: String,
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
        let dict = DictionaryBuilder::from_config(&config).expect("failed to build dict");

        let path = PathBuf::from(&self.out);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("failed to create dir");
        }
        let mut output = std::fs::File::create(&self.out).expect("failed to create file");
        dict.build(&mut output).expect("failed to build dict");
        println!("Built ipa.dict to {}", self.out)
    }
}

fn main() {
    IPADictBuilderCommand::parse().run();
}
