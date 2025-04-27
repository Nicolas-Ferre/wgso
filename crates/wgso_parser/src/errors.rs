use std::ops::Range;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuleError {
    #[error("invalid rule file: {0}")]
    Deserialization(serde_yml::Error),
    #[error("invalid rule file: {0}")]
    Validation(serde_valid::validation::Errors),
}

#[derive(Error, Debug)]
#[error("parsing failed: {message} (span: {span:?}, file: {path})")]
pub struct ParsingError {
    pub path: PathBuf,
    pub span: Range<usize>,
    pub message: String,
}
