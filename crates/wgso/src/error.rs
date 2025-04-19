use crate::directive::tokens::Ident;
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
    /// A directive parsing error.
    DirectiveParsing(PathBuf, Span, String),
    /// Two shaders have been found with the same name.
    ShaderConflict(Ident, Ident),
    /// Two storages have been found with the same name.
    StorageConflict(PathBuf, PathBuf, String),
    /// WGSL code contains a feature unsupported by WGSO.
    UnsupportedWgslFeature(PathBuf, String),
}

impl Error {
    /// Renders the error nicely.
    pub fn render(&self, program: &Program) -> String {
        match self {
            Self::Io(path, error) => Self::io_message(path, error),
            Self::WgpuValidation(error) => Self::wgpu_validation_message(error),
            Self::WgslParsing(path, error) => Self::wgsl_parsing_message(program, path, error),
            Self::DirectiveParsing(path, span, error) => {
                Self::directive_parsing_message(program, path, span.clone(), error)
            }
            Self::ShaderConflict(first, second) => {
                Self::shader_conflict_message(program, first, second)
            }
            Self::StorageConflict(first, second, name) => {
                Self::storage_conflict_message(program, first, second, name)
            }
            Self::UnsupportedWgslFeature(path, message) => {
                Self::unsupported_wgsl_feature(program, path, message)
            }
        }
    }

    pub(crate) fn path(&self) -> Option<&Path> {
        match self {
            Self::Io(path, _) // no-coverage (not easy to test)
            | Self::WgslParsing(path, _)
            | Self::DirectiveParsing(path, _, _)
            | Self::StorageConflict(path, _, _)
            | Self::UnsupportedWgslFeature(path, _) => Some(path),
            Self::ShaderConflict(first, _) => Some(&first.path),
            Self::WgpuValidation(_) => None, // no-coverage (never called in practice)
        }
    }

    fn io_message(path: &Path, error: &io::Error) -> String {
        format!(
            "{}",
            Renderer::styled().render(Level::Error.title(&format!("{}: {error}", path.display())))
        )
    }

    fn wgsl_parsing_message(program: &Program, path: &Path, error: &ParseError) -> String {
        let path_str = path.display().to_string();
        let mut snippet = Snippet::source(program.files.code(path))
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

    fn wgpu_validation_message(error: &str) -> String {
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
                    Snippet::source(program.files.code(path))
                        .fold(true)
                        .origin(&path.display().to_string())
                        .annotation(Level::Error.span(span))
                )
            )
        )
    }

    fn shader_conflict_message(program: &Program, first: &Ident, second: &Ident) -> String {
        format!(
            "{}",
            Renderer::styled().render(
                Level::Error
                    .title(&format!("same name `{}` used for two shaders", first.label))
                    .snippet(
                        Snippet::source(program.files.code(&first.path))
                            .fold(true)
                            .origin(&first.path.display().to_string())
                            .annotation(
                                Level::Error
                                    .span(first.span.clone())
                                    .label("first definition"),
                            )
                    )
                    .snippet(
                        Snippet::source(program.files.code(&second.path))
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

    fn storage_conflict_message(
        program: &Program,
        first: &Path,
        second: &Path,
        name: &str,
    ) -> String {
        format!(
            "{}",
            Renderer::styled().render(
                Level::Error
                    .title(&format!(
                        "same name `{name}` used for two storage variables"
                    ))
                    .snippet(
                        Snippet::source(program.files.code(first))
                            .fold(true)
                            .origin(&first.display().to_string())
                    )
                    .snippet(
                        Snippet::source(program.files.code(second))
                            .fold(true)
                            .origin(&second.display().to_string())
                    )
            )
        )
    }

    fn unsupported_wgsl_feature(program: &Program, path: &Path, message: &str) -> String {
        format!(
            "{}",
            Renderer::styled().render(
                Level::Error.title(message).snippet(
                    Snippet::source(program.files.code(path))
                        .fold(true)
                        .origin(&path.display().to_string())
                )
            )
        )
    }
}
