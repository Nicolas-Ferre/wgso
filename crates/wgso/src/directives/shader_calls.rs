use crate::directives::{Directive, DirectiveKind};
use crate::program::file::Files;
use crate::program::module::{Module, Modules};
use crate::Error;
use fxhash::FxHashSet;
use itertools::Itertools;
use std::mem;
use std::ops::Range;
use std::path::{Path, PathBuf};
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
        self.buffer("vertex_buffer_var", "vertex_buffer_field")
    }

    pub(crate) fn instance_buffer(&self) -> DirectiveArgValue {
        self.buffer("instance_buffer_var", "instance_buffer_field")
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
        tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| {
                token.label.as_deref() == Some("arg_name") && token.slice == arg_name
            })
            .map(|(index, token)| Self::extract_arg(&tokens, index, token))
            .next()
            .expect("internal error: directive arguments should be validated")
    }

    pub(crate) fn item_ident(&self, root_path: &Path) -> (PathBuf, String) {
        let path = if self.find_all_by_label("path_segment").count() == 1 {
            self.path().into()
        } else {
            self.segment_path(root_path, false)
        };
        let name = self
            .find_all_by_label("path_segment")
            .last()
            .expect("internal error: cannot find shader call item name")
            .slice
            .clone();
        (path, name)
    }

    pub(crate) fn item_slice(&self) -> String {
        self.find_all_by_label("path_segment")
            .map(|segment| &segment.slice)
            .join(".")
    }

    pub(crate) fn item_span(&self) -> Range<usize> {
        let first = self
            .find_all_by_label("path_segment")
            .next()
            .expect("internal error: cannot find shader call item name");
        let last = self
            .find_all_by_label("path_segment")
            .last()
            .expect("internal error: cannot find shader call item name");
        first.span.start..last.span.end
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

    fn buffer(&self, var_label: &str, field_label: &str) -> DirectiveArgValue {
        assert_eq!(self.kind(), DirectiveKind::Draw);
        let mut current_var = None;
        let mut current_fields = vec![];
        for token in &self.tokens {
            if token.label.as_deref() == Some(var_label) {
                current_var = Some(token.clone());
            } else if token.label.as_deref() == Some(field_label) {
                current_fields.push(token.clone());
            }
        }
        if let Some(var) = current_var.take() {
            DirectiveArgValue::new(var, mem::take(&mut current_fields))
        } else {
            unreachable!("internal error: directive arguments should be validated");
        }
    }
}

pub(crate) fn check(directives: &[Directive], root_path: &Path, errors: &mut Vec<Error>) {
    for directive in directives {
        if let Some(shader_def_kind) = shader_def_kind(directive.kind()) {
            check_shader_name(root_path, directive, directives, shader_def_kind, errors);
        }
    }
}

pub(crate) fn check_args(
    root_path: &Path,
    files: &Files,
    modules: &Modules,
    errors: &mut Vec<Error>,
) {
    for directive in files.run_directives() {
        let shader_module = shader_module(root_path, directive, modules);
        check_arg_names(directive, shader_module, errors);
        check_arg_value(modules, directive, shader_module, errors);
    }
    for directive in files.draw_directives() {
        let shader_module = shader_module(root_path, directive, modules);
        check_arg_names(directive, shader_module, errors);
        check_arg_value(modules, directive, shader_module, errors);
        let shader_ident = directive.item_ident(root_path);
        if let Some((shader_directive, module)) = modules.render.get(&shader_ident) {
            check_buffer(true, modules, directive, shader_directive, module, errors);
            check_buffer(false, modules, directive, shader_directive, module, errors);
        }
    }
}

fn check_shader_name(
    root_path: &Path,
    directive: &Directive,
    directives: &[Directive],
    shader_def_kind: DirectiveKind,
    errors: &mut Vec<Error>,
) {
    let item_ident = directive.item_ident(root_path);
    let is_shader_found = directives.iter().any(|directive| {
        directive.kind() == shader_def_kind
            && directive.path() == item_ident.0
            && directive.shader_name().slice == item_ident.1
    });
    if !is_shader_found {
        errors.push(Error::DirectiveParsing(ParsingError {
            path: directive.path().into(),
            span: directive.item_span(),
            message: "shader not found".into(),
        }));
    }
}

