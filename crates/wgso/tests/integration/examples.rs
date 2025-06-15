use std::fs;
use std::path::PathBuf;
use wgso::Runner;

#[rstest::rstest]
fn run_valid_code(#[files("../../examples/*")] path: PathBuf) {
    if path.join("_").is_dir() {
        fs::remove_dir_all(path.join("_")).unwrap();
    }
    wgso_deps::retrieve_dependencies(path.join("wgso.yaml")).unwrap();
    let mut runner = Runner::new(path.as_path(), None, Some((10, 8))).unwrap();
    runner.run_step().unwrap();
}
