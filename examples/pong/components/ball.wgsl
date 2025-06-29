#mod main
#import _.std.math.constant

const BALL_RADIUS = 0.03;
const UNSET_TRAIL_POSITION = vec2f(F32_MAX, F32_MAX);

struct Ball {
    position: vec3f,
    velocity: vec2f,
    trail_position_1: vec2f,
    trail_position_2: vec2f,
    trail_position_3: vec2f,
}

#mod state
#import ~.main
#import _.std.math.constant
#import _.std.physics.collision
#import _.std.state.storage

const _BALL_DEFAULT_SPEED = 1.0;
const _BALL_ACCELERATION = 0.1;
const _BALL_PADDLE_ANGLE_FACTOR = 0.1;

fn init_ball(z: f32, direction: vec2f) -> Ball {
    let position = vec3f(0, 0, z);
    return Ball(
        position,
        normalize(direction) * _BALL_DEFAULT_SPEED,
        position.xy,
        UNSET_TRAIL_POSITION,
        UNSET_TRAIL_POSITION,
    );
}

fn update_ball(ball: Ball) -> Ball {
    var updated = ball;
    updated.velocity += normalize(updated.velocity) * _BALL_ACCELERATION * std_.time.frame_delta_secs;
    updated.position += vec3f(updated.velocity * std_.time.frame_delta_secs, 0);
    return updated;
}

fn apply_ball_wall_collision(ball: Ball, wall_y: f32) -> Ball {
    var updated = ball;
    updated.velocity.y *= -1;
    updated.position.y = clamp(updated.position.y, -wall_y + BALL_RADIUS, wall_y - BALL_RADIUS);
    updated = _add_trail_position(updated, updated.position.xy);
    return updated;
}

fn apply_ball_paddle_collision(ball: Ball, paddle_position: vec2f, collision: Collision) -> Ball {
    var updated = ball;
    let direction = sign(-updated.velocity.x);
    updated.velocity = length(updated.velocity) * normalize(vec2f(
        direction * _BALL_PADDLE_ANGLE_FACTOR,
        updated.position.y - paddle_position.y,
    ));
    let oriented_penetration = vec2f(collision.penetration.x, collision.penetration.y);
    updated.position += vec3f(oriented_penetration, 0);
    updated = _add_trail_position(updated, updated.position.xy);
    return updated;
}

fn _add_trail_position(ball: Ball, position: vec2f) -> Ball {
    var updated = ball;
    updated.trail_position_3 = updated.trail_position_2;
    updated.trail_position_2 = updated.trail_position_1;
    updated.trail_position_1 = position;
    return updated;
}

#shader<render, Vertex, Ball> render
#import ~.main
#import config.constant
#import _.std.color.constant
#import _.std.math.distance
#import _.std.math.matrix
#import _.std.math.vector
#import _.std.state.type
#import _.std.vertex.transform
#import _.std.vertex.type

const COLOR = CYAN;
const GLOW_FACTOR = 0.005;
const THICKNESS = 0.005;
const TRAIL_WIDTH = BALL_RADIUS * 2;

var<uniform> surface: Surface;

struct Fragment {
    @builtin(position)
    clip_position: vec4f,
    @location(0)
    world_position: vec2f,
    @location(1)
    ball_position: vec2f,
    @location(2)
    trail_position_1: vec2f,
    @location(3)
    trail_position_2: vec2f,
    @location(4)
    trail_position_3: vec2f,
}

@vertex
fn vs_main(vertex: Vertex, instance: Ball) -> Fragment {
    let scale_factor = scale_factor(surface.size, VISIBLE_AREA_MIN_SIZE);
    let position = vertex.position.xy * vec2f(3, 3);
    return Fragment(
        vec4f(position * scale_factor, instance.position.z, 1),
        position,
        instance.position.xy,
        instance.trail_position_1,
        instance.trail_position_2,
        instance.trail_position_3,
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
    return CYAN * max(exterior_brighness, max(interior_brighness, middle_brightness));
}

fn trail_color(fragment: Fragment) -> vec4f {
    const TRAIL_WIDTH = BALL_RADIUS * 2;
    var brightness = 0.;
    var accumulated_dist = 0.;
    if distance(fragment.world_position, fragment.ball_position) > BALL_RADIUS {
        brightness = trail_section_brightness(
            fragment.ball_position,
            fragment.trail_position_1,
            fragment.world_position,
            brightness,
            &accumulated_dist,
        );
        brightness = trail_section_brightness(
            fragment.trail_position_1,
            fragment.trail_position_2,
            fragment.world_position,
            brightness,
            &accumulated_dist,
        );
        brightness = trail_section_brightness(
            fragment.trail_position_2,
            fragment.trail_position_3,
            fragment.world_position,
            brightness,
            &accumulated_dist,
        );
    }
    return CYAN * brightness;
}

fn trail_section_brightness(
    bound1: vec2f,
    bound2: vec2f,
    world_position: vec2f,
    brightness: f32,
    accumulated_dist: ptr<function, f32>,
) -> f32 {
    if all(bound1 == UNSET_TRAIL_POSITION) || all(bound2 == UNSET_TRAIL_POSITION) {
        return brightness;
    }
    let width_dist = TRAIL_WIDTH - segment_signed_dist(world_position, bound1, bound2);
    let length_dist = *accumulated_dist + distance(world_position, bound1);
    let is_trail = width_dist >= 0 && width_dist < TRAIL_WIDTH;
    *accumulated_dist += distance(bound1, bound2);
    return max(brightness, select(0., 1., is_trail) * pow(width_dist / length_dist, 2));
}
