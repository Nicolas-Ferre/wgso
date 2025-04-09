var<storage, read_write> primitive_array_value: array<u32, 10>;
var<storage, read_write> struct_array_value: array<TestStruct, 10>;

struct TestStruct {
    f32_field: f32,
}
