use crate::program::file::SourceFolder;
use crate::program::section::Sections;
use crate::program::type_::Type;
use crate::{directives, Error};
use file::Files;
use itertools::Itertools;
use module::Modules;
use std::path::{Path, PathBuf};

pub(crate) mod file;
pub(crate) mod module;
pub(crate) mod section;
pub(crate) mod type_;
mod wgsl;

/// A parsed WGSO program.
#[derive(Debug)]
pub struct Program {
    /// The errors found during parsing.
    pub errors: Vec<Error>,
    pub(crate) root_path: PathBuf,
    pub(crate) files: Files,
    pub(crate) sections: Sections,
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

    pub(crate) fn parse(source: impl SourceFolder) -> Self {
        let root_path = source.path();
        let mut errors = vec![];
        let directive_rules = directives::load_rules();
        let files = Files::new(source, &directive_rules, &mut errors);
        if !errors.is_empty() {
            return Self {
                errors,
                root_path,
                files,
                sections: Sections::default(),
                modules: Modules::default(),
            };
        }
        directives::defs::check(&files, &mut errors);
        let sections = Sections::new(&files, &root_path);
        for section in sections.iter() {
            directives::calls::check(section.directives(), &files, &root_path, &mut errors);
        }
        if !errors.is_empty() {
            return Self {
                errors,
                root_path,
                files,
                sections,
                modules: Modules::default(),
            };
        }
        let modules = Modules::new(&root_path, &sections, &mut errors);
        if !errors.is_empty() {
            return Self {
                errors,
                root_path,
                files,
                sections,
                modules,
            };
        }
        directives::defs::check_params(&modules, &mut errors);
        directives::calls::check_args(&root_path, &sections, &modules, &mut errors);
        if !errors.is_empty() {
            return Self {
                errors,
                root_path,
                files,
                sections,
                modules,
            };
        }
        directives::toggle::check(&sections, &modules, &root_path, &mut errors);
        Self {
            errors,
            root_path,
            files,
            sections,
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
        let storage = &self.modules.storages.get(segments[0])?;
        let field_type = storage.type_.field_name_type(&segments[1..])?;
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
