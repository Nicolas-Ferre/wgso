use crate::directive::common::ShaderArg;
use crate::directive::compute_shader::ComputeShaderDirective;
use crate::directive::draw::DrawDirective;
use crate::directive::render_shader::RenderShaderDirective;
use crate::directive::run::RunDirective;
use crate::directive::token::Ident;
use crate::file::Files;
use crate::module::{Module, Modules};
use crate::type_::Type;
use crate::Error;
use fxhash::{FxHashMap, FxHashSet};
use itertools::Itertools;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use wgpu::Limits;

#[derive(Debug)]
pub(crate) struct Resources {
    pub(crate) storages: FxHashMap<String, Arc<Type>>,
    pub(crate) compute_shaders: FxHashMap<String, (ComputeShaderDirective, Arc<Module>)>,
    pub(crate) render_shaders: FxHashMap<String, (RenderShaderDirective, Arc<Module>)>,
    pub(crate) runs: Vec<RunDirective>,
    pub(crate) draws: Vec<DrawDirective>,
}

impl Resources {
    pub(crate) fn new(files: &Files, modules: &Modules, errors: &mut Vec<Error>) -> Self {
        let resources = Self {
            storages: Self::storages(modules, errors),
            compute_shaders: Self::compute_shaders(modules, errors),
            render_shaders: Self::render_shaders(modules, errors),
            runs: files
                .iter()
                .flat_map(|file| file.directives.runs.iter().cloned())
                .sorted_by_key(|directive| {
                    (
                        !directive.is_init,
                        directive.priority,
                        directive.shader_name.path.clone(),
                    )
                })
                .collect(),
            draws: files
                .iter()
                .flat_map(|file| file.directives.draws.iter().cloned())
                .sorted_by_key(|directive| (directive.priority, directive.shader_name.path.clone()))
                .collect(),
        };
        for directive in &resources.runs {
            resources.validate_shader_call(true, &directive.shader_name, &directive.args, errors);
        }
        for directive in &resources.draws {
            resources.validate_shader_call(false, &directive.shader_name, &directive.args, errors);
            let shader_name = &directive.shader_name.label;
            if let Some((shader_directive, module)) = resources.render_shaders.get(shader_name) {
                resources.validate_vertex_buffer(directive, shader_directive, module, errors);
            }
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
                        let existing = existing.get();
                        if existing.1 != binding.type_ {
                            errors.push(Error::StorageConflict(
                                existing.0.files[0].path.clone(),
                                module.files[0].path.clone(),
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

    fn compute_shaders(
        modules: &Modules,
        errors: &mut Vec<Error>,
    ) -> FxHashMap<String, (ComputeShaderDirective, Arc<Module>)> {
        let mut shaders = FxHashMap::default();
        for module in modules.iter() {
            for directive in &module.files[0].directives.compute_shaders {
                match shaders.entry(directive.shader_name.label.clone()) {
                    Entry::Vacant(entry) => {
                        entry.insert((directive.clone(), module.clone()));
                    }
                    Entry::Occupied(existing) => {
                        errors.push(Error::ShaderConflict(
                            existing.get().0.shader_name.clone(),
                            directive.shader_name.clone(),
                            "compute",
                        ));
                    }
                }
            }
        }
        shaders
    }

    fn render_shaders(
        modules: &Modules,
        errors: &mut Vec<Error>,
    ) -> FxHashMap<String, (RenderShaderDirective, Arc<Module>)> {
        let mut shaders = FxHashMap::default();
        for module in modules.iter() {
            for directive in &module.files[0].directives.render_shaders {
                match shaders.entry(directive.shader_name.label.clone()) {
                    Entry::Vacant(entry) => {
                        entry.insert((directive.clone(), module.clone()));
                    }
                    Entry::Occupied(existing) => {
                        errors.push(Error::ShaderConflict(
                            existing.get().0.shader_name.clone(),
                            directive.shader_name.clone(),
                            "render",
                        ));
                    }
                }
            }
        }
        shaders
    }

    fn validate_shader_call(
        &self,
        is_compute: bool,
        shader_name: &Ident,
        args: &FxHashMap<String, ShaderArg>,
        errors: &mut Vec<Error>,
    ) {
        let Some(shader_module) = self.find_shader_module(is_compute, shader_name, errors) else {
            return;
        };
        Self::validate_run_arg_names(shader_name, args, shader_module, errors);
        self.validate_run_arg_value(shader_name, args, shader_module, errors);
    }

    fn find_shader_module(
        &self,
        is_compute: bool,
        shader_name: &Ident,
        errors: &mut Vec<Error>,
    ) -> Option<&Module> {
        if is_compute {
            if let Some((_, module)) = self.compute_shaders.get(&shader_name.label) {
                Some(module)
            } else {
                errors.push(Error::DirectiveParsing(
                    shader_name.path.clone(),
                    shader_name.span.clone(),
                    "compute shader not found".into(),
                ));
                None
            }
        } else if let Some((_, module)) = self.render_shaders.get(&shader_name.label) {
            Some(module)
        } else {
            errors.push(Error::DirectiveParsing(
                shader_name.path.clone(),
                shader_name.span.clone(),
                "render shader not found".into(),
            ));
            None
        }
    }

    fn validate_run_arg_names(
        shader_name: &Ident,
        args: &FxHashMap<String, ShaderArg>,
        shader_module: &Module,
        errors: &mut Vec<Error>,
    ) {
        let shader_uniform_names: FxHashSet<_> = shader_module.uniform_names().collect();
        let run_arg_names: FxHashSet<_> = args.keys().collect();
        for &missing_arg in shader_uniform_names.difference(&run_arg_names) {
            errors.push(Error::DirectiveParsing(
                shader_name.path.clone(),
                shader_name.span.clone(),
                format!("missing uniform argument `{missing_arg}`"),
            ));
        }
        for &unknown_arg in run_arg_names.difference(&shader_uniform_names) {
            errors.push(Error::DirectiveParsing(
                shader_name.path.clone(),
                args[unknown_arg].name.span.clone(),
                format!(
                    "no uniform variable `{unknown_arg}` in shader `{}`",
                    shader_name.label
                ),
            ));
        }
    }

    fn validate_run_arg_value(
        &self,
        shader_name: &Ident,
        args: &FxHashMap<String, ShaderArg>,
        shader_module: &Module,
        errors: &mut Vec<Error>,
    ) {
        let offset_alignment = Limits::default().min_uniform_buffer_offset_alignment;
        for (name, arg) in args {
            let Some(storage_type) = self.storages.get(&arg.value.buffer_name.label) else {
                errors.push(Error::DirectiveParsing(
                    shader_name.path.clone(),
                    arg.value.span(),
                    format!("unknown storage variable `{}`", arg.value.buffer_name.label),
                ));
                continue;
            };
            let arg_type = match storage_type.field_ident_type(&arg.value.fields) {
                Ok(arg_type) => arg_type,
                Err(error) => {
                    errors.push(error);
                    continue;
                }
            };
            let Some(uniform) = shader_module.uniform_binding(name) else {
                continue;
            };
            if &*uniform.type_ != arg_type {
                errors.push(Error::DirectiveParsing(
                    shader_name.path.clone(),
                    arg.value.span(),
                    format!(
                        "found argument with type `{}`, expected uniform type `{}`",
                        arg_type.label, uniform.type_.label
                    ),
                ));
            } else if arg_type.offset % offset_alignment != 0 {
                errors.push(Error::DirectiveParsing(
                    shader_name.path.clone(),
                    arg.value.span(),
                    format!(
                        "value has an offset of {} bytes in `{}`, which is not a multiple of 256 bytes",
                        arg_type.offset,
                        arg.value.buffer_name.label,
                    ),
                ));
            }
        }
    }

    fn validate_vertex_buffer(
        &self,
        draw_directive: &DrawDirective,
        shader_directive: &RenderShaderDirective,
        shader_module: &Module,
        errors: &mut Vec<Error>,
    ) {
        let Some(expected_item_type) =
            shader_module.type_(&shader_directive.vertex_type_name.label)
        else {
            errors.push(Error::DirectiveParsing(
                shader_directive.vertex_type_name.path.clone(),
                shader_directive.vertex_type_name.span.clone(),
                format!(
                    "type `{}` not found",
                    shader_directive.vertex_type_name.label
                ),
            ));
            return;
        };
        for (name, field) in &expected_item_type.fields {
            if field.label != "i32"
                && field.label != "u32"
                && field.label != "f32"
                && !field.label.starts_with("vec2<")
                && !field.label.starts_with("vec3<")
                && !field.label.starts_with("vec4<")
            {
                errors.push(Error::DirectiveParsing(
                    shader_directive.vertex_type_name.path.clone(),
                    shader_directive.vertex_type_name.span.clone(),
                    format!(
                        "field `{name}` of type `{}` cannot be used as vertex data",
                        field.label
                    ),
                ));
            }
        }
        let vertex_buffer = &draw_directive.vertex_buffer;
        let Some(storage_type) = self.storages.get(&vertex_buffer.buffer_name.label) else {
            errors.push(Error::DirectiveParsing(
                draw_directive.shader_name.path.clone(),
                vertex_buffer.span(),
                format!(
                    "unknown storage variable `{}`",
                    vertex_buffer.buffer_name.label
                ),
            ));
            return;
        };
        let arg_type = match storage_type.field_ident_type(&vertex_buffer.fields) {
            Ok(arg_type) => arg_type,
            Err(error) => {
                errors.push(error);
                return;
            }
        };
        let Some((arg_item_type, _)) = arg_type.array_params.as_ref() else {
            errors.push(Error::DirectiveParsing(
                draw_directive.shader_name.path.clone(),
                vertex_buffer.span(),
                "found non-array argument".into(),
            ));
            return;
        };
        if expected_item_type != &**arg_item_type {
            errors.push(Error::DirectiveParsing(
                draw_directive.shader_name.path.clone(),
                vertex_buffer.span(),
                format!(
                    "found vertex type `{}`, expected `{}`",
                    arg_item_type.label, expected_item_type.label
                ),
            ));
        }
    }
}
