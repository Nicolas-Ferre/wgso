#shader<compute> init_triangles

#import triangles.storages

@compute
@workgroup_size(1, 1, 1)
fn main() {
    triangles.vertices = array(
        Vertex(vec2f(0., 0.5)),
        Vertex(vec2f(-0.5, -0.5)),
        Vertex(vec2f(0.5, -0.5)),
    );
    triangles.instance1 = Triangle(vec2f(0.25, -0.25), 3.14 / 4);
    triangles.instance2 = Triangle(vec2f(0., 0.), 3.14 / 8);
    triangles.instance3 = Triangle(vec2f(-0.25, 0.25), 0.);
}
