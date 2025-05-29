#run update_ball_trail()
#draw<100> ball_trail<ball.vertices, ball_trail.particles>(global=global)

const MAX_BALL_TRAIL_PARTICLES = 2000;
const BALL_TRAIL_PARTICLE_RADIUS = 0.01;

struct BallTrail {
    particles: array<BallTrailParticle, MAX_BALL_TRAIL_PARTICLES>,
}

struct BallTrailParticle {
    position: vec2f,
    radius: f32,
}
