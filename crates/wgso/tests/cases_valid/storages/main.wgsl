#mod primitive

var<storage, read_write> u32_value: u32;
var<storage, read_write> i32_value: i32;
var<storage, read_write> f32_value: f32;
var<storage, read_write> vec3f_value: vec3f;
var<storage, read_write> vec3f32_value: vec3<f32>;

#mod array

var<storage, read_write> primitive_array_value: array<u32, 10>;
var<storage, read_write> struct_array_value: array<TestStruct, 10>;
var<storage, read_write> unsized_array_value: array<TestStruct>;

struct TestStruct {
    f32_field: f32,
}

#mod struct

var<storage, read_write> struct_value: TestStruct;

struct TestStruct {
    vec3_field: vec3f,
    vec2_field: vec2f,
    inner_struct_field: InnerStruct,
}

struct InnerStruct {
    f32_field: f32,
}
