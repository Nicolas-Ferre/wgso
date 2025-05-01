use crate::directive::{Directive, DirectiveKind};
use crate::file::Files;
use crate::module::{Module, Modules};
use crate::type_::Type;
use crate::Error;
use fxhash::{FxHashMap, FxHashSet};
use itertools::Itertools;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use wgpu::Limits;
use wgso_parser::ParsingError;

#[derive(Debug, Default)]
pub(crate) struct Resources {
    pub(crate) storages: FxHashMap<String, Arc<Type>>,
    pub(crate) compute_shaders: FxHashMap<String, (Directive, Arc<Module>)>,
    pub(crate) render_shaders: FxHashMap<String, (Directive, Arc<Module>)>,
    pub(crate) runs: Vec<Directive>, // TODO: separate run and init
    pub(crate) draws: Vec<Directive>,
}

impl Resources {
    pub(crate) fn new(files: &Files, modules: &Modules, errors: &mut Vec<Error>) -> Self {
        let resources = Self {
            storages: Self::storages(modules, errors),
            compute_shaders: Self::shaders(DirectiveKind::ComputeShader, modules),
            render_shaders: Self::shaders(DirectiveKind::RenderShader, modules),
            runs: Self::runs(files),
            draws: Self::draws(files),
        };
        for directive in &resources.runs {
            resources.validate_shader_call(directive, errors);
        }
        for directive in &resources.draws {
            resources.validate_shader_call(directive, errors);
            let shader_name = &directive.shader_name().slice;
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

    fn shaders(
        kind: DirectiveKind,
        modules: &Modules,
    ) -> FxHashMap<String, (Directive, Arc<Module>)> {
        modules
            .iter()
            .flat_map(|module| {
                crate::directive::find_all_by_kind(&module.files[0].directives, kind)
                    .map(|directive| (directive.clone(), module.clone()))
            })
            .map(|(directive, module)| (directive.shader_name().slice.clone(), (directive, module)))
            .collect()
    }

    fn runs(files: &Files) -> Vec<Directive> {
        let init = files.iter().flat_map(|file| {
            crate::directive::find_all_by_kind(&file.directives, DirectiveKind::Init)
                .map(|directive| (directive.clone(), true))
        });
        let runs = files.iter().flat_map(|file| {
            crate::directive::find_all_by_kind(&file.directives, DirectiveKind::Run)
                .map(|directive| (directive.clone(), false))
        });
        init.chain(runs)
            .sorted_by_key(|(directive, is_init)| {
                (
                    !is_init,
                    directive.priority(),
                    directive.shader_name().slice.clone(),
                )
            })
            .map(|(directive, _)| directive)
            .collect()
    }

    fn draws(files: &Files) -> Vec<Directive> {
        files
            .iter()
            .flat_map(|file| {
                crate::directive::find_all_by_kind(&file.directives, DirectiveKind::Draw).cloned()
            })
            .sorted_by_key(|directive| {
                (directive.priority(), directive.shader_name().slice.clone())
            })
            .collect()
    }

    fn validate_shader_call(&self, directive: &Directive, errors: &mut Vec<Error>) {
        let shader_module = if directive.kind() == DirectiveKind::Draw {
            &self.render_shaders[&directive.shader_name().slice].1
        } else {
            &self.compute_shaders[&directive.shader_name().slice].1
        };
        Self::validate_run_arg_names(directive, shader_module, errors);
        self.validate_run_arg_value(directive, shader_module, errors);
    }

    fn validate_run_arg_names(
        directive: &Directive,
        shader_module: &Module,
        errors: &mut Vec<Error>,
    ) {
        let shader_name = directive.shader_name();
        let args = directive.args();
        let shader_uniform_names: FxHashSet<_> = shader_module.uniform_names().collect();
        let run_arg_names: FxHashSet<_> = args.iter().map(|arg| &arg.name.slice).collect();
        for &missing_arg in shader_uniform_names.difference(&run_arg_names) {
            errors.push(Error::DirectiveParsing(ParsingError {
                path: shader_name.path.clone(),
                span: shader_name.span.clone(),
                message: format!("missing uniform argument `{missing_arg}`"),
            }));
        }
        for &unknown_arg in run_arg_names.difference(&shader_uniform_names) {
            errors.push(Error::DirectiveParsing(ParsingError {
                path: shader_name.path.clone(),
                span: directive.arg(unknown_arg).name.span.clone(),
                message: format!(
                    "no uniform variable `{unknown_arg}` in shader `{}`",
                    shader_name.slice
                ),
            }));
        }
        let mut param_names = FxHashSet::default();
        for arg in &args {
            if !param_names.insert(&arg.name.slice) {
                errors.push(Error::DirectiveParsing(ParsingError {
                    path: arg.name.path.clone(),
                    span: arg.name.span.clone(),
                    message: "duplicated parameter".into(),
                }));
            }
        }
    }

    fn validate_run_arg_value(
        &self,
        directive: &Directive,
        shader_module: &Module,
        errors: &mut Vec<Error>,
    ) {
        let offset_alignment = Limits::default().min_uniform_buffer_offset_alignment;
        let shader_name = directive.shader_name();
        for arg in directive.args() {
            let Some(storage_type) = self.storages.get(&arg.value.var.slice) else {
                errors.push(Error::DirectiveParsing(ParsingError {
                    path: shader_name.path.clone(),
                    span: arg.value.span,
                    message: format!("unknown storage variable `{}`", arg.value.var.slice),
                }));
                continue;
            };
            let arg_type = match storage_type.field_ident_type(&arg.value.fields) {
                Ok(arg_type) => arg_type,
                Err(error) => {
                    errors.push(error);
                    continue;
                }
            };
            let Some(uniform) = shader_module.uniform_binding(&arg.name.slice) else {
                continue;
            };
            if &*uniform.type_ != arg_type {
                errors.push(Error::DirectiveParsing(ParsingError {
                    path: shader_name.path.clone(),
                    span: arg.value.span,
                    message: format!(
                        "found argument with type `{}`, expected uniform type `{}`",
                        arg_type.label, uniform.type_.label
                    ),
                }));
            } else if arg_type.offset % offset_alignment != 0 {
                errors.push(Error::DirectiveParsing(ParsingError {
                    path: shader_name.path.clone(),
                    span: arg.value.span,
                    message: format!(
                        "value has an offset of {} bytes in `{}`, which is not a multiple of 256 bytes",
                        arg_type.offset,
                        arg.value.var.slice,
                    ),
                }));
            }
        }
    }

    fn validate_vertex_buffer(
        &self,
        draw_directive: &Directive,
        shader_directive: &Directive,
        shader_module: &Module,
        errors: &mut Vec<Error>,
    ) {
        let vertex_type_name = shader_directive.vertex_type();
        let Some(expected_item_type) = shader_module.type_(&vertex_type_name.slice) else {
            errors.push(Error::DirectiveParsing(ParsingError {
                path: vertex_type_name.path.clone(),
                span: vertex_type_name.span.clone(),
                message: format!("type `{}` not found", vertex_type_name.slice),
            }));
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
                errors.push(Error::DirectiveParsing(ParsingError {
                    path: vertex_type_name.path.clone(),
                    span: vertex_type_name.span.clone(),
                    message: format!(
                        "field `{name}` of type `{}` cannot be used as vertex data",
                        field.label
                    ),
                }));
            }
        }
        let vertex_buffer = draw_directive.vertex_buffer();
        let Some(storage_type) = self.storages.get(&vertex_buffer.var.slice) else {
            errors.push(Error::DirectiveParsing(ParsingError {
                path: vertex_buffer.var.path.clone(),
                span: vertex_buffer.span,
                message: format!("unknown storage variable `{}`", vertex_buffer.var.slice),
            }));
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
            errors.push(Error::DirectiveParsing(ParsingError {
                path: draw_directive.shader_name().path.clone(),
                span: vertex_buffer.span,
                message: "found non-array argument".into(),
            }));
            return;
        };
        if expected_item_type != &**arg_item_type {
            errors.push(Error::DirectiveParsing(ParsingError {
                path: draw_directive.shader_name().path.clone(),
                span: vertex_buffer.span,
                message: format!(
                    "found vertex type `{}`, expected `{}`",
                    arg_item_type.label, expected_item_type.label
                ),
            }));
        }
    }
}
