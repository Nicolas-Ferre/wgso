use web_time::{Instant, SystemTime};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, Touch, TouchPhase};
use winit::keyboard::KeyCode;

#[derive(Debug, Default)]
pub(crate) struct StdState {
    pub(crate) time: StdTimeState,
    pub(crate) surface: SurfaceState,
    pub(crate) keyboard: StdKeyboardState,
    pub(crate) mouse: StdMouseState,
    pub(crate) touch: StdTouchState,
}

impl StdState {
    pub(crate) fn update(&mut self, surface_size: (u32, u32)) {
        self.surface.update(surface_size);
        self.keyboard.update();
        self.mouse.update();
        self.touch.update();
        self.time.update();
    }
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
                .duration_since(SystemTime::UNIX_EPOCH)
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

    fn update(&mut self) {
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

    fn update(&mut self, size: (u32, u32)) {
        self.size = size;
    }
}

// coverage: off (not easy to test)

const KEYBOARD_KEY_COUNT: usize = KeyCode::F35 as usize;

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
    pub(crate) fn data(&self) -> Vec<u8> {
        self.keys.iter().flat_map(|state| state.data()).collect()
    }

    pub(crate) fn update_key(&mut self, key: KeyCode, state: ElementState) {
        let key_index = key as usize;
        match state {
            ElementState::Pressed => self.keys[key_index].press(),
            ElementState::Released => self.keys[key_index].release(),
        }
    }

    fn update(&mut self) {
        for key in &mut self.keys {
            key.refresh();
        }
    }
}

const MOUSE_BUTTON_COUNT: usize = 5;
const MOUSE_MAX_SPECIAL_BUTTON_COUNT: usize = 32;

#[derive(Debug)]
pub(crate) struct StdMouseState {
    buttons: [InputState; MOUSE_BUTTON_COUNT],
    special_buttons: [InputState; MOUSE_MAX_SPECIAL_BUTTON_COUNT],
    position: (f32, f32),
    delta: (f32, f32),
    wheel_delta: (f32, f32),
    wheel_delta_unit: u32,
}

impl Default for StdMouseState {
    fn default() -> Self {
        Self {
            buttons: [InputState::default(); MOUSE_BUTTON_COUNT],
            special_buttons: [InputState::default(); MOUSE_MAX_SPECIAL_BUTTON_COUNT],
            position: (0., 0.),
            delta: (0., 0.),
            wheel_delta: (0., 0.),
            wheel_delta_unit: 0,
        }
    }
}

impl StdMouseState {
    pub(crate) fn data(&self) -> Vec<u8> {
        self.buttons
            .iter()
            .flat_map(|state| state.data())
            .chain([0, 0, 0, 0])
            .chain(self.special_buttons.iter().flat_map(|state| state.data()))
            .chain(self.position.0.to_ne_bytes())
            .chain(self.position.1.to_ne_bytes())
            .chain(self.delta.0.to_ne_bytes())
            .chain(self.delta.1.to_ne_bytes())
            .chain(self.wheel_delta.0.to_ne_bytes())
            .chain(self.wheel_delta.1.to_ne_bytes())
            .chain(self.wheel_delta_unit.to_ne_bytes())
            .chain([0, 0, 0, 0])
            .collect()
    }

