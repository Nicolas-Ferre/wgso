#import ~.main

var<storage, read_write> vertices: VertexState;

struct VertexState {
    triangle: array<Vertex, 3>,
    rectangle: array<Vertex, 6>,
}
