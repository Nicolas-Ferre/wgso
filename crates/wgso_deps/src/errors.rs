use std::io;
use std::path::PathBuf;
use thiserror::Error;
use zip::result::ZipError;

/// An error occurring during dependency management.
#[derive(Error, Debug)]
pub enum Error {
    /// An I/O error.
    #[error("I/O error at path {0}: {1}")]
    Io(PathBuf, io::Error),
    /// A copy error.
    #[error("Copy error from path {0} to path {1}: {2}")]
    Copy(PathBuf, PathBuf, fs_extra::error::Error),
    /// An HTTP request error.
    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    #[error("HTTP request error: {0}")]
    Request(reqwest::Error),
    /// An error while extracting a ZIP archive.
    #[error("ZIP extraction error: {0}")]
    Zip(ZipError),
    /// A deserialization error.
    #[error("invalid format: {0}")]
    Deserialization(serde_yml::Error),
    /// A deserialization error.
    #[error("dependency '{0}' has no configured source")]
    NoDependencySource(String),
}
