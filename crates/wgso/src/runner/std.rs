use std::time::{Instant, SystemTime, UNIX_EPOCH};
use winit::event::ElementState;
use winit::keyboard::KeyCode;

#[derive(Debug, Default)]
pub(crate) struct StdState {
    pub(crate) time: StdTimeState,
    pub(crate) surface: SurfaceState,
    pub(crate) keyboard: StdKeyboardState,
}

#[derive(Debug)]
pub(crate) struct StdTimeState {
    pub(crate) frame_delta_secs: f32,
    frame_index: u32,
    start_secs: u32,
    last_frame_end: Instant,
}

impl Default for StdTimeState {
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

impl StdTimeState {
    pub(crate) fn data(&self) -> Vec<u8> {
        self.frame_delta_secs
            .to_ne_bytes()
            .into_iter()
            .chain(self.frame_index.to_ne_bytes())
            .chain(self.start_secs.to_ne_bytes())
            .collect()
    }

    pub(crate) fn update(&mut self) {
        let now = Instant::now();
        self.frame_delta_secs = (now - self.last_frame_end).as_secs_f32();
        self.last_frame_end = now;
        self.frame_index += 1;
    }
}

#[derive(Debug, Default)]
pub(crate) struct SurfaceState {
    size: (u32, u32),
}

impl SurfaceState {
    pub(crate) fn data(&self) -> Vec<u8> {
        self.size
            .0
            .to_ne_bytes()
            .into_iter()
            .chain(self.size.1.to_ne_bytes())
            .collect()
    }

    pub(crate) fn update(&mut self, size: (u32, u32)) {
        self.size = size;
    }
}

// coverage: off (not easy to test)

const KEYBOARD_KEY_COUNT: usize = KeyCode::F35 as usize;
const KEYBOARD_KEY_DATA_SIZE: usize =
    KEYBOARD_KEY_COUNT * size_of::<u32>().div_euclid(size_of::<u8>());

#[derive(Debug)]
pub(crate) struct StdKeyboardState {
    keys: [InputState; KEYBOARD_KEY_COUNT],
}

impl Default for StdKeyboardState {
    fn default() -> Self {
        Self {
            keys: [InputState::default(); KEYBOARD_KEY_COUNT],
        }
    }
}

impl StdKeyboardState {
    pub(crate) fn data(&self) -> [u8; KEYBOARD_KEY_DATA_SIZE] {
        let mut data = [0; KEYBOARD_KEY_DATA_SIZE];
        for (key_index, key_state) in self.keys.iter().enumerate() {
            data[key_index * 4..(key_index + 1) * 4].copy_from_slice(&key_state.data());
        }
        data
    }

    pub(crate) fn update_key(&mut self, key: KeyCode, state: ElementState) {
        let key_index = key as usize;
        match state {
            ElementState::Pressed => self.keys[key_index].press(),
            ElementState::Released => self.keys[key_index].release(),
        }
    }

    pub(crate) fn refresh(&mut self) {
        for key in &mut self.keys {
            key.refresh();
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct InputState {
    data: u32,
}

impl InputState {
    const IS_PRESSED_BIT: u8 = 0;
    const IS_JUST_PRESSED_BIT: u8 = 1;
    const IS_JUST_RELEASED_BIT: u8 = 2;

    fn data(self) -> [u8; 4] {
        self.data.to_ne_bytes()
    }

    fn press(&mut self) {
        if !self.bit(Self::IS_PRESSED_BIT) {
            self.set_bit(Self::IS_PRESSED_BIT);
            self.set_bit(Self::IS_JUST_PRESSED_BIT);
        }
    }

    fn release(&mut self) {
        if self.bit(Self::IS_PRESSED_BIT) {
            self.clear_bit(Self::IS_PRESSED_BIT);
            self.set_bit(Self::IS_JUST_RELEASED_BIT);
        }
    }

    fn refresh(&mut self) {
        self.clear_bit(Self::IS_JUST_PRESSED_BIT);
        self.clear_bit(Self::IS_JUST_RELEASED_BIT);
    }

    fn bit(self, position: u8) -> bool {
        (self.data & (1 << position)) != 0
    }

    fn set_bit(&mut self, position: u8) {
        self.data |= 1 << position;
    }

    fn clear_bit(&mut self, position: u8) {
        self.data &= !(1 << position);
    }
}
