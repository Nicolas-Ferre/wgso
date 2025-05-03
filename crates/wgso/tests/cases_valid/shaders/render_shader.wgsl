#shader<render, Vertex> test_render

var<uniform> instance: Instance;

struct Vertex {
    position: vec2f,
}

struct Instance {
    position: vec2f,
    color: vec4f,
}

struct Fragment {
    @builtin(position)
    position: vec4f,
};

@vertex
fn vs_main(vertex: Vertex) -> Fragment {
    return Fragment(vec4f(vertex.position / 2. + instance.position, 0., 1.));
}

@fragment
fn fs_main(frag: Fragment) -> @location(0) vec4f {
    return instance.color;
}
