use crate::directive::run::RunDirective;
use crate::directive::shader::ShaderDirective;
use crate::module::{Module, Modules};
use crate::type_::Type;
use crate::Error;
use fxhash::{FxHashMap, FxHashSet};
use std::collections::hash_map::Entry;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct Resources {
    pub(crate) storages: FxHashMap<String, Arc<Type>>,
    pub(crate) compute_shaders: FxHashMap<String, (ShaderDirective, Arc<Module>)>,
    pub(crate) runs: Vec<(RunDirective, Arc<Module>)>,
}

impl Resources {
    pub(crate) fn new(modules: &Modules, errors: &mut Vec<Error>) -> Self {
        let resources = Self {
            storages: Self::storages(modules, errors),
            compute_shaders: Self::compute_shaders(modules, errors),
            runs: modules
                .iter()
                .flat_map(|module| {
                    module
                        .file
                        .directives
                        .runs()
                        .map(|directive| (directive.clone(), module.clone()))
                })
                .collect(),
        };
        for (directive, module) in &resources.runs {
            resources.validate_run(directive, module, errors);
        }
        resources
    }

    fn storages(modules: &Modules, errors: &mut Vec<Error>) -> FxHashMap<String, Arc<Type>> {
        let mut storages = FxHashMap::default();
        for module in modules.iter() {
            for (name, binding) in module.storage_bindings() {
                match storages.entry(name.clone()) {
                    Entry::Vacant(entry) => {
                        entry.insert((module.clone(), binding.type_.clone()));
                    }
                    Entry::Occupied(existing) => {
                        errors.push(Error::StorageConflict(
                            existing.get().0.file.path.clone(),
                            module.file.path.clone(),
                            name.clone(),
                        ));
                    }
                }
            }
        }
        storages
            .into_iter()
            .map(|(name, (_, type_))| (name, type_))
            .collect()
    }

    fn compute_shaders(
        modules: &Modules,
        errors: &mut Vec<Error>,
    ) -> FxHashMap<String, (ShaderDirective, Arc<Module>)> {
        let mut shaders = FxHashMap::default();
        for module in modules.iter() {
            for directive in module.file.directives.compute_shaders() {
                match shaders.entry(directive.name.label.clone()) {
                    Entry::Vacant(entry) => {
                        entry.insert((directive.clone(), module.clone()));
                    }
                    Entry::Occupied(existing) => {
                        errors.push(Error::ShaderConflict(
                            existing.get().0.name.clone(),
                            directive.name.clone(),
                        ));
                    }
                }
            }
        }
        shaders
    }

    fn validate_run(&self, directive: &RunDirective, module: &Module, errors: &mut Vec<Error>) {
        let Some(shader_module) = self.find_shader_module(directive, module, errors) else {
            return;
        };
        Self::validate_run_arg_names(directive, module, errors, shader_module);
        self.validate_run_arg_value(directive, module, errors, shader_module);
    }

    fn find_shader_module(
        &self,
        directive: &RunDirective,
        module: &Module,
        errors: &mut Vec<Error>,
    ) -> Option<&Arc<Module>> {
        if let Some((_, module)) = self.compute_shaders.get(&directive.name.label) {
            Some(module)
        } else {
            errors.push(Error::DirectiveParsing(
                module.file.path.clone(),
                directive.name.span.clone(),
                "shader not found".into(),
            ));
            None
        }
    }

    fn validate_run_arg_names(
        directive: &RunDirective,
        module: &Module,
        errors: &mut Vec<Error>,
        shader_module: &Arc<Module>,
    ) {
        let shader_uniform_names: FxHashSet<_> = shader_module.uniform_names().collect();
        let run_arg_names: FxHashSet<_> = directive.args.keys().collect();
        for &missing_arg in shader_uniform_names.difference(&run_arg_names) {
            errors.push(Error::DirectiveParsing(
                module.file.path.clone(),
                directive.name.span.clone(),
                format!("missing uniform argument `{missing_arg}`"),
            ));
        }
        for &unknown_arg in run_arg_names.difference(&shader_uniform_names) {
            errors.push(Error::DirectiveParsing(
                module.file.path.clone(),
                directive.args[unknown_arg].name.span.clone(),
                format!(
                    "no uniform variable `{unknown_arg}` in shader `{}`",
                    directive.name.label
                ),
            ));
        }
    }

    fn validate_run_arg_value(
        &self,
        directive: &RunDirective,
        module: &Module,
        errors: &mut Vec<Error>,
        shader_module: &Arc<Module>,
    ) {
        for (name, arg) in &directive.args {
            if let Some(storage_type) = self.storages.get(&arg.value.label) {
                if let Some(uniform) = shader_module.uniform_binding(name) {
                    if &uniform.type_ != storage_type {
                        errors.push(Error::DirectiveParsing(
                            module.file.path.clone(),
                            arg.value.span.clone(),
                            format!(
                                "found buffer with type `{}`, expected uniform type `{}`",
                                storage_type.label, uniform.type_.label
                            ),
                        ));
                    }
                }
            } else {
                errors.push(Error::DirectiveParsing(
                    module.file.path.clone(),
                    arg.value.span.clone(),
                    format!("unknown storage variable `{}`", arg.value.label),
                ));
            }
        }
    }
}
