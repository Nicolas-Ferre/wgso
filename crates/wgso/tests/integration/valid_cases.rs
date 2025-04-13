use std::fs;
use std::path::PathBuf;
use wgso::Runner;

#[rstest::rstest]
fn run_valid_code(#[files("./tests/cases_valid/*")] path: PathBuf) {
    let mut runner = Runner::new(&path).unwrap();
    runner.run_step().unwrap();
    let mut buffers = runner
        .buffers()
        .map(|buffer| format!("{buffer}={:?}", runner.read(buffer)))
        .collect::<Vec<_>>();
    buffers.sort_unstable();
    let actual = buffers.join("\n");
    let buffers_path = path.join(".expected");
    if buffers_path.exists() {
        assert_eq!(
            fs::read_to_string(buffers_path).unwrap(),
            actual,
            "mismatching result for valid {:?} case",
            path.file_stem().unwrap(),
        );
    } else {
        fs::write(buffers_path, actual).unwrap();
        panic!("expected buffers saved on disk, please check and rerun the tests");
    }
}
