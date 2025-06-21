#[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
#[derive(Debug)]
pub(crate) struct RunnerWatcher {
    _watcher: notify::RecommendedWatcher,
    watcher_events: std::sync::mpsc::Receiver<notify::Result<notify::Event>>,
    next_update: Option<web_time::Instant>,
}

#[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
impl RunnerWatcher {
    const SECONDS_BEFORE_RELOADING: u64 = 2;

    pub(crate) fn new(folder_path: &std::path::Path) -> Self {
        use notify::Watcher;
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::RecommendedWatcher::new(tx, notify::Config::default())
            .expect("cannot initialize watcher");
        watcher
            .watch(folder_path, notify::RecursiveMode::Recursive)
            .expect("cannot watch program directory");
        Self {
            _watcher: watcher,
            watcher_events: rx,
            next_update: None,
        }
    }

    pub(crate) fn detect_changes(&mut self) -> bool {
        let is_updated = self.watcher_events.try_iter().any(|event| match event {
            Ok(event) => event.kind.is_create() || event.kind.is_modify() || event.kind.is_remove(),
            Err(_) => false, // no-coverage (not easy to test)
        });
        if is_updated {
            self.next_update = Some(
                web_time::Instant::now()
                    + std::time::Duration::from_secs(Self::SECONDS_BEFORE_RELOADING),
            );
        }
        if self
            .next_update
            .is_some_and(|next_update| web_time::Instant::now() >= next_update)
        {
            self.next_update = None;
            true
        } else {
            false
        }
    }
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
#[derive(Debug)]
pub(crate) struct RunnerWatcher {}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
impl RunnerWatcher {
    pub(crate) fn new(_folder_path: &std::path::Path) -> Self {
        Self {}
    }

    #[allow(clippy::unused_self, clippy::needless_pass_by_ref_mut)]
    pub(crate) fn detect_changes(&mut self) -> bool {
        false
    }
}
