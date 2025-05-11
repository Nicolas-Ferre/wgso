//! Storage used retrieve or send information to the CPU.

#import ~.input

/// The number of recognized keyboard keys.
const KEYBOARD_KEY_COUNT = 193;

/// Main storage variable of the standard library.
var<storage, read_write> std_: Std;

/// Main storage type of the standard library.
struct Std {
    /// Time information retrieved from the CPU.
    time: Time,
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

/// Keyboard state.
struct Keyboard {
    /// The state of keyboard keys.
    keys: array<InputState, KEYBOARD_KEY_COUNT>,
}
