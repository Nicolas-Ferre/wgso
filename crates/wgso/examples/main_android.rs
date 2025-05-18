//! WGSO Android runner.

// coverage: off (not easy to test)

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: android_activity::AndroidApp) {
    static PROJECT_DIR: include_dir::Dir<'_> = include_dir::include_dir!("$PROGRAM_PATH");
    let program_path = app
        .internal_data_path()
        .expect("inaccessible internal data path")
        .join("program");
    if program_path.is_dir() {
        std::fs::remove_dir_all(&program_path).expect("previous program removal failed");
    }
    std::fs::create_dir_all(&program_path).expect("program folder creation failed");
    PROJECT_DIR
        .extract(&program_path)
        .expect("program extraction failed");
    let args = wgso::RunArgs {
        path: program_path,
        buffer: vec![],
        fps: false,
    };
    args.run_android(app);
}
