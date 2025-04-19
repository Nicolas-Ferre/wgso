use crate::file::Files;
use crate::module::Modules;
use crate::resource::Resources;
use crate::Error;
use std::path::Path;

/// A parsed WGSO program.
#[derive(Debug)]
pub struct Program {
    /// The errors found during parsing.
    pub errors: Vec<Error>,
    pub(crate) files: Files,
    pub(crate) resources: Resources,
}

impl Program {
    pub(crate) fn parse(folder_path: impl AsRef<Path>) -> Self {
        let folder_path = folder_path.as_ref();
        let mut errors = vec![];
        let files = Files::new(folder_path, &mut errors);
        let modules = Modules::new(&files, &mut errors);
        let resources = Resources::new(&modules, &mut errors);
        Self {
            errors,
            files,
            resources,
        }
    }

    pub(crate) fn with_sorted_errors(mut self) -> Self {
        self.errors
            .sort_unstable_by_key(|e| e.path().map(Path::to_path_buf));
        self
    }
}
