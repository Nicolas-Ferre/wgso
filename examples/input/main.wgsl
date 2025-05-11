#init init_rectangles()
#run update_rectangles()
#draw draw_rectangles<rectangles.vertices, rectangles.keyboard>()

#import _.std.vertex

struct Rectangles {
    vertices: array<Vertex, 6>,
    keyboard: KeyboardRect,
}

struct KeyboardRect {
    position: vec2f,
    color: vec4f,
}
