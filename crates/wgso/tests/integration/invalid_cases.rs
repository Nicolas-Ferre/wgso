use itertools::Itertools;
use std::fs;
use std::path::PathBuf;
use wgso::Runner;

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
    let errors = match Runner::new(&path) {
        Ok(mut runner) => runner
            .run_step()
            .expect_err("invalid code has successfully compiled")
            .render_errors(),
        Err(program) => program.render_errors(),
    };
    let actual = String::from_utf8(strip_ansi_escapes::strip(errors)).unwrap();
    let error_path = path.join(".expected");
    if error_path.exists() {
        assert_eq!(
            fs::read_to_string(error_path).unwrap(),
            actual,
            "mismatching result for invalid {:?} case",
            path.file_name().unwrap(),
        );
    } else {
        fs::write(error_path, actual).unwrap();
        panic!("expected error saved on disk, please check and rerun the tests");
    }
}
