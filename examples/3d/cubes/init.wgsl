#shader<compute> init_cubes

#import ~.storage
#import _.std.quaternion

@compute
@workgroup_size(1, 1, 1)
fn main() {
    cubes.vertices = cube_vertices();
}
