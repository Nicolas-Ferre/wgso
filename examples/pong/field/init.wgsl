#shader<compute> init_field

#import ~.storage
#import _.std.vertex

@compute
@workgroup_size(1, 1, 1)
fn main() {
    field.vertices = rectangle_vertices();
    field.instance.time = 0;
}
