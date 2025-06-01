use crate::directives::{Directive, DirectiveKind};
use crate::program::section::{Section, Sections};
use crate::program::type_;
use crate::program::type_::Type;
use crate::program::wgsl::{Binding, BindingKind, WgslModule};
use crate::Error;
use fxhash::{FxHashMap, FxHashSet};
use itertools::Itertools;
use std::collections::hash_map::Entry;
use std::iter;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Default)]
pub(crate) struct Modules {
    pub(crate) storages: FxHashMap<String, Arc<Type>>,
    pub(crate) compute: FxHashMap<(PathBuf, String), Arc<Module>>,
    pub(crate) render: FxHashMap<(PathBuf, String), Arc<Module>>,
}

impl Modules {
    pub(crate) fn new(root_path: &Path, sections: &Sections, errors: &mut Vec<Error>) -> Self {
        let modules = sections
            .iter()
            .filter_map(|section| match Module::new(root_path, section, sections) {
                Ok(module) => Some(Arc::new(module)),
                Err(error) => {
                    errors.push(error);
                    None
                }
            })
            .collect::<Vec<_>>();
        Self {
            storages: Self::storages(&modules, errors),
            compute: Self::shaders(&modules, DirectiveKind::ComputeShader),
            render: Self::shaders(&modules, DirectiveKind::RenderShader),
        }
    }

    fn storages(modules: &[Arc<Module>], errors: &mut Vec<Error>) -> FxHashMap<String, Arc<Type>> {
        let mut storages = FxHashMap::default();
        for module in modules {
            for (name, binding) in module.storage_bindings() {
                match storages.entry(name.clone()) {
                    Entry::Vacant(entry) => {
                        entry.insert((module.clone(), binding.type_.clone()));
                    }
                    Entry::Occupied(existing) => {
                        let existing = existing.get();
                        if existing.1 != binding.type_ {
                            errors.push(Error::StorageConflict(
                                existing.0.wgsl.sections[0].path().into(),
                                module.wgsl.sections[0].path().into(),
                                name.clone(),
                            ));
                        }
                    }
                }
            }
        }
        storages
            .into_iter()
            .map(|(name, (_, type_))| (name, type_))
            .collect()
    }

    fn shaders(
        modules: &[Arc<Module>],
        kind: DirectiveKind,
    ) -> FxHashMap<(PathBuf, String), Arc<Module>> {
        modules
            .iter()
            .filter(|module| module.main_directive().kind() == kind)
            .map(|module| (module.wgsl.sections[0].ident(), module.clone()))
            .collect()
    }
}

#[derive(Debug)]
pub(crate) struct Module {
    pub(crate) code: String,
    wgsl: WgslModule,
    types: FxHashMap<String, Type>,
    bindings: FxHashMap<String, Binding>,
}

impl Module {
    pub(crate) fn new(
        root_path: &Path,
        section: &Arc<Section>,
        sections: &Sections,
    ) -> Result<Self, Error> {
        let (code, sections) = Self::extract_code(root_path, section, sections);
        let mut wgsl = WgslModule::new(&code, sections)?;
        let bindings = wgsl.configure_bindings();
        wgsl.configure_buffer_types();
        Ok(Self {
            code: wgsl.to_code()?,
            types: wgsl.extract_types(),
            wgsl,
            bindings,
        })
    }

    pub(crate) fn binding_count(&self) -> usize {
        self.bindings.len()
    }

    pub(crate) fn storage_bindings(&self) -> impl Iterator<Item = (&String, &Binding)> + '_ {
        self.bindings
            .iter()
            .filter(|(_, binding)| binding.kind == BindingKind::Storage)
    }

    pub(crate) fn uniform_bindings(&self) -> impl Iterator<Item = (&String, &Binding)> + '_ {
        self.bindings
            .iter()
            .filter(|(_, binding)| binding.kind == BindingKind::Uniform)
    }

    pub(crate) fn uniform_names(&self) -> impl Iterator<Item = &String> + '_ {
        self.bindings
            .iter()
            .filter(|(_, binding)| binding.kind == BindingKind::Uniform)
            .map(|(name, _)| name)
    }

    pub(crate) fn uniform_binding(&self, name: &str) -> Option<&Binding> {
        self.bindings
            .iter()
            .find(|(binding_name, binding)| {
                binding.kind == BindingKind::Uniform && binding_name == &name
            })
            .map(|(_, binding)| binding)
    }

    pub(crate) fn type_(&self, name: &str) -> Option<&Type> {
        let type_name = type_::normalize_type_name(name);
        self.types.get(&type_name)
    }

    pub(crate) fn main_directive(&self) -> &Directive {
        &self.wgsl.sections[0].directive
    }

    fn extract_code(
        root_path: &Path,
        section: &Arc<Section>,
        sections: &Sections,
    ) -> (String, Vec<Arc<Section>>) {
        let imported_sections: Vec<_> = Self::extract_section_idents(root_path, section, sections)
            .into_iter()
            .map(|ident| sections.get(&ident).clone())
            .sorted_unstable_by_key(|current_section| {
                current_section.path() != section.path()
                    || current_section.directive.section_name().slice
                        != section.directive.section_name().slice
            })
            .collect();
        let code = imported_sections
            .iter()
            .map(|section| {
                section
                    .code()
                    .lines()
                    .map(|line| {
                        if line.trim_start().starts_with('#') {
                            format!("{: ^1$}\n", "", line.len())
                        } else {
                            format!("{line}\n")
                        }
                    })
                    .join("")
            })
            .join("");
        (code, imported_sections)
    }

    fn extract_section_idents(
        root_path: &Path,
        section: &Arc<Section>,
        sections: &Sections,
    ) -> Vec<(PathBuf, String)> {
        let mut idents: FxHashSet<_> = iter::once(section.ident()).collect();
        let mut last_path_count = 0;
        while last_path_count < idents.len() {
            last_path_count = idents.len();
            for ident in idents.clone() {
                let import_directives = sections
                    .get(&ident)
                    .directives()
                    .filter(|directive| directive.kind() == DirectiveKind::Import);
                for directive in import_directives {
                    idents.insert(directive.item_ident(root_path));
                }
            }
        }
        idents.into_iter().collect()
    }
}
