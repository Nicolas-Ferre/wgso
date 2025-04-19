use crate::directive::{Directive, RunDirective};
use crate::file::File;
use crate::storage::Storage;
use crate::wgsl_module::WgslModule;
use crate::Error;
use fxhash::{FxHashMap, FxHashSet};
use itertools::Itertools;
use std::collections::hash_map::Entry;
use std::path::{Path, PathBuf};

/*
// TODO: how to handle type conflict (same name, different disconnected modules) -> use also fields
// TODO: how to parse types efficiently -> recreate type string from naga::Module
// TODO: also use Naga to add bindings and write Module to wgsl string

- Files
    - path -> Arc<File>
        - path
        - code
        - [Directive]

- Modules
    - [Module]
        - [Arc<File>]
        - name -> StorageBinding
        - name -> UniformBinding
        - code

- Storages
    - name -> Arc<Storage>

- ComputeShaders
    - name -> [ComputeShader]
        - Arc<Module>

- run => iterate on Files

- (Storage)
    - Type

- (StorageBinding)
    - Arc<Storage>
    - binding_index

- (UniformBinding)
    - Type
    - binding_index

- (Type) -> impl PartialEq, Eq
    - size
    - fields: {name -> type}
 */

/// A parsed WGSO program.
#[derive(Debug)]
pub struct Program {
    /// The errors found during parsing.
    pub errors: Vec<Error>,
    pub(crate) files: FxHashMap<PathBuf, File>,
    pub(crate) wgsl_modules: Vec<WgslModule>,
    pub(crate) storages: FxHashMap<String, Storage>,
    pub(crate) compute_shaders: FxHashMap<String, WgslModule>,
    pub(crate) runs: Vec<RunDirective>,
}

impl Program {
    pub(crate) fn parse(folder_path: impl AsRef<Path>) -> Self {
        let folder_path = folder_path.as_ref();
        let mut program = Self {
            errors: vec![],
            files: FxHashMap::default(),
            wgsl_modules: vec![],
            storages: FxHashMap::default(),
            compute_shaders: FxHashMap::default(),
            runs: vec![],
        };
        let mut errors = vec![];
        program.files = Self::read_dir(folder_path, &mut errors);
        program.wgsl_modules = program.parse_wgsl_modules(&mut errors);
        program.storages = program.extract_storages(&mut errors);
        program.compute_shaders = program.extract_compute_shaders(&mut errors);
        program.runs = program.extract_runs();
        program.validate_run_params(&mut errors);
        program.errors = errors;
        program
    }

    pub(crate) fn with_sorted_errors(mut self) -> Self {
        self.errors
            .sort_unstable_by_key(|e| e.path().map(Path::to_path_buf));
        self
    }

    fn read_dir(folder_path: &Path, errors: &mut Vec<Error>) -> FxHashMap<PathBuf, File> {
        File::read_dir(folder_path)
            .into_iter()
            .filter_map(|file| match file {
                Ok(file) => Some((file.path.clone(), file)),
                Err(error) => {
                    errors.push(error);
                    None
                }
            })
            .collect()
    }

    fn parse_wgsl_modules(&self, errors: &mut Vec<Error>) -> Vec<WgslModule> {
        self.files
            .values()
            .filter_map(|file| WgslModule::parse(file, errors))
            .sorted_unstable_by_key(|file| file.path.clone())
            .collect()
    }

    fn extract_storages(&self, errors: &mut Vec<Error>) -> FxHashMap<String, Storage> {
        let mut storages = FxHashMap::default();
        for wgsl in self
            .wgsl_modules
            .iter()
            .sorted_unstable_by_key(|wgsl| &wgsl.path)
        {
            for storage in Storage::extract(wgsl) {
                match storages.entry(storage.name.clone()) {
                    Entry::Vacant(entry) => {
                        entry.insert(storage);
                    }
                    Entry::Occupied(existing) => {
                        errors.push(Error::StorageConflict(
                            existing.get().clone(),
                            storage.clone(),
                        ));
                    }
                }
            }
        }
        storages
    }

    fn extract_compute_shaders(&self, errors: &mut Vec<Error>) -> FxHashMap<String, WgslModule> {
        let mut modules = FxHashMap::default();
        let mut directives = FxHashMap::default();
        for module in &self.wgsl_modules {
            for directive in &module.directives {
                if let Directive::ComputeShader(directive) = directive {
                    match directives.entry(directive.name.clone()) {
                        Entry::Vacant(entry) => {
                            entry.insert(directive.clone());
                            modules.insert(directive.name.clone(), module.clone());
                        }
                        Entry::Occupied(existing) => {
                            errors.push(Error::ShaderConflict(
                                existing.get().clone(),
                                directive.clone(),
                            ));
                        }
                    }
                }
            }
        }
        modules
    }

    fn extract_runs(&self) -> Vec<RunDirective> {
        self.wgsl_modules
            .iter()
            .flat_map(|wgsl| &wgsl.directives)
            .filter_map(|directive| {
                if let Directive::Run(directive) = directive {
                    Some(directive.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn validate_run_params(&self, errors: &mut Vec<Error>) {
        for run in &self.runs {
            let shader = &self.compute_shaders[&run.name];
            let expected_uniform_names: FxHashSet<_> = shader.uniform_bindings.keys().collect();
            let actual_uniform_names: FxHashSet<_> = run.params.keys().collect();
            for &missing_param in expected_uniform_names.difference(&actual_uniform_names) {
                errors.push(Error::DirectiveParsing(
                    run.path.clone(),
                    run.span.clone(),
                    format!("missing uniform argument `{missing_param}`"),
                ));
            }
            for &unknown_param in actual_uniform_names.difference(&expected_uniform_names) {
                errors.push(Error::DirectiveParsing(
                    run.path.clone(),
                    run.params[unknown_param].name_span.clone(),
                    format!(
                        "no uniform variable `{unknown_param}` in shader `{}`",
                        run.name
                    ),
                ));
            }
            for (name, param) in &run.params {
                if let Some(storage) = self.storages.get(&param.value) {
                    if let Some(uniform) = shader.uniform_bindings.get(name) {
                        if uniform.type_ != storage.type_ {
                            errors.push(Error::DirectiveParsing(
                                run.path.clone(),
                                param.value_span.clone(),
                                format!(
                                    "found buffer with type `{}`, expected uniform type `{}`",
                                    storage.type_, uniform.type_
                                ),
                            ));
                        }
                    }
                } else {
                    errors.push(Error::DirectiveParsing(
                        run.path.clone(),
                        param.value_span.clone(),
                        format!("unknown storage variable `{}`", param.value),
                    ));
                }
            }
        }
    }
}
