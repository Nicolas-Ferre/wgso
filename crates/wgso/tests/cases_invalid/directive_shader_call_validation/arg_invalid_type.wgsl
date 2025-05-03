#shader<compute> arg_invalid_type
#shader<render, u32, u32> arg_invalid_type

#init arg_invalid_type(value=buffer_arg_invalid_type)
#run arg_invalid_type(value=buffer_arg_invalid_type)
#draw arg_invalid_type<vertices, instances>(value=buffer_arg_invalid_type)

var<storage, read_write> buffer_arg_invalid_type: i32;

var<uniform> value: u32;
