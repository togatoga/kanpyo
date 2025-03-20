use kanpyo_dict::trie::da::KeywordID;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenClass {
    Dummy,
    Known,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub id: KeywordID,
    pub class: TokenClass,
    pub position: usize, // byte position
    pub start: usize,    // char position
    pub end: usize,      // char position
    pub surface: String,
}

impl Token {
    pub fn new(
        id: KeywordID,
        class: TokenClass,
        position: usize,
        start: usize,
        end: usize,
        surface: impl Into<String>,
    ) -> Self {
        Self {
            id,
            class,
            position,
            start,
            end,
            surface: surface.into(),
        }
    }

    pub fn length(&self) -> usize {
        self.end - self.start
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.class == other.class
            && self.position == other.position
            && self.start == other.start
            && self.end == other.end
            && self.surface == other.surface
    }
}

impl Eq for Token {}
