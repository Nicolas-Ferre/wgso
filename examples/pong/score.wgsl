#mod main
#init ~.init()
#run ~.update()
#draw<900> ~.render_segment<vertices.rectangle, scores.segments>(surface=surface)

const SEGMENT_COUNT_PER_DIGIT = 7;
const DIGIT_COUNT_PER_SCORE = 2;
const SEGMENT_COUNT = SEGMENT_COUNT_PER_DIGIT * DIGIT_COUNT_PER_SCORE * 2;
const SCORE_ZOOM_FACTOR = 0.5;

struct Scores {
    left: u32,
    right: u32,
    segments: array<ScoreSegment, SEGMENT_COUNT>
}

struct ScoreSegment {
    position: vec2f,
    size: vec2f,
    is_enabled: u32,
}

#mod storage
#import ~.main

var<storage, read_write> scores: Scores;

#shader<compute> init
#import ~.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    init_digit(SEGMENT_COUNT_PER_DIGIT * 0, vec2f(-0.25, 0.35));
    init_digit(SEGMENT_COUNT_PER_DIGIT * 1, vec2f(-0.15, 0.35));
    init_digit(SEGMENT_COUNT_PER_DIGIT * 2, vec2f(0.15, 0.35));
    init_digit(SEGMENT_COUNT_PER_DIGIT * 3, vec2f(0.25, 0.35));
}

fn init_digit(first_index: u32, position: vec2f) {
    scores.segments[first_index + 0].position = vec2f(0, -0.15) * SCORE_ZOOM_FACTOR + position;
    scores.segments[first_index + 0].size = vec2f(0.1, 0.01) * SCORE_ZOOM_FACTOR;
    scores.segments[first_index + 1].position = vec2f(0, 0) * SCORE_ZOOM_FACTOR + position;
    scores.segments[first_index + 1].size = vec2f(0.1, 0.01) * SCORE_ZOOM_FACTOR;
    scores.segments[first_index + 2].position = vec2f(-0.07, -0.075) * SCORE_ZOOM_FACTOR + position;
    scores.segments[first_index + 2].size = vec2f(0.01, 0.1) * SCORE_ZOOM_FACTOR;
    scores.segments[first_index + 3].position = vec2f(0.07, -0.075) * SCORE_ZOOM_FACTOR + position;
    scores.segments[first_index + 3].size = vec2f(0.01, 0.1) * SCORE_ZOOM_FACTOR;
    scores.segments[first_index + 4].position = vec2f(0, 0.15) * SCORE_ZOOM_FACTOR + position;
    scores.segments[first_index + 4].size = vec2f(0.1, 0.01) * SCORE_ZOOM_FACTOR;
    scores.segments[first_index + 5].position = vec2f(-0.07, 0.075) * SCORE_ZOOM_FACTOR + position;
    scores.segments[first_index + 5].size = vec2f(0.01, 0.1) * SCORE_ZOOM_FACTOR;
    scores.segments[first_index + 6].position = vec2f(0.07, 0.075) * SCORE_ZOOM_FACTOR + position;
    scores.segments[first_index + 6].size = vec2f(0.01, 0.1) * SCORE_ZOOM_FACTOR;
}

#shader<compute> update
#import ~.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    update_digit(SEGMENT_COUNT_PER_DIGIT * 0, scores.left / 10 % 10);
    update_digit(SEGMENT_COUNT_PER_DIGIT * 1, scores.left % 10);
    update_digit(SEGMENT_COUNT_PER_DIGIT * 2, scores.right / 10 % 10);
    update_digit(SEGMENT_COUNT_PER_DIGIT * 3, scores.right % 10);
}

fn update_digit(first_index: u32, value: u32) {
    scores.segments[first_index + 0].is_enabled =
        u32(value == 0 || value == 2 || value == 3 || value == 5 || value == 6 || value == 8 || value == 9);
    scores.segments[first_index + 1].is_enabled =
        u32(value == 2 || value == 3 || value == 4 || value == 5 || value == 6 || value == 8 || value == 9);
    scores.segments[first_index + 2].is_enabled =
        u32(value == 0 || value == 2 || value == 6 || value == 8);
    scores.segments[first_index + 3].is_enabled =
        u32(value == 0 || value == 1 || value == 3 || value == 4 || value == 5 || value == 6 || value == 7 || value == 8 || value == 9);
    scores.segments[first_index + 4].is_enabled =
        u32(value == 0 || value == 2 || value == 3 || value == 5 || value == 6 || value == 7 || value == 8 || value == 9);
    scores.segments[first_index + 5].is_enabled =
        u32(value == 0 || value == 4 || value == 5 || value == 6 || value == 8 || value == 9);
    scores.segments[first_index + 6].is_enabled =
        u32(value == 0 || value == 1 || value == 2 || value == 3 || value == 4 || value == 7 || value == 8 || value == 9);
}

#shader<render, Vertex, ScoreSegment> render_segment
#import ~.main
#import surface.main
#import _.std.vertex.type

const MIN_Z = 0.7;
const MAX_Z = 0.8;
const ENABLED_COLOR = vec3f(1., 0.1, 0.1);
const DISABLED_COLOR = ENABLED_COLOR * 0.03;
const GLOW_FACTOR = 0.003;

var<uniform> surface: SurfaceData;

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
fn vs_main(
    vertex: Vertex,
    instance: ScoreSegment,
    @builtin(instance_index) instance_index: u32
) -> Fragment {
    let position = vertex.position.xy * instance.size * 10 + instance.position;
    let z = (1 - f32(instance_index) / f32(SEGMENT_COUNT)) * (MAX_Z - MIN_Z) + MIN_Z;
    return Fragment(
        vec4f(position * surface_ratio(surface.size), z, 1),
        position,
        instance.position,
        instance.size,
        select(DISABLED_COLOR, ENABLED_COLOR, instance.is_enabled == 1)
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    return segment_color(fragment);
}

fn segment_color(fragment: Fragment) -> vec4f {
    let corner_diameter = min(fragment.segment_size.x, fragment.segment_size.y);
    let dist = max(0, rect_signed_dist(fragment.world_position - fragment.segment_position, fragment.segment_size - corner_diameter) - corner_diameter);
    let brighness = step(0, dist) * exp(-dist / GLOW_FACTOR);
    return vec4f(fragment.color, 1.) * brighness;
}

fn rect_signed_dist(frag_position: vec2f, size: vec2f) -> f32 {
    let distance = abs(frag_position) - size / 2;
    let exterior_dist = length(max(distance, vec2f(0.0)));
    let interior_dist = min(max(distance.x, distance.y), 0.0);
    return exterior_dist + interior_dist;
}
