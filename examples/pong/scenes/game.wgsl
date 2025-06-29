#mod main
#init ~.init()
#run ~.update()
#run ~.update_particles()
#draw components.field.render<game.rectangle_vertices, game.field>(surface=std_.surface)
#draw components.digit.render<game.rectangle_vertices, game.left_score.segments>(surface=std_.surface)
#draw components.digit.render<game.rectangle_vertices, game.right_score.segments>(surface=std_.surface)
#draw components.paddle.render<game.rectangle_vertices, game.left_paddle>(surface=std_.surface)
#draw components.paddle.render<game.rectangle_vertices, game.right_paddle>(surface=std_.surface)
#draw components.ball.render<game.rectangle_vertices, game.ball>(surface=std_.surface)
#draw components.particle.render<game.rectangle_vertices, game.particles>(surface=std_.surface)

const FIELD_Z = 0.9;
const LEFT_SCORE_Z = 0.8;
const RIGHT_SCORE_Z = 0.7;
const PADDLE_Z = 0.6;
const BALL_Z = 0.5;
const PARTICLE_Z = 0.4;

#mod storage
#import components.ball.state
#import components.field.state
#import components.number.state
#import components.paddle.state
#import components.particle.state
#import _.std.vertex.type

const PARTICLE_COUNT_PER_COLLISION = 30;
const MAX_PARTICLE_COUNT = PARTICLE_COUNT_PER_COLLISION * 6;

struct Game {
    rectangle_vertices: array<Vertex, 6>,
    field: Field,
    ball: Ball,
    left_paddle: Paddle,
    right_paddle: Paddle,
    left_score: Number,
    right_score: Number,
    particles: array<Particle, MAX_PARTICLE_COUNT>,
    next_particle_index: u32,
}

var<storage, read_write> game: Game;

#shader<compute> init
#import ~.main
#import ~.storage
#import _.std.math.random
#import _.std.vertex.model
#import _.std.state.storage

const PADDLE_POSITION_X = 0.8;

@compute
@workgroup_size(1, 1, 1)
fn main() {
    var seed = std_.time.start_secs;
    let ball_direction = 2 * random_f32(&seed, 0, 1) - 1;
    game = Game(
        rectangle_vertices(),
        init_field(FIELD_Z),
        init_ball(BALL_Z, vec2f(ball_direction, 0)),
        init_paddle(-PADDLE_POSITION_X, PADDLE_Z),
        init_paddle(PADDLE_POSITION_X, PADDLE_Z),
        init_number(vec3f(), 0),
        init_number(vec3f(), 0),
        array<Particle, MAX_PARTICLE_COUNT>(),
        0,
    );
}

#shader<compute> update
#import ~.main
#import ~.storage
#import config.constant
#import _.std.physics.collision

const LEFT_SCORE_POSITION = vec3f(-0.2, 0.35, LEFT_SCORE_Z);
const RIGHT_SCORE_POSITION = vec3f(0.2, 0.35, RIGHT_SCORE_Z);

@compute
@workgroup_size(1, 1, 1)
fn main() {
    game.field = update_field(game.field);
    game.left_paddle = update_paddle(game.left_paddle, FIELD_SIZE.y / 2, KB_KEY_W, KB_KEY_S);
    game.right_paddle = update_paddle(game.right_paddle, FIELD_SIZE.y / 2, KB_ARROW_UP, KB_ARROW_DOWN);
    game.ball = update_ball(game.ball);
    check_ball_collisions();
    check_score_update();
    game.left_score = init_number(LEFT_SCORE_POSITION, game.left_score.value);
    game.right_score = init_number(RIGHT_SCORE_POSITION, game.right_score.value);
}

fn check_ball_collisions() {
    let left_paddle_collision = ball_paddle_collision(game.left_paddle);
    if left_paddle_collision.is_colliding {
        game.ball = apply_ball_paddle_collision(game.ball, game.left_paddle.position.xy, left_paddle_collision);
        create_particles(game.ball.position.xy, vec2f(-sign(game.ball.velocity.x), 0));
    }
    let right_paddle_collision = ball_paddle_collision(game.right_paddle);
    if right_paddle_collision.is_colliding {
        game.ball = apply_ball_paddle_collision(game.ball, game.right_paddle.position.xy, right_paddle_collision);
        create_particles(game.ball.position.xy, vec2f(-sign(game.ball.velocity.x), 0));
    }
    if is_ball_colliding_with_wall() {
        game.ball = apply_ball_wall_collision(game.ball, FIELD_SIZE.y / 2);
        create_particles(game.ball.position.xy, vec2f(0, -sign(game.ball.velocity.y)));
    }
}

fn check_score_update() {
    if game.ball.position.x - BALL_RADIUS < -FIELD_SIZE.x / 2 {
        create_particles(game.ball.position.xy, vec2f(0, 0));
        game.right_score.value += 1;
        game.ball = init_ball(BALL_Z, vec2f(-1, 0));
        game.left_paddle = reset_paddle(game.left_paddle);
        game.right_paddle = reset_paddle(game.right_paddle);
    } else if game.ball.position.x + BALL_RADIUS > FIELD_SIZE.x / 2 {
        create_particles(game.ball.position.xy, vec2f(0, 0));
        game.left_score.value += 1;
        game.ball = init_ball(BALL_Z, vec2f(1, 0));
        game.left_paddle = reset_paddle(game.left_paddle);
        game.right_paddle = reset_paddle(game.right_paddle);
    }
}

fn is_ball_colliding_with_wall() -> bool {
    return abs(game.ball.position.y) + BALL_RADIUS > FIELD_SIZE.y / 2;
}

fn ball_paddle_collision(paddle: Paddle) -> Collision {
    return _aabb_collision(
        paddle.position.xy, PADDLE_SIZE,
        game.ball.position.xy, vec2f(BALL_RADIUS * 2),
    );
}

fn create_particles(position: vec2f, normal: vec2f) {
    var seed = 0u;
    let normal_angle = angle_vec2f(normal, vec2f(0, 1));
    let min_angle = select(normal_angle - PI / 4, 0., all(normal == vec2f(0, 0)));
    let max_angle = select(normal_angle + PI / 4, 2. * PI, all(normal == vec2f(0, 0)));
    for (var i = game.next_particle_index; i < game.next_particle_index + PARTICLE_COUNT_PER_COLLISION; i++) {
        let z = PARTICLE_Z + 0.0001 * f32(MAX_PARTICLE_COUNT - i);
        game.particles[i] = init_particle(vec3f(position, z), min_angle, max_angle, &seed);
    }
    game.next_particle_index = (game.next_particle_index + PARTICLE_COUNT_PER_COLLISION) % MAX_PARTICLE_COUNT;
}

#shader<compute> update_particles
#import ~.storage

@compute
@workgroup_size(MAX_PARTICLE_COUNT, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let particle = &game.particles[global_id.x];
    *particle = update_particle(*particle);
}
