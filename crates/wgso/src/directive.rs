use crate::Error;
use itertools::Itertools;
use std::fmt::Debug;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{iter, mem};
use wgso_parser::{Rule, Token};

const CALL_DIRECTIVE_KINDS: &[DirectiveKind] =
    &[DirectiveKind::Init, DirectiveKind::Run, DirectiveKind::Draw];

pub(crate) fn load_rules() -> Vec<Rule> {
    wgso_parser::load_rules(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/config/directives.yaml"
    )))
    .expect("internal error: directive config should be valid")
}

pub(crate) fn parse_file(
    code: &str,
    path: &Path,
    rules: &[Rule],
    errors: &mut Vec<Error>,
) -> Vec<Vec<Token>> {
    let mut parsed_directives = vec![];
    let mut offset = 0;
    for line in code.lines() {
        if let Some(directive) = line.trim_start().strip_prefix("#") {
            let current_offset = offset + line.len() - directive.len();
            match wgso_parser::parse(directive, current_offset, path, rules) {
                Ok(tokens) => parsed_directives.push(tokens),
                Err(error) => errors.push(Error::DirectiveParsing(error)),
            }
        }
        offset += line.len() + 1;
    }
    parsed_directives
}

pub(crate) fn find_all_by_kind(
    directives: &[Vec<Token>],
    kind: DirectiveKind,
) -> impl Iterator<Item = &[Token]> {
    directives
        .iter()
        .filter(move |directive| self::kind(directive) == kind)
        .map(Vec::as_slice)
}

pub(crate) fn code(directive: &[Token]) -> String {
    iter::once("#")
        .chain(directive.iter().map(|token| token.slice.as_str()))
        .join(" ")
}

pub(crate) fn path(directive: &[Token]) -> &Path {
    &directive[0].path
}

pub(crate) fn span(directive: &[Token]) -> Range<usize> {
    let min_span = directive[0].span.start;
    let max_span = directive[directive.len() - 1].span.end;
    min_span..max_span
}

pub(crate) fn kind(directive: &[Token]) -> DirectiveKind {
    match directive[0].slice.as_str() {
        "shader" => match directive[2].slice.as_str() {
            "compute" => DirectiveKind::ComputeShader,
            "render" => DirectiveKind::RenderShader,
            _ => unreachable!("internal error: unrecognized shader directive"),
        },
        "init" => DirectiveKind::Init,
        "run" => DirectiveKind::Run,
        "draw" => DirectiveKind::Draw,
        "import" => DirectiveKind::Import,
        _ => unreachable!("internal error: unrecognized directive"),
    }
}

// TODO: change to associated function
pub(crate) fn shader_name(directive: &[Token]) -> &Token {
    find_one_by_label(directive, "shader_name")
}

pub(crate) fn priority(directive: &[Token]) -> i32 {
    assert!(CALL_DIRECTIVE_KINDS.contains(&kind(directive)));
    if let Some(priority) = find_all_by_label(directive, "priority").next() {
        convert_to_integer(priority)
    } else {
        0
    }
}

pub(crate) fn workgroup_count(directive: &[Token]) -> (u16, u16, u16) {
    assert_eq!(kind(directive), DirectiveKind::ComputeShader);
    let mut tokens = find_all_by_label(directive, "workgroup_count");
    let workgroup_count_x = tokens.next().map_or(1, convert_to_integer);
    let workgroup_count_y = tokens.next().map_or(1, convert_to_integer);
    let workgroup_count_z = tokens.next().map_or(1, convert_to_integer);
    (workgroup_count_x, workgroup_count_y, workgroup_count_z)
}

pub(crate) fn vertex_type(directive: &[Token]) -> &Token {
    assert_eq!(kind(directive), DirectiveKind::RenderShader);
    find_one_by_label(directive, "vertex_type")
}

pub(crate) fn vertex_buffer(directive: &[Token]) -> DirectiveArgValue {
    assert_eq!(kind(directive), DirectiveKind::Draw);
    let mut current_var = None;
    let mut current_fields = vec![];
    for token in directive {
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

pub(crate) fn args(directive: &[Token]) -> Vec<DirectiveArg> {
    assert!(CALL_DIRECTIVE_KINDS.contains(&kind(directive)));
    let tokens = arg_tokens(directive);
    let mut args = vec![];
    for (index, token) in tokens.iter().enumerate() {
        if token.label.as_deref() == Some("arg_name") {
            args.push(extract_arg(&tokens, index, token));
        }
    }
    args
}

pub(crate) fn arg(directive: &[Token], arg_name: &str) -> DirectiveArg {
    assert!(CALL_DIRECTIVE_KINDS.contains(&kind(directive)));
    let tokens = arg_tokens(directive);
    for (index, token) in tokens.iter().enumerate() {
        if token.label.as_deref() == Some("arg_name") && token.slice == arg_name {
            return extract_arg(&tokens, index, token);
        }
    }
    unreachable!("internal error: directive arguments should be validated");
}

pub(crate) fn import_path(directive: &[Token]) -> PathBuf {
    let segment_count = find_all_by_label(directive, "import_segment").count();
    find_all_by_label(directive, "import_segment")
        .enumerate()
        .map(|(index, segment)| {
            if index == segment_count - 1 {
                format!("{}.wgsl", segment.slice.clone())
            } else {
                segment.slice.clone()
            }
        })
        .collect::<PathBuf>()
}

fn find_one_by_label<'a>(directive: &'a [Token], label: &str) -> &'a Token {
    directive
        .iter()
        .find(|token| token.label.as_deref() == Some(label))
        .expect("internal error: cannot find directive token by label")
}

fn find_all_by_label<'a>(
    directive: &'a [Token],
    label: &'a str,
) -> impl Iterator<Item = &'a Token> {
    directive
        .iter()
        .filter(|token| token.label.as_deref() == Some(label))
}

fn convert_to_integer<T>(token: &Token) -> T
where
    T: FromStr,
    T::Err: Debug,
{
    token
        .slice
        .parse::<T>()
        .expect("internal error: directive integers should be validated")
}

fn arg_tokens(directive: &[Token]) -> Vec<&Token> {
    directive
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DirectiveKind {
    ComputeShader,
    RenderShader,
    Init,
    Run,
    Draw,
    Import,
}
