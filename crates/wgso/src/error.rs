use crate::directive::ShaderDirective;
use crate::storage::Storage;
use crate::Program;
use annotate_snippets::{Level, Renderer, Snippet};
use logos::Span;
use std::io;
use std::path::{Path, PathBuf};
use wgpu::naga::front::wgsl::ParseError;

/// A WGSO error.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An I/O error.
    Io(PathBuf, io::Error),
    /// A WGPU validation error.
    WgpuValidation(String),
    /// A Naga parsing error.
    WgslParsing(PathBuf, ParseError),
    /// Two shaders have been found with the same name.
    ShaderConflict(ShaderDirective, ShaderDirective),
    /// Two storages have been found with the same name.
    StorageConflict(Storage, Storage),
    /// A directive parsing error.
    DirectiveParsing(PathBuf, Span, String),
}

impl Error {
    /// Renders the error nicely.
    pub fn render(&self, program: &Program) -> String {
        match self {
            Self::Io(path, error) => Self::io_message(path, error),
            Self::WgpuValidation(error) => Self::wgpu_message(error),
            Self::WgslParsing(path, error) => Self::wgsl_parsing_message(program, path, error),
            Self::ShaderConflict(first, second) => {
                Self::shader_conflict_message(program, first, second)
            }
            Self::StorageConflict(first, second) => {
                Self::storage_conflict_message(program, first, second)
            }
            Self::DirectiveParsing(path, span, error) => {
                Self::directive_parsing_message(program, path, span.clone(), error)
            }
        }
    }

    pub(crate) fn path(&self) -> Option<&Path> {
        match self {
            Self::Io(path, _) | Self::WgslParsing(path, _) | Self::DirectiveParsing(path, _, _) => {
                Some(path)
            }
            Self::ShaderConflict(shader, _) => Some(&shader.path),
            Self::StorageConflict(storage, _) => Some(&storage.path),
            Self::WgpuValidation(_) => None,
        }
    }

    fn io_message(path: &Path, error: &io::Error) -> String {
        format!(
            "{}",
            Renderer::styled().render(Level::Error.title(&format!("{}: {error}", path.display())))
        )
    }

    fn wgsl_parsing_message(program: &Program, path: &PathBuf, error: &ParseError) -> String {
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

    fn shader_conflict_message(
        program: &Program,
        first: &ShaderDirective,
        second: &ShaderDirective,
    ) -> String {
        format!(
            "{}",
            Renderer::styled().render(
                Level::Error
                    .title(&format!("same name `{}` used for two shaders", first.name))
                    .snippet(
                        Snippet::source(&program.files[&first.path].code)
                            .fold(true)
                            .origin(&first.path.display().to_string())
                            .annotation(
                                Level::Error
                                    .span(first.span.clone())
                                    .label("first definition"),
                            )
                    )
                    .snippet(
                        Snippet::source(&program.files[&second.path].code)
                            .fold(true)
                            .origin(&second.path.display().to_string())
                            .annotation(
                                Level::Error
                                    .span(second.span.clone())
                                    .label("second definition"),
                            )
                    )
            )
        )
    }

    fn storage_conflict_message(program: &Program, first: &Storage, second: &Storage) -> String {
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
                            .origin(&first.path.display().to_string())
                            .annotation(
                                Level::Error
                                    .span(first.span.to_range().unwrap_or(0..0))
                                    .label("first definition"),
                            )
                    )
                    .snippet(
                        Snippet::source(&program.files[&second.path].code)
                            .fold(true)
                            .origin(&second.path.display().to_string())
                            .annotation(
                                Level::Error
                                    .span(second.span.to_range().unwrap_or(0..0))
                                    .label("second definition"),
                            )
                    )
            )
        )
    }

    fn wgpu_message(error: &str) -> String {
        format!("{}", Renderer::styled().render(Level::Error.title(error)))
    }

    fn directive_parsing_message(
        program: &Program,
        path: &Path,
        span: Span,
        error: &str,
    ) -> String {
        format!(
            "{}",
            Renderer::styled().render(
                Level::Error.title(error).snippet(
                    Snippet::source(&program.files[path].code)
                        .fold(true)
                        .origin(&path.display().to_string())
                        .annotation(Level::Error.span(span))
                )
            )
        )
    }
}
