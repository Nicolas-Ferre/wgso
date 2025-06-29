#mod main

const PADDLE_SIZE = vec2f(0.06, 0.3);

struct Paddle {
    position: vec3f,
}

#mod state
#import ~.main
#import _.std.input.keyboard
#import _.std.state.storage

const _PADDLE_SPEED = 2.;

fn init_paddle(x: f32, z: f32) -> Paddle {
    return Paddle(vec3f(x, 0, z));
}

fn reset_paddle(paddle: Paddle) -> Paddle {
    var updated = paddle;
    updated.position.y = 0;
    return updated;
}

fn update_paddle(paddle: Paddle, max_position_y: f32, up_key: u32, down_key: u32) -> Paddle {
    var updated = paddle;
    let direction = input_axis(std_.keyboard.keys[up_key], std_.keyboard.keys[down_key]);
    let paddle_max_position_y = max_position_y - PADDLE_SIZE.y / 2;
    updated.position.y -= _PADDLE_SPEED * std_.time.frame_delta_secs * direction;
    updated.position.y = clamp(updated.position.y, -paddle_max_position_y, paddle_max_position_y);
    return updated;
}

#shader<render, Vertex, Paddle> render
#import ~.main
#import config.constant
#import _.std.color.constant
#import _.std.math.distance
#import _.std.state.type
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
