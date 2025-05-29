#mod<compute> init_triangles

#import ~.storage

@compute
@workgroup_size(1, 1, 1)
fn main() {
    triangles.vertices = array(
        Vertex(vec3f(0., 0.5, 0), vec3f(0, 0, 1)),
        Vertex(vec3f(-0.5, -0.5, 0), vec3f(0, 0, 1)),
        Vertex(vec3f(0.5, -0.5, 0), vec3f(0, 0, 1)),
    );
    triangles.instances = array(
        Triangle(vec2f(0.25, -0.25), 3.14 / 4),
        Triangle(vec2f(0., 0.), 3.14 / 8),
        Triangle(vec2f(-0.25, 0.25), 0.),
    );
}
