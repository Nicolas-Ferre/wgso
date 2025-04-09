use wgso::{Error, Runner};

#[test]
fn invalid_directory_path() {
    let error = Runner::new("invalid_path").expect_err("invalid path has not returned error");
    assert_eq!(error.errors.len(), 1);
    assert!(matches!(error.errors[0], Error::Io(_, _)));
}
