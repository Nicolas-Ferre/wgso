use std::fs;
use std::path::{Path, PathBuf};

#[rstest::rstest]
fn run_valid_code(#[files("./tests/cases_valid/*")] path: PathBuf) {
    let rules_bytes = fs::read(path.join("rules.yaml")).unwrap();
    let code = fs::read_to_string(path.join("code")).unwrap();
    let rules = wgso_parser::load_rules(&rules_bytes).unwrap();
    let tokens = wgso_parser::parse(&code, 0, Path::new("path"), &rules);
    let actual = format!("{tokens:#?}");
    let expected_path = path.join(".expected");
    if expected_path.exists() {
        assert_eq!(
            fs::read_to_string(expected_path).unwrap(),
            actual,
            "mismatching result for valid {:?} case",
            path.file_stem().unwrap(),
        );
    } else {
        fs::write(expected_path, actual).unwrap();
        panic!("expected buffers saved on disk, please check and rerun the tests");
    }
}
