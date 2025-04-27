use std::fmt::Debug;
use std::mem;
use std::path::Path;
use std::str::FromStr;
use wgso_parser::{ParsingError, Rule, Token};

pub(crate) fn load_rules() -> Vec<Rule> {
    wgso_parser::load_rules(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/config/directives.yaml"
    )))
    .expect("internal error: directive config should be valid")
}

pub(crate) fn parse_code(
    code: &str,
    path: &Path,
    rules: &[Rule],
    errors: &mut Vec<ParsingError>,
) -> Vec<Vec<Token>> {
    let mut parsed_directives = vec![];
    let mut offset = 0;
    for line in code.lines() {
        if let Some(directive) = line.trim_start().strip_prefix("#") {
            let current_offset = offset + line.len() - directive.len();
            match wgso_parser::parse(directive, current_offset, path, rules) {
                Ok(tokens) => parsed_directives.push(tokens),
                Err(error) => errors.push(error),
            }
        }
        offset += line.len();
    }
    parsed_directives
}

pub(crate) fn kind(directive: &[Token]) -> DirectiveKind {
    match directive[0].slice.as_str() {
        "shader" => match directive[2].slice.as_str() {
            "compute" => DirectiveKind::ComputeShader,
            "render" => DirectiveKind::RenderShader,
            _ => unreachable!("internal error: unrecognized shader directive"),
        },
        "init" => DirectiveKind::Init,
        "run" => DirectiveKind::Run,
        "draw" => DirectiveKind::Draw,
        _ => unreachable!("internal error: unrecognized directive"),
    }
}

pub(crate) fn shader_name(directive: &[Token]) -> &Token {
    find_one_by_label(directive, "shader_name")
}

pub(crate) fn vertex_buffer(directive: &[Token]) -> &Token {
    assert_eq!(kind(directive), DirectiveKind::Draw);
    find_one_by_label(directive, "vertex_buffer")
}

pub(crate) fn vertex_type(directive: &[Token]) -> &Token {
    assert_eq!(kind(directive), DirectiveKind::RenderShader);
    find_one_by_label(directive, "vertex_type")
}

pub(crate) fn workgroup_count(directive: &[Token]) -> (u16, u16, u16) {
    assert_eq!(kind(directive), DirectiveKind::ComputeShader);
    let mut tokens = find_all_by_label(directive, "workgroup_count");
    let workgroup_count_x = tokens.next().map_or(1, convert_to_integer);
    let workgroup_count_y = tokens.next().map_or(1, convert_to_integer);
    let workgroup_count_z = tokens.next().map_or(1, convert_to_integer);
    (workgroup_count_x, workgroup_count_y, workgroup_count_z)
}

pub(crate) fn priority(directive: &[Token]) -> i32 {
    assert!(matches!(
        kind(directive),
        DirectiveKind::Init | DirectiveKind::Run | DirectiveKind::Draw
    ));
    convert_to_integer(find_one_by_label(directive, "priority"))
}

pub(crate) fn args(directive: &[Token]) -> Vec<DirectiveArg> {
    assert!(matches!(
        kind(directive),
        DirectiveKind::Init | DirectiveKind::Run | DirectiveKind::Draw
    ));
    let mut args = vec![];
    let mut current_name = None;
    let mut current_var = None;
    let mut current_field = vec![];
    for token in directive {
        match token.label.as_deref() {
            Some("arg_name") => current_name = Some(token.clone()),
            Some("arg_var") => current_var = Some(token.clone()),
            Some("arg_field") => current_field.push(token.clone()),
            _ => {
                if let (Some(name), Some(var)) = (current_name.take(), current_var.take()) {
                    args.push(DirectiveArg {
                        name,
                        var,
                        fields: mem::take(&mut current_field),
                    });
                }
            }
        }
    }
    args
}

fn find_one_by_label<'a>(directive: &'a [Token], label: &str) -> &'a Token {
    directive
        .iter()
        .find(|token| token.label.as_deref() == Some(label))
        .expect("internal error: cannot find directive token by label")
}

fn find_all_by_label<'a>(
    directive: &'a [Token],
    label: &'a str,
) -> impl Iterator<Item = &'a Token> {
    directive
        .iter()
        .filter(|token| token.label.as_deref() == Some(label))
}

fn convert_to_integer<T>(token: &Token) -> T
where
    T: FromStr,
    T::Err: Debug,
{
    token
        .slice
        .parse::<T>()
        .expect("internal error: directive integers should be validated")
}

pub(crate) struct DirectiveArg {
    pub(crate) name: Token,
    pub(crate) var: Token,
    pub(crate) fields: Vec<Token>,
}

#[derive(Debug, PartialEq, Eq)]
enum DirectiveKind {
    ComputeShader,
    RenderShader,
    Init,
    Run,
    Draw,
}

// TODO: remove
#[test]
fn test() {
    let rules = load_rules();
    let code = "#shader<compute> shadername\nokokokok\n    #   run<-999999> aa(a=b.e.f, c=d)";
    let mut errors = vec![];
    dbg!(parse_code(
        code,
        Path::new("filename.wgsl"),
        &rules,
        &mut errors
    ));
    dbg!(errors);
}
