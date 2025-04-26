use crate::directive::token::{Ident, Lexer, Token, TokenKind};
use crate::Error;

#[derive(Debug, Clone)]
pub(crate) struct RenderShaderDirective {
    pub(crate) code: String,
    pub(crate) shader_name: Ident,
    pub(crate) vertex_type_name: Ident,
}

impl RenderShaderDirective {
    pub(crate) fn parse(lexer: &mut Lexer<'_>, hashtag: &Token<'_>) -> Result<Self, Error> {
        lexer.next_expected(&[TokenKind::Comma])?;
        let vertex_type_name = Ident::parse(lexer)?;
        lexer.next_expected(&[TokenKind::CloseAngleBracket])?;
        let shader_name = Ident::parse(lexer)?;
        Ok(Self {
            code: lexer
                .source_slice(hashtag.span.start..lexer.offset())
                .into(),
            shader_name,
            vertex_type_name,
        })
    }
}
