use crate::directive::tokens::{Ident, Lexer, Token, TokenKind};
use crate::Error;
use fxhash::FxHashMap;
use std::collections::hash_map::Entry;
use std::ops::Range;

#[derive(Debug, Clone)]
pub(crate) struct RunDirective {
    pub(crate) name: Ident,
    pub(crate) args: FxHashMap<String, RunArg>,
    pub(crate) code: String,
    pub(crate) is_init: bool,
    pub(crate) priority: i32,
}

impl RunDirective {
    pub(crate) fn parse(
        lexer: &mut Lexer<'_>,
        hashtag: &Token<'_>,
        is_init: bool,
    ) -> Result<Self, Error> {
        let priority = Self::parse_priority(lexer)?;
        let name = Ident::parse(lexer)?;
        lexer.next_expected(&[TokenKind::OpenParenthesis])?;
        let mut args = FxHashMap::default();
        let close_parenthesis = &[TokenKind::CloseParenthesis];
        while lexer.clone().next_expected(close_parenthesis).is_err() {
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
        lexer.next_expected(close_parenthesis)?;
        Ok(Self {
            name,
            args,
            code: lexer
                .source_slice(hashtag.span.start..lexer.offset())
                .into(),
            is_init,
            priority,
        })
    }

    fn parse_priority(lexer: &mut Lexer<'_>) -> Result<i32, Error> {
        let open_bracket = &[TokenKind::OpenAngleBracket];
        if lexer.clone().next_expected(open_bracket).is_ok() {
            lexer.next_expected(open_bracket)?;
            let path = lexer.path().to_path_buf();
            let priority = lexer.next_expected(&[TokenKind::Integer])?;
            let priority_value = priority.slice.parse::<i32>().map_err(|_| {
                Error::DirectiveParsing(
                    path,
                    priority.span,
                    "priority is not a valid `i32` value".into(),
                )
            })?;
            lexer.next_expected(&[TokenKind::CloseAngleBracket])?;
            Ok(priority_value)
        } else {
            Ok(0)
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RunArg {
    pub(crate) name: Ident,
    pub(crate) value: RunArgValue,
}

impl RunArg {
    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, Error> {
        let name = Ident::parse(lexer)?;
        lexer.next_expected(&[TokenKind::Equal])?;
        let value = RunArgValue::parse(lexer)?;
        Ok(Self { name, value })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RunArgValue {
    pub(crate) buffer_name: Ident,
    pub(crate) fields: Vec<Ident>,
}

impl RunArgValue {
    pub(crate) fn span(&self) -> Range<usize> {
        let end = self
            .fields
            .last()
            .map_or(self.buffer_name.span.end, |field| field.span.end);
        self.buffer_name.span.start..end
    }

    fn parse(lexer: &mut Lexer<'_>) -> Result<Self, Error> {
        let buffer_name = Ident::parse(lexer)?;
        let mut fields = vec![];
        while lexer.clone().next_expected(&[TokenKind::Dot]).is_ok() {
            lexer.next_expected(&[TokenKind::Dot])?;
            fields.push(Ident::parse(lexer)?);
        }
        Ok(Self {
            buffer_name,
            fields,
        })
    }
}
