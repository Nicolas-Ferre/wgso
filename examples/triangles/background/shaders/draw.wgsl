#shader<render, Vertex, Background> background

#import vertices.types
#import background.types

struct Fragment {
    @builtin(position)
    position: vec4f,
    @location(0)
    color: vec4f,
};

@vertex
fn vs_main(vertex: Vertex, instance: Background) -> Fragment {
    return Fragment(vec4f(vertex.position, 0.5, 1.), instance.color);
}

@fragment
fn fs_main(frag: Fragment) -> @location(0) vec4f {
    return frag.color;
}
