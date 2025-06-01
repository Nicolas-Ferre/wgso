#mod main
#init ~.compute(param1=buffer_arg_duplicated, param2=buffer_arg_duplicated, param1=buffer_arg_duplicated)
#run ~.compute(param1=buffer_arg_duplicated, param2=buffer_arg_duplicated, param1=buffer_arg_duplicated)
#draw ~.render<vertices, instances>(param1=buffer_arg_duplicated, param2=buffer_arg_duplicated, param1=buffer_arg_duplicated)

var<storage, read_write> buffer_arg_duplicated: u32;

var<uniform> param1: u32;
var<uniform> param2: u32;

#shader<compute> compute
#import ~.main

#shader<render, u32, u32> render
#import ~.main
