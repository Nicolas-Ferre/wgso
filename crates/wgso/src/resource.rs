use crate::directive::DirectiveKind;
use crate::file::Files;
use crate::module::{Module, Modules};
use crate::type_::Type;
use crate::Error;
use fxhash::{FxHashMap, FxHashSet};
use itertools::Itertools;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use wgpu::Limits;
use wgso_parser::{ParsingError, Token};

#[derive(Debug)]
pub(crate) struct Resources {
    pub(crate) storages: FxHashMap<String, Arc<Type>>,
    pub(crate) compute_shaders: FxHashMap<String, (Vec<Token>, Arc<Module>)>,
    pub(crate) render_shaders: FxHashMap<String, (Vec<Token>, Arc<Module>)>,
    pub(crate) runs: Vec<Vec<Token>>, // TODO: separate run and init
    pub(crate) draws: Vec<Vec<Token>>,
}

impl Resources {
    pub(crate) fn new(files: &Files, modules: &Modules, errors: &mut Vec<Error>) -> Self {
        let resources = Self {
            storages: Self::storages(modules, errors),
            compute_shaders: Self::compute_shaders(modules, errors),
            render_shaders: Self::render_shaders(modules, errors),
            runs: Self::runs(files),
            draws: Self::draws(files),
        };
        for directive in &resources.runs {
            resources.validate_shader_call(directive, errors);
        }
        for directive in &resources.draws {
            resources.validate_shader_call(directive, errors);
            let shader_name = &crate::directive::shader_name(directive).slice;
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
    ) -> FxHashMap<String, (Vec<Token>, Arc<Module>)> {
        let mut shaders = FxHashMap::default();
        for module in modules.iter() {
            let compute_shader_directives = crate::directive::find_all_by_kind(
                &module.files[0].directives,
                DirectiveKind::ComputeShader,
            );
            for directive in compute_shader_directives {
                let shader_name = crate::directive::shader_name(directive);
                match shaders.entry(shader_name.slice.clone()) {
                    Entry::Vacant(entry) => {
                        entry.insert((directive.to_vec(), module.clone()));
                    }
                    Entry::Occupied(existing) => {
                        errors.push(Error::ShaderConflict(
                            crate::directive::shader_name(&existing.get().0).clone(),
                            shader_name.clone(),
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
    ) -> FxHashMap<String, (Vec<Token>, Arc<Module>)> {
        let mut shaders = FxHashMap::default();
        for module in modules.iter() {
            let render_shader_directives = crate::directive::find_all_by_kind(
                &module.files[0].directives,
                DirectiveKind::RenderShader,
            );
            for directive in render_shader_directives {
                let shader_name = crate::directive::shader_name(directive);
                match shaders.entry(shader_name.slice.clone()) {
                    Entry::Vacant(entry) => {
                        entry.insert((directive.to_vec(), module.clone()));
                    }
                    Entry::Occupied(existing) => {
                        errors.push(Error::ShaderConflict(
                            crate::directive::shader_name(&existing.get().0).clone(),
                            shader_name.clone(),
                            "render",
                        ));
                    }
                }
            }
        }
        shaders
    }

    fn runs(files: &Files) -> Vec<Vec<Token>> {
        let init = files.iter().flat_map(|file| {
            crate::directive::find_all_by_kind(&file.directives, DirectiveKind::Init)
                .map(|directive| (directive.to_vec(), true))
        });
        let runs = files.iter().flat_map(|file| {
            crate::directive::find_all_by_kind(&file.directives, DirectiveKind::Run)
                .map(|directive| (directive.to_vec(), false))
        });
        init.chain(runs)
            .sorted_by_key(|(directive, is_init)| {
                (
                    !is_init,
                    crate::directive::priority(directive),
                    crate::directive::shader_name(directive).slice.clone(),
                )
            })
            .map(|(directive, _)| directive)
            .collect()
    }

    fn draws(files: &Files) -> Vec<Vec<Token>> {
        files
            .iter()
            .flat_map(|file| {
                crate::directive::find_all_by_kind(&file.directives, DirectiveKind::Draw)
                    .map(<[Token]>::to_vec)
            })
            .sorted_by_key(|directive| {
                (
                    crate::directive::priority(directive),
                    crate::directive::shader_name(directive).slice.clone(),
                )
            })
            .collect()
    }

    fn validate_shader_call(&self, directive: &[Token], errors: &mut Vec<Error>) {
        let Some(shader_module) = self.find_shader_module(directive, errors) else {
            return;
        };
        Self::validate_run_arg_names(directive, shader_module, errors);
        self.validate_run_arg_value(directive, shader_module, errors);
    }

    fn find_shader_module(&self, directive: &[Token], errors: &mut Vec<Error>) -> Option<&Module> {
        let shader_name = crate::directive::shader_name(directive);
        let shader = if crate::directive::kind(directive) == DirectiveKind::Draw {
            self.render_shaders.get(&shader_name.slice)
        } else {
            self.compute_shaders.get(&shader_name.slice)
        };
        if let Some((_, module)) = shader {
            Some(module)
        } else {
            errors.push(Error::DirectiveParsing(ParsingError {
                path: shader_name.path.clone(),
                span: shader_name.span.clone(),
                message: "shader not found".into(),
            }));
            None
        }
    }

    fn validate_run_arg_names(
        directive: &[Token],
        shader_module: &Module,
        errors: &mut Vec<Error>,
    ) {
        let shader_name = crate::directive::shader_name(directive);
        let args = crate::directive::args(directive);
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
                span: crate::directive::arg(directive, unknown_arg)
                    .name
                    .span
                    .clone(),
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
        directive: &[Token],
        shader_module: &Module,
        errors: &mut Vec<Error>,
    ) {
        let offset_alignment = Limits::default().min_uniform_buffer_offset_alignment;
        let shader_name = crate::directive::shader_name(directive);
        for arg in crate::directive::args(directive) {
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
        draw_directive: &[Token],
        shader_directive: &[Token],
        shader_module: &Module,
        errors: &mut Vec<Error>,
    ) {
        let vertex_type_name = crate::directive::vertex_type(shader_directive);
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
        let vertex_buffer = crate::directive::vertex_buffer(draw_directive);
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
                path: crate::directive::shader_name(draw_directive).path.clone(),
                span: vertex_buffer.span,
                message: "found non-array argument".into(),
            }));
            return;
        };
        if expected_item_type != &**arg_item_type {
            errors.push(Error::DirectiveParsing(ParsingError {
                path: crate::directive::shader_name(draw_directive).path.clone(),
                span: vertex_buffer.span,
                message: format!(
                    "found vertex type `{}`, expected `{}`",
                    arg_item_type.label, expected_item_type.label
                ),
            }));
        }
    }
}
