#mod main

const SEGMENT_COUNT_PER_DIGIT = 7;

struct Digit {
    segments: array<DigitSegment, SEGMENT_COUNT_PER_DIGIT>,
}

struct DigitSegment {
    position: vec3f,
    size: vec2f,
    is_enabled: u32,
}

#mod state
#import ~.main

const _SEGMENT_HORIZONTAL_SIZE = vec2f(0.35, 0.035);
const _SEGMENT_VERTICAL_SIZE = vec2f(0.035, 0.35);

fn init_digit(position: vec3f, height: f32, value: u32) -> Digit {
    var digit = Digit();
    digit.segments[0].position = vec3f(0, -0.5, 0.0009) * height + position;
    digit.segments[1].position = vec3f(0, 0, 0.0008) * height + position;
    digit.segments[2].position = vec3f(-0.25, -0.25, 0.0007) * height + position;
    digit.segments[3].position = vec3f(0.25, -0.25, 0.0006) * height + position;
    digit.segments[4].position = vec3f(0, 0.5, 0.0005) * height + position;
    digit.segments[5].position = vec3f(-0.25, 0.25, 0.0004) * height + position;
    digit.segments[6].position = vec3f(0.25, 0.25, 0.0003) * height + position;
    digit.segments[0].size = _SEGMENT_HORIZONTAL_SIZE * height;
    digit.segments[1].size = _SEGMENT_HORIZONTAL_SIZE * height;
    digit.segments[2].size = _SEGMENT_VERTICAL_SIZE * height;
    digit.segments[3].size = _SEGMENT_VERTICAL_SIZE * height;
    digit.segments[4].size = _SEGMENT_HORIZONTAL_SIZE * height;
    digit.segments[5].size = _SEGMENT_VERTICAL_SIZE * height;
    digit.segments[6].size = _SEGMENT_VERTICAL_SIZE * height;
    digit.segments[0].is_enabled =
        u32(value == 0 || value == 2 || value == 3 || value == 5 || value == 6 || value == 8 || value == 9);
    digit.segments[1].is_enabled =
        u32(value == 2 || value == 3 || value == 4 || value == 5 || value == 6 || value == 8 || value == 9);
    digit.segments[2].is_enabled =
        u32(value == 0 || value == 2 || value == 6 || value == 8);
    digit.segments[3].is_enabled =
        u32(value == 0 || value == 1 || value == 3 || value == 4 || value == 5 || value == 6 || value == 7 || value == 8 || value == 9);
    digit.segments[4].is_enabled =
        u32(value == 0 || value == 2 || value == 3 || value == 5 || value == 6 || value == 7 || value == 8 || value == 9);
    digit.segments[5].is_enabled =
        u32(value == 0 || value == 4 || value == 5 || value == 6 || value == 8 || value == 9);
    digit.segments[6].is_enabled =
        u32(value == 0 || value == 1 || value == 2 || value == 3 || value == 4 || value == 7 || value == 8 || value == 9);
    return digit;
}

#shader<render, Vertex, DigitSegment> render
#import ~.main
#import config.constant
#import _.std.math.distance
#import _.std.state.type
#import _.std.vertex.transform
#import _.std.vertex.type

const ENABLED_COLOR = vec3f(1., 0.1, 0.1);
const DISABLED_COLOR = ENABLED_COLOR * 0.03;
const GLOW_FACTOR = 0.003;

var<uniform> surface: Surface;

struct Fragment {
    @builtin(position)
    clip_position: vec4f,
    @location(0)
    world_position: vec2f,
    @location(1)
    segment_position: vec2f,
    @location(2)
    segment_size: vec2f,
    @location(3)
    color: vec3f,
}

@vertex
fn vs_main(vertex: Vertex, instance: DigitSegment) -> Fragment {
    let scale_factor = scale_factor(surface.size, VISIBLE_AREA_MIN_SIZE);
    let position = vertex.position.xy * instance.size * 10 + instance.position.xy;
    return Fragment(
        vec4f(position * scale_factor, instance.position.z, 1),
        position,
        instance.position.xy,
        instance.size,
        select(DISABLED_COLOR, ENABLED_COLOR, instance.is_enabled == 1)
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    let corner_diameter = min(fragment.segment_size.x, fragment.segment_size.y);
    let dist = max(0, rect_signed_dist(fragment.world_position - fragment.segment_position, fragment.segment_size - corner_diameter) - corner_diameter);
    let brighness = step(0, dist) * exp(-dist / GLOW_FACTOR);
    return vec4f(fragment.color, 1.) * brighness;
}
