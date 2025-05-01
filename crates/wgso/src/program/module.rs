use crate::directives::{Directive, DirectiveKind};
use crate::program::file::{File, Files};
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
    pub(crate) compute_shaders: FxHashMap<String, (Directive, Arc<Module>)>,
    pub(crate) render_shaders: FxHashMap<String, (Directive, Arc<Module>)>,
}

impl Modules {
    pub(crate) fn new(root_path: &Path, files: &Files, errors: &mut Vec<Error>) -> Self {
        let modules = files
            .iter()
            .filter_map(|file| match Module::new(root_path, file, files) {
                Ok(module) => Some(Arc::new(module)),
                Err(error) => {
                    errors.push(error);
                    None
                }
            })
            .collect::<Vec<_>>();
        Self {
            storages: Self::storages(&modules, errors),
            compute_shaders: Self::shaders(&modules, DirectiveKind::ComputeShader),
            render_shaders: Self::shaders(&modules, DirectiveKind::RenderShader),
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
                                existing.0.wgsl.files[0].path.clone(),
                                module.wgsl.files[0].path.clone(),
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
    ) -> FxHashMap<String, (Directive, Arc<Module>)> {
        modules
            .iter()
            .flat_map(|module| {
                crate::directives::find_all_by_kind(&module.wgsl.files[0].directives, kind)
                    .map(|directive| (directive.clone(), module.clone()))
            })
            .map(|(directive, module)| (directive.shader_name().slice.clone(), (directive, module)))
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
    pub(crate) fn new(root_path: &Path, file: &Arc<File>, files: &Files) -> Result<Self, Error> {
        let (code, module_files) = Self::extract_code(root_path, file, files);
        let mut wgsl = WgslModule::new(&code, module_files)?;
        let bindings = wgsl.configure_bindings();
        wgsl.configure_vertex_buffer();
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
        let type_name = type_::normalize_type_name(name)?;
        self.types.get(&type_name)
    }

    fn extract_code(root_path: &Path, file: &Arc<File>, files: &Files) -> (String, Vec<Arc<File>>) {
        let files: Vec<_> = Self::extract_file_paths(root_path, file, files)
            .into_iter()
            .map(|path| files.get(&path).clone())
            .sorted_unstable_by_key(|current_file| {
                (current_file.path != file.path, file.path.clone())
            })
            .collect();
        let code = files
            .iter()
            .map(|file| {
                file.code
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
        (code, files)
    }

    fn extract_file_paths(root_path: &Path, file: &Arc<File>, files: &Files) -> Vec<PathBuf> {
        let mut paths: FxHashSet<_> = iter::once(file.path.clone()).collect();
        let mut last_path_count = 0;
        while last_path_count < paths.len() {
            last_path_count = paths.len();
            for path in paths.clone() {
                let import_directives = crate::directives::find_all_by_kind(
                    &files.get(&path).directives,
                    DirectiveKind::Import,
                );
                for directive in import_directives {
                    paths.insert(root_path.join(directive.import_path()));
                }
            }
        }
        paths.into_iter().collect()
    }
}
