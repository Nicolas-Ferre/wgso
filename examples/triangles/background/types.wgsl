#import vertices.types

struct BackgroundState {
    instance: Background,
    vertices: array<Vertex, 6>,
}

struct Background {
    color: vec4f,
}
