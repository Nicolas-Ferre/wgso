#mod main
#init ~.init()
#import _.std.vertex.type

struct Vertices {
    rectangle: array<Vertex, 6>,
}

#mod storage
#import ~.main

var<storage, read_write> vertices: Vertices;

#shader<compute> init
#import ~.storage
#import _.std.vertex.model

@compute
@workgroup_size(1, 1, 1)
fn main() {
    vertices.rectangle = rectangle_vertices();
}
