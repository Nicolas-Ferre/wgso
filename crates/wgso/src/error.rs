use crate::program::file::File;
use crate::Program;
use annotate_snippets::{Level, Renderer, Snippet};
use naga::valid::ValidationError;
use naga::WithSpan;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{error, io};
use wgpu::naga::front::wgsl::ParseError;
use wgso_parser::{ParsingError, Token};

/// A WGSO error.
#[derive(Debug)]
#[non_exhaustive]
#[allow(private_interfaces)]
pub enum Error {
    /// An I/O error.
    Io(PathBuf, io::Error),
    /// A WGPU validation error.
    WgpuValidation(String),
    /// A Naga parsing error.
    WgslParsing(Vec<Arc<File>>, ParseError),
    /// A Naga validation error.
    WgslValidation(Vec<Arc<File>>, WithSpan<ValidationError>),
    /// A directive parsing error.
    DirectiveParsing(ParsingError),
    /// Two shaders have been found with the same name.
    ///
    /// Last value is the type of shader.
    ShaderConflict(Token, Token, &'static str),
    /// Two storages have been found with the same name.
    StorageConflict(PathBuf, PathBuf, String),
    /// WGSL code contains a feature unsupported by WGSO.
    UnsupportedWgslFeature(PathBuf, String),
    /// Program cannot be reloaded because storage structure has changed.
    ChangedStorageStructure,
}

impl Error {
    /// Renders the error nicely.
    pub fn render(&self, program: &Program) -> String {
        match self {
            Self::Io(path, error) => Self::io_message(path, error),
            Self::WgpuValidation(error) => Self::wgpu_validation_message(error),
            Self::WgslParsing(files, error) => Self::wgsl_parsing_message(program, files, error),
            Self::WgslValidation(files, error) => {
                Self::wgsl_validation_message(program, files, error)
            }
            Self::DirectiveParsing(error) => Self::directive_parsing_message(program, error),
            Self::ShaderConflict(first, second, type_) => {
                Self::shader_conflict_message(program, first, second, type_)
            }
            Self::StorageConflict(first, second, name) => {
                Self::storage_conflict_message(program, first, second, name)
            }
            Self::UnsupportedWgslFeature(path, message) => {
                Self::unsupported_wgsl_feature_message(program, path, message)
            }
            Self::ChangedStorageStructure => Self::changed_storage_structure_message(),
        }
    }

    pub(crate) fn path(&self) -> Option<&Path> {
        match self {
            Self::Io(path, _) // no-coverage (not easy to test)
            | Self::StorageConflict(path, _, _)
            | Self::UnsupportedWgslFeature(path, _) => Some(path),
            Self::DirectiveParsing(error) => Some(&error.path),
            Self::WgslParsing(module, error) => Some(Self::wgsl_parsing_error_path(module, error)),
            Self::WgslValidation(module, error) => Some(Self::wgsl_validation_error_path(module, error)),
            Self::ShaderConflict(first, _, _) => Some(&first.path),
            Self::WgpuValidation(_)|Self::ChangedStorageStructure => None, // no-coverage (never called in practice)
        }
    }

    fn wgsl_parsing_error_path<'a>(files: &'a [Arc<File>], error: &'a ParseError) -> &'a Path {
        Self::merged_file(
            files,
            error
                .labels()
                .next()
                .map_or(0, |(span, _)| span.to_range().unwrap_or(0..0).start),
        )
        .0
    }

