use crate::directives::Directive;
use crate::Error;
use fxhash::FxHashMap;
use itertools::Itertools;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::vec::IntoIter;
use wgso_parser::{ParsingError, Rule};

#[derive(Debug)]
pub(crate) struct Files {
    files: FxHashMap<PathBuf, Arc<File>>,
    pub(crate) directives: Vec<Directive>,
}

impl Files {
    pub(crate) fn new(
        source: impl SourceFolder,
        directive_rules: &[Rule],
        errors: &mut Vec<Error>,
    ) -> Self {
        let files: FxHashMap<_, _> = source.parse(directive_rules, errors);
        let directives = Self::directives(&files);
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

    fn directives(files: &FxHashMap<PathBuf, Arc<File>>) -> Vec<Directive> {
        files
            .values()
            .sorted_unstable_by_key(|file| &file.path)
            .flat_map(|file| file.directives.iter().cloned())
            .collect()
    }

    fn is_wgsl_file(file: &walkdir::DirEntry) -> bool {
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
    fn new(code: String, path: &Path, directive_rules: &[Rule], errors: &mut Vec<Error>) -> Self {
        let directives = crate::directives::parse_file(&code, path, directive_rules, errors);
        Self::check_file_header(path, &code, errors);
        Self {
            path: path.into(),
            directives,
            code,
        }
    }

    fn check_file_header(path: &Path, code: &str, errors: &mut Vec<Error>) {
        let mut offset = 0;
        for line in code.lines() {
            let trimmed_line = line.trim_start();
            if let Some(directive) = trimmed_line.strip_prefix("#") {
                if directive.trim_start().starts_with("mod")
                    || directive.trim_start().starts_with("shader")
                {
                    return;
                }
            }
            if !trimmed_line.is_empty() && !trimmed_line.starts_with("//") {
                errors.push(Error::DirectiveParsing(ParsingError {
                    path: path.into(),
                    span: offset..offset,
                    message: "file should start with `#mod` or `#shader` directive".into(),
                }));
                return;
            }
            offset += line.len() + 1;
        }
    }
}

/// A trait implemented for source folder accessors.
pub trait SourceFolder: Clone {
    /// Extract all WGSL files.
    #[allow(private_interfaces)]
    fn parse(
        self,
        directive_rules: &[Rule],
        errors: &mut Vec<Error>,
    ) -> FxHashMap<PathBuf, Arc<File>>;

    /// Returns folder path.
    fn path(&self) -> PathBuf;
}

impl SourceFolder for &Path {
    #[allow(private_interfaces)]
    fn parse(
        self,
        directive_rules: &[Rule],
        errors: &mut Vec<Error>,
    ) -> FxHashMap<PathBuf, Arc<File>> {
        walkdir::WalkDir::new(self)
            .follow_links(true)
            .into_iter()
            .filter_map(|file| match file {
                Ok(file) => {
                    if Files::is_wgsl_file(&file) {
                        match fs::read_to_string(file.path()) {
                            Ok(code) => Some(File::new(code, file.path(), directive_rules, errors)),
                            // coverage: off (not easy to test)
                            Err(error) => {
                                errors.push(Error::Io(file.path().into(), error));
                                None
                            } // coverage: on
                        }
                        .map(|file| (file.path.clone(), Arc::new(file)))
                    } else {
                        None
                    }
                }
                Err(error) => {
                    if let Some(error) = error.into_io_error() {
                        errors.push(Error::Io(self.into(), error));
                    }
                    None
                }
            })
            .collect()
    }

    fn path(&self) -> PathBuf {
        self.into()
    }
}

// coverage: off (not used on native platforms)
impl SourceFolder for include_dir::Dir<'_> {
    #[allow(private_interfaces)]
    fn parse(
        self,
        directive_rules: &[Rule],
        errors: &mut Vec<Error>,
    ) -> FxHashMap<PathBuf, Arc<File>> {
        self.entries()
            .iter()
            .flat_map(|entry| match entry {
                include_dir::DirEntry::Dir(dir) => {
                    Self::parse(dir.clone(), directive_rules, errors)
                }
                include_dir::DirEntry::File(file) => {
                    if file.path().extension() == Some(OsStr::new("wgsl")) {
                        std::iter::once((
                            file.path().into(),
                            Arc::new(File::new(
                                String::from_utf8_lossy(file.contents()).into(),
                                file.path(),
                                directive_rules,
                                errors,
                            )),
                        ))
                        .collect()
                    } else {
                        FxHashMap::default()
                    }
                }
            })
            .collect()
    }

    fn path(&self) -> PathBuf {
        include_dir::Dir::path(self).into()
    }
}
// coverage: on
