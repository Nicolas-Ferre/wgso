//! WGSO web runner.

// coverage: off (not easy to test)

#[cfg(target_arch = "wasm32")]
fn main() {
    static PROJECT_DIR: include_dir::Dir<'_> = include_dir::include_dir!("$PROGRAM_PATH");
    let args = wgso::RunArgs {
        path: "".into(),
        buffer: vec![],
        fps: false,
    };
    args.run_web(PROJECT_DIR.clone());
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
