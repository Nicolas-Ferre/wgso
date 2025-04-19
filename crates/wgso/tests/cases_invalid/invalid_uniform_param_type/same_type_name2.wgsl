#shader<compute> test_same_type_name

var<uniform> param: MyStruct;

struct MyStruct {
    field: i32,
}
