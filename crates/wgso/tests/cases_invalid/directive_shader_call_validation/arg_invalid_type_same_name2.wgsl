#mod<compute> arg_invalid_type_same_name
#mod<render, u32, u32> arg_invalid_type_same_name

var<uniform> param: MyStruct;

var<private> variable: u32;

struct MyStruct {
    field: i32,
}
