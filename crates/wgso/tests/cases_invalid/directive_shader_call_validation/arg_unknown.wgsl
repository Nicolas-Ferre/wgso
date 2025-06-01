#mod main
#init ~.compute(arg_unknown_param=buffer_arg_unknown)
#run ~.compute(arg_unknown_param=buffer_arg_unknown)
#draw ~.render<vertices, instances>(arg_unknown_param=buffer_arg_unknown)

var<storage, read_write> buffer_arg_unknown: u32;

#shader<compute> compute
#import ~.main

#shader<render, u32, u32> render
#import ~.main
