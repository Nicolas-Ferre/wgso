use crate::directive::{Directive, RunDirective};
use crate::file::File;
use crate::storage::Storage;
use crate::wgsl::WgslModule;
use crate::Error;
use fxhash::FxHashMap;
use itertools::Itertools;
use std::collections::hash_map::Entry;
use std::path::{Path, PathBuf};

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
}
