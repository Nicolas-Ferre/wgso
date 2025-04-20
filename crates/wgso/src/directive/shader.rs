use crate::directive::tokens::{Ident, Lexer, Token, TokenKind};
use crate::Error;

#[derive(Debug, Clone)]
pub(crate) struct ShaderDirective {
    pub(crate) name: Ident,
    pub(crate) code: String,
    pub(crate) workgroup_count_x: u16,
    pub(crate) workgroup_count_y: u16,
    pub(crate) workgroup_count_z: u16,
}

impl ShaderDirective {
    pub(crate) fn parse(lexer: &mut Lexer<'_>, hashtag: &Token<'_>) -> Result<Self, Error> {
        lexer.next_expected(&[TokenKind::OpenAngleBracket])?;
        lexer.next_expected(&[TokenKind::Compute])?;
        let workgroup_count_x = Self::parse_workgroup_count(lexer)?;
        let workgroup_count_y = Self::parse_workgroup_count(lexer)?;
        let workgroup_count_z = Self::parse_workgroup_count(lexer)?;
        lexer.next_expected(&[TokenKind::CloseAngleBracket])?;
        Ok(Self {
            name: Ident::parse(lexer)?,
            code: lexer
                .source_slice(hashtag.span.start..lexer.offset())
                .into(),
            workgroup_count_x,
            workgroup_count_y,
            workgroup_count_z,
        })
    }

    fn parse_workgroup_count(lexer: &mut Lexer<'_>) -> Result<u16, Error> {
        if lexer.clone().next_expected(&[TokenKind::Comma]).is_ok() {
            lexer.next_expected(&[TokenKind::Comma])?;
            let path = lexer.path().to_path_buf();
            let count = lexer.next_expected(&[TokenKind::Integer])?;
            Ok(count.slice.parse::<u16>().map_err(|_| {
                Error::DirectiveParsing(
                    path,
                    count.span,
                    "workgroup count is not a valid `u16` value".into(),
                )
            })?)
        } else {
            Ok(1)
        }
    }
}
