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
