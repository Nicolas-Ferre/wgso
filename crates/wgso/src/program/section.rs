use crate::directives::{toggle, Directive, DirectiveKind};
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
    pub(crate) fn new(files: &Files, root_path: &Path) -> Self {
        Self {
            sections: files
                .iter()
                .flat_map(|file| Self::file_sections(file, files, root_path))
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

    pub(crate) fn toggle_directives(&self) -> impl Iterator<Item = &Directive> {
        self.sections
            .values()
            .flat_map(|section| section.directives())
            .filter(|directive| directive.kind() == DirectiveKind::Toggle)
    }

    pub(crate) fn run_directives(&self) -> impl Iterator<Item = (&Directive, &Section)> {
        self.sections
            .values()
            .flat_map(|section| {
                section
                    .directives()
                    .map(move |directive| (directive, section))
            })
            .filter(|(directive, _)| {
                directive.kind() == DirectiveKind::Run || directive.kind() == DirectiveKind::Init
            })
            .enumerate()
            .sorted_unstable_by_key(|(index, (directive, _))| {
                (
                    directive.kind() != DirectiveKind::Init,
                    -directive.priority(),
                    directive.path(),
                    *index,
                )
            })
            .map(|(_, (directive, section))| (directive, &**section))
    }

    pub(crate) fn draw_directives(&self) -> impl Iterator<Item = (&Directive, &Section)> {
        self.sections
            .values()
            .flat_map(|section| {
                section
                    .directives()
                    .map(move |directive| (directive, section))
            })
            .filter(|(directive, _)| directive.kind() == DirectiveKind::Draw)
            .enumerate()
            .sorted_unstable_by_key(|(index, (directive, _))| {
                (-directive.priority(), directive.path(), *index)
            })
            .map(|(_, (directive, section))| (directive, &**section))
    }

    fn file_sections<'a>(
        file: &'a Arc<File>,
        files: &'a Files,
        root_path: &'a Path,
    ) -> impl Iterator<Item = Section> + 'a {
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
                    directive: directive.clone(),
                    span: directive.span().start..section_end,
                    toggle_var_names: files
                        .directives
                        .iter()
                        .filter(|toggle_directive| {
                            toggle_directive.kind() == DirectiveKind::Toggle
                                && toggle::has_section_path_prefix(
                                    directive,
                                    &toggle_directive.segment_path(root_path),
                                )
                        })
                        .map(|directive| directive.toggle_value_buffer().path())
                        .collect(),
                    file: file.clone(),
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
    pub(crate) toggle_var_names: Vec<String>,
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
