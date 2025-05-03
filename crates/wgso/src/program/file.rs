use crate::directives::{Directive, DirectiveKind};
use crate::Error;
use fxhash::FxHashMap;
use itertools::Itertools;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::vec::IntoIter;
use walkdir::{DirEntry, WalkDir};
use wgso_parser::Rule;

#[derive(Debug)]
pub(crate) struct Files {
    files: FxHashMap<PathBuf, Arc<File>>,
    pub(crate) directives: Vec<Directive>,
}

impl Files {
    pub(crate) fn new(path: &Path, directive_rules: &[Rule], errors: &mut Vec<Error>) -> Self {
        let files: FxHashMap<_, _> = WalkDir::new(path)
            .into_iter()
            .filter_map(|file| match file {
                Ok(file) => {
                    if Self::is_wgsl_file(&file) {
                        File::new(file.path(), directive_rules, errors)
                            .map(|file| (file.path.clone(), Arc::new(file)))
                    } else {
                        None
                    }
                }
                Err(error) => {
                    if let Some(error) = error.into_io_error() {
                        errors.push(Error::Io(path.into(), error));
                    }
                    None
                }
            })
            .collect();
        let directives = files
            .values()
            .sorted_unstable_by_key(|file| &file.path)
            .flat_map(|file| file.directives.iter().cloned())
            .collect();
        Self { files, directives }
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

    pub(crate) fn run_directives(&self) -> impl Iterator<Item = &Directive> {
        self.directives
            .iter()
            .filter(|directive| {
                directive.kind() == DirectiveKind::Run || directive.kind() == DirectiveKind::Init
            })
            .sorted_by_key(|directive| {
                (
                    directive.kind() != DirectiveKind::Init,
                    directive.priority(),
                    directive.shader_name().slice.clone(),
                )
            })
    }

    pub(crate) fn draw_directives(&self) -> impl Iterator<Item = &Directive> {
        self.directives
            .iter()
            .filter(|directive| directive.kind() == DirectiveKind::Draw)
            .sorted_by_key(|directive| {
                (directive.priority(), directive.shader_name().slice.clone())
            })
    }

    fn is_wgsl_file(file: &DirEntry) -> bool {
        !file.file_type().is_dir() && file.path().extension() == Some(OsStr::new("wgsl"))
    }
}

#[derive(Debug)]
pub(crate) struct File {
    pub(crate) path: PathBuf,
    pub(crate) code: String,
    pub(crate) directives: Vec<Directive>,
}

impl File {
    fn new(path: &Path, directive_rules: &[Rule], errors: &mut Vec<Error>) -> Option<Self> {
        match fs::read_to_string(path) {
            Ok(code) => Some(Self {
                path: path.into(),
                directives: crate::directives::parse_file(&code, path, directive_rules, errors),
                code,
            }),
            // coverage: off (not easy to test)
            Err(error) => {
                errors.push(Error::Io(path.into(), error));
                None
            }
        } // coverage: on
    }
}
