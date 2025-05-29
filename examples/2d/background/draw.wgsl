#mod<render, Vertex, BackgroundInstance> background

#import ~.main
#import _.std.vertex

struct Fragment {
    @builtin(position)
    position: vec4f,
    @location(0)
    color: vec4f,
};

@vertex
fn vs_main(vertex: Vertex, instance: BackgroundInstance) -> Fragment {
    return Fragment(vec4f(vertex.position.xy * 2, 0.5, 1.), instance.color);
}

@fragment
fn fs_main(frag: Fragment) -> @location(0) vec4f {
    return frag.color;
}
