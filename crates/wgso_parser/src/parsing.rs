use crate::rules::Rule;
use crate::{ChoiceRule, ParsingError, PatternRule, RepeatRule};
use itertools::Itertools;
use std::ops::Range;
use std::path::{Path, PathBuf};

/// Parse a source string located in a given file path and starting at a given offset.
///
/// # Errors
///
/// An error is returned if the source cannot be parsed with using rules.
pub fn parse(
    source: &str,
    offset: usize,
    path: &Path,
    rules: &[Rule],
) -> Result<Vec<Token>, ParsingError> {
    let mut ctx = Context {
        path,
        remaining_source: source,
        initial_len: source.len(),
        initial_offset: offset,
    };
    let rules = parse_rules(&mut ctx, rules).map_err(|(error, _)| error)?;
    ctx.remaining_source = ctx.remaining_source.trim_start();
    if ctx.remaining_source.is_empty() {
        Ok(rules)
    } else {
        Err(ParsingError {
            path: path.into(),
            span: ctx.offset()..ctx.offset() + ctx.remaining_source.len(),
            message: "unexpected tokens".into(),
        })
    }
}

fn parse_rules(ctx: &mut Context<'_>, rules: &[Rule]) -> Result<Vec<Token>, (ParsingError, bool)> {
    let rules = rules
        .iter()
        .map(|rule| parse_rule(ctx, rule))
        .collect::<Vec<_>>();
    let first_token_parsed = rules[0].is_ok();
    Ok(rules
        .into_iter()
        .collect::<Result<Vec<Vec<Token>>, ParsingError>>()
        .map_err(|error| (error, first_token_parsed))?
        .into_iter()
        .flatten()
        .collect())
}

fn parse_rule(ctx: &mut Context<'_>, rule: &Rule) -> Result<Vec<Token>, ParsingError> {
    match rule {
        Rule::Token(token) => parse_token(ctx, token),
        Rule::Pattern(rule) => parse_pattern(ctx, rule),
        Rule::Repeat(rule) => parse_repeat(ctx, rule),
        Rule::Choice(choices) => parse_choice(ctx, choices),
    }
}

fn parse_token(ctx: &mut Context<'_>, token: &str) -> Result<Vec<Token>, ParsingError> {
    ctx.remaining_source = ctx.remaining_source.trim_start();
    if let Some(remaining_source) = ctx.remaining_source.strip_prefix(token) {
        let is_alphanum_token = token
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric());
        let is_next_char_alphanum = remaining_source
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric());
        if !is_alphanum_token || !is_next_char_alphanum {
            let span_start = ctx.offset();
            ctx.remaining_source = remaining_source;
            return Ok(vec![Token {
                slice: token.into(),
                label: None,
                span: span_start..ctx.offset(),
                path: ctx.path.into(),
            }]);
        }
    }
    Err(parsing_error(
        ctx,
        &format!("`{token}`"),
        ctx.offset()..ctx.offset(),
    ))
}

fn parse_pattern(ctx: &mut Context<'_>, rule: &PatternRule) -> Result<Vec<Token>, ParsingError> {
    ctx.remaining_source = ctx.remaining_source.trim_start();
    let initial_offset = ctx.offset();
    let is_integer = rule.config.min.is_some() || rule.config.max.is_some();
    let chat_condition = |char: char, char_index| {
        (is_integer && (char.is_ascii_digit() || (char_index == 0 && char == '-')))
            || (!is_integer
                && char.is_ascii_alphanumeric()
                && (rule.config.is_digit_prefix_allowed.unwrap_or(true)
                    || char_index > 0
                    || char.is_ascii_alphabetic()))
            || (rule.config.is_underscore_allowed.unwrap_or(false) && char == '_')
    };
    let token = parse_conditional(
        &mut ctx.clone(),
        &rule.label,
        &rule.config.label,
        chat_condition,
    )?;
    if is_integer {
        match token.slice.parse::<i128>() {
            Ok(value) => {
                if !(rule.config.min.map_or(true, |min| value >= min)
                    && rule.config.max.map_or(true, |max| value <= max))
                {
                    return Err(parsing_error(
                        ctx,
                        &rule.config.label,
                        initial_offset..initial_offset + token.slice.len(),
                    ));
                }
            }
            Err(_) => {
                return Err(parsing_error(
                    ctx,
                    &rule.config.label,
                    initial_offset..initial_offset + token.slice.len(),
                ))
            }
        }
    }
    Ok(vec![parse_conditional(
        ctx,
        &rule.label,
        &rule.config.label,
        chat_condition,
    )?])
}

