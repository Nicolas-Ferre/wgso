use std::path::PathBuf;
use wgso::Runner;

#[rstest::rstest]
fn run_valid_code(#[files("../../examples/*")] path: PathBuf) {
    wgso_deps::retrieve_dependencies(path.join("wgso.yaml")).unwrap();
    let mut runner = Runner::new(&path, None, Some((10, 8))).unwrap();
    runner.run_step().unwrap();
}
