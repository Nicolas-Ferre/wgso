#shader<compute> unknown_storage_field
#run unknown_storage_field(param=buffer.field)

var<storage> buffer: u32;

var<uniform> param: u32;
