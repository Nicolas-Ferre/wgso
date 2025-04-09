#![allow(missing_docs)]

fn main() {
    println!("cargo::rerun-if-changed=tests/cases_valid");
    println!("cargo::rerun-if-changed=tests/cases_invalid");
}
