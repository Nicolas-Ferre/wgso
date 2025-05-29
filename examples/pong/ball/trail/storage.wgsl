#import ~.main

var<storage, read_write> ball_trail: BallTrail;

fn create_ball_trail_particle(position: vec2f) {
    let index = ball_trail_next_particle_id();
    ball_trail.particles[index].position = position;
    ball_trail.particles[index].radius = BALL_TRAIL_PARTICLE_RADIUS;
}

fn ball_trail_next_particle_id() -> u32 {
    for (var i = 0u; i < MAX_BALL_TRAIL_PARTICLES; i++) {
        if ball_trail.particles[i].radius <= 0. {
            return i;
        }
    }
    return 0;
}
