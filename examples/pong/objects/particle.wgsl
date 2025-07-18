#mod main

struct Particle {
    position: vec3f,
    velocity: vec2f,
    brightness: f32,
}

#mod compute
#import ~.main
#import _.std.math.matrix
#import _.std.math.quaternion
#import _.std.math.random
#import _.std.io.compute

const PARTICLE_BRIGHTNESS_REDUCTION_SPEED = 2.;
const PARTICLE_DECELERATION = 2;
const PARTICLE_MIN_SPEED = 0.1;
const PARTICLE_MAX_SPEED = 1.0;

fn init_particle(position: vec3f, min_angle: f32, max_angle: f32, seed: ptr<function, u32>) -> Particle {
    let angle = random_f32(seed, min_angle, max_angle);
    let speed = random_f32(seed, PARTICLE_MIN_SPEED, PARTICLE_MAX_SPEED);
    let velocity = rotation_mat(quat(vec3f(0, 0, 1), angle)) * vec4f(0, speed, 0, 1);
    return Particle(position, velocity.xy, 1.);
}

fn update_particle(particle: Particle) -> Particle {
    var updated = particle;
    updated.position += vec3f(updated.velocity * std_.time.frame_delta_secs, 0);
    updated.velocity -= PARTICLE_DECELERATION * updated.velocity * std_.time.frame_delta_secs;
    updated.brightness -= PARTICLE_BRIGHTNESS_REDUCTION_SPEED * std_.time.frame_delta_secs;
    updated.brightness = max(updated.brightness, 0);
    return updated;
}

#shader<render, Vertex, Particle> render
#import ~.main
#import constant.main
#import _.std.color.constant
#import _.std.io.main
#import _.std.vertex.transform
#import _.std.vertex.type

const RADIUS = 0.01;
const COLOR = CYAN;

var<uniform> surface: Surface;

struct Fragment {
    @builtin(position)
    clip_position: vec4f,
    @location(0)
    world_position: vec2f,
    @location(1)
    particle_position: vec2f,
    @location(2)
    particle_brightness: f32,
}

@vertex
fn vs_main(vertex: Vertex, instance: Particle) -> Fragment {
    let scale_factor = scale_factor(surface.size, VISIBLE_AREA_MIN_SIZE);
    if instance.brightness <= 0 {
        return Fragment(vec4f(-10, -10, 0, 1), vec2f(), vec2f(), 0);
    }
    let position = vertex.position.xy * RADIUS * 2 + instance.position.xy;
    return Fragment(
        vec4f(position * scale_factor, instance.position.z, 1),
        position,
        instance.position.xy,
        instance.brightness,
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    let dist = distance(fragment.particle_position, fragment.world_position);
    if dist > RADIUS {
        discard;
    }
    let distance_brightness = 1 - dist / RADIUS;
    return COLOR * distance_brightness * fragment.particle_brightness;
}
