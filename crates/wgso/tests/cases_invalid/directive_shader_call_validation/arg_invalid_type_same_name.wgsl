#mod main
#init ~.compute(param=buffer_arg_invalid_type_same_name)
#run ~.compute(param=buffer_arg_invalid_type_same_name)
#draw ~.render<vertices, instances>(param=buffer_arg_invalid_type_same_name)

var<storage, read_write> buffer_arg_invalid_type_same_name: MyStruct;

struct MyStruct {
    field: u32,
}

#shader<compute> compute

var<uniform> param: MyStruct;

var<private> variable: u32;

struct MyStruct {
    field: i32,
}

#shader<render, u32, u32> render

var<uniform> param: MyStruct;

var<private> variable: u32;

struct MyStruct {
    field: i32,
}
