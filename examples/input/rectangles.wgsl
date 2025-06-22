#mod main
#init ~.init()
#run ~.update()
#draw ~.render<rectangles.vertices, rectangles.keyboard>(ratio=rectangles.ratio)
#draw ~.render<rectangles.vertices, rectangles.mouse>(ratio=rectangles.ratio)
#draw ~.render<rectangles.vertices, rectangles.fingers>(ratio=rectangles.ratio)
#import _.std.state.type
#import _.std.vertex.type

const RECT_SIZE = vec2f(0.3, 0.3);
const RECT_SPEED = 0.02;

struct Rectangles {
    ratio: f32,
    vertices: array<Vertex, 6>,
    keyboard: Rect,
    mouse: Rect,
    fingers: array<Rect, MAX_FINGER_COUNT>,
}

struct Rect {
    position: vec2f,
    color: vec4f,
}

fn ratio_2d(surface_ratio: f32) -> vec2f {
    return select(vec2f(1, surface_ratio), vec2f(1 / surface_ratio, 1), surface_ratio > 1);
}

#mod storage
#import ~.main

var<storage, read_write> rectangles: Rectangles;

#shader<compute> init
#import ~.storage
#import _.std.color.constant
#import _.std.state.storage
#import _.std.vertex.model

@compute
@workgroup_size(1, 1, 1)
fn main() {
    rectangles.vertices = rectangle_vertices();
    rectangles.keyboard.position = vec2f(-0.5, 0.5);
    rectangles.keyboard.color = WHITE;
    rectangles.mouse.position = vec2f(0.5, 0.5);
    rectangles.mouse.color = WHITE;
}

#shader<compute> update
#import ~.storage
#import _.std.color.constant
#import _.std.input.keyboard
#import _.std.input.mouse
#import _.std.state.storage
#import _.std.vertex.type

@compute
@workgroup_size(1, 1, 1)
fn main() {
    rectangles.ratio = f32(std_.surface.size.x) / f32(std_.surface.size.y);
    update_keyboard();
    update_mouse();
    update_touch();
}

fn update_keyboard() {
    rectangles.keyboard.position += RECT_SPEED * input_direction(
        std_.keyboard.keys[KB_ARROW_LEFT],
        std_.keyboard.keys[KB_ARROW_RIGHT],
        std_.keyboard.keys[KB_ARROW_UP],
        std_.keyboard.keys[KB_ARROW_DOWN],
    );
    let enter_state = std_.keyboard.keys[KB_ENTER];
    if is_just_pressed(enter_state) {
        rectangles.keyboard.color = BLUE;
    } else if is_just_released(enter_state) {
        rectangles.keyboard.color = GREEN;
    } else if is_pressed(enter_state) {
        rectangles.keyboard.color = RED;
    }
}

fn update_mouse() {
    rectangles.mouse.position = surface_to_world_position(std_.mouse.position);
    let enter_state = std_.mouse.buttons[MS_BUTTON_LEFT];
    if is_just_pressed(enter_state) {
        rectangles.mouse.color = BLUE;
    } else if is_just_released(enter_state) {
        rectangles.mouse.color = GREEN;
    } else if is_pressed(enter_state) {
        rectangles.mouse.color = RED;
    }
    let wheel_factor = select(1. / 256., 1. / 10., std_.mouse.wheel.delta_unit == MS_WHEEL_LINES);
    rectangles.mouse.color.r += std_.mouse.wheel.delta.y * wheel_factor;
    rectangles.mouse.color.b += std_.mouse.wheel.delta.x * wheel_factor;
    rectangles.mouse.color = clamp(rectangles.mouse.color, vec4f(0, 0, 0, 0), vec4f(1, 1, 1, 1));
}

fn update_touch() {
    for (var i = 0u; i < MAX_FINGER_COUNT; i++) {
        let finger = &std_.touch.fingers[i];
        let rectangle = &rectangles.fingers[i];
        rectangle.position = surface_to_world_position(finger.position);
        if is_just_pressed(finger.state) {
            rectangle.color = BLUE;
        } else if is_just_released(finger.state) {
            rectangle.color = INVISIBLE;
        } else if is_pressed(finger.state) {
            rectangle.color = RED;
        }
    }
}

fn surface_to_world_position(surface_position: vec2f) -> vec2f {
    return (surface_position / vec2f(std_.surface.size) - vec2f(0.5, 0.5))
        * vec2f(2, -2) / ratio_2d(rectangles.ratio);
}

#shader<render, Vertex, Rect> render
#import ~.main
#import _.std.vertex.type

var<uniform> ratio: f32;

struct Fragment {
    @builtin(position)
    position: vec4f,
    @location(0)
    color: vec4f,
}

@vertex
fn vs_main(vertex: Vertex, instance: Rect) -> Fragment {
    let position = vertex.position.xy * RECT_SIZE + instance.position;
    return Fragment(
        vec4f(position * ratio_2d(ratio), 0, 1),
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
