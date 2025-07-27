use std::path::Path;
use wgso::Runner;

#[test]
fn test_toggle() {
    wgso_deps::retrieve_dependencies("tests/cases_valid/toggle/wgso.yaml").unwrap();
    let mut runner =
        Runner::new(Path::new("tests/cases_valid/toggle"), None, Some((4, 3))).unwrap();
    runner.run_step().unwrap();
    assert_eq!(runner.read("is_toggle_enabled"), vec![1, 0, 0, 0]);
    assert_eq!(runner.read("state"), vec![0, 0, 0, 0]);
    assert!(runner.read("toggle_state").is_empty());
    assert_eq!(runner.read_target()[0], 0);
    runner.run_step().unwrap();
    assert_eq!(runner.read("is_toggle_enabled"), vec![0, 0, 0, 0]);
    assert_eq!(runner.read("state"), vec![1, 0, 0, 0]);
    assert_eq!(runner.read("toggle_state"), vec![1, 0, 0, 0]);
    assert_eq!(runner.read_target()[0], 255);
    runner.run_step().unwrap();
    assert_eq!(runner.read("is_toggle_enabled"), vec![1, 0, 0, 0]);
    assert_eq!(runner.read("state"), vec![1, 0, 0, 0]);
    assert!(runner.read("toggle_state").is_empty());
    assert_eq!(runner.read_target()[0], 0);
}
