use dict::trie::da::KeywordID;

pub enum TokenClass {
    Dummy,
    Known,
    Unknown,
}

pub struct Token {
    pub id: KeywordID,
    pub class: TokenClass,
    pub position: usize, // byte position
    pub start: usize,    // char position
    pub end: usize,      // char position
    pub surface: String,
}
