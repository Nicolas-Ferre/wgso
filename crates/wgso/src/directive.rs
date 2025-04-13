use crate::Error;
use fxhash::FxHashMap;
use itertools::Itertools;
use logos::{Lexer, Logos};
use std::collections::hash_map::Entry;
use std::ops::Range;
use std::path::{Path, PathBuf};

pub(crate) fn parse(line: &str, path: &Path, offset: usize) -> Result<Directive, Error> {
    let mut lexer = Lexer::new(line);
    let span = offset..offset + line.len();
    match next_token(&mut lexer, path, offset, &[Token::Shader, Token::Run])? {
        (Token::Shader, _, _) => parse_shader(&mut lexer, path, offset, span),
        (Token::Run, _, _) => parse_run(&mut lexer, path, offset, span),
        _ => unreachable!("internal error: unexpected token"),
    }
}

fn parse_shader(
    lexer: &mut Lexer<'_, Token>,
    path: &Path,
    offset: usize,
    span: Range<usize>,
) -> Result<Directive, Error> {
    next_token(lexer, path, offset, &[Token::OpenAngleBracket])?;
    next_token(lexer, path, offset, &[Token::Compute])?;
    next_token(lexer, path, offset, &[Token::CloseAngleBracket])?;
    let (_, name, _) = next_token(lexer, path, offset, &[Token::Ident])?;
    Ok(Directive::ComputeShader(ShaderDirective {
        name: name.into(),
        path: path.to_path_buf(),
        span,
    }))
}

fn parse_run(
    lexer: &mut Lexer<'_, Token>,
    path: &Path,
    offset: usize,
    span: Range<usize>,
) -> Result<Directive, Error> {
    let (_, name, _) = next_token(lexer, path, offset, &[Token::Ident])?;
    next_token(lexer, path, offset, &[Token::OpenParenthesis])?;
    let mut params = FxHashMap::default();
    let close_parenthesis = &[Token::CloseParenthesis];
    while next_token(&mut lexer.clone(), path, offset, close_parenthesis).is_err() {
        if !params.is_empty() {
            next_token(lexer, path, offset, &[Token::Comma])?;
        }
        let (_, param_name, param_name_span) = next_token(lexer, path, offset, &[Token::Ident])?;
        next_token(lexer, path, offset, &[Token::Equal])?;
        let (_, param_value, param_value_span) = next_token(lexer, path, offset, &[Token::Ident])?;
        match params.entry(param_name.into()) {
            Entry::Occupied(_) => {
                return Err(Error::DirectiveParsing(
                    path.into(),
                    param_name_span.start + offset..param_name_span.end + offset,
                    "duplicated parameter".into(),
                ))
            }
            Entry::Vacant(entry) => {
                entry.insert(RunParam {
                    name_span: param_name_span.start + offset..param_name_span.end + offset,
                    value_span: param_value_span.start + offset..param_value_span.end + offset,
                    value: param_value.into(),
                });
            }
        }
    }
    next_token(lexer, path, offset, close_parenthesis)?;
    Ok(Directive::Run(RunDirective {
        path: path.to_path_buf(),
        span,
        source: lexer.source().into(),
        name: name.into(),
        params,
    }))
}

fn next_token<'a>(
    lexer: &mut Lexer<'a, Token>,
    path: &Path,
    offset: usize,
    expected: &[Token],
) -> Result<(Token, &'a str, Range<usize>), Error> {
    let (last_expected, other_expected) = expected
        .split_last()
        .expect("internal error: expected list of tokens should not be empty");
    lexer
        .next()
        .ok_or_else(|| parsing_error(lexer, path, offset, "unexpected end of line"))?
        .map_err(|()| parsing_error(lexer, path, offset, "unexpected token"))
        .and_then(|token| {
            if expected.is_empty() || expected.contains(&token) {
                Ok((token, lexer.slice(), lexer.span()))
            } else {
                let last_expected_label = last_expected.label();
                Err(parsing_error(
                    lexer,
                    path,
                    offset,
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

fn parsing_error(
    lexer: &Lexer<'_, Token>,
    path: &Path,
    offset: usize,
    message: impl Into<String>,
) -> Error {
    Error::DirectiveParsing(
        path.into(),
        lexer.span().start + offset..lexer.span().end + offset,
        message.into(),
    )
}

#[derive(Debug, Clone)]
pub(crate) enum Directive {
    ComputeShader(ShaderDirective),
    Run(RunDirective),
}

/// A parsed `#shader` directive.
#[derive(Debug, Clone)]
pub struct ShaderDirective {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) span: Range<usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct RunDirective {
    pub(crate) path: PathBuf,
    pub(crate) span: Range<usize>,
    pub(crate) source: String,
    pub(crate) name: String,
    pub(crate) params: FxHashMap<String, RunParam>,
}

#[derive(Debug, Clone)]
pub(crate) struct RunParam {
    pub(crate) name_span: Range<usize>,
    pub(crate) value_span: Range<usize>,
    pub(crate) value: String,
}

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
#[logos(skip r"[ \t]+")]
enum Token {
    #[token("shader")]
    Shader,

    #[token("compute")]
    Compute,

    #[token("run")]
    Run,

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
}

impl Token {
    // coverage: off (not all labels are used in practice)
    fn label(self) -> &'static str {
        match self {
            Self::Shader => "`shader`",
            Self::Compute => "`compute`",
            Self::Run => "`run`",
            Self::Comma => "`,`",
            Self::Equal => "`=`",
            Self::OpenParenthesis => "`(`",
            Self::CloseParenthesis => "`)`",
            Self::OpenAngleBracket => "`<`",
            Self::CloseAngleBracket => "`>`",
            Self::Ident => "identifier",
        }
    }
    // coverage: on
}
