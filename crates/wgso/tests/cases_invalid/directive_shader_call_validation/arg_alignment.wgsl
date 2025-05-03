#shader<compute> arg_alignment
#shader<render, u32, u32> arg_alignment

#init arg_alignment(param=buffer_arg_alignment.field2)
#run arg_alignment(param=buffer_arg_alignment.field2)
#draw arg_alignment<vertices, instances>(param=buffer_arg_alignment.field2)

var<storage> buffer_arg_alignment: TestStruct;

var<uniform> param: u32;

struct TestStruct {
    field1: u32,
    field2: u32,
}
