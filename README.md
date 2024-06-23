# Kanpyo

Kanpyo is Japanese morphological analyzer written in Rust inspired by [Kagome](https://github.com/ikawaha/kagome).

## Caution

This is a work in progress. I would break the API without notice.

## Installation

You can install `kanpyo` via `cargo`:

```shell script
cargo install kanpyo
```

You need a dictionary to use `kanpyo` and can build a dictionary by the following.

```shell script
cd kanpyo-dict
cargo run --release --bin ipa-dict-builder -- 
```

Create a dictionary file in user's config directory.

- Linux
  - $HOME/.config/kanpyo/
- macOS
  - $HOME/Library/Application Support/kanpyo

You're ready to use `kanpyo`!

## Usage

### Tokenize

TODO

### Graphviz

TODO

### Server

TODO


### TODO

- [ ] Support various dictionaries(Sudachi, UniDic, neologd, etc.)
- [ ] Support server mode
- [ ] Support search mode