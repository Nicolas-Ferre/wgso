#mod main
#init ~.init()
#run ~.update()
#draw<500> ~.render<vertices.rectangle, ball.instance>(surface=std_.surface, collisions=ball.collisions)

const BALL_RADIUS = 0.03;
const BALL_DEFAULT_SPEED = 1.5;
const BALL_COLOR = vec3f(0, 251, 255) / 255.;
const BALL_PREVIOUS_COLLISION_COUNT = 3;

struct BallData {
    collisions: array<BallCollision, BALL_PREVIOUS_COLLISION_COUNT>,
    instance: BallInstance,
}

struct BallInstance {
    position: vec2f,
    velocity: vec2f,
}

struct BallCollision {
    position: vec2f,
    is_set: u32,
    _phantom: mat4x4f,
}

#mod storage
#import ~.main

var<storage, read_write> ball: BallData;

fn reset_ball(direction: vec2f) {
    ball.instance.position = vec2f(0, 0);
    ball.instance.velocity = normalize(direction) * BALL_DEFAULT_SPEED;
    ball.collisions = array<BallCollision, BALL_PREVIOUS_COLLISION_COUNT>();
    ball.collisions[0].position = ball.instance.position;
    ball.collisions[0].is_set = 1;
}

#shader<compute> init
#import ~.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    reset_ball(vec2f(1, 0));
}

#shader<compute> update
#import ~.storage
#import ~.~.particles.storage
#import field.main
#import paddles.storage
#import scores.storage
#import _.std.state.storage

const COLLISION_DIRECTION_X_WEIGHT = 0.1;

@compute
@workgroup_size(1, 1, 1)
fn main() {
    resolve_horizontal_wall_collisions();
    resolve_vertical_wall_collisions();
    resolve_paddle_collisions(0);
    resolve_paddle_collisions(1);
}

fn resolve_horizontal_wall_collisions() {
    const HALF_LIMIT = FIELD_SIZE.y / 2 - BALL_RADIUS;
    let ball = &ball.instance;
    ball.position += ball.velocity * std_.time.frame_delta_secs;
    ball.velocity.y = select(ball.velocity.y, -ball.velocity.y, abs(ball.position.y) > HALF_LIMIT);
    let fixed_position = vec2f(ball.position.x, clamp(ball.position.y, -HALF_LIMIT, HALF_LIMIT));
    if any(abs(ball.position.y) > HALF_LIMIT) {
        register_collision(fixed_position);
        add_collision_particles(fixed_position, (ball.position - fixed_position) * vec2f(-1, 1));
    }
    ball.position = fixed_position;
}

fn resolve_vertical_wall_collisions() {
    const HALF_LIMIT = FIELD_SIZE / 2 - BALL_RADIUS;
    let ball = &ball.instance;
    if ball.position.x < -FIELD_SIZE.x / 2 {
        scores.right += 1;
        add_collision_particles(ball.position, vec2f(0, 0));
        reset_ball(vec2f(-1, 0));
        reset_paddles();
    } else if ball.position.x > FIELD_SIZE.x / 2 {
        scores.left += 1;
        add_collision_particles(ball.position, vec2f(0, 0));
        reset_ball(vec2f(1, 0));
        reset_paddles();
    }
}

fn resolve_paddle_collisions(paddle_index: u32) {
    let paddle_position = paddles.instances[paddle_index].position;
    let ball = &ball.instance;
    let collision_normal = aabb_collision_normal(paddle_position, PADDLE_SIZE, ball.position, vec2f(BALL_RADIUS * 2));
    if any(collision_normal != vec2f(0, 0)) {
        let direction = sign(-ball.velocity.x);
        ball.velocity = length(ball.velocity) * normalize(vec2f(
            direction * COLLISION_DIRECTION_X_WEIGHT,
            ball.position.y - paddle_position.y,
        ));
        if collision_normal.x != 0 {
            ball.position.x = paddle_position.x + collision_normal.x * (PADDLE_SIZE.x / 2 + BALL_RADIUS);
        } else {
            ball.position.y = paddle_position.y + collision_normal.y * (PADDLE_SIZE.y / 2 + BALL_RADIUS);
        }
        register_collision(ball.position);
        add_collision_particles(ball.position, vec2f(direction, 0.0));
    }
}

fn aabb_collision_normal(pos1: vec2f, size1: vec2f, pos2: vec2f, size2: vec2f) -> vec2f {
    let delta = pos2 - pos1;
    let overlap = (size1 + size2) / 2 - abs(delta);
    if any(overlap <= vec2f(0)) {
        return vec2f(0, 0); // no collision
    }
    if overlap.x < overlap.y {
        return vec2f(sign(delta.x), 0);
    } else {
        return vec2f(0, sign(delta.y));
    }
}

fn register_collision(collision_position: vec2f) {
    for (var i = 0; i < BALL_PREVIOUS_COLLISION_COUNT - 1; i++) {
        let index = BALL_PREVIOUS_COLLISION_COUNT - 1 - i;
        ball.collisions[index].position = ball.collisions[index - 1].position;
        ball.collisions[index].is_set = ball.collisions[index - 1].is_set;
    }
    ball.collisions[0].position = collision_position;
}

#shader<render, Vertex, BallInstance> render
#import ~.main
#import surface.main
#import field.main
#import _.std.math.vector
#import _.std.math.matrix
#import _.std.state.type
#import _.std.vertex.type

const Z = 0.5;
const MOTION_Z = 0.1;
const MOTION_BRIGHNESS = 0.2;
const GLOW_FACTOR = 0.005;
const THICKNESS = 0.005;

var<uniform> surface: Surface;
var<uniform> collisions: array<BallCollision, BALL_PREVIOUS_COLLISION_COUNT>;

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
        vec4f(position * surface_ratio(surface.size), Z, 1),
        position,
        instance.position,
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
    return vec4f(BALL_COLOR, 1.) * max(exterior_brighness, max(interior_brighness, middle_brightness));
}

fn trail_color(fragment: Fragment) -> vec4f {
    const TRAIL_WIDTH = BALL_RADIUS * 2;
    var brightness = 0.;
    var accumulated_dist = 0.;
    if distance(fragment.world_position, fragment.ball_position) > BALL_RADIUS {
        for (var i = 0; i < BALL_PREVIOUS_COLLISION_COUNT; i++) {
            if collisions[i].is_set == 0 {
                continue;
            }
            let bound1 = collisions[i].position;
            let bound2 = select(collisions[i - 1].position, fragment.ball_position, i == 0);
            let width_dist = TRAIL_WIDTH - segment_signed_dist(fragment.world_position, bound1, bound2);
            let length_dist = accumulated_dist + distance(fragment.world_position, bound2);
            let is_trail = width_dist >= 0 && width_dist < TRAIL_WIDTH;
            brightness = max(brightness, select(0., 1., is_trail) * pow(width_dist / length_dist, 2));
            accumulated_dist += distance(bound1, bound2);
        }
    }
    return vec4f(BALL_COLOR, 1) * brightness;
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
