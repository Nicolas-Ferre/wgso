use crate::directives::{Directive, DirectiveKind};
use crate::program::file::Files;
use crate::Error;
use std::path::{Path, PathBuf};
use wgso_parser::ParsingError;

impl Directive {
    pub(crate) fn import_path(&self, root_path: &Path) -> PathBuf {
        self.segment_path(root_path, true)
    }
}

pub(crate) fn check(
    directives: &[Directive],
    files: &Files,
    root_path: &Path,
    errors: &mut Vec<Error>,
) {
    for directive in directives {
        if directive.kind() == DirectiveKind::Import {
            let path = directive.import_path(root_path);
            if !files.exists(&path) {
                errors.push(Error::DirectiveParsing(ParsingError {
                    path: directive.path().into(),
                    span: directive.span(),
                    message: format!("file at '{}' does not exist", path.display()),
                }));
            }
        }
    }
}
