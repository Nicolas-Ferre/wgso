#mod main

struct Rectangle {
    position: vec2f,
    color: vec4f,
}

#mod compute
#import ~.main
#import constant.main
#import _.std.color.constant
#import _.std.input.keyboard
#import _.std.input.mouse
#import _.std.io.compute
#import _.std.vertex.transform

const RECT_SPEED = 0.02;

fn init_rectangle(position: vec2f) -> Rectangle {
    return Rectangle(position, WHITE);
}

fn update_rectangle_with_keyboard(rectangle: Rectangle) -> Rectangle {
    var updated = rectangle;
    updated.position += RECT_SPEED * input_direction(
        std_.keyboard.keys[KB_ARROW_LEFT],
        std_.keyboard.keys[KB_ARROW_RIGHT],
        std_.keyboard.keys[KB_ARROW_UP],
        std_.keyboard.keys[KB_ARROW_DOWN],
    );
    let enter_state = std_.keyboard.keys[KB_ENTER];
    if is_just_pressed(enter_state) {
        updated.color = BLUE;
    } else if is_just_released(enter_state) {
        updated.color = GREEN;
    } else if is_pressed(enter_state) {
        updated.color = RED;
    }
    return updated;
}

fn update_rectangle_with_mouse(rectangle: Rectangle) -> Rectangle {
    var updated = rectangle;
    let mouse_coords = pixel_to_world_coords(std_.mouse.position, std_.surface.size);
    let scale_factor = scale_factor(std_.surface.size, VISIBLE_AREA_MIN_SIZE);
    updated.position = mouse_coords / scale_factor;
    let enter_state = std_.mouse.buttons[MS_BUTTON_LEFT];
    if is_just_pressed(enter_state) {
        updated.color = BLUE;
    } else if is_just_released(enter_state) {
        updated.color = GREEN;
    } else if is_pressed(enter_state) {
        updated.color = RED;
    }
    let wheel_factor = select(1. / 256., 1. / 10., std_.mouse.wheel.delta_unit == MS_WHEEL_LINES);
    updated.color.r += std_.mouse.wheel.delta.y * wheel_factor;
    updated.color.b += std_.mouse.wheel.delta.x * wheel_factor;
    updated.color = clamp(updated.color, vec4f(0, 0, 0, 0), vec4f(1, 1, 1, 1));
    return updated;
}

fn update_rectangle_with_touch(rectangle: Rectangle, finger_index: u32) -> Rectangle {
    var updated = rectangle;
    let finger = &std_.touch.fingers[finger_index];
    let finger_coords = pixel_to_world_coords(finger.position, std_.surface.size);
    let scale_factor = scale_factor(std_.surface.size, VISIBLE_AREA_MIN_SIZE);
    updated.position = finger_coords / scale_factor;
    if is_just_pressed(finger.state) {
        updated.color = BLUE;
    } else if is_just_released(finger.state) {
        updated.color = INVISIBLE;
    } else if is_pressed(finger.state) {
        updated.color = RED;
    } else {
        updated.position = HIDDEN_RECT_POSITION;
    }
    return updated;
}

#shader<render, Vertex, Rectangle> render
#import ~.main
#import constant.main
#import _.std.io.main
#import _.std.vertex.transform
#import _.std.vertex.type

const RECT_SIZE = vec2f(0.3, 0.3);

var<uniform> surface: Surface;

struct Fragment {
    @builtin(position)
    position: vec4f,
    @location(0)
    color: vec4f,
}

@vertex
fn vs_main(vertex: Vertex, instance: Rectangle) -> Fragment {
    let scale_factor = scale_factor(surface.size, VISIBLE_AREA_MIN_SIZE);
    let position = vertex.position.xy * RECT_SIZE + instance.position;
    return Fragment(
        vec4f(position * scale_factor, 0, 1),
        instance.color,
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    if fragment.color.a < 0.5 {
        discard;
    }
    return fragment.color;
}
