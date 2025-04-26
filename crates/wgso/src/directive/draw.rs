use crate::directive::common;
use crate::directive::common::{ShaderArg, ShaderArgValue};
use crate::directive::token::{Ident, Lexer, Token, TokenKind};
use crate::Error;
use fxhash::FxHashMap;

#[derive(Debug, Clone)]
pub(crate) struct DrawDirective {
    pub(crate) shader_name: Ident,
    pub(crate) vertex_buffer: ShaderArgValue,
    pub(crate) args: FxHashMap<String, ShaderArg>,
    pub(crate) code: String,
    pub(crate) priority: i32,
}

impl DrawDirective {
    pub(crate) fn parse(lexer: &mut Lexer<'_>, hashtag: &Token<'_>) -> Result<Self, Error> {
        let priority = common::parse_priority(lexer)?;
        let shader_name = Ident::parse(lexer)?;
        lexer.next_expected(&[TokenKind::OpenAngleBracket])?;
        let vertex_buffer = ShaderArgValue::parse(lexer)?;
        lexer.next_expected(&[TokenKind::CloseAngleBracket])?;
        let args = common::parse_shader_args(lexer)?;
        Ok(Self {
            shader_name,
            vertex_buffer,
            args,
            code: lexer
                .source_slice(hashtag.span.start..lexer.offset())
                .into(),
            priority,
        })
    }
}
