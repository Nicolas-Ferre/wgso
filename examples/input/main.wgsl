#init ~.init.init_rectangles()
#run ~.update.update_rectangles()
#draw ~.draw.draw_rectangles<rectangles.vertices, rectangles.keyboard>(ratio=rectangles.ratio)
#draw ~.draw.draw_rectangles<rectangles.vertices, rectangles.mouse>(ratio=rectangles.ratio)

#import _.std.vertex

struct Rectangles {
    ratio: f32,
    vertices: array<Vertex, 6>,
    keyboard: Rect,
    mouse: Rect,
}

struct Rect {
    position: vec2f,
    color: vec4f,
}

fn ratio_2d(surface_ratio: f32) -> vec2f {
    return select(vec2f(1, surface_ratio), vec2f(1 / surface_ratio, 1), surface_ratio > 1);
}
