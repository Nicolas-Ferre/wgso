#mod main
#import ~.main
#import objects.ball.compute
#import objects.field.compute
#import objects.number.compute
#import objects.paddle.compute
#import objects.particle.compute

#init ~.init()
#run ~.update()
#run ~.update_particles()
#draw objects.field.render<vertices.rectangle, game.field>(surface=std_.surface)
#draw objects.digit.render<vertices.rectangle, game.left_score.segments>(surface=std_.surface)
#draw objects.digit.render<vertices.rectangle, game.right_score.segments>(surface=std_.surface)
#draw objects.paddle.render<vertices.rectangle, game.left_paddle>(surface=std_.surface)
#draw objects.paddle.render<vertices.rectangle, game.right_paddle>(surface=std_.surface)
#draw objects.ball.render<vertices.rectangle, game.ball>(surface=std_.surface)
#draw objects.particle.render<vertices.rectangle, game.particles>(surface=std_.surface)

const FIELD_Z = 0.9;
const LEFT_SCORE_Z = 0.8;
const RIGHT_SCORE_Z = 0.7;
const PADDLE_Z = 0.6;
const BALL_Z = 0.5;
const PARTICLE_Z = 0.4;

const PARTICLE_COUNT_PER_COLLISION = 30;
const MAX_PARTICLE_COUNT = PARTICLE_COUNT_PER_COLLISION * 6;
const LEFT_SCORE_POSITION = vec2f(-0.2, 0.35);
const RIGHT_SCORE_POSITION = vec2f(0.2, 0.35);

struct Game {
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
#import _.std.math.random
#import _.std.io.compute

const PADDLE_POSITION_X = 0.8;

@compute
@workgroup_size(1, 1, 1)
fn main() {
    var seed = std_.time.start_secs;
    let ball_direction = 2 * random_f32(&seed, 0, 1) - 1;
    game.field = init_field(FIELD_Z);
    game.ball = init_ball(BALL_Z, vec2f(ball_direction, 0));
    game.left_paddle = init_paddle(vec3f(-PADDLE_POSITION_X, 0, PADDLE_Z));
    game.right_paddle = init_paddle(vec3f(PADDLE_POSITION_X, 0, PADDLE_Z));
    game.left_score = init_number(vec3f(LEFT_SCORE_POSITION, LEFT_SCORE_Z), 0);
    game.right_score = init_number(vec3f(RIGHT_SCORE_POSITION, RIGHT_SCORE_Z), 0);
    game.next_particle_index = 0;
}

#shader<compute> update
#import ~.main
#import constant.main
#import scenes.orchestrator.main
#import _.std.physics.collision

@compute
@workgroup_size(1, 1, 1)
fn main() {
    game.field = update_field(game.field);
    update_paddles();
    game.ball = update_ball(game.ball);
    check_ball_collisions();
    check_score_update();
    game.left_score = init_number(vec3f(LEFT_SCORE_POSITION, LEFT_SCORE_Z), game.left_score.value);
    game.right_score = init_number(vec3f(RIGHT_SCORE_POSITION, RIGHT_SCORE_Z), game.right_score.value);
}

fn update_paddles() {
    game.left_paddle = update_player_paddle(game.left_paddle, FIELD_SIZE.y / 2, KB_KEY_W, KB_KEY_S, F32_MIN, 0);
    if bool(orchestrator.is_multiplayer) {
        game.right_paddle = update_player_paddle(game.right_paddle, FIELD_SIZE.y / 2, KB_ARROW_UP, KB_ARROW_DOWN, 0, F32_MAX);
    } else {
        game.right_paddle = update_bot_paddle(game.right_paddle, FIELD_SIZE.y / 2, game.ball.position.y);
    }
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
    return aabb_collision(
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
#import ~.main

@compute
@workgroup_size(MAX_PARTICLE_COUNT, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let particle = &game.particles[global_id.x];
    *particle = update_particle(*particle);
}
