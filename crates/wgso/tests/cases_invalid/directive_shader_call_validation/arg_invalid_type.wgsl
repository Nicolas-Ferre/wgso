#mod main
#init ~.compute(value=buffer_arg_invalid_type)
#run ~.compute(value=buffer_arg_invalid_type)
#draw ~.render<vertices, instances>(value=buffer_arg_invalid_type)

var<storage, read_write> buffer_arg_invalid_type: i32;

var<uniform> value: u32;

#shader<compute> compute
#import ~.main

#shader<render, u32, u32> render
#import ~.main
