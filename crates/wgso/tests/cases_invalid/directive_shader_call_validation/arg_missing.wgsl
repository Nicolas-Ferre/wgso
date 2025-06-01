#mod main
#init ~.compute()
#run ~.compute()
#draw ~.render<vertices, instances>()

var<uniform> param: u32;

#shader<compute> compute
#import ~.main

#shader<render, u32, u32> render
#import ~.main
