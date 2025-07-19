#mod main
#import _.std.vertex.type

#init ~.init()

struct Vertices {
    rectangle: array<Vertex, 6>,
}

var<storage, read_write> vertices: Vertices;

#shader<compute> init
#import ~.main
#import _.std.vertex.model

@compute
@workgroup_size(1, 1, 1)
fn main() {
    vertices.rectangle = rectangle_vertices();
}
