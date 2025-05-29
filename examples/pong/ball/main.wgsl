#init init_ball()
#run update_ball()
#draw ball<ball.vertices, ball.instance>(global=global)

#import _.std.vertex

const BALL_RADIUS = 0.03;

struct BallData {
    vertices: array<Vertex, 6>,
    instance: Ball,
}

struct Ball {
    position: vec2f,
    velocity: vec2f,
}
