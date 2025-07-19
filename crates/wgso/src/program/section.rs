use crate::directives::{Directive, DirectiveKind};
use crate::program::file::{File, Files};
use fxhash::FxHashMap;
use itertools::Itertools;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Default)]
pub(crate) struct Sections {
    sections: FxHashMap<(PathBuf, String), Arc<Section>>,
}

impl Sections {
    pub(crate) fn new(files: &Files) -> Self {
        Self {
            sections: files
                .iter()
                .flat_map(Self::file_sections)
                .map(|section| {
                    (
                        (
                            section.directive.path().into(),
                            section.directive.section_name().slice.clone(),
                        ),
                        Arc::new(section),
                    )
                })
                .collect(),
        }
    }

    pub(crate) fn get(&self, path: &(PathBuf, String)) -> &Arc<Section> {
        &self.sections[path]
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Arc<Section>> {
        self.sections
            .iter()
            .sorted_unstable_by_key(|((path, name), _)| (path, name))
            .map(|(_, section)| section)
    }

    pub(crate) fn run_directives(&self) -> impl Iterator<Item = &Directive> {
        self.sections
            .values()
            .flat_map(|section| section.directives())
            .filter(|directive| {
                directive.kind() == DirectiveKind::Run || directive.kind() == DirectiveKind::Init
            })
            .enumerate()
            .sorted_unstable_by_key(|(index, directive)| {
                (
                    directive.kind() != DirectiveKind::Init,
                    -directive.priority(),
                    directive.path(),
                    *index,
                )
            })
            .map(|(_, directive)| directive)
    }

    pub(crate) fn draw_directives(&self) -> impl Iterator<Item = &Directive> {
        self.sections
            .values()
            .flat_map(|section| section.directives())
            .filter(|directive| directive.kind() == DirectiveKind::Draw)
            .enumerate()
            .sorted_unstable_by_key(|(index, directive)| {
                (-directive.priority(), directive.path(), *index)
            })
            .map(|(_, directive)| directive)
    }

    fn file_sections(file: &Arc<File>) -> impl Iterator<Item = Section> + '_ {
        let directives: Vec<_> = Self::section_directives(file).collect();
        directives
            .clone()
            .into_iter()
            .enumerate()
            .map(move |(index, directive)| {
                let section_end = directives
                    .get(index + 1)
                    .map_or(file.code.len(), |d| d.span().start);
                Section {
                    file: file.clone(),
                    span: directive.span().start..section_end,
                    directive: directive.clone(),
                }
            })
    }

    fn section_directives(file: &Arc<File>) -> impl Iterator<Item = &Directive> {
        file.directives.iter().filter(|directive| {
            [
                DirectiveKind::Mod,
                DirectiveKind::ComputeShader,
                DirectiveKind::RenderShader,
            ]
            .contains(&directive.kind())
        })
    }
}

#[derive(Debug)]
pub(crate) struct Section {
    pub(crate) directive: Directive,
    pub(crate) span: Range<usize>,
    file: Arc<File>,
}

impl Section {
    pub(crate) fn ident(&self) -> (PathBuf, String) {
        (
            self.file.path.clone(),
            self.directive.section_name().slice.clone(),
        )
    }

    pub(crate) fn directives(&self) -> impl Iterator<Item = &Directive> {
        self.file
            .directives
            .iter()
            .filter(|directive| self.span.contains(&directive.span().start))
    }

    pub(crate) fn code(&self) -> &str {
        &self.file.code[self.span.clone()]
    }

    pub(crate) fn path(&self) -> &Path {
        &self.file.path
    }
}
