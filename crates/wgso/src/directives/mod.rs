#![allow(clippy::multiple_inherent_impl)]

use crate::directives::calls::DirectiveArg;
use crate::program::module::Modules;
use crate::program::type_::Type;
use crate::Error;
use itertools::Itertools;
use std::fmt::Debug;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{iter, mem};
use wgso_parser::{ParsingError, Rule, Token};

pub(crate) mod calls;
pub(crate) mod defs;
pub(crate) mod toggle;

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
        if line.trim_start().starts_with('#') {
            match wgso_parser::parse(line, offset, path, rules) {
                Ok(tokens) => parsed_directives.push(Directive { tokens }),
                Err(error) => errors.push(Error::DirectiveParsing(error)),
            }
        }
        offset += line.len() + 1;
    }
    parsed_directives
}

#[derive(Debug, Clone)]
pub(crate) struct Directive {
    tokens: Vec<Token>,
}

impl Directive {
    pub(crate) fn code(&self) -> String {
        self.tokens
            .iter()
            .map(|token| token.slice.as_str())
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
        match self.tokens[1].slice.as_str() {
            "mod" => DirectiveKind::Mod,
            "shader" => match self.tokens[3].slice.as_str() {
                "compute" => DirectiveKind::ComputeShader,
                "render" => DirectiveKind::RenderShader,
                _ => unreachable!("internal error: unrecognized shader directive"),
            },
            "init" => DirectiveKind::Init,
            "run" => DirectiveKind::Run,
            "draw" => DirectiveKind::Draw,
            "import" => DirectiveKind::Import,
            "toggle" => DirectiveKind::Toggle,
            _ => unreachable!("internal error: unrecognized directive"),
        }
    }

    pub(crate) fn section_name(&self) -> &Token {
        self.find_one_by_label("section_name")
    }

    pub(crate) fn item_ident(&self, root_path: &Path) -> (PathBuf, String) {
        let path: PathBuf = self
            .segment_path(root_path)
            .parent()
            .expect("internal error: segment path should have at least one non-parent segment")
            .with_extension("wgsl");
        let name = self
            .find_all_by_label("path_segment")
            .last()
            .expect("internal error: cannot find item name")
            .slice
            .clone();
        (path, name)
    }

    pub(crate) fn segment_path(&self, root_path: &Path) -> PathBuf {
        let is_relative = self
            .find_all_by_label("path_segment")
            .next()
            .is_some_and(|segment| segment.slice == "~");
        let root_path = if is_relative {
            self.path().with_extension("")
        } else {
            root_path.into()
        };
        self.find_all_by_label("path_segment")
            .enumerate()
            .filter(|(index, _)| *index != 0 || !is_relative)
            .fold(root_path, |path, (_, segment)| {
                if segment.slice == "~" {
                    path.parent().map(Path::to_path_buf).unwrap_or(path)
                } else {
                    path.join(&segment.slice)
                }
            })
    }

    pub(crate) fn buffers(&self) -> Vec<BufferRef> {
        match self.kind() {
            DirectiveKind::Init | DirectiveKind::Run => {
                self.args().into_iter().map(|arg| arg.value).collect()
            }
            // coverage: off (not used for now)
            DirectiveKind::Draw => self
                .args()
                .into_iter()
                .map(|arg| arg.value)
                .chain([self.vertex_buffer(), self.instance_buffer()])
                .collect(),
            // coverage: on
            DirectiveKind::Toggle => vec![self.toggle_value_buffer()],
            DirectiveKind::Mod
            | DirectiveKind::Import
            | DirectiveKind::ComputeShader
            | DirectiveKind::RenderShader => {
                vec![]
            }
        }
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

    fn extract_arg(tokens: &[&Token], index: usize, token: &Token) -> DirectiveArg {
        DirectiveArg {
            name: (*token).clone(),
            value: BufferRef::new(
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

    fn buffer(&self, var_label: &str, field_label: &str) -> BufferRef {
        assert!([
            DirectiveKind::Draw,
            DirectiveKind::Import,
            DirectiveKind::Toggle
        ]
        .contains(&self.kind()));
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
            BufferRef::new(var, mem::take(&mut current_fields))
        } else {
            unreachable!("internal error: directive arguments should be validated");
        }
    }
}

fn find_buffer_type(
    buffer: &BufferRef,
    modules: &Modules,
    errors: &mut Vec<Error>,
) -> Option<Type> {
    let Some(storage) = modules.storages.get(&buffer.var.slice) else {
        errors.push(Error::DirectiveParsing(ParsingError {
            path: buffer.var.path.clone(),
            span: buffer.span.clone(),
            message: format!("unknown storage variable `{}`", buffer.var.slice),
        }));
        return None;
    };
    match storage.type_.field_ident_type(&buffer.fields) {
        Ok(arg_type) => Some(arg_type.clone()),
        Err(error) => {
            errors.push(error);
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DirectiveKind {
    Mod,
    ComputeShader,
    RenderShader,
    Init,
    Run,
    Draw,
    Import,
    Toggle,
}

#[derive(Debug)]
pub(crate) struct BufferRef {
    pub(crate) span: Range<usize>,
    pub(crate) var: Token,
    pub(crate) fields: Vec<Token>,
}

impl BufferRef {
    fn new(var: Token, fields: Vec<Token>) -> Self {
        Self {
            span: var.span.start..fields.last().map_or(var.span.end, |field| field.span.end),
            var,
            fields,
        }
    }

    pub(crate) fn path(&self) -> String {
        iter::once(&self.var.slice)
            .chain(self.fields.iter().map(|field| &field.slice))
            .join(".")
    }
}
