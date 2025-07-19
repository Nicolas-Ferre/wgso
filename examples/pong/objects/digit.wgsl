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

#mod compute
#import ~.main

const SEGMENT_HORIZONTAL_SIZE = vec2f(0.35, 0.035);
const SEGMENT_VERTICAL_SIZE = vec2f(0.035, 0.35);

fn init_digit(position: vec3f, height: f32, value: u32) -> Digit {
    var top = DigitSegment();
    top.position = vec3f(0, 0.5, 0.0009) * height + position;
    top.size = SEGMENT_HORIZONTAL_SIZE * height;
    top.is_enabled = u32(array(true, false, true, true, false, true, true, true, true, true)[value]);
    var middle = DigitSegment();
    middle.position = vec3f(0, 0, 0.0008) * height + position;
    middle.size = SEGMENT_HORIZONTAL_SIZE * height;
    middle.is_enabled = u32(array(false, false, true, true, true, true, true, false, true, true)[value]);
    var bottom = DigitSegment();
    bottom.position = vec3f(0, -0.5, 0.0007) * height + position;
    bottom.size = SEGMENT_HORIZONTAL_SIZE * height;
    bottom.is_enabled = u32(array(true, false, true, true, false, true, true, false, true, true)[value]);
    var top_left = DigitSegment();
    top_left.position = vec3f(-0.25, 0.25, 0.0006) * height + position;
    top_left.size = SEGMENT_VERTICAL_SIZE * height;
    top_left.is_enabled = u32(array(true, false, false, false, true, true, true, false, true, true)[value]);
    var top_right = DigitSegment();
    top_right.position = vec3f(0.25, 0.25, 0.0005) * height + position;
    top_right.size = SEGMENT_VERTICAL_SIZE * height;
    top_right.is_enabled = u32(array(true, true, true, true, true, false, false, true, true, true)[value]);
    var bottom_left = DigitSegment();
    bottom_left.position = vec3f(-0.25, -0.25, 0.0004) * height + position;
    bottom_left.size = SEGMENT_VERTICAL_SIZE * height;
    bottom_left.is_enabled = u32(array(true, false, true, false, false, false, true, false, true, false)[value]);
    var bottom_right = DigitSegment();
    bottom_right.position = vec3f(0.25, -0.25, 0.0003) * height + position;
    bottom_right.size = SEGMENT_VERTICAL_SIZE * height;
    bottom_right.is_enabled = u32(array(true, true, false, true, true, true, true, true, true, true)[value]);
    return Digit(array(
        top,
        middle,
        bottom,
        top_left,
        top_right,
        bottom_left,
        bottom_right,
    ));
}

#shader<render, Vertex, DigitSegment> render
#import ~.main
#import constant.main
#import _.std.math.distance
#import _.std.io.main
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
        select(DISABLED_COLOR, ENABLED_COLOR, bool(instance.is_enabled))
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    let corner_diameter = min(fragment.segment_size.x, fragment.segment_size.y);
    let dist = max(0, rect_signed_dist(fragment.world_position - fragment.segment_position, fragment.segment_size - corner_diameter) - corner_diameter);
    let brighness = step(0, dist) * exp(-dist / GLOW_FACTOR);
    return vec4f(fragment.color, 1.) * brighness;
}
