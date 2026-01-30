use std::path::PathBuf;
use std::process::Command;

const MECAB_IPADIC_URL: &str =
    "https://github.com/togatoga/kanpyo/releases/download/dict-v0.1.0/mecab-ipadic.dict";

fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_MECAB_IPADIC");

    if std::env::var("CARGO_FEATURE_MECAB_IPADIC").is_ok() {
        let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));
        let dict_path = out_dir.join("mecab-ipadic.dict");

        // Download dictionary if not cached
        if !dict_path.exists() {
            download_dict(MECAB_IPADIC_URL, &dict_path);
        }

        println!(
            "cargo:rustc-env=KANPYO_MECAB_IPADIC_PATH={}",
            dict_path.display()
        );
        println!("cargo:rerun-if-changed={}", dict_path.display());
    }
}

fn download_dict(url: &str, dest: &PathBuf) {
    eprintln!("Downloading MeCab IPA dictionary from {}...", url);

    let status = Command::new("curl")
        .args(["-L", "-f", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .expect("Failed to execute curl. Please ensure curl is installed.");

    if !status.success() {
        panic!(
            "Failed to download dictionary from {}. curl exit code: {:?}",
            url,
            status.code()
        );
    }

    eprintln!("Downloaded dictionary to {:?}", dest);
}