    pub(crate) fn update_button(&mut self, button: MouseButton, state: ElementState) {
        let (button_index, button_id) = match button {
            MouseButton::Left => (Some(0), None),
            MouseButton::Right => (Some(1), None),
            MouseButton::Middle => (Some(2), None),
            MouseButton::Back => (Some(3), None),
            MouseButton::Forward => (Some(4), None),
            MouseButton::Other(id) => (None, Some(id)),
        };
        match (button_index, button_id) {
            (Some(button_index), _) => match state {
                ElementState::Pressed => self.buttons[button_index].press(),
                ElementState::Released => self.buttons[button_index].release(),
            },
            (_, Some(button_id)) => {
                let button = if let Some(button) = self.existing_special_button(button_id) {
                    Some(button)
                } else {
                    self.new_special_button()
                };
                if let Some(button) = button {
                    button.set_id(button_id);
                    match state {
                        ElementState::Pressed => button.press(),
                        ElementState::Released => button.release(),
                    }
                }
            }
            _ => unreachable!("internal error: invalid button"),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn update_position(&mut self, position: PhysicalPosition<f64>) {
        self.position = (position.x as f32, position.y as f32);
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn update_delta(&mut self, delta: (f64, f64)) {
        self.delta.0 += delta.0 as f32;
        self.delta.1 += delta.1 as f32;
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn update_wheel_delta(&mut self, delta: MouseScrollDelta) {
        match delta {
            MouseScrollDelta::LineDelta(columns, rows) => {
                self.wheel_delta.0 += columns;
                self.wheel_delta.1 += rows;
                self.wheel_delta_unit = 0;
            }
            MouseScrollDelta::PixelDelta(delta) => {
                self.wheel_delta.0 += delta.x as f32;
                self.wheel_delta.1 += delta.y as f32;
                self.wheel_delta_unit = 1;
            }
        }
    }

    fn update(&mut self) {
        for button in &mut self.buttons {
            button.refresh();
        }
        for button in &mut self.special_buttons {
            button.refresh();
        }
        self.delta = (0., 0.);
        self.wheel_delta = (0., 0.);
    }

    fn existing_special_button(&mut self, button_id: u16) -> Option<&mut InputState> {
        self.special_buttons
            .iter_mut()
            .find(|state| state.id() == button_id)
    }

    fn new_special_button(&mut self) -> Option<&mut InputState> {
        self.special_buttons.iter_mut().find(|state| {
            !state.bit(InputState::IS_PRESSED_BIT) && !state.bit(InputState::IS_JUST_RELEASED_BIT)
        })
    }
}

const MAX_FINGER_COUNT: usize = 10;

#[derive(Debug, Default)]
pub(crate) struct StdTouchState {
    fingers: [Finger; MAX_FINGER_COUNT],
}

impl StdTouchState {
    pub(crate) fn data(&self) -> Vec<u8> {
        self.fingers.iter().flat_map(Finger::data).collect()
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn update_finger(&mut self, touch: Touch) {
        let finger_id = (touch.id % u64::from(u16::MAX)) as u16;
        let finger = if let Some(finger) = self.existing_finger(finger_id) {
            Some(finger)
        } else {
            self.new_finger()
        };
        if let Some(finger) = finger {
            finger.state.set_id(finger_id);
            match touch.phase {
                TouchPhase::Started => {
                    finger.position = (touch.location.x as f32, touch.location.y as f32);
                    finger.state.press();
                }
                TouchPhase::Moved => {
                    let position = (touch.location.x as f32, touch.location.y as f32);
                    finger.delta = (
                        position.0 - finger.position.0,
                        position.1 - finger.position.1,
                    );
                    finger.position = position;
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    finger.state.release();
                }
            }
        }
    }

    fn update(&mut self) {
        for finger in &mut self.fingers {
            finger.refresh();
        }
    }

    fn existing_finger(&mut self, finger_id: u16) -> Option<&mut Finger> {
        self.fingers
            .iter_mut()
            .find(|finger| finger.state.id() == finger_id)
    }

    fn new_finger(&mut self) -> Option<&mut Finger> {
        self.fingers.iter_mut().find(|finger| {
            !finger.state.bit(InputState::IS_PRESSED_BIT)
                && !finger.state.bit(InputState::IS_JUST_RELEASED_BIT)
        })
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Finger {
    state: InputState,
    position: (f32, f32),
    delta: (f32, f32),
}

impl Finger {
    pub(crate) fn data(&self) -> Vec<u8> {
        self.state
            .data()
            .into_iter()
            .chain([0, 0, 0, 0])
            .chain(self.position.0.to_ne_bytes())
            .chain(self.position.1.to_ne_bytes())
            .chain(self.delta.0.to_ne_bytes())
            .chain(self.delta.1.to_ne_bytes())
            .collect()
    }

    fn refresh(&mut self) {
        self.state.refresh();
        self.delta = (0., 0.);
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
    const ID_BIT_OFFSET: u8 = 16;

    fn data(self) -> [u8; 4] {
        self.data.to_ne_bytes()
    }

    #[allow(clippy::cast_possible_truncation)]
    fn id(self) -> u16 {
        (self.data >> Self::ID_BIT_OFFSET) as u16
    }

    fn set_id(&mut self, id: u16) {
        self.data |= u32::from(id) << Self::ID_BIT_OFFSET;
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
