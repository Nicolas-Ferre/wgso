#init init_rectangles()
#run update_rectangles()
#draw draw_rectangles<rectangles.vertices, rectangles.keyboard>(ratio=rectangles.ratio)
#draw draw_rectangles<rectangles.vertices, rectangles.mouse>(ratio=rectangles.ratio)

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
