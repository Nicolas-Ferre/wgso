use regex::Regex;
use std::ops::Range;

pub(crate) fn storage_name_span(code: &str, name: &str) -> Range<usize> {
    let var_regex_match = storage_regex(name)
        .captures(code)
        .expect("internal error: not found storage regex")
        .get(1)
        .expect("internal error: not found storage regex group");
    var_regex_match.start()..var_regex_match.end()
}

pub(crate) fn storage_var_start(code: &str, name: &str) -> usize {
    storage_regex(name)
        .find(code)
        .expect("internal error: not found storage regex")
        .start()
}

pub(crate) fn storage_var_type(code: &str, name: &str) -> String {
    storage_regex(name)
        .captures(code)
        .expect("internal error: not found storage regex")
        .get(2)
        .expect("internal error: not found storage regex group")
        .as_str()
        .replace(' ', "")
}

pub(crate) fn uniform_var_start(code: &str, name: &str) -> usize {
    uniform_regex(name)
        .find(code)
        .expect("internal error: not found uniform regex")
        .start()
}

pub(crate) fn uniform_var_type(code: &str, name: &str) -> String {
    uniform_regex(name)
        .captures(code)
        .expect("internal error: not found uniform regex")
        .get(2)
        .expect("internal error: not found uniform regex group")
        .as_str()
        .replace(' ', "")
}

fn storage_regex(name: &str) -> Regex {
    Regex::new(&format!(
        r"var\s*<\s*storage[a-z_,\s]*>\s*({name})\s*:([^=;]+)"
    ))
    .expect("internal error: invalid storage regex")
}

fn uniform_regex(name: &str) -> Regex {
    Regex::new(&format!(r"var\s*<\s*uniform\s*>\s*({name})\s*:([^=;]+)",))
        .expect("internal error: invalid uniform regex")
}
