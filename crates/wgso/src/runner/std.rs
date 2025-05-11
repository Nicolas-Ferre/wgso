use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub(crate) struct StdState {
    pub(crate) frame_delta_secs: f32,
    frame_index: u32,
    start_secs: u32,
    last_frame_end: Instant,
}

impl Default for StdState {
    #[allow(clippy::cast_possible_truncation)]
    fn default() -> Self {
        Self {
            frame_delta_secs: 0.0,
            frame_index: 0,
            start_secs: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_secs() as u32),
            last_frame_end: Instant::now(),
        }
    }
}

impl StdState {
    pub(crate) fn time_data(&self) -> Vec<u8> {
        self.frame_delta_secs
            .to_ne_bytes()
            .into_iter()
            .chain(self.frame_index.to_ne_bytes())
            .chain(self.start_secs.to_ne_bytes())
            .collect()
    }

    pub(crate) fn update_time(&mut self) {
        let now = Instant::now();
        self.frame_delta_secs = (now - self.last_frame_end).as_secs_f32();
        self.last_frame_end = now;
        self.frame_index += 1;
    }
}