fn check_arg_names(directive: &Directive, shader_module: &Module, errors: &mut Vec<Error>) {
    let args = directive.args();
    let shader_uniform_names: FxHashSet<_> = shader_module.uniform_names().collect();
    let run_arg_names: FxHashSet<_> = args.iter().map(|arg| &arg.name.slice).collect();
    for &missing_arg in shader_uniform_names.difference(&run_arg_names) {
        errors.push(Error::DirectiveParsing(ParsingError {
            path: directive.path().into(),
            span: directive.item_span(),
            message: format!("missing uniform argument `{missing_arg}`"),
        }));
    }
    for &unknown_arg in run_arg_names.difference(&shader_uniform_names) {
        errors.push(Error::DirectiveParsing(ParsingError {
            path: directive.path().into(),
            span: directive.arg(unknown_arg).name.span.clone(),
            message: format!(
                "no uniform variable `{unknown_arg}` in shader `{}`",
                directive.item_slice()
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
    for arg in directive.args() {
        let Some(storage_type) = modules.storages.get(&arg.value.var.slice) else {
            errors.push(Error::DirectiveParsing(ParsingError {
                path: directive.path().into(),
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
                path: directive.path().into(),
                span: arg.value.span,
                message: format!(
                    "found argument with type `{}`, expected uniform type `{}`",
                    arg_type.label, uniform.type_.label
                ),
            }));
        } else if arg_type.offset % offset_alignment != 0 {
            errors.push(Error::DirectiveParsing(ParsingError {
                path: directive.path().into(),
                span: arg.value.span,
                message: format!(
                    "value has an offset of {} bytes in `{}`, which is not a multiple of 256 bytes",
                    arg_type.offset, arg.value.var.slice,
                ),
            }));
        }
    }
}

fn check_buffer(
    is_vertex: bool,
    modules: &Modules,
    draw_directive: &Directive,
    shader_directive: &Directive,
    shader_module: &Module,
    errors: &mut Vec<Error>,
) {
    let type_name = if is_vertex {
        shader_directive.vertex_type()
    } else {
        shader_directive.instance_type()
    };
    let Some(expected_item_type) = shader_module.type_(&type_name.slice) else {
        return;
    };
    let buffer = if is_vertex {
        draw_directive.vertex_buffer()
    } else {
        draw_directive.instance_buffer()
    };
    let Some(storage_type) = modules.storages.get(&buffer.var.slice) else {
        errors.push(Error::DirectiveParsing(ParsingError {
            path: buffer.var.path.clone(),
            span: buffer.span,
            message: format!("unknown storage variable `{}`", buffer.var.slice),
        }));
        return;
    };
    let arg_type = match storage_type.field_ident_type(&buffer.fields) {
        Ok(arg_type) => arg_type,
        Err(error) => {
            errors.push(error);
            return;
        }
    };
    let arg_item_type = match (arg_type.array_params.as_ref(), is_vertex) {
        (Some((arg_item_type, _)), _) => arg_item_type,
        (None, false) => arg_type,
        (None, true) => {
            errors.push(Error::DirectiveParsing(ParsingError {
                path: draw_directive.path().into(),
                span: buffer.span,
                message: "found non-array argument".into(),
            }));
            return;
        }
    };
    if expected_item_type != arg_item_type {
        errors.push(Error::DirectiveParsing(ParsingError {
            path: draw_directive.path().into(),
            span: buffer.span,
            message: format!(
                "found item type `{}`, expected `{}`",
                arg_item_type.label, expected_item_type.label
            ),
        }));
    }
}

fn shader_def_kind(call_kind: DirectiveKind) -> Option<DirectiveKind> {
    match call_kind {
        DirectiveKind::Init | DirectiveKind::Run => Some(DirectiveKind::ComputeModule),
        DirectiveKind::Draw => Some(DirectiveKind::RenderModule),
        DirectiveKind::ComputeModule | DirectiveKind::RenderModule | DirectiveKind::Import => None,
    }
}

fn shader_module<'a>(root_path: &Path, directive: &Directive, modules: &'a Modules) -> &'a Module {
    if directive.kind() == DirectiveKind::Draw {
        &modules.render[&directive.item_ident(root_path)].1
    } else {
        &modules.compute[&directive.item_ident(root_path)].1
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
