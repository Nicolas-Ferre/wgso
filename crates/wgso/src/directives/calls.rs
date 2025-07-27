use crate::directives::{BufferRef, Directive, DirectiveKind};
use crate::program::file::Files;
use crate::program::module::{Module, Modules};
use crate::program::section::Sections;
use crate::Error;
use fxhash::FxHashSet;
use itertools::Itertools;
use std::ops::Range;
use std::path::Path;
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

    pub(crate) fn vertex_buffer(&self) -> BufferRef {
        self.buffer("vertex_buffer_var", "vertex_buffer_field")
    }

    pub(crate) fn instance_buffer(&self) -> BufferRef {
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
}

pub(crate) fn check<'a>(
    directives: impl Iterator<Item = &'a Directive>,
    files: &Files,
    root_path: &Path,
    errors: &mut Vec<Error>,
) {
    let directives: Vec<_> = directives.collect();
    for directive in &directives {
        if let Some(shader_def_kind) = shader_def_kind(directive.kind()) {
            check_shader_name(root_path, directive, files, shader_def_kind, errors);
        }
    }
}

pub(crate) fn check_args(
    root_path: &Path,
    sections: &Sections,
    modules: &Modules,
    errors: &mut Vec<Error>,
) {
    for (directive, _) in sections.run_directives() {
        let shader_module = shader_module(root_path, directive, modules);
        check_arg_names(directive, shader_module, errors);
        check_arg_value(modules, directive, shader_module, errors);
    }
    for (directive, _) in sections.draw_directives() {
        let shader_module = shader_module(root_path, directive, modules);
        check_arg_names(directive, shader_module, errors);
        check_arg_value(modules, directive, shader_module, errors);
        let shader_ident = directive.item_ident(root_path);
        if let Some(module) = modules.render.get(&shader_ident) {
            check_buffer(true, modules, directive, module, errors);
            check_buffer(false, modules, directive, module, errors);
        }
    }
}

fn check_shader_name(
    root_path: &Path,
    directive: &Directive,
    files: &Files,
    shader_def_kind: DirectiveKind,
    errors: &mut Vec<Error>,
) {
    let item_ident = directive.item_ident(root_path);
    if !files.exists(&item_ident.0) {
        errors.push(Error::DirectiveParsing(ParsingError {
            path: directive.path().into(),
            span: directive.item_span(),
            message: format!("'{}' file does not exist", item_ident.0.display()),
        }));
        return;
    }
    let is_shader_found = files.directives.iter().any(|directive| {
        directive.kind() == shader_def_kind
            && directive.path() == item_ident.0
            && directive.section_name().slice == item_ident.1
    });
    if !is_shader_found {
        errors.push(Error::DirectiveParsing(ParsingError {
            path: directive.path().into(),
            span: directive.item_span(),
            message: format!(
                "`{}` module not found in file '{}'",
                item_ident.1,
                item_ident.0.display(),
            ),
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
        let Some(arg_type) = super::find_buffer_type(&arg.value, modules, errors) else {
            return;
        };
        let Some(uniform) = shader_module.uniform_binding(&arg.name.slice) else {
            continue;
        };
        if *uniform.type_ != arg_type {
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
    shader_module: &Module,
    errors: &mut Vec<Error>,
) {
    let shader_directive = shader_module.main_directive();
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
    let Some(arg_type) = super::find_buffer_type(&buffer, modules, errors) else {
        return;
    };
    let arg_item_type = match (arg_type.array_params.as_ref(), is_vertex) {
        (Some((arg_item_type, _)), _) => arg_item_type,
        (None, false) => &arg_type,
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
        DirectiveKind::Init | DirectiveKind::Run => Some(DirectiveKind::ComputeShader),
        DirectiveKind::Draw => Some(DirectiveKind::RenderShader),
        DirectiveKind::Import => Some(DirectiveKind::Mod),
        DirectiveKind::Mod
        | DirectiveKind::ComputeShader
        | DirectiveKind::RenderShader
        | DirectiveKind::Toggle => None,
    }
}

fn shader_module<'a>(root_path: &Path, directive: &Directive, modules: &'a Modules) -> &'a Module {
    if directive.kind() == DirectiveKind::Draw {
        &modules.render[&directive.item_ident(root_path)]
    } else {
        &modules.compute[&directive.item_ident(root_path)]
    }
}

#[derive(Debug)]
pub(crate) struct DirectiveArg {
    pub(crate) name: Token,
    pub(crate) value: BufferRef,
}
