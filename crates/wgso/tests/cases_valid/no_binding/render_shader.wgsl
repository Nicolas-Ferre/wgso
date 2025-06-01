#shader<render, Vertex, Instance> test_render
#draw ~.test_render<vertices, instances>()

var<storage, read> vertices: array<Vertex, 3>;
var<storage, read> instances: array<Instance, 1>;

struct Vertex {
    position: vec2f,
}

struct Instance {
    position: vec2f,
}

struct Fragment {
    @builtin(position)
    position: vec4f,
};

@vertex
fn vs_main(vertex: Vertex) -> Fragment {
    return Fragment(vec4f(vertex.position / 2., 0., 1.));
}

@fragment
fn fs_main(frag: Fragment) -> @location(0) vec4f {
    discard;
}
