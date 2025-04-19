var<storage, read_write> arg: MyStruct;

struct MyStruct {
    field: u32,
}

#run test_same_type_name(param=arg)
