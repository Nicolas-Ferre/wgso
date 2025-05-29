#![allow(clippy::multiple_inherent_impl)]

use crate::Error;
use itertools::Itertools;
use std::fmt::Debug;
use std::iter;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use wgso_parser::{Rule, Token};

pub(crate) mod imports;
pub(crate) mod shader_calls;
pub(crate) mod shader_defs;

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
            "mod" => match self.tokens[2].slice.as_str() {
                "compute" => DirectiveKind::ComputeModule,
                "render" => DirectiveKind::RenderModule,
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

    fn segment_path(&self, root_path: &Path, is_last_segment_included: bool) -> PathBuf {
        let segment_count = self.find_all_by_label("path_segment").count();
        let is_relative = self
            .find_all_by_label("path_segment")
            .next()
            .is_some_and(|segment| segment.slice == "~");
        let root_path = if is_relative {
            let file_path = self.path();
            file_path.parent().unwrap_or(file_path).to_path_buf()
        } else {
            root_path.to_path_buf()
        };
        self.find_all_by_label("path_segment")
            .enumerate()
            .filter(|(index, segment)| *index != 0 || segment.slice != "~")
            .filter(|(index, _)| is_last_segment_included || *index != segment_count - 1)
            .fold(root_path, |path, (index, segment)| {
                let last_index = if is_last_segment_included {
                    segment_count - 1
                } else {
                    segment_count - 2
                };
                if index == last_index {
                    path.join(format!("{}.wgsl", segment.slice))
                } else if segment.slice == "~" {
                    path.parent().map(Path::to_path_buf).unwrap_or(path)
                } else {
                    path.join(&segment.slice)
                }
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DirectiveKind {
    ComputeModule,
    RenderModule,
    Init,
    Run,
    Draw,
    Import,
}
