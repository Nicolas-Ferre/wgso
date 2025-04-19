use crate::directive::tokens::{Ident, Lexer, Token, TokenKind};
use crate::Error;
use fxhash::FxHashMap;
use std::collections::hash_map::Entry;

#[derive(Debug, Clone)]
pub(crate) struct RunDirective {
    pub(crate) name: Ident,
    pub(crate) args: FxHashMap<String, RunArg>,
    pub(crate) code: String,
}

impl RunDirective {
    pub(crate) fn parse(lexer: &mut Lexer<'_>, hashtag: &Token<'_>) -> Result<Self, Error> {
        let name = Ident::parse(lexer)?;
        lexer.next_expected(&[TokenKind::OpenParenthesis])?;
        let mut args = FxHashMap::default();
        while lexer
            .clone()
            .next_expected(&[TokenKind::CloseParenthesis])
            .is_err()
        {
            if !args.is_empty() {
                lexer.next_expected(&[TokenKind::Comma])?;
            }
            let arg = RunArg::parse(lexer)?;
            match args.entry(arg.name.label.clone()) {
                Entry::Occupied(_) => {
                    return Err(Error::DirectiveParsing(
                        lexer.path().into(),
                        arg.name.span,
                        "duplicated parameter".into(),
                    ))
                }
                Entry::Vacant(entry) => {
                    entry.insert(arg);
                }
            }
        }
        lexer.next_expected(&[TokenKind::CloseParenthesis])?;
        Ok(Self {
            name,
            args,
            code: lexer
                .source_slice(hashtag.span.start..lexer.offset())
                .into(),
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RunArg {
    pub(crate) name: Ident,
    pub(crate) value: Ident,
}

impl RunArg {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, Error> {
        let name = Ident::parse(lexer)?;
        lexer.next_expected(&[TokenKind::Equal])?;
        let value = Ident::parse(lexer)?;
        Ok(Self { name, value })
    }
}
