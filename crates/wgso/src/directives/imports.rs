use crate::directives::{Directive, DirectiveKind};
use crate::program::file::Files;
use crate::Error;
use std::path::{Path, PathBuf};
use wgso_parser::ParsingError;

impl Directive {
    pub(crate) fn import_path(&self) -> PathBuf {
        let segment_count = self.find_all_by_label("import_segment").count();
        self.find_all_by_label("import_segment")
            .enumerate()
            .map(|(index, segment)| {
                if index == segment_count - 1 {
                    format!("{}.wgsl", segment.slice.clone())
                } else {
                    segment.slice.clone()
                }
            })
            .collect::<PathBuf>()
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
            let path = root_path.join(directive.import_path());
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
