#init ~.arg_invalid_type_same_name2.arg_invalid_type_same_name(param=buffer_arg_invalid_type_same_name)
#run ~.arg_invalid_type_same_name2.arg_invalid_type_same_name(param=buffer_arg_invalid_type_same_name)
#draw ~.arg_invalid_type_same_name2.arg_invalid_type_same_name<vertices, instances>(param=buffer_arg_invalid_type_same_name)

var<storage, read_write> buffer_arg_invalid_type_same_name: MyStruct;

struct MyStruct {
    field: u32,
}
