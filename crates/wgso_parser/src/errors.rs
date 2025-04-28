use std::ops::Range;
use std::path::PathBuf;
use thiserror::Error;

/// An error occurring during rule parsing.
#[derive(Error, Debug)]
pub enum RuleError {
    /// A deserialization error.
    #[error("invalid rule file: {0}")]
    Deserialization(serde_yml::Error),
    /// A validation error.
    #[error("invalid rule file: {0}")]
    Validation(serde_valid::validation::Errors),
}

/// An error occurring during code parsing.
#[derive(Error, Debug)]
#[error("parsing failed: {message} (span: {span:?}, file: {path})")]
pub struct ParsingError {
    /// The path of the file where the error occurs.
    pub path: PathBuf,
    /// The span where the error occurs.
    pub span: Range<usize>,
    /// The error message.
    pub message: String,
}
