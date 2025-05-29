#mod<render, Vertex, Instance> test_render

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
    @location(0)
    color: vec4f,
};

@vertex
fn vs_main(vertex: Vertex, instance: Instance) -> Fragment {
    return Fragment(vec4f(vertex.position / 2. + instance.position, 0., 1.), instance.color);
}

@fragment
fn fs_main(frag: Fragment) -> @location(0) vec4f {
    return frag.color;
}
