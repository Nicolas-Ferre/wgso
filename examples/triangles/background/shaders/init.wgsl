#shader<compute> init_background

#import background.storages

@compute
@workgroup_size(1, 1, 1)
fn main() {
    background.vertices = array(
        Vertex(vec2f(-1, -1)),
        Vertex(vec2f(-1, 1)),
        Vertex(vec2f(1, -1)),
        Vertex(vec2f(1, 1)),
        Vertex(vec2f(1, -1)),
        Vertex(vec2f(-1, 1)),
    );
}
