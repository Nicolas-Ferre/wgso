#import ~.main

var<storage, read_write> state: State;

struct State {
    vertices: array<Vertex, 6>,
    instance: Instance,
}
