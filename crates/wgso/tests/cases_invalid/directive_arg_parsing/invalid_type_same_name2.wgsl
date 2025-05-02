#shader<compute> invalid_type_same_name

var<uniform> param: MyStruct;

struct MyStruct {
    field: i32,
}
