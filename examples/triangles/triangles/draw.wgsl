#shader<render, Vertex, Triangle> triangle

#import ~.main
#import vertices.main

struct Fragment {
    @builtin(position)
    position: vec4f,
    @location(0)
    relative_position: vec4f,
    @location(1)
    brightness: f32,
};

@vertex
fn vs_main(vertex: Vertex, instance: Triangle) -> Fragment {
    let position = vec4f(vertex.position + instance.position, 0., 1.);
    return Fragment(position, position, triangle_brightness(instance.brightness_param));
}

@fragment
fn fs_main(frag: Fragment) -> @location(0) vec4f {
    let red = (frag.relative_position.x + 1) / 2;
    let green = (frag.relative_position.y + 1) / 2;
    let blue = red * green;
    return vec4f(red, green, blue, 1) * frag.brightness;
}
