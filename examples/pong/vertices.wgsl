#mod main
#init ~.init()

#mod storage
#import _.std.vertex.type

struct Vertices {
    rectangle: array<Vertex, 6>,
}

var<storage, read_write> vertices: Vertices;

#shader<compute> init
#import ~.storage
#import _.std.vertex.model

@compute
@workgroup_size(1, 1, 1)
fn main() {
    vertices = Vertices(rectangle_vertices());
}
