use crate::Error;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

pub(crate) fn load(file_path: impl AsRef<Path>) -> Result<Config, Error> {
    let file_path = file_path.as_ref();
    let file = File::open(file_path).map_err(|e| Error::Io(file_path.into(), e))?;
    serde_yml::from_reader(file).map_err(Error::Deserialization)
}

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) dependencies: HashMap<String, DependencyConfig>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DependencyConfig {
    pub(crate) path: Option<PathBuf>,
    pub(crate) url: Option<String>,
}
