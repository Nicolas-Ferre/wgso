#run invalid_type_same_name(param=buffer_invalid_type_same_name)

var<storage, read_write> buffer_invalid_type_same_name: MyStruct;

struct MyStruct {
    field: u32,
}
