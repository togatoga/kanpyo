# Kanpyo

[![Crates.io](https://img.shields.io/crates/v/kanpyo.svg)](https://crates.io/crates/kanpyo)

Kanpyo is Japanese morphological analyzer written in Rust inspired by [Kagome](https://github.com/ikawaha/kagome).

## Caution

This is a work in progress. I would break the API without notice.

## Installation

You can install `kanpyo` via `cargo` or `git`(development version).

```shell script
cargo install kanpyo
```

or

```shell script
cargo install --git https://github.com/togatoga/kanpyo kanpyo
```

You need a dictionary to use `kanpyo` and can build and install a dictionary by the following.

```shell script
cd kanpyo-dict
tar xvf resource/mecab-ipadic-2.7.0-20070801.tar.gz -C resource
cargo run --release --bin ipa-dict-builder -- --dict resource/mecab-ipadic-2.7.0-20070801
```

A dictionary is installed in the following directory:

- Linux
  - $HOME/.config/kanpyo/
- macOS
  - $HOME/Library/Application Support/kanpyo

You're ready to use `kanpyo`!

## Usage

```shell script
kanpyo --help
Japanese Morphological Analyzer

Usage: kanpyo [COMMAND]

Commands:
  tokenize  Tokenize input text
  graphviz  Output lattice in Graphviz format
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Tokenize

```shell script
kanpyo tokenize "すもももももももものうち"          
すもも  名詞,一般,*,*,*,*,すもも,スモモ,スモモ
も      助詞,係助詞,*,*,*,*,も,モ,モ
もも    名詞,一般,*,*,*,*,もも,モモ,モモ
も      助詞,係助詞,*,*,*,*,も,モ,モ
もも    名詞,一般,*,*,*,*,もも,モモ,モモ
の      助詞,連体化,*,*,*,*,の,ノ,ノ
うち    名詞,非自立,副詞可能,*,*,*,うち,ウチ,ウチ
EOS
```

#### REPL mode

```shell script
kanpyo
自然言語処理
自然    名詞,形容動詞語幹,*,*,*,*,自然,シゼン,シゼン
言語    名詞,一般,*,*,*,*,言語,ゲンゴ,ゲンゴ
処理    名詞,サ変接続,*,*,*,*,処理,ショリ,ショリ
EOS
形態素解析
形態素  名詞,一般,*,*,*,*,形態素,ケイタイソ,ケイタイソ
解析    名詞,サ変接続,*,*,*,*,解析,カイセキ,カイセキ
EOS
```

#### From piped standard input

```shell script
echo "自然言語処理" | kanpyo
自然    名詞,形容動詞語幹,*,*,*,*,自然,シゼン,シゼン
言語    名詞,一般,*,*,*,*,言語,ゲンゴ,ゲンゴ
処理    名詞,サ変接続,*,*,*,*,処理,ショリ,ショリ
EOS
```

### Graphviz

Print lattice in Graphviz format for debugging.

```shell script
kanpyo graphviz "自然言語処理" | dot -Tpng -o lattice.png
```

![lattice](https://github.com/togatoga/kanpyo/assets/7335831/d68ea754-51f9-458e-ac5f-50955be3c581)

### TODO

- [ ] Support various dictionaries(Sudachi, UniDic, neologd, etc.)
- [ ] Support server mode
- [ ] Support search mode
- [ ] Tests for load dictionary and tokenize
