use crate::directive::tokens::{Ident, Lexer, Token, TokenKind};
use crate::Error;

#[derive(Debug, Clone)]
pub(crate) struct ShaderDirective {
    pub(crate) name: Ident,
    pub(crate) code: String,
}

impl ShaderDirective {
    pub(crate) fn parse(lexer: &mut Lexer<'_>, hashtag: &Token<'_>) -> Result<Self, Error> {
        lexer.next_expected(&[TokenKind::OpenAngleBracket])?;
        lexer.next_expected(&[TokenKind::Compute])?;
        lexer.next_expected(&[TokenKind::CloseAngleBracket])?;
        Ok(Self {
            name: Ident::parse(lexer)?,
            code: lexer
                .source_slice(hashtag.span.start..lexer.offset())
                .into(),
        })
    }
}
