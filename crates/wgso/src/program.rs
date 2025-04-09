use crate::file::File;
use crate::storage::Storage;
use crate::wgsl::Wgsl;
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
    pub(crate) wgsl: Vec<Wgsl>,
    pub(crate) storages: FxHashMap<String, Storage>,
}

impl Program {
    /// Parses a WGSO program directory.
    ///
    /// # Errors
    ///
    /// An error is returned if the parsing has failed.
    pub(crate) fn parse(folder_path: impl AsRef<Path>) -> Self {
        let folder_path = folder_path.as_ref();
        let mut program = Self {
            errors: vec![],
            files: FxHashMap::default(),
            wgsl: vec![],
            storages: FxHashMap::default(),
        };
        let mut errors = vec![];
        program.files = Self::read_dir(folder_path, &mut errors);
        program.wgsl = program.parse_wgsl(&mut errors);
        program.storages = program.extract_storages(&mut errors);
        program.errors = errors;
        program
            .errors
            .sort_unstable_by_key(|e| e.path().to_path_buf());
        program
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

    fn parse_wgsl(&self, errors: &mut Vec<Error>) -> Vec<Wgsl> {
        self.files
            .values()
            .filter_map(|file| Wgsl::parse(file, errors))
            .collect()
    }

    fn extract_storages(&self, errors: &mut Vec<Error>) -> FxHashMap<String, Storage> {
        let mut storages = FxHashMap::default();
        for wgsl in self.wgsl.iter().sorted_unstable_by_key(|wgsl| &wgsl.path) {
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
}
