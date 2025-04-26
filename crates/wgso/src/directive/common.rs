use crate::directive::token::{Ident, Lexer, TokenKind};
use crate::Error;
use fxhash::FxHashMap;
use std::collections::hash_map::Entry;
use std::ops::Range;

pub(crate) fn parse_priority(lexer: &mut Lexer<'_>) -> Result<i32, Error> {
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

pub(crate) fn parse_shader_args(
    lexer: &mut Lexer<'_>,
) -> Result<FxHashMap<String, ShaderArg>, Error> {
    lexer.next_expected(&[TokenKind::OpenParenthesis])?;
    let mut args = FxHashMap::default();
    let close_parenthesis = &[TokenKind::CloseParenthesis];
    while lexer.clone().next_expected(close_parenthesis).is_err() {
        if !args.is_empty() {
            lexer.next_expected(&[TokenKind::Comma])?;
        }
        let arg = ShaderArg::parse(lexer)?;
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
    Ok(args)
}

#[derive(Debug, Clone)]
pub(crate) struct ShaderArg {
    pub(crate) name: Ident,
    pub(crate) value: ShaderArgValue,
}

impl ShaderArg {
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, Error> {
        let name = Ident::parse(lexer)?;
        lexer.next_expected(&[TokenKind::Equal])?;
        let value = ShaderArgValue::parse(lexer)?;
        Ok(Self { name, value })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ShaderArgValue {
    pub(crate) buffer_name: Ident,
    pub(crate) fields: Vec<Ident>,
}

impl ShaderArgValue {
    pub(crate) fn span(&self) -> Range<usize> {
        let end = self
            .fields
            .last()
            .map_or(self.buffer_name.span.end, |field| field.span.end);
        self.buffer_name.span.start..end
    }

    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, Error> {
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
