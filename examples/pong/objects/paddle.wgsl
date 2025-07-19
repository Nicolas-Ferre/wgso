#mod main

const PADDLE_SIZE = vec2f(0.06, 0.3);

struct Paddle {
    position: vec3f,
}

#mod compute
#import ~.main
#import constant.main
#import _.std.input.keyboard
#import _.std.io.compute
#import _.std.vertex.transform

const PADDLE_PLAYER_SPEED = 2.;
const PADDLE_BOT_SPEED = 1.2;

fn init_paddle(position: vec3f) -> Paddle {
    return Paddle(position);
}

fn reset_paddle(paddle: Paddle) -> Paddle {
    var updated = paddle;
    updated.position.y = 0;
    return updated;
}

fn update_player_paddle(
    paddle: Paddle, max_position_y: f32,
    up_key: u32, down_key: u32,
    touch_min_x: f32, touch_max_x: f32,
) -> Paddle {
    var updated = paddle;
    var offset = _keyboard_offset(up_key, down_key);
    if offset == 0 {
        offset = _touch_offset(updated.position.y, touch_min_x, touch_max_x);
    }
    updated.position.y += offset;
    let max_paddle_y = max_position_y - PADDLE_SIZE.y / 2;
    updated.position.y = clamp(updated.position.y, -max_paddle_y, max_paddle_y);
    return updated;
}

fn update_bot_paddle(paddle: Paddle, max_position_y: f32, target_y: f32) -> Paddle {
    var updated = paddle;
    updated.position.y += _offset_to_target(updated.position.y, target_y, PADDLE_BOT_SPEED);
    let max_paddle_y = max_position_y - PADDLE_SIZE.y / 2;
    updated.position.y = clamp(updated.position.y, -max_paddle_y, max_paddle_y);
    return updated;
}

fn _keyboard_offset(up_key: u32, down_key: u32) -> f32 {
    let direction = input_axis(std_.keyboard.keys[down_key], std_.keyboard.keys[up_key]);
    return PADDLE_PLAYER_SPEED * std_.time.frame_delta_secs * direction;
}

fn _touch_offset(position_y: f32, touch_min_x: f32, touch_max_x: f32) -> f32 {
    let scale_factor = scale_factor(std_.surface.size, VISIBLE_AREA_MIN_SIZE);
    for (var finger_index = 0u; finger_index < MAX_FINGER_COUNT; finger_index++) {
        let finger = std_.touch.fingers[finger_index];
        let finger_position = pixel_to_world_coords(finger.position, std_.surface.size) / scale_factor;
        if is_pressed(finger.state) && finger_position.x >= touch_min_x && finger_position.x <= touch_max_x {
            return _offset_to_target(position_y, finger_position.y, PADDLE_PLAYER_SPEED);
        }
    }
    return 0;
}

fn _offset_to_target(position_y: f32, target_position_y: f32, speed: f32) -> f32 {
    let max_offset = speed * std_.time.frame_delta_secs;
    return clamp(target_position_y - position_y, -max_offset, max_offset);
}

#shader<render, Vertex, Paddle> render
#import ~.main
#import constant.main
#import _.std.color.constant
#import _.std.math.distance
#import _.std.io.main
#import _.std.vertex.transform
#import _.std.vertex.type

const GLOW_FACTOR = 0.003;
const THICKNESS = 0.008;
const COLOR = CYAN;

var<uniform> surface: Surface;

struct Fragment {
    @builtin(position)
    clip_position: vec4f,
    @location(0)
    world_position: vec2f,
    @location(1)
    paddle_position: vec2f,
}

@vertex
fn vs_main(vertex: Vertex, instance: Paddle) -> Fragment {
    let scale_factor = scale_factor(surface.size, VISIBLE_AREA_MIN_SIZE);
    let position = vertex.position.xy * PADDLE_SIZE * 2 + instance.position.xy;
    return Fragment(
        vec4f(position * scale_factor, instance.position.z, 1),
        position,
        instance.position.xy,
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    let dist = rect_signed_dist(fragment.world_position - fragment.paddle_position, PADDLE_SIZE - PADDLE_SIZE.x) - PADDLE_SIZE.x / 2;
    let exterior_brighness = step(0, dist) * exp(-dist / GLOW_FACTOR);
    let interior_brighness = step(THICKNESS, -dist) * exp((dist + THICKNESS) / GLOW_FACTOR);
    let middle_brightness = step(-dist, THICKNESS) * step(dist, 0);
    return COLOR * max(exterior_brighness, max(interior_brighness, middle_brightness));
}
