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
) -> Vec<Directive> {
    let mut parsed_directives = vec![];
    let mut offset = 0;
    for line in code.lines() {
        if let Some(directive) = line.trim_start().strip_prefix("#") {
            let current_offset = offset + line.len() - directive.len();
            match wgso_parser::parse(directive, current_offset, path, rules) {
                Ok(tokens) => parsed_directives.push(Directive { tokens }),
                Err(error) => errors.push(Error::DirectiveParsing(error)),
            }
        }
        offset += line.len() + 1;
    }
    parsed_directives
}

pub(crate) fn find_all_by_kind(
    directives: &[Directive],
    kind: DirectiveKind,
) -> impl Iterator<Item = &Directive> {
    directives
        .iter()
        .filter(move |directive| directive.kind() == kind)
}

#[derive(Debug, Clone)]
pub(crate) struct Directive {
    tokens: Vec<Token>,
}

impl Directive {
    pub(crate) fn code(&self) -> String {
        iter::once("#")
            .chain(self.tokens.iter().map(|token| token.slice.as_str()))
            .join(" ")
    }

    pub(crate) fn path(&self) -> &Path {
        &self.tokens[0].path
    }

    pub(crate) fn span(&self) -> Range<usize> {
        let min_span = self.tokens[0].span.start;
        let max_span = self.tokens[self.tokens.len() - 1].span.end;
        min_span..max_span
    }

    pub(crate) fn kind(&self) -> DirectiveKind {
        match self.tokens[0].slice.as_str() {
            "shader" => match self.tokens[2].slice.as_str() {
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

    pub(crate) fn shader_name(&self) -> &Token {
        self.find_one_by_label("shader_name")
    }

    pub(crate) fn priority(&self) -> i32 {
        assert!(CALL_DIRECTIVE_KINDS.contains(&self.kind()));
        if let Some(priority) = self.find_all_by_label("priority").next() {
            Self::convert_to_integer(priority)
        } else {
            0
        }
    }

    pub(crate) fn workgroup_count(&self) -> (u16, u16, u16) {
        assert_eq!(self.kind(), DirectiveKind::ComputeShader);
        let mut tokens = self.find_all_by_label("workgroup_count");
        let workgroup_count_x = tokens.next().map_or(1, Self::convert_to_integer);
        let workgroup_count_y = tokens.next().map_or(1, Self::convert_to_integer);
        let workgroup_count_z = tokens.next().map_or(1, Self::convert_to_integer);
        (workgroup_count_x, workgroup_count_y, workgroup_count_z)
    }

    pub(crate) fn vertex_type(&self) -> &Token {
        assert_eq!(self.kind(), DirectiveKind::RenderShader);
        self.find_one_by_label("vertex_type")
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

    pub(crate) fn import_path(&self) -> PathBuf {
        let segment_count = self.find_all_by_label("import_segment").count();
        self.find_all_by_label("import_segment")
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

    fn find_one_by_label(&self, label: &str) -> &Token {
        self.tokens
            .iter()
            .find(|token| token.label.as_deref() == Some(label))
            .expect("internal error: cannot find directive token by label")
    }

    fn find_all_by_label<'a>(&'a self, label: &'a str) -> impl Iterator<Item = &'a Token> {
        self.tokens
            .iter()
            .filter(|token| token.label.as_deref() == Some(label))
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
