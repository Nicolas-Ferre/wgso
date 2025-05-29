#mod<compute> arg_duplicated
#mod<render, u32, u32> arg_duplicated

#init arg_duplicated(param1=buffer_arg_duplicated, param2=buffer_arg_duplicated, param1=buffer_arg_duplicated)
#run arg_duplicated(param1=buffer_arg_duplicated, param2=buffer_arg_duplicated, param1=buffer_arg_duplicated)
#draw arg_duplicated<vertices, instances>(param1=buffer_arg_duplicated, param2=buffer_arg_duplicated, param1=buffer_arg_duplicated)

var<storage, read_write> buffer_arg_duplicated: u32;

var<uniform> param1: u32;
var<uniform> param2: u32;
