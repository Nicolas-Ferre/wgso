#![allow(clippy::multiple_inherent_impl)]

use crate::Error;
use itertools::Itertools;
use std::fmt::Debug;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use wgso_parser::{Rule, Token};

pub(crate) mod calls;
pub(crate) mod defs;

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
            let current_offset = offset + line.len() - line.len();
            match wgso_parser::parse(line, current_offset, path, rules) {
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
            _ => unreachable!("internal error: unrecognized directive"),
        }
    }

    pub(crate) fn section_name(&self) -> &Token {
        self.find_one_by_label("section_name")
    }

    pub(crate) fn item_ident(&self, root_path: &Path) -> (PathBuf, String) {
        let path = self.segment_path(root_path);
        let name = self
            .find_all_by_label("path_segment")
            .last()
            .expect("internal error: cannot find item name")
            .slice
            .clone();
        (path, name)
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

    fn segment_path(&self, root_path: &Path) -> PathBuf {
        let segment_count = self.find_all_by_label("path_segment").count();
        let is_relative = self
            .find_all_by_label("path_segment")
            .next()
            .is_some_and(|segment| segment.slice == "~");
        let root_path = if is_relative {
            self.path().into()
        } else {
            root_path.to_path_buf()
        };
        self.find_all_by_label("path_segment")
            .enumerate()
            .filter(|(index, _)| *index != 0 || !is_relative)
            .filter(|(index, _)| *index != segment_count - 1)
            .fold(root_path, |path, (index, segment)| {
                if index == segment_count - 2 {
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
    Mod,
    ComputeShader,
    RenderShader,
    Init,
    Run,
    Draw,
    Import,
}
