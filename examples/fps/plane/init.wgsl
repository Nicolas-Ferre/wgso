#mod<compute> init_plane

#import ~.storage
#import _.std.vertex

@compute
@workgroup_size(1, 1, 1)
fn main() {
    plane.vertices = rectangle_vertices();
    plane.instance.size = vec2f(10, 10);
    plane.instance.color = vec4f(1, 1, 1, 1);
}
