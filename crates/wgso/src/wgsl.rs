use crate::error::Error;
use crate::file::File;
use naga::front::wgsl;
use naga::Module;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct Wgsl {
    pub(crate) path: PathBuf,
    pub(crate) module: Module,
    pub(crate) code: String,
}

impl Wgsl {
    pub(crate) fn parse(file: &File, errors: &mut Vec<Error>) -> Option<Self> {
        match wgsl::parse_str(&file.code) {
            Ok(module) => Some(Self {
                path: file.path.clone(),
                module,
                code: file.code.clone(),
            }),
            Err(error) => {
                errors.push(Error::Parsing(file.path.clone(), error));
                None
            }
        }
    }
}
