use crate::directive::{imports, shader_calls, shader_defs};
use crate::file::Files;
use crate::module::Modules;
use crate::resource::Resources;
use crate::Error;
use itertools::Itertools;
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
        let directive_rules = crate::directive::load_rules();
        let files = Files::new(root_path, &directive_rules, &mut errors);
        let directives: Vec<_> = files
            .iter()
            .flat_map(|file| file.directives.iter().cloned())
            .collect();
        imports::check(&directives, &files, root_path, &mut errors);
        shader_defs::check(&directives, &mut errors);
        shader_calls::check(&directives, &mut errors);
        if !errors.is_empty() {
            return Self {
                errors,
                files,
                resources: Resources::default(),
            };
        }
        let modules = Modules::new(root_path, &files, &mut errors);
        let resources = Resources::new(&files, &modules, &mut errors);
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
