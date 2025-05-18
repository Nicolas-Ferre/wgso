#shader<render, Vertex, Rect> draw_rectangles

#import ~.main
#import _.std.vertex

const RECT_SIZE = vec2f(0.3, 0.3);

var<uniform> ratio: f32;

struct Fragment {
    @builtin(position)
    position: vec4f,
    @location(0)
    color: vec4f,
}

@vertex
fn vs_main(vertex: Vertex, instance: Rect) -> Fragment {
    let position = vertex.position.xy * RECT_SIZE + instance.position;
    let ratio = select(vec2f(1, ratio), vec2f(1 / ratio, 1), ratio > 1);
    return Fragment(
        vec4f(position * ratio, 0, 1),
        instance.color,
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4f {
    return fragment.color;
}
