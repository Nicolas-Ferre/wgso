#mod main
#run ~.update()
#draw<0> ~.render<vertices.rectangle, collision_effects.instances>(surface=std_.surface)

const COLLISION_MAX_PARTICLE_COUNT = 180;
const PARTICLE_COUNT_PER_COLLISION = 30;

struct CollisionParticles {
    instances: array<CollisionParticle, COLLISION_MAX_PARTICLE_COUNT>,
}

struct CollisionParticle {
    position: vec2f,
    velocity: vec2f,
    brightness: f32,
}

#mod storage
#import ~.main
#import _.std.math.constant
#import _.std.math.matrix
#import _.std.math.random
#import _.std.math.vector

var<storage, read_write> collision_effects: CollisionParticles;

fn add_collision_particles(position: vec2f, normal: vec2f) {
    let first_index = next_collision_particle_index();
    let normal_angle = angle_vec2f(vec2f(0, 1), normal);
    var seed = 0u;
    let min_angle = select(normal_angle - PI / 4, 0., all(normal == vec2f(0, 0)));
    let max_angle = select(normal_angle + PI / 4, 2. * PI, all(normal == vec2f(0, 0)));
    for (var i = first_index; i < first_index + PARTICLE_COUNT_PER_COLLISION; i++) {
        let angle = random_f32(&seed, min_angle, max_angle);
        let speed = random_f32(&seed, 0.1, 1.);
        let velocity = rotation_mat(quat(vec3f(0, 0, 1), angle)) * vec4f(0, speed, 0, 1);
        collision_effects.instances[i] = CollisionParticle(position, velocity.xy, 1.);
    }
}

fn next_collision_particle_index() -> u32 {
    for (var i = 0u; i < COLLISION_MAX_PARTICLE_COUNT; i++) {
        if collision_effects.instances[i].brightness <= 0 {
            return i;
        }
    }
    return 0;
}

#shader<compute, 10> update
#import ~.storage
#import _.std.state.storage

const THREAD_COUNT = COLLISION_MAX_PARTICLE_COUNT / 10;
const BRIGHTNESS_REDUCTION_SPEED = 2.;
const DECELERATION = 2;

@compute
@workgroup_size(THREAD_COUNT, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let particle = &collision_effects.instances[global_id.x];
    particle.position += particle.velocity * std_.time.frame_delta_secs;
    particle.velocity -= DECELERATION * particle.velocity * std_.time.frame_delta_secs;
    particle.brightness -= BRIGHTNESS_REDUCTION_SPEED * std_.time.frame_delta_secs;
    particle.brightness = max(particle.brightness, 0);
}

#shader<render, Vertex, CollisionParticle> render
#import ~.main
#import ball.main.main
#import surface.main
#import _.std.color.constant
#import _.std.state.type
#import _.std.vertex.type

const MIN_Z = 0.;
const MAX_Z = 0.1;
const RADIUS = 0.01;

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
fn vs_main(
    vertex: Vertex,
    instance: CollisionParticle,
    @builtin(instance_index) instance_index: u32,
) -> Fragment {
    if instance.brightness <= 0 {
        return Fragment(vec4f(-10, -10, 0, 1), vec2f(), vec2f(), 0);
    }
    let position = vertex.position.xy * RADIUS * 2 + instance.position;
    let z = (1 - f32(instance_index) / f32(COLLISION_MAX_PARTICLE_COUNT)) * (MAX_Z - MIN_Z) + MIN_Z;
    return Fragment(
        vec4f(position * surface_ratio(surface.size), z, 1),
        position,
        instance.position,
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
    return vec4f(BALL_COLOR, 1) * distance_brightness * fragment.particle_brightness;
}
