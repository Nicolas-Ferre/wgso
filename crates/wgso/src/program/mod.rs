use crate::directives::{imports, shader_calls, shader_defs};
use crate::program::type_::Type;
use crate::Error;
use file::Files;
use itertools::Itertools;
use module::Modules;
use std::path::{Path, PathBuf};

pub(crate) mod file;
pub(crate) mod module;
pub(crate) mod type_;
mod wgsl;

/// A parsed WGSO program.
#[derive(Debug)]
pub struct Program {
    /// The errors found during parsing.
    pub errors: Vec<Error>,
    pub(crate) root_path: PathBuf,
    pub(crate) files: Files,
    pub(crate) modules: Modules,
}

impl Program {
    /// Render found errors.
    pub fn render_errors(&self) -> String {
        self.errors
            .iter()
            .map(|err| err.render(self))
            .unique()
            .join("\n")
    }

    pub(crate) fn parse(root_path: impl AsRef<Path>) -> Self {
        let root_path = root_path.as_ref();
        let mut errors = vec![];
        let directive_rules = crate::directives::load_rules();
        let files = Files::new(root_path, &directive_rules, &mut errors);
        if !errors.is_empty() {
            return Self {
                errors,
                root_path: root_path.into(),
                files,
                modules: Modules::default(),
            };
        }
        imports::check(&files.directives, &files, root_path, &mut errors);
        shader_defs::check(&files.directives, &mut errors);
        shader_calls::check(&files.directives, root_path, &mut errors);
        if !errors.is_empty() {
            return Self {
                errors,
                root_path: root_path.into(),
                files,
                modules: Modules::default(),
            };
        }
        let modules = Modules::new(root_path, &files, &mut errors);
        if !errors.is_empty() {
            return Self {
                errors,
                root_path: root_path.into(),
                files,
                modules,
            };
        }
        shader_defs::check_params(&modules, &mut errors);
        shader_calls::check_args(root_path, &files, &modules, &mut errors);
        Self {
            errors,
            root_path: root_path.into(),
            files,
            modules,
        }
    }

    pub(crate) fn with_sorted_errors(mut self) -> Self {
        self.errors
            .sort_unstable_by_key(|e| e.path().map(Path::to_path_buf));
        self
    }

    pub(crate) fn parse_field(&self, field_path: &str) -> Option<StorageField<'_>> {
        let segments: Vec<_> = field_path.split('.').collect();
        let storage_type = &self.modules.storages.get(segments[0])?;
        let field_type = storage_type.field_name_type(&segments[1..])?;
        Some(StorageField {
            buffer_name: segments[0].into(),
            type_: field_type,
        })
    }
}

#[derive(Debug)]
pub(crate) struct StorageField<'a> {
    pub(crate) buffer_name: String,
    pub(crate) type_: &'a Type,
}
