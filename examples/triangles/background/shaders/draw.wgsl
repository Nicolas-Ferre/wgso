#shader<render, Vertex> background

#import background.types

struct Fragment {
    @builtin(position)
    position: vec4f,
};

var<uniform> instance: Background;

@vertex
fn vs_main(vertex: Vertex) -> Fragment {
    return Fragment(vec4f(vertex.position, 0.5, 1.));
}

@fragment
fn fs_main(frag: Fragment) -> @location(0) vec4f {
    return instance.color;
}
