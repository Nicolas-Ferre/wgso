#mod<render, u32, u32> buffer_non_array

#draw buffer_non_array<invalid_vertices, invalid_instances>()

var<storage, read_write> invalid_vertices: u32;
var<storage, read_write> invalid_instances: u32;
