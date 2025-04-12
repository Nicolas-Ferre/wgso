use crate::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct File {
    pub(crate) path: PathBuf,
    pub(crate) code: String,
}

impl File {
    pub(crate) fn read_dir(path: &Path) -> Vec<Result<Self, Error>> {
        let error_fn = |error| vec![Err(Error::Io(path.into(), error))];
        fs::read_dir(path).map_or_else(error_fn, |dir| {
            dir.flat_map(|entry| {
                entry.map_or_else(error_fn, |entry| {
                    let file_path = entry.path();
                    if file_path.is_dir() {
                        Self::read_dir(&file_path)
                    } else if file_path.extension() == Some(OsStr::new("wgsl")) {
                        vec![Self::read_file(&file_path)]
                    } else {
                        vec![]
                    }
                })
            })
            .collect()
        })
    }

    fn read_file(path: &Path) -> Result<Self, Error> {
        Ok(Self {
            path: path.into(),
            code: fs::read_to_string(path).map_err(|error| Error::Io(path.into(), error))?,
        })
    }
}
