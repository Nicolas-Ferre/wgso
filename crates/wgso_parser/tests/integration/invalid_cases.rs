use itertools::Itertools;
use std::fs;
use std::path::{Path, PathBuf};

#[rstest::rstest]
fn run_invalid_code(#[files("./tests/cases_invalid/*")] path: PathBuf) {
    let path = PathBuf::from(format!(
        // make error paths relative
        "./tests/cases_invalid/{}",
        path.components()
            .skip(path.components().count() - 1)
            .map(|a| a.as_os_str().to_str().unwrap())
            .join("/")
    ));
    let rules_bytes = fs::read(path.join("rules.yaml")).unwrap();
    let code = fs::read_to_string(path.join("code")).unwrap();
    let error = match wgso_parser::load_rules(&rules_bytes) {
        Ok(rules) => {
            let error = wgso_parser::parse(&code, 0, Path::new("path"), &rules)
                .expect_err("invalid case has successfully finished");
            format!("{error}")
        }
        Err(error) => {
            format!("{error}")
        }
    };
    let error_path = path.join(".expected");
    if error_path.exists() {
        assert_eq!(
            fs::read_to_string(error_path).unwrap(),
            error,
            "mismatching result for invalid {:?} case",
            path.file_name().unwrap(),
        );
    } else {
        fs::write(error_path, error).unwrap();
        panic!("expected error saved on disk, please check and rerun the tests");
    }
}