    fn wgsl_validation_error_path<'a>(
        files: &'a [Arc<File>],
        error: &'a WithSpan<ValidationError>,
    ) -> &'a Path {
        Self::merged_file(
            files,
            error
                .spans()
                .next()
                .map_or(0, |(span, _)| span.to_range().unwrap_or(0..0).start),
        )
        .0
    }

    fn merged_file(files: &[Arc<File>], offset: usize) -> (&Path, usize, usize) {
        let mut current_offset = 0;
        for (index, file) in files.iter().enumerate() {
            if offset < current_offset + file.code.len() || index == files.len() - 1 {
                return (&file.path, current_offset, file.code.len());
            }
            current_offset += file.code.len();
        }
        unreachable!("internal error: invalid span")
    }

    fn io_message(path: &Path, error: &io::Error) -> String {
        format!(
            "{}",
            Renderer::styled().render(Level::Error.title(&format!("{}: {error}", path.display())))
        )
    }

    fn wgsl_parsing_message(program: &Program, files: &[Arc<File>], error: &ParseError) -> String {
        let mut message = Level::Error.title(error.message());
        let paths: Vec<_> = error
            .labels()
            .map(|(naga_span, _)| {
                let span = naga_span.to_range().unwrap_or(0..0);
                let (path, offset, max_len) = Self::merged_file(files, span.start);
                let path_str = path.display().to_string();
                (
                    (span.start - offset).min(max_len)..(span.end - offset).min(max_len),
                    path,
                    path_str,
                )
            })
            .collect();
        for ((_, label), (span, path, path_str)) in error.labels().zip(&paths) {
            message = message.snippet(
                Snippet::source(&program.files.get(path).code)
                    .fold(true)
                    .origin(path_str)
                    .annotation(Level::Error.span(span.clone()).label(label)),
            );
        }
        format!("{}", Renderer::styled().render(message))
    }

    fn wgsl_validation_message(
        program: &Program,
        files: &[Arc<File>],
        error: &WithSpan<ValidationError>,
    ) -> String {
        let paths: Vec<_> = error
            .spans()
            .map(|(naga_span, label)| {
                let span = naga_span.to_range().unwrap_or(0..0);
                let (path, offset, max_len) = Self::merged_file(files, span.start);
                let path_str = path.display().to_string();
                (
                    label,
                    (span.start - offset).min(max_len)..(span.end - offset).min(max_len),
                    path,
                    path_str,
                )
            })
            .collect();
        let error_message = error.to_string();
        let mut message = Level::Error.title(&error_message);
        let source = error::Error::source(error.as_inner()).map(ToString::to_string);
        if let Some(source) = &source {
            message = message.footer(Level::Info.title(source));
        };
        for (label, span, path, path_str) in &paths {
            message = message.snippet(
                Snippet::source(&program.files.get(path).code)
                    .fold(true)
                    .origin(path_str)
                    .annotation(Level::Error.span(span.clone()).label(label)),
            );
        }
        format!(
            "{}",
            Renderer::styled().render(message.footer(Level::Info.title(&format!(
                "The error comes from file '{}'",
                files[0].path.display()
            ))))
        )
    }

    fn wgpu_validation_message(error: &str) -> String {
        format!("{}", Renderer::styled().render(Level::Error.title(error)))
    }

    fn directive_parsing_message(program: &Program, error: &ParsingError) -> String {
        format!(
            "{}",
            Renderer::styled().render(
                Level::Error.title(&error.message).snippet(
                    Snippet::source(&program.files.get(&error.path).code)
                        .fold(true)
                        .origin(&error.path.display().to_string())
                        .annotation(Level::Error.span(error.span.clone()))
                )
            )
        )
    }

    fn shader_conflict_message(
        program: &Program,
        first: &Token,
        second: &Token,
        type_: &str,
    ) -> String {
        format!(
            "{}",
            Renderer::styled().render(
                Level::Error
                    .title(&format!(
                        "same name `{}` used for two {type_} shaders",
                        first.slice
                    ))
                    .snippet(
                        Snippet::source(&program.files.get(&first.path).code)
                            .fold(true)
                            .origin(&first.path.display().to_string())
                            .annotation(
                                Level::Error
                                    .span(first.span.clone())
                                    .label("first definition"),
                            )
                    )
                    .snippet(
                        Snippet::source(&program.files.get(&second.path).code)
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
                        Snippet::source(&program.files.get(first).code)
                            .fold(true)
                            .origin(&first.display().to_string())
                    )
                    .snippet(
                        Snippet::source(&program.files.get(second).code)
                            .fold(true)
                            .origin(&second.display().to_string())
                    )
            )
        )
    }

    fn unsupported_wgsl_feature_message(program: &Program, path: &Path, message: &str) -> String {
        format!(
            "{}",
            Renderer::styled().render(
                Level::Error.title(message).snippet(
                    Snippet::source(&program.files.get(path).code)
                        .fold(true)
                        .origin(&path.display().to_string())
                )
            )
        )
    }

    fn changed_storage_structure_message() -> String {
        format!(
            "{}",
            Renderer::styled().render(
                Level::Error
                    .title("program cannot be hot-reloaded because storages have been changed")
            )
        )
    }
}
