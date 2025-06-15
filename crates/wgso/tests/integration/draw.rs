use std::path::Path;
use wgso::Runner;

#[test]
fn test_empty_surface() {
    let runner = Runner::new(Path::new("tests/cases_valid/shaders"), None, Some((4, 3))).unwrap();
    assert_eq!(
        runner.read_target(),
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // row 1
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // row 2
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // row 3
        ]
    );
}
