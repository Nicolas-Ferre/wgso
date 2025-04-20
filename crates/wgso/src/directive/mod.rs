use crate::directive::import::ImportDirective;
use crate::Error;
use run::RunDirective;
use shader::ShaderDirective;
use std::path::Path;
use tokens::{Lexer, Token, TokenKind};

pub(crate) mod import;
pub(crate) mod run;
pub(crate) mod shader;
pub(crate) mod tokens;

#[derive(Debug)]
pub(crate) struct Directives {
    directives: Vec<Directive>,
}

impl Directives {
    pub(crate) fn parse(path: &Path, code: &str, errors: &mut Vec<Error>) -> Self {
        let mut lexer = Lexer::new(code, path);
        let mut directives = vec![];
        let mut skip_until_next_line = false;
        while let Some(token) = lexer.next() {
            if let Ok(token) = token {
                if token.kind == TokenKind::LineBreak {
                    skip_until_next_line = false;
                } else if !skip_until_next_line && token.kind == TokenKind::Hashtag {
                    match Self::parse_directive(&mut lexer, token) {
                        Ok(directive) => {
                            directives.push(directive);
                            if let Err(error) = lexer.next_expected(&[TokenKind::LineBreak]) {
                                errors.push(error);
                            }
                        }
                        Err(error) => errors.push(error),
                    }
                } else {
                    skip_until_next_line = true;
                }
            } else {
                skip_until_next_line = true;
            }
        }
        Self { directives }
    }

    pub(crate) fn imports(&self) -> impl Iterator<Item = &ImportDirective> + '_ {
        self.directives
            .iter()
            .filter_map(|directive| match directive {
                Directive::Import(directive) => Some(directive),
                Directive::Shader(_) | Directive::Run(_) => None,
            })
    }

    pub(crate) fn compute_shaders(&self) -> impl Iterator<Item = &ShaderDirective> + '_ {
        self.directives
            .iter()
            .filter_map(|directive| match directive {
                Directive::Shader(directive) => Some(directive),
                Directive::Import(_) | Directive::Run(_) => None,
            })
    }

    pub(crate) fn runs(&self) -> impl Iterator<Item = &RunDirective> + '_ {
        self.directives
            .iter()
            .filter_map(|directive| match directive {
                Directive::Run(directive) => Some(directive),
                Directive::Import(_) | Directive::Shader(_) => None,
            })
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn parse_directive(lexer: &mut Lexer<'_>, hashtag: Token<'_>) -> Result<Directive, Error> {
        let token = lexer.next_expected(&[
            TokenKind::Import,
            TokenKind::Shader,
            TokenKind::Init,
            TokenKind::Run,
        ])?;
        Ok(match token.kind {
            TokenKind::Import => Directive::Import(ImportDirective::parse(lexer, &hashtag)?),
            TokenKind::Shader => Directive::Shader(ShaderDirective::parse(lexer, &hashtag)?),
            TokenKind::Init => Directive::Run(RunDirective::parse(lexer, &hashtag, true)?),
            TokenKind::Run => Directive::Run(RunDirective::parse(lexer, &hashtag, false)?),
            _ => unreachable!("internal error: unexpected token"),
        })
    }
}

#[derive(Debug)]
enum Directive {
    Import(ImportDirective),
    Shader(ShaderDirective),
    Run(RunDirective),
}