fn parse_conditional(
    ctx: &mut Context<'_>,
    label: &str,
    type_: &str,
    is_valid_char: impl Fn(char, usize) -> bool,
) -> Result<Token, ParsingError> {
    let mut ident_len = 0;
    for char in ctx.remaining_source.chars() {
        if is_valid_char(char, ident_len) {
            ident_len += 1;
        } else {
            break;
        }
    }
    if ident_len == 0 {
        Err(parsing_error(ctx, type_, ctx.offset()..ctx.offset()))
    } else {
        let span_start = ctx.offset();
        let ident = &ctx.remaining_source[..ident_len];
        ctx.remaining_source = &ctx.remaining_source[ident_len..];
        Ok(Token {
            slice: ident.into(),
            label: Some(label.into()),
            span: span_start..ctx.offset(),
            path: ctx.path.into(),
        })
    }
}

fn parse_repeat(ctx: &mut Context<'_>, rule: &RepeatRule) -> Result<Vec<Token>, ParsingError> {
    let mut times = 0;
    let mut all_tokens = vec![];
    loop {
        if rule.max.is_some_and(|max| times >= max) {
            break;
        }
        match parse_rules(&mut ctx.clone(), &rule.group) {
            Ok(_) => all_tokens.extend(parse_rules(ctx, &rule.group).map_err(|(error, _)| error)?),
            Err((error, first_token_parsed)) => {
                if first_token_parsed || times < rule.min {
                    return Err(error);
                }
                break;
            }
        }
        times += 1;
    }
    Ok(all_tokens)
}

fn parse_choice(ctx: &mut Context<'_>, choices: &[ChoiceRule]) -> Result<Vec<Token>, ParsingError> {
    for choice in choices {
        if parse_token(&mut ctx.clone(), &choice.token).is_ok() {
            let mut token = parse_token(ctx, &choice.token)?;
            token.extend(parse_rules(ctx, &choice.next).map_err(|(error, _)| error)?);
            return Ok(token);
        }
    }
    let (last_choice, first_choices) = choices
        .split_last()
        .expect("internal error: there should be at least two choices");
    let expected_tokens = format!(
        "{} or `{}`",
        first_choices
            .iter()
            .map(|t| format!("`{}`", t.token))
            .join(", "),
        last_choice.token
    );
    Err(parsing_error(
        ctx,
        &expected_tokens,
        ctx.offset()..ctx.offset(),
    ))
}

fn parsing_error(ctx: &Context<'_>, token: &str, span: Range<usize>) -> ParsingError {
    ParsingError {
        path: ctx.path.into(),
        span,
        message: format!("expected {token}"),
    }
}

/// A parsed token.
#[derive(Debug, Clone)]
pub struct Token {
    /// The text corresponding to the token.
    pub slice: String,
    /// The label to identity the token more easily.
    pub label: Option<String>,
    /// The span of the token.
    pub span: Range<usize>,
    /// The file path of the token.
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
struct Context<'a> {
    path: &'a Path,
    remaining_source: &'a str,
    initial_len: usize,
    initial_offset: usize,
}

impl Context<'_> {
    fn offset(&self) -> usize {
        self.initial_len - self.remaining_source.len() + self.initial_offset
    }
}
