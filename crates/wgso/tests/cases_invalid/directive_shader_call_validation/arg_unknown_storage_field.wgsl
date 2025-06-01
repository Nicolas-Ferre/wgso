#mod main
#init ~.compute(param=buffer.field)
#run ~.compute(param=buffer.field)
#draw ~.render<vertices, instances>(param=buffer.field)

var<storage> buffer: u32;

var<uniform> param: u32;

#shader<compute> compute
#import ~.main

#shader<render, u32, u32> render
#import ~.main
