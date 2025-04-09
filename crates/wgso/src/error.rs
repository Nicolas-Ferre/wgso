use crate::storage::Storage;
use crate::Program;
use annotate_snippets::{Level, Renderer, Snippet};
use naga::front::wgsl::ParseError;
use std::io;
use std::path::{Path, PathBuf};

/// A WGSO error.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An I/O error.
    Io(PathBuf, io::Error),
    /// A Naga parsing error.
    Parsing(PathBuf, ParseError),
    /// Two storages have been found with the same name.
    StorageConflict(Storage, Storage),
}

impl Error {
    /// Renders the error nicely.
    pub fn render(&self, program: &Program) -> String {
        match self {
            Self::Io(path, error) => {
                format!(
                    "{}",
                    Renderer::styled()
                        .render(Level::Error.title(&format!("{}: {error}", path.display())))
                )
            }
            Self::Parsing(path, error) => {
                let path_str = path.display().to_string();
                let mut snippet = Snippet::source(&program.files[path].code)
                    .fold(true)
                    .origin(&path_str);
                for (span, label) in error.labels() {
                    snippet = snippet.annotation(
                        Level::Error
                            .span(span.to_range().unwrap_or(0..0))
                            .label(label),
                    );
                }
                format!(
                    "{}",
                    Renderer::styled().render(Level::Error.title(error.message()).snippet(snippet))
                )
            }
            Self::StorageConflict(first, second) => {
                let first_path_str = first.path.display().to_string();
                let second_path_str = second.path.display().to_string();
                format!(
                    "{}",
                    Renderer::styled().render(
                        Level::Error
                            .title(&format!(
                                "same name `{}` used for two storage variables",
                                first.name
                            ))
                            .snippet(
                                Snippet::source(&program.files[&first.path].code)
                                    .fold(true)
                                    .origin(&first_path_str)
                                    .annotation(
                                        Level::Error
                                            .span(first.span.to_range().unwrap_or(0..0))
                                            .label("first definition"),
                                    )
                            )
                            .snippet(
                                Snippet::source(&program.files[&second.path].code)
                                    .fold(true)
                                    .origin(&second_path_str)
                                    .annotation(
                                        Level::Error
                                            .span(second.span.to_range().unwrap_or(0..0))
                                            .label("second definition"),
                                    )
                            )
                    )
                )
            }
        }
    }

    pub(crate) fn path(&self) -> &Path {
        match self {
            Self::Io(path, _) | Self::Parsing(path, _) => path,
            Self::StorageConflict(first, _) => &first.path,
        }
    }
}
