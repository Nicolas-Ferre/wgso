use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

const SECONDS_BEFORE_RELOADING: u64 = 2;

#[derive(Debug)]
pub(crate) struct RunnerWatcher {
    _watcher: RecommendedWatcher,
    watcher_events: Receiver<notify::Result<Event>>,
    next_update: Option<Instant>,
}

impl RunnerWatcher {
    pub(crate) fn new(folder_path: &Path) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher =
            RecommendedWatcher::new(tx, Config::default()).expect("cannot initialize watcher");
        watcher
            .watch(folder_path, RecursiveMode::Recursive)
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
            self.next_update = Some(Instant::now() + Duration::from_secs(SECONDS_BEFORE_RELOADING));
        }
        if self
            .next_update
            .is_some_and(|next_update| Instant::now() >= next_update)
        {
            self.next_update = None;
            true
        } else {
            false
        }
    }
}
