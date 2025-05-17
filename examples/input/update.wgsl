#shader<compute> update_rectangles

#import ~.storage
#import _.std.color
#import _.std.input
#import _.std.storage
#import _.std.vertex

const RECTANGLE_SPEED = 0.02;

@compute
@workgroup_size(1, 1, 1)
fn main() {
    rectangles.ratio = f32(std_.surface.size.x) / f32(std_.surface.size.y);
    update_keyboard();
    update_mouse();
}

fn update_keyboard() {
    rectangles.keyboard.position += RECTANGLE_SPEED * input_direction(
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
    rectangles.mouse.position = (std_.mouse.position / vec2f(std_.surface.size) - vec2f(0.5, 0.5))
        * vec2f(2, -rectangles.ratio);
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
