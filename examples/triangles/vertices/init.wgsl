#shader<compute> init_vertices

#import ~.storages

@compute
@workgroup_size(1, 1, 1)
fn main() {
    vertices.triangle = array(
        Vertex(vec2f(0., 0.5)),
        Vertex(vec2f(-0.5, -0.5)),
        Vertex(vec2f(0.5, -0.5)),
    );
    vertices.rectangle = array(
        Vertex(vec2f(-1, -1)),
        Vertex(vec2f(-1, 1)),
        Vertex(vec2f(1, -1)),
        Vertex(vec2f(1, 1)),
        Vertex(vec2f(1, -1)),
        Vertex(vec2f(-1, 1)),
    );
}
