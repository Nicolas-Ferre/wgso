#shader<render, u32, u32> render
#draw ~.render<invalid_vertices, invalid_instances>()

var<storage, read_write> invalid_vertices: u32;
var<storage, read_write> invalid_instances: u32;
