use std::io;
use std::num::{ParseIntError, TryFromIntError};

/// Error types for kanpyo
#[derive(Debug, thiserror::Error)]
pub enum KanpyoError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Zip file read/write error
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// Dictionary file parse error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Invalid data format
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    /// Encoding error
    #[error("Encoding error: failed to decode text")]
    EncodingError,

    /// Integer parsing error
    #[error("Failed to parse integer: {0}")]
    ParseInt(#[from] ParseIntError),

    /// Integer conversion error
    #[error("Failed to convert integer: {0}")]
    TryFromInt(#[from] TryFromIntError),

    /// CSV parsing error
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    /// Cost value out of range
    #[error("Cost value {0} is out of range (must fit in i16)")]
    CostOutOfRange(i64),

    /// Character category not found
    #[error("Character category not found: {0}")]
    CharCategoryNotFound(String),

    /// Dictionary build error
    #[error("Failed to build dictionary: {0}")]
    DictBuild(String),

    /// Trie build error
    #[error("Failed to build trie: {0}")]
    TrieBuild(String),
}

pub type Result<T> = std::result::Result<T, KanpyoError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = KanpyoError::CostOutOfRange(100000);
        assert_eq!(
            err.to_string(),
            "Cost value 100000 is out of range (must fit in i16)"
        );

        let err = KanpyoError::CharCategoryNotFound("UNKNOWN".to_string());
        assert_eq!(err.to_string(), "Character category not found: UNKNOWN");

        let err = KanpyoError::EncodingError;
        assert_eq!(err.to_string(), "Encoding error: failed to decode text");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let kanpyo_err = KanpyoError::from(io_err);
        assert!(matches!(kanpyo_err, KanpyoError::Io(_)));
    }

    #[test]
    fn test_error_from_parse_int() {
        let parse_err = "abc".parse::<i32>().unwrap_err();
        let kanpyo_err = KanpyoError::from(parse_err);
        assert!(matches!(kanpyo_err, KanpyoError::ParseInt(_)));
    }
}
