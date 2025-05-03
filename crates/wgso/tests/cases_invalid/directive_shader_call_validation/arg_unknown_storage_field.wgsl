#shader<compute> arg_unknown_storage_field
#shader<render, u32, u32> arg_unknown_storage_field

#init arg_unknown_storage_field(param=buffer.field)
#run arg_unknown_storage_field(param=buffer.field)
#draw arg_unknown_storage_field<vertices, instances>(param=buffer.field)

var<storage> buffer: u32;

var<uniform> param: u32;
