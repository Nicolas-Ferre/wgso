#shader<compute> init

#import ~.storages

@compute
@workgroup_size(1, 1, 1)
fn main() {
    state.vertices = array(
        Vertex(vec2f(-1, -1)),
        Vertex(vec2f(-1, 1)),
        Vertex(vec2f(1, -1)),
        Vertex(vec2f(1, 1)),
        Vertex(vec2f(1, -1)),
        Vertex(vec2f(-1, 1)),
    );
    state.instance = Instance(vec2f(0, 0));
}
