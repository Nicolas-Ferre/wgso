#mod main
#init ~.init()
#run ~.update()
#draw<200> ~.render<vertices.rectangle, paddles.instances>(surface=surface)

const PADDLE_COUNT = 2;
const PADDLE_X_POSITION = 0.8;
const PADDLE_SIZE = vec2f(0.06, 0.3);
const PADDLE_SPEED = 2;

struct Paddles {
    instances: array<Paddle, PADDLE_COUNT>,
}

struct Paddle {
    position: vec2f,
}

#mod storage
#import ~.main

var<storage, read_write> paddles: Paddles;

fn reset_paddles() {
    paddles.instances[0].position = vec2f(-PADDLE_X_POSITION, 0);
    paddles.instances[1].position = vec2f(PADDLE_X_POSITION, 0);
}

#shader<compute> init
#import ~.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    reset_paddles();
}

#shader<compute> update
#import ~.storage
#import field.main
#import _.std.input.keyboard
#import _.std.state.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    move_paddle(0, KB_KEY_W, KB_KEY_S);
    move_paddle(1, KB_ARROW_UP, KB_ARROW_DOWN);
}

fn move_paddle(paddle_index: u32, up_key: u32, down_key: u32) {
    let instance = &paddles.instances[paddle_index];
    instance.position.y -= PADDLE_SPEED * std_.time.frame_delta_secs
        * input_axis(std_.keyboard.keys[up_key], std_.keyboard.keys[down_key]);
    instance.position.y = clamp(instance.position.y, -(FIELD_SIZE.y - PADDLE_SIZE.y) / 2, (FIELD_SIZE.y +- PADDLE_SIZE.y) / 2);
}

#shader<render, Vertex, Paddle> render
#import ~.main
#import ball.main
#import surface.main
#import _.std.vertex.type

const Z = 0.2;
const GLOW_FACTOR = 0.003;
const THICKNESS = 0.008;
const COLOR = BALL_COLOR;

var<uniform> surface: SurfaceData;

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
    let position = vertex.position.xy * PADDLE_SIZE * 2 + instance.position;
    return Fragment(
        vec4f(position * surface_ratio(surface.size), Z, 1),
        position,
        instance.position,
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    return paddle_color(fragment);
}

fn paddle_color(fragment: Fragment) -> vec4f {
    let dist = rect_signed_dist(fragment.world_position - fragment.paddle_position, PADDLE_SIZE - PADDLE_SIZE.x) - PADDLE_SIZE.x / 2;
    let exterior_brighness = step(0, dist) * exp(-dist / GLOW_FACTOR);
    let interior_brighness = step(THICKNESS, -dist) * exp((dist + THICKNESS) / GLOW_FACTOR);
    let middle_brightness = step(-dist, THICKNESS) * step(dist, 0);
    return vec4f(COLOR, 1.) * max(exterior_brighness, max(interior_brighness, middle_brightness));
}

fn rect_signed_dist(frag_position: vec2f, size: vec2f) -> f32 {
    let distance = abs(frag_position) - size / 2;
    let exterior_dist = length(max(distance, vec2f(0.0)));
    let interior_dist = min(max(distance.x, distance.y), 0.0);
    return exterior_dist + interior_dist;
}
