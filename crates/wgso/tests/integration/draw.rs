use wgso::Runner;

#[test]
fn test_empty_surface() {
    let mut runner = Runner::new("tests/cases_valid/storages", None, Some((4, 3))).unwrap();
    assert_eq!(
        runner.read_target(),
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // row 1
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // row 2
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // row 3
        ]
    );
    runner.run_step().unwrap();
    assert_eq!(
        runner.read_target(),
        vec![
            0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, // row 1
            0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, // row 2
            0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, // row 3
        ]
    );
}
