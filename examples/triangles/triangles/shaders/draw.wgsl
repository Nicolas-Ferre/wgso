#shader<render, Vertex> triangle

#import triangles.types

struct Fragment {
    @builtin(position)
    position: vec4f,
    @location(0)
    relative_position: vec4f,
};

var<uniform> instance: Triangle;

@vertex
fn vs_main(vertex: Vertex) -> Fragment {
    let position = vec4f(vertex.position + instance.position, 0., 1.);
    return Fragment(position, position);
}

@fragment
fn fs_main(frag: Fragment) -> @location(0) vec4f {
    let red = (frag.relative_position.x + 1) / 2;
    let green = (frag.relative_position.y + 1) / 2;
    let blue = red * green;
    let brightness = triangle_brightness(instance.brightness_param);
    return vec4f(red, green, blue, 1) * brightness;
}
