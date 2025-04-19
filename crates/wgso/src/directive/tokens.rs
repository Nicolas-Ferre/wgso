use crate::Error;
use itertools::Itertools;
use logos::Logos;
use std::ops::Range;
use std::path::{Path, PathBuf};

/// A parsed identifier.
#[derive(Debug, Clone)]
pub struct Ident {
    pub(crate) label: String,
    pub(crate) span: Range<usize>,
    pub(crate) path: PathBuf,
}

impl Ident {
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, Error> {
        let token = lexer.next_expected(&[TokenKind::Ident])?;
        Ok(Self {
            label: token.slice.into(),
            span: token.span,
            path: lexer.path().into(),
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Lexer<'a> {
    lexer: logos::Lexer<'a, TokenKind>,
    path: &'a Path,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(code: &'a str, path: &'a Path) -> Self {
        Self {
            lexer: logos::Lexer::new(code),
            path,
        }
    }

    pub(crate) fn source_slice(&self, span: Range<usize>) -> &str {
        &self.lexer.source()[span]
    }

    pub(crate) fn offset(&self) -> usize {
        self.lexer.span().end
    }

    pub(crate) fn path(&self) -> &Path {
        self.path
    }

    pub(crate) fn next_expected(&mut self, expected: &[TokenKind]) -> Result<Token<'_>, Error> {
        let (last_expected, other_expected) = expected
            .split_last()
            .expect("internal error: expected list of tokens should not be empty");
        self.next()
            .ok_or_else(|| {
                Error::DirectiveParsing(
                    self.path.into(),
                    self.lexer.span(),
                    "unexpected end of file".into(),
                )
            })?
            .and_then(|token| {
                if expected.is_empty() || expected.contains(&token.kind) {
                    Ok(token)
                } else {
                    let last_expected_label = last_expected.label();
                    Err(Error::DirectiveParsing(
                        self.path.into(),
                        token.span,
                        if other_expected.is_empty() {
                            format!("expected {last_expected_label}")
                        } else {
                            format!(
                                "expected {} or {last_expected_label}",
                                other_expected.iter().map(|t| t.label()).join(", "),
                            )
                        },
                    ))
                }
            })
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next().map(|kind| {
            kind.map(|kind| Token {
                kind,
                span: self.lexer.span(),
                slice: self.lexer.slice(),
            })
            .map_err(|()| {
                Error::DirectiveParsing(
                    self.path.into(),
                    self.lexer.span(),
                    "unexpected token".into(),
                )
            })
        })
    }
}

pub(crate) struct Token<'a> {
    pub(crate) kind: TokenKind,
    pub(crate) span: Range<usize>,
    pub(crate) slice: &'a str,
}

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
#[logos(skip r"[ \t]+")]
pub(crate) enum TokenKind {
    #[token("shader")]
    Shader,

    #[token("compute")]
    Compute,

    #[token("run")]
    Run,

    #[token("#")]
    Hashtag,

    #[token(",")]
    Comma,

    #[token("=")]
    Equal,

    #[token("(")]
    OpenParenthesis,

    #[token(")")]
    CloseParenthesis,

    #[token("<")]
    OpenAngleBracket,

    #[token(">")]
    CloseAngleBracket,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,

    #[regex("(\r\n|\r|\n)")]
    LineBreak,
}

impl TokenKind {
    // coverage: off (not all labels are used in practice)
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Shader => "`shader`",
            Self::Compute => "`compute`",
            Self::Run => "`run`",
            Self::Hashtag => "`#`",
            Self::Comma => "`,`",
            Self::Equal => "`=`",
            Self::OpenParenthesis => "`(`",
            Self::CloseParenthesis => "`)`",
            Self::OpenAngleBracket => "`<`",
            Self::CloseAngleBracket => "`>`",
            Self::Ident => "identifier",
            Self::LineBreak => "line break",
        }
    }
    // coverage: on
}
