#shader<render, Vertex, Field> field

#import ~.main
#import global.main
#import _.std.math
#import _.std.storage_types

const Z = 0.9;
const SHAPE_FACTOR = 2.;
const BORDER_GLOW_FACTOR = 0.002;
const BORDER_COLOR_ROTATION_SPEED = PI / 2;
const BORDER_THICKNESS = 0.02;
const SEPARATOR_COLOR = vec3f(1, 1, 1);
const SEPARATOR_HEIGHT = 0.9;
const SEPARATOR_THICKNESS = 0.0025;
const SEPARATOR_GLOW_FACTOR = 0.001;

var<uniform> global: Global;

struct Fragment {
    @builtin(position)
    clip_position: vec4f,
    @location(0)
    world_position: vec2f,
    @location(1)
    time: f32,
}

@vertex
fn vs_main(vertex: Vertex, instance: Field) -> Fragment {
    let position = vertex.position.xy * FIELD_SIZE * SHAPE_FACTOR;
    return Fragment(
        vec4f(position * surface_ratio(global.surface_size), Z, 1),
        position,
        global.elapsed_secs,
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    return vec4f(border_color(fragment) + separator_color(fragment), 1.);
}

fn border_color(fragment: Fragment) -> vec3f {
    let angle = vec2_angle(fragment.world_position, vec2f(1.));
    let rotated_angle = angle + fragment.time * BORDER_COLOR_ROTATION_SPEED;
    let dist = abs(rect_signed_dist(fragment.world_position, FIELD_SIZE + vec2f(BORDER_THICKNESS)));
    let brightness = brightness(dist, BORDER_THICKNESS, BORDER_GLOW_FACTOR);
    let color = color(rotated_angle);
    return brightness * color;
}

fn separator_color(fragment: Fragment) -> vec3f {
    const HALF_SIZE = FIELD_SIZE.y * SEPARATOR_HEIGHT / 2;
    let dist = segment_signed_dist(fragment.world_position, vec2f(0, -HALF_SIZE), vec2f(0, HALF_SIZE));
    return brightness(dist, SEPARATOR_THICKNESS, SEPARATOR_GLOW_FACTOR) * SEPARATOR_COLOR;
}

fn brightness(signed_dist: f32, thickness: f32, glow_factor: f32) -> f32 {
    let exterior_factor = clamp(glow_factor / (signed_dist - thickness / 2), 0, 1);
    let interior_factor = step(signed_dist, thickness / 2);
    return exterior_factor + interior_factor;
}

fn color(angle: f32) -> vec3f {
    const a = vec3f(0.50, 0.50, 0.50);
    const b = vec3f(0.50, 0.50, 0.50);
    const c = vec3f(1.00, 1.00, 1.00);
    const d = vec3f(0.00, 0.33, 0.67);
    let normalized_angle = angle / (2. * PI);
    return a + b * cos(6.283185 * (c * normalized_angle + d));
}

fn rect_signed_dist(frag_position: vec2f, size: vec2f) -> f32 {
    let distance = abs(frag_position) - size / 2;
    let exterior_dist = length(max(distance, vec2f(0.0)));
    let interior_dist = min(max(distance.x, distance.y), 0.0);
    return exterior_dist + interior_dist;
}

fn segment_signed_dist(frag_position: vec2f, vertex1: vec2f, vertex2: vec2f) -> f32 {
    let distance1 = frag_position - vertex1;
    let distance2 = vertex2 - vertex1;
    let factor = clamp(dot(distance1, distance2) / dot(distance2, distance2), 0., 1.);
    return length(distance1 - distance2 * factor);
}
