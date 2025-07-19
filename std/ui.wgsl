/// Types for UI widgets.
#mod main

const BUTTON_STATE_NONE = 0u;
const BUTTON_STATE_HOVERED = 1u;
const BUTTON_STATE_PRESSED = 2u;
const BUTTON_STATE_RELEASED = 3u;

/// Properties of a UI button.
struct UiButton {
    /// Position of the button in world units.
    position: vec3f,
    /// Size of the button in world units.
    size: vec2f,
    /// State of the button.
    ///
    /// Can be one of the following values:
    /// - `BUTTON_STATE_NONE`: no interaction with the button
    /// - `BUTTON_STATE_HOVERED`: mouse is hovering the button
    /// - `BUTTON_STATE_PRESSED`: mouse or finger is pressing the button
    /// - `BUTTON_STATE_RELEASED`: mouse or finger just released the button
    state: u32,
}

/// Functions to update UI widgets.
#mod compute
#import ~.main
#import ~.~.input.mouse
#import ~.~.input.state
#import ~.~.io.compute
#import ~.~.math.matrix
#import ~.~.physics.collision

/// Creates a new button.
fn init_ui_button(position: vec3f, size: vec2f) -> UiButton {
    return UiButton(position, size, BUTTON_STATE_NONE);
}

/// Updates the state of a button.
///
/// Both mouse left button and touch events are checked.
fn update_ui_button(button: UiButton, view_mat_arr: array<vec4f, 4>) -> UiButton {
    var updated = button;
    var new_state = _ui_button_state(
        updated,
        std_.mouse.position,
        std_.mouse.buttons[MS_BUTTON_LEFT],
        view_mat_arr,
    );
    for (var finger_index = 0u; finger_index < MAX_FINGER_COUNT; finger_index++) {
        let finger = std_.touch.fingers[finger_index];
        let is_pressed = is_pressed(finger.state);
        new_state = max(new_state, _ui_button_state(
            updated,
            finger.position,
            finger.state,
            view_mat_arr,
        ));
    }
    updated.state = new_state;
    return updated;
}

fn _ui_button_state(button: UiButton, cursor_position: vec2f, cursor_state: u32, view_mat_arr: array<vec4f, 4>) -> u32 {
    if _is_in_ui_button(button, cursor_position, view_mat_arr) {
        if is_pressed(cursor_state) {
            return BUTTON_STATE_PRESSED;
        } else if is_just_released(cursor_state) {
            return BUTTON_STATE_RELEASED;
        } else {
            return BUTTON_STATE_HOVERED;
        }
    } else {
        return BUTTON_STATE_NONE;
    }
}

fn _is_in_ui_button(button: UiButton, pixel_position: vec2f, view_mat_arr: array<vec4f, 4>) -> bool {
    let view_mat = array_to_mat4x4f(view_mat_arr);
    let world_coords = pixel_to_world_coords(pixel_position, std_.surface.size);
    let final_world_position = (view_mat * vec4f(world_coords, 0, 1)).xy;
    return aabb_collision(button.position.xy, button.size, final_world_position, vec2f(0, 0)).is_colliding;
}
