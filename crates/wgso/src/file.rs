use crate::Error;
use fxhash::FxHashMap;
use itertools::Itertools;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::vec::IntoIter;
use wgso_parser::{Rule, Token};
// TODO: flatten logic with a lib

#[derive(Debug)]
pub(crate) struct Files {
    files: FxHashMap<PathBuf, Arc<File>>,
}

impl Files {
    pub(crate) fn new(path: &Path, directive_rules: &[Rule], errors: &mut Vec<Error>) -> Self {
        Self {
            files: Self::list_wgsl_files_recursively(path, directive_rules, errors)
                .into_iter()
                .map(|file| (file.path.clone(), Arc::new(file)))
                .collect(),
        }
    }

    pub(crate) fn iter(&self) -> IntoIter<&Arc<File>> {
        self.files
            .values()
            .sorted_unstable_by_key(|file| &file.path)
    }

    pub(crate) fn get(&self, path: &Path) -> &Arc<File> {
        &self.files[path]
    }

    pub(crate) fn exists(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    fn list_wgsl_files_recursively(
        path: &Path,
        directive_rules: &[Rule],
        errors: &mut Vec<Error>,
    ) -> Vec<File> {
        let error_fn = |error, errors: &mut Vec<_>| {
            errors.push(error);
            vec![]
        };
        match fs::read_dir(path) {
            Ok(dirs) => dirs
                .flat_map(|dir| match dir {
                    Ok(entry) => {
                        let file_path = entry.path();
                        if file_path.is_dir() {
                            Self::list_wgsl_files_recursively(&file_path, directive_rules, errors)
                        } else if file_path.extension() == Some(OsStr::new("wgsl")) {
                            match File::new(&file_path, directive_rules, errors) {
                                Ok(file) => vec![file],
                                Err(error) => error_fn(error, errors), // no-coverage (not easy to test)
                            }
                        } else {
                            vec![]
                        }
                    }
                    Err(error) => error_fn(Error::Io(path.into(), error), errors), // no-coverage (not easy to test)
                })
                .collect(),
            Err(error) => error_fn(Error::Io(path.into(), error), errors),
        }
    }
}

#[derive(Debug)]
pub(crate) struct File {
    pub(crate) path: PathBuf,
    pub(crate) code: String,
    pub(crate) directives: Vec<Vec<Token>>,
}

impl File {
    fn new(path: &Path, directive_rules: &[Rule], errors: &mut Vec<Error>) -> Result<Self, Error> {
        let code = fs::read_to_string(path).map_err(|error| Error::Io(path.into(), error))?;
        Ok(Self {
            path: path.into(),
            directives: crate::directive::parse_file(&code, path, directive_rules, errors),
            code,
        })
    }
}
