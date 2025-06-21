//! WGSO Android runner.

// coverage: off (not easy to test)

/// Android entrypoint.
#[cfg(target_os = "android")]
#[allow(unsafe_code)]
#[no_mangle]
pub extern "Rust" fn android_main(app: android_activity::AndroidApp) {
    static PROJECT_DIR: include_dir::Dir<'_> = include_dir::include_dir!("$PROGRAM_PATH");
    let args = wgso::RunArgs {
        path: "".into(),
        buffer: vec![],
        fps: false,
    };
    args.run_android(app, PROJECT_DIR.clone());
}
