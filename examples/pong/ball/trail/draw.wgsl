#shader<render, Vertex, BallTrailParticle> ball_trail

#import ~.main
#import global.main
#import _.std.vertex

const Z = 0.1;
const COLOR = vec3f(61, 194, 255) / 255. * 4;
const GLOW_FACTOR = 0.005;

var<uniform> global: Global;

struct Fragment {
    @builtin(position)
    clip_position: vec4f,
    @location(0)
    world_position: vec2f,
    @location(1)
    particle_position: vec2f,
    @location(2)
    particle_radius: f32,
}

@vertex
fn vs_main(
    vertex: Vertex,
    instance: BallTrailParticle,
    @builtin(instance_index) instance_index: u32,
) -> Fragment {
    if instance.radius < 0 {
        return Fragment(vec4f(-10, -10, -10, 1), vec2f(), vec2f(), 0);
    }
    let position = vertex.position.xy * BALL_TRAIL_PARTICLE_RADIUS * 4 + instance.position;
    return Fragment(
        vec4f(position * surface_ratio(global.surface_size), Z + 0.2 - f32(instance_index) / 100000, 1),
        position,
        instance.position,
        instance.radius,
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    let dist = circle_signed_dist(fragment.world_position - fragment.particle_position, fragment.particle_radius);
    let brighness = clamp(exp(-dist / GLOW_FACTOR), 0, 1);
    return vec4f(COLOR, 1) * brighness;
}

fn circle_signed_dist(position: vec2f, radius: f32) -> f32 {
    return length(position) - radius;
}
