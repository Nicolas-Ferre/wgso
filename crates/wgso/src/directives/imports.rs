use crate::directives::{Directive, DirectiveKind};
use crate::program::file::Files;
use crate::Error;
use std::path::{Path, PathBuf};
use wgso_parser::ParsingError;

impl Directive {
    pub(crate) fn import_path(&self, root_path: &Path) -> PathBuf {
        let segment_count = self.find_all_by_label("import_segment").count();
        let is_relative = self
            .find_all_by_label("import_segment")
            .next()
            .is_some_and(|segment| segment.slice == "~");
        let root_path = if is_relative {
            let file_path = self.path();
            file_path.parent().unwrap_or(file_path).to_path_buf()
        } else {
            root_path.to_path_buf()
        };
        self.find_all_by_label("import_segment")
            .enumerate()
            .filter(|(index, segment)| *index != 0 || segment.slice != "~")
            .fold(root_path, |path, (index, segment)| {
                if index == segment_count - 1 {
                    path.join(format!("{}.wgsl", segment.slice))
                } else if segment.slice == "~" {
                    path.parent().map(Path::to_path_buf).unwrap_or(path)
                } else {
                    path.join(&segment.slice)
                }
            })
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
