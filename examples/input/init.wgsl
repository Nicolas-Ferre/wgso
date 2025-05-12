#shader<compute> init_rectangles

#import ~.storage
#import _.std.color
#import _.std.vertex

@compute
@workgroup_size(1, 1, 1)
fn main() {
    rectangles.vertices = rectangle_vertices();
    rectangles.keyboard.position = vec2f(0, 0);
    rectangles.keyboard.color = WHITE;
}
