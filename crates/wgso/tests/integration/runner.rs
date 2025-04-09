use wgso::{Error, Runner};

#[test]
fn run_invalid_directory_path() {
    let program = Runner::new("invalid_path").expect_err("invalid path has not returned error");
    assert_eq!(program.errors.len(), 1);
    assert!(matches!(program.errors[0], Error::Io(_, _)));
    assert!(program.errors[0]
        .render(&program)
        .contains("invalid_path: No such file or directory"));
}

#[test]
fn retrieve_not_existing_buffer() {
    let runner = Runner::new("tests/cases_valid/storages").unwrap();
    assert_eq!(runner.read("invalid"), vec![]);
}
