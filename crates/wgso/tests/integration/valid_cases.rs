use itertools::Itertools;
use std::fs;
use std::path::PathBuf;
use wgso::Runner;

#[rstest::rstest]
fn run_valid_code(#[files("./tests/cases_valid/*")] path: PathBuf) {
    if path.join("_").is_dir() {
        fs::remove_dir_all(path.join("_")).unwrap();
    }
    wgso_deps::retrieve_dependencies(path.join("wgso.yaml")).unwrap();
    let mut runner = Runner::new(&path, None, Some((10, 8))).unwrap();
    runner.run_step().unwrap();
    let target_buffer = runner.read_target();
    let target_buffer_str = target_buffer
        .chunks(10 * 4)
        .map(|row| {
            format!(
                "    {}",
                row.chunks(4)
                    .map(|row| format!(
                        "{:02X}{:02X}{:02X}{:02X}, ",
                        row[0], row[1], row[2], row[3]
                    ))
                    .join("")
            )
        })
        .join("\n");
    let mut buffers = runner
        .buffers()
        .map(|buffer| {
            if buffer == "std_" {
                format!(
                    "std_.time.frame_index={:?}",
                    runner.read("std_.time.frame_index")
                )
            } else {
                format!("{buffer}={:?}", runner.read(buffer))
            }
        })
        .collect::<Vec<_>>();
    buffers.sort_unstable();
    buffers.insert(0, format!("target=[\n{target_buffer_str}\n]"));
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
