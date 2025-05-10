#shader<compute> init_background

#import ~.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    background.vertices = rectangle_vertices();
}
