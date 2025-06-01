#mod main
#init ~.compute(param=buffer_arg_alignment.field2)
#run ~.compute(param=buffer_arg_alignment.field2)
#draw ~.render<vertices, instances>(param=buffer_arg_alignment.field2)

var<storage> buffer_arg_alignment: TestStruct;

var<uniform> param: u32;

struct TestStruct {
    field1: u32,
    field2: u32,
}

#shader<compute> compute
#import ~.main

#shader<render, u32, u32> render
#import ~.main
