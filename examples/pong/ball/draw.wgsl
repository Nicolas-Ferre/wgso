#shader<render, Vertex, Ball> ball

#import ~.main
#import surface.main
#import field.main
#import _.std.math
#import _.std.storage_types

const Z = 0.0;
const COLOR = vec3f(61, 194, 255) / 255.;
const GLOW_FACTOR = 0.005;
const THICKNESS = 0.015;

var<uniform> surface: SurfaceState;

struct Fragment {
    @builtin(position)
    clip_position: vec4f,
    @location(0)
    world_position: vec2f,
    @location(1)
    ball_position: vec2f,
}

@vertex
fn vs_main(vertex: Vertex, instance: Ball) -> Fragment {
    let position = vertex.position.xy * FIELD_SIZE;
    return Fragment(
        vec4f(position * surface_ratio(surface.size), Z, 1),
        position,
        instance.position,
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    let dist = circle_signed_dist(fragment.world_position - fragment.ball_position, BALL_RADIUS);
    let exterior_brighness = step(0, dist) * exp(-dist / GLOW_FACTOR);
    let interior_brighness = step(THICKNESS, -dist) * exp((dist + THICKNESS) / GLOW_FACTOR);
    let middle_brightness = step(-dist, THICKNESS) * step(dist, 0);
    return vec4f(COLOR, 1) * max(exterior_brighness, max(interior_brighness, middle_brightness));
}

fn circle_signed_dist(position: vec2f, radius: f32) -> f32 {
    return length(position) - radius;
}
