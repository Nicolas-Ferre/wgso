#mod state
#import ~.instance
#import _.std.vertex.model

#toggle<is_toggle_enabled> toggle
#run ~.update()

var<storage, read_write> state: u32;
var<storage, read_write> is_toggle_enabled: u32;
var<storage, read_write> vertices: array<Vertex, 6>;
var<storage, read_write> instance: Instance;

#mod instance

struct Instance {
    _phantom: u32,
}

#shader<compute> update
#import ~.state

@compute
@workgroup_size(1, 1, 1)
fn main() {
    is_toggle_enabled = u32(!bool(is_toggle_enabled));
    vertices = rectangle_vertices();
}
