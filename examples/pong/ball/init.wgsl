#shader<compute> init_ball

#import ~.storage
#import _.std.vertex

@compute
@workgroup_size(1, 1, 1)
fn main() {
    ball.vertices = rectangle_vertices();
    ball.instance.position = vec2f(-0.1, -0.1);
    ball.instance.velocity = vec2f(-0.5, 0.5)*2;
}
