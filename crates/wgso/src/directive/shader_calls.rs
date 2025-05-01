use crate::directive::{Directive, DirectiveKind};
use crate::Error;
use std::mem;
use std::ops::Range;
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

fn shader_def_kind(call_kind: DirectiveKind) -> Option<DirectiveKind> {
    match call_kind {
        DirectiveKind::Init | DirectiveKind::Run => Some(DirectiveKind::ComputeShader),
        DirectiveKind::Draw => Some(DirectiveKind::RenderShader),
        DirectiveKind::ComputeShader | DirectiveKind::RenderShader | DirectiveKind::Import => None,
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
