use std::sync::Mutex;
use std::time::Duration;
use std::{fs, thread};
use wgso::{Program, Runner};

const EXPECTED_DEFAULT_TARGET: &[u8] = &[
    0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, // row 1
    0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255, // row 2
    0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, // row 3
];
const EXPECTED_CHANGED_TARGET: &[u8] = &[
    0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, // row 1
    0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, // row 2
    0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, // row 3
];
const PROGRAM_PATH: &str = "tests/case_hot_reload";
const DRAW_WGSL_PATH: &str = "tests/case_hot_reload/draw.wgsl";
const STORAGES_WGSL_PATH: &str = "tests/case_hot_reload/storages.wgsl";

static MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn reload_with_valid_program() {
    let _lock = MUTEX.lock();
    let mut runner = Runner::new(PROGRAM_PATH, None, Some((4, 3))).unwrap();
    runner.reload_on_change().unwrap();
    runner.run_step().unwrap();
    assert_eq!(runner.read_target(), EXPECTED_DEFAULT_TARGET);
    let initial_code = fs::read_to_string(DRAW_WGSL_PATH).unwrap();
    let modified_code = initial_code.replace("vec4f(1, 1, 1, 1)", "vec4f(0, 0, 0, 0)");
    let reloading_result = update_code(&mut runner, &modified_code, DRAW_WGSL_PATH);
    let run_result = runner.run_step();
    fs::write(DRAW_WGSL_PATH, initial_code).unwrap();
    assert!(reloading_result.is_ok());
    assert!(run_result.is_ok());
    assert_eq!(runner.read_target(), EXPECTED_CHANGED_TARGET);
}

#[test]
fn reload_with_invalid_program() {
    let _lock = MUTEX.lock();
    let mut runner = Runner::new(PROGRAM_PATH, None, Some((4, 3))).unwrap();
    let initial_code = fs::read_to_string(DRAW_WGSL_PATH).unwrap();
    let modified_code = initial_code.replace("vec4f(1, 1, 1, 1)", "vec4f(1, 1, 1)");
    let reloading_result = update_code(&mut runner, &modified_code, DRAW_WGSL_PATH);
    let run_result = runner.run_step();
    fs::write(DRAW_WGSL_PATH, initial_code).unwrap();
    assert!(reloading_result.is_err());
    assert!(run_result.is_ok());
    assert_eq!(runner.read_target(), EXPECTED_DEFAULT_TARGET);
}

#[test]
fn reload_with_wgpu_error() {
    let _lock = MUTEX.lock();
    let mut runner = Runner::new(PROGRAM_PATH, None, Some((4, 3))).unwrap();
    let initial_code = fs::read_to_string(DRAW_WGSL_PATH).unwrap();
    let modified_code = initial_code.replace("#import ~.main", "#import ~.storages");
    let reloading_result = update_code(&mut runner, &modified_code, DRAW_WGSL_PATH);
    let run_result = runner.run_step();
    fs::write(DRAW_WGSL_PATH, initial_code).unwrap();
    assert!(reloading_result.is_err());
    assert!(run_result.is_ok());
    assert_eq!(runner.read_target(), EXPECTED_DEFAULT_TARGET);
}

#[test]
fn reload_with_changed_storage() {
    let _lock = MUTEX.lock();
    let mut runner = Runner::new(PROGRAM_PATH, None, Some((4, 3))).unwrap();
    let initial_code = fs::read_to_string(STORAGES_WGSL_PATH).unwrap();
    let modified_code = initial_code.replace("State", "ModifiedState");
    let reloading_result = update_code(&mut runner, &modified_code, STORAGES_WGSL_PATH);
    let run_result = runner.run_step();
    fs::write(STORAGES_WGSL_PATH, initial_code).unwrap();
    assert_eq!(
        reloading_result
            .expect_err("reloading should return an error")
            .render_errors(),
        "\u{1b}[1m\u{1b}[91merror\u{1b}[0m: \u{1b}[1mprogram cannot be hot-reloaded because storages have been changed\u{1b}[0m"
    );
    assert!(run_result.is_ok());
    assert_eq!(runner.read_target(), EXPECTED_DEFAULT_TARGET);
}

#[allow(clippy::result_large_err)]
fn update_code(runner: &mut Runner, code: &str, path: &str) -> Result<(), Program> {
    fs::write(path, code).unwrap();
    runner.reload_on_change()?;
    thread::sleep(Duration::from_millis(100));
    runner.reload_on_change()?;
    thread::sleep(Duration::from_millis(100));
    runner.reload_on_change()?;
    thread::sleep(Duration::from_secs(2));
    runner.reload_on_change()
}
