#shader<render, Vertex, Instance> rectangle

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
