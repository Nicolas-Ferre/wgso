use crate::directives::{Directive, DirectiveKind};
use crate::program::file::Files;
use crate::program::module::{Module, Modules};
use crate::Error;
use fxhash::FxHashSet;
use std::mem;
use std::ops::Range;
use wgpu::Limits;
use wgso_parser::{ParsingError, Token};

const CALL_DIRECTIVE_KINDS: &[DirectiveKind] =
    &[DirectiveKind::Init, DirectiveKind::Run, DirectiveKind::Draw];

impl Directive {
    pub(crate) fn priority(&self) -> i32 {
        assert!(CALL_DIRECTIVE_KINDS.contains(&self.kind()));
        if let Some(priority) = self.find_all_by_label("priority").next() {
            Self::convert_to_integer(priority)
        } else {
            0
        }
    }

    pub(crate) fn vertex_buffer(&self) -> DirectiveArgValue {
        assert_eq!(self.kind(), DirectiveKind::Draw);
        let mut current_var = None;
        let mut current_fields = vec![];
        for token in &self.tokens {
            match token.label.as_deref() {
                Some("vertex_buffer_var") => current_var = Some(token.clone()),
                Some("vertex_buffer_field") => current_fields.push(token.clone()),
                _ => {}
            }
        }
        if let Some(var) = current_var.take() {
            DirectiveArgValue::new(var, mem::take(&mut current_fields))
        } else {
            unreachable!("internal error: directive arguments should be validated");
        }
    }

    pub(crate) fn args(&self) -> Vec<DirectiveArg> {
        assert!(CALL_DIRECTIVE_KINDS.contains(&self.kind()));
        let tokens = self.arg_tokens();
        let mut args = vec![];
        for (index, token) in tokens.iter().enumerate() {
            if token.label.as_deref() == Some("arg_name") {
                args.push(Self::extract_arg(&tokens, index, token));
            }
        }
        args
    }

    pub(crate) fn arg(&self, arg_name: &str) -> DirectiveArg {
        assert!(CALL_DIRECTIVE_KINDS.contains(&self.kind()));
        let tokens = self.arg_tokens();
        for (index, token) in tokens.iter().enumerate() {
            if token.label.as_deref() == Some("arg_name") && token.slice == arg_name {
                return Self::extract_arg(&tokens, index, token);
            }
        }
        unreachable!("internal error: directive arguments should be validated");
    }

    fn arg_tokens(&self) -> Vec<&Token> {
        self.tokens
            .iter()
            .filter(|token| {
                token.label.as_deref() == Some("arg_name")
                    || token.label.as_deref() == Some("arg_var")
                    || token.label.as_deref() == Some("arg_field")
            })
            .collect::<Vec<_>>()
    }

    fn extract_arg(tokens: &[&Token], index: usize, token: &Token) -> DirectiveArg {
        DirectiveArg {
            name: (*token).clone(),
            value: DirectiveArgValue::new(
                tokens[index + 1].clone(),
                tokens[index + 2..]
                    .iter()
                    .take_while(|token| token.label.as_deref() != Some("arg_name"))
                    .copied()
                    .cloned()
                    .collect(),
            ),
        }
    }
}

pub(crate) fn check(directives: &[Directive], errors: &mut Vec<Error>) {
    for directive in directives {
        if let Some(shader_def_kind) = shader_def_kind(directive.kind()) {
            check_shader_name(directive, directives, shader_def_kind, errors);
        }
    }
}

pub(crate) fn check_args(files: &Files, modules: &Modules, errors: &mut Vec<Error>) {
    for directive in files.run_directives() {
        let shader_module = shader_module(directive, modules);
        check_arg_names(directive, shader_module, errors);
        check_arg_value(modules, directive, shader_module, errors);
    }
    for directive in files.draw_directives() {
        let shader_module = shader_module(directive, modules);
        check_arg_names(directive, shader_module, errors);
        check_arg_value(modules, directive, shader_module, errors);
        let shader_name = &directive.shader_name().slice;
        if let Some((shader_directive, module)) = modules.render_shaders.get(shader_name) {
            check_vertex_buffer(modules, directive, shader_directive, module, errors);
        }
    }
}

fn check_shader_name(
    directive: &Directive,
    directives: &[Directive],
    shader_def_kind: DirectiveKind,
    errors: &mut Vec<Error>,
) {
    let shader_name = directive.shader_name();
    let is_shader_found = directives.iter().any(|directive| {
        directive.kind() == shader_def_kind && directive.shader_name().slice == shader_name.slice
    });
    if !is_shader_found {
        errors.push(Error::DirectiveParsing(ParsingError {
            path: shader_name.path.clone(),
            span: shader_name.span.clone(),
            message: "shader not found".into(),
        }));
    }
}

fn check_arg_names(directive: &Directive, shader_module: &Module, errors: &mut Vec<Error>) {
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

fn check_arg_value(
    modules: &Modules,
    directive: &Directive,
    shader_module: &Module,
    errors: &mut Vec<Error>,
) {
    let offset_alignment = Limits::default().min_uniform_buffer_offset_alignment;
    let shader_name = directive.shader_name();
    for arg in directive.args() {
        let Some(storage_type) = modules.storages.get(&arg.value.var.slice) else {
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
                    arg_type.offset, arg.value.var.slice,
                ),
            }));
        }
    }
}

fn check_vertex_buffer(
    modules: &Modules,
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
    for (name, field_type) in &expected_item_type.fields {
        if !field_type.is_vertex_compatible() {
            errors.push(Error::DirectiveParsing(ParsingError {
                path: vertex_type_name.path.clone(),
                span: vertex_type_name.span.clone(),
                message: format!(
                    "field `{name}` of type `{}` cannot be used as vertex data",
                    field_type.label
                ),
            }));
        }
    }
    let vertex_buffer = draw_directive.vertex_buffer();
    let Some(storage_type) = modules.storages.get(&vertex_buffer.var.slice) else {
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

fn shader_def_kind(call_kind: DirectiveKind) -> Option<DirectiveKind> {
    match call_kind {
        DirectiveKind::Init | DirectiveKind::Run => Some(DirectiveKind::ComputeShader),
        DirectiveKind::Draw => Some(DirectiveKind::RenderShader),
        DirectiveKind::ComputeShader | DirectiveKind::RenderShader | DirectiveKind::Import => None,
    }
}

fn shader_module<'a>(directive: &Directive, modules: &'a Modules) -> &'a Module {
    if directive.kind() == DirectiveKind::Draw {
        &modules.render_shaders[&directive.shader_name().slice].1
    } else {
        &modules.compute_shaders[&directive.shader_name().slice].1
    }
}

#[derive(Debug)]
pub(crate) struct DirectiveArg {
    pub(crate) name: Token,
    pub(crate) value: DirectiveArgValue,
}

#[derive(Debug)]
pub(crate) struct DirectiveArgValue {
    pub(crate) span: Range<usize>,
    pub(crate) var: Token,
    pub(crate) fields: Vec<Token>,
}

impl DirectiveArgValue {
    fn new(var: Token, fields: Vec<Token>) -> Self {
        Self {
            span: var.span.start..fields.last().map_or(var.span.end, |field| field.span.end),
            var,
            fields,
        }
    }
}
