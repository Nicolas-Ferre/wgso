//! Storage types used retrieve or send information to the CPU.

#import ~.input

/// The number of recognized keyboard keys.
const KEYBOARD_KEY_COUNT = 193;
/// The number of recognized standard mouse buttons.
const MOUSE_BUTTON_COUNT = 5;
/// The number of recognized special mouse buttons.
const MOUSE_SPECIAL_BUTTON_COUNT = 32;

/// Main storage type of the standard library.
struct Std {
    /// Time information retrieved from the CPU.
    time: Time,
    /// Surface properties.
    surface: Surface,
    /// Keyboard state retrieved from the CPU.
    keyboard: Keyboard,
    /// Mouse state retrieved from the CPU.
    mouse: Mouse,
}

/// Time information.
struct Time {
    /// Time taken to execute the previous frame, in seconds.
    frame_delta_secs: f32,
    /// Index of the current frame.
    frame_index: u32,
    /// Program start time, in seconds since Unix Epoch.
    start_secs: u32,
}

/// Surface properties.
struct Surface {
    /// Size of the surface in pixels.
    size: vec2u,
}

/// Keyboard state.
struct Keyboard {
    /// The state of keyboard keys.
    ///
    /// Index is one of `KB_*`.
    keys: array<InputState, KEYBOARD_KEY_COUNT>,
}

/// Mouse state.
struct Mouse {
    /// The state of standard mouse buttons.
    ///
    /// Index is one of `MS_BUTTON_*`.
    buttons: array<InputState, MOUSE_BUTTON_COUNT>,
    /// The state of special mouse buttons.
    special_buttons: array<InputState, MOUSE_SPECIAL_BUTTON_COUNT>,
    /// The mouse position in pixels from top-left corner of the surface.
    position: vec2f,
    /// The mouse delta in pixels.
    delta: vec2f,
    /// The mouse wheel state.
    wheel: MouseWheel,
}

/// Mouse wheel state.
struct MouseWheel {
    delta: vec2f,
    /// Either `MS_WHEEL_LINES` or `MS_WHEEL_PIXELS`.
    delta_unit: u32,
}
