use crate::directive::draw::DrawDirective;
use crate::directive::import::ImportDirective;
use crate::directive::render_shader::RenderShaderDirective;
use crate::Error;
use compute_shader::ComputeShaderDirective;
use run::RunDirective;
use std::path::Path;
use token::{Lexer, Token, TokenKind};

pub(crate) mod common;
pub(crate) mod compute_shader;
pub(crate) mod draw;
pub(crate) mod import;
pub(crate) mod render_shader;
pub(crate) mod run;
pub(crate) mod token;

#[derive(Debug)]
pub(crate) struct Directives {
    pub(crate) imports: Vec<ImportDirective>,
    pub(crate) compute_shaders: Vec<ComputeShaderDirective>,
    pub(crate) render_shaders: Vec<RenderShaderDirective>,
    pub(crate) runs: Vec<RunDirective>,
    pub(crate) draws: Vec<DrawDirective>,
}

impl Directives {
    pub(crate) fn parse(path: &Path, code: &str, errors: &mut Vec<Error>) -> Self {
        let mut lexer = Lexer::new(code, path);
        let mut directives = Self {
            imports: vec![],
            compute_shaders: vec![],
            render_shaders: vec![],
            runs: vec![],
            draws: vec![],
        };
        let mut skip_until_next_line = false;
        while let Some(token) = lexer.next() {
            let Ok(token) = token else {
                skip_until_next_line = true;
                continue;
            };
            if token.kind == TokenKind::LineBreak {
                skip_until_next_line = false;
            } else if !skip_until_next_line && token.kind == TokenKind::Hashtag {
                if let Err(error) = directives.parse_directive(&mut lexer, token) {
                    errors.push(error);
                } else if let Err(error) = lexer.next_expected(&[TokenKind::LineBreak]) {
                    errors.push(error);
                }
            } else {
                skip_until_next_line = true;
            }
        }
        directives
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn parse_directive(&mut self, lexer: &mut Lexer<'_>, hashtag: Token<'_>) -> Result<(), Error> {
        let token = lexer.next_expected(&[
            TokenKind::Import,
            TokenKind::Shader,
            TokenKind::Init,
            TokenKind::Run,
            TokenKind::Draw,
        ])?;
        match token.kind {
            TokenKind::Import => self.imports.push(ImportDirective::parse(lexer, &hashtag)?),
            TokenKind::Shader => {
                lexer.next_expected(&[TokenKind::OpenAngleBracket])?;
                let token = lexer.next_expected(&[TokenKind::Compute, TokenKind::Render])?;
                if token.kind == TokenKind::Compute {
                    self.compute_shaders
                        .push(ComputeShaderDirective::parse(lexer, &hashtag)?);
                } else {
                    self.render_shaders
                        .push(RenderShaderDirective::parse(lexer, &hashtag)?);
                }
            }
            TokenKind::Init => self.runs.push(RunDirective::parse(lexer, &hashtag, true)?),
            TokenKind::Run => self.runs.push(RunDirective::parse(lexer, &hashtag, false)?),
            TokenKind::Draw => self.draws.push(DrawDirective::parse(lexer, &hashtag)?),
            _ => unreachable!("internal error: unexpected token"),
        }
        Ok(())
    }
}
