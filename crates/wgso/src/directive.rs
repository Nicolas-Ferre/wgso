use crate::Error;
use itertools::Itertools;
use logos::{Lexer, Logos, Span};
use std::path::{Path, PathBuf};

pub(crate) fn parse(line: &str, path: &Path, offset: usize) -> Result<Directive, Error> {
    let mut lexer = Lexer::new(line);
    let span = offset..offset + line.len();
    match next_token(&mut lexer, path, offset, vec![Token::Shader, Token::Run])? {
        (Token::Shader, _, _) => parse_shader(&mut lexer, path, offset, span),
        (Token::Run, _, _) => parse_run(&mut lexer, path, offset),
        _ => unreachable!("internal error: unexpected token"),
    }
}

fn parse_shader(
    lexer: &mut Lexer<'_, Token>,
    path: &Path,
    offset: usize,
    span: Span,
) -> Result<Directive, Error> {
    next_token(lexer, path, offset, vec![Token::OpenAngleBracket])?;
    next_token(lexer, path, offset, vec![Token::Compute])?;
    next_token(lexer, path, offset, vec![Token::CloseAngleBracket])?;
    let (_, name, _) = next_token(lexer, path, offset, vec![Token::Ident])?;
    Ok(Directive::ComputeShader(ShaderDirective {
        name: name.into(),
        path: path.to_path_buf(),
        span,
    }))
}

fn parse_run(lexer: &mut Lexer<'_, Token>, path: &Path, offset: usize) -> Result<Directive, Error> {
    let (_, name, _) = next_token(lexer, path, offset, vec![Token::Ident])?;
    next_token(lexer, path, offset, vec![Token::OpenParenthesis])?;
    next_token(lexer, path, offset, vec![Token::CloseParenthesis])?;
    Ok(Directive::Run(RunDirective { name: name.into() }))
}

fn next_token<'a>(
    lexer: &mut Lexer<'a, Token>,
    path: &Path,
    offset: usize,
    expected: Vec<Token>,
) -> Result<(Token, &'a str, Span), Error> {
    let (last_expected, other_expected) = expected
        .split_last()
        .expect("internal error: expected list of tokens should not be empty");
    lexer
        .next()
        .ok_or_else(|| parsing_error(lexer, path, offset, "unexpected end of file"))?
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
    pub(crate) span: Span,
}

#[derive(Debug, Clone)]
pub(crate) struct RunDirective {
    pub(crate) name: String,
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
            Self::OpenParenthesis => "`(`",
            Self::CloseParenthesis => "`)`",
            Self::OpenAngleBracket => "`<`",
            Self::CloseAngleBracket => "`>`",
            Self::Ident => "identifier",
        }
    }
    // coverage: on
}
