#shader<compute> init_triangles

#import ~.storages

@compute
@workgroup_size(1, 1, 1)
fn main() {
    triangles.instances = array(
        Triangle(vec2f(0.25, -0.25), 3.14 / 4),
        Triangle(vec2f(0., 0.), 3.14 / 8),
        Triangle(vec2f(-0.25, 0.25), 0.),
    );
}
