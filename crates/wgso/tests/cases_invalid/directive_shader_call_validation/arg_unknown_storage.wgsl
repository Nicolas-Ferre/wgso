#mod main
#init ~.compute(param=arg_unknown_storage)
#run ~.compute(param=arg_unknown_storage)
#draw ~.render<vertices, instances>(param=arg_unknown_storage)

var<uniform> param: u32;

#shader<compute> compute
#import ~.main

#shader<render, u32, u32> render
#import ~.main
