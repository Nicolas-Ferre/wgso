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

fn storage_regex(name: &str) -> Regex {
    Regex::new(&format!(r"var\s*<\s*storage[a-z_,\s]*>\s*({name})\s*:"))
        .expect("internal error: invalid storage regex")
}
