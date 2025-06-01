#mod main
#init ~.init()
#draw ~.render<state.vertices, state.instance>()

struct Vertex {
    position: vec2f,
}

struct Instance {
    position: vec2f,
}

#mod storage
#import ~.main

var<storage, read_write> state: State;

struct State {
    vertices: array<Vertex, 6>,
    instance: Instance,
}

#shader<compute> init
#import ~.storage

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

#shader<render, Vertex, Instance> render
#import ~.main

struct Fragment {
    @builtin(position)
    position: vec4f,
}

@vertex
fn vs_main(vertex: Vertex, instance: Instance) -> Fragment {
    return Fragment(vec4f(vertex.position / 2 + instance.position, 0, 1));
}

@fragment
fn fs_main(fragement: Fragment) -> @location(0) vec4f {
    return vec4f(1, 1, 1, 1);
}
