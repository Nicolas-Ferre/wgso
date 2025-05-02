use wgso::{Error, Runner};

#[test]
fn run_invalid_directory_path() {
    let program =
        Runner::new("invalid_path", None, None).expect_err("invalid path has not returned error");
    assert_eq!(program.errors.len(), 1);
    assert!(matches!(program.errors[0], Error::Io(_, _)));
    assert!(program.errors[0]
        .render(&program)
        .contains("invalid_path: No such file or directory"));
}

#[test]
fn retrieve_not_existing_buffer() {
    let runner = Runner::new("tests/cases_valid/storages", None, None).unwrap();
    assert!(runner.read("invalid").is_empty());
}

#[test]
fn read_valid_buffer_field() {
    let mut runner = Runner::new("tests/cases_valid/uniforms", None, None).unwrap();
    runner.run_step().unwrap();
    assert_eq!(runner.read("modes.inner.mode1"), vec![1, 0, 0, 0]);
}

#[test]
fn read_invalid_buffer_field() {
    let runner = Runner::new("tests/cases_valid/uniforms", None, None).unwrap();
    assert!(runner.read("modes.invalid").is_empty());
}

#[test]
fn run_init_shaders_only_once() {
    let mut runner = Runner::new("tests/cases_valid/ordering", None, None).unwrap();
    runner.run_step().unwrap();
    runner.run_step().unwrap();
    assert_eq!(runner.read("buffer"), vec![65, 0, 0, 0]);
}
