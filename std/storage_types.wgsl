//! Storage types used retrieve or send information to the CPU.

#import ~.input

/// The number of recognized keyboard keys.
const KEYBOARD_KEY_COUNT = 193;

/// Main storage type of the standard library.
struct Std {
    /// Time information retrieved from the CPU.
    time: Time,
    /// Surface properties.
    surface: Surface,
    /// Keyboard state retrieved from the CPU.
    keyboard: Keyboard,
}

/// Time information.
struct Time {
    /// Time taken to execute the previous frame, in seconds.
    frame_delta_secs: f32,
    /// Index of the current frame.
    frame_index: f32,
    /// Program start time, in seconds since Unix Epoch.
    start_secs: u32,
}

/// Surface properties.
struct Surface {
    size: vec2u,
}

/// Keyboard state.
struct Keyboard {
    /// The state of keyboard keys.
    keys: array<InputState, KEYBOARD_KEY_COUNT>,
}
