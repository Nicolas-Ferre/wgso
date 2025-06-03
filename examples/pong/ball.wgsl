#mod main
#init ~.init()
#run ~.update()
#draw ~.render<ball.vertices, ball.instance>(global=global, collisions=ball.collisions)
#import _.std.vertex.type

const BALL_RADIUS = 0.03;
const BALL_DEFAULT_SPEED = 1.0;
const PREVIOUS_COLLISION_COUNT = 3;

struct BallData {
    collisions: array<BallCollision, PREVIOUS_COLLISION_COUNT>,
    vertices: array<Vertex, 6>,
    instance: BallInstance,
}

struct BallInstance {
    position: vec2f,
    velocity: vec2f,
}

struct BallCollision {
    position: vec2f,
    _phantom: mat4x4f,
}

#mod storage
#import ~.main

var<storage, read_write> ball: BallData;

#shader<compute> init
#import ~.storage
#import _.std.vertex.model

@compute
@workgroup_size(1, 1, 1)
fn main() {
    ball.vertices = rectangle_vertices();
    ball.instance.position = vec2f(-0.1, -0.1);
    ball.instance.velocity = normalize(vec2f(-1, 1)) * BALL_DEFAULT_SPEED;
    ball.collisions[0].position = ball.instance.position;
}

#shader<compute> update
#import ~.storage
#import field.main
#import _.std.state.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    const HALF_LIMIT = FIELD_SIZE / 2 - BALL_RADIUS;
    let instance = &ball.instance;
    instance.position += instance.velocity * std_.time.frame_delta_secs;
    instance.velocity = select(instance.velocity, -instance.velocity, abs(instance.position) > HALF_LIMIT);
    if any(abs(instance.position) > HALF_LIMIT) {
        add_collision(instance.position);
    }
    instance.position = clamp(instance.position, -HALF_LIMIT, HALF_LIMIT);
}

fn add_collision(collision_position: vec2f) {
    for (var i = 0; i < PREVIOUS_COLLISION_COUNT - 1; i++) {
        let index = PREVIOUS_COLLISION_COUNT - 1 - i;
        ball.collisions[index].position = ball.collisions[index - 1].position;
    }
    ball.collisions[0].position = collision_position;
}

#shader<render, Vertex, BallInstance> render
#import ~.main
#import global.main
#import field.main
#import _.std.math.vector
#import _.std.math.matrix

const Z = 0.0;
const MOTION_Z = 0.1;
const MOTION_BRIGHNESS = 0.2;
const COLOR = vec3f(0, 251, 255) / 255.;
const GLOW_FACTOR = 0.005;
const THICKNESS = 0.005;

var<uniform> global: Global;
var<uniform> collisions: array<BallCollision, PREVIOUS_COLLISION_COUNT>;

struct Fragment {
    @builtin(position)
    clip_position: vec4f,
    @location(0)
    world_position: vec2f,
    @location(1)
    ball_position: vec2f,
}

@vertex
fn vs_main(vertex: Vertex, instance: BallInstance) -> Fragment {
    let position = vertex.position.xy * FIELD_SIZE;
    return Fragment(
        vec4f(position * surface_ratio(global.surface_size), Z, 1),
        position,
        instance.position.xy,
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    return max(ball_color(fragment), trail_color(fragment));
}

fn ball_color(fragment: Fragment) -> vec4f {
    let dist = circle_signed_dist(fragment.world_position - fragment.ball_position, BALL_RADIUS);
    let exterior_brighness = step(0, dist) * exp(-dist / GLOW_FACTOR);
    let interior_brighness = step(THICKNESS, -dist) * exp((dist + THICKNESS) / GLOW_FACTOR);
    let middle_brightness = step(-dist, THICKNESS) * step(dist, 0);
    return vec4f(COLOR, 1.) * max(exterior_brighness, max(interior_brighness, middle_brightness));
}

fn trail_color(fragment: Fragment) -> vec4f {
    const TRAIL_WIDTH = BALL_RADIUS * 2;
    var brightness = 0.;
    var accumulated_dist = 0.;
    if distance(fragment.world_position, fragment.ball_position) > BALL_RADIUS {
        for (var i = 0; i < PREVIOUS_COLLISION_COUNT; i++) {
            let bound1 = collisions[i].position;
            if all(bound1 == vec2f(0, 0)) {
                continue;
            }
            let bound2 = select(collisions[i - 1].position, fragment.ball_position, i == 0);
            let width_dist = TRAIL_WIDTH - segment_signed_dist(fragment.world_position, bound1, bound2);
            let length_dist = accumulated_dist + distance(fragment.world_position, bound2);
            let is_trail = width_dist >= 0 && width_dist < TRAIL_WIDTH;
            brightness = max(brightness, select(0., 1., is_trail) * pow(1 * width_dist / length_dist, 2));
            accumulated_dist += distance(bound1, bound2);
        }
    }
    return vec4f(COLOR, 1) * brightness;
}

fn circle_signed_dist(position: vec2f, radius: f32) -> f32 {
    return length(position) - radius;
}

fn segment_signed_dist(p: vec2f, a: vec2f, b: vec2f) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0., 1.);
    return length(pa - ba * h);
}
